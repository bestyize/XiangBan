// Official spec checked 2026-10-28: 2026-07-28 Streamable HTTP plus legacy
// 2025-11-25 initialization. JSON responses only; no sessions or SSE are needed.
use crate::{
    audio, openai, security,
    server::{permit, ApiError, App, OPERATION_TIMEOUT},
    Capability, ClientInfo, VoiceError,
};
use axum::{
    extract::{rejection::JsonRejection, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};

const MODERN: &str = "2026-07-28";
const VERSIONS: [&str; 4] = [MODERN, "2025-11-25", "2025-06-18", "2025-03-26"];
const VERSION_META: &str = "io.modelcontextprotocol/protocolVersion";
fn rpc_error(
    status: StatusCode,
    id: Value,
    code: i32,
    message: &str,
    data: Option<Value>,
) -> Response {
    let mut value = json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}});
    if let Some(data) = data {
        value["error"]["data"] = data;
    }
    (status, Json(value)).into_response()
}
fn result(id: Value, mut value: Value, modern: bool) -> Response {
    if modern {
        value["resultType"] = json!("complete");
    }
    Json(json!({"jsonrpc":"2.0","id":id,"result":value})).into_response()
}
fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    (headers.get_all(name).iter().count() == 1)
        .then(|| headers.get(name)?.to_str().ok())
        .flatten()
}
fn decoded_header(headers: &HeaderMap, name: &str) -> Option<String> {
    let value = header_text(headers, name)?;
    if let Some(encoded) = value
        .strip_prefix("=?base64?")
        .and_then(|s| s.strip_suffix("?="))
    {
        String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .ok()?,
        )
        .ok()
    } else {
        Some(value.to_owned())
    }
}

pub(crate) async fn handle(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    payload: Result<Json<Value>, JsonRejection>,
) -> Response {
    let accept = header_text(&headers, header::ACCEPT.as_str()).unwrap_or("");
    let accepts = |kind: &str| {
        accept.split(',').any(|s| {
            s.split(';').next().unwrap_or("").trim() == kind
                && !s.split(';').any(|part| part.trim() == "q=0")
        })
    };
    if !accepts("application/json") || !accepts("text/event-stream") {
        return rpc_error(
            StatusCode::NOT_ACCEPTABLE,
            Value::Null,
            -32600,
            "Accept must include application/json and text/event-stream",
            None,
        );
    }
    let Json(body) = match payload {
        Ok(body) => body,
        Err(_) => {
            return rpc_error(
                StatusCode::BAD_REQUEST,
                Value::Null,
                -32700,
                "invalid JSON-RPC JSON",
                None,
            )
        }
    };
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let notification = body.get("id").is_none();
    let Some(method) = body.get("method").and_then(Value::as_str) else {
        return rpc_error(
            StatusCode::BAD_REQUEST,
            id,
            -32600,
            "invalid JSON-RPC request",
            None,
        );
    };
    if body["jsonrpc"] != "2.0"
        || !body.is_object()
        || !(notification || id.is_string() || id.is_i64() || id.is_u64())
        || body.get("params").is_some_and(|p| !p.is_object())
        || body.get("result").is_some()
        || body.get("error").is_some()
    {
        return rpc_error(
            StatusCode::BAD_REQUEST,
            Value::Null,
            -32600,
            "invalid JSON-RPC request",
            None,
        );
    }
    let params = body.get("params").cloned().unwrap_or_else(|| json!({}));
    let version = header_text(&headers, "mcp-protocol-version");
    let meta = &params["_meta"];
    let modern = version == Some(MODERN) || meta.get(VERSION_META).is_some();
    if modern {
        let requested = meta
            .get(VERSION_META)
            .and_then(Value::as_str)
            .or(version)
            .unwrap_or("");
        if requested != MODERN {
            return rpc_error(
                StatusCode::BAD_REQUEST,
                id,
                -32022,
                "Unsupported protocol version",
                Some(json!({"supported":VERSIONS,"requested":requested})),
            );
        }
        if version != Some(MODERN)
            || meta.get(VERSION_META).and_then(Value::as_str) != Some(MODERN)
            || decoded_header(&headers, "mcp-method").as_deref() != Some(method)
            || (method == "tools/call"
                && decoded_header(&headers, "mcp-name").as_deref() != params["name"].as_str())
        {
            return rpc_error(StatusCode::BAD_REQUEST, id, -32020, "HeaderMismatch", None);
        }
        if !meta["io.modelcontextprotocol/clientInfo"]["name"].is_string()
            || !meta["io.modelcontextprotocol/clientInfo"]["version"].is_string()
            || !meta["io.modelcontextprotocol/clientCapabilities"].is_object()
        {
            return rpc_error(
                StatusCode::BAD_REQUEST,
                id,
                -32602,
                "required client metadata missing",
                None,
            );
        }
    } else if version.is_some_and(|v| !VERSIONS[1..].contains(&v)) {
        return rpc_error(
            StatusCode::BAD_REQUEST,
            id,
            -32600,
            "unsupported MCP-Protocol-Version",
            None,
        );
    }
    if notification {
        // Notifications never invoke devices and never carry a JSON-RPC response.
        return if method.starts_with("notifications/") {
            StatusCode::ACCEPTED.into_response()
        } else {
            StatusCode::BAD_REQUEST.into_response()
        };
    }
    match method {
        "initialize" if !modern => {
            let Some(requested) = params["protocolVersion"].as_str() else {
                return rpc_error(StatusCode::OK, id, -32602, "protocolVersion required", None);
            };
            if !params["capabilities"].is_object()
                || !params["clientInfo"]["name"].is_string()
                || !params["clientInfo"]["version"].is_string()
            {
                return rpc_error(
                    StatusCode::OK,
                    id,
                    -32602,
                    "capabilities and clientInfo required",
                    None,
                );
            }
            let selected = if VERSIONS[1..].contains(&requested) {
                requested
            } else {
                VERSIONS[1]
            };
            result(
                id,
                json!({"protocolVersion":selected,"capabilities":{"tools":{}},"serverInfo":{"name":"xiaoban-voice-service","version":env!("CARGO_PKG_VERSION")}}),
                false,
            )
        }
        "server/discover" if modern => result(
            id,
            json!({"supportedVersions":VERSIONS,"capabilities":{"tools":{}},"_meta":{"io.modelcontextprotocol/serverInfo":{"name":"xiaoban-voice-service","version":env!("CARGO_PKG_VERSION")}}}),
            true,
        ),
        "ping" => result(id, json!({}), modern),
        "tools/list" => {
            if params.get("cursor").is_some() {
                return rpc_error(
                    StatusCode::OK,
                    id,
                    -32602,
                    "no pagination cursor supported",
                    None,
                );
            }
            result(id, json!({"tools":tools(&app)}), modern)
        }
        "tools/call" => {
            let Some(name) = params["name"].as_str() else {
                return rpc_error(StatusCode::OK, id, -32602, "tool name required", None);
            };
            if !tools(&app).iter().any(|tool| tool["name"] == name) {
                return rpc_error(
                    StatusCode::OK,
                    id,
                    -32602,
                    "unknown or unavailable tool",
                    None,
                );
            }
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let args = match arguments(name, &args) {
                Ok(args) => args,
                Err(message) => return rpc_error(StatusCode::OK, id, -32602, message, None),
            };
            let client = security::client(&headers);
            match tokio::time::timeout(OPERATION_TIMEOUT, call(app, client, name, args)).await {
                Ok(Ok(value)) => result(
                    id,
                    json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false}),
                    modern,
                ),
                Ok(Err(error)) if error.status == StatusCode::FORBIDDEN => error.into_response(),
                failure => {
                    let busy = matches!(&failure, Ok(Err(error)) if error.status == StatusCode::TOO_MANY_REQUESTS);
                    let message = match failure {
                        Ok(Err(error)) if error.status == StatusCode::TOO_MANY_REQUESTS => {
                            if matches!(name, "listen" | "converse") {
                                "麦克风正被占用"
                            } else {
                                "扬声器正被占用"
                            }
                        }
                        Ok(Err(_)) => "语音后端不可用或调用失败",
                        _ => "语音工具调用超时",
                    };
                    let mut response = result(
                        id,
                        json!({"content":[{"type":"text","text":message}],"isError":true}),
                        modern,
                    );
                    if busy {
                        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                        response
                            .headers_mut()
                            .insert(header::RETRY_AFTER, "2".parse().unwrap());
                    }
                    response
                }
            }
        }
        _ => rpc_error(
            if modern {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::OK
            },
            id,
            -32601,
            "Method not found",
            None,
        ),
    }
}

fn tools(app: &App) -> Vec<Value> {
    let mut tools = Vec::new();
    for name in ["speak", "listen", "converse", "notify"] {
        if (matches!(name, "speak" | "converse" | "notify") && app.backends.speaker.is_none())
            || (matches!(name, "listen" | "converse") && app.backends.microphone.is_none())
        {
            continue;
        }
        let mut properties = json!({});
        let text = matches!(name, "speak" | "converse" | "notify");
        if text {
            properties["text"] = json!({"type":"string","minLength":1,"maxLength":4000});
        }
        if matches!(name, "speak" | "converse") {
            properties["voice"] = json!({"type":"string"});
        }
        if matches!(name, "listen" | "converse") {
            properties["max_seconds"] =
                json!({"type":"integer","minimum":1,"maximum":30,"default":15});
        }
        tools.push(json!({"name":name,"description":match name {"speak"=>"朗读文字，播放完返回", "listen"=>"收听一句话并识别", "converse"=>"朗读后收听回答", _=>"后台播放提示，立即返回"},"inputSchema":{"type":"object","properties":properties,"required":if text { vec!["text"] } else { vec![] },"additionalProperties":false}}));
    }
    tools
}
struct Args {
    text: String,
    voice: Option<String>,
    seconds: u32,
}
fn arguments(name: &str, value: &Value) -> Result<Args, &'static str> {
    let object = value.as_object().ok_or("arguments must be an object")?;
    let text = if name != "listen" {
        let text = value["text"].as_str().ok_or("text required")?;
        openai::validate_text(text).map_err(|_| "text must contain 1..4000 characters")?;
        text.to_owned()
    } else {
        String::new()
    };
    let voice = value
        .get("voice")
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or("voice must be a string")
        })
        .transpose()?;
    let seconds = value
        .get("max_seconds")
        .map(|v| {
            v.as_u64()
                .filter(|v| (1..=30).contains(v))
                .ok_or("max_seconds must be 1..30")
        })
        .transpose()?
        .unwrap_or(15) as u32;
    for key in object.keys() {
        if !(key == "text" && name != "listen"
            || key == "voice" && matches!(name, "speak" | "converse")
            || key == "max_seconds" && matches!(name, "listen" | "converse"))
        {
            return Err("unknown tool argument");
        }
    }
    Ok(Args {
        text,
        voice,
        seconds,
    })
}
async fn playback(app: &App, args: &Args) -> Result<(), ApiError> {
    let audio = openai::synthesize(app, &args.text, args.voice.as_deref(), None).await?;
    app.backends
        .speaker
        .as_ref()
        .ok_or_else(|| VoiceError::Unavailable("no speaker".into()))?
        .play(audio)
        .await?;
    Ok(())
}
async fn call(
    app: Arc<App>,
    client: ClientInfo,
    name: &str,
    args: Args,
) -> Result<Value, ApiError> {
    // Reserve resources before prompting. try_acquire never waits behind another call.
    let _mic = if matches!(name, "listen" | "converse") {
        Some(permit(&app.listening)?)
    } else {
        None
    };
    let _asr = if matches!(name, "listen" | "converse") {
        Some(permit(&app.transcription)?)
    } else {
        None
    };
    let speaker = if name != "listen" {
        Some(permit(&app.speaking)?)
    } else {
        None
    };
    let capabilities: &[Capability] = match name {
        "listen" => &[Capability::Listen, Capability::Transcribe],
        "converse" => &[
            Capability::Synthesize,
            Capability::Speak,
            Capability::Listen,
            Capability::Transcribe,
        ],
        _ => &[Capability::Synthesize, Capability::Speak],
    };
    app.approve(&client, capabilities).await?;
    if name == "notify" {
        let background_app = app.clone();
        let mut tasks = app.background.lock().unwrap();
        while tasks.try_join_next().is_some() {}
        tasks.spawn(async move {
            let _speaker = speaker;
            let _ = tokio::time::timeout(OPERATION_TIMEOUT, playback(&background_app, &args)).await;
        });
        return Ok(json!({"queued":true}));
    }
    let _speaker = speaker;
    if name != "listen" {
        playback(&app, &args).await?;
    }
    if matches!(name, "listen" | "converse") {
        let pcm = tokio::time::timeout(
            Duration::from_secs(args.seconds as u64 + 5),
            app.backends
                .microphone
                .as_ref()
                .ok_or_else(|| VoiceError::Unavailable("no microphone".into()))?
                .listen(args.seconds),
        )
        .await
        .map_err(|_| VoiceError::Unavailable("microphone timed out".into()))??;
        audio::validate_pcm(&pcm)?;
        if pcm.samples.len() > args.seconds as usize * audio::SAMPLE_RATE {
            return Err(
                VoiceError::InvalidInput("microphone exceeded requested duration".into()).into(),
            );
        }
        let silent = pcm.samples.len() < audio::SAMPLE_RATE / 10
            || pcm.samples.iter().all(|s| s.abs() < 0.0001);
        let text = if silent {
            String::new()
        } else {
            app.transcribe(pcm, None).await?
        };
        return Ok(if name == "converse" {
            json!({"spoken":true,"text":text})
        } else {
            json!({"text":text})
        });
    }
    Ok(json!({"spoken":true}))
}

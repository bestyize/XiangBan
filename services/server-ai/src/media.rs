//! 图片生成、语音识别与实时语音。三者都经 New API 转发到后台按产品配置的模型：客户端不传也看不到真实模型名，
//! 鉴权和额度检查与对话接口相同，用量由 New API 按模型倍率从钱包扣，对账时结算进额度桶。

use crate::{
    binding,
    error::AppError,
    extract::AuthUser,
    products::{self, Capability, MediaSetting},
    proxy::{error_response, map_body_error, map_upstream_error, require_quota, Api},
    quota,
    state::AppState,
};
use axum::{
    extract::{
        multipart::{Multipart, MultipartRejection},
        rejection::BytesRejection,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio_tungstenite::tungstenite::{self, client::IntoClientRequest};

fn unavailable(capability: Capability) -> AppError {
    let code = match capability {
        Capability::Speech => "speech_unavailable",
        Capability::Image => "image_unavailable",
        Capability::Transcribe => "transcribe_unavailable",
        Capability::Realtime => "realtime_unavailable",
    };
    AppError::Coded { status: StatusCode::SERVICE_UNAVAILABLE, code, message: format!("小伴 AI {}暂未开放", capability.label()) }
}

/// What the caller's product set up for this capability in the admin; none, or switched off, means not offered.
pub(crate) async fn setting(state: &AppState, auth: &AuthUser, capability: Capability) -> Result<MediaSetting, AppError> {
    products::media(&state.pool, &auth.product.product_id, capability).await?.ok_or_else(|| unavailable(capability))
}

/// The caller's binding, once their login and quota check out, and their paid quota if the capability needs it.
pub(crate) async fn caller(state: &AppState, auth: AuthUser, setting: &MediaSetting) -> Result<binding::ActiveBinding, AppError> {
    let active = binding::ensure_active(state, auth.uid, &auth.product).await?;
    require_quota(state, active.id).await?;
    if setting.requires_paid && quota::paid_total(&state.pool, active.id).await? <= 0 {
        return Err(AppError::Coded { status: StatusCode::FORBIDDEN, code: "paid_required", message: "这项功能需要先兑换额度才能使用".into() });
    }
    sqlx::query("UPDATE bindings SET last_used_at = now() WHERE id = $1").bind(active.id).execute(&state.pool).await?;
    Ok(active)
}

// ---- 图片生成 ----

pub const IMAGE_MAX_PROMPT_CHARS: usize = 4000;
pub const IMAGE_MAX_COUNT: u64 = 4;
/// The largest reply relayed back (base64 images are big).
const IMAGE_MAX_REPLY: usize = 48 * 1024 * 1024;

/// OpenAI-style image request for the configured model. The client asks for a prompt, how many and one of the
/// offered sizes; nothing else reaches the upstream.
pub fn image_payload(model: &str, sizes: &[String], body: &[u8]) -> Result<Value, AppError> {
    let request: Value = serde_json::from_slice(body).map_err(|_| AppError::BadRequest("请求体不是有效的 JSON".into()))?;
    let prompt = request.get("prompt").and_then(Value::as_str).map(str::trim).unwrap_or_default();
    if prompt.is_empty() || prompt.chars().count() > IMAGE_MAX_PROMPT_CHARS {
        return Err(AppError::BadRequest(format!("prompt 需要 1 到 {IMAGE_MAX_PROMPT_CHARS} 个字")));
    }
    let n = match request.get("n") {
        None | Some(Value::Null) => 1,
        Some(v) => v.as_u64().filter(|n| (1..=IMAGE_MAX_COUNT).contains(n)).ok_or_else(|| AppError::BadRequest(format!("n 需要 1 到 {IMAGE_MAX_COUNT}")))?,
    };
    let size = match request.get("size").and_then(Value::as_str) {
        Some(asked) => sizes.iter().find(|s| *s == asked).cloned().ok_or_else(|| AppError::BadRequest(format!("size 只能是 {}", sizes.join("、"))))?,
        None => sizes.first().cloned().unwrap_or_else(|| "1024x1024".into()),
    };
    Ok(json!({ "model": model, "prompt": prompt, "n": n, "size": size }))
}

/// Keeps only the images: no model, usage or provider fields reach the client.
pub fn image_reply(value: &Value) -> Option<Value> {
    let data: Vec<Value> = value.get("data")?.as_array()?.iter().filter_map(|item| {
        let mut out = serde_json::Map::new();
        for key in ["b64_json", "url", "revised_prompt"] {
            if let Some(v) = item.get(key).and_then(Value::as_str).filter(|s| !s.is_empty()) { out.insert(key.into(), json!(v)); }
        }
        (out.contains_key("b64_json") || out.contains_key("url")).then_some(Value::Object(out))
    }).collect();
    if data.is_empty() { return None; }
    Some(json!({ "created": value.get("created").and_then(Value::as_i64).unwrap_or_default(), "data": data }))
}

/// `POST /v1/images/generations`.
pub async fn image_generations(State(state): State<AppState>, auth: Result<AuthUser, AppError>, body: Result<Bytes, BytesRejection>) -> Response {
    let result = async {
        let body = body.map_err(|error| map_body_error(error.status()))?;
        let auth = auth?;
        let setting = setting(&state, &auth, Capability::Image).await?;
        let payload = image_payload(&setting.model, &setting.sizes, &body)?;
        let active = caller(&state, auth, &setting).await?;
        let payload = serde_json::to_vec(&payload).map_err(|e| AppError::Internal(e.to_string()))?;
        let upstream = state.newapi.relay("/v1/images/generations", &active.key, payload, &[]).await?;
        let status = upstream.status();
        let reply = upstream.bytes().await.map_err(|_| AppError::BadGateway("图片生成暂时不可用".into()))?;
        if !status.is_success() {
            let text = String::from_utf8_lossy(&reply);
            tracing::warn!("图片上游返回 {status}: {}", text.chars().take(300).collect::<String>());
            return Err(map_upstream_error(status, &text));
        }
        if reply.len() > IMAGE_MAX_REPLY { return Err(AppError::BadGateway("生成的图片太大".into())); }
        let value: Value = serde_json::from_slice(&reply).map_err(|_| AppError::BadGateway("图片生成返回异常".into()))?;
        let images = image_reply(&value).ok_or_else(|| AppError::BadGateway("图片生成没有返回图片".into()))?;
        Ok::<_, AppError>(Json(images).into_response())
    }
    .await;
    result.unwrap_or_else(|e| error_response(Api::ChatCompletions, e))
}

// ---- 语音识别 ----

/// The largest recording one request may carry (OpenAI's limit); its route lets bodies this big through.
pub const TRANSCRIBE_MAX_FILE: usize = 25 * 1024 * 1024;
pub const TRANSCRIBE_BODY_LIMIT: usize = TRANSCRIBE_MAX_FILE + 1024 * 1024;
const TRANSCRIBE_MAX_PROMPT_CHARS: usize = 1000;

/// What the client uploaded, checked.
#[derive(Debug, Default)]
pub struct Recording {
    pub file: Vec<u8>,
    pub file_name: String,
    pub content_type: String,
    pub language: Option<String>,
    pub prompt: Option<String>,
}

fn audio_type(content_type: &str) -> bool {
    content_type.starts_with("audio/") || content_type.starts_with("video/") || content_type == "application/octet-stream"
}

async fn read_recording(mut multipart: Multipart) -> Result<Recording, AppError> {
    let mut recording = Recording::default();
    let bad = |message: &str| AppError::BadRequest(message.into());
    while let Some(field) = multipart.next_field().await.map_err(|e| if e.status() == StatusCode::PAYLOAD_TOO_LARGE { map_body_error(e.status()) } else { bad("上传的内容不完整") })? {
        match field.name().unwrap_or_default() {
            "file" => {
                let name: String = field.file_name().unwrap_or("audio").chars().filter(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_')).take(100).collect();
                let content_type = field.content_type().unwrap_or("application/octet-stream").to_ascii_lowercase();
                if !audio_type(&content_type) { return Err(bad("只能上传音频文件")); }
                let data = field.bytes().await.map_err(|e| if e.status() == StatusCode::PAYLOAD_TOO_LARGE { map_body_error(e.status()) } else { bad("上传的内容不完整") })?;
                if data.len() > TRANSCRIBE_MAX_FILE { return Err(map_body_error(StatusCode::PAYLOAD_TOO_LARGE)); }
                recording.file = data.to_vec();
                recording.file_name = if name.is_empty() { "audio".into() } else { name };
                recording.content_type = content_type;
            }
            "language" => {
                let text = field.text().await.map_err(|_| bad("language 读取失败"))?;
                let text = text.trim();
                if !text.is_empty() {
                    if text.len() > 16 || !text.chars().all(|c| c.is_ascii_alphabetic() || c == '-') { return Err(bad("language 需要是语言代码，如 zh、en")); }
                    recording.language = Some(text.to_ascii_lowercase());
                }
            }
            "prompt" => {
                let text = field.text().await.map_err(|_| bad("prompt 读取失败"))?;
                let text = text.trim();
                if !text.is_empty() { recording.prompt = Some(text.chars().take(TRANSCRIBE_MAX_PROMPT_CHARS).collect()); }
            }
            // `model` and anything else are the gateway's to decide.
            _ => {}
        }
    }
    if recording.file.is_empty() { return Err(bad("请上传要识别的音频 file")); }
    Ok(recording)
}

/// `POST /v1/audio/transcriptions`: `multipart/form-data` with `file`, and optionally `language` and `prompt`.
/// Returns `{ text }`.
pub async fn transcriptions(State(state): State<AppState>, auth: Result<AuthUser, AppError>, multipart: Result<Multipart, MultipartRejection>) -> Response {
    let result = async {
        let auth = auth?;
        let setting = setting(&state, &auth, Capability::Transcribe).await?;
        let multipart = multipart.map_err(|_| AppError::BadRequest("请用 multipart/form-data 上传音频".into()))?;
        let recording = read_recording(multipart).await?;
        let active = caller(&state, auth, &setting).await?;
        let part = reqwest::multipart::Part::bytes(recording.file).file_name(recording.file_name).mime_str(&recording.content_type)
            .map_err(|_| AppError::BadRequest("音频类型无效".into()))?;
        let mut form = reqwest::multipart::Form::new().text("model", setting.model).text("response_format", "json").part("file", part);
        if let Some(language) = recording.language { form = form.text("language", language); }
        if let Some(prompt) = recording.prompt { form = form.text("prompt", prompt); }
        let upstream = state.newapi.relay_multipart("/v1/audio/transcriptions", &active.key, form).await?;
        let status = upstream.status();
        let text = upstream.text().await.unwrap_or_default();
        if !status.is_success() {
            tracing::warn!("识别上游返回 {status}: {}", text.chars().take(300).collect::<String>());
            return Err(map_upstream_error(status, &text));
        }
        let value: Value = serde_json::from_str(&text).map_err(|_| AppError::BadGateway("语音识别返回异常".into()))?;
        let heard = value.get("text").and_then(Value::as_str).ok_or_else(|| AppError::BadGateway("语音识别没有返回文字".into()))?;
        Ok::<_, AppError>(Json(json!({ "text": heard.trim() })).into_response())
    }
    .await;
    result.unwrap_or_else(|e| error_response(Api::ChatCompletions, e))
}

// ---- 实时语音 ----

/// What the client sees in place of the real model's name.
pub const REALTIME_LABEL: &str = "realtime";
/// Browsers cannot set headers on a WebSocket, so they may send the login token as a subprotocol instead.
pub const TOKEN_PROTOCOL: &str = "xiaoban-token.";
/// How often the caller's quota is looked at again during a conversation.
const QUOTA_RECHECK: Duration = Duration::from_secs(60);
/// The largest single event either side may send (audio arrives in small chunks).
const REALTIME_MAX_MESSAGE: usize = 4 * 1024 * 1024;

/// A client event on its way up: only JSON objects pass, and the model stays the gateway's choice.
pub fn client_event(text: &str) -> Option<String> {
    let mut value: Value = serde_json::from_str(text).ok()?;
    let object = value.as_object_mut()?;
    if let Some(session) = object.get_mut("session").and_then(Value::as_object_mut) { session.remove("model"); }
    if let Some(response) = object.get_mut("response").and_then(Value::as_object_mut) { response.remove("model"); }
    Some(value.to_string())
}

/// An upstream event on its way down, with the model's name replaced.
pub fn upstream_event(text: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(text) else { return text.to_string() };
    for key in ["session", "response"] {
        if let Some(model) = value.get_mut(key).and_then(|v| v.get_mut("model")) { *model = json!(REALTIME_LABEL); }
    }
    value.to_string()
}

fn realtime_token(headers: &HeaderMap) -> Option<String> {
    let bearer = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")).map(str::trim).filter(|s| !s.is_empty());
    if let Some(token) = bearer { return Some(token.to_string()); }
    let protocols = headers.get(header::SEC_WEBSOCKET_PROTOCOL).and_then(|v| v.to_str().ok())?;
    protocols.split(',').map(str::trim).find_map(|p| p.strip_prefix(TOKEN_PROTOCOL)).filter(|s| !s.is_empty()).map(str::to_string)
}

/// `GET /v1/realtime` (WebSocket): OpenAI Realtime events, relayed to the configured model through New API.
/// The login token comes in `Authorization: Bearer …` or, from a browser, as the subprotocol `xiaoban-token.<token>`
/// alongside `realtime`. One conversation lasts at most the product's configured time and ends when the quota runs out.
pub async fn realtime(State(state): State<AppState>, headers: HeaderMap, ws: Result<WebSocketUpgrade, axum::extract::ws::rejection::WebSocketUpgradeRejection>) -> Response {
    let result = async {
        let ws = ws.map_err(|_| AppError::BadRequest("请用 WebSocket 连接".into()))?;
        let token = realtime_token(&headers).ok_or_else(|| AppError::Unauthorized("请先登录".into()))?;
        let auth = AuthUser::from_token(&state, &token).await?;
        let setting = setting(&state, &auth, Capability::Realtime).await?;
        let active = caller(&state, auth, &setting).await?;
        // Connect upstream first, so a failure is an ordinary HTTP error rather than a socket that opens and dies.
        let mut request = format!("{}?model={}", state.newapi.ws_url("/v1/realtime"), urlencode(&setting.model)).into_client_request()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        request.headers_mut().insert(header::AUTHORIZATION, format!("Bearer sk-{}", active.key).parse().map_err(|_| AppError::Internal("bad key".into()))?);
        request.headers_mut().insert("openai-beta", "realtime=v1".parse().unwrap());
        let config = tungstenite::protocol::WebSocketConfig::default().max_message_size(Some(REALTIME_MAX_MESSAGE)).max_frame_size(Some(REALTIME_MAX_MESSAGE));
        let (upstream, _) = tokio_tungstenite::connect_async_with_config(request, Some(config), false).await.map_err(|e| {
            tracing::warn!("实时语音上游连接失败: {e}");
            match e {
                tungstenite::Error::Http(response) => map_upstream_error(response.status(), ""),
                _ => AppError::BadGateway("实时语音暂时不可用".into()),
            }
        })?;
        let limit = Duration::from_secs(setting.max_secs.clamp(*products::REALTIME_SECS.start(), *products::REALTIME_SECS.end()) as u64);
        let binding_id = active.id;
        Ok::<_, AppError>(ws.protocols(["realtime"]).max_message_size(REALTIME_MAX_MESSAGE).on_upgrade(move |client| pump(state, binding_id, client, upstream, limit)))
    }
    .await;
    result.unwrap_or_else(|e| error_response(Api::ChatCompletions, e))
}

fn urlencode(raw: &str) -> String {
    raw.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect()
}

fn error_event(code: &str, message: &str) -> Message {
    Message::Text(Utf8Bytes::from(json!({ "type": "error", "error": { "type": code, "code": code, "message": message } }).to_string()))
}

async fn close(client: &mut futures_util::stream::SplitSink<WebSocket, Message>, code: u16, reason: &str) {
    let _ = client.send(Message::Close(Some(CloseFrame { code, reason: Utf8Bytes::from(reason.to_string()) }))).await;
}

/// Carries events both ways until either side closes, the time is up, or the quota runs out.
async fn pump(
    state: AppState,
    binding_id: i64,
    client: WebSocket,
    upstream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    limit: Duration,
) {
    let (mut client_tx, mut client_rx) = client.split();
    let (mut up_tx, mut up_rx) = upstream.split();
    let deadline = tokio::time::sleep(limit);
    tokio::pin!(deadline);
    let mut recheck = tokio::time::interval(QUOTA_RECHECK);
    recheck.tick().await;
    loop {
        tokio::select! {
            _ = &mut deadline => {
                let _ = client_tx.send(error_event("session_time_limit", "本次实时对话已到时长上限，可以重新开始")).await;
                close(&mut client_tx, 4000, "time limit").await;
                break;
            }
            _ = recheck.tick() => {
                if require_quota(&state, binding_id).await.is_err() {
                    let _ = client_tx.send(error_event("insufficient_quota", "额度已用完，请兑换额度后继续使用")).await;
                    close(&mut client_tx, 4002, "insufficient quota").await;
                    break;
                }
            }
            message = client_rx.next() => {
                let forward = match message {
                    Some(Ok(Message::Text(text))) => match client_event(text.as_str()) {
                        Some(event) => tungstenite::Message::Text(event.into()),
                        None => continue,
                    },
                    Some(Ok(Message::Ping(data))) => { let _ = client_tx.send(Message::Pong(data)).await; continue; }
                    Some(Ok(Message::Pong(_))) | Some(Ok(Message::Binary(_))) => continue,
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                };
                if up_tx.send(forward).await.is_err() {
                    close(&mut client_tx, 1011, "upstream closed").await;
                    break;
                }
            }
            message = up_rx.next() => {
                let forward = match message {
                    Some(Ok(tungstenite::Message::Text(text))) => Message::Text(Utf8Bytes::from(upstream_event(text.as_str()))),
                    Some(Ok(tungstenite::Message::Ping(data))) => { let _ = up_tx.send(tungstenite::Message::Pong(data)).await; continue; }
                    Some(Ok(tungstenite::Message::Close(_))) | None | Some(Err(_)) => { close(&mut client_tx, 1000, "").await; break; }
                    Some(Ok(_)) => continue,
                };
                if client_tx.send(forward).await.is_err() { break; }
            }
        }
    }
    let _ = up_tx.send(tungstenite::Message::Close(None)).await;
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn images_ask_only_for_what_the_gateway_offers() {
        let sizes = vec!["1024x1024".to_string(), "1536x1024".to_string()];
        assert_eq!(image_payload("real", &sizes, r#"{"prompt":" 一只猫 ","model":"evil"}"#.as_bytes()).unwrap(), json!({ "model": "real", "prompt": "一只猫", "n": 1, "size": "1024x1024" }));
        assert_eq!(image_payload("real", &sizes, r#"{"prompt":"x","n":2,"size":"1536x1024"}"#.as_bytes()).unwrap()["size"], "1536x1024");
        for bad in [r#"{"prompt":""}"#, r#"{"prompt":"x","n":5}"#, r#"{"prompt":"x","size":"4096x4096"}"#, "not json"] {
            assert!(image_payload("real", &sizes, bad.as_bytes()).is_err(), "{bad}");
        }
    }

    #[test]
    fn image_replies_carry_images_only() {
        let reply = image_reply(&json!({ "created": 7, "model": "gpt-image-x", "usage": { "total_tokens": 9 }, "data": [{ "b64_json": "AAA", "revised_prompt": "猫" }, { "nothing": true }] })).unwrap();
        assert_eq!(reply, json!({ "created": 7, "data": [{ "b64_json": "AAA", "revised_prompt": "猫" }] }));
        assert!(image_reply(&json!({ "data": [] })).is_none());
    }

    #[test]
    fn realtime_events_keep_the_model_hidden_both_ways() {
        let up = client_event(r#"{"type":"session.update","session":{"model":"gpt-evil","voice":"alloy"}}"#).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&up).unwrap(), json!({ "type": "session.update", "session": { "voice": "alloy" } }));
        assert!(client_event("not json").is_none());
        assert!(client_event("[1,2]").is_none());
        let down = upstream_event(r#"{"type":"session.created","session":{"model":"qwen-omni-realtime","id":"s"}}"#);
        assert_eq!(serde_json::from_str::<Value>(&down).unwrap()["session"]["model"], REALTIME_LABEL);
        assert_eq!(upstream_event("raw"), "raw");
    }

    #[test]
    fn a_browser_may_send_its_token_as_a_subprotocol() {
        let mut headers = HeaderMap::new();
        headers.insert(header::SEC_WEBSOCKET_PROTOCOL, "realtime, xiaoban-token.abc.def".parse().unwrap());
        assert_eq!(realtime_token(&headers).as_deref(), Some("abc.def"));
        headers.insert(header::AUTHORIZATION, "Bearer xyz".parse().unwrap());
        assert_eq!(realtime_token(&headers).as_deref(), Some("xyz"), "a header wins");
        assert_eq!(realtime_token(&HeaderMap::new()), None);
    }
}

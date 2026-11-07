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

/// Explicit 2.6 variants use Images/GIF; the bare 2.6 subscription alias is static-only.
pub fn gif_image_model(model: &str) -> bool {
    matches!(model, "gpt-image-2.6-sunburst" | "gpt-image-2.6-flare")
}

pub fn image_formats(model: &str) -> Vec<&'static str> {
    if gif_image_model(model) { vec!["png", "gif"] } else { vec!["png"] }
}

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
    let format = match request.get("output_format") {
        None | Some(Value::Null) => "png",
        Some(value) => value.as_str().filter(|format| image_formats(model).contains(format))
            .ok_or_else(|| AppError::BadRequest("当前图片服务不支持这个输出格式".into()))?,
    };
    let mut payload = json!({ "model": model, "prompt": prompt, "n": n, "size": size });
    if format == "gif" {
        if n != 1 { return Err(AppError::BadRequest("GIF 每次只能生成 1 张".into())); }
        let count = request.get("frame_count").map_or(Some(9), Value::as_u64)
            .filter(|n| (2..=16).contains(n)).ok_or_else(|| AppError::BadRequest("GIF 帧数需要 2 到 16".into()))?;
        let duration = request.get("frame_duration_ms").map_or(Some(200), Value::as_u64)
            .filter(|n| (20..=2000).contains(n)).ok_or_else(|| AppError::BadRequest("GIF 每帧时长需要 20 到 2000 毫秒".into()))?;
        if size != "auto" {
            let dimensions = size.split_once('x').and_then(|(w,h)| Some((w.parse::<u64>().ok()?, h.parse::<u64>().ok()?)));
            if !dimensions.is_some_and(|(w,h)| w > 0 && h > 0 && w <= 8192 && h <= 8192 && w % 16 == 0 && h % 16 == 0 && w * h * count <= 32 * 1024 * 1024) {
                return Err(AppError::BadRequest("GIF 尺寸须为 16 的倍数，所有帧总像素不能超过 32M；请减小尺寸或帧数".into()));
            }
        }
        payload["output_format"] = json!("gif");
        payload["frame_count"] = json!(count);
        payload["frame_duration_ms"] = json!(duration);
    } else if request.get("frame_count").is_some() || request.get("frame_duration_ms").is_some() {
        return Err(AppError::BadRequest("帧数和每帧时长仅用于 GIF".into()));
    }
    Ok(payload)
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

fn native_gemini_image(model: &str) -> bool {
    matches!(model, "gemini-3.1-flash-image" | "gemini-3-pro-image" | "gemini-3.1-flash-lite-image" | "gemini-nano-banana-2.1")
}

fn native_gpt_image(model: &str) -> bool {
    matches!(model, "gpt-image-1.5" | "gpt-image-2" | "gpt-image-2.5" | "gpt-image-2.5-flare" | "gpt-image-2.5-sunburst" | "gpt-image-2.6")
}

/// New API maps the outer image-model alias to a Responses-capable coordinator.
/// The tool retains the image model; billing remains attached to the original alias.
fn responses_image_payload(payload: &Value) -> Result<Value, AppError> {
    let size = payload["size"].as_str().unwrap_or("auto");
    if !matches!(size, "auto" | "1024x1024" | "1536x1024" | "1024x1536") {
        return Err(AppError::BadRequest("当前 GPT 生图不支持这个尺寸，请使用后台配置的标准尺寸".into()));
    }
    Ok(json!({
        "model":payload["model"], "instructions":"Generate exactly one image for the prompt. Use the image_generation tool once.", "store":false, "stream":true,
        "input":[{"role":"user","content":[{"type":"input_text","text":payload["prompt"]}]}],
        "tools":[{"type":"image_generation","model":payload["model"],"action":"generate","size":size,"output_format":"png"}],
        "tool_choice":{"type":"image_generation"}, "parallel_tool_calls":false
    }))
}

fn responses_image_reply(reply: &[u8]) -> Option<Value> {
    fn collect(item: &Value, data: &mut Vec<Value>) {
        if item["type"] != "image_generation_call" || item["status"] != "completed" || data.len() >= IMAGE_MAX_COUNT as usize { return; }
        let Some(image) = item["result"].as_str().filter(|v| !v.is_empty()) else { return; };
        if !data.iter().any(|v| v["b64_json"].as_str() == Some(image)) {
            data.push(json!({"b64_json":image}));
        }
    }
    fn event(value: &Value, data: &mut Vec<Value>) {
        if value["type"] == "response.output_item.done" { collect(&value["item"], data); }
        let response = value.get("response").unwrap_or(value);
        if let Some(output) = response["output"].as_array() {
            for item in output { collect(item, data); }
        }
    }
    let mut data = Vec::new();
    if let Ok(value) = serde_json::from_slice::<Value>(reply) {
        event(&value, &mut data);
    } else {
        let text = std::str::from_utf8(reply).ok()?.replace("\r\n", "\n");
        for frame in text.split("\n\n") {
            let json = frame.lines().filter_map(|line| line.strip_prefix("data:").map(str::trim_start)).collect::<Vec<_>>().join("\n");
            if let Ok(value) = serde_json::from_str::<Value>(&json) { event(&value, &mut data); }
        }
    }
    if data.is_empty() { return None; }
    Some(json!({"created":chrono::Utc::now().timestamp(),"data":data}))
}

fn gemini_image_payload(payload: &Value) -> Result<Value, AppError> {
    let aspect = match payload["size"].as_str().unwrap_or("auto") {
        "auto" | "1024x1024" => "1:1",
        "1536x1024" => "3:2", "1024x1536" => "2:3",
        "1792x1024" => "16:9", "1024x1792" => "9:16",
        _ => return Err(AppError::BadRequest("当前 Gemini 生图不支持这个尺寸，请使用后台配置的标准尺寸".into())),
    };
    Ok(json!({
        "contents":[{"role":"user","parts":[{"text":payload["prompt"]}]}],
        "generationConfig":{"candidateCount":1,"responseModalities":["TEXT","IMAGE"],"imageConfig":{"aspectRatio":aspect,"imageSize":"1K"}}
    }))
}

fn gemini_image_reply(value: &Value) -> Option<Value> {
    let data: Vec<Value> = value.get("candidates")?.as_array()?.iter()
        .filter_map(|candidate| candidate.get("content")?.get("parts")?.as_array())
        .flatten().filter(|part| part["thought"].as_bool() != Some(true))
        .filter_map(|part| {
            let image = part.get("inlineData").or_else(|| part.get("inline_data"))?;
            let mime = image.get("mimeType").or_else(|| image.get("mime_type"))?.as_str()?;
            if !matches!(mime, "image/png" | "image/jpeg" | "image/webp") { return None; }
            let data = image.get("data")?.as_str().filter(|s| !s.is_empty())?;
            Some(json!({"b64_json":data}))
        }).take(IMAGE_MAX_COUNT as usize).collect();
    if data.is_empty() { return None; }
    Some(json!({"created":chrono::Utc::now().timestamp(),"data":data}))
}

async fn image_upstream_bytes(state: &AppState, key: &str, path: &str, payload: &Value) -> Result<Vec<u8>, AppError> {
    let payload = serde_json::to_vec(payload).map_err(|e| AppError::Internal(e.to_string()))?;
    let mut upstream = state.newapi.relay(path, key, payload, &[]).await?;
    let status = upstream.status();
    if upstream.content_length().is_some_and(|size| size > IMAGE_MAX_REPLY as u64) { return Err(AppError::BadGateway("生成的图片太大".into())); }
    let mut reply = Vec::new();
    while let Some(chunk) = upstream.chunk().await.map_err(|_| AppError::BadGateway("图片生成响应中断，结果可能已生成，请勿自动重试".into()))? {
        if reply.len().saturating_add(chunk.len()) > IMAGE_MAX_REPLY { return Err(AppError::BadGateway("生成的图片太大".into())); }
        reply.extend_from_slice(&chunk);
    }
    if !status.is_success() {
        let text = String::from_utf8_lossy(&reply);
        tracing::warn!("图片上游返回 {status}: {}", text.chars().take(300).collect::<String>());
        return Err(map_upstream_error(status, &text));
    }
    Ok(reply)
}

async fn image_upstream(state: &AppState, key: &str, path: &str, payload: &Value) -> Result<Value, AppError> {
    let reply = image_upstream_bytes(state, key, path, payload).await?;
    serde_json::from_slice(&reply).map_err(|_| AppError::BadGateway("图片生成返回异常".into()))
}

/// `POST /v1/images/generations`.
pub async fn image_generations(State(state): State<AppState>, auth: Result<AuthUser, AppError>, body: Result<Bytes, BytesRejection>) -> Response {
    let result = async {
        let body = body.map_err(|error| map_body_error(error.status()))?;
        let auth = auth?;
        let setting = setting(&state, &auth, Capability::Image).await?;
        let payload = image_payload(&setting.model, &setting.sizes, &body)?;
        let gpt = native_gpt_image(&setting.model);
        // Validate before any billable call. Native image tools produce one image per request.
        let native_payload = if gpt { Some(responses_image_payload(&payload)?) }
            else if native_gemini_image(&setting.model) { Some(gemini_image_payload(&payload)?) } else { None };
        let active = caller(&state, auth, &setting).await?;
        let images = if let Some(native_payload) = native_payload {
            let path = if gpt { "/v1/responses".into() } else { format!("/v1beta/models/{}:generateContent", setting.model) };
            let wanted = payload["n"].as_u64().unwrap_or(1) as usize;
            let mut data = Vec::new();
            let mut encoded_bytes = 0usize;
            for _ in 0..wanted {
                // No retries: each native request is charged by New API through this caller's own key.
                let generated = if gpt {
                    image_upstream_bytes(&state, &active.key, &path, &native_payload).await
                        .and_then(|reply| responses_image_reply(&reply).ok_or_else(|| AppError::BadGateway("图片生成没有返回图片，结果可能未知，请勿自动重试".into())))
                } else {
                    image_upstream(&state, &active.key, &path, &native_payload).await
                        .and_then(|value| gemini_image_reply(&value).ok_or_else(|| AppError::BadGateway("图片生成没有返回图片".into())))
                };
                let generated = match generated {
                    Ok(value) => value,
                    Err(error) if data.is_empty() => return Err(error),
                    Err(_) => break, // Keep images already generated; the desktop reports partial delivery.
                };
                for image in generated["data"].as_array().into_iter().flatten().take(wanted - data.len()) {
                    encoded_bytes += image["b64_json"].as_str().map_or(0, str::len);
                    if encoded_bytes > IMAGE_MAX_REPLY - 4096 { break; }
                    data.push(image.clone());
                }
                if data.len() >= wanted || encoded_bytes > IMAGE_MAX_REPLY - 4096 { break; }
            }
            if data.is_empty() { return Err(AppError::BadGateway("生成的图片太大".into())); }
            json!({"created":chrono::Utc::now().timestamp(),"data":data})
        } else {
            let value = image_upstream(&state, &active.key, "/v1/images/generations", &payload).await?;
            image_reply(&value).ok_or_else(|| AppError::BadGateway("图片生成没有返回图片".into()))?
        };
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
    fn gif_requests_use_images_api_and_reject_unsupported_or_invalid_options_before_billing() {
        let sizes = vec!["1024x1024".into()];
        let body = br#"{"prompt":"cat waving in a loop","output_format":"gif","frame_count":9,"frame_duration_ms":200,"model":"evil","stream":true}"#;
        let payload = image_payload("gpt-image-2.6-flare", &sizes, body).unwrap();
        assert_eq!(payload, json!({"model":"gpt-image-2.6-flare","prompt":"cat waving in a loop","n":1,"size":"1024x1024","output_format":"gif","frame_count":9,"frame_duration_ms":200}));
        assert!(!native_gpt_image("gpt-image-2.6-flare"), "GIF-capable models go directly through Images, without the subscription streaming adapter");
        assert_eq!(image_formats("gpt-image-2.6-sunburst"), vec!["png", "gif"]);
        assert_eq!(image_formats("gpt-image-2.5"), vec!["png"]);
        assert!(image_payload("gpt-image-2.5", &sizes, body).is_err());
        for options in [json!({"n":2}), json!({"frame_count":1}), json!({"frame_count":17}), json!({"frame_count":2.5}), json!({"frame_duration_ms":19}), json!({"frame_duration_ms":2001}), json!({"output_format":"svg"})] {
            let mut request = json!({"prompt":"cat","output_format":"gif"});
            for (key,value) in options.as_object().unwrap() { request[key] = value.clone(); }
            assert!(image_payload("gpt-image-2.6-flare", &sizes, &serde_json::to_vec(&request).unwrap()).is_err());
        }
        assert!(image_payload("gpt-image-2.6-flare", &sizes, br#"{"prompt":"cat","frame_count":9}"#).is_err());
        assert!(image_payload("gpt-image-2.6-flare", &["8192x8192".into()], br#"{"prompt":"cat","output_format":"gif"}"#).is_err());
    }

    #[test]
    fn gpt_images_use_a_single_responses_tool_and_validate_size_before_billing() {
        let payload = image_payload("gpt-image-2.5", &["1024x1024".into()], br#"{"prompt":"cat","n":4,"model":"ignore-client-model"}"#).unwrap();
        let native = responses_image_payload(&payload).unwrap();
        assert!(native_gpt_image("gpt-image-2.5"));
        assert!(!native_gpt_image("ag/gemini-3.1-flash-image"));
        assert_eq!(native["model"], "gpt-image-2.5");
        assert_eq!(native["tools"][0]["model"], "gpt-image-2.5");
        assert_eq!(native["input"][0]["content"][0]["text"], "cat");
        assert_eq!(native["tools"].as_array().unwrap().len(), 1);
        assert_eq!(native["parallel_tool_calls"], false);
        // Codex subscription endpoints reject max_tool_calls even though the API supports it.
        assert!(native.get("max_tool_calls").is_none());
        assert_eq!(native["stream"], true);
        assert_eq!(native["store"], false);
        assert!(native.get("n").is_none());
        assert!(responses_image_payload(&json!({"prompt":"cat","size":"1792x1024"})).is_err());
    }

    #[test]
    fn gpt_26_subscription_alias_preserves_tool_model_and_rejects_gif_before_billing() {
        let sizes = vec!["auto".into()];
        let payload = image_payload("gpt-image-2.6", &sizes, br#"{"prompt":"cat"}"#).unwrap();
        assert!(native_gpt_image("gpt-image-2.6"));
        let native = responses_image_payload(&payload).unwrap();
        assert_eq!(native["model"], "gpt-image-2.6");
        assert_eq!(native["tools"][0]["model"], "gpt-image-2.6");
        assert_eq!(native["tools"][0]["output_format"], "png");
        assert_eq!(image_formats("gpt-image-2.6"), vec!["png"]);
        assert!(image_payload("gpt-image-2.6", &sizes, br#"{"prompt":"cat","output_format":"gif"}"#).is_err());
    }

    #[test]
    fn responses_images_ignore_partial_events_and_deduplicate_completed_images() {
        let item = json!({"type":"image_generation_call","status":"completed","id":"image-1","result":"final-image","revised_prompt":"private","model":"private"});
        let events = [
            json!({"type":"response.image_generation_call.partial_image","partial_image_b64":"partial"}),
            json!({"type":"response.output_item.done","item":{"type":"image_generation_call","status":"in_progress","result":"incomplete"}}),
            json!({"type":"response.output_item.done","item":item}),
            json!({"type":"response.completed","response":{"model":"private","usage":{"input_tokens":100},"output":[item,{"type":"message","content":[{"text":"private"}]}]}}),
        ];
        let sse = events.iter().map(|event| format!("event: message\r\ndata: {event}\r\n\r\n")).collect::<String>();
        let reply = responses_image_reply(sse.as_bytes()).unwrap();
        assert_eq!(reply["data"], json!([{"b64_json":"final-image"}]));
        assert!(reply.get("model").is_none() && reply.get("usage").is_none());
        assert!(responses_image_reply(b"data: {\"type\":\"response.failed\"}\n\n").is_none());
        assert!(responses_image_reply(b"data: {\"type\":\"response.output_item.done\",\"item\":").is_none());
    }

    #[test]
    fn responses_images_accept_json_and_preserve_an_already_completed_image() {
        let item = json!({"type":"image_generation_call","status":"completed","result":"image"});
        let json = serde_json::to_vec(&json!({"output":[item]})).unwrap();
        assert_eq!(responses_image_reply(&json).unwrap()["data"], json!([{"b64_json":"image"}]));
        let stream = format!("data: {}\n\ndata: {{\"type\":\"response.failed\"}}\n\n", json!({"type":"response.output_item.done","item":item}));
        assert_eq!(responses_image_reply(stream.as_bytes()).unwrap()["data"].as_array().unwrap().len(), 1);
        assert!(responses_image_reply(br#"{"output":[{"type":"message","text":"no image"}]}"#).is_none());
    }

    #[test]
    fn gemini_images_use_native_modalities_without_exposing_the_model() {
        let payload = image_payload("gemini-3.1-flash-image", &["1024x1024".into(),"1536x1024".into()], br#"{"prompt":"cat","n":4,"size":"1536x1024"}"#).unwrap();
        assert!(native_gemini_image("gemini-3.1-flash-image"));
        assert!(!native_gemini_image("gpt-image-1.5"));
        assert!(!native_gemini_image("gemini-4-argon"));
        let native = gemini_image_payload(&payload).unwrap();
        assert_eq!(native["contents"][0]["parts"][0]["text"], "cat");
        assert_eq!(native["generationConfig"]["candidateCount"], 1);
        assert_eq!(native["generationConfig"]["imageConfig"]["aspectRatio"], "3:2");
        assert_eq!(native["generationConfig"]["responseModalities"], json!(["TEXT","IMAGE"]));
        assert!(native.get("model").is_none() && native.get("n").is_none());
        assert!(gemini_image_payload(&json!({"prompt":"cat","size":"8x8"})).is_err());
    }

    #[test]
    fn gemini_image_replies_exclude_thoughts_text_usage_and_non_images() {
        let value = json!({"modelVersion":"private-model","usageMetadata":{"totalTokenCount":99},"candidates":[{"content":{"parts":[
            {"thought":true,"inlineData":{"mimeType":"image/png","data":"hidden-thought"}},
            {"text":"private explanatory text"},
            {"inlineData":{"mimeType":"image/png","data":"visible-image"}},
            {"inline_data":{"mime_type":"image/jpeg","data":"second-image"}},
            {"inlineData":{"mimeType":"text/html","data":"not-an-image"}}
        ]}}]});
        let reply = gemini_image_reply(&value).unwrap();
        assert_eq!(reply["data"], json!([{"b64_json":"visible-image"},{"b64_json":"second-image"}]));
        assert!(reply.get("modelVersion").is_none() && reply.get("usageMetadata").is_none());
        assert!(gemini_image_reply(&json!({"candidates":[{"content":{"parts":[{"text":"blocked"}]}}]})).is_none());
    }

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

//! 对外的调用代理，支持三种协议：OpenAI 的 `chat/completions` 和 `responses`，以及 Anthropic 的 `messages`。
//!
//! 客户端只认档位名：请求里的 `model` 是档位，这里换成真实模型再转发；响应、流式事件和错误信息里都不回传真实模型名。
//! 三种协议共用同一条处理链（验签、选档位、查余额、转发），区别只有转发路径、要透传的协议头、
//! 响应里 `model` 所在的位置，以及错误体的格式（SDK 要能直接读出 message）。

#[path = "protocol_limits.rs"]
mod protocol_limits;
use protocol_limits::output_limit_error;

use crate::{binding, error::AppError, extract::AuthUser, products, quota, state::AppState};
use axum::{
    body::Body,
    extract::{rejection::BytesRejection, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Api {
    ChatCompletions,
    Responses,
    Messages,
}

impl Api {
    fn path(self) -> &'static str {
        match self {
            Api::ChatCompletions => "/v1/chat/completions",
            Api::Responses => "/v1/responses",
            Api::Messages => "/v1/messages",
        }
    }
}

/// 只透传协议必需的头，且只用于 Messages。密钥类的头（`Authorization`、`x-api-key`）里是用户自己的登录凭证，绝不能转给网关。
const ANTHROPIC_HEADERS: [&str; 2] = ["anthropic-version", "anthropic-beta"];

fn forwarded_headers(api: Api, headers: &HeaderMap) -> Vec<(&'static str, String)> {
    if api != Api::Messages {
        return Vec::new();
    }
    ANTHROPIC_HEADERS
        .iter()
        .filter_map(|name| headers.get(*name).and_then(|v| v.to_str().ok()).map(|v| (*name, v.to_string())))
        .collect()
}

fn anthropic_error_type(status: StatusCode) -> &'static str {
    match status.as_u16() {
        400 => "invalid_request_error",
        401 => "authentication_error",
        402 => "billing_error",
        403 => "permission_error",
        404 => "not_found_error",
        413 => "request_too_large",
        429 => "rate_limit_error",
        529 => "overloaded_error",
        s if s >= 500 => "api_error",
        _ => "invalid_request_error",
    }
}

/// OpenAI 风格是 `{error:{message,type,code}}`，Anthropic 风格是 `{type:"error",error:{type,message}}`。
/// 两边都多带一个稳定的 `code`，供我们自己的客户端据此分支（如 `insufficient_quota`）。
fn error_body(api: Api, status: StatusCode, code: &str, message: &str) -> Value {
    if api == Api::Messages {
        return json!({ "type": "error", "error": { "type": anthropic_error_type(status), "message": message, "code": code } });
    }
    let kind = if status.is_client_error() { "invalid_request_error" } else { "api_error" };
    json!({ "error": { "message": message, "type": kind, "code": code } })
}

fn error_response(api: Api, err: AppError) -> Response {
    let (status, code, message) = err.parts();
    (status, Json(error_body(api, status, code, &message))).into_response()
}

pub async fn list_models(State(state): State<AppState>, auth: Result<AuthUser, AppError>) -> Response {
    let result = async {
        let auth = auth?;
        let tiers = products::tiers(&state.pool, &auth.product.product_id, true).await?;
        let data: Vec<Value> = tiers
            .iter()
            .map(|t| json!({ "id": t.tier, "object": "model", "created": 0, "owned_by": "xiangban" }))
            .collect();
        Ok::<_, AppError>(Json(json!({ "object": "list", "data": data })).into_response())
    }
    .await;
    result.unwrap_or_else(|e| error_response(Api::ChatCompletions, e))
}

pub async fn chat_completions(State(state): State<AppState>, auth: Result<AuthUser, AppError>, headers: HeaderMap, body: Result<Bytes, BytesRejection>) -> Response {
    serve(Api::ChatCompletions, &state, auth, &headers, body).await
}

pub async fn responses(State(state): State<AppState>, auth: Result<AuthUser, AppError>, headers: HeaderMap, body: Result<Bytes, BytesRejection>) -> Response {
    serve(Api::Responses, &state, auth, &headers, body).await
}

pub async fn messages(State(state): State<AppState>, auth: Result<AuthUser, AppError>, headers: HeaderMap, body: Result<Bytes, BytesRejection>) -> Response {
    serve(Api::Messages, &state, auth, &headers, body).await
}

async fn serve(api: Api, state: &AppState, auth: Result<AuthUser, AppError>, headers: &HeaderMap, body: Result<Bytes, BytesRejection>) -> Response {
    let body = match body {
        Ok(body) => body,
        Err(error) => return error_response(api, map_body_error(error.status())),
    };
    match handle(api, state, auth, headers, body).await {
        Ok(response) => response,
        Err(err) => error_response(api, err),
    }
}

async fn handle(api: Api, state: &AppState, auth: Result<AuthUser, AppError>, headers: &HeaderMap, body: Bytes) -> Result<Response, AppError> {
    let auth = auth?;
    let mut request: Value =
        serde_json::from_slice(&body).map_err(|_| AppError::BadRequest("请求体不是有效的 JSON".into()))?;
    let fields = request.as_object_mut().ok_or_else(|| AppError::BadRequest("请求体需要是 JSON 对象".into()))?;

    let requested = fields.get("model").and_then(Value::as_str).map(str::to_string);
    let tiers = products::tiers(&state.pool, &auth.product.product_id, true).await?;
    let tier = products::resolve_tier(&tiers, requested.as_deref()).ok_or_else(|| AppError::Coded {
        status: StatusCode::BAD_REQUEST,
        code: "model_not_found",
        message: "未知的模型档位".into(),
    })?;

    let active = binding::ensure_active(state, auth.uid, &auth.product).await?;
    require_quota(state, active.id).await?;
    if tier.requires_paid && quota::paid_total(&state.pool, active.id).await? <= 0 {
        return Err(AppError::Coded {
            status: StatusCode::FORBIDDEN,
            code: "paid_required",
            message: "该档位需要先兑换额度才能使用".into(),
        });
    }

    fields.insert("model".into(), Value::String(tier.model.clone()));
    validate_reasoning(api, fields, &crate::reasoning::levels(&tier.model, tier.reasoning_efforts.as_deref()))?;
    sqlx::query("UPDATE bindings SET last_used_at = now() WHERE id = $1").bind(active.id).execute(&state.pool).await?;

    let payload = serde_json::to_vec(&request).map_err(|e| AppError::Internal(e.to_string()))?;
    let upstream = state.newapi.relay(api.path(), &active.key, payload, &forwarded_headers(api, headers)).await?;
    respond(upstream, &tier.tier).await
}

async fn require_quota(state: &AppState, binding_id: i64) -> Result<(), AppError> {
    let total = {
        let mut conn = state.pool.acquire().await?;
        quota::live_total(&mut conn, binding_id).await?
    };
    if total <= 0 {
        return Err(AppError::Coded {
            status: StatusCode::PAYMENT_REQUIRED,
            code: "insufficient_quota",
            message: "额度已用完，请兑换额度后继续使用".into(),
        });
    }
    Ok(())
}

/// The longest text one speech request may carry. Clients read answers sentence by sentence.
const SPEECH_MAX_CHARS: usize = 1000;
/// The largest audio relayed back for one request.
const SPEECH_MAX_AUDIO: usize = 8 * 1024 * 1024;

/// OpenAI-style speech request for the configured upstream model. The client never names the model; it may
/// ask for one of the offered voices.
fn speech_payload(model: &str, voices: &[String], body: &[u8]) -> Result<Value, AppError> {
    let request: Value = serde_json::from_slice(body).map_err(|_| AppError::BadRequest("请求体不是有效的 JSON".into()))?;
    let input = request.get("input").and_then(Value::as_str).map(str::trim).unwrap_or_default();
    if input.is_empty() {
        return Err(AppError::BadRequest("请提供要朗读的文字".into()));
    }
    if input.chars().count() > SPEECH_MAX_CHARS {
        return Err(AppError::Coded { status: StatusCode::PAYLOAD_TOO_LARGE, code: "request_too_large", message: "一次朗读的文字太长，请分句发送".into() });
    }
    let voice = match request.get("voice").and_then(Value::as_str) {
        Some(asked) => voices.iter().find(|v| *v == asked).cloned().ok_or_else(|| AppError::BadRequest("没有这个音色".into()))?,
        None => voices.first().cloned().unwrap_or_else(|| "alloy".into()),
    };
    let speed = request.get("speed").and_then(Value::as_f64).unwrap_or(1.0).clamp(0.5, 2.0);
    Ok(json!({ "model": model, "input": input, "voice": voice, "speed": speed, "response_format": "mp3" }))
}

/// `POST /v1/audio/speech`: one sentence to MP3, billed to the caller's account like any other call.
pub async fn audio_speech(State(state): State<AppState>, auth: Result<AuthUser, AppError>, body: Result<Bytes, BytesRejection>) -> Response {
    let result = async {
        let body = body.map_err(|error| map_body_error(error.status()))?;
        let auth = auth?;
        let model = state.config.speech_model.clone().ok_or(AppError::Coded {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "speech_unavailable",
            message: "小伴 AI 语音暂未开放".into(),
        })?;
        let payload = speech_payload(&model, &state.config.speech_voices, &body)?;
        let active = binding::ensure_active(&state, auth.uid, &auth.product).await?;
        require_quota(&state, active.id).await?;
        sqlx::query("UPDATE bindings SET last_used_at = now() WHERE id = $1").bind(active.id).execute(&state.pool).await?;
        let payload = serde_json::to_vec(&payload).map_err(|e| AppError::Internal(e.to_string()))?;
        let upstream = state.newapi.relay("/v1/audio/speech", &active.key, payload, &[]).await?;
        let status = upstream.status();
        if !status.is_success() {
            let text = upstream.text().await.unwrap_or_default();
            tracing::warn!("语音上游返回 {status}: {}", text.chars().take(300).collect::<String>());
            return Err(map_upstream_error(status, &text));
        }
        let audio = upstream.bytes().await.map_err(|_| AppError::BadGateway("AI 语音暂时不可用".into()))?;
        if audio.is_empty() || audio.len() > SPEECH_MAX_AUDIO {
            return Err(AppError::BadGateway("AI 语音没有返回可用的音频".into()));
        }
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "audio/mpeg")
            .header(header::CACHE_CONTROL, "no-store")
            .body(Body::from(audio))
            .map_err(|e| AppError::Internal(e.to_string()))
    }
    .await;
    result.unwrap_or_else(|e| error_response(Api::ChatCompletions, e))
}

fn validate_reasoning(api: Api, fields: &Map<String, Value>, levels: &[String]) -> Result<(), AppError> {
    let value = match api {
        Api::Responses => fields.get("reasoning").and_then(|v| v.get("effort")),
        Api::ChatCompletions => fields.get("reasoning_effort"),
        Api::Messages => fields.get("output_config").and_then(|v| v.get("effort")),
    };
    if let Some(value) = value {
        if !value.as_str().is_some_and(|s| levels.iter().any(|level| level == s)) {
            return Err(AppError::Coded { status: StatusCode::BAD_REQUEST, code: "unsupported_reasoning_effort", message: "该模式不支持所选思考等级，请刷新模式列表后重试".into() });
        }
    }
    Ok(())
}

async fn respond(upstream: reqwest::Response, tier: &str) -> Result<Response, AppError> {
    let status = upstream.status();
    if !status.is_success() {
        let text = upstream.text().await.unwrap_or_default();
        tracing::warn!("上游返回 {status}: {}", text.chars().take(300).collect::<String>());
        return Err(map_upstream_error(status, &text));
    }

    let is_stream = upstream
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/event-stream"));
    if is_stream {
        let stream = relabel_stream(Box::pin(upstream.bytes_stream()), tier.to_string());
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
            .header("x-accel-buffering", "no")
            .body(Body::from_stream(stream))
            .map_err(|e| AppError::Internal(e.to_string()));
    }

    let bytes = upstream.bytes().await.map_err(|_| AppError::BadGateway("AI 服务暂时不可用".into()))?;
    let mut value: Value =
        serde_json::from_slice(&bytes).map_err(|_| AppError::BadGateway("AI 服务返回了无法识别的内容".into()))?;
    relabel_model(&mut value, tier);
    Ok(Json(value).into_response())
}

/// 上游的报错原文可能带真实模型名或渠道信息，只按状态归类成通用文案。
fn map_upstream_error(status: StatusCode, body: &str) -> AppError {
    let lower = body.to_lowercase();
    if status == StatusCode::TOO_MANY_REQUESTS {
        return AppError::Coded { status, code: "rate_limited", message: "请求过于频繁，请稍后再试".into() };
    }
    if status == StatusCode::PAYMENT_REQUIRED || lower.contains("quota") && lower.contains("not enough") {
        return AppError::Coded {
            status: StatusCode::PAYMENT_REQUIRED,
            code: "insufficient_quota",
            message: "额度暂时不足，若刚兑换过请稍候几秒再试".into(),
        };
    }
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        return map_body_error(status);
    }
    if status == StatusCode::BAD_REQUEST {
        if ["context_length_exceeded", "context_window_exceeded", "maximum context length", "prompt is too long", "input token count exceeds", "too many input tokens"].iter().any(|needle| lower.contains(needle)) {
            return AppError::Coded { status, code: "context_length_exceeded", message: "对话超过模型的上下文上限，请压缩会话或调低上下文上限后继续".into() };
        }
        if let Some(limit) = output_limit_error(&lower) {
            let message = limit.map_or_else(|| "max_tokens exceeds upstream output limit".to_string(),
                |limit| format!("max_tokens exceeds output limit of {limit}"));
            return AppError::Coded { status, code: "output_limit_exceeded", message };
        }
        return AppError::Coded { status, code: "bad_request", message: "请求参数与模型不兼容，请检查模型协议、工具和图片支持".into() };
    }
    AppError::BadGateway("AI 服务暂时不可用，请稍后再试".into())
}

fn map_body_error(status: StatusCode) -> AppError {
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::Coded { status, code: "request_too_large", message: "请求体积过大，请减少图片、降低截图尺寸或压缩会话后继续".into() }
    } else {
        AppError::BadRequest("请求内容无法读取，请重新发送".into())
    }
}

fn set_model(fields: &mut Map<String, Value>, tier: &str) -> bool {
    match fields.get_mut("model") {
        Some(slot) if slot.is_string() => {
            *slot = Value::String(tier.to_string());
            true
        }
        _ => false,
    }
}

/// 把响应里承载模型名的固定位置改成档位名，返回有没有改动：
/// 顶层 `model`（对话补全；Messages 的整条消息）、`response.model`（Responses 的流式事件）、
/// `message.model`（Messages 的 `message_start` 事件）。
/// 只动这三处。内容里的工具参数等用户数据也可能有叫 `model` 的字段，改了就会弄坏他们的数据。
fn relabel_model(value: &mut Value, tier: &str) -> bool {
    let Some(fields) = value.as_object_mut() else { return false };
    let mut changed = set_model(fields, tier);
    for nested in ["response", "message"] {
        if let Some(inner) = fields.get_mut(nested).and_then(Value::as_object_mut) {
            changed |= set_model(inner, tier);
        }
    }
    changed
}

/// 逐行改写 SSE 事件里的 `model`。上游一个数据块可能在行中间截断，所以按换行缓冲，凑齐整行再处理。
fn relabel_stream<S>(inner: S, tier: String) -> impl Stream<Item = Result<Bytes, std::io::Error>>
where
    S: Stream<Item = reqwest::Result<Bytes>> + Unpin + Send + 'static,
{
    struct State<S> {
        inner: S,
        buffer: Vec<u8>,
        tier: String,
        done: bool,
    }
    futures_util::stream::unfold(State { inner, buffer: Vec::new(), tier, done: false }, |mut st| async move {
        if st.done {
            return None;
        }
        loop {
            match st.inner.next().await {
                Some(Ok(chunk)) => {
                    st.buffer.extend_from_slice(&chunk);
                    let out = drain_complete_lines(&mut st.buffer, &st.tier);
                    if !out.is_empty() {
                        return Some((Ok(Bytes::from(out)), st));
                    }
                }
                Some(Err(e)) => {
                    st.done = true;
                    return Some((Err(std::io::Error::other(e)), st));
                }
                None => {
                    st.done = true;
                    let rest = std::mem::take(&mut st.buffer);
                    return if rest.is_empty() { None } else { Some((Ok(Bytes::from(rest)), st)) };
                }
            }
        }
    })
}

/// 取出缓冲区里所有以换行结尾的整行并改写，不完整的尾部留在缓冲区。
fn drain_complete_lines(buffer: &mut Vec<u8>, tier: &str) -> Vec<u8> {
    let Some(last_newline) = buffer.iter().rposition(|b| *b == b'\n') else { return Vec::new() };
    let complete: Vec<u8> = buffer.drain(..=last_newline).collect();
    let mut out = Vec::with_capacity(complete.len());
    for line in complete.split_inclusive(|b| *b == b'\n') {
        match std::str::from_utf8(line) {
            Ok(text) => {
                let (content, ending) = match text.strip_suffix("\r\n") {
                    Some(c) => (c, "\r\n"),
                    None => (text.strip_suffix('\n').unwrap_or(text), "\n"),
                };
                out.extend_from_slice(relabel_line(content, tier).as_bytes());
                out.extend_from_slice(ending.as_bytes());
            }
            Err(_) => out.extend_from_slice(line),
        }
    }
    out
}

fn relabel_line(line: &str, tier: &str) -> String {
    let Some(payload) = line.strip_prefix("data:") else { return line.to_string() };
    let payload = payload.trim();
    if payload.is_empty() || payload == "[DONE]" {
        return line.to_string();
    }
    let Ok(mut value) = serde_json::from_str::<Value>(payload) else { return line.to_string() };
    if relabel_model(&mut value, tier) {
        format!("data: {value}")
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn model_is_relabeled_only_inside_data_events() {
        let line = r#"data: {"id":"1","model":"deepseek-v4.1-flash","choices":[]}"#;
        let out = relabel_line(line, "standard");
        assert!(out.contains(r#""model":"standard""#) && !out.contains("deepseek"), "{out}");
        assert_eq!(relabel_line("data: [DONE]", "standard"), "data: [DONE]");
        assert_eq!(relabel_line(": keep-alive", "standard"), ": keep-alive");
        assert_eq!(relabel_line("event: message", "standard"), "event: message");
        assert_eq!(relabel_line("data: not json", "standard"), "data: not json");
        assert_eq!(relabel_line(r#"data: {"id":"1"}"#, "standard"), r#"data: {"id":"1"}"#);
    }

    #[test]
    fn relabeling_reaches_the_known_nested_locations() {
        let mut responses_event = json!({ "type": "response.completed", "response": { "id": "r1", "model": "claude-sonnet-5.5", "status": "completed" } });
        assert!(relabel_model(&mut responses_event, "premium"));
        assert_eq!(responses_event["response"]["model"], "premium");

        let mut message_start = json!({ "type": "message_start", "message": { "id": "m1", "model": "claude-sonnet-5.5-20260101", "role": "assistant" } });
        assert!(relabel_model(&mut message_start, "premium"));
        assert_eq!(message_start["message"]["model"], "premium", "a dated upstream model id is replaced too, not matched by name");

        let mut whole_message = json!({ "id": "m1", "type": "message", "model": "claude-sonnet-5.5", "content": [] });
        assert!(relabel_model(&mut whole_message, "premium"));
        assert_eq!(whole_message["model"], "premium");

        let line = r#"data: {"type":"message_start","message":{"model":"real-model","id":"m1"}}"#;
        let out = relabel_line(line, "premium");
        assert!(out.contains(r#""model":"premium""#) && !out.contains("real-model"), "{out}");
    }

    #[test]
    fn user_data_called_model_is_never_touched() {
        for event in [
            json!({ "type": "content_block_start", "content_block": { "type": "tool_use", "input": { "model": "keep-me" } } }),
            json!({ "type": "response.output_item.done", "item": { "type": "function_call", "arguments": "{\"model\":\"keep-me\"}", "model": "keep-me" } }),
            json!({ "choices": [{ "delta": { "tool_calls": [{ "function": { "arguments": "{\"model\":\"keep-me\"}" } }] } }] }),
            json!({ "type": "error", "error": { "message": "x", "model": "keep-me" } }),
            json!({ "message": "plain string, not an object", "model_info": "keep-me" }),
        ] {
            let mut copy = event.clone();
            assert!(!relabel_model(&mut copy, "standard"), "{event}");
            assert_eq!(copy, event);
        }
        let mut non_string = json!({ "model": 42 });
        assert!(!relabel_model(&mut non_string, "standard"));
    }

    #[test]
    fn partial_lines_wait_for_their_newline() {
        let mut buffer = br#"data: {"model":"real","x":1}"#.to_vec();
        assert!(drain_complete_lines(&mut buffer, "standard").is_empty());
        assert!(!buffer.is_empty());

        buffer.extend_from_slice(b"\n\ndata: [DONE]\n\n");
        let out = String::from_utf8(drain_complete_lines(&mut buffer, "standard")).unwrap();
        assert!(out.contains(r#""model":"standard""#) && !out.contains("real"), "{out}");
        assert!(out.ends_with("data: [DONE]\n\n"), "{out:?}");
        assert!(buffer.is_empty());
    }

    #[test]
    fn crlf_line_endings_survive() {
        let mut buffer = b"data: {\"model\":\"real\"}\r\n\r\n".to_vec();
        let out = String::from_utf8(drain_complete_lines(&mut buffer, "t")).unwrap();
        assert_eq!(out, "data: {\"model\":\"t\"}\r\n\r\n");
    }

    #[test]
    fn overflow_errors_are_actionable_without_exposing_upstream_details() {
        for body in [
            r#"{"error":{"code":"context_length_exceeded","message":"private-model maximum context length is 128000"}}"#,
            r#"{"error":{"message":"[400]: {\"error\":{\"message\":\"The input token count exceeds the maximum number of tokens allowed 1048576.\"}}"}}"#,
            r#"{"error":{"message":"prompt is too long: 1214141 tokens > 1048576 maximum (channel secret)"}}"#,
        ] {
            let (_, code, message) = map_upstream_error(StatusCode::BAD_REQUEST, body).parts();
            assert_eq!(code, "context_length_exceeded");
            assert!(message.contains("压缩会话"));
            assert!(!message.contains("private-model") && !message.contains("channel") && !message.contains("1048576"));
        }
        assert_eq!(map_upstream_error(StatusCode::PAYLOAD_TOO_LARGE, "private channel").parts().1, "request_too_large");
        assert_eq!(map_body_error(StatusCode::PAYLOAD_TOO_LARGE).parts().1, "request_too_large");
        assert_eq!(map_upstream_error(StatusCode::BAD_REQUEST, "unexpected parameter reason").parts().1, "bad_request");
        for api in [Api::Responses, Api::Messages, Api::ChatCompletions] {
            assert_eq!(error_body(api, StatusCode::BAD_REQUEST, "context_length_exceeded", "超限")["error"]["code"], "context_length_exceeded");
        }
    }

    #[test]
    fn upstream_errors_never_echo_their_body() {
        let leaky = r#"{"error":{"message":"model deepseek-v4.1-flash not available on channel 7"}}"#;
        for status in [StatusCode::BAD_GATEWAY, StatusCode::INTERNAL_SERVER_ERROR, StatusCode::BAD_REQUEST, StatusCode::TOO_MANY_REQUESTS] {
            let (_, _, message) = map_upstream_error(status, leaky).parts();
            assert!(!message.contains("deepseek") && !message.contains("channel"), "{status}: {message}");
        }
        assert_eq!(map_upstream_error(StatusCode::TOO_MANY_REQUESTS, "").parts().1, "rate_limited");
        assert_eq!(map_upstream_error(StatusCode::PAYMENT_REQUIRED, "").parts().1, "insufficient_quota");
        assert_eq!(map_upstream_error(StatusCode::INTERNAL_SERVER_ERROR, "").parts().0, StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn errors_use_each_protocols_own_shape() {
        let openai = error_body(Api::Responses, StatusCode::PAYMENT_REQUIRED, "insufficient_quota", "额度已用完");
        assert_eq!(openai["error"]["message"], "额度已用完");
        assert_eq!(openai["error"]["code"], "insufficient_quota");
        assert!(openai.get("type").is_none());

        let anthropic = error_body(Api::Messages, StatusCode::PAYMENT_REQUIRED, "insufficient_quota", "额度已用完");
        assert_eq!(anthropic["type"], "error");
        assert_eq!(anthropic["error"]["type"], "billing_error");
        assert_eq!(anthropic["error"]["message"], "额度已用完");
        assert_eq!(anthropic["error"]["code"], "insufficient_quota");

        for (status, expected) in [
            (StatusCode::BAD_REQUEST, "invalid_request_error"),
            (StatusCode::UNAUTHORIZED, "authentication_error"),
            (StatusCode::FORBIDDEN, "permission_error"),
            (StatusCode::TOO_MANY_REQUESTS, "rate_limit_error"),
            (StatusCode::BAD_GATEWAY, "api_error"),
            (StatusCode::SERVICE_UNAVAILABLE, "api_error"),
        ] {
            assert_eq!(anthropic_error_type(status), expected, "{status}");
        }
    }

    #[test]
    fn only_protocol_headers_are_forwarded_and_never_credentials() {
        let mut headers = HeaderMap::new();
        for (name, value) in [
            ("anthropic-version", "2023-06-01"),
            ("anthropic-beta", "tools-2024-04-04"),
            ("authorization", "Bearer user-jwt"),
            ("x-api-key", "user-jwt"),
            ("cookie", "session=1"),
            ("x-forwarded-for", "203.0.113.9"),
        ] {
            headers.insert(name, HeaderValue::from_static(value));
        }
        assert_eq!(
            forwarded_headers(Api::Messages, &headers),
            vec![("anthropic-version", "2023-06-01".to_string()), ("anthropic-beta", "tools-2024-04-04".to_string())]
        );
        assert!(forwarded_headers(Api::Responses, &headers).is_empty());
        assert!(forwarded_headers(Api::ChatCompletions, &headers).is_empty());
        assert!(forwarded_headers(Api::Messages, &HeaderMap::new()).is_empty());
    }
}

#[cfg(test)]
mod reasoning_tests {
    use super::*;
    #[test]
    fn effort_is_validated_for_each_protocol_without_rewriting_the_payload() {
        let levels = vec!["low".into(), "high".into(), "max".into()];
        for (api, payload) in [
            (Api::Responses, json!({"reasoning":{"effort":"max"}})),
            (Api::ChatCompletions, json!({"reasoning_effort":"high"})),
            (Api::Messages, json!({"output_config":{"effort":"low"}})),
        ] {
            let fields = payload.as_object().unwrap();
            assert!(validate_reasoning(api, fields, &levels).is_ok());
            assert!(validate_reasoning(api, fields, &[]).is_err());
            assert_eq!(Value::Object(fields.clone()), payload);
        }
        assert!(validate_reasoning(Api::Responses, json!({}).as_object().unwrap(), &[]).is_ok());
        assert!(validate_reasoning(Api::ChatCompletions, json!({"reasoning_effort":123}).as_object().unwrap(), &levels).is_err());
    }
}

#[cfg(test)]
mod speech_tests {
    use super::*;
    fn voices() -> Vec<String> { vec!["zh_female".into(), "zh_male".into()] }

    #[test]
    fn the_gateway_names_the_model_and_defaults_the_voice() {
        let payload = speech_payload("tts-1", &voices(), r#"{"input":" 你好。 ","model":"gpt-anything"}"#.as_bytes()).unwrap();
        assert_eq!(payload, json!({ "model": "tts-1", "input": "你好。", "voice": "zh_female", "speed": 1.0, "response_format": "mp3" }));
    }

    #[test]
    fn only_offered_voices_are_accepted_and_speed_is_bounded() {
        let payload = speech_payload("tts-1", &voices(), r#"{"input":"好","voice":"zh_male","speed":9}"#.as_bytes()).unwrap();
        assert_eq!((payload["voice"].as_str(), payload["speed"].as_f64()), (Some("zh_male"), Some(2.0)));
        assert!(speech_payload("tts-1", &voices(), r#"{"input":"好","voice":"someone_else"}"#.as_bytes()).is_err());
    }

    #[test]
    fn empty_or_overlong_text_is_refused() {
        assert!(speech_payload("tts-1", &voices(), r#"{"input":"   "}"#.as_bytes()).is_err());
        let long = format!(r#"{{"input":"{}"}}"#, "字".repeat(SPEECH_MAX_CHARS + 1));
        assert!(matches!(speech_payload("tts-1", &voices(), long.as_bytes()), Err(AppError::Coded { code: "request_too_large", .. })));
    }
}

//! OpenAI 兼容的调用代理。客户端只认档位名：请求里的 `model` 是档位，这里换成真实模型再转发，
//! 响应里的 `model` 和上游错误信息都不回传真实模型名。

use crate::{
    binding, error::AppError, extract::AuthUser, products, quota, state::AppState,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde_json::{json, Value};

/// 把 AppError 渲染成 OpenAI 风格的错误体，让现成的 SDK 能直接读出 message。
fn openai_error(err: AppError) -> Response {
    let (status, code, message) = err.parts();
    let kind = if status.is_client_error() { "invalid_request_error" } else { "api_error" };
    (status, Json(json!({ "error": { "message": message, "type": kind, "code": code } }))).into_response()
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
    result.unwrap_or_else(openai_error)
}

pub async fn chat_completions(State(state): State<AppState>, auth: Result<AuthUser, AppError>, body: Bytes) -> Response {
    match handle(&state, auth, body).await {
        Ok(response) => response,
        Err(err) => openai_error(err),
    }
}

async fn handle(state: &AppState, auth: Result<AuthUser, AppError>, body: Bytes) -> Result<Response, AppError> {
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

    let total = {
        let mut conn = state.pool.acquire().await?;
        quota::live_total(&mut conn, active.id).await?
    };
    if total <= 0 {
        return Err(AppError::Coded {
            status: StatusCode::PAYMENT_REQUIRED,
            code: "insufficient_quota",
            message: "额度已用完，请兑换额度后继续使用".into(),
        });
    }
    if tier.requires_paid && quota::paid_total(&state.pool, active.id).await? <= 0 {
        return Err(AppError::Coded {
            status: StatusCode::FORBIDDEN,
            code: "paid_required",
            message: "该档位需要先兑换额度才能使用".into(),
        });
    }

    fields.insert("model".into(), Value::String(tier.model.clone()));
    sqlx::query("UPDATE bindings SET last_used_at = now() WHERE id = $1").bind(active.id).execute(&state.pool).await?;

    let payload = serde_json::to_vec(&request).map_err(|e| AppError::Internal(e.to_string()))?;
    let upstream = state.newapi.chat_completions(&active.key, payload).await?;
    relay(upstream, &tier.tier).await
}

async fn relay(upstream: reqwest::Response, tier: &str) -> Result<Response, AppError> {
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
    if let Some(fields) = value.as_object_mut() {
        if fields.contains_key("model") {
            fields.insert("model".into(), Value::String(tier.to_string()));
        }
    }
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
    if status == StatusCode::BAD_REQUEST || status == StatusCode::PAYLOAD_TOO_LARGE {
        return AppError::Coded { status, code: "bad_request", message: "请求被拒绝，请检查参数后重试".into() };
    }
    AppError::BadGateway("AI 服务暂时不可用，请稍后再试".into())
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
    match serde_json::from_str::<Value>(payload) {
        Ok(Value::Object(mut fields)) if fields.contains_key("model") => {
            fields.insert("model".into(), Value::String(tier.to_string()));
            format!("data: {}", Value::Object(fields))
        }
        _ => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// 统一错误：响应体是 `{ "code": "…", "message": "…" }`，`code` 供客户端按机读处理。
#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    /// 带机读错误码的业务错误（如 code_expired）。
    #[error("{message}")]
    Coded { status: StatusCode, code: &'static str, message: String },
    #[error("{message}")]
    TooManyRequests { message: String, retry_after: i64 },
    #[error("{0}")]
    ServiceUnavailable(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn coded(status: StatusCode, code: &'static str, message: &str) -> Self {
        AppError::Coded { status, code, message: message.into() }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, retry_after) = match &self {
            AppError::Database(err) => {
                tracing::error!("database error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", "服务内部错误".to_string(), None)
            }
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.clone(), None),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "unauthorized", m.clone(), None),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "forbidden", m.clone(), None),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m.clone(), None),
            AppError::Coded { status, code, message } => (*status, *code, message.clone(), None),
            AppError::TooManyRequests { message, retry_after } => (StatusCode::TOO_MANY_REQUESTS, "rate_limited", message.clone(), Some(*retry_after)),
            AppError::ServiceUnavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable", m.clone(), None),
            AppError::Internal(m) => {
                tracing::error!("internal error: {m}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", "服务内部错误".to_string(), None)
            }
        };
        let mut body = json!({ "code": code, "message": message });
        if let Some(secs) = retry_after {
            body["retry_after"] = json!(secs);
        }
        let mut response = (status, Json(body)).into_response();
        if let Some(secs) = retry_after {
            if let Ok(v) = HeaderValue::from_str(&secs.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, v);
            }
        }
        response
    }
}

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

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
    #[error("{0}")]
    Conflict(String),
    #[error("{message}")]
    TooManyRequests { message: String, retry_after: i64 },
    #[error("{0}")]
    ServiceUnavailable(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn not_found(what: &str) -> Self {
        AppError::NotFound(format!("{what}不存在"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, retry_after) = match &self {
            AppError::Database(err) => {
                tracing::error!("database error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", "服务暂时不可用".to_string(), None)
            }
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.clone(), None),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "unauthorized", m.clone(), None),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "forbidden", m.clone(), None),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m.clone(), None),
            AppError::Conflict(m) => (StatusCode::CONFLICT, "conflict", m.clone(), None),
            AppError::TooManyRequests { message, retry_after } => {
                (StatusCode::TOO_MANY_REQUESTS, "too_many_requests", message.clone(), Some(*retry_after))
            }
            AppError::ServiceUnavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable", m.clone(), None),
            AppError::Internal(m) => {
                tracing::error!("internal error: {m}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", "服务暂时不可用".to_string(), None)
            }
        };

        let mut body = json!({ "error": true, "code": code, "message": message });
        if let Some(secs) = retry_after {
            body["retry_after"] = json!(secs);
        }
        let mut response = (status, Json(body)).into_response();
        if let Some(secs) = retry_after {
            if let Ok(value) = HeaderValue::from_str(&secs.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}

pub type AppResult<T> = Result<T, AppError>;

use axum::{
    http::StatusCode,
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
    /// 客户端需要据此分支处理的错误，`code` 是稳定的机读值（如 insufficient_quota）。
    #[error("{message}")]
    Coded { status: StatusCode, code: &'static str, message: String },
    #[error("{0}")]
    ServiceUnavailable(String),
    #[error("{0}")]
    BadGateway(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// 状态码、机读错误码和面向用户的文案。内部错误只说「暂时不可用」，细节写日志。
    pub fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            AppError::Database(err) => {
                tracing::error!("database error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", "服务暂时不可用".to_string())
            }
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.clone()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "unauthorized", m.clone()),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "forbidden", m.clone()),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, "conflict", m.clone()),
            AppError::Coded { status, code, message } => (*status, *code, message.clone()),
            AppError::ServiceUnavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable", m.clone()),
            AppError::BadGateway(m) => (StatusCode::BAD_GATEWAY, "bad_gateway", m.clone()),
            AppError::Internal(m) => {
                tracing::error!("internal error: {m}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", "服务暂时不可用".to_string())
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        (status, Json(json!({ "error": true, "code": code, "message": message }))).into_response()
    }
}

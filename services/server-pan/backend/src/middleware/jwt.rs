use crate::error::AppError;
use crate::models::Claims;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

/// 从 URI 查询串中提取 `token=` 参数值。
///
/// 用于兼容浏览器的 `<a href>` 直链下载 / 预览场景：
/// 浏览器发起这类请求时无法携带自定义 `Authorization` 头，
/// 因此允许通过 `?token=<JWT>` 传递登录凭证。
fn token_from_query(parts: &Parts) -> Option<String> {
    let query = parts.uri.query()?;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("token=") {
            if !value.is_empty() {
                return Some(percent_decode(value));
            }
        }
    }
    None
}

/// 极简百分号解码（JWT 由 base64url + `.` 组成，通常无需解码，
/// 但查询串中可能被 URL 编码，这里做一次安全还原）。
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// 共享的 JWT 密钥获取逻辑（支持从扩展中或环境变量读取）
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. 优先读取 Authorization: Bearer <token> 请求头
        let token_from_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.to_string());

        // 2. 回退到 ?token=<JWT> 查询参数（供 <a href> 直链下载 / 预览使用）
        let token = match token_from_header {
            Some(t) => t,
            None => token_from_query(parts)
                .ok_or_else(|| AppError::Unauthorized("缺少身份凭证（Authorization 头或 token 参数）".to_string()))?,
        };

        if token.is_empty() {
            return Err(AppError::Unauthorized("身份凭证为空".to_string()));
        }

        // 获取 JWT 密钥
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "pandown-super-secret-key-2026-apple-ui".to_string());

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("登录凭证无效或已过期: {}", e)))?;

        Ok(AuthUser(token_data.claims))
    }
}

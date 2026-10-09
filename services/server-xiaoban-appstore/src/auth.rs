use crate::{error::AppError, models::AdminClaims};
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn hash_password(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash: PasswordHash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    Argon2::default()
        .verify_password(password.as_bytes(), hash)
        .is_ok()
}

pub fn generate_token(username: &str, secret: &str) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .as_secs() as usize;

    // Token 有效期 7 天
    let exp = now + 7 * 24 * 3600;

    let claims = AdminClaims {
        sub: username.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {e}")))
}

pub fn verify_token(token: &str, secret: &str) -> Result<AdminClaims, AppError> {
    let token_data = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("登录令牌无效或已过期，请重新登录".to_string()))?;

    Ok(token_data.claims)
}

// Axum 提取器：自动解析 Authorization: Bearer <token>
pub struct AuthAdmin(pub AdminClaims);

impl<S> FromRequestParts<S> for AuthAdmin
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("缺少授权凭证".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("无效的授权凭证格式".to_string()));
        }

        let token = &auth_header[7..];
        // 从环境变量或默认配置读取 jwt_secret
        let secret = std::env::var("APPSTORE_JWT_SECRET")
            .unwrap_or_else(|_| "xiaoban-appstore-super-secret-key-2026-curated".to_string());

        let claims = verify_token(token, &secret)?;
        Ok(AuthAdmin(claims))
    }
}

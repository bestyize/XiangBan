use crate::{error::AppError, state::AppState};
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    /// 通过相伴账号登录时的账号 ID；每次请求都会核对绑定关系，解绑后立即失效。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acct: Option<String>,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash: PasswordHash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    Argon2::default().verify_password(password.as_bytes(), hash).is_ok()
}

pub fn issue_token(username: &str, secret: &str) -> Result<String, AppError> {
    issue_token_for(username, secret, 7 * 24 * 3600, None)
}

pub fn issue_token_for(username: &str, secret: &str, ttl_secs: usize, acct: Option<String>) -> Result<String, AppError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| AppError::Internal(e.to_string()))?.as_secs() as usize;
    let claims = Claims { sub: username.to_string(), exp: now + ttl_secs, acct };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("token: {e}")))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|d| d.claims)
        .map_err(|_| AppError::Unauthorized("登录已失效，请重新登录".into()))
}

/// 提取器：校验 `Authorization: Bearer <token>`。
pub struct Admin(pub Claims);

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("缺少登录凭证".into()))?;
        let claims = verify_token(value, &state.config.jwt_secret)?;
        if let Some(acct) = &claims.acct {
            let id = uuid::Uuid::parse_str(acct).map_err(|_| AppError::Unauthorized("登录已失效，请重新登录".into()))?;
            let bound: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM admin_accounts WHERE account_id = $1)").bind(id).fetch_one(&state.pool).await?;
            if !bound {
                return Err(AppError::Unauthorized("该相伴账号已不是后台管理员".into()));
            }
        }
        Ok(Admin(claims))
    }
}

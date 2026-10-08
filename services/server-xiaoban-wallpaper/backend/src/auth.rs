use crate::{
    error::{ApiError, Result},
    AppState,
};
use argon2::{
    password_hash::{PasswordHash, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap},
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::Row;

pub fn random_token() -> String {
    let mut b = [0u8; 32];
    rand::rng().fill_bytes(&mut b);
    hex::encode(b)
}
pub fn digest(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}
pub async fn hash_password(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(
                password.as_bytes(),
                &SaltString::generate(&mut rand_core::OsRng),
            )
            .map(|h| h.to_string())
            .map_err(ApiError::internal)
    })
    .await
    .map_err(ApiError::internal)?
}
pub async fn verify_password(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).is_ok_and(|h| {
            Argon2::default()
                .verify_password(password.as_bytes(), &h)
                .is_ok()
        })
    })
    .await
    .unwrap_or(false)
}
pub async fn initialize(state: &AppState) -> Result<()> {
    sqlx::migrate!()
        .run(&state.pool)
        .await
        .map_err(ApiError::internal)?;
    if sqlx::query("SELECT id FROM administrators WHERE id=1")
        .fetch_optional(&state.pool)
        .await?
        .is_none()
    {
        let hash = hash_password("123456".into()).await?;
        sqlx::query("INSERT INTO administrators(id,username,password_hash) VALUES(1,'admin',$1) ON CONFLICT(id) DO NOTHING").bind(hash).execute(&state.pool).await?;
    }
    Ok(())
}
#[derive(Clone)]
pub struct Admin {
    pub username: String,
    pub initial_password: bool,
    pub csrf: String,
    pub session_hash: String,
}
impl FromRequestParts<AppState> for Admin {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let token = parts
            .headers
            .get(header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|c| {
                c.split(';')
                    .find_map(|p| p.trim().strip_prefix("xiaoban_admin="))
            })
            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or_else(ApiError::unauthorized)?;
        let hash = digest(token);
        let row=sqlx::query("SELECT a.username,a.initial_password,s.csrf_token FROM admin_sessions s JOIN administrators a ON a.id=s.administrator_id WHERE s.token_hash=$1 AND s.expires_at>NOW()")
            .bind(&hash).fetch_optional(&state.pool).await?.ok_or_else(ApiError::unauthorized)?;
        Ok(Self {
            username: row.get("username"),
            initial_password: row.get("initial_password"),
            csrf: row.get("csrf_token"),
            session_hash: hash,
        })
    }
}
pub fn origin(headers: &HeaderMap, state: &AppState) -> Result<()> {
    if headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) == Some("cross-site") {
        return Err(ApiError(
            axum::http::StatusCode::FORBIDDEN,
            "请从本站操作".into(),
        ));
    }
    if let Some(value) = headers.get(header::ORIGIN) {
        if value.to_str().ok() != Some(state.config.public_origin.as_str()) {
            return Err(ApiError(
                axum::http::StatusCode::FORBIDDEN,
                "请求来源不匹配".into(),
            ));
        }
    }
    Ok(())
}
pub fn mutation(headers: &HeaderMap, admin: &Admin, state: &AppState) -> Result<()> {
    origin(headers, state)?;
    if headers.get("x-csrf-token").and_then(|v| v.to_str().ok()) != Some(admin.csrf.as_str()) {
        return Err(ApiError(
            axum::http::StatusCode::FORBIDDEN,
            "会话校验失败，请刷新后重试".into(),
        ));
    }
    Ok(())
}
pub fn cookie(token: &str, secure: bool, clear: bool) -> String {
    format!(
        "xiaoban_admin={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{}",
        if clear { 0 } else { 28800 },
        if secure { "; Secure" } else { "" }
    )
}

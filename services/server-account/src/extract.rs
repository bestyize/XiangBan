use crate::{
    admin_auth::{self, AdminIdentity},
    error::AppError,
    state::AppState,
    tokens,
};
use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{header, request::Parts},
};
use std::{convert::Infallible, net::SocketAddr};
use uuid::Uuid;

pub struct AuthUser {
    pub uid: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("缺少登录凭证".into()))?;
        let claims = tokens::verify_access(&state.keys, &state.config.issuer, token)?;
        let uid = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized("登录凭证无效".into()))?;
        Ok(AuthUser { uid })
    }
}

pub struct AuthAdmin(pub AdminIdentity);

impl FromRequestParts<AppState> for AuthAdmin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("缺少管理员凭证".into()))?;
        admin_auth::authenticate(state, token).await.map(AuthAdmin)
    }
}

/// 来源 IP：仅在 ACCOUNT_TRUST_PROXY=true 时信任反向代理设置的 X-Real-IP。
pub struct ClientIp(pub String);

impl FromRequestParts<AppState> for ClientIp {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        if state.config.trust_proxy {
            if let Some(ip) = parts.headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
                return Ok(ClientIp(ip.trim().chars().take(45).collect()));
            }
        }
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Ok(ClientIp(peer))
    }
}

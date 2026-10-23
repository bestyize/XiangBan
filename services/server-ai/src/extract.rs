use crate::{
    admin_auth::{self, AdminIdentity},
    error::AppError,
    products::{self, Product},
    state::AppState,
};
use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{header, request::Parts},
};
use std::{convert::Infallible, net::SocketAddr};
use uuid::Uuid;

fn bearer(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Anthropic 的 SDK 把密钥放在 `x-api-key` 头里。用户接口两种都认，`Authorization` 优先；管理员只认 `Authorization`。
fn bearer_or_api_key(parts: &Parts) -> Option<&str> {
    bearer(parts).or_else(|| {
        parts.headers.get("x-api-key").and_then(|v| v.to_str().ok()).map(str::trim).filter(|s| !s.is_empty())
    })
}

/// 账号中心登录用户，并由令牌 `aud` 解析出所属产品。客户端不属于任何已启用产品时拒绝。
pub struct AuthUser {
    pub uid: Uuid,
    pub product: Product,
}

impl AuthUser {
    /// Checks a login token from wherever it came (a header, or a WebSocket subprotocol for browsers).
    pub async fn from_token(state: &AppState, token: &str) -> Result<Self, AppError> {
        let verified = state.verifier.verify(token).await?;
        let product = products::for_client(&state.pool, &verified.client_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("该应用暂未开通 AI 功能".into()))?;
        Ok(AuthUser { uid: verified.uid, product })
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer_or_api_key(parts).ok_or_else(|| AppError::Unauthorized("请先登录".into()))?;
        AuthUser::from_token(state, token).await
    }
}

/// 来源 IP：仅在 AI_TRUST_PROXY=true 时信任反向代理设置的 X-Real-IP。
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

pub struct AuthAdmin(pub AdminIdentity);

impl FromRequestParts<AppState> for AuthAdmin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer(parts).ok_or_else(|| AppError::Unauthorized("缺少管理员凭证".into()))?;
        admin_auth::authenticate(state, token).await.map(AuthAdmin)
    }
}

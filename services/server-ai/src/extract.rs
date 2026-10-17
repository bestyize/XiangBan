use crate::{
    admin_auth::{self, AdminIdentity},
    error::AppError,
    products::{self, Product},
    state::AppState,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
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

/// 账号中心登录用户，并由令牌 `aud` 解析出所属产品。客户端不属于任何已启用产品时拒绝。
pub struct AuthUser {
    pub uid: Uuid,
    pub product: Product,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer(parts).ok_or_else(|| AppError::Unauthorized("请先登录".into()))?;
        let verified = state.verifier.verify(token).await?;
        let product = products::for_client(&state.pool, &verified.client_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("该应用暂未开通 AI 功能".into()))?;
        Ok(AuthUser { uid: verified.uid, product })
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

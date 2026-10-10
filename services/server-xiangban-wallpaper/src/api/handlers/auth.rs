use crate::{
    api::state::AppState,
    auth::AuthUser,
    error::AppError,
};
use axum::{extract::State, Json};
use serde_json::{json, Value};

/// GET /api/v1/auth/config
///
/// 返回统一账号中心的对接配置，供 Web 前端、桌面端和移动端确定认证源与 client_id。
pub async fn get_auth_config(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "success": true,
        "issuer": state.config.account_issuer,
        "jwks_url": state.config.resolved_jwks_url(),
        "clients": state.config.account_allowed_clients,
        "default_client": "xiangban-wallpaper-web",
    }))
}

/// GET /api/v1/user/me
///
/// 读取当前已登录用户的认证信息（需携带 Bearer 访问令牌）。
pub async fn get_current_user(
    user: AuthUser,
    State(_state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "success": true,
        "user": {
            "uid": user.uid.to_string(),
            "client_id": user.client_id,
        }
    })))
}

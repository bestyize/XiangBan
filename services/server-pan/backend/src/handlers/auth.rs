use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::{ChangePasswordRequest, LoginRequest, LoginResponse, UserDto};
use crate::services::AuthService;
use axum::{
    extract::State,
    Json,
};
use serde_json::{json, Value};


pub async fn login(
    State(auth_service): State<AuthService>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let res = auth_service.login(payload).await?;
    Ok(Json(res))
}

pub async fn get_me(
    auth: AuthUser,
    State(auth_service): State<AuthService>,
) -> AppResult<Json<UserDto>> {
    let user = auth_service.get_user_by_id(auth.0.user_id).await?;
    Ok(Json(user))
}

pub async fn change_password(
    auth: AuthUser,
    State(auth_service): State<AuthService>,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<Value>> {
    auth_service
        .change_password(auth.0.user_id, &payload.old_password, &payload.new_password)
        .await?;
    Ok(Json(json!({ "message": "密码修改成功" })))
}

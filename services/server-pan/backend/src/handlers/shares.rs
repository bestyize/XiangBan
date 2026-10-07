use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::{CreateShareRequest, CreateShareResponse, ShareDto, UpdateShareRequest};
use crate::services::ShareService;
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

pub async fn create_share(
    auth: AuthUser,
    State(share_service): State<ShareService>,
    Json(payload): Json<CreateShareRequest>,
) -> AppResult<Json<CreateShareResponse>> {
    let res = share_service.create_share(auth.0.user_id, payload).await?;
    Ok(Json(res))
}

pub async fn list_my_shares(
    auth: AuthUser,
    State(share_service): State<ShareService>,
) -> AppResult<Json<Vec<ShareDto>>> {
    let list = share_service.list_my_shares(auth.0.user_id).await?;
    Ok(Json(list))
}

pub async fn update_share(
    auth: AuthUser,
    State(share_service): State<ShareService>,
    Path(code): Path<String>,
    Json(payload): Json<UpdateShareRequest>,
) -> AppResult<Json<Value>> {
    share_service
        .update_share(auth.0.user_id, &code, payload)
        .await?;
    Ok(Json(json!({ "message": "分享已更新" })))
}

pub async fn delete_share(
    auth: AuthUser,
    State(share_service): State<ShareService>,
    Path(code): Path<String>,
) -> AppResult<Json<Value>> {
    share_service.delete_share(auth.0.user_id, &code).await?;
    Ok(Json(json!({ "message": "分享已取消" })))
}

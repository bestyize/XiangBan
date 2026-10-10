use crate::{
    api::state::AppState,
    auth::OptionalAuthUser,
    error::AppError,
    models::WallpaperItem,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct CheckFavoritesQuery {
    pub ids: String,
}

/// GET /api/v1/favorites
pub async fn list_favorites(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.0.as_ref().map(|u| u.uid);
    let items = state.db.list_favorites(user_id).await?;
    Ok(Json(json!({
        "success": true,
        "authenticated": auth.0.is_some(),
        "user_id": auth.0.as_ref().map(|u| u.uid.to_string()),
        "total": items.len(),
        "items": items
    })))
}

/// POST /api/v1/favorites
pub async fn add_favorite(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Json(mut item): Json<WallpaperItem>,
) -> Result<Json<Value>, AppError> {
    // This route is public: the stored-copy record is written by the server only.
    item.local_copy = None;
    let user_id = auth.0.as_ref().map(|u| u.uid);
    state.db.add_favorite(user_id, &item).await?;
    Ok(Json(json!({
        "success": true,
        "authenticated": auth.0.is_some(),
        "user_id": auth.0.as_ref().map(|u| u.uid.to_string()),
        "message": "Wallpaper added to favorites",
        "id": item.id
    })))
}

/// DELETE /api/v1/favorites/:id
pub async fn remove_favorite(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.0.as_ref().map(|u| u.uid);
    let removed = state.db.remove_favorite(user_id, &id).await?;
    Ok(Json(json!({
        "success": true,
        "authenticated": auth.0.is_some(),
        "user_id": auth.0.as_ref().map(|u| u.uid.to_string()),
        "removed": removed,
        "id": id
    })))
}

/// GET /api/v1/favorites/check?ids=id1,id2,id3
pub async fn check_favorites(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Query(query): Query<CheckFavoritesQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.0.as_ref().map(|u| u.uid);
    let ids: Vec<String> = query
        .ids
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let favorite_ids = state.db.check_favorites(user_id, &ids).await?;
    Ok(Json(json!({
        "success": true,
        "authenticated": auth.0.is_some(),
        "favorite_ids": favorite_ids
    })))
}

/// GET /api/v1/favorites/:id/status
pub async fn is_favorite(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.0.as_ref().map(|u| u.uid);
    let is_fav = state.db.is_favorite(user_id, &id).await?;
    Ok(Json(json!({
        "success": true,
        "authenticated": auth.0.is_some(),
        "is_favorite": is_fav,
        "id": id
    })))
}

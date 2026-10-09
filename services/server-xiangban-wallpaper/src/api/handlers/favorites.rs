use crate::{api::state::AppState, error::AppError, models::WallpaperItem};
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

/// GET /api/v1/favorites
pub async fn list_favorites(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let items = state.db.list_favorites().await?;
    Ok(Json(json!({
        "success": true,
        "total": items.len(),
        "items": items
    })))
}

/// POST /api/v1/favorites
pub async fn add_favorite(
    State(state): State<AppState>,
    Json(mut item): Json<WallpaperItem>,
) -> Result<Json<Value>, AppError> {
    // This route is public: the stored-copy record is written by the server only.
    item.local_copy = None;
    state.db.add_favorite(&item).await?;
    Ok(Json(json!({
        "success": true,
        "message": "Wallpaper added to favorites",
        "id": item.id
    })))
}

/// DELETE /api/v1/favorites/:id
pub async fn remove_favorite(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let removed = state.db.remove_favorite(&id).await?;
    Ok(Json(json!({
        "success": true,
        "removed": removed,
        "id": id
    })))
}

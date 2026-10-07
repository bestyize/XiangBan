use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::StorageStats;
use crate::services::FileService;
use axum::{extract::State, Json};

pub async fn get_stats(
    auth: AuthUser,
    State(file_service): State<FileService>,
) -> AppResult<Json<StorageStats>> {
    let stats = file_service.get_stats(auth.0.user_id).await?;
    Ok(Json(stats))
}

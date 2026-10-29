use crate::{api::state::AppState, error::AppError, services::SyncReport};
use axum::{extract::State, Json};
use serde_json::{json, Value};

/// POST /api/v1/collector/sync
pub async fn trigger_collector_sync(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let report: SyncReport = state.collector.run_sync().await?;

    Ok(Json(json!({
        "success": true,
        "message": "Wallpaper collection task finished",
        "report": report
    })))
}

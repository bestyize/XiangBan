use crate::{api::state::AppState, error::AppError};
use axum::{extract::State, Json};
use serde_json::{json, Value};

/// GET /api/v1/providers
pub async fn list_providers(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let providers = state.registry.list_info();
    let configured_count = providers.iter().filter(|p| p.is_configured).count();

    Ok(Json(json!({
        "success": true,
        "total": providers.len(),
        "configured_count": configured_count,
        "providers": providers
    })))
}

use crate::api::state::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};

/// GET /api/v1/health
pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let providers = state.registry.list_info();
    let configured_count = providers.iter().filter(|p| p.is_configured).count();

    Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "database": {
            "postgres_connected": state.db.is_postgres_connected(),
            "mode": if state.db.is_postgres_connected() { "PostgreSQL" } else { "In-Memory Fallback" }
        },
        "providers": {
            "total": providers.len(),
            "configured": configured_count
        }
    }))
}

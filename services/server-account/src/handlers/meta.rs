use crate::{state::AppState, tokens};
use axum::{extract::State, Json};
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "server-account", "version": env!("CARGO_PKG_VERSION") }))
}

pub async fn jwks(State(state): State<AppState>) -> Json<Value> {
    Json(tokens::jwks(&state.keys))
}

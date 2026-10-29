//! 同一账号下的对端设备列表：手机看到自己的电脑，电脑看到自己的手机。
use super::{counterparts, owned_device};
use crate::{auth::AuthUser, error::AppError, state::AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct PeerQuery {
    pub device_id: String,
}

pub async fn list(State(state): State<AppState>, AuthUser(uid): AuthUser, Query(q): Query<PeerQuery>) -> Result<Json<Value>, AppError> {
    let me = owned_device(&state.pool, uid, &q.device_id, None).await?;
    let items: Vec<Value> = counterparts(&state.pool, uid, &me.kind)
        .await?
        .iter()
        .map(|d| json!({ "id": d.id, "kind": d.kind, "name": d.name, "online": state.hub.is_online(&d.id) }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

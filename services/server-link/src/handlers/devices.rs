use super::{counterparts, owned_device, valid_install_id, Device, DEVICE_COLUMNS};
use crate::{auth::AuthUser, error::AppError, frames, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct NewDevice {
    pub kind: String,
    pub name: String,
    /// 客户端首次运行时随机生成并保存的编号（16–64 位字母数字、`-`、`_`）。
    pub install_id: String,
}

fn new_id(kind: &str) -> String {
    let prefix = if kind == "desktop" { "dsk_" } else { "phn_" };
    format!("{prefix}{}", &uuid::Uuid::new_v4().simple().to_string()[..20])
}

/// 通知账号下已在线的对端：多了一台设备（或名称变了），并让它们之间可以互发消息。
async fn announce(state: &AppState, device: &Device) {
    if let Ok(peers) = counterparts(&state.pool, device.uid, &device.kind).await {
        for peer in peers {
            state.hub.add_pair(&device.id, &peer.id);
            state.hub.send(&peer.id, frames::peer_added(&json!({ "id": device.id, "kind": device.kind, "name": device.name, "online": state.hub.is_online(&device.id) })));
        }
    }
}

/// 登记设备。同一账号同一安装编号重复登记返回已有的设备（幂等），名称会更新。
pub async fn create(State(state): State<AppState>, AuthUser(uid): AuthUser, Json(body): Json<NewDevice>) -> Result<(StatusCode, Json<Value>), AppError> {
    if body.kind != "desktop" && body.kind != "phone" {
        return Err(AppError::BadRequest("kind 只能是 desktop 或 phone".into()));
    }
    let name = body.name.trim();
    if name.is_empty() || name.chars().count() > 40 {
        return Err(AppError::BadRequest("设备名称需要 1–40 个字".into()));
    }
    if !valid_install_id(&body.install_id) {
        return Err(AppError::BadRequest("install_id 需要 16–64 位字母、数字、- 或 _".into()));
    }
    let existing: Option<Device> = sqlx::query_as(&format!("SELECT {DEVICE_COLUMNS} FROM devices WHERE uid = $1 AND install_id = $2 AND revoked_at IS NULL"))
        .bind(uid)
        .bind(&body.install_id)
        .fetch_optional(&state.pool)
        .await?;
    if let Some(mut device) = existing {
        if device.kind != body.kind {
            return Err(AppError::BadRequest("这个安装编号已用于另一种设备".into()));
        }
        sqlx::query("UPDATE devices SET name = $2 WHERE id = $1").bind(&device.id).bind(name).execute(&state.pool).await?;
        device.name = name.to_string();
        announce(&state, &device).await;
        return Ok((StatusCode::OK, Json(json!({ "device_id": device.id, "created": false }))));
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM devices WHERE uid = $1 AND revoked_at IS NULL").bind(uid).fetch_one(&state.pool).await?;
    if count >= state.config.max_devices_per_user {
        return Err(AppError::coded(StatusCode::CONFLICT, "too_many_devices", "设备数量已达上限，请先移除不用的设备"));
    }
    let id = new_id(&body.kind);
    let device: Device = sqlx::query_as(&format!("INSERT INTO devices (id, uid, kind, name, install_id) VALUES ($1, $2, $3, $4, $5) RETURNING {DEVICE_COLUMNS}"))
        .bind(&id)
        .bind(uid)
        .bind(&body.kind)
        .bind(name)
        .bind(&body.install_id)
        .fetch_one(&state.pool)
        .await?;
    announce(&state, &device).await;
    Ok((StatusCode::CREATED, Json(json!({ "device_id": id, "created": true }))))
}

pub async fn list(State(state): State<AppState>, AuthUser(uid): AuthUser) -> Result<Json<Value>, AppError> {
    let devices: Vec<Device> = sqlx::query_as(&format!("SELECT {DEVICE_COLUMNS} FROM devices WHERE uid = $1 AND revoked_at IS NULL ORDER BY created_at"))
        .bind(uid)
        .fetch_all(&state.pool)
        .await?;
    let items: Vec<Value> = devices
        .iter()
        .map(|d| json!({ "id": d.id, "kind": d.kind, "name": d.name, "created_at": d.created_at, "online": state.hub.is_online(&d.id) }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

/// 撤销设备：它从对端的列表里消失，它自己的连接被断开（客户端下次启动会重新登记成新设备）。
pub async fn remove(State(state): State<AppState>, AuthUser(uid): AuthUser, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let device = owned_device(&state.pool, uid, &id, None).await?;
    let peers = counterparts(&state.pool, uid, &device.kind).await?;
    sqlx::query("UPDATE devices SET revoked_at = now() WHERE id = $1").bind(&device.id).execute(&state.pool).await?;
    for peer in &peers {
        state.hub.remove_pair(&device.id, &peer.id);
        state.hub.send(&peer.id, frames::peer_removed(&device.id));
    }
    if let Some(conn) = state.hub.drop_device(&device.id) {
        conn.close_with(frames::error("device_revoked"));
    }
    Ok(StatusCode::NO_CONTENT)
}

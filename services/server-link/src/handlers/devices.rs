use super::{owned_device, valid_public_key, Device};
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
    pub public_key: String,
}

fn new_id(kind: &str) -> String {
    let prefix = if kind == "desktop" { "dsk_" } else { "phn_" };
    format!("{prefix}{}", &uuid::Uuid::new_v4().simple().to_string()[..20])
}

/// 登记设备。同一账号同一公钥重复登记返回已有的设备（幂等），名称会更新。
pub async fn create(State(state): State<AppState>, AuthUser(uid): AuthUser, Json(body): Json<NewDevice>) -> Result<(StatusCode, Json<Value>), AppError> {
    if body.kind != "desktop" && body.kind != "phone" {
        return Err(AppError::BadRequest("kind 只能是 desktop 或 phone".into()));
    }
    let name = body.name.trim();
    if name.is_empty() || name.chars().count() > 40 {
        return Err(AppError::BadRequest("设备名称需要 1–40 个字".into()));
    }
    if !valid_public_key(&body.public_key) {
        return Err(AppError::BadRequest("公钥格式不对（需要 base64url 的 65 字节 P-256 未压缩点）".into()));
    }
    let existing: Option<Device> = sqlx::query_as(
        "SELECT id, uid, kind, name, public_key, created_at FROM devices WHERE uid = $1 AND public_key = $2 AND revoked_at IS NULL",
    )
    .bind(uid)
    .bind(&body.public_key)
    .fetch_optional(&state.pool)
    .await?;
    if let Some(device) = existing {
        if device.kind != body.kind {
            return Err(AppError::BadRequest("这把公钥已用于另一种设备".into()));
        }
        sqlx::query("UPDATE devices SET name = $2 WHERE id = $1").bind(&device.id).bind(name).execute(&state.pool).await?;
        return Ok((StatusCode::OK, Json(json!({ "device_id": device.id, "created": false }))));
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM devices WHERE uid = $1 AND revoked_at IS NULL").bind(uid).fetch_one(&state.pool).await?;
    if count >= state.config.max_devices_per_user {
        return Err(AppError::coded(StatusCode::CONFLICT, "too_many_devices", "设备数量已达上限，请先移除不用的设备"));
    }
    let id = new_id(&body.kind);
    sqlx::query("INSERT INTO devices (id, uid, kind, name, public_key) VALUES ($1, $2, $3, $4, $5)")
        .bind(&id)
        .bind(uid)
        .bind(&body.kind)
        .bind(name)
        .bind(&body.public_key)
        .execute(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({ "device_id": id, "created": true }))))
}

pub async fn list(State(state): State<AppState>, AuthUser(uid): AuthUser) -> Result<Json<Value>, AppError> {
    let devices: Vec<Device> = sqlx::query_as(
        "SELECT id, uid, kind, name, public_key, created_at FROM devices WHERE uid = $1 AND revoked_at IS NULL ORDER BY created_at",
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = devices
        .iter()
        .map(|d| json!({ "id": d.id, "kind": d.kind, "name": d.name, "public_key": d.public_key, "created_at": d.created_at, "online": state.hub.is_online(&d.id) }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

/// 撤销设备：它的所有配对一并解除，对端收到 `unpaired`，它自己的连接被断开。
pub async fn remove(State(state): State<AppState>, AuthUser(uid): AuthUser, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let device = owned_device(&state.pool, uid, &id, None).await?;
    let peers: Vec<String> = sqlx::query_scalar(
        "SELECT CASE WHEN desktop_id = $1 THEN phone_id ELSE desktop_id END FROM pairings
         WHERE (desktop_id = $1 OR phone_id = $1) AND status IN ('active', 'pending')",
    )
    .bind(&device.id)
    .fetch_all(&state.pool)
    .await?;
    sqlx::query("UPDATE pairings SET status = 'revoked', revoked_at = now() WHERE (desktop_id = $1 OR phone_id = $1) AND status IN ('active', 'pending')")
        .bind(&device.id)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE devices SET revoked_at = now() WHERE id = $1").bind(&device.id).execute(&state.pool).await?;
    for peer in &peers {
        state.hub.remove_pair(&device.id, peer);
        state.hub.send(peer, frames::unpaired(&device.id));
        state.hub.send(peer, frames::presence(&device.id, false));
    }
    if let Some(conn) = state.hub.drop_device(&device.id) {
        conn.close_with(frames::error("device_revoked"));
    }
    Ok(StatusCode::NO_CONTENT)
}

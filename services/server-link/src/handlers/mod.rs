pub mod devices;
pub mod pairing;

use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Clone, Debug)]
pub struct Device {
    pub id: String,
    #[allow(dead_code)]
    pub uid: Uuid,
    pub kind: String,
    pub name: String,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
}

impl Device {
    /// 发给对端的设备信息（含公钥，供端到端加密派生密钥）。
    pub fn peer_json(&self) -> Value {
        json!({ "id": self.id, "kind": self.kind, "name": self.name, "public_key": self.public_key })
    }
}

/// 读取自己名下、未撤销的设备；别人的或不存在的一律当作不存在，避免被用来探测设备编号。
pub async fn owned_device(pool: &PgPool, uid: Uuid, id: &str, kind: Option<&str>) -> Result<Device, AppError> {
    let device: Option<Device> = sqlx::query_as(
        "SELECT id, uid, kind, name, public_key, created_at FROM devices WHERE id = $1 AND uid = $2 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(uid)
    .fetch_optional(pool)
    .await?;
    let device = device.ok_or_else(|| AppError::NotFound("设备不存在".into()))?;
    if kind.is_some_and(|k| k != device.kind) {
        return Err(AppError::BadRequest(format!("这个接口只能用于{}设备", if kind == Some("desktop") { "桌面" } else { "手机" })));
    }
    Ok(device)
}

/// 公钥必须是 base64url 的 65 字节 SEC1 未压缩 P-256 点。
pub fn valid_public_key(text: &str) -> bool {
    B64.decode(text).is_ok_and(|b| b.len() == 65 && b[0] == 4)
}

#[derive(Serialize)]
pub struct Health {
    pub ok: bool,
    pub online: usize,
}

pub async fn health(State(state): State<AppState>) -> Result<Json<Health>, AppError> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(Health { ok: true, online: state.hub.online_count() }))
}

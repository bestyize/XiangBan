pub mod devices;
pub mod peers;

use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
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
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

pub const DEVICE_COLUMNS: &str = "id, uid, kind, name, created_at";

impl Device {
    /// 发给对端看的设备信息。
    pub fn peer_json(&self) -> Value {
        json!({ "id": self.id, "kind": self.kind, "name": self.name })
    }
}

/// 读取自己名下、未撤销的设备；别人的或不存在的一律当作不存在，避免被用来探测设备编号。
pub async fn owned_device(pool: &PgPool, uid: Uuid, id: &str, kind: Option<&str>) -> Result<Device, AppError> {
    let device: Option<Device> = sqlx::query_as(&format!("SELECT {DEVICE_COLUMNS} FROM devices WHERE id = $1 AND uid = $2 AND revoked_at IS NULL"))
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

/// 同一账号下、与 `kind` 相反类型的设备：手机的对端是电脑，电脑的对端是手机。
pub async fn counterparts(pool: &PgPool, uid: Uuid, kind: &str) -> Result<Vec<Device>, AppError> {
    let other = if kind == "desktop" { "phone" } else { "desktop" };
    Ok(sqlx::query_as(&format!("SELECT {DEVICE_COLUMNS} FROM devices WHERE uid = $1 AND kind = $2 AND revoked_at IS NULL ORDER BY created_at"))
        .bind(uid)
        .bind(other)
        .fetch_all(pool)
        .await?)
}

/// 安装编号：客户端首次运行时随机生成并保存，用来让同一台设备重复登记时拿到同一个设备编号。
pub fn valid_install_id(text: &str) -> bool {
    (16..=64).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
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

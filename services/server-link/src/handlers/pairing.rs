//! 配对：桌面申请一次性配对码 → 手机扫码兑换 → 桌面验证证明后确认。
//! 中转**不验证**手机的证明（它不知道二维码里的一次性密钥），只负责原样转给桌面。
use super::{owned_device, valid_public_key, Device};
use crate::{auth::AuthUser, error::AppError, frames, state::AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Duration;
use uuid::Uuid;

const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

fn hash_code(code: &str) -> String {
    let normalized: String = code.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase();
    Sha256::digest(normalized.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn too_many(wait: i64) -> AppError {
    AppError::TooManyRequests { message: "操作太频繁，请稍后再试".into(), retry_after: wait }
}

#[derive(Deserialize)]
pub struct NewCode {
    pub device_id: String,
}

pub async fn create_code(State(state): State<AppState>, AuthUser(uid): AuthUser, Json(body): Json<NewCode>) -> Result<Json<Value>, AppError> {
    let desktop = owned_device(&state.pool, uid, &body.device_id, Some("desktop")).await?;
    if let Some(wait) = state.limiter.hit(&format!("code:{}", desktop.id), state.config.pair_codes_per_hour, Duration::from_secs(3600)) {
        return Err(too_many(wait));
    }
    let code: String = {
        let mut rng = rand::thread_rng();
        (0..8).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect()
    };
    let expires_at = Utc::now() + chrono::Duration::from_std(state.config.pair_code_ttl).unwrap_or_else(|_| chrono::Duration::minutes(5));
    // 同一台桌面同一时刻只有一个有效的码，新的顶掉旧的。
    sqlx::query("DELETE FROM pair_codes WHERE desktop_id = $1 AND used_at IS NULL").bind(&desktop.id).execute(&state.pool).await?;
    sqlx::query("INSERT INTO pair_codes (code_hash, desktop_id, uid, expires_at) VALUES ($1, $2, $3, $4)")
        .bind(hash_code(&code))
        .bind(&desktop.id)
        .bind(uid)
        .bind(expires_at)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "code": code, "expires_at": expires_at })))
}

#[derive(Deserialize)]
pub struct Redeem {
    pub code: String,
    pub device_id: String,
    pub proof: String,
}

pub async fn redeem(State(state): State<AppState>, AuthUser(uid): AuthUser, Json(body): Json<Redeem>) -> Result<(StatusCode, Json<Value>), AppError> {
    let phone = owned_device(&state.pool, uid, &body.device_id, Some("phone")).await?;
    if let Some(wait) = state.limiter.hit(&format!("redeem:{uid}"), state.config.pair_redeems_per_minute, Duration::from_secs(60)) {
        return Err(too_many(wait));
    }
    if B64.decode(&body.proof).map(|b| b.len()) != Ok(32) {
        return Err(AppError::BadRequest("证明格式不对".into()));
    }
    let hash = hash_code(&body.code);
    // 原子地占用一次兑换名额：必须同账号、未使用、未过期、没超过兑换次数。
    let desktop_id: Option<String> = sqlx::query_scalar(
        "UPDATE pair_codes SET redeems = redeems + 1
         WHERE code_hash = $1 AND uid = $2 AND used_at IS NULL AND expires_at > now() AND redeems < $3
         RETURNING desktop_id",
    )
    .bind(&hash)
    .bind(uid)
    .bind(state.config.pair_code_max_redeems)
    .fetch_optional(&state.pool)
    .await?;
    let Some(desktop_id) = desktop_id else {
        let row: Option<(bool, bool, i32)> = sqlx::query_as("SELECT used_at IS NOT NULL, expires_at <= now(), redeems FROM pair_codes WHERE code_hash = $1 AND uid = $2")
            .bind(&hash)
            .bind(uid)
            .fetch_optional(&state.pool)
            .await?;
        return Err(match row {
            Some((true, _, _)) => AppError::coded(StatusCode::GONE, "code_used", "配对码已经用过了，请在电脑上重新生成"),
            Some((_, false, n)) if n >= state.config.pair_code_max_redeems => AppError::coded(StatusCode::TOO_MANY_REQUESTS, "too_many_attempts", "尝试次数过多，配对码已作废，请在电脑上重新生成"),
            // 过期、不存在、不是这个账号的码，对外是同一个说法，不给探测码是否存在的机会。
            _ => AppError::coded(StatusCode::GONE, "code_expired", "配对码无效或已过期，请在电脑上重新生成"),
        });
    };
    let desktop: Device = sqlx::query_as("SELECT id, uid, kind, name, public_key, created_at FROM devices WHERE id = $1 AND revoked_at IS NULL")
        .bind(&desktop_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::coded(StatusCode::GONE, "code_expired", "配对码无效或已过期，请在电脑上重新生成"))?;
    if !valid_public_key(&phone.public_key) {
        return Err(AppError::Internal("设备公钥异常".into()));
    }
    // 同一对设备同时只留一个待确认的请求。
    sqlx::query("UPDATE pairings SET status = 'revoked', revoked_at = now() WHERE desktop_id = $1 AND phone_id = $2 AND status = 'pending'")
        .bind(&desktop.id)
        .bind(&phone.id)
        .execute(&state.pool)
        .await?;
    let pairing_id = Uuid::new_v4();
    sqlx::query("INSERT INTO pairings (id, uid, desktop_id, phone_id, code_hash, status, phone_proof) VALUES ($1, $2, $3, $4, $5, 'pending', $6)")
        .bind(pairing_id)
        .bind(uid)
        .bind(&desktop.id)
        .bind(&phone.id)
        .bind(&hash)
        .bind(&body.proof)
        .execute(&state.pool)
        .await?;
    state.hub.send(&desktop.id, frames::pair_request(&pairing_id.to_string(), &phone.peer_json(), &body.proof));
    Ok((StatusCode::ACCEPTED, Json(json!({ "pairing_id": pairing_id, "desktop": desktop.peer_json() }))))
}

#[derive(sqlx::FromRow)]
struct PairingRow {
    id: Uuid,
    desktop_id: String,
    phone_id: String,
    code_hash: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
}

async fn load_pairing(state: &AppState, uid: Uuid, id: Uuid) -> Result<PairingRow, AppError> {
    sqlx::query_as("SELECT id, desktop_id, phone_id, code_hash, status, created_at FROM pairings WHERE id = $1 AND uid = $2")
        .bind(id)
        .bind(uid)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("配对不存在".into()))
}

async fn device_by_id(state: &AppState, id: &str) -> Result<Device, AppError> {
    sqlx::query_as("SELECT id, uid, kind, name, public_key, created_at FROM devices WHERE id = $1 AND revoked_at IS NULL")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".into()))
}

/// 桌面确认（它已经验证过手机的证明）。待确认的请求超过配对码有效期就不能再确认。
pub async fn confirm(State(state): State<AppState>, AuthUser(uid): AuthUser, Path(id): Path<Uuid>) -> Result<Json<Value>, AppError> {
    let p = load_pairing(&state, uid, id).await?;
    if p.status != "pending" {
        return Err(AppError::coded(StatusCode::CONFLICT, "not_pending", "这个配对请求已经处理过了"));
    }
    if Utc::now() - p.created_at > chrono::Duration::from_std(state.config.pair_code_ttl).unwrap_or_else(|_| chrono::Duration::minutes(5)) {
        return Err(AppError::coded(StatusCode::GONE, "pairing_expired", "配对请求已过期，请重新扫码"));
    }
    let desktop = device_by_id(&state, &p.desktop_id).await?;
    let phone = device_by_id(&state, &p.phone_id).await?;
    let mut tx = state.pool.begin().await?;
    // 先到先得：只有仍是 pending 的才能转成 active。
    sqlx::query("UPDATE pairings SET status = 'revoked', revoked_at = now() WHERE desktop_id = $1 AND phone_id = $2 AND status = 'active'")
        .bind(&p.desktop_id)
        .bind(&p.phone_id)
        .execute(&mut *tx)
        .await?;
    let activated = sqlx::query("UPDATE pairings SET status = 'active', activated_at = now() WHERE id = $1 AND status = 'pending'").bind(p.id).execute(&mut *tx).await?;
    if activated.rows_affected() == 0 {
        return Err(AppError::coded(StatusCode::CONFLICT, "not_pending", "这个配对请求已经处理过了"));
    }
    if let Some(hash) = &p.code_hash {
        sqlx::query("UPDATE pair_codes SET used_at = now() WHERE code_hash = $1").bind(hash).execute(&mut *tx).await?;
    }
    tx.commit().await?;

    state.hub.add_pair(&desktop.id, &phone.id);
    state.hub.send(&phone.id, frames::pair_confirmed(&p.id.to_string(), &desktop.peer_json()));
    state.hub.send(&desktop.id, frames::presence(&phone.id, state.hub.is_online(&phone.id)));
    state.hub.send(&phone.id, frames::presence(&desktop.id, state.hub.is_online(&desktop.id)));
    Ok(Json(json!({ "pairing_id": p.id, "phone": phone.peer_json() })))
}

pub async fn reject(State(state): State<AppState>, AuthUser(uid): AuthUser, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    let p = load_pairing(&state, uid, id).await?;
    let done = sqlx::query("UPDATE pairings SET status = 'rejected', revoked_at = now() WHERE id = $1 AND status = 'pending'").bind(p.id).execute(&state.pool).await?;
    if done.rows_affected() == 0 {
        return Err(AppError::coded(StatusCode::CONFLICT, "not_pending", "这个配对请求已经处理过了"));
    }
    state.hub.send(&p.phone_id, frames::pair_rejected(&p.id.to_string()));
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub device_id: String,
    pub status: Option<String>,
}

pub async fn list(State(state): State<AppState>, AuthUser(uid): AuthUser, Query(q): Query<ListQuery>) -> Result<Json<Value>, AppError> {
    let device = owned_device(&state.pool, uid, &q.device_id, None).await?;
    let status = q.status.as_deref().unwrap_or("active");
    if status != "active" && status != "pending" {
        return Err(AppError::BadRequest("status 只能是 active 或 pending".into()));
    }
    let rows: Vec<PairingRow> = sqlx::query_as(
        "SELECT id, desktop_id, phone_id, code_hash, status, created_at FROM pairings
         WHERE (desktop_id = $1 OR phone_id = $1) AND status = $2 ORDER BY created_at",
    )
    .bind(&device.id)
    .bind(status)
    .fetch_all(&state.pool)
    .await?;
    let mut items = Vec::new();
    for r in rows {
        let peer_id = if r.desktop_id == device.id { &r.phone_id } else { &r.desktop_id };
        let peer = device_by_id(&state, peer_id).await?;
        let mut item = json!({ "pairing_id": r.id, "status": r.status, "created_at": r.created_at, "peer": peer.peer_json(), "online": state.hub.is_online(&peer.id) });
        // 桌面取待确认请求时要带上手机的证明，用来验证。
        if status == "pending" && device.kind == "desktop" {
            let proof: String = sqlx::query_scalar("SELECT phone_proof FROM pairings WHERE id = $1").bind(r.id).fetch_one(&state.pool).await?;
            item["proof"] = json!(proof);
        }
        items.push(item);
    }
    Ok(Json(json!({ "items": items })))
}

/// 解除配对（或取消待确认的请求）。账号下任一端都可以操作，两端设备都会收到 `unpaired`。
pub async fn remove(State(state): State<AppState>, AuthUser(uid): AuthUser, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    let p = load_pairing(&state, uid, id).await?;
    let done = sqlx::query("UPDATE pairings SET status = 'revoked', revoked_at = now() WHERE id = $1 AND status IN ('active', 'pending')").bind(p.id).execute(&state.pool).await?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound("配对不存在".into()));
    }
    state.hub.remove_pair(&p.desktop_id, &p.phone_id);
    state.hub.send(&p.desktop_id, frames::unpaired(&p.phone_id));
    state.hub.send(&p.phone_id, frames::unpaired(&p.desktop_id));
    Ok(StatusCode::NO_CONTENT)
}

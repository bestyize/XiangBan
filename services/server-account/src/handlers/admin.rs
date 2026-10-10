use crate::{
    admin_auth::{self, AdminIdentity},
    error::AppError,
    extract::AuthAdmin,
    identifier::{self, Identifier, Kind},
    password, sessions,
    state::AppState,
    users::{self, IdentityView},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

const MAX_ADMIN_FAILURES: i32 = 5;
const BAD_ADMIN_CREDENTIALS: &str = "用户名或密码错误";

async fn audit(state: &AppState, admin: &AdminIdentity, action: &str, target: Option<&str>, detail: Option<String>) -> Result<(), AppError> {
    sqlx::query("INSERT INTO admin_audit (admin_username, action, target, detail) VALUES ($1, $2, $3, $4)")
        .bind(&admin.username)
        .bind(action)
        .bind(target)
        .bind(detail)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct LoginReq {
    username: String,
    password: String,
}

pub async fn login(State(state): State<AppState>, Json(req): Json<LoginReq>) -> Result<Json<Value>, AppError> {
    let username = req.username.trim().to_lowercase();
    let row: Option<(Uuid, String, i32, bool)> = sqlx::query_as(
        "SELECT id, password_hash, token_version, COALESCE(locked_until > now(), false) FROM admins WHERE username = $1",
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, hash, token_version, locked)) = row else {
        password::verify(req.password, state.dummy_password_hash.to_string()).await;
        return Err(AppError::Unauthorized(BAD_ADMIN_CREDENTIALS.into()));
    };
    if locked {
        return Err(AppError::TooManyRequests { message: "密码错误次数过多，请 15 分钟后再试".into(), retry_after: 900 });
    }
    if !password::verify(req.password, hash).await {
        sqlx::query(
            "UPDATE admins SET
                locked_until = CASE WHEN failed_logins + 1 >= $2 THEN now() + interval '15 minutes' ELSE locked_until END,
                failed_logins = CASE WHEN failed_logins + 1 >= $2 THEN 0 ELSE failed_logins + 1 END
             WHERE id = $1",
        )
        .bind(id)
        .bind(MAX_ADMIN_FAILURES)
        .execute(&state.pool)
        .await?;
        return Err(AppError::Unauthorized(BAD_ADMIN_CREDENTIALS.into()));
    }

    sqlx::query("UPDATE admins SET last_login_at = now(), failed_logins = 0, locked_until = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    let ttl = state.config.admin_token_ttl_secs;
    let token = admin_auth::issue(&state.keys.sub.admin_jwt_secret, id, token_version, ttl)?;
    Ok(Json(json!({ "token": token, "expires_in": ttl, "admin": { "username": username } })))
}

pub async fn me(AuthAdmin(admin): AuthAdmin) -> Json<Value> {
    Json(json!({ "username": admin.username }))
}

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    current_password: String,
    new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(req): Json<ChangePasswordReq>,
) -> Result<Json<Value>, AppError> {
    password::validate(&req.new_password)?;
    let (hash,): (String,) = sqlx::query_as("SELECT password_hash FROM admins WHERE id = $1")
        .bind(admin.id)
        .fetch_one(&state.pool)
        .await?;
    if !password::verify(req.current_password, hash).await {
        return Err(AppError::Unauthorized("当前密码错误".into()));
    }
    let new_hash = password::hash(req.new_password).await?;
    sqlx::query("UPDATE admins SET password_hash = $2, token_version = token_version + 1 WHERE id = $1")
        .bind(admin.id)
        .bind(new_hash)
        .execute(&state.pool)
        .await?;
    audit(&state, &admin, "admin.password_change", None, None).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn stats(State(state): State<AppState>, _admin: AuthAdmin) -> Result<Json<Value>, AppError> {
    let (total, active, disabled, new_24h, new_7d, logged_in_24h): (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),
                COUNT(*) FILTER (WHERE status = 'active'),
                COUNT(*) FILTER (WHERE status <> 'active'),
                COUNT(*) FILTER (WHERE created_at > now() - interval '1 day'),
                COUNT(*) FILTER (WHERE created_at > now() - interval '7 days'),
                COUNT(*) FILTER (WHERE last_login_at > now() - interval '1 day')
         FROM users",
    )
    .fetch_one(&state.pool)
    .await?;
    let (active_sessions,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM refresh_tokens WHERE revoked_at IS NULL AND expires_at > now()")
            .fetch_one(&state.pool)
            .await?;
    let by_kind: Vec<(String, i64)> = sqlx::query_as(
        "SELECT kind, COUNT(*) FROM verification_codes WHERE created_at > now() - interval '1 day' GROUP BY kind ORDER BY kind",
    )
    .fetch_all(&state.pool)
    .await?;
    let top_ips: Vec<(String, i64)> = sqlx::query_as(
        "SELECT ip, COUNT(*) AS c FROM verification_codes WHERE created_at > now() - interval '1 day'
         GROUP BY ip ORDER BY c DESC, ip LIMIT 5",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({
        "users": { "total": total, "active": active, "disabled": disabled, "new_24h": new_24h, "new_7d": new_7d, "logged_in_24h": logged_in_24h },
        "active_sessions": active_sessions,
        "codes_24h": {
            "total": by_kind.iter().map(|(_, n)| n).sum::<i64>(),
            "by_kind": by_kind.into_iter().map(|(kind, count)| json!({ "kind": kind, "count": count })).collect::<Vec<_>>(),
            "top_ips": top_ips.into_iter().map(|(ip, count)| json!({ "ip": ip, "count": count })).collect::<Vec<_>>(),
        },
    })))
}

#[derive(Deserialize)]
pub struct ListQuery {
    q: Option<String>,
    status: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Serialize)]
struct UserRow {
    uid: Uuid,
    nickname: String,
    avatar_url: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
    last_login_at: Option<DateTime<Utc>>,
    identities: Vec<IdentityView>,
}

fn like_pattern(q: &str) -> String {
    let escaped: String = q
        .chars()
        .flat_map(|c| if matches!(c, '\\' | '%' | '_') { vec!['\\', c] } else { vec![c] })
        .collect();
    format!("%{escaped}%")
}

fn parse_status(status: &str) -> Result<&str, AppError> {
    match status {
        "active" | "disabled" => Ok(status),
        _ => Err(AppError::BadRequest("状态只能是 active 或 disabled".into())),
    }
}

/// 搜索：UUID 精确匹配 uid；邮箱/手机号精确匹配已绑定标识；其余（含上述输入）按昵称模糊匹配。
pub async fn list_users(
    State(state): State<AppState>,
    _admin: AuthAdmin,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, AppError> {
    let q = query.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let status = query.status.as_deref().filter(|s| !s.is_empty()).map(parse_status).transpose()?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let mut uid_filter: Option<Uuid> = None;
    let mut like: Option<String> = None;
    if let Some(q) = q {
        if let Ok(uid) = Uuid::parse_str(q) {
            uid_filter = Some(uid);
        } else if let Ok(ident) = Identifier::parse(q) {
            uid_filter = users::find_uid(&state, &ident).await?;
        }
        like = Some(like_pattern(q));
    }
    let has_query = q.is_some();

    const FILTER: &str = "(NOT $3::bool OR u.uid = $1::uuid OR u.nickname ILIKE $2::text) AND ($4::text IS NULL OR u.status = $4::text)";
    let (total,): (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) FROM users u WHERE {FILTER}"))
        .bind(uid_filter)
        .bind(&like)
        .bind(has_query)
        .bind(status)
        .fetch_one(&state.pool)
        .await?;
    let rows: Vec<(Uuid, String, Option<String>, String, DateTime<Utc>, Option<DateTime<Utc>>)> = sqlx::query_as(&format!(
        "SELECT u.uid, u.nickname, u.avatar_url, u.status, u.created_at, u.last_login_at
         FROM users u WHERE {FILTER} ORDER BY u.created_at DESC, u.uid LIMIT $5 OFFSET $6"
    ))
    .bind(uid_filter)
    .bind(&like)
    .bind(has_query)
    .bind(status)
    .bind(page_size)
    .bind((page - 1) * page_size)
    .fetch_all(&state.pool)
    .await?;

    let uids: Vec<Uuid> = rows.iter().map(|r| r.0).collect();
    let ident_rows: Vec<(Uuid, String, Vec<u8>)> =
        sqlx::query_as("SELECT uid, kind, identifier_enc FROM identities WHERE uid = ANY($1) ORDER BY id")
            .bind(&uids)
            .fetch_all(&state.pool)
            .await?;
    let mut by_uid: HashMap<Uuid, Vec<IdentityView>> = HashMap::new();
    for (uid, kind, enc) in ident_rows {
        let Some(parsed) = Kind::from_db(&kind) else { continue };
        let value = state.keys.sub.decrypt(&kind, &enc)?;
        by_uid.entry(uid).or_default().push(IdentityView { kind, masked: identifier::mask(parsed, &value) });
    }

    let items: Vec<UserRow> = rows
        .into_iter()
        .map(|(uid, nickname, avatar_url, status, created_at, last_login_at)| UserRow {
            identities: by_uid.remove(&uid).unwrap_or_default(),
            uid,
            nickname,
            avatar_url,
            status,
            created_at,
            last_login_at,
        })
        .collect();
    Ok(Json(json!({ "items": items, "total": total, "page": page, "page_size": page_size })))
}

pub async fn get_user(State(state): State<AppState>, _admin: AuthAdmin, Path(uid): Path<Uuid>) -> Result<Json<Value>, AppError> {
    let row: Option<(String, Option<DateTime<Utc>>)> = sqlx::query_as("SELECT status, last_login_at FROM users WHERE uid = $1")
        .bind(uid)
        .fetch_optional(&state.pool)
        .await?;
    let Some((status, last_login_at)) = row else {
        return Err(AppError::NotFound("用户不存在".into()));
    };
    let user = users::view(&state, uid).await?;
    let sessions: Vec<(String, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT client_id, device_name, ip, created_at, expires_at FROM refresh_tokens
         WHERE uid = $1 AND revoked_at IS NULL AND expires_at > now() ORDER BY created_at DESC LIMIT 50",
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({
        "user": user,
        "status": status,
        "last_login_at": last_login_at,
        "sessions": sessions.into_iter().map(|(client_id, device_name, ip, created_at, expires_at)| json!({
            "client_id": client_id, "device_name": device_name, "ip": ip, "created_at": created_at, "expires_at": expires_at,
        })).collect::<Vec<_>>(),
    })))
}

#[derive(Deserialize)]
pub struct SetStatusReq {
    status: String,
}

pub async fn set_user_status(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(uid): Path<Uuid>,
    Json(req): Json<SetStatusReq>,
) -> Result<Json<Value>, AppError> {
    let status = parse_status(&req.status)?;
    let updated = sqlx::query("UPDATE users SET status = $2, updated_at = now() WHERE uid = $1")
        .bind(uid)
        .bind(status)
        .execute(&state.pool)
        .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound("用户不存在".into()));
    }
    if status == "disabled" {
        sessions::revoke_all(&state, uid).await?;
    }
    audit(&state, &admin, &format!("user.{status}"), Some(&uid.to_string()), None).await?;
    Ok(Json(json!({ "ok": true, "status": status })))
}

pub async fn revoke_user_sessions(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(uid): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM users WHERE uid = $1)")
        .bind(uid)
        .fetch_one(&state.pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound("用户不存在".into()));
    }
    sessions::revoke_all(&state, uid).await?;
    audit(&state, &admin, "user.revoke_sessions", Some(&uid.to_string()), None).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn list_clients(State(state): State<AppState>, _admin: AuthAdmin) -> Result<Json<Value>, AppError> {
    let rows: Vec<(String, String, bool, DateTime<Utc>, i64)> = sqlx::query_as(
        "SELECT c.client_id, c.name, c.enabled, c.created_at, COUNT(r.id)
         FROM clients c
         LEFT JOIN refresh_tokens r ON r.client_id = c.client_id AND r.revoked_at IS NULL AND r.expires_at > now()
         GROUP BY c.client_id, c.name, c.enabled, c.created_at
         ORDER BY c.created_at, c.client_id",
    )
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(client_id, name, enabled, created_at, active_sessions)| {
            json!({ "client_id": client_id, "name": name, "enabled": enabled, "created_at": created_at, "active_sessions": active_sessions })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

fn validate_client_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 40 || name.chars().any(char::is_control) {
        return Err(AppError::BadRequest("应用名称需为 1-40 个字符".into()));
    }
    Ok(name)
}

#[derive(Deserialize)]
pub struct CreateClientReq {
    client_id: String,
    name: String,
}

pub async fn create_client(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Json(req): Json<CreateClientReq>,
) -> Result<Json<Value>, AppError> {
    let id = req.client_id.trim();
    let id_ok = (2..=64).contains(&id.len())
        && id.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'));
    if !id_ok {
        return Err(AppError::BadRequest("client_id 需为 2-64 位小写字母、数字、点、下划线或连字符，且以字母或数字开头".into()));
    }
    let name = validate_client_name(&req.name)?;
    let inserted = sqlx::query("INSERT INTO clients (client_id, name) VALUES ($1, $2) ON CONFLICT (client_id) DO NOTHING")
        .bind(id)
        .bind(name)
        .execute(&state.pool)
        .await?;
    if inserted.rows_affected() == 0 {
        return Err(AppError::Conflict("该 client_id 已存在".into()));
    }
    audit(&state, &admin, "client.create", Some(id), Some(name.to_string())).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct UpdateClientReq {
    name: Option<String>,
    enabled: Option<bool>,
}

pub async fn update_client(
    State(state): State<AppState>,
    AuthAdmin(admin): AuthAdmin,
    Path(client_id): Path<String>,
    Json(req): Json<UpdateClientReq>,
) -> Result<Json<Value>, AppError> {
    let name = req.name.as_deref().map(validate_client_name).transpose()?;
    let updated = sqlx::query("UPDATE clients SET name = COALESCE($2, name), enabled = COALESCE($3, enabled) WHERE client_id = $1")
        .bind(&client_id)
        .bind(name)
        .bind(req.enabled)
        .execute(&state.pool)
        .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound("应用不存在".into()));
    }
    let action = match req.enabled {
        Some(true) => "client.enable",
        Some(false) => "client.disable",
        None => "client.rename",
    };
    audit(&state, &admin, action, Some(&client_id), name.map(str::to_string)).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct AuditQuery {
    limit: Option<i64>,
}

pub async fn audit_log(State(state): State<AppState>, _admin: AuthAdmin, Query(query): Query<AuditQuery>) -> Result<Json<Value>, AppError> {
    let rows: Vec<(String, String, Option<String>, Option<String>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT admin_username, action, target, detail, created_at FROM admin_audit ORDER BY id DESC LIMIT $1",
    )
    .bind(query.limit.unwrap_or(50).clamp(1, 200))
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(admin, action, target, detail, created_at)| json!({ "admin": admin, "action": action, "target": target, "detail": detail, "created_at": created_at }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

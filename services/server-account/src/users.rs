use crate::{
    error::AppError,
    identifier::{self, Identifier, Kind},
    state::AppState,
    username::{self, Username},
};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct IdentityView {
    pub kind: String,
    pub masked: String,
}

#[derive(Serialize)]
pub struct UserView {
    pub uid: Uuid,
    /// 建号时分配的数字账号 ID（字符串），唯一且永不修改。
    pub account_id: String,
    /// 唯一且不可修改的用户名，也是对外展示的名字。
    pub username: String,
    pub avatar_url: Option<String>,
    pub has_password: bool,
    pub identities: Vec<IdentityView>,
    pub created_at: DateTime<Utc>,
    /// 已验证邮箱明文，仅 `GET /api/me` 且令牌属于 `ACCOUNT_EMAIL_CLIENTS` 时返回，供业务服务按邮箱关联旧账号。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

pub async fn find_uid(state: &AppState, ident: &Identifier) -> Result<Option<Uuid>, AppError> {
    let hash = state.keys.sub.identifier_hash(ident.kind.as_str(), &ident.value);
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT uid FROM identities WHERE kind = $1 AND identifier_hash = $2")
        .bind(ident.kind.as_str())
        .bind(hash)
        .fetch_optional(&state.pool)
        .await?;
    Ok(row.map(|(uid,)| uid))
}

pub async fn ensure_active(state: &AppState, uid: Uuid) -> Result<(), AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT status FROM users WHERE uid = $1")
        .bind(uid)
        .fetch_optional(&state.pool)
        .await?;
    match row {
        Some((status,)) if status == "active" => Ok(()),
        Some(_) => Err(AppError::Forbidden("账号已被停用".into())),
        None => Err(AppError::Unauthorized("登录已失效，请重新登录".into())),
    }
}

pub fn username_taken() -> AppError {
    AppError::Coded { status: StatusCode::CONFLICT, code: "username_taken", message: "这个用户名已被占用，换一个吧".into() }
}

pub async fn username_exists(state: &AppState, key: &str) -> Result<bool, AppError> {
    let row: Option<(bool,)> = sqlx::query_as("SELECT true FROM users WHERE username_key = $1")
        .bind(key)
        .fetch_optional(&state.pool)
        .await?;
    Ok(row.is_some())
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(e) if e.code().as_deref() == Some("23505"))
}

/// 一次事务里建号：用户名、密码和已验证的登录标识同时写入，不会留下缺任何一项的半成品账号。
pub async fn register(
    state: &AppState,
    ident: &Identifier,
    username: &Username,
    password_hash: &str,
) -> Result<Uuid, AppError> {
    let identity_exists = || AppError::Coded {
        status: StatusCode::CONFLICT,
        code: "identity_exists",
        message: "这个邮箱或手机号已经注册过了，请直接登录".into(),
    };
    if find_uid(state, ident).await?.is_some() {
        return Err(identity_exists());
    }

    let kind = ident.kind.as_str();
    let uid = Uuid::new_v4();
    let mut tx = state.pool.begin().await?;
    sqlx::query("INSERT INTO users (uid, username, username_key, password_hash) VALUES ($1, $2, $3, $4)")
        .bind(uid)
        .bind(&username.display)
        .bind(&username.key)
        .bind(password_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| if is_unique_violation(&e) { username_taken() } else { e.into() })?;
    let inserted = sqlx::query(
        "INSERT INTO identities (uid, kind, identifier_hash, identifier_enc, verified_at)
         VALUES ($1, $2, $3, $4, now()) ON CONFLICT (kind, identifier_hash) DO NOTHING",
    )
    .bind(uid)
    .bind(kind)
    .bind(state.keys.sub.identifier_hash(kind, &ident.value))
    .bind(state.keys.sub.encrypt(kind, &ident.value)?)
    .execute(&mut *tx)
    .await?;
    if inserted.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(identity_exists());
    }
    tx.commit().await?;
    Ok(uid)
}

/// 管理员改名：用户名对用户本人不可修改，这是唯一入口。
pub async fn admin_set_username(state: &AppState, uid: Uuid, username: &Username) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE users SET username = $2, username_key = $3, updated_at = now() WHERE uid = $1")
        .bind(uid)
        .bind(&username.display)
        .bind(&username.key)
        .execute(&state.pool)
        .await
        .map_err(|e| if is_unique_violation(&e) { username_taken() } else { e.into() })?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("用户不存在".into()));
    }
    Ok(())
}

pub struct LoginRow {
    pub uid: Uuid,
    pub password_hash: Option<String>,
    pub locked: bool,
}

/// 密码登录的账号查找：用户名、邮箱、手机号三选一；格式不合法或查无此人都返回 `None`。
pub async fn find_for_password_login(state: &AppState, raw: &str) -> Result<Option<LoginRow>, AppError> {
    const SELECT: &str = "SELECT u.uid, u.password_hash, COALESCE(u.locked_until > now(), false)";
    let row: Option<(Uuid, Option<String>, bool)> = if username::looks_like_username(raw) {
        sqlx::query_as(&format!("{SELECT} FROM users u WHERE u.username_key = $1"))
            .bind(raw.trim().to_ascii_lowercase())
            .fetch_optional(&state.pool)
            .await?
    } else {
        let Ok(ident) = Identifier::parse(raw) else { return Ok(None) };
        sqlx::query_as(&format!(
            "{SELECT} FROM identities i JOIN users u ON u.uid = i.uid WHERE i.kind = $1 AND i.identifier_hash = $2"
        ))
        .bind(ident.kind.as_str())
        .bind(state.keys.sub.identifier_hash(ident.kind.as_str(), &ident.value))
        .fetch_optional(&state.pool)
        .await?
    };
    Ok(row.map(|(uid, password_hash, locked)| LoginRow { uid, password_hash, locked }))
}

pub async fn touch_login(state: &AppState, uid: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET last_login_at = now(), failed_logins = 0, locked_until = NULL WHERE uid = $1")
        .bind(uid)
        .execute(&state.pool)
        .await?;
    Ok(())
}

pub async fn view(state: &AppState, uid: Uuid) -> Result<UserView, AppError> {
    let user: Option<(String, String, Option<String>, bool, DateTime<Utc>)> = sqlx::query_as(
        "SELECT account_id::text, username, avatar_url, password_hash IS NOT NULL, created_at FROM users WHERE uid = $1",
    )
    .bind(uid)
    .fetch_optional(&state.pool)
    .await?;
    let Some((account_id, username, avatar_url, has_password, created_at)) = user else {
        return Err(AppError::Unauthorized("登录已失效，请重新登录".into()));
    };

    let rows: Vec<(String, Vec<u8>)> =
        sqlx::query_as("SELECT kind, identifier_enc FROM identities WHERE uid = $1 ORDER BY id")
            .bind(uid)
            .fetch_all(&state.pool)
            .await?;
    let mut identities = Vec::with_capacity(rows.len());
    for (kind, enc) in rows {
        let Some(parsed) = Kind::from_db(&kind) else { continue };
        let value = state.keys.sub.decrypt(&kind, &enc)?;
        identities.push(IdentityView { kind, masked: identifier::mask(parsed, &value) });
    }

    Ok(UserView { uid, account_id, username, avatar_url, has_password, identities, created_at, email: None })
}

pub async fn verified_email(state: &AppState, uid: Uuid) -> Result<Option<String>, AppError> {
    let row: Option<(Vec<u8>,)> =
        sqlx::query_as("SELECT identifier_enc FROM identities WHERE uid = $1 AND kind = 'email' AND verified_at IS NOT NULL")
            .bind(uid)
            .fetch_optional(&state.pool)
            .await?;
    row.map(|(enc,)| state.keys.sub.decrypt("email", &enc)).transpose()
}

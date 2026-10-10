use crate::{
    crypto,
    error::AppError,
    identifier::{self, Identifier, Kind},
    state::AppState,
};
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
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub has_password: bool,
    pub identities: Vec<IdentityView>,
    pub created_at: DateTime<Utc>,
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

/// 返回 (uid, 是否新注册)。验证码通过即视为该标识符已验证。
pub async fn get_or_create(state: &AppState, ident: &Identifier) -> Result<(Uuid, bool), AppError> {
    if let Some(uid) = find_uid(state, ident).await? {
        return Ok((uid, false));
    }

    let kind = ident.kind.as_str();
    let uid = Uuid::new_v4();
    let mut tx = state.pool.begin().await?;
    sqlx::query("INSERT INTO users (uid, nickname) VALUES ($1, $2)")
        .bind(uid)
        .bind(format!("伴友{}", crypto::random_code()))
        .execute(&mut *tx)
        .await?;
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
        let existing = find_uid(state, ident)
            .await?
            .ok_or_else(|| AppError::Internal("identity vanished during registration".into()))?;
        return Ok((existing, false));
    }
    tx.commit().await?;
    Ok((uid, true))
}

pub async fn touch_login(state: &AppState, uid: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET last_login_at = now(), failed_logins = 0, locked_until = NULL WHERE uid = $1")
        .bind(uid)
        .execute(&state.pool)
        .await?;
    Ok(())
}

pub async fn view(state: &AppState, uid: Uuid) -> Result<UserView, AppError> {
    let user: Option<(String, Option<String>, bool, DateTime<Utc>)> =
        sqlx::query_as("SELECT nickname, avatar_url, password_hash IS NOT NULL, created_at FROM users WHERE uid = $1")
            .bind(uid)
            .fetch_optional(&state.pool)
            .await?;
    let Some((nickname, avatar_url, has_password, created_at)) = user else {
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

    Ok(UserView { uid, nickname, avatar_url, has_password, identities, created_at })
}

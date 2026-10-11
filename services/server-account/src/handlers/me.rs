use crate::{
    codes::{self, Purpose},
    error::AppError,
    extract::{AuthUser, ClientIp},
    identifier::Identifier,
    password,
    state::AppState,
    users::{self, UserView},
};
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

pub async fn get_me(State(state): State<AppState>, user: AuthUser) -> Result<Json<UserView>, AppError> {
    users::ensure_active(&state, user.uid).await?;
    let mut view = users::view(&state, user.uid).await?;
    if state.config.email_clients.contains(&user.client_id) {
        view.email = users::verified_email(&state, user.uid).await?;
    }
    Ok(Json(view))
}

#[derive(Deserialize)]
pub struct UpdateMeReq {
    /// 空字符串表示清除头像。用户名不可修改，所以这里没有它。
    avatar_url: Option<String>,
}

pub async fn update_me(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UpdateMeReq>,
) -> Result<Json<UserView>, AppError> {
    users::ensure_active(&state, user.uid).await?;

    if let Some(avatar) = &req.avatar_url {
        let avatar = avatar.trim();
        if !avatar.is_empty() && (!avatar.starts_with("https://") || avatar.len() > 512 || avatar.chars().any(char::is_whitespace)) {
            return Err(AppError::BadRequest("头像地址需为 https 链接且不超过 512 字符".into()));
        }
        sqlx::query("UPDATE users SET avatar_url = NULLIF($2, ''), updated_at = now() WHERE uid = $1")
            .bind(user.uid)
            .bind(avatar)
            .execute(&state.pool)
            .await?;
    }
    users::view(&state, user.uid).await.map(Json)
}

#[derive(Deserialize)]
pub struct SetPasswordReq {
    current_password: Option<String>,
    new_password: String,
}

pub async fn set_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SetPasswordReq>,
) -> Result<Json<Value>, AppError> {
    users::ensure_active(&state, user.uid).await?;
    password::validate(&req.new_password)?;

    let (existing, username): (Option<String>, String) =
        sqlx::query_as("SELECT password_hash, username FROM users WHERE uid = $1")
            .bind(user.uid)
            .fetch_one(&state.pool)
            .await?;
    if req.new_password.eq_ignore_ascii_case(&username) {
        return Err(AppError::BadRequest("密码不能和用户名相同".into()));
    }
    if let Some(hash) = existing {
        let current = req.current_password.ok_or_else(|| AppError::BadRequest("请输入当前密码".into()))?;
        if !password::verify(current, hash).await {
            return Err(AppError::Unauthorized("当前密码错误".into()));
        }
    }

    let hash = password::hash(req.new_password).await?;
    sqlx::query("UPDATE users SET password_hash = $2, updated_at = now() WHERE uid = $1")
        .bind(user.uid)
        .bind(hash)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct BindCodeReq {
    identifier: String,
}

async fn check_bindable(state: &AppState, uid: uuid::Uuid, ident: &Identifier) -> Result<(), AppError> {
    if let Some(owner) = users::find_uid(state, ident).await? {
        return Err(AppError::Conflict(if owner == uid {
            "该账号已绑定在当前用户".into()
        } else {
            "该账号已被其他用户绑定".into()
        }));
    }
    let (has_kind,): (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM identities WHERE uid = $1 AND kind = $2)")
        .bind(uid)
        .bind(ident.kind.as_str())
        .fetch_one(&state.pool)
        .await?;
    if has_kind {
        return Err(AppError::Conflict("当前用户已绑定同类型账号".into()));
    }
    Ok(())
}

pub async fn send_bind_code(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    user: AuthUser,
    Json(req): Json<BindCodeReq>,
) -> Result<Json<Value>, AppError> {
    users::ensure_active(&state, user.uid).await?;
    let ident = Identifier::parse(&req.identifier)?;
    check_bindable(&state, user.uid, &ident).await?;
    codes::issue(&state, &ident, Purpose::Bind, &ip).await?;
    Ok(Json(json!({
        "ok": true,
        "cooldown_seconds": state.config.resend_cooldown_secs,
        "expires_in": state.config.code_ttl_secs,
    })))
}

#[derive(Deserialize)]
pub struct BindReq {
    identifier: String,
    code: String,
}

pub async fn bind_identity(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<BindReq>,
) -> Result<Json<UserView>, AppError> {
    users::ensure_active(&state, user.uid).await?;
    let ident = Identifier::parse(&req.identifier)?;
    check_bindable(&state, user.uid, &ident).await?;
    codes::verify(&state, &ident, Purpose::Bind, &req.code).await?;

    let kind = ident.kind.as_str();
    let inserted = sqlx::query(
        "INSERT INTO identities (uid, kind, identifier_hash, identifier_enc, verified_at)
         VALUES ($1, $2, $3, $4, now()) ON CONFLICT (kind, identifier_hash) DO NOTHING",
    )
    .bind(user.uid)
    .bind(kind)
    .bind(state.keys.sub.identifier_hash(kind, &ident.value))
    .bind(state.keys.sub.encrypt(kind, &ident.value)?)
    .execute(&state.pool)
    .await?;
    if inserted.rows_affected() == 0 {
        return Err(AppError::Conflict("该账号已被其他用户绑定".into()));
    }
    users::view(&state, user.uid).await.map(Json)
}

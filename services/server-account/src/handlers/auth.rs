use crate::{
    codes::{self, Purpose},
    error::AppError,
    extract::{AuthUser, ClientIp},
    identifier::Identifier,
    password,
    sessions::{self, TokenPair},
    state::AppState,
    users::{self, UserView},
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

const MAX_PASSWORD_FAILURES: i32 = 5;
const BAD_CREDENTIALS: &str = "账号或密码错误";

#[derive(Serialize)]
pub struct AuthResponse {
    #[serde(flatten)]
    tokens: TokenPair,
    is_new_user: bool,
    user: UserView,
}

async fn require_client(state: &AppState, client_id: &str) -> Result<(), AppError> {
    let row: Option<(bool,)> = sqlx::query_as("SELECT enabled FROM clients WHERE client_id = $1")
        .bind(client_id)
        .fetch_optional(&state.pool)
        .await?;
    match row {
        Some((true,)) => Ok(()),
        _ => Err(AppError::BadRequest("未知或已停用的应用 client_id".into())),
    }
}

async fn respond(
    state: &AppState,
    uid: Uuid,
    is_new_user: bool,
    client_id: &str,
    device_name: Option<&str>,
    ip: &str,
) -> Result<Json<AuthResponse>, AppError> {
    users::touch_login(state, uid).await?;
    let tokens = sessions::issue(state, uid, client_id, None, device_name, ip).await?;
    let user = users::view(state, uid).await?;
    Ok(Json(AuthResponse { tokens, is_new_user, user }))
}

#[derive(Deserialize)]
pub struct SendCodeReq {
    identifier: String,
    purpose: Option<String>,
}

pub async fn send_code(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(req): Json<SendCodeReq>,
) -> Result<Json<Value>, AppError> {
    let purpose = match req.purpose.as_deref().unwrap_or("login") {
        "login" => Purpose::Login,
        "reset_password" => Purpose::ResetPassword,
        _ => return Err(AppError::BadRequest("不支持的验证码用途".into())),
    };
    let ident = Identifier::parse(&req.identifier)?;

    // 重置密码时不暴露账号是否存在：不存在则静默成功、不发送。
    let should_send = purpose == Purpose::Login || users::find_uid(&state, &ident).await?.is_some();
    if should_send {
        codes::issue(&state, &ident, purpose, &ip).await?;
    }
    Ok(Json(json!({
        "ok": true,
        "cooldown_seconds": state.config.resend_cooldown_secs,
        "expires_in": state.config.code_ttl_secs,
    })))
}

#[derive(Deserialize)]
pub struct CodeLoginReq {
    identifier: String,
    code: String,
    client_id: String,
    device_name: Option<String>,
}

pub async fn login_with_code(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(req): Json<CodeLoginReq>,
) -> Result<Json<AuthResponse>, AppError> {
    let ident = Identifier::parse(&req.identifier)?;
    require_client(&state, &req.client_id).await?;
    codes::verify(&state, &ident, Purpose::Login, &req.code).await?;

    let (uid, is_new) = users::get_or_create(&state, &ident).await?;
    users::ensure_active(&state, uid).await?;
    respond(&state, uid, is_new, &req.client_id, req.device_name.as_deref(), &ip).await
}

#[derive(Deserialize)]
pub struct PasswordLoginReq {
    identifier: String,
    password: String,
    client_id: String,
    device_name: Option<String>,
}

pub async fn login_with_password(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(req): Json<PasswordLoginReq>,
) -> Result<Json<AuthResponse>, AppError> {
    let ident = Identifier::parse(&req.identifier).map_err(|_| AppError::Unauthorized(BAD_CREDENTIALS.into()))?;
    require_client(&state, &req.client_id).await?;

    let row: Option<(Uuid, Option<String>, bool)> = sqlx::query_as(
        "SELECT u.uid, u.password_hash, COALESCE(u.locked_until > now(), false)
         FROM identities i JOIN users u ON u.uid = i.uid
         WHERE i.kind = $1 AND i.identifier_hash = $2",
    )
    .bind(ident.kind.as_str())
    .bind(state.keys.sub.identifier_hash(ident.kind.as_str(), &ident.value))
    .fetch_optional(&state.pool)
    .await?;

    let Some((uid, Some(hash), locked)) = row else {
        password::verify(req.password, state.dummy_password_hash.to_string()).await;
        return Err(AppError::Unauthorized(BAD_CREDENTIALS.into()));
    };
    if locked {
        return Err(AppError::TooManyRequests { message: "密码错误次数过多，请稍后再试或使用验证码登录".into(), retry_after: 900 });
    }

    if !password::verify(req.password, hash).await {
        sqlx::query(
            "UPDATE users SET
                locked_until = CASE WHEN failed_logins + 1 >= $2 THEN now() + interval '15 minutes' ELSE locked_until END,
                failed_logins = CASE WHEN failed_logins + 1 >= $2 THEN 0 ELSE failed_logins + 1 END
             WHERE uid = $1",
        )
        .bind(uid)
        .bind(MAX_PASSWORD_FAILURES)
        .execute(&state.pool)
        .await?;
        return Err(AppError::Unauthorized(BAD_CREDENTIALS.into()));
    }

    users::ensure_active(&state, uid).await?;
    respond(&state, uid, false, &req.client_id, req.device_name.as_deref(), &ip).await
}

#[derive(Deserialize)]
pub struct ResetPasswordReq {
    identifier: String,
    code: String,
    new_password: String,
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordReq>,
) -> Result<Json<Value>, AppError> {
    let ident = Identifier::parse(&req.identifier)?;
    password::validate(&req.new_password)?;
    codes::verify(&state, &ident, Purpose::ResetPassword, &req.code).await?;

    let uid = users::find_uid(&state, &ident)
        .await?
        .ok_or_else(|| AppError::BadRequest("验证码错误或已过期".into()))?;
    let hash = password::hash(req.new_password).await?;
    sqlx::query("UPDATE users SET password_hash = $2, failed_logins = 0, locked_until = NULL, updated_at = now() WHERE uid = $1")
        .bind(uid)
        .bind(hash)
        .execute(&state.pool)
        .await?;
    sessions::revoke_all(&state, uid).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct RefreshReq {
    refresh_token: String,
    client_id: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(req): Json<RefreshReq>,
) -> Result<Json<TokenPair>, AppError> {
    sessions::rotate(&state, &req.refresh_token, &req.client_id, &ip).await.map(Json)
}

#[derive(Deserialize)]
pub struct LogoutReq {
    refresh_token: String,
}

pub async fn logout(State(state): State<AppState>, Json(req): Json<LogoutReq>) -> Result<Json<Value>, AppError> {
    sessions::revoke(&state, &req.refresh_token).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn logout_all(State(state): State<AppState>, user: AuthUser) -> Result<Json<Value>, AppError> {
    sessions::revoke_all(&state, user.uid).await?;
    Ok(Json(json!({ "ok": true })))
}

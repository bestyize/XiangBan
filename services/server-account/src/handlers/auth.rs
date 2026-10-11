use crate::{
    codes::{self, Purpose},
    error::AppError,
    extract::{AuthUser, ClientIp},
    identifier::Identifier,
    password, registration,
    sessions::{self, TokenPair},
    state::AppState,
    username,
    users::{self, UserView},
};
use axum::{
    extract::{Query, State},
    Json,
};
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

#[derive(Serialize)]
#[serde(untagged)]
pub enum CodeLoginResponse {
    /// 这个邮箱 / 手机号已有账号：直接登录。
    Session(AuthResponse),
    /// 新的邮箱 / 手机号：验证码通过，但还没有账号，客户端带着凭证去设置用户名和密码。
    RegistrationRequired(RegistrationRequired),
}

#[derive(Serialize)]
pub struct RegistrationRequired {
    registration_required: bool,
    registration_token: String,
    expires_in: i64,
}

pub async fn login_with_code(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(req): Json<CodeLoginReq>,
) -> Result<Json<CodeLoginResponse>, AppError> {
    let ident = Identifier::parse(&req.identifier)?;
    require_client(&state, &req.client_id).await?;
    codes::verify(&state, &ident, Purpose::Login, &req.code).await?;

    let Some(uid) = users::find_uid(&state, &ident).await? else {
        return Ok(Json(CodeLoginResponse::RegistrationRequired(RegistrationRequired {
            registration_required: true,
            registration_token: registration::issue(&state.keys.sub.register_jwt_secret, &ident)?,
            expires_in: registration::TTL_SECS,
        })));
    };
    users::ensure_active(&state, uid).await?;
    let session = respond(&state, uid, false, &req.client_id, req.device_name.as_deref(), &ip).await?;
    Ok(Json(CodeLoginResponse::Session(session.0)))
}

#[derive(Deserialize)]
pub struct RegisterReq {
    registration_token: String,
    username: String,
    password: String,
    client_id: String,
    device_name: Option<String>,
}

pub async fn register(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(req): Json<RegisterReq>,
) -> Result<Json<AuthResponse>, AppError> {
    require_client(&state, &req.client_id).await?;
    let ident = registration::verify(&state.keys.sub.register_jwt_secret, &req.registration_token)?;
    let name = username::parse(&req.username)?;
    password::validate(&req.password)?;
    if req.password.eq_ignore_ascii_case(&name.display) {
        return Err(AppError::BadRequest("密码不能和用户名相同".into()));
    }
    let hash = password::hash(req.password).await?;
    let uid = users::register(&state, &ident, &name, &hash).await?;
    respond(&state, uid, true, &req.client_id, req.device_name.as_deref(), &ip).await
}

#[derive(Deserialize)]
pub struct UsernameQuery {
    username: String,
}

/// 设置账号页边输入边查重。只是提示：真正的唯一性由 `register` 里的唯一索引保证。
pub async fn username_available(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Query(query): Query<UsernameQuery>,
) -> Result<Json<Value>, AppError> {
    state.username_limiter.check(&ip).map_err(|retry_after| AppError::TooManyRequests {
        message: "查询太频繁，请稍后再试".into(),
        retry_after,
    })?;
    let name = match username::parse(&query.username) {
        Ok(name) => name,
        Err(AppError::BadRequest(message)) => return Ok(Json(json!({ "available": false, "message": message }))),
        Err(other) => return Err(other),
    };
    let taken = users::username_exists(&state, &name.key).await?;
    Ok(Json(json!({
        "available": !taken,
        "message": if taken { Value::from("这个用户名已被占用，换一个吧") } else { Value::Null },
    })))
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
    require_client(&state, &req.client_id).await?;

    // 标识可以是用户名、邮箱或手机号；查无此人、没设密码、格式不对一律走同样的耗时与提示。
    let found = users::find_for_password_login(&state, &req.identifier).await?;
    let Some(users::LoginRow { uid, password_hash: Some(hash), locked }) = found else {
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

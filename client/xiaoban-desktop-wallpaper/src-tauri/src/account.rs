//! User account authentication and persistence for Xiaoban desktop.
//! Integrates with the unified XiangBan Account Center (server-account).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager};

pub const DEFAULT_ACCOUNT_URL: &str = "https://account.xiangban.xyz";
const CLIENT_ID: &str = "xiaoban-desktop-wallpaper";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountUser {
    pub uid: String,
    /// 唯一且不可修改的用户名，也是对外展示的名字。
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub user: AccountUser,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCodeResult {
    pub ok: bool,
    pub cooldown_seconds: u64,
    pub expires_in: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    pub is_new_user: bool,
    pub session: AccountSession,
}

/// 验证码登录的结果：已有账号直接登录；新的邮箱 / 手机号要先设置用户名和密码。
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CodeLoginOutcome {
    #[serde(rename_all = "camelCase")]
    Session { is_new_user: bool, session: AccountSession },
    #[serde(rename_all = "camelCase")]
    Registration { registration_token: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsernameCheck {
    pub available: bool,
    pub message: Option<String>,
}

/// 账号中心的错误：`code` 是机读值（username_taken、registration_expired、identity_exists 等）。
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AccountFailure {
    pub message: String,
    pub code: Option<String>,
}

impl From<String> for AccountFailure {
    fn from(message: String) -> Self {
        Self { message, code: None }
    }
}

impl From<&str> for AccountFailure {
    fn from(message: &str) -> Self {
        message.to_string().into()
    }
}

#[derive(Default)]
pub struct AccountState {
    pub session: Mutex<Option<AccountSession>>,
    /// 刷新令牌每次使用都会轮换，旧的再用会让整个会话作废，所以同一时刻只能有一个刷新请求。
    refreshing: tauri::async_runtime::Mutex<()>,
}

/// 访问令牌剩余有效期低于这个值就先刷新，免得请求在路上过期。
const REFRESH_MARGIN_SECS: i64 = 120;

pub fn account_base_url() -> String {
    std::env::var("XIAOBAN_ACCOUNT_URL")
        .unwrap_or_else(|_| DEFAULT_ACCOUNT_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(8))
        .user_agent(concat!("XiaoBanDesktop/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("创建网络连接失败：{e}"))
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config/account.json"))
}

pub fn load_session(app: &tauri::AppHandle) -> Option<AccountSession> {
    let path = config_path(app).ok()?;
    let text = std::fs::read(path).ok()?;
    serde_json::from_slice::<AccountSession>(&text).ok()
}

pub fn save_session(app: &tauri::AppHandle, session: &AccountSession) -> Result<(), String> {
    let path = config_path(app)?;
    crate::library::save_json(&path, session)
}

pub fn clear_session(app: &tauri::AppHandle) -> Result<(), String> {
    let path = config_path(app)?;
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("无法清除本地登录会话：{e}"))?;
    }
    Ok(())
}

fn device_name() -> String {
    let os = if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Windows"
    };
    format!("小伴桌面端 ({os})")
}

fn parse_error_response(status: reqwest::StatusCode, body_text: &str) -> AccountFailure {
    let parsed = serde_json::from_str::<Value>(body_text).ok();
    let code = parsed.as_ref().and_then(|v| v.get("code")).and_then(Value::as_str).map(String::from);
    if let Some(msg) = parsed.as_ref().and_then(|v| v.get("message")).and_then(Value::as_str) {
        return AccountFailure { message: msg.to_string(), code };
    }
    let message = match status.as_u16() {
        400 => "请求参数有误，请检查输入",
        401 => "账号或验证码不正确，或已过期",
        429 => "请求过于频繁，请稍后再试",
        500..=599 => "账号服务暂时不可用，请稍后重试",
        other => return format!("账号服务返回状态异常 ({other})").into(),
    };
    AccountFailure { message: message.into(), code }
}

/// 阻塞地请求账号中心；只能在 `spawn_blocking` 里调用。
fn request_json(method: reqwest::Method, path: &str, payload: Option<Value>) -> Result<Value, AccountFailure> {
    let url = format!("{}{}", account_base_url(), path);
    let mut request = client()?.request(method, &url);
    if let Some(payload) = payload {
        request = request.header("Content-Type", "application/json").body(serde_json::to_string(&payload).map_err(|e| e.to_string())?);
    }
    let res = request.send().map_err(|e| format!("连接账号服务失败：{e}"))?;
    let status = res.status();
    let body = res.text().map_err(|e| format!("读取账号服务响应失败：{e}"))?;
    if !status.is_success() {
        return Err(parse_error_response(status, &body));
    }
    serde_json::from_str(&body).map_err(|e| format!("解析账号服务响应失败：{e}").into())
}

async fn call_json(method: reqwest::Method, path: String, payload: Option<Value>) -> Result<Value, AccountFailure> {
    tauri::async_runtime::spawn_blocking(move || request_json(method, &path, payload))
        .await
        .map_err(|e| AccountFailure::from(e.to_string()))?
}

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// 账号中心的登录 / 注册响应 → 本地会话。
fn parse_login(val: &Value, now_sec: i64) -> Result<LoginResult, AccountFailure> {
    let access_token = val["access_token"].as_str().ok_or("缺少 access_token")?.to_string();
    let refresh_token = val["refresh_token"].as_str().ok_or("缺少 refresh_token")?.to_string();
    let expires_in = val["expires_in"].as_i64().unwrap_or(1800);
    let user_val = &val["user"];
    let user = AccountUser {
        uid: user_val["uid"].as_str().ok_or("缺少 uid")?.to_string(),
        username: user_val["username"].as_str().filter(|s| !s.is_empty()).ok_or("缺少 username")?.to_string(),
        avatar_url: user_val["avatar_url"].as_str().map(String::from),
    };
    Ok(LoginResult {
        is_new_user: val["is_new_user"].as_bool().unwrap_or(false),
        session: AccountSession { access_token, refresh_token, expires_at: now_sec + expires_in, user },
    })
}

/// 刷新接口只返回新的令牌对，用户信息沿用本地的。
fn parse_refresh(old: &AccountSession, val: &Value, now_sec: i64) -> Result<AccountSession, AccountFailure> {
    Ok(AccountSession {
        access_token: val["access_token"].as_str().ok_or("缺少 access_token")?.to_string(),
        refresh_token: val["refresh_token"].as_str().ok_or("缺少 refresh_token")?.to_string(),
        expires_at: now_sec + val["expires_in"].as_i64().unwrap_or(1800),
        user: old.user.clone(),
    })
}

fn is_fresh(session: &AccountSession, now_sec: i64) -> bool {
    session.expires_at - now_sec > REFRESH_MARGIN_SECS
}

/// 返回一个还有足够有效期的访问令牌，必要时先用刷新令牌换新的。
/// 账号中心明确拒绝刷新令牌时，本地会话已经无法恢复，按退出登录处理，界面会回到未登录。
pub async fn fresh_access_token(app: &tauri::AppHandle) -> Result<String, AccountFailure> {
    let state = app.state::<AccountState>();
    let signed_in = || state.session.lock().unwrap().clone().ok_or_else(|| AccountFailure::from("请先登录小伴账号"));
    let current = signed_in()?;
    if is_fresh(&current, now_secs()) {
        return Ok(current.access_token);
    }
    let _only_one = state.refreshing.lock().await;
    let current = signed_in()?;
    if is_fresh(&current, now_secs()) {
        return Ok(current.access_token);
    }
    let val = match call_json(
        reqwest::Method::POST,
        "/api/auth/refresh".into(),
        Some(json!({ "refresh_token": current.refresh_token, "client_id": CLIENT_ID })),
    )
    .await
    {
        Ok(val) => val,
        Err(failure) => {
            if failure.code.as_deref() == Some("unauthorized") {
                let _ = logout_account(app.clone()).await;
            }
            return Err(failure);
        }
    };
    let next = parse_refresh(&current, &val, now_secs())?;
    {
        let mut slot = state.session.lock().unwrap();
        if slot.as_ref().map(|s| s.refresh_token.as_str()) != Some(current.refresh_token.as_str()) {
            return Err("账号已切换，请重试".into());
        }
        *slot = Some(next.clone());
    }
    // 写盘失败不影响本次运行；最坏是下次启动需要重新登录。
    let _ = save_session(app, &next);
    Ok(next.access_token)
}

fn parse_code_login(val: &Value, now_sec: i64) -> Result<CodeLoginOutcome, AccountFailure> {
    if val["registration_required"].as_bool() == Some(true) {
        let token = val["registration_token"].as_str().ok_or("缺少 registration_token")?;
        return Ok(CodeLoginOutcome::Registration { registration_token: token.to_string() });
    }
    let LoginResult { is_new_user, session } = parse_login(val, now_sec)?;
    Ok(CodeLoginOutcome::Session { is_new_user, session })
}

fn require_identifier(identifier: &str) -> Result<String, AccountFailure> {
    let identifier = identifier.trim();
    if identifier.is_empty() {
        return Err("请输入账号".into());
    }
    Ok(identifier.to_string())
}

fn remember(app: &tauri::AppHandle, result: &LoginResult) -> Result<(), AccountFailure> {
    save_session(app, &result.session)?;
    *app.state::<AccountState>().session.lock().unwrap() = Some(result.session.clone());
    crate::ai_catalogue::account_changed(app);
    let _ = app.emit("account-changed", Some(result.session.clone()));
    Ok(())
}

#[tauri::command]
pub async fn send_account_code(identifier: String, purpose: Option<String>) -> Result<SendCodeResult, AccountFailure> {
    let identifier = require_identifier(&identifier)?;
    let purpose = match purpose.as_deref().unwrap_or("login") {
        "reset_password" => "reset_password",
        _ => "login",
    };
    let parsed = call_json(reqwest::Method::POST, "/api/auth/code/send".into(), Some(json!({ "identifier": identifier, "purpose": purpose }))).await?;
    Ok(SendCodeResult {
        ok: true,
        cooldown_seconds: parsed.get("cooldown_seconds").and_then(Value::as_u64).unwrap_or(60),
        expires_in: parsed.get("expires_in").and_then(Value::as_u64).unwrap_or(300),
    })
}

#[tauri::command]
pub async fn login_account_with_code(app: tauri::AppHandle, identifier: String, code: String) -> Result<CodeLoginOutcome, AccountFailure> {
    let identifier = require_identifier(&identifier)?;
    let code = code.trim().to_string();
    if code.is_empty() {
        return Err("请输入验证码".into());
    }
    let val = call_json(
        reqwest::Method::POST,
        "/api/auth/code/login".into(),
        Some(json!({ "identifier": identifier, "code": code, "client_id": CLIENT_ID, "device_name": device_name() })),
    )
    .await?;
    let outcome = parse_code_login(&val, now_secs())?;
    if let CodeLoginOutcome::Session { is_new_user, session } = &outcome {
        remember(&app, &LoginResult { is_new_user: *is_new_user, session: session.clone() })?;
    }
    Ok(outcome)
}

#[tauri::command]
pub async fn login_account_with_password(app: tauri::AppHandle, identifier: String, password: String) -> Result<LoginResult, AccountFailure> {
    let identifier = require_identifier(&identifier)?;
    if password.is_empty() {
        return Err("请输入密码".into());
    }
    let val = call_json(
        reqwest::Method::POST,
        "/api/auth/password/login".into(),
        Some(json!({ "identifier": identifier, "password": password, "client_id": CLIENT_ID, "device_name": device_name() })),
    )
    .await?;
    let result = parse_login(&val, now_secs())?;
    remember(&app, &result)?;
    Ok(result)
}

#[tauri::command]
pub async fn register_account(
    app: tauri::AppHandle,
    registration_token: String,
    username: String,
    password: String,
) -> Result<LoginResult, AccountFailure> {
    let val = call_json(
        reqwest::Method::POST,
        "/api/auth/register".into(),
        Some(json!({
            "registration_token": registration_token,
            "username": username.trim(),
            "password": password,
            "client_id": CLIENT_ID,
            "device_name": device_name()
        })),
    )
    .await?;
    let result = parse_login(&val, now_secs())?;
    remember(&app, &result)?;
    Ok(result)
}

#[tauri::command]
pub async fn reset_account_password(identifier: String, code: String, new_password: String) -> Result<(), AccountFailure> {
    let identifier = require_identifier(&identifier)?;
    call_json(
        reqwest::Method::POST,
        "/api/auth/password/reset".into(),
        Some(json!({ "identifier": identifier, "code": code.trim(), "new_password": new_password })),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn check_account_username(username: String) -> Result<UsernameCheck, AccountFailure> {
    let query = url::form_urlencoded::byte_serialize(username.trim().as_bytes()).collect::<String>();
    let val = call_json(reqwest::Method::GET, format!("/api/auth/username/available?username={query}"), None).await?;
    Ok(UsernameCheck {
        available: val["available"].as_bool().unwrap_or(false),
        message: val["message"].as_str().map(String::from),
    })
}

#[tauri::command]
pub fn get_account_session(app: tauri::AppHandle) -> Option<AccountSession> {
    app.state::<AccountState>().session.lock().unwrap().clone()
}

#[tauri::command]
pub async fn logout_account(app: tauri::AppHandle) -> Result<(), String> {
    let session = app.state::<AccountState>().session.lock().unwrap().take();
    let _ = clear_session(&app);
    crate::ai_catalogue::account_changed(&app);
    let _ = app.emit("account-changed", None::<AccountSession>);

    if let Some(sess) = session {
        let url = format!("{}/api/auth/logout", account_base_url());
        let _ = tauri::async_runtime::spawn_blocking(move || {
            let payload = serde_json::to_string(&json!({
                "refresh_token": sess.refresh_token
            }))
            .ok()?;
            let _ = client()
                .ok()?
                .post(&url)
                .header("Content-Type", "application/json")
                .body(payload)
                .send();
            Some(())
        })
        .await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn login_body() -> Value {
        json!({
            "access_token": "a", "refresh_token": "r", "expires_in": 1800, "is_new_user": true,
            "user": { "uid": "123e4567-e89b-12d3-a456-426614174000", "username": "Alice_01", "avatar_url": null, "identities": [] }
        })
    }

    #[test]
    fn account_session_serialization_round_trip() {
        let user = AccountUser {
            uid: "123e4567-e89b-12d3-a456-426614174000".into(),
            username: "Alice_01".into(),
            avatar_url: Some("https://example.com/avatar.png".into()),
        };
        let session = AccountSession {
            access_token: "access.token.123".into(),
            refresh_token: "refresh.token.456".into(),
            expires_at: 1890000000,
            user,
        };

        let json_text = serde_json::to_string(&session).unwrap();
        let deserialized: AccountSession = serde_json::from_str(&json_text).unwrap();
        assert_eq!(session, deserialized);
        assert_eq!(deserialized.user.username, "Alice_01");
    }

    #[test]
    fn a_session_saved_by_an_older_version_is_treated_as_signed_out() {
        let old = r#"{"accessToken":"a","refreshToken":"r","expiresAt":1,"user":{"uid":"u","nickname":"阿伴","avatarUrl":null,"email":"a@b.com"}}"#;
        assert!(serde_json::from_str::<AccountSession>(old).is_err());
    }

    #[test]
    fn parse_error_response_keeps_the_message_and_the_machine_code() {
        let err_json = r#"{"error":true,"code":"username_taken","message":"这个用户名已被占用，换一个吧"}"#;
        let failure = parse_error_response(reqwest::StatusCode::CONFLICT, err_json);
        assert_eq!(failure, AccountFailure { message: "这个用户名已被占用，换一个吧".into(), code: Some("username_taken".into()) });

        let fallback = parse_error_response(reqwest::StatusCode::TOO_MANY_REQUESTS, "raw error");
        assert_eq!(fallback, AccountFailure { message: "请求过于频繁，请稍后再试".into(), code: None });
    }

    #[test]
    fn parse_login_reads_the_username_and_expiry() {
        let result = parse_login(&login_body(), 1000).unwrap();
        assert!(result.is_new_user);
        assert_eq!(result.session.user.username, "Alice_01");
        assert_eq!(result.session.expires_at, 2800);
        assert!(parse_login(&json!({ "access_token": "a", "refresh_token": "r", "user": { "uid": "u" } }), 0).is_err());
    }

    #[test]
    fn a_refresh_swaps_the_tokens_and_keeps_the_user() {
        let old = parse_login(&login_body(), 1000).unwrap().session;
        let next = parse_refresh(&old, &json!({ "access_token": "a2", "refresh_token": "r2", "token_type": "Bearer", "expires_in": 900 }), 5000).unwrap();
        assert_eq!((next.access_token.as_str(), next.refresh_token.as_str(), next.expires_at), ("a2", "r2", 5900));
        assert_eq!(next.user, old.user);
        assert!(parse_refresh(&old, &json!({ "access_token": "a2" }), 0).is_err());
    }

    #[test]
    fn a_token_close_to_expiry_counts_as_stale() {
        let session = parse_login(&login_body(), 0).unwrap().session;
        assert!(is_fresh(&session, 1000));
        assert!(!is_fresh(&session, session.expires_at - REFRESH_MARGIN_SECS));
        assert!(!is_fresh(&session, session.expires_at + 1));
    }

    #[test]
    fn code_login_distinguishes_an_existing_account_from_a_registration() {
        match parse_code_login(&login_body(), 0).unwrap() {
            CodeLoginOutcome::Session { session, .. } => assert_eq!(session.user.username, "Alice_01"),
            other => panic!("unexpected {other:?}"),
        }
        let pending = json!({ "registration_required": true, "registration_token": "tok", "expires_in": 900 });
        match parse_code_login(&pending, 0).unwrap() {
            CodeLoginOutcome::Registration { registration_token } => assert_eq!(registration_token, "tok"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn the_outcome_reaches_the_ui_with_a_kind_tag_in_camel_case() {
        let pending = CodeLoginOutcome::Registration { registration_token: "tok".into() };
        assert_eq!(serde_json::to_value(pending).unwrap(), json!({ "kind": "registration", "registrationToken": "tok" }));
    }
}

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
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub email: String,
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

#[derive(Default)]
pub struct AccountState {
    pub session: Mutex<Option<AccountSession>>,
}

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

fn parse_error_response(status: reqwest::StatusCode, body_text: &str) -> String {
    if let Ok(val) = serde_json::from_str::<Value>(body_text) {
        if let Some(msg) = val.get("message").and_then(Value::as_str) {
            return msg.to_string();
        }
    }
    match status.as_u16() {
        400 => "请求参数有误，请检查邮箱格式".into(),
        401 => "验证码错误或已过期".into(),
        429 => "请求过于频繁，请稍后再试".into(),
        500..=599 => "账号服务暂时不可用，请稍后重试".into(),
        _ => format!("账号服务返回状态异常 ({})", status.as_u16()),
    }
}

pub async fn send_code(email: &str) -> Result<SendCodeResult, String> {
    let email = email.trim().to_string();
    if !email.contains('@') || email.len() < 5 {
        return Err("请输入有效的邮箱地址".into());
    }

    let url = format!("{}/api/auth/code/send", account_base_url());
    tauri::async_runtime::spawn_blocking(move || {
        let payload = serde_json::to_string(&json!({
            "identifier": email,
            "purpose": "login"
        }))
        .map_err(|e| e.to_string())?;

        let res = client()?
            .post(&url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .map_err(|e| format!("发送验证码连接失败：{e}"))?;

        let status = res.status();
        let body = res
            .text()
            .map_err(|e| format!("读取账号服务响应失败：{e}"))?;

        if !status.is_success() {
            return Err(parse_error_response(status, &body));
        }

        let parsed: Value = serde_json::from_str(&body)
            .map_err(|e| format!("解析验证码响应失败：{e}"))?;

        Ok(SendCodeResult {
            ok: true,
            cooldown_seconds: parsed.get("cooldown_seconds").and_then(Value::as_u64).unwrap_or(60),
            expires_in: parsed.get("expires_in").and_then(Value::as_u64).unwrap_or(300),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn login_with_code_internal(
    email: &str,
    code: &str,
) -> Result<LoginResult, String> {
    let email = email.trim().to_string();
    let code = code.trim().to_string();
    if !email.contains('@') {
        return Err("请输入有效的邮箱地址".into());
    }
    if code.is_empty() {
        return Err("请输入验证码".into());
    }

    let url = format!("{}/api/auth/code/login", account_base_url());
    let device = device_name();

    tauri::async_runtime::spawn_blocking(move || {
        let payload = serde_json::to_string(&json!({
            "identifier": email,
            "code": code,
            "client_id": CLIENT_ID,
            "device_name": device
        }))
        .map_err(|e| e.to_string())?;

        let res = client()?
            .post(&url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .map_err(|e| format!("登录连接失败：{e}"))?;

        let status = res.status();
        let body = res
            .text()
            .map_err(|e| format!("读取登录响应失败：{e}"))?;

        if !status.is_success() {
            return Err(parse_error_response(status, &body));
        }

        let val: Value = serde_json::from_str(&body)
            .map_err(|e| format!("解析登录返回数据失败：{e}"))?;

        let access_token = val["access_token"].as_str().ok_or("缺少 access_token")?.to_string();
        let refresh_token = val["refresh_token"].as_str().ok_or("缺少 refresh_token")?.to_string();
        let expires_in = val["expires_in"].as_i64().unwrap_or(1800);
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let is_new_user = val["is_new_user"].as_bool().unwrap_or(false);
        let user_val = &val["user"];
        let uid = user_val["uid"].as_str().ok_or("缺少 uid")?.to_string();
        let nickname = user_val["nickname"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("小伴伙伴")
            .to_string();
        let avatar_url = user_val["avatar_url"].as_str().map(String::from);

        let user = AccountUser {
            uid,
            nickname,
            avatar_url,
            email,
        };

        let session = AccountSession {
            access_token,
            refresh_token,
            expires_at: now_sec + expires_in,
            user,
        };

        Ok(LoginResult {
            is_new_user,
            session,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn send_account_code(email: String) -> Result<SendCodeResult, String> {
    send_code(&email).await
}

#[tauri::command]
pub async fn login_account_with_code(
    app: tauri::AppHandle,
    email: String,
    code: String,
) -> Result<LoginResult, String> {
    let result = login_with_code_internal(&email, &code).await?;
    save_session(&app, &result.session)?;
    *app.state::<AccountState>().session.lock().unwrap() = Some(result.session.clone());
    let _ = app.emit("account-changed", Some(result.session.clone()));
    Ok(result)
}

#[tauri::command]
pub fn get_account_session(app: tauri::AppHandle) -> Option<AccountSession> {
    app.state::<AccountState>().session.lock().unwrap().clone()
}

#[tauri::command]
pub async fn logout_account(app: tauri::AppHandle) -> Result<(), String> {
    let session = app.state::<AccountState>().session.lock().unwrap().take();
    let _ = clear_session(&app);
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

    #[test]
    fn account_session_serialization_round_trip() {
        let user = AccountUser {
            uid: "123e4567-e89b-12d3-a456-426614174000".into(),
            nickname: "测试小伴用户".into(),
            avatar_url: Some("https://example.com/avatar.png".into()),
            email: "test@example.com".into(),
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
        assert_eq!(deserialized.user.email, "test@example.com");
    }

    #[test]
    fn parse_error_response_extracts_message() {
        let err_json = r#"{"error":true,"code":"unauthorized","message":"验证码错误或已失效"}"#;
        let msg = parse_error_response(reqwest::StatusCode::UNAUTHORIZED, err_json);
        assert_eq!(msg, "验证码错误或已失效");

        let fallback_msg = parse_error_response(reqwest::StatusCode::TOO_MANY_REQUESTS, "raw error");
        assert_eq!(fallback_msg, "请求过于频繁，请稍后再试");
    }
}

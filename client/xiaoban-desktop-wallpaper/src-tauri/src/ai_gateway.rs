//! 小伴 AI 网关（server-ai）的客户端：余额、兑换码，以及把它接成聊天可选的模型服务。
//! 访问令牌由账号模块保管并按需刷新，不会进入网页层，也不会写进 ai.json。
use crate::account::{self, AccountFailure};
use crate::ai_config::{self, AiConfig, AiConfigView};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tauri::{Emitter, Manager};

pub const DEFAULT_AI_URL: &str = "https://ai.xiangban.xyz";
/// 写在 `active_provider` 里的固定编号。它不在供应商列表里，因为地址、协议和密钥都不是用户可编辑的。
pub const GATEWAY_ID: &str = "xiaoban-ai";
const MAX_CODE_LEN: usize = 64;
const MAX_TIER_LEN: usize = 64;
/// 网关对三种协议都转发，但 Codex 这类渠道只支持 Responses，所以统一用它；档位背后换哪家模型，客户端都不用改。
const GATEWAY_PROTOCOL: &str = "responses";

/// 聊天每发一次模型请求就来取一次的密钥，让超过令牌有效期的长对话也能继续。
pub type KeySource = Arc<dyn Fn() -> String + Send + Sync>;

fn ai_base_url() -> String {
    std::env::var("XIAOBAN_AI_URL").unwrap_or_else(|_| DEFAULT_AI_URL.to_string()).trim_end_matches('/').to_string()
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Bucket {
    pub id: i64,
    /// `trial`（试用）、`monthly`（月卡）、`topup`（加油包）等，由网关决定。
    pub kind: String,
    pub total: i64,
    pub remaining: i64,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Trial {
    pub available: bool,
    pub quota: i64,
    pub days: i64,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProductInfo {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Balance {
    pub product: ProductInfo,
    /// 1 美元对应的额度点数，界面据此换算成美元。
    pub quota_per_unit: i64,
    pub total: i64,
    pub paid: i64,
    pub buckets: Vec<Bucket>,
    pub trial: Trial,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Redeemed {
    pub kind: String,
    pub quota: i64,
    pub expires_at: Option<String>,
    pub balance: i64,
}

fn failure_from(status: reqwest::StatusCode, body: &str) -> AccountFailure {
    let parsed = serde_json::from_str::<Value>(body).ok();
    let code = parsed.as_ref().and_then(|v| v.get("code")).and_then(Value::as_str).map(String::from);
    if let Some(message) = parsed.as_ref().and_then(|v| v.get("message")).and_then(Value::as_str) {
        return AccountFailure { message: message.to_string(), code };
    }
    let message = match status.as_u16() {
        429 => "请求过于频繁，请稍后再试".to_string(),
        500..=599 => "小伴 AI 服务暂时不可用，请稍后重试".to_string(),
        other => format!("小伴 AI 服务返回状态异常 ({other})"),
    };
    AccountFailure { message, code }
}

async fn call(app: &tauri::AppHandle, method: reqwest::Method, path: &str, payload: Option<Value>) -> Result<Value, AccountFailure> {
    let token = account::fresh_access_token(app).await?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("XiaoBanDesktop/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("创建网络连接失败：{e}"))?;
    let mut request = client.request(method, format!("{}{}", ai_base_url(), path)).bearer_auth(token);
    if let Some(payload) = payload {
        request = request.header("Content-Type", "application/json").body(serde_json::to_string(&payload).map_err(|e| e.to_string())?);
    }
    let response = request.send().await.map_err(|_| "连接小伴 AI 服务失败，请检查网络后重试")?;
    let status = response.status();
    let body = response.text().await.map_err(|_| "读取小伴 AI 服务响应失败")?;
    if !status.is_success() {
        return Err(failure_from(status, &body));
    }
    serde_json::from_str(&body).map_err(|_| AccountFailure::from("小伴 AI 服务返回的内容无法识别"))
}

#[tauri::command]
pub async fn ai_gateway_balance(app: tauri::AppHandle) -> Result<Balance, AccountFailure> {
    let value = call(&app, reqwest::Method::GET, "/api/ai/balance", None).await?;
    serde_json::from_value(value).map_err(|_| AccountFailure::from("小伴 AI 服务返回的余额格式不支持"))
}

#[tauri::command]
pub async fn ai_gateway_redeem(app: tauri::AppHandle, code: String) -> Result<Redeemed, AccountFailure> {
    let code = code.trim();
    if code.is_empty() {
        return Err("请输入兑换码".into());
    }
    if code.chars().count() > MAX_CODE_LEN {
        return Err("兑换码太长，请检查是否复制完整".into());
    }
    let value = call(&app, reqwest::Method::POST, "/api/ai/redeem", Some(json!({ "code": code }))).await?;
    serde_json::from_value(value).map_err(|_| AccountFailure::from("小伴 AI 服务返回的兑换结果格式不支持"))
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Tier {
    pub tier: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub is_default: bool,
    pub requires_paid: bool,
    /// 这个账号现在能不能用：付费档位要先有付费额度。
    pub available: bool,
}

#[tauri::command]
pub async fn ai_gateway_tiers(app: tauri::AppHandle) -> Result<Vec<Tier>, AccountFailure> {
    let mut value = call(&app, reqwest::Method::GET, "/api/ai/tiers", None).await?;
    serde_json::from_value(value["tiers"].take()).map_err(|_| AccountFailure::from("小伴 AI 服务返回的档位格式不支持"))
}

fn clean_tier(raw: &str) -> Result<String, String> {
    let tier = raw.trim();
    if tier.is_empty() {
        return Err("请先选择一个模型档位".into());
    }
    if tier.chars().count() > MAX_TIER_LEN || tier.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("模型档位不对".into());
    }
    Ok(tier.to_string())
}

/// 把聊天的“当前模型服务”切到小伴 AI。地址、协议和档位写进快照字段，聊天页据此判断已配置；
/// 令牌每次发送时才取，不落盘。
fn select(config: &mut AiConfig, tier: &str, base_url: &str) {
    config.active_provider = GATEWAY_ID.into();
    config.gateway_tier = tier.into();
    config.protocol = GATEWAY_PROTOCOL.into();
    config.base_url = base_url.into();
    config.model = tier.into();
    config.api_key.clear();
}

#[tauri::command]
pub fn use_ai_gateway(app: tauri::AppHandle, tier: String) -> Result<AiConfigView, String> {
    let tier = clean_tier(&tier)?;
    let path = ai_config::path(&app)?;
    let view = {
        let state = app.state::<ai_config::AiConfigState>();
        let _io = state.io.lock().unwrap();
        let mut config = ai_config::load(&path).unwrap_or_default();
        if !config.gateway_enabled {
            return Err("小伴 AI 入口已被关闭，暂时不能启用".into());
        }
        select(&mut config, &tier, &format!("{}/v1", ai_base_url()));
        ai_config::save(&path, &config).map_err(|e| format!("无法保存模型配置：{e}"))?;
        config.view(&ai_config::default_workspace(&app)?)
    };
    let _ = app.emit_to("control", "ai-config-changed", &view);
    Ok(view)
}

pub fn is_active(config: &AiConfig) -> bool {
    config.active_provider == GATEWAY_ID
}

/// Chat is set to 小伴 AI, but the site has closed that entry.
pub fn switched_off(config: &AiConfig) -> bool {
    is_active(config) && !config.gateway_enabled
}

/// 模型请求失败时内核给的是 `HTTP 402: {"error":true,"code":...,"message":"..."}`，
/// 聊天里只需要网关写给用户的那句话；认不出格式就原样返回。
pub fn readable_error(raw: &str) -> String {
    let Some(start) = raw.find('{') else { return raw.to_string() };
    serde_json::from_str::<Value>(&raw[start..])
        .ok()
        .and_then(|body| body.get("message").and_then(Value::as_str).filter(|m| !m.trim().is_empty()).map(str::to_string))
        .unwrap_or_else(|| raw.to_string())
}

/// 小伴 AI 生效时，换上本次要用的地址、档位和有效令牌；其他服务商原样返回。
fn resolve(mut config: AiConfig, token: &str, base_url: &str) -> Result<AiConfig, String> {
    let tier = clean_tier(&config.gateway_tier).map_err(|_| "还没选小伴 AI 的模型档位，请到“设置 → AI 模型”选择".to_string())?;
    select(&mut config, &tier, base_url);
    config.api_key = token.into();
    Ok(config)
}

/// 在聊天自己的线程里调用（会等待网络）。返回本次请求用的配置，以及长对话里续用令牌的取密钥函数。
pub fn prepare(app: &tauri::AppHandle, config: AiConfig) -> Result<(AiConfig, Option<KeySource>), String> {
    if !is_active(&config) {
        return Ok((config, None));
    }
    let token = tauri::async_runtime::block_on(account::fresh_access_token(app)).map_err(|f| f.message)?;
    let config = resolve(config, &token, &format!("{}/v1", ai_base_url()))?;
    let handle = app.clone();
    let source: KeySource = Arc::new(move || tauri::async_runtime::block_on(account::fresh_access_token(&handle)).unwrap_or_else(|_| token.clone()));
    Ok((config, Some(source)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://ai.example.com/v1";

    fn on_other_provider() -> AiConfig {
        AiConfig {
            active_provider: "deepseek".into(),
            protocol: "responses".into(),
            base_url: "https://api.deepseek.com".into(),
            model: "deepseek-v4-pro".into(),
            api_key: "sk-own-key".into(),
            gateway_tier: "premium".into(),
            ..AiConfig::default()
        }
    }

    #[test]
    fn choosing_the_gateway_replaces_the_snapshot_and_drops_the_old_key() {
        let mut config = on_other_provider();
        select(&mut config, "standard", BASE);
        assert!(is_active(&config));
        assert_eq!((config.protocol.as_str(), config.base_url.as_str(), config.model.as_str()), ("responses", BASE, "standard"));
        assert_eq!(config.gateway_tier, "standard");
        assert!(config.api_key.is_empty(), "the account token is never stored, nor is another provider's key");
        assert!(config.is_complete(), "the chat page treats it as configured");
    }

    #[test]
    fn chat_through_the_gateway_is_blocked_only_while_it_is_selected_and_the_site_has_closed_it() {
        let mut config = on_other_provider();
        config.gateway_enabled = false;
        assert!(!switched_off(&config), "another provider keeps working whatever the switch says");
        select(&mut config, "standard", BASE);
        assert!(switched_off(&config));
        config.gateway_enabled = true;
        assert!(!switched_off(&config));
    }

    #[test]
    fn the_other_providers_are_not_touched_when_the_gateway_is_chosen() {
        let mut config = on_other_provider();
        let before = config.providers.clone();
        select(&mut config, "standard", BASE);
        assert!(config.providers == before);
    }

    #[test]
    fn a_request_carries_the_fresh_token_and_the_chosen_tier() {
        let mut config = on_other_provider();
        select(&mut config, "premium", BASE);
        let resolved = resolve(config, "jwt.fresh.token", BASE).unwrap();
        assert_eq!((resolved.api_key.as_str(), resolved.model.as_str(), resolved.base_url.as_str()), ("jwt.fresh.token", "premium", BASE));
    }

    #[test]
    fn using_the_gateway_without_a_tier_is_refused_with_a_hint() {
        let mut config = on_other_provider();
        config.active_provider = GATEWAY_ID.into();
        config.gateway_tier.clear();
        assert!(resolve(config, "t", BASE).unwrap_err().contains("模型档位"));
    }

    #[test]
    fn tier_names_are_checked_before_they_are_saved() {
        assert_eq!(clean_tier("  standard ").unwrap(), "standard");
        for bad in ["", "   ", "a b", "bad\nname", &"x".repeat(65)] {
            assert!(clean_tier(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_gateway_error_is_shown_as_the_sentence_written_for_the_user() {
        let raw = r#"HTTP 402: {"error":true,"code":"insufficient_quota","message":"额度已用完，请兑换额度后继续使用"}"#;
        assert_eq!(readable_error(raw), "额度已用完，请兑换额度后继续使用");
    }

    #[test]
    fn an_error_in_another_shape_is_left_alone() {
        for raw in ["request failed: timed out", "HTTP 502: <html>bad gateway</html>", r#"HTTP 500: {"detail":"x"}"#, r#"HTTP 500: {"message":"  "}"#, "HTTP 500: {broken"] {
            assert_eq!(readable_error(raw), raw);
        }
    }

    #[test]
    fn a_saved_config_without_the_tier_field_still_loads() {
        let old = r#"{"activeProvider":"deepseek","baseUrl":"https://api.deepseek.com","model":"m"}"#;
        let config: AiConfig = serde_json::from_str(old).unwrap();
        assert!(config.gateway_tier.is_empty() && !is_active(&config));
    }

    #[test]
    fn tiers_arrive_in_camel_case_and_a_missing_description_is_fine() {
        let tiers: Vec<Tier> = serde_json::from_value(json!([
            { "tier": "standard", "display_name": "标准", "description": "日常对话", "is_default": true, "requires_paid": false, "available": true },
            { "tier": "premium", "display_name": "高级", "is_default": false, "requires_paid": true, "available": false }
        ])).unwrap();
        let ui = serde_json::to_value(&tiers).unwrap();
        assert_eq!(ui[0]["displayName"], "标准");
        assert_eq!(ui[1]["requiresPaid"], true);
        assert_eq!(ui[1]["description"], "");
        assert!(!tiers[1].available);
    }

    #[test]
    fn the_balance_reaches_the_ui_in_camel_case_without_internal_ids() {
        let server = json!({
            "product": { "product_id": "wallpaper", "name": "小伴壁纸" },
            "quota_per_unit": 500000, "total": 250000, "paid": 0,
            "buckets": [{ "id": 7, "kind": "trial", "total": 250000, "remaining": 250000, "expires_at": "2026-10-25T00:00:00Z", "created_at": "2026-10-18T00:00:00Z" }],
            "trial": { "available": false, "quota": 250000, "days": 7 }
        });
        let balance: Balance = serde_json::from_value(server).unwrap();
        let ui = serde_json::to_value(&balance).unwrap();
        assert_eq!(ui["quotaPerUnit"], 500000);
        assert_eq!(ui["product"], json!({ "name": "小伴壁纸" }));
        assert_eq!(ui["buckets"][0]["expiresAt"], "2026-10-25T00:00:00Z");
        assert_eq!(ui["trial"], json!({ "available": false, "quota": 250000, "days": 7 }));
    }

    #[test]
    fn a_bucket_without_an_expiry_is_allowed() {
        let bucket: Bucket = serde_json::from_value(json!({ "id": 1, "kind": "topup", "total": 10, "remaining": 4, "expires_at": null, "created_at": "2026-10-18T00:00:00Z" })).unwrap();
        assert_eq!(bucket.expires_at, None);
    }

    #[test]
    fn a_redeem_result_is_read_from_the_server_shape() {
        let done: Redeemed = serde_json::from_value(json!({ "ok": true, "kind": "monthly", "quota": 5000000, "expires_at": "2026-11-18T00:00:00Z", "balance": 5250000 })).unwrap();
        assert_eq!((done.kind.as_str(), done.quota, done.balance), ("monthly", 5000000, 5250000));
        assert_eq!(serde_json::to_value(&done).unwrap()["expiresAt"], "2026-11-18T00:00:00Z");
    }

    #[test]
    fn server_errors_keep_their_message_and_machine_code() {
        let body = r#"{"error":true,"code":"too_many_attempts","message":"尝试次数过多，请稍后再试"}"#;
        assert_eq!(
            failure_from(reqwest::StatusCode::TOO_MANY_REQUESTS, body),
            AccountFailure { message: "尝试次数过多，请稍后再试".into(), code: Some("too_many_attempts".into()) }
        );
        let proxy_page = failure_from(reqwest::StatusCode::BAD_GATEWAY, "<html>502</html>");
        assert_eq!(proxy_page, AccountFailure { message: "小伴 AI 服务暂时不可用，请稍后重试".into(), code: None });
    }
}

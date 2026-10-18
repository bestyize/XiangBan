//! Native-only catalogue synchronization. Credentials never enter the webview.
use crate::ai_config::{self, AiConfig, AiConfigView, ModelConfig, ProviderConfig};
use serde::Deserialize;
use std::{collections::{BTreeMap, HashSet}, time::{Duration, SystemTime, UNIX_EPOCH}};
use tauri::{Emitter, Manager};

#[derive(Default)]
pub struct CatalogueState {
    operation: tauri::async_runtime::Mutex<()>,
    attempted_at: std::sync::Mutex<u64>,
    last_result: std::sync::Mutex<Option<Result<AiConfigView, String>>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalogue {
    version: u32,
    refresh_after_seconds: u64,
    /// Missing from catalogues of sites that predate the switch, which means on.
    #[serde(default)]
    xiaoban_ai: XiaobanAiSwitch,
    providers: Vec<RemoteProvider>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct XiaobanAiSwitch {
    #[serde(default = "on")]
    enabled: bool,
}
impl Default for XiaobanAiSwitch {
    fn default() -> Self { Self { enabled: true } }
}
fn on() -> bool { true }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteProvider {
    id: String, provider: String, #[serde(default)] description: String,
    end_point: String, protocol: String, #[serde(rename = "type")] provider_type: String,
    #[serde(default)] apikey: String, default_model: String, models: Vec<ModelConfig>,
    #[serde(default)] endpoints: BTreeMap<String, String>,
    #[serde(default)] console_url: String, #[serde(default)] console_text: String,
}
fn protocol(value: &str) -> Result<String, String> {
    match value {
        "OpenAI Responses" => Ok("responses".into()),
        "OpenAI Chat Completions" => Ok("chat".into()),
        "Anthropic Messages" => Ok("anthropic".into()),
        _ => Err("供应商使用了当前客户端不支持的协议".into()),
    }
}
fn address(raw: &str) -> bool {
    url::Url::parse(raw).is_ok_and(|u| u.scheme() == "https" && u.host_str().is_some()
        && u.username().is_empty() && u.password().is_none() && u.query().is_none() && u.fragment().is_none())
}
pub fn merge(config: &mut AiConfig, catalogue: Catalogue) -> Result<(), String> {
    if catalogue.version != 1 || !(300..=86400).contains(&catalogue.refresh_after_seconds) || catalogue.providers.len() > 50 {
        return Err("模型配置版本或刷新周期不支持".into());
    }
    let mut next = config.clone();
    let mut ids = HashSet::new();
    for p in catalogue.providers {
        if p.id.is_empty() || p.id == "custom" || p.id.starts_with("custom_") || !ids.insert(p.id.clone())
            || !matches!(p.provider_type.as_str(), "third" | "free" | "paid") || !address(&p.end_point)
            || !p.models.iter().any(|m| m.id == p.default_model) || p.models.len() > 200 {
            return Err("模型供应商配置无效".into());
        }
        let mut models = HashSet::new();
        if p.models.iter().any(|m| m.id.trim().is_empty() || !models.insert(m.id.clone())) { return Err("模型列表无效".into()); }
        if p.provider_type == "paid" && p.apikey.is_empty() { return Err("官方服务未提供调用凭证".into()); }
        let proto = protocol(&p.protocol)?;
        let mut endpoints = BTreeMap::new();
        for (k, v) in p.endpoints { if !address(&v) { return Err("协议地址无效".into()); } endpoints.insert(protocol(&k)?, v.trim_end_matches('/').to_string()); }
        endpoints.entry(proto.clone()).or_insert_with(|| p.end_point.trim_end_matches('/').to_string());
        let index = next.providers.iter().position(|v| v.id == p.id);
        let mut local = index.map(|i| next.providers[i].clone()).unwrap_or_default();
        if index.is_some() && local.is_custom() { return Err("远端供应商 ID 与本地自定义配置冲突".into()); }
        let first = index.is_none();
        let old_type = if !local.is_custom() && local.provider_type == "custom" { "third".into() } else { local.provider_type.clone() };
        // The user may have pointed a third-party preset at their own proxy. Keep that address/key.
        if p.provider_type != "third" || first || local.api_key.is_empty() || old_type != "third" {
            local.protocol = proto.clone();
            local.base_url = p.end_point.trim_end_matches('/').to_string();
        }
        if p.provider_type != "third" { local.api_key = p.apikey; }
        else if old_type != "third" && old_type != "custom" { local.api_key.clear(); }
        if first { local.selection_mode = "recommended".into(); }
        if first || local.selection_mode == "recommended" { local.model = p.default_model.clone(); }
        local.id = p.id; local.name = p.provider; local.description = p.description;
        local.provider_type = p.provider_type; local.remote = true; local.available = true;
        local.default_model = p.default_model; local.models = p.models; local.endpoints = endpoints;
        local.console_url = p.console_url; local.console_text = p.console_text;
        if let Some(i) = index { next.providers[i] = local; } else { next.providers.push(local); }
    }
    for p in &mut next.providers {
        let legacy = ["deepseek", "zhipu", "kimi", "minimax"].contains(&p.id.as_str());
        if (p.remote || legacy) && !ids.contains(&p.id) {
            p.remote = true; p.available = false;
            if p.provider_type != "third" { p.api_key.clear(); }
        }
    }
    next.catalogue_received = true;
    next.gateway_enabled = catalogue.xiaoban_ai.enabled;
    next.catalogue_refresh_after = catalogue.refresh_after_seconds;
    activate_snapshot(&mut next);
    *config = next;
    Ok(())
}
fn activate_snapshot(config: &mut AiConfig) {
    if let Some(p) = config.providers.iter().find(|p| p.id == config.active_provider) {
        config.protocol = p.protocol.clone(); config.base_url = p.base_url.clone();
        config.model = p.model.clone(); config.api_key = p.api_key.clone();
    }
}
fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn owner(app: &tauri::AppHandle) -> String {
    crate::account::get_account_session(app.clone()).map(|s| s.user.uid).unwrap_or_default()
}
/// Called synchronously when the account changes, including logout, before any new chat can start.
pub fn account_changed(app: &tauri::AppHandle) {
    let Ok(path) = ai_config::path(app) else { return };
    let state = app.state::<ai_config::AiConfigState>();
    let _io = state.io.lock().unwrap();
    if let Ok(mut config) = ai_config::load(&path) {
        let uid = owner(app);
        if config.catalogue_owner == uid { return; }
        for p in &mut config.providers {
            if p.remote && (p.provider_type == "paid" || (p.provider_type == "free" && !p.api_key.is_empty())) {
                p.api_key.clear();
                p.available = false;
            }
        }
        config.catalogue_owner = uid; config.catalogue_fetched_at = 0; config.catalogue_etag.clear();
        activate_snapshot(&mut config);
        let _ = ai_config::save(&path, &config);
    }
}
pub fn usable(app: &tauri::AppHandle, config: &AiConfig) -> Result<(), String> {
    if crate::ai_gateway::switched_off(config) {
        return Err("小伴 AI 入口已被关闭，请到“设置 → AI 模型”选择其他服务商".into());
    }
    if let Some(p) = config.providers.iter().find(|p| p.id == config.active_provider) {
        if !p.available { return Err("当前模型供应商已停用，请切换模型".into()); }
        if p.remote && p.provider_type == "paid" {
            if config.catalogue_owner != owner(app) || config.catalogue_owner.is_empty() || p.api_key.is_empty()
                || !crate::account::get_account_session(app.clone()).is_some_and(|s| s.expires_at > now() as i64) {
                return Err("请登录并刷新模型配置后使用官方服务".into());
            }
        } else if p.remote && p.provider_type == "free" && !p.api_key.is_empty() {
            if config.catalogue_owner != owner(app) || config.catalogue_owner.is_empty()
                || !crate::account::get_account_session(app.clone()).is_some_and(|s| s.expires_at > now() as i64) {
                return Err("请登录并刷新模型配置后使用官方服务".into());
            }
        }
        if p.remote && p.provider_type != "third" {
            if !p.models.iter().any(|m| m.id == p.model) { return Err("当前模型已停用，请切换模型".into()); }
        }
    }
    Ok(())
}
#[tauri::command]
pub async fn refresh_ai_providers(app: tauri::AppHandle, force: Option<bool>) -> Result<AiConfigView, String> {
    let state = app.state::<CatalogueState>();
    let _operation = match state.operation.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            let _guard = state.operation.lock().await;
            let last = state.last_result.lock().unwrap().clone();
            return last.unwrap_or_else(|| ai_config::get_ai_config(app.clone()));
        }
    };
    let result = refresh_inner(app.clone(), force).await;
    *state.last_result.lock().unwrap() = Some(result.clone());
    result
}
async fn refresh_inner(app: tauri::AppHandle, force: Option<bool>) -> Result<AiConfigView, String> {
    let state = app.state::<CatalogueState>();
    account_changed(&app);
    let current = ai_config::current(&app)?;
    let uid = owner(&app);
    let time = now();
    let attempted = *state.attempted_at.lock().unwrap();
    if !force.unwrap_or(false) && (time.saturating_sub(current.catalogue_fetched_at) < current.catalogue_refresh_after
        || time.saturating_sub(attempted) < 60) {
        return ai_config::get_ai_config(app.clone());
    }
    *state.attempted_at.lock().unwrap() = time;
    let origin = crate::website::site_url(&crate::settings(&app).website_url)?;
    if origin != crate::website::DEFAULT_SITE && !app.state::<crate::DesktopState>().isolated {
        return Err("登录模型配置只能从官方站点获取".into());
    }
    let client = reqwest::Client::builder().timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()).build().map_err(|_| "无法创建模型配置连接")?;
    let mut request = client.get(format!("{origin}/api/v1/ai/providers"));
    if let Some(session) = crate::account::get_account_session(app.clone()) { request = request.bearer_auth(session.access_token); }
    if !current.catalogue_etag.is_empty() { request = request.header(reqwest::header::IF_NONE_MATCH, &current.catalogue_etag); }
    let response = request.send().await.map_err(|_| "模型配置拉取失败，请检查网络后重试")?;
    let status = response.status();
    let etag = response.headers().get(reqwest::header::ETAG).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let catalogue = if status == reqwest::StatusCode::NOT_MODIFIED {
        if !current.catalogue_received { return Err("模型配置缓存不存在，请重新拉取".into()); } None
    } else {
        if !status.is_success() { return Err(if status == reqwest::StatusCode::UNAUTHORIZED { "登录凭证已失效，请重新登录" } else { "模型配置服务暂不可用，请稍后重试" }.into()); }
        if response.content_length().is_some_and(|n| n > 1024 * 1024) { return Err("模型配置过大".into()); }
        let bytes = response.bytes().await.map_err(|_| "模型配置读取失败")?;
        if bytes.len() > 1024 * 1024 { return Err("模型配置过大".into()); }
        Some(serde_json::from_slice::<Catalogue>(&bytes).map_err(|_| "模型配置格式不支持")?)
    };
    if uid != owner(&app) { return Err("账号已切换，请重新拉取模型配置".into()); }
    let view = {
        let path = ai_config::path(&app)?;
        let io = app.state::<ai_config::AiConfigState>(); let _io = io.io.lock().unwrap();
        let mut next = ai_config::load(&path).map_err(|_| "模型配置读取失败")?;
        if let Some(catalogue) = catalogue { merge(&mut next, catalogue)?; }
        next.catalogue_owner = uid; next.catalogue_fetched_at = time; next.catalogue_etag = etag;
        ai_config::save(&path, &next).map_err(|_| "模型配置保存失败")?;
        next.view(&ai_config::default_workspace(&app)?)
    };
    let _ = app.emit_to("control", "ai-config-changed", &view);
    Ok(view)
}
pub fn start(app: &tauri::AppHandle) {
    let handle = app.clone();
    std::thread::spawn(move || {
        let _ = tauri::async_runtime::block_on(refresh_ai_providers(handle.clone(), Some(true)));
        loop {
            std::thread::sleep(Duration::from_secs(60));
            let _ = tauri::async_runtime::block_on(refresh_ai_providers(handle.clone(), None));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn catalogue(model: &str) -> Catalogue {
        serde_json::from_value(serde_json::json!({"version":1,"refreshAfterSeconds":21600,"providers":[{
            "id":"deepseek","provider":"DeepSeek","endPoint":"https://api.deepseek.com","protocol":"OpenAI Chat Completions",
            "type":"third","defaultModel":model,"models":[{"id":model,"name":model}]}]})).unwrap()
    }
    #[test] fn recommended_updates_but_manual_and_keys_survive() {
        let mut c = AiConfig::default();
        c.providers[0].api_key = "user-key".into();
        c.providers[0].selection_mode = "manual".into(); c.providers[0].model = "chosen".into();
        merge(&mut c, catalogue("new-default")).unwrap();
        assert_eq!(c.providers[0].model, "chosen"); assert_eq!(c.providers[0].api_key, "user-key");
        c.providers[0].selection_mode = "recommended".into();
        merge(&mut c, catalogue("next-default")).unwrap(); assert_eq!(c.providers[0].model, "next-default");
    }
    #[test] fn the_xiaoban_ai_entry_follows_the_catalogue_and_is_on_when_the_site_says_nothing() {
        let mut c = AiConfig::default();
        assert!(c.gateway_enabled, "on before any catalogue arrived");
        let mut off = catalogue("m");
        off.xiaoban_ai.enabled = false;
        merge(&mut c, off).unwrap();
        assert!(!c.gateway_enabled);
        merge(&mut c, catalogue("m")).unwrap();
        assert!(c.gateway_enabled, "a catalogue that does not mention it turns it back on");

        let silent: Catalogue = serde_json::from_value(serde_json::json!({"version":1,"refreshAfterSeconds":21600,"providers":[]})).unwrap();
        assert!(silent.xiaoban_ai.enabled, "an older site sends no field");
        let empty: Catalogue = serde_json::from_value(serde_json::json!({"version":1,"refreshAfterSeconds":21600,"xiaobanAi":{},"providers":[]})).unwrap();
        assert!(empty.xiaoban_ai.enabled);
    }
    #[test] fn a_refused_catalogue_does_not_switch_the_entry() {
        let mut c = AiConfig::default();
        let mut bad = catalogue("m");
        bad.xiaoban_ai.enabled = false;
        bad.providers[0].protocol = "unknown".into();
        assert!(merge(&mut c, bad).is_err());
        assert!(c.gateway_enabled);
    }
    #[test] fn invalid_response_does_not_destroy_cache() {
        let mut c = AiConfig::default(); let before = c.clone(); let mut invalid = catalogue("m");
        invalid.providers[0].protocol = "unknown".into();
        assert!(merge(&mut c, invalid).is_err()); assert_eq!(c, before);
    }
}

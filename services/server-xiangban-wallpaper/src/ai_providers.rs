//! Model service catalogue, separate from wallpaper source providers.
use crate::{api::AppState, auth::OptionalAuthUser, error::AppError, store::Store};
use axum::{extract::State, http::{header, HeaderMap, HeaderValue, StatusCode}, response::{IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

const KEY: &str = "ai.providers";
fn enabled() -> bool { true }
fn interval() -> u64 { 21600 }

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: String,
    pub name: String,
    #[serde(default)] pub description: String,
    #[serde(default = "enabled")] pub enabled: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccess {
    pub uid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub apikey: Option<String>,
    #[serde(default)] pub has_key: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub provider: String,
    #[serde(default)] pub description: String,
    pub end_point: String,
    pub protocol: String,
    #[serde(rename = "type")] pub provider_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub apikey: Option<String>,
    #[serde(default)] pub has_key: bool,
    #[serde(default)] pub clear_key: bool,
    #[serde(default = "enabled")] pub enabled: bool,
    pub default_model: String,
    pub models: Vec<Model>,
    #[serde(default)] pub endpoints: BTreeMap<String, String>,
    #[serde(default)] pub console_url: String,
    #[serde(default)] pub console_text: String,
    /// Paid credentials are explicitly granted per user, never a shared upstream master key.
    #[serde(default)] pub user_access: Vec<UserAccess>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalogue {
    pub version: u32,
    pub revision: u64,
    #[serde(default = "interval")] pub refresh_after_seconds: u64,
    pub providers: Vec<Provider>,
}
impl Default for Catalogue {
    fn default() -> Self {
        serde_json::from_str(include_str!("ai-provider-defaults.json")).expect("valid built-in AI catalogue")
    }
}
fn bad(message: &str) -> AppError { AppError::BadRequest(message.into()) }
fn protocol(value: &str) -> bool {
    matches!(value, "OpenAI Responses" | "OpenAI Chat Completions" | "Anthropic Messages")
}
fn address(value: &str) -> bool {
    reqwest::Url::parse(value).is_ok_and(|url| url.scheme() == "https" && url.host_str().is_some()
        && url.username().is_empty() && url.password().is_none() && url.query().is_none() && url.fragment().is_none())
}
impl Catalogue {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.version != 1 || !(300..=86400).contains(&self.refresh_after_seconds) || self.providers.len() > 50 {
            return Err(bad("配置版本、刷新周期或供应商数量无效"));
        }
        let mut ids = HashSet::new();
        for p in &self.providers {
            if p.id.is_empty() || p.id.len() > 80 || p.id == "custom" || p.id.starts_with("custom_")
                || !p.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || !ids.insert(&p.id) || p.provider.trim().is_empty() || p.provider.len() > 200
                || p.description.len() > 2000 {
                return Err(bad("供应商 ID 或名称无效、重复；custom 为本地保留类型"));
            }
            if !matches!(p.provider_type.as_str(), "third" | "free" | "paid") || !protocol(&p.protocol) {
                return Err(bad("供应商类型或协议不支持"));
            }
            if p.enabled && !address(&p.end_point) { return Err(bad("已启用的供应商必须填写有效 HTTPS 基础地址")); }
            if p.endpoints.iter().any(|(k, v)| !protocol(k) || !address(v)) {
                return Err(bad("协议地址映射无效"));
            }
            if !p.console_url.is_empty() && !address(&p.console_url) { return Err(bad("控制台链接必须使用 HTTPS")); }
            let mut models = HashSet::new();
            if p.models.len() > 200 { return Err(bad("单个供应商最多配置 200 个模型")); }
            for m in &p.models {
                if m.id.trim().is_empty() || m.id.len() > 200 || m.id != m.id.trim()
                    || m.name.trim().is_empty() || m.name.len() > 200 || m.description.len() > 2000
                    || !models.insert(&m.id) { return Err(bad("模型 ID、名称无效或重复")); }
            }
            if p.enabled && !p.models.iter().any(|m| m.enabled && m.id == p.default_model) {
                return Err(bad("默认模型必须存在且已启用"));
            }
            if p.apikey.as_ref().is_some_and(|key| key.len() > 2048 || key.chars().any(char::is_whitespace)) {
                return Err(bad("凭证包含空白字符或过长"));
            }
            if p.enabled && p.provider_type == "free" && p.apikey.as_deref().unwrap_or("").is_empty() {
                return Err(bad("免费服务需要配置专用的受限调用凭证"));
            }
            let mut users = HashSet::new();
            for access in &p.user_access {
                if uuid::Uuid::parse_str(&access.uid).is_err() || !users.insert(&access.uid)
                    || access.apikey.as_deref().unwrap_or("").is_empty()
                    || access.apikey.as_ref().is_some_and(|k| k.len() > 2048 || k.chars().any(char::is_whitespace)) {
                    return Err(bad("付费授权需要唯一的用户 UID 和该用户的受限凭证"));
                }
            }
        }
        Ok(())
    }
    pub fn redacted(&self) -> Self {
        let mut view = self.clone();
        for p in &mut view.providers {
            p.has_key = p.apikey.as_ref().is_some_and(|k| !k.is_empty());
            p.apikey = None;
            for a in &mut p.user_access {
                a.has_key = a.apikey.as_ref().is_some_and(|k| !k.is_empty());
                a.apikey = None;
            }
        }
        view
    }
    fn merge_credentials(&mut self, previous: &Self) {
        for p in &mut self.providers {
            let old = previous.providers.iter().find(|v| v.id == p.id);
            if p.provider_type == "third" || p.clear_key {
                p.apikey = None;
            } else if p.apikey.as_deref().is_none_or(str::is_empty) {
                p.apikey = old.and_then(|o| o.apikey.clone());
            }
            p.clear_key = false;
            for a in &mut p.user_access {
                if a.apikey.as_deref().is_none_or(str::is_empty) {
                    a.apikey = old.and_then(|o| o.user_access.iter().find(|v| v.uid == a.uid)).and_then(|v| v.apikey.clone());
                }
            }
            if p.provider_type != "paid" { p.user_access.clear(); }
            if p.provider_type == "paid" { p.apikey = None; }
        }
    }
    pub fn public(&self, uid: Option<&str>) -> serde_json::Value {
        let providers: Vec<_> = self.providers.iter().filter(|p| p.enabled).filter_map(|p| {
            let key = match p.provider_type.as_str() {
                "third" => None,
                "free" => { uid?; p.apikey.clone() },
                "paid" => Some(p.user_access.iter().find(|a| Some(a.uid.as_str()) == uid)?.apikey.clone()?),
                _ => return None,
            };
            let models: Vec<_> = p.models.iter().filter(|m| m.enabled)
                .map(|m| serde_json::json!({"id":m.id,"name":m.name,"description":m.description})).collect();
            let mut value = serde_json::json!({
                "id":p.id,"provider":p.provider,"description":p.description,"endPoint":p.end_point,
                "protocol":p.protocol,"type":p.provider_type,"defaultModel":p.default_model,
                "models":models,"endpoints":p.endpoints,"consoleUrl":p.console_url,"consoleText":p.console_text
            });
            if let Some(key) = key { value["apikey"] = key.into(); }
            Some(value)
        }).collect();
        serde_json::json!({"version":1,"revision":self.revision.to_string(),
            "refreshAfterSeconds":self.refresh_after_seconds,"providers":providers})
    }
}
async fn load(store: &Store) -> Result<Catalogue, AppError> {
    match store.get_setting(KEY).await? {
        Some(value) => serde_json::from_str(&value).map_err(|_| AppError::Internal("模型配置无法读取".into())),
        None => Ok(Catalogue::default()),
    }
}
pub async fn admin_get(State(state): State<AppState>) -> Result<Json<Catalogue>, AppError> {
    Ok(Json(load(state.admin.store()?).await?.redacted()))
}
pub async fn admin_save(State(state): State<AppState>, Json(mut value): Json<Catalogue>) -> Result<Json<Catalogue>, AppError> {
    let result = state.admin.store()?.update_ai_catalogue(|old| {
        if value.revision != old.revision { return Err(bad("配置已被其他管理员修改，请重新加载后保存")); }
        value.merge_credentials(old);
        value.validate()?;
        value.revision = old.revision.checked_add(1).ok_or_else(|| bad("配置版本已达上限"))?;
        *old = value;
        Ok(())
    }).await?;
    Ok(Json(result.redacted()))
}
pub async fn get(State(state): State<AppState>, auth: OptionalAuthUser, headers: HeaderMap) -> Result<Response, AppError> {
    let uid = auth.0.map(|u| u.uid.to_string());
    let value = load(state.admin.store()?).await?.public(uid.as_deref());
    let body = serde_json::to_vec(&value).map_err(|_| AppError::Internal("模型配置无法序列化".into()))?;
    let etag = format!("\"{:x}\"", Sha256::digest(&body));
    let same = headers.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()).is_some_and(|v| v == etag);
    let mut response = if same { StatusCode::NOT_MODIFIED.into_response() } else { Json(value).into_response() };
    response.headers_mut().insert(header::ETAG, HeaderValue::from_str(&etag).map_err(|_| AppError::Internal("配置校验标识无效".into()))?);
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("private, no-cache"));
    response.headers_mut().insert(header::VARY, HeaderValue::from_static("Authorization"));
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    const UID: &str = "00000000-0000-0000-0000-000000000001";
    fn configured() -> Catalogue {
        let mut c = Catalogue::default();
        let mut p = c.providers[0].clone();
        p.id = "official".into(); p.provider_type = "free".into(); p.apikey = Some("restricted-token".into());
        c.providers.push(p); c
    }
    #[test] fn defaults_are_valid_and_never_send_user_keys() {
        let c = Catalogue::default(); c.validate().unwrap();
        assert!(c.public(None)["providers"].as_array().unwrap().iter().all(|p| p.get("apikey").is_none()));
    }
    #[test] fn paid_keys_are_only_returned_to_the_granted_user() {
        let mut c = configured(); let p = c.providers.last_mut().unwrap();
        p.provider_type = "paid".into(); p.user_access.push(UserAccess {uid:UID.into(),apikey:Some("user-token".into()),has_key:false});
        assert_eq!(c.public(None)["providers"].as_array().unwrap().len(), 4);
        assert_eq!(c.public(Some(UID))["providers"][4]["apikey"], "user-token");
        assert_eq!(c.public(Some("other"))["providers"].as_array().unwrap().len(), 4);
        assert!(!serde_json::to_string(&c.redacted()).unwrap().contains("user-token"));
    }
    #[test] fn blank_admin_key_preserves_secret_and_default_must_be_enabled() {
        let old = configured(); let mut next = old.redacted(); next.merge_credentials(&old);
        assert_eq!(next.providers[4].apikey.as_deref(), Some("restricted-token"));
        next.providers[4].models[0].enabled = false;
        assert!(next.validate().is_err());
    }
}

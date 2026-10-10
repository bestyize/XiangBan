use super::auth::{AdminSession, SESSION_COOKIE, SESSION_COOKIE_MAX_AGE};
use crate::{
    api::state::AppState,
    error::AppError,
    hotlink::{self, Kind, Overrides, Policy},
    runtime::{self, is_known_provider, CacheSettings, ProxyStrategy, PROVIDER_IDS},
    store::{KeyRow, ProxyRow},
};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_KEYS_PER_PROVIDER: i64 = 50;
const MAX_PROXIES: usize = 100;

type ApiResult = Result<Json<Value>, AppError>;

fn provider_name(id: &str) -> &'static str {
    match id {
        "unsplash" => "Unsplash",
        "pixabay" => "Pixabay",
        "pexels" => "Pexels",
        "okslop" => "okslop",
        "bing" => "Bing",
        _ => "",
    }
}

fn require_provider(id: &str) -> Result<(), AppError> {
    if is_known_provider(id) {
        Ok(())
    } else {
        Err(AppError::NotFound(format!("未知的供应商：{id}")))
    }
}

/// Pushes saved changes into the running service. Search results are dropped whenever the set of
/// usable providers changes, since the cached ones were assembled from the old set.
async fn apply_changes(state: &AppState, clear_cache: bool) -> Result<(), AppError> {
    state.runtime.reload(state.admin.store()?).await?;
    if clear_cache {
        state.cache.clear().await?;
    }
    Ok(())
}

/* ---- input rules ---- */

fn clean_label(raw: &str) -> Result<String, AppError> {
    let label = raw.trim();
    let length = label.chars().count();
    if length == 0 || length > 40 || label.chars().any(char::is_control) {
        return Err(AppError::BadRequest("名称需为 1 到 40 个字符".to_string()));
    }
    Ok(label.to_string())
}

fn clean_api_key(raw: &str) -> Result<String, AppError> {
    let key = raw.trim();
    if !(8..=256).contains(&key.len()) || !key.chars().all(|c| c.is_ascii_graphic()) {
        return Err(AppError::BadRequest("密钥需为 8 到 256 位，且不能包含空格或不可见字符".to_string()));
    }
    Ok(key.to_string())
}

fn clean_proxy_url(raw: &str) -> Result<String, AppError> {
    let invalid = || {
        AppError::BadRequest(
            "代理地址格式不正确，例如 http://127.0.0.1:7890 或 socks5://user:pass@host:1080".to_string(),
        )
    };
    let text = raw.trim();
    if text.is_empty() || text.len() > 512 {
        return Err(invalid());
    }
    let url = reqwest::Url::parse(text).map_err(|_| invalid())?;
    if !matches!(url.scheme(), "http" | "https" | "socks5" | "socks5h") || url.host_str().is_none() {
        return Err(invalid());
    }
    reqwest::Proxy::all(text).map_err(|_| invalid())?;
    Ok(text.to_string())
}

/* ---- views: keys and proxy passwords are never sent back in full ---- */

fn key_preview(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() >= 12 {
        let head: String = chars[..4].iter().collect();
        let tail: String = chars[chars.len() - 4..].iter().collect();
        format!("{head}…{tail}")
    } else {
        "••••••••".to_string()
    }
}

/// The proxy address without its username and password, and whether it had any.
fn mask_proxy_url(raw: &str) -> (String, bool) {
    match reqwest::Url::parse(raw) {
        Ok(mut url) => {
            let has_auth = !url.username().is_empty() || url.password().is_some();
            let _ = url.set_username("");
            let _ = url.set_password(None);
            (url.as_str().trim_end_matches('/').to_string(), has_auth)
        }
        Err(_) => ("（无效地址）".to_string(), false),
    }
}

fn key_view(state: &AppState, key: &KeyRow) -> Value {
    let health = state.runtime.key_health(key.id);
    json!({
        "id": key.id,
        "label": key.label,
        "preview": key_preview(&key.api_key),
        "enabled": key.enabled,
        "created_at": key.created_at,
        "requests": health.requests,
        "failures": health.failures,
        "last_status": health.last_status,
        "cooldown_secs": health.cooldown_remaining().map(|d| d.as_secs() + 1),
    })
}

fn proxy_view(state: &AppState, proxy: &ProxyRow) -> Value {
    let health = state.runtime.proxy_health(proxy.id);
    let (url, has_auth) = mask_proxy_url(&proxy.url);
    json!({
        "id": proxy.id,
        "name": proxy.name,
        "url": url,
        "has_auth": has_auth,
        "enabled": proxy.enabled,
        "created_at": proxy.created_at,
        "requests": health.requests,
        "failures": health.failures,
        "last_error": health.last_error,
        "cooldown_secs": health.cooldown_remaining().map(|d| d.as_secs() + 1),
    })
}

/* ---- sign in / out ---- */

fn session_cookie(token: &str, max_age: u64, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    // The token is hex and the attributes are fixed, so this is always a valid header value.
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}={token}; Path=/api/v1/admin; HttpOnly; SameSite=Strict; Max-Age={max_age}{secure}"
    ))
    .expect("session cookie is valid ASCII")
}

fn is_https(headers: &HeaderMap) -> bool {
    headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("https"))
}

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    username: String,
    password: String,
}

pub async fn login(State(state): State<AppState>, headers: HeaderMap, Json(body): Json<LoginBody>) -> Result<Response, AppError> {
    let (token, info) = state.admin.login(body.username.trim(), &body.password).await?;
    let mut response = Json(info).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, session_cookie(&token, SESSION_COOKIE_MAX_AGE, is_https(&headers)));
    Ok(response)
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap, Extension(session): Extension<AdminSession>) -> Response {
    state.admin.logout(&session.token);
    let mut response = Json(json!({ "success": true })).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, session_cookie("", 0, is_https(&headers)));
    response
}

pub async fn me(State(state): State<AppState>) -> ApiResult {
    Ok(Json(json!(state.admin.account_info().await?)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountBody {
    current_password: String,
    new_username: Option<String>,
    new_password: Option<String>,
}

pub async fn update_account(
    State(state): State<AppState>,
    Extension(session): Extension<AdminSession>,
    Json(body): Json<UpdateAccountBody>,
) -> ApiResult {
    let new_username = body.new_username.as_deref().map(str::trim).filter(|n| !n.is_empty());
    let new_password = body.new_password.as_deref().filter(|p| !p.is_empty());
    let info = state
        .admin
        .update_account(&session, &body.current_password, new_username, new_password)
        .await?;
    Ok(Json(json!(info)))
}

/* ---- provider keys ---- */

pub async fn list_providers(State(state): State<AppState>) -> ApiResult {
    let rows = state.admin.store()?.list_keys().await?;
    let hotlink = Overrides::load(state.admin.store()?).await?;
    let providers: Vec<Value> = PROVIDER_IDS
        .iter()
        .map(|id| {
            let keys: Vec<Value> = rows
                .iter()
                .filter(|k| k.provider == *id)
                .map(|k| key_view(&state, k))
                .collect();
            json!({
                "id": id,
                "name": provider_name(id),
                "use_proxy": state.runtime.uses_proxy(id),
                "hotlink": hotlink_view(&hotlink, id),
                "keys": keys,
            })
        })
        .collect();
    Ok(Json(json!({ "providers": providers })))
}

/// Per media type: the policy in force, the provider's default and whether it was changed.
fn hotlink_view(overrides: &Overrides, provider: &str) -> Value {
    let one = |kind: Kind| {
        json!({
            "policy": overrides.provider_policy(provider, kind).as_str(),
            "default": hotlink::default_policy(provider, kind).as_str(),
            "custom": overrides.configured(provider, kind).and_then(Policy::parse).is_some(),
        })
    };
    json!({ "image": one(Kind::Image), "video": one(Kind::Video) })
}

#[derive(Debug, Deserialize)]
pub struct AddKeyBody {
    label: String,
    key: String,
}

pub async fn add_key(State(state): State<AppState>, Path(provider): Path<String>, Json(body): Json<AddKeyBody>) -> ApiResult {
    require_provider(&provider)?;
    let label = clean_label(&body.label)?;
    let key = clean_api_key(&body.key)?;

    let store = state.admin.store()?;
    if store.count_keys(&provider).await? >= MAX_KEYS_PER_PROVIDER {
        return Err(AppError::BadRequest(format!("每个供应商最多添加 {MAX_KEYS_PER_PROVIDER} 个密钥")));
    }
    let id = store.insert_key(&provider, &label, &key).await?;
    apply_changes(&state, true).await?;
    Ok(Json(json!({ "success": true, "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeyBody {
    label: Option<String>,
    enabled: Option<bool>,
}

pub async fn update_key(
    State(state): State<AppState>,
    Path((provider, id)): Path<(String, i64)>,
    Json(body): Json<UpdateKeyBody>,
) -> ApiResult {
    require_provider(&provider)?;
    if body.label.is_none() && body.enabled.is_none() {
        return Err(AppError::BadRequest("没有需要修改的内容".to_string()));
    }
    let label = body.label.as_deref().map(clean_label).transpose()?;

    let updated = state
        .admin
        .store()?
        .update_key(&provider, id, label.as_deref(), body.enabled)
        .await?;
    if !updated {
        return Err(AppError::NotFound("密钥不存在".to_string()));
    }
    apply_changes(&state, body.enabled.is_some()).await?;
    Ok(Json(json!({ "success": true })))
}

pub async fn delete_key(State(state): State<AppState>, Path((provider, id)): Path<(String, i64)>) -> ApiResult {
    require_provider(&provider)?;
    if !state.admin.store()?.delete_key(&provider, id).await? {
        return Err(AppError::NotFound("密钥不存在".to_string()));
    }
    apply_changes(&state, true).await?;
    Ok(Json(json!({ "success": true })))
}

/// `source` / `copy` set the provider's policy; `default` removes the override.
#[derive(Debug, Default, Deserialize)]
pub struct HotlinkBody {
    image: Option<String>,
    video: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderSettingsBody {
    use_proxy: Option<bool>,
    hotlink: Option<HotlinkBody>,
}

/// `None` clears the override, `Some` sets it.
fn parse_hotlink_choice(raw: &str) -> Result<Option<Policy>, AppError> {
    match raw {
        "default" => Ok(None),
        other => Policy::parse(other).map(Some).ok_or_else(|| {
            AppError::BadRequest("直链策略只能是 source、copy 或 default".to_string())
        }),
    }
}

pub async fn set_provider_settings(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(body): Json<ProviderSettingsBody>,
) -> ApiResult {
    require_provider(&provider)?;
    let hotlink_changes = match &body.hotlink {
        Some(h) => [(Kind::Image, h.image.as_deref()), (Kind::Video, h.video.as_deref())]
            .into_iter()
            .filter_map(|(kind, raw)| raw.map(|r| parse_hotlink_choice(r).map(|choice| (kind, choice))))
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    if body.use_proxy.is_none() && hotlink_changes.is_empty() {
        return Err(AppError::BadRequest("没有需要修改的内容".to_string()));
    }
    let store = state.admin.store()?;
    for (kind, choice) in hotlink_changes {
        let key = hotlink::setting_key(&provider, kind);
        match choice {
            Some(policy) => store.set_setting(&key, policy.as_str()).await?,
            None => store.delete_setting(&key).await?,
        }
    }
    if let Some(use_proxy) = body.use_proxy {
        runtime::save_use_proxy(store, &provider, use_proxy).await?;
        apply_changes(&state, false).await?;
    }
    Ok(Json(json!({ "success": true })))
}

/* ---- proxy pool ---- */

pub async fn list_proxies(State(state): State<AppState>) -> ApiResult {
    let rows = state.admin.store()?.list_proxies().await?;
    let proxies: Vec<Value> = rows.iter().map(|p| proxy_view(&state, p)).collect();
    Ok(Json(json!({
        "strategy": state.runtime.proxy_strategy(),
        "proxies": proxies,
    })))
}

#[derive(Debug, Deserialize)]
pub struct AddProxyBody {
    name: String,
    url: String,
}

pub async fn add_proxy(State(state): State<AppState>, Json(body): Json<AddProxyBody>) -> ApiResult {
    let name = clean_label(&body.name)?;
    let url = clean_proxy_url(&body.url)?;

    let store = state.admin.store()?;
    if store.list_proxies().await?.len() >= MAX_PROXIES {
        return Err(AppError::BadRequest(format!("代理池最多添加 {MAX_PROXIES} 个代理")));
    }
    let id = store.insert_proxy(&name, &url).await?;
    apply_changes(&state, false).await?;
    Ok(Json(json!({ "success": true, "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProxyBody {
    name: Option<String>,
    enabled: Option<bool>,
}

pub async fn update_proxy(State(state): State<AppState>, Path(id): Path<i64>, Json(body): Json<UpdateProxyBody>) -> ApiResult {
    if body.name.is_none() && body.enabled.is_none() {
        return Err(AppError::BadRequest("没有需要修改的内容".to_string()));
    }
    let name = body.name.as_deref().map(clean_label).transpose()?;
    if !state.admin.store()?.update_proxy(id, name.as_deref(), body.enabled).await? {
        return Err(AppError::NotFound("代理不存在".to_string()));
    }
    apply_changes(&state, false).await?;
    Ok(Json(json!({ "success": true })))
}

pub async fn delete_proxy(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult {
    if !state.admin.store()?.delete_proxy(id).await? {
        return Err(AppError::NotFound("代理不存在".to_string()));
    }
    apply_changes(&state, false).await?;
    Ok(Json(json!({ "success": true })))
}

pub async fn test_proxy(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult {
    let proxy = state
        .admin
        .store()?
        .get_proxy(id)
        .await?
        .ok_or_else(|| AppError::NotFound("代理不存在".to_string()))?;
    let results = runtime::probe_proxy(&proxy.url).await;
    Ok(Json(json!({ "results": results })))
}

#[derive(Debug, Deserialize)]
pub struct ProxySettingsBody {
    strategy: ProxyStrategy,
}

pub async fn set_proxy_settings(State(state): State<AppState>, Json(body): Json<ProxySettingsBody>) -> ApiResult {
    runtime::save_proxy_strategy(state.admin.store()?, body.strategy).await?;
    apply_changes(&state, false).await?;
    Ok(Json(json!({ "success": true })))
}

/* ---- search cache ---- */

async fn cache_overview(state: &AppState) -> ApiResult {
    let stats = state.cache.stats().await?;
    let served = stats.hits + stats.misses;
    Ok(Json(json!({
        "settings": state.runtime.cache_settings(),
        "stats": stats,
        "hit_rate": if served > 0 { Some(stats.hits as f64 / served as f64) } else { None },
    })))
}

pub async fn get_cache(State(state): State<AppState>) -> ApiResult {
    cache_overview(&state).await
}

pub async fn set_cache_settings(State(state): State<AppState>, Json(settings): Json<CacheSettings>) -> ApiResult {
    runtime::save_cache_settings(state.admin.store()?, &settings).await?;
    apply_changes(&state, false).await?;
    // A shorter lifetime or a lower limit takes effect right away rather than on the next search.
    state.cache.enforce_limits().await;
    cache_overview(&state).await
}

pub async fn clear_cache(State(state): State<AppState>) -> ApiResult {
    let cleared = state.cache.clear().await?;
    Ok(Json(json!({ "success": true, "cleared": cleared })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_preview_never_reveals_the_middle() {
        assert_eq!(key_preview("abcd1234efgh5678"), "abcd…5678");
        assert_eq!(key_preview("shortkey"), "••••••••");
        assert!(!key_preview("abcd1234efgh5678").contains("1234"));
    }

    #[test]
    fn proxy_credentials_are_stripped_from_views() {
        let (url, has_auth) = mask_proxy_url("socks5://user:secret@10.0.0.5:1080");
        assert_eq!(url, "socks5://10.0.0.5:1080");
        assert!(has_auth);
        assert!(!url.contains("secret"));

        let (url, has_auth) = mask_proxy_url("http://127.0.0.1:7890");
        assert_eq!(url, "http://127.0.0.1:7890");
        assert!(!has_auth);
    }

    #[test]
    fn proxy_urls_are_validated() {
        assert!(clean_proxy_url("http://127.0.0.1:7890").is_ok());
        assert!(clean_proxy_url("socks5h://user:pw@host:1080").is_ok());
        assert!(clean_proxy_url("ftp://host:21").is_err());
        assert!(clean_proxy_url("127.0.0.1:7890").is_err());
        assert!(clean_proxy_url("").is_err());
    }

    #[test]
    fn api_keys_and_labels_are_validated() {
        assert!(clean_api_key("  abcdefgh12345678  ").is_ok());
        assert_eq!(clean_api_key("  abcdefgh12345678  ").unwrap(), "abcdefgh12345678");
        assert!(clean_api_key("short").is_err());
        assert!(clean_api_key("has space inside 1234").is_err());
        assert!(clean_api_key("line\nbreak12345678").is_err());
        assert!(clean_label("   ").is_err());
        assert!(clean_label("主账号").is_ok());
    }
}

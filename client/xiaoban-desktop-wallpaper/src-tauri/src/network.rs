//! Network policy for trusted desktop HTTP clients. Never broadcast proxy configuration to
//! wallpaper, game, or third-party WebViews (which can contain untrusted HTML).
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::{OnceLock, RwLock}, time::Duration};
use tauri::Manager;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NetworkConfig {
    /// system = reqwest OS/env discovery, direct = explicitly disable proxies,
    /// manual = send HTTP(S) through the configured HTTP(S)/SOCKS5 proxy.
    pub mode: String,
    pub proxy_url: String,
}
impl Default for NetworkConfig {
    fn default() -> Self { Self { mode: "system".into(), proxy_url: String::new() } }
}
static ACTIVE: OnceLock<RwLock<NetworkConfig>> = OnceLock::new();
fn active() -> &'static RwLock<NetworkConfig> { ACTIVE.get_or_init(|| RwLock::new(NetworkConfig::default())) }
pub fn current() -> NetworkConfig { active().read().unwrap_or_else(|e| e.into_inner()).clone() }

fn validate(mode: String, proxy_url: String) -> Result<NetworkConfig, String> {
    if !["system", "direct", "manual"].contains(&mode.as_str()) { return Err("网络模式无效".into()); }
    let proxy_url = proxy_url.trim().to_string();
    if mode == "manual" {
        if proxy_url.is_empty() || proxy_url.len() > 2048 || proxy_url.chars().any(char::is_control) {
            return Err("请填写合法的代理地址".into());
        }
        let url = reqwest::Url::parse(&proxy_url).map_err(|_| "代理地址无效，请包含协议和主机".to_string())?;
        if !["http", "https", "socks5", "socks5h"].contains(&url.scheme())
            || url.host_str().is_none() || url.cannot_be_a_base()
            || url.username() != "" || url.password().is_some()
            || url.query().is_some() || url.fragment().is_some() || url.path() != "/"
        { return Err("代理只支持 HTTP、HTTPS、SOCKS5/SOCKS5H，且地址不能包含凭据或路径".into()); }
        reqwest::Proxy::all(proxy_url.as_str()).map_err(|_| "无法识别代理服务器".to_string())?;
    }
    Ok(NetworkConfig { mode, proxy_url: if mode == "manual" { proxy_url } else { String::new() } })
}
fn path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("network.json"))
}
pub fn initialize(app: &tauri::AppHandle) -> Result<(), String> {
    let config = match fs::read(path(app)?) {
        Ok(data) => serde_json::from_slice::<NetworkConfig>(&data).map_err(|e| e.to_string())
            .and_then(|parsed| validate(parsed.mode, parsed.proxy_url)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(NetworkConfig::default()),
        Err(error) => Err(format!("无法读取网络配置：{error}")),
    };
    // A damaged optional network preference must never prevent the desktop from opening.
    *active().write().unwrap_or_else(|e| e.into_inner()) = config.unwrap_or_else(|error| {
        eprintln!("[network] invalid saved settings, using system proxy: {error}");
        NetworkConfig::default()
    });
    Ok(())
}
pub fn async_builder(builder: reqwest::ClientBuilder) -> Result<reqwest::ClientBuilder, String> {
    let config = current();
    Ok(match config.mode.as_str() {
        "direct" => builder.no_proxy(),
        "manual" => builder.no_proxy().proxy(reqwest::Proxy::all(config.proxy_url.as_str()).map_err(|e| e.to_string())?),
        _ => builder,
    })
}
pub fn blocking_builder(builder: reqwest::blocking::ClientBuilder) -> Result<reqwest::blocking::ClientBuilder, String> {
    let config = current();
    Ok(match config.mode.as_str() {
        "direct" => builder.no_proxy(),
        "manual" => builder.no_proxy().proxy(reqwest::Proxy::all(config.proxy_url.as_str()).map_err(|e| e.to_string())?),
        _ => builder,
    })
}
#[tauri::command]
pub fn get_network_config(window: tauri::WebviewWindow) -> Result<NetworkConfig, String> {
    if window.label() != "control" { return Err("只能在控制台查看网络设置".into()); }
    Ok(current())
}
#[tauri::command]
pub fn save_network_config(app: tauri::AppHandle, window: tauri::WebviewWindow, mode: String, proxy_url: String) -> Result<NetworkConfig, String> {
    if window.label() != "control" { return Err("只能在控制台修改网络设置".into()); }
    let config = validate(mode, proxy_url)?;
    let file = path(&app)?;
    if let Some(dir) = file.parent() { fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(&temporary, &file).map_err(|e| e.to_string())?;
    *active().write().unwrap_or_else(|e| e.into_inner()) = config.clone();
    Ok(config)
}
#[tauri::command]
pub async fn test_network_connection(window: tauri::WebviewWindow, url: String) -> Result<String, String> {
    if window.label() != "control" { return Err("只能在控制台测试网络".into()); }
    let target = reqwest::Url::parse(&url).map_err(|_| "请输入完整的 HTTPS 网址".to_string())?;
    if target.scheme() != "https" || target.username() != "" || target.password().is_some() {
        return Err("只支持测试不含凭据的 HTTPS 网址".into());
    }
    let client = async_builder(reqwest::Client::builder().connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(12)).redirect(reqwest::redirect::Policy::none()))?
        .build().map_err(|e| e.to_string())?;
    let started = std::time::Instant::now();
    // Any HTTP status confirms a working transport, even 401 on a model endpoint.
    let result = client.get(target).send().await.map_err(|e| format!("连接失败：{e}"))?;
    Ok(format!("已连接 · HTTP {} · {} ms", result.status().as_u16(), started.elapsed().as_millis()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn validates_proxy_modes_and_rejects_credentials() {
        assert_eq!(validate("system".into(), "ignored".into()).unwrap(), NetworkConfig::default());
        assert_eq!(validate("direct".into(), "ignored".into()).unwrap().mode, "direct");
        assert!(validate("manual".into(), "http://127.0.0.1:7890".into()).is_ok());
        assert!(validate("manual".into(), "socks5h://127.0.0.1:1080".into()).is_ok());
        for url in ["", "ftp://host", "http://u:p@host:1234", "http://host/a", "http://host/?x=1"] {
            assert!(validate("manual".into(), url.into()).is_err(), "{url}");
        }
    }
}

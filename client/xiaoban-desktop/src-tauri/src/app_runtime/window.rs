//! V2 App Runtime Window management.
//! Opens and controls dedicated application windows (`app-runtime-<id>`).
//! Completely decoupled from desktop wallpaper and companion pet layers.

use std::sync::Mutex;
#[cfg(target_os = "macos")]
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
mod macos_navigation;

const BRIDGE_JS: &str = include_str!("../../resources/runtime-v2/bridge.js");

pub fn window_label(app_id: &str) -> String {
    format!("app-runtime-{app_id}")
}

pub fn app_id_from_label(label: &str) -> Option<&str> {
    label.strip_prefix("app-runtime-")
}

pub struct AppRuntimeState {
    pub lock: Mutex<()>,
    #[cfg(debug_assertions)]
    pub smoke_ready: Mutex<std::collections::HashSet<String>>,
}

impl Default for AppRuntimeState {
    fn default() -> Self {
        Self {
            lock: Mutex::new(()),
            #[cfg(debug_assertions)]
            smoke_ready: Mutex::new(std::collections::HashSet::new()),
        }
    }
}

pub fn default_dimensions(orientation: Option<&str>) -> (f64, f64) {
    match orientation {
        Some("portrait") => (460.0, 820.0),
        Some("landscape") => (1024.0, 640.0),
        _ => (1024.0, 720.0),
    }
}

pub fn network_allowed() -> bool {
    #[cfg(windows)]
    { true }
    #[cfg(target_os = "macos")]
    {
        static SUPPORTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *SUPPORTED.get_or_init(|| {
            std::process::Command::new("/usr/bin/sw_vers").arg("-productVersion").output()
                .ok().filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .and_then(|version| version.split('.').next()?.parse::<u32>().ok())
                .is_some_and(|major| major >= 14)
        })
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    { false }
}

pub async fn open_app(
    app: &AppHandle,
    app_id: &str,
    title: &str,
    orientation: Option<&str>,
    entry: Option<&str>,
) -> Result<WebviewWindow, String> {
    let (window, pending_entry) = show_app_window(app, app_id, title, orientation, entry)?;
    if let Some(entry_url) = pending_entry {
        let result = async {
            let directory = crate::builtin_games::ensure_async(app, app_id).await?;
            let entry_url = if app_id == crate::games::FEATURED_WHALE_GIRL {
                let bytes = std::fs::read(directory.join("manifest.json")).map_err(|e| e.to_string())?;
                let manifest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                let [entry, _, _] = super::pack::check_manifest(&manifest)?;
                app_url(app_id, &entry)?
            } else { entry_url };
            window.navigate(entry_url).map_err(|e| e.to_string())
        }.await;
        if let Err(error) = result {
            if app.get_webview_window(window.label()).is_some() {
                let _ = window.navigate(app_url(app_id, super::protocol::LAUNCH_FAILED)?);
                return Err(error);
            }
        }
    }
    Ok(window)
}

fn app_url(app_id: &str, entry: &str) -> Result<url::Url, String> {
    url::Url::parse(&format!("{}{entry}", super::protocol::base_url(app_id)))
        .map_err(|e| format!("URL 解析错误: {e}"))
}

fn show_app_window(
    app: &AppHandle,
    app_id: &str,
    title: &str,
    orientation: Option<&str>,
    entry: Option<&str>,
) -> Result<(WebviewWindow, Option<url::Url>), String> {
    if !super::storage::valid_app_id(app_id) {
        return Err("非法应用编号".into());
    }

    let state = app.state::<AppRuntimeState>();
    let _guard = state.lock.lock().map_err(|e| e.to_string())?;

    let label = window_label(app_id);

    // If already open, focus it and bring to front
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok((window, None));
    }

    let builtin = crate::game_packs::is_builtin(app_id);
    let installed;
    let (title, orientation, entry) = if builtin {
        (title, orientation, entry)
    } else {
        installed = crate::game_packs::find_user_game(&crate::storage::root(app)?, app_id)?;
        if installed["runtimeProtocol"] != 2 { return Err("这件作品需要在旧版游戏窗口中打开".into()); }
        (installed["name"].as_str().unwrap_or(title), installed["orientation"].as_str(), installed["entry"].as_str())
    };
    let (width, height) = default_dimensions(orientation);
    let allow_network = network_allowed();
    let entry_url = app_url(app_id, entry.unwrap_or("index.html"))?;
    let first_url = if builtin { app_url(app_id, super::protocol::LAUNCH_LOADING)? } else { entry_url.clone() };

    let webview_url = if cfg!(windows) {
        WebviewUrl::External(first_url.clone())
    } else if cfg!(target_os = "macos") && allow_network {
        // Start the package only after WebKit's frame-aware navigation policy is installed.
        WebviewUrl::External(url::Url::parse("about:blank").expect("valid blank URL"))
    } else {
        WebviewUrl::CustomProtocol(first_url.clone())
    };

    let display_title = if title.trim().is_empty() {
        format!("小伴 · 应用 ({app_id})")
    } else {
        format!("小伴 · {title}")
    };

    let own_path_for_navigation = format!("/{app_id}/");
    let proxy = if allow_network {
        Some(url::Url::parse(&format!("http://127.0.0.1:{}", super::network::port()?))
            .map_err(|e| format!("应用网络代理地址无效: {e}"))?)
    } else { None };

    let builder = WebviewWindowBuilder::new(app, &label, webview_url)
        .title(&display_title)
        .inner_size(width, height)
        .min_inner_size(320.0, 240.0)
        .resizable(true)
        .decorations(true)
        .shadow(true)
        .center()
        .visible(true)
        .on_navigation(move |url| {
            let own_origin = if cfg!(windows) {
                url.scheme() == "http" && url.host_str() == Some("app-pack.localhost")
            } else {
                url.scheme() == "app-pack" && url.host_str() == Some("localhost")
            };
            own_origin && url.path().starts_with(&own_path_for_navigation)
        })
        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
        .initialization_script_for_all_frames(BRIDGE_JS);

    #[cfg(windows)]
    let builder = builder
        .proxy_url(proxy.expect("Windows application network proxy is enabled"))
        .data_directory(crate::storage::root(app)?.join("apps").join("webview").join(app_id))
        .additional_browser_args("--proxy-bypass-list=<-loopback>");

    #[cfg(target_os = "macos")]
    let builder = {
        if let Some(proxy) = proxy {
            let digest = Sha256::digest(format!("xiaoban-app-runtime:{app_id}").as_bytes());
            let mut identifier = [0u8; 16];
            identifier.copy_from_slice(&digest[..16]);
            builder.proxy_url(proxy).data_store_identifier(identifier)
        } else {
            builder.incognito(true)
        }
    };

    let window = builder.build().map_err(|e| format!("创建应用窗口失败: {e}"))?;
    #[cfg(target_os = "macos")]
    if allow_network {
        if let Err(error) = macos_navigation::open(&window, first_url) {
            let _ = window.close();
            return Err(error);
        }
    }
    let _ = window.set_focus();

    Ok((window, builtin.then_some(entry_url)))
}

pub fn close_app(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let label = window_label(app_id);
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| format!("关闭应用窗口失败: {e}"))?;
    }
    Ok(())
}

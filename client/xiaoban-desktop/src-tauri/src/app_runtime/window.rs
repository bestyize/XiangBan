//! V2 App Runtime Window management.
//! Opens and controls dedicated application windows (`app-runtime-<id>`).
//! Completely decoupled from desktop wallpaper and companion pet layers.

use std::sync::Mutex;
#[cfg(target_os = "macos")]
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const BRIDGE_JS: &str = include_str!("../../resources/runtime-v2/bridge.js");

pub fn window_label(app_id: &str) -> String {
    format!("app-runtime-{app_id}")
}

pub fn app_id_from_label(label: &str) -> Option<&str> {
    label.strip_prefix("app-runtime-")
}

pub struct AppRuntimeState {
    pub lock: Mutex<()>,
}

impl Default for AppRuntimeState {
    fn default() -> Self {
        Self {
            lock: Mutex::new(()),
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
        return Ok(window);
    }

    let (width, height) = default_dimensions(orientation);
    let entry_file = entry.unwrap_or("index.html");

    let webview_url = if cfg!(windows) {
        WebviewUrl::External(
            url::Url::parse(&format!("http://app-pack.localhost/{app_id}/{entry_file}"))
                .map_err(|e| format!("URL 解析错误: {e}"))?,
        )
    } else {
        WebviewUrl::CustomProtocol(
            url::Url::parse(&format!("app-pack://localhost/{app_id}/{entry_file}"))
                .map_err(|e| format!("URL 解析错误: {e}"))?,
        )
    };

    let display_title = if title.trim().is_empty() {
        format!("小伴 · 应用 ({app_id})")
    } else {
        format!("小伴 · {title}")
    };

    let own_path_for_navigation = format!("/{app_id}/");
    let proxy = if network_allowed() {
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
        .initialization_script(BRIDGE_JS.to_string());

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
    let _ = window.set_focus();

    Ok(window)
}

pub fn close_app(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let label = window_label(app_id);
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| format!("关闭应用窗口失败: {e}"))?;
    }
    Ok(())
}

//! Open public playable works inside Xiaoban without importing a pack into the user's library.
mod link;

use crate::{app_runtime, community, games};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;

pub use link::{Work, PREFIX};
pub const WINDOW_PREFIX: &str = "community-work-";

// Local asset and preview protocols are available to native UI windows only.
pub fn deny_local_resource(label: &str) -> Option<tauri::http::Response<Vec<u8>>> {
    label.starts_with(WINDOW_PREFIX).then(|| tauri::http::Response::builder()
        .status(403).body(Vec::new()).expect("valid empty response"))
}

pub fn launch(app: &AppHandle, work: Work) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let local = match &work {
            Work::Builtin(id) if games::IDS.contains(&id.as_str()) => Some(id.as_str()),
            Work::Idea(id) if id == "aquarium" => Some(games::FEATURED_AQUARIUM),
            Work::Idea(id) if id == "whale-girl" => Some(games::FEATURED_WHALE_GIRL),
            _ => None,
        };
        if let Some(id) = local {
            if let Ok(catalog) = games::builtin_catalog() {
                if let Some(manifest) = catalog.iter().find(|item| item["id"].as_str() == Some(id)) {
                    if app_runtime::window::open_app(&app, id, manifest["name"].as_str().unwrap_or(id),
                        manifest["orientation"].as_str(), manifest["entry"].as_str()).await.is_ok() { return; }
                }
            }
        }
        if let Err(error) = open_web_player(&app, &work) {
            app.dialog().message(format!("无法打开作品：{error}")).title("小伴桌面").show(|_| {});
        }
    });
}

fn open_web_player(app: &AppHandle, work: &Work) -> Result<(), String> {
    // A single launch may be delivered by both the OS deep-link event and the single-instance plugin.
    let state = app.state::<community::CommunityState>();
    let _guard = state.work_windows.lock().map_err(|e| e.to_string())?;
    let suffix = match work {
        Work::Builtin(id) => format!("builtin-{id}"),
        Work::Idea(id) => format!("idea-{id}"),
        Work::Pack(id, version) => format!("pack-{id}-{version}"),
    };
    let label = format!("{WINDOW_PREFIX}{suffix}");
    if let Some(window) = app.get_webview_window(&label) {
        window.unminimize().and_then(|_| window.show()).and_then(|_| window.set_focus()).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let origin = url::Url::parse(community::DEFAULT_ORIGIN).map_err(|e| e.to_string())?;
    let url = origin.join(&work.web_path()).map_err(|e| e.to_string())?;
    WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .title("小伴桌面 · 作品").inner_size(1100.0, 760.0).min_inner_size(340.0, 320.0).center()
        .initialization_script("if(window===window.top && location.origin==='https://chat.xiangban.xyz') Object.defineProperty(window,'__xiaobanWorkHost',{value:true});")
        .on_navigation(|url| match url.scheme() {
            "https" => url.host_str().is_some_and(|h| h == "xiangban.xyz" || h.ends_with(".xiangban.xyz")),
            "about" => matches!(url.path(), "blank" | "srcdoc"),
            "blob" => url.path().starts_with("https://chat.xiangban.xyz/"),
            _ => false,
        })
        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
        .build().map_err(|e| e.to_string())?;
    Ok(())
}

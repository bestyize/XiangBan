//! 小伴市场 commands of the AI store. Like the rest of the store, network and archive work stays off
//! the UI thread, only the control window may call them, and every install is a reviewed, pinned one.
use super::{store, Dirs};
use agent_skill_discovery::market::{self, Market, Package};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Mutex};

/// The market the app ships with. Like the website address, it is built in and not user-editable.
pub const DEFAULT_MARKET: &str = "https://market.xiangban.xyz";

fn market_origin() -> String {
    // Debug builds may point at a local market while developing; release builds cannot be redirected.
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var("XIAOBAN_MARKET_URL") { return url; }
    DEFAULT_MARKET.into()
}
pub(super) fn market() -> Result<Market, String> { Market::new(&market_origin()) }
fn package(kind: String, namespace: String, slug: String, version: String) -> Package {
    Package { kind, namespace, slug, version }
}
fn backups(app: &tauri::AppHandle) -> Result<PathBuf, String> { Ok(Dirs::of(app)?.chat.join("skill-backups")) }

/// A `xiaoban://market/...` request waiting for the console to show it. The console asks for it when
/// it starts and again whenever it is told a new one arrived, so a link that opens a closed app is not lost.
#[derive(Default)]
pub struct PendingLink(Mutex<Option<Value>>);

pub fn receive_link(app: &tauri::AppHandle, text: &str) {
    use tauri::{Emitter, Manager};
    let Ok(link) = market::parse_link(text) else { return };
    *app.state::<PendingLink>().0.lock().unwrap_or_else(|e| e.into_inner()) = Some(link);
    let _ = crate::present_control(app);
    let _ = app.emit("market-link", ());
    // When the link started the app, the console window may not exist yet; show it once it does.
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let _ = crate::present_control(&handle);
    });
}

#[tauri::command]
pub fn ai_market_take_link(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Option<Value>, String> {
    use tauri::Manager;
    store::control(&window)?;
    Ok(app.state::<PendingLink>().0.lock().unwrap_or_else(|e| e.into_inner()).take())
}
#[tauri::command]
pub fn ai_market_info(window: tauri::WebviewWindow) -> Result<Value, String> {
    store::control(&window)?;
    Ok(json!({"origin": market_origin()}))
}
#[tauri::command]
pub async fn ai_market_browse(
    window: tauri::WebviewWindow, kind: String, keyword: String, page: u32, category: String,
    source: String, sort_by: String, variant: String,
) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || market()?.browse(&kind, &keyword, page, &category, &source, &sort_by, &variant))
        .await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn ai_market_categories(window: tauri::WebviewWindow, kind: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || market()?.categories(&kind)).await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn ai_market_detail(window: tauri::WebviewWindow, kind: String, namespace: String, slug: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || market()?.detail(&kind, &namespace, &slug)).await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn ai_market_inspect(window: tauri::WebviewWindow, kind: String, namespace: String, slug: String, version: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || market()?.inspect(&package(kind, namespace, slug, version)))
        .await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn ai_market_install(app: tauri::AppHandle, window: tauri::WebviewWindow, kind: String, namespace: String, slug: String, version: String, sha256: String) -> Result<(), String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        let (market, dirs, pkg) = (market()?, Dirs::of(&app)?, package(kind, namespace, slug, version));
        // Connectors and newer experts are WorkBuddy plugins; skills and older experts are SKILL.md packages.
        if market.is_plugin(&pkg)? {
            market.install_plugin(&dirs.plugins(), &dirs.plugin_backups(), &pkg, &sha256).map(|_| ())
        } else {
            market.install(&dirs.skills(), &pkg, &sha256)
        }
    }).await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn ai_market_group_inspect(app: tauri::AppHandle, window: tauri::WebviewWindow, kind: String, namespace: String, slug: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || market()?.inspect_group(&Dirs::of(&app)?.skills(), &kind, &namespace, &slug))
        .await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn ai_market_group_install(app: tauri::AppHandle, window: tauri::WebviewWindow, kind: String, namespace: String, slug: String, sha256: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        market()?.install_group(&Dirs::of(&app)?.skills(), &kind, &namespace, &slug, &sha256)
    }).await.map_err(|e| e.to_string())?
}
/// Installed packages that have a newer version in the market.
#[tauri::command]
pub async fn ai_market_updates(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || { let dirs = Dirs::of(&app)?; market()?.check_updates_all(&dirs.skills(), &dirs.plugins()) })
        .await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn ai_market_update(app: tauri::AppHandle, window: tauri::WebviewWindow, kind: String, namespace: String, slug: String, version: String, sha256: String) -> Result<(), String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        let (market, dirs, pkg) = (market()?, Dirs::of(&app)?, package(kind, namespace, slug, version));
        if market.is_plugin(&pkg)? {
            market.install_plugin(&dirs.plugins(), &dirs.plugin_backups(), &pkg, &sha256).map(|_| ())
        } else {
            market.update(&dirs.skills(), &backups(&app)?, &pkg, &sha256)
        }
    }).await.map_err(|e| e.to_string())?
}
/// Fields of the lock file that the installed list shows for a package from the market.
pub fn lock_summary(lock: &Value) -> Value {
    json!({"market": lock["origin"]["service"] == "xiaoban-market", "kind": lock["origin"]["kind"]})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_built_in_market_is_used_and_it_is_a_valid_origin() {
        assert!(market::valid_origin(DEFAULT_MARKET).is_ok());
        assert_eq!(lock_summary(&json!({"origin": {"service": "xiaoban-market", "kind": "expert"}}))["kind"], "expert");
        assert_eq!(lock_summary(&json!({"provider": "skillhub"}))["market"], false);
    }
}

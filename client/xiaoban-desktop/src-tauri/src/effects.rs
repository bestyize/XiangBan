//! The desktop-wide particle layer: one transparent, click-through window laid over the desktop picture, so the
//! effect works with any wallpaper (or the plain system desktop) behind it. It is a separate desktop window
//! and not part of a wallpaper window, so changing the wallpaper never touches it.
use crate::{effect_packs, preferences::Settings, settings, storage, DesktopState};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;

pub const LABEL: &str = "effects";
/// Serialises opening and closing; every pass reads the settings again, so the last pass always wins.
static RECONCILE: Mutex<()> = Mutex::new(());
static PLACEMENT: Mutex<String> = Mutex::new(String::new());
pub fn is_effect(label: &str) -> bool { label == LABEL || label.strip_prefix("effects-").is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())) }
pub fn labels(app: &tauri::AppHandle) -> Vec<String> { app.webview_windows().keys().filter(|s| is_effect(s)).cloned().collect() }
fn targets<'a>(areas: &'a [crate::widget_geometry::MonitorArea], selection: &str) -> Vec<&'a crate::widget_geometry::MonitorArea> {
    if selection == "*" { areas.iter().collect() } else { crate::widget_geometry::resolve(areas, if selection.is_empty() { None } else { Some(selection) }).into_iter().collect() }
}
pub fn monitor_state(app: &tauri::AppHandle) -> Result<Value, String> {
    let areas = crate::widgets::monitors(app)?;
    let selected = settings(app).effects.monitor;
    let active: Vec<_> = targets(&areas, &selected).iter().map(|m| m.id.clone()).collect();
    Ok(json!({"selected": selected, "active": active, "fallback": !selected.is_empty() && selected != "*" && !areas.iter().any(|m| m.id == selected), "monitors": areas}))
}

fn folder(app: &tauri::AppHandle) -> Result<PathBuf, String> { Ok(storage::root(app)?.join("effects")) }

/// What the page needs: the pack's data (it checks it itself), the user's parameters, and the global animation settings.
fn payload(app: &tauri::AppHandle, config: &Settings, pack: &Value) -> Value {
    json!({"effect": pack, "params": config.effects.params_of(&config.effects.selected), "paused": config.paused,
        "fps": config.fps, "test": app.state::<DesktopState>().isolated})
}

/// The chosen pack, unless nothing is chosen, it can no longer be found, or a game owns the screen.
fn wanted(app: &tauri::AppHandle, config: &Settings) -> Option<Value> {
    if config.effects.selected.is_empty() || crate::games::is_active(app) { return None; }
    effect_packs::find(&folder(app).ok()?, &config.effects.selected)
}

pub fn render_payload(app: &tauri::AppHandle) -> Value {
    let config = settings(app);
    // Windows custom wallpaper draws its own primary-screen effect. Never draw it there
    // when the user selected only a secondary screen.
    let primary_selected = crate::widgets::monitors(app).ok().is_some_and(|areas| targets(&areas, &config.effects.monitor).iter().any(|m| m.primary));
    if !primary_selected { return Value::Null; }
    wanted(app, &config).map(|pack| payload(app, &config, &pack)).unwrap_or(Value::Null)
}

/// Brings the overlay in line with the settings without blocking the caller (commands may run on the
/// UI thread, which must stay free to process the window being created or destroyed).
pub fn sync(app: &tauri::AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || reconcile(&app));
}

/// The blocking form of [`sync`].
pub fn reconcile(app: &tauri::AppHandle) {
    let _guard = RECONCILE.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let current = settings(app);
    let Some(pack) = wanted(app, &current) else { if let Err(error) = close(app) { let _ = app.emit("desktop-error", error); } return; };
    let areas = match crate::widgets::monitors(app) { Ok(areas) => areas, Err(e) => { let _ = app.emit("desktop-error", e); return; } };
    let selected = targets(&areas, &current.effects.monitor);
    if selected.is_empty() { let _ = app.emit("desktop-error", "没有找到显示器"); return; }
    // When a Windows custom wallpaper renders the primary effect itself, keep
    // the other screens' overlays; do not place an overlapping primary overlay.
    #[cfg(windows)] let composing = crate::content::composing(app);
    #[cfg(not(windows))] let composing = false;
    let overlay: Vec<_> = selected.iter().copied().filter(|m| !(composing && m.primary)).collect();
    let signature = serde_json::to_string(&overlay).unwrap_or_default();
    let mut placement = PLACEMENT.lock().unwrap_or_else(|e| e.into_inner());
    if *placement != signature || labels(app).len() != overlay.len() {
        if let Err(error) = close(app) { let _ = app.emit("desktop-error", error); return; }
        for (index, area) in overlay.iter().enumerate() {
            let label = if index == 0 { LABEL.to_string() } else { format!("effects-{index}") };
            if let Err(error) = open(app, &current, &pack, &label, area) {
                let _ = close(app); *placement = String::new();
                let _ = app.emit("desktop-error", format!("桌面特效无法显示：{error}")); return;
            }
        }
        *placement = signature;
    } else {
        for label in labels(app) { if let Some(window) = app.get_webview_window(&label) {
            let _ = window.eval(format!("window.dispatchEvent(new CustomEvent('effects-config',{{detail:{}}}));", payload(app, &current, &pack)));
        } }
    }
}

/// Destroys every overlay, including the extra screens, and waits for their labels to disappear.
pub fn close(app: &tauri::AppHandle) -> Result<(), String> {
    for label in labels(app) {
        app.state::<DesktopState>().ready_scenes.lock().unwrap_or_else(|e| e.into_inner()).remove(&label);
        if let Some(window) = app.get_webview_window(&label) { window.destroy().map_err(|e| e.to_string())?; }
        let deadline = Instant::now() + Duration::from_secs(3);
        while app.get_webview_window(&label).is_some() {
            if Instant::now() >= deadline { return Err("桌面特效窗口仍在关闭，请稍后重试".into()); }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(())
}

fn open(app: &tauri::AppHandle, current: &Settings, pack: &Value, label: &str, area: &crate::widget_geometry::MonitorArea) -> Result<(), String> {
    app.state::<DesktopState>().ready_scenes.lock().unwrap_or_else(|e| e.into_inner()).remove(label);
    app.state::<crate::content::ContentState>().reports.lock().unwrap_or_else(|e| e.into_inner()).remove(label);
    #[cfg(windows)]
    let monitor = app.available_monitors().map_err(|e| e.to_string())?.into_iter().find(|m| m.position().x as f64 == area.x && m.position().y as f64 == area.y).ok_or("显示器已断开")?;
    #[cfg(windows)] let parent = crate::windows::desktop_parent()?;
    let builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html?scene=effects".into()))
        .title("小伴 · 桌面特效").decorations(false).shadow(false).resizable(false).skip_taskbar(true)
        .transparent(true).visible(false).focused(false)
        .initialization_script(format!("window.__XIAOBAN_EFFECTS__={};", payload(app, current, pack)))
        .inner_size(area.width / area.scale, area.height / area.scale)
        .position(area.x / area.scale, area.y / area.scale);
    #[cfg(target_os = "macos")]
    let builder = builder.incognito(true).background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    #[cfg(windows)]
    let builder = builder.parent_raw(::windows::Win32::Foundation::HWND(parent as _)).data_directory(storage::webview(&storage::root(app)?));
    let window = builder.build().map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        #[cfg(windows)] crate::windows::attach_wallpaper(&window, &monitor, parent)?;
        window.set_ignore_cursor_events(true).map_err(|e| e.to_string())?;
        #[cfg(target_os = "macos")] {
            crate::macos::configure_scene(&window, true, false)?;
            crate::macos::place_widget(&window, area.x, area.y, area.width, area.height)?;
        }
        Ok(())
    })();
    if result.is_err() { let _ = window.destroy(); }
    result
}

fn control_only(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" { return Err("只有设置面板可以管理桌面特效".into()); }
    Ok(())
}

/// Every effect the control window can offer: the built-in ones and the user's own files.
/// Listing also re-reads the files that are in use, so a pack edited in place shows up at once.
#[tauri::command]
pub fn list_desktop_effects(window: tauri::WebviewWindow) -> Result<Value, String> {
    control_only(&window)?;
    let directory = folder(window.app_handle())?;
    let loaded = effect_packs::all(&directory);
    sync(window.app_handle());
    Ok(json!({"packs": loaded.packs, "errors": loaded.errors, "directory": directory}))
}

/// Opens the folder for the user's own effect packs, creating it (with an example, which is not loaded) if needed.
#[tauri::command]
pub fn open_effects_directory(window: tauri::WebviewWindow) -> Result<(), String> {
    control_only(&window)?;
    let directory = folder(window.app_handle())?;
    ensure_folder(&directory);
    storage::show_in_file_manager(&directory)
}

/// Creates the folder with its example on first use, so there is something to copy from.
fn ensure_folder(directory: &Path) {
    if directory.exists() { return; }
    let _ = std::fs::create_dir_all(directory);
    let _ = std::fs::write(directory.join(effect_packs::EXAMPLE_NAME), effect_packs::EXAMPLE);
}

/// Copies a pack file the user picked into the effects folder, under the name its `id` asks for.
/// The file is checked first, so a broken one is refused with the reason instead of landing in the folder.
/// When a pack with the same id is already there the user is asked before anything is replaced.
#[tauri::command]
pub async fn import_effect_pack(window: tauri::WebviewWindow) -> Result<Value, String> {
    control_only(&window)?;
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let chosen = app.dialog().file().set_parent(&window).set_title("选择特效包（JSON 文件）")
            .add_filter("特效包", &["json"]).blocking_pick_file();
        let Some(chosen) = chosen else { return Ok(json!({ "status": "cancelled" })); };
        let source = chosen.into_path().map_err(|e| e.to_string())?;
        let text = std::fs::read_to_string(&source).map_err(|_| "无法读取这个文件（需要 UTF-8 文本的 JSON）".to_string())?;
        if text.len() as u64 > 64 * 1024 { return Err("特效包超过 64 KB".into()); }
        let pack: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("JSON 格式错误：{e}"))?;
        effect_packs::validate(&pack)?;
        let directory = folder(&app)?;
        ensure_folder(&directory);
        let id = pack["id"].as_str().unwrap_or_default().to_string();
        // Two packs sharing an id would mean the second is silently ignored, so ask before replacing one.
        if let Some(taken) = effect_packs::file_with_id(&directory, &id) {
            let confirmed = app.dialog()
                .message(format!("特效文件夹里已经有一个使用 id「{id}」的特效包（{taken}）。\n导入这个会替换掉它。"))
                .title("替换已有的特效包")
                .kind(tauri_plugin_dialog::MessageDialogKind::Warning)
                .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom("替换".into(), "取消".into()))
                .parent(&window).blocking_show();
            if !confirmed { return Ok(json!({ "status": "declined", "id": id, "existing": taken })); }
            if taken != format!("{id}.json") {
                std::fs::remove_file(directory.join(&taken)).map_err(|e| format!("无法替换已有的特效包：{e}"))?;
            }
        }
        let saved = effect_packs::save(&directory, &pack, true)?;
        sync(&app);
        Ok(json!({
            "status": "imported", "id": id, "name": pack["name"].as_str().unwrap_or_default(),
            "file": saved.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        }))
    }).await.map_err(|e| e.to_string())?
}

/// Whether `id` names an effect that can be used now (checked before a choice is saved).
pub fn exists(app: &tauri::AppHandle, id: &str) -> bool {
    folder(app).ok().and_then(|directory| effect_packs::find(&directory, id)).is_some()
}

/// Saves a pack into the effects folder and tells the overlay to pick it up. Used by the assistant's
/// `effect_create` tool; the settings page goes through `import_effect_pack` instead.
pub fn save_pack(app: &tauri::AppHandle, pack: &Value, overwrite: bool) -> Result<String, String> {
    let directory = folder(app)?;
    ensure_folder(&directory);
    let path = effect_packs::save(&directory, pack, overwrite)?;
    sync(app);
    Ok(path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default())
}

/// Deletes a custom pack from the effects folder. If in use, the desktop resets to no effect.
pub fn delete_pack(app: &tauri::AppHandle, id: &str) -> Result<String, String> {
    let directory = folder(app)?;
    let file = effect_packs::delete(&directory, id)?;
    let config = crate::settings(app);
    if config.effects.selected == id {
        let mut choice = serde_json::to_value(&config.effects).map_err(|e| e.to_string())?;
        choice["selected"] = json!("");
        let _ = crate::set_desktop_effects(app.clone(), choice);
    } else {
        sync(app);
    }
    Ok(file)
}

#[tauri::command]
pub fn delete_desktop_effect(window: tauri::WebviewWindow, id: String) -> Result<Value, String> {
    control_only(&window)?;
    let app = window.app_handle();
    let file = delete_pack(app, &id)?;
    Ok(json!({ "ok": true, "id": id, "file": file }))
}



#[cfg(test)]
mod monitor_tests {
    use super::*;
    fn screen(id: &str, primary: bool) -> crate::widget_geometry::MonitorArea {
        crate::widget_geometry::MonitorArea { id: id.into(), name: id.into(), x: 0., y: 0., width: 1920., height: 1080., scale: 1., primary }
    }
    #[test]
    fn selected_all_and_disconnected_displays_resolve() {
        let screens = vec![screen("main", true), screen("other", false)];
        assert_eq!(targets(&screens, "")[0].id, "main");
        assert_eq!(targets(&screens, "other")[0].id, "other");
        assert_eq!(targets(&screens, "*").len(), 2);
        assert_eq!(targets(&screens[..1], "other")[0].id, "main");
        assert_eq!(targets(&screens, "other")[0].id, "other");
    }
}

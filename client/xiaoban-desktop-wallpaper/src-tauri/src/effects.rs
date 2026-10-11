//! The desktop-wide particle layer: one transparent, click-through window laid over the desktop picture, so the
//! effect works with any wallpaper (or the plain system desktop) behind it. It is a separate desktop window
//! and not part of a wallpaper window, so changing the wallpaper never touches it.
use crate::{effect_packs, preferences::Settings, settings, storage, DesktopState};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "effects";
/// Serialises opening and closing; every pass reads the settings again, so the last pass always wins.
static RECONCILE: Mutex<()> = Mutex::new(());

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
    // HWND-backed transparent WebViews cannot alpha-compose over sibling WebViews
    // on Windows. A custom wallpaper renders the same effect in its own page.
    #[cfg(windows)]
    if crate::content::composing(app) && wanted(app, &current).is_some() {
        if let Some(window) = app.get_webview_window(LABEL) { let _ = window.hide(); }
        return;
    }
    match (app.get_webview_window(LABEL), wanted(app, &current)) {
        (Some(window), Some(pack)) => {
            let _ = window.eval(format!("window.dispatchEvent(new CustomEvent('effects-config',{{detail:{}}}));", payload(app, &current, &pack)));
            let _ = window.show();
        }
        (Some(_), None) => { if let Err(error) = close(app) { let _ = app.emit("desktop-error", error); } }
        (None, Some(pack)) => {
            if let Err(error) = open(app, &current, &pack) { let _ = app.emit("desktop-error", format!("桌面特效无法显示：{error}")); }
        }
        (None, None) => {}
    }
}

/// Destroys the overlay and waits until Tauri has forgotten the label, so it can be opened again at once.
pub fn close(app: &tauri::AppHandle) -> Result<(), String> {
    app.state::<DesktopState>().ready_scenes.lock().unwrap().remove(LABEL);
    if let Some(window) = app.get_webview_window(LABEL) {
        window.destroy().map_err(|e| e.to_string())?;
        let deadline = Instant::now() + Duration::from_secs(3);
        while app.get_webview_window(LABEL).is_some() {
            if Instant::now() >= deadline { return Err("桌面特效窗口仍在关闭，请稍后重试".into()); }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(())
}

fn open(app: &tauri::AppHandle, current: &Settings, pack: &Value) -> Result<(), String> {
    app.state::<DesktopState>().ready_scenes.lock().unwrap().remove(LABEL);
    app.state::<crate::content::ContentState>().reports.lock().unwrap().remove(LABEL);
    let monitor = app.primary_monitor().map_err(|e| e.to_string())?.ok_or("没有主显示器")?;
    let scale = monitor.scale_factor();
    #[cfg(windows)]
    let parent = crate::windows::desktop_parent()?;
    let builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?scene=effects".into()))
        .title("小伴 · 桌面特效")
        .decorations(false).shadow(false).resizable(false).skip_taskbar(true)
        .transparent(true).visible(false).focused(false)
        .initialization_script(format!("window.__XIAOBAN_EFFECTS__={};", payload(app, current, pack)))
        .inner_size(monitor.size().width as f64 / scale, monitor.size().height as f64 / scale)
        .position(monitor.position().x as f64 / scale, monitor.position().y as f64 / scale);
    // macOS puts it at a desktop window level once the page reports ready (see `scene_ready`).
    #[cfg(target_os = "macos")]
    let builder = builder.incognito(true).background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    // Host it below Explorer's icons, beside the wallpaper and desktop companion.
    #[cfg(windows)]
    let builder = builder.parent_raw(::windows::Win32::Foundation::HWND(parent as _))
        .data_directory(storage::webview(&storage::root(app)?));
    let window = builder.build().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    if let Err(error) = crate::windows::attach_wallpaper(&window, &monitor, parent) {
        let _ = window.destroy();
        return Err(error);
    }
    if let Err(error) = window.set_ignore_cursor_events(true) {
        let _ = window.destroy();
        return Err(error.to_string());
    }
    #[cfg(target_os = "macos")]
    if let Err(error) = crate::macos::configure_scene(&window, true, true) {
        let _ = window.destroy();
        return Err(error);
    }
    Ok(())
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
    if !directory.exists() {
        std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let _ = std::fs::write(directory.join(effect_packs::EXAMPLE_NAME), effect_packs::EXAMPLE);
    }
    storage::show_in_file_manager(&directory)
}

/// Whether `id` names an effect that can be used now (checked before a choice is saved).
pub fn exists(app: &tauri::AppHandle, id: &str) -> bool {
    folder(app).ok().and_then(|directory| effect_packs::find(&directory, id)).is_some()
}

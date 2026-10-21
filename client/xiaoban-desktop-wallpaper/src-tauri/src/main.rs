#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(any(target_os = "macos", test))]
mod coordinates;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(any(windows, test))]
mod runtime;
mod preferences;
mod startup;
mod library;
mod website;
mod content;
mod account;
mod ai_config;
mod ai_reasoning;
mod tts;
mod asr;
mod ai_catalogue;
mod ai_gateway;
mod chat;
mod storage;
mod local_media;
mod resource_packs;
mod games;
mod game_packs;
mod rituals;
mod widgets;
mod pets;
mod pet_packs;
mod custom_widgets;
mod widget_schedule;
mod widget_market;
mod widget_geometry;
mod desk_game;
mod desk_windows;
mod reminders;
mod presence;
mod proactivity;
mod proactivity_context;
mod care_system;
mod care_weather;
mod effect_choice;
mod effect_packs;
mod effects;
mod effects_smoke;
mod float_assistant;
mod assistant_capture;
mod computer_permissions;
mod capture_editor;
mod assistant_hit_test;
mod window_control;

use preferences::{Preferences, Settings};
use serde::Serialize;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Mutex, atomic::{AtomicBool, Ordering}},
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, Theme, WebviewUrl, WebviewWindowBuilder};

#[derive(Default)]
struct DesktopState {
    settings: Mutex<Settings>,
    ready_scenes: Mutex<HashSet<String>>,
    preferences_path: Mutex<Option<PathBuf>>,
    data_root: Mutex<Option<PathBuf>>,
    preferences_io: Mutex<()>,
    update_lock: Mutex<()>,
    autostart_initialized: AtomicBool,
    startup_name: String,
    startup_allowed: bool,
    startup_error: Mutex<Option<String>>,
    startup_menu: Mutex<Option<tauri::menu::CheckMenuItem<tauri::Wry>>>,
    isolated: bool,
    ui_report: Mutex<Option<serde_json::Value>>,
    #[cfg(target_os = "macos")]
    reopen_count: std::sync::atomic::AtomicU32,
}

fn settings(app: &tauri::AppHandle) -> Settings {
    app.state::<DesktopState>().settings.lock().unwrap().clone()
}

fn ui_smoke_enabled(app: &tauri::AppHandle) -> bool {
    app.state::<DesktopState>().isolated && std::env::var("XIAOBAN_UI_SMOKE").as_deref() == Ok("1")
}

#[tauri::command]
fn ui_smoke_report(app: tauri::AppHandle, window: tauri::WebviewWindow, report: serde_json::Value) -> Result<(), String> {
    if window.label() != "control" || !ui_smoke_enabled(&app) { return Err("UI smoke is only available to the isolated control window".into()); }
    if serde_json::to_vec(&report).map_err(|error| error.to_string())?.len() > 16 * 1024 { return Err("UI smoke report exceeds 16 KiB".into()); }
    *app.state::<DesktopState>().ui_report.lock().unwrap() = Some(report);
    Ok(())
}

async fn wait_ui_smoke(app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(report) = app.state::<DesktopState>().ui_report.lock().unwrap().clone() {
            if report["ok"] == true { return Ok(report); }
            return Err(report["error"].as_str().unwrap_or("UI smoke reported failure without details").to_string());
        }
        if Instant::now() >= deadline { return Err("Vue control UI did not report within 20 seconds; check CSP, IPC, or frontend initialization".into()); }
        tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(100))).await.map_err(|error| error.to_string())?;
    }
}

fn publish(app: &tauri::AppHandle) {
    let settings = settings(app);
    let item = app.state::<DesktopState>().startup_menu.lock().unwrap().clone();
    if let Some(item) = item { let _ = item.set_checked(settings.autostart); }
    if let Some(menu)=app.get_webview_window("widgetmenu"){let _=menu.eval(format!("window.dispatchEvent(new CustomEvent('widget-menu-appearance',{{detail:{}}}));",serde_json::json!(settings.appearance)));}
    let _ = app.emit("settings-changed", settings);
    content::sync(app);
    effects::sync(app);
    pets::sync_preferences(app);
}

fn persist(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    let state = app.state::<DesktopState>();
    let _write = state.preferences_io.lock().unwrap();
    let path = state.preferences_path.lock().unwrap().clone().ok_or("无法找到设置目录")?;
    preferences::save(&path, &Preferences { settings: settings.clone(),
        autostart_initialized: state.autostart_initialized.load(Ordering::Relaxed) })
        .map_err(|error| format!("无法保存设置：{error}"))
}

fn initialize_preferences(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<DesktopState>();
    let directory = storage::root(app)?;
    let legacy = if state.isolated { None } else { Some(app.path().app_config_dir().map_err(|e| e.to_string())?) };
    storage::initialize(&directory, legacy.as_deref()).map_err(|e| format!("用户数据迁移失败，原数据已保留：{e}"))?;
    let path = directory.join("config/preferences.json");
    let prefs = match preferences::load(&path) {
        Ok(prefs) => prefs,
        Err(error) => {
            *state.startup_error.lock().unwrap() = Some(format!("设置读取失败，已使用默认场景：{error}"));
            // An unreadable file must never silently turn startup back on.
            Preferences { autostart_initialized: true, ..Default::default() }
        }
    };
    *state.preferences_path.lock().unwrap() = Some(path);
    *state.settings.lock().unwrap() = prefs.settings;
    state.autostart_initialized.store(prefs.autostart_initialized, Ordering::Relaxed);
    if state.startup_allowed {
        let result = if !prefs.autostart_initialized {
            state.autostart_initialized.store(true, Ordering::Relaxed);
            startup::set(app, settings(app).autostart)
        } else if settings(app).autostart {
            match startup::enabled(app) {
                Ok(true) => startup::refresh(app),
                Ok(false) => startup::set(app, true),
                Err(error) => Err(error),
            }
        } else {
            Ok(())
        };
        if let Err(error) = result {
            *state.startup_error.lock().unwrap() = Some(format!("开机自动启动未能设置：{error}"));
        }
        match startup::enabled(app) {
            Ok(enabled) => state.settings.lock().unwrap().autostart = enabled,
            Err(error) => {
                state.settings.lock().unwrap().autostart = false;
                *state.startup_error.lock().unwrap() = Some(format!("无法读取系统启动设置：{error}"));
            }
        }
        if let Err(error) = persist(app, &settings(app)) {
            *state.startup_error.lock().unwrap() = Some(error);
        }
    } else {
        state.settings.lock().unwrap().autostart = false;
    }
    if std::env::args().any(|argument| argument == "--wallpaper") {
        state.settings.lock().unwrap().mode = "wallpaper".into();
    }
    Ok(())
}

#[derive(Serialize)]
struct StartupStatus { available: bool, error: Option<String> }

#[tauri::command]
fn get_startup_status(app: tauri::AppHandle) -> StartupStatus {
    let state = app.state::<DesktopState>();
    let error = state.startup_error.lock().unwrap().clone();
    StartupStatus { available: state.startup_allowed, error }
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<Settings, String> {
    let state = app.state::<DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    if !state.startup_allowed { return Err("开发预览不登记开机启动，请使用正式应用设置。".into()); }
    let previous = settings(&app);
    startup::set(&app, enabled)?;
    let mut next = previous.clone();
    next.autostart = startup::enabled(&app)?;
    if next.autostart != enabled { return Err("系统未能应用开机启动设置，请在系统设置中检查。".into()); }
    state.autostart_initialized.store(true, Ordering::Relaxed);
    if let Err(error) = persist(&app, &next) {
        let _ = startup::set(&app, previous.autostart);
        return Err(error);
    }
    *state.settings.lock().unwrap() = next;
    *state.startup_error.lock().unwrap() = None;
    publish(&app);
    Ok(settings(&app))
}

fn open_scene(app: &tauri::AppHandle, mode: &str) -> Result<(), String> {
    if pets::active(app) { pets::sync(app); return Ok(()); }
    if !settings(app).companion_enabled { return Ok(()); }
    if let Some(window) = app.get_webview_window("pet") {
        #[cfg(not(windows))]
        window.set_always_on_top(mode == "floating").map_err(|e| e.to_string())?;
        #[cfg(target_os = "macos")]
        macos::configure_scene(&window, mode == "wallpaper", false)?;
        window.show().map_err(|e| e.to_string())?;
        #[cfg(windows)]
        windows::configure_companion(&window, mode == "wallpaper")?;
        window_control::apply_saved(app, &window, if mode == "wallpaper" { "desktop" } else { "topmost" })?;
        return Ok(());
    }
    let monitor = app.primary_monitor().map_err(|e| e.to_string())?.ok_or("没有找到主显示器")?;
    let scale = monitor.scale_factor();
    let s = settings(app);
    let scene_w = s.size * 1.5;
    let scene_h = s.size * 1.22;
    let builder = WebviewWindowBuilder::new(app, "pet", WebviewUrl::App("index.html?scene=floating".into()))
        .title("小伴 · 桌面宠物")
        .decorations(false).shadow(false).resizable(false).skip_taskbar(true)
        .always_on_top(mode == "floating").transparent(true).visible(false).focused(false)
        .inner_size(scene_w, scene_h)
        .position(monitor.position().x as f64 / scale + monitor.size().width as f64 / scale - scene_w - 70.0,
                  monitor.position().y as f64 / scale + monitor.size().height as f64 / scale - scene_h - 100.0);
    #[cfg(target_os = "macos")]
    let builder = builder.incognito(true).background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    #[cfg(windows)]
    let builder = builder.data_directory(storage::webview(&storage::root(app)?));
    let window = builder.build().map_err(|e| e.to_string())?;
    window.set_ignore_cursor_events(true).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    if let Err(error) = windows::configure_companion(&window, mode == "wallpaper") {
        let _ = window.destroy();
        return Err(error);
    }
    #[cfg(target_os = "macos")]
    macos::configure_scene(&window, mode == "wallpaper", false)?;
    window_control::apply_saved(app, &window, if mode == "wallpaper" { "desktop" } else { "topmost" })?;
    Ok(())
}

#[tauri::command]
fn get_platform() -> &'static str {
    std::env::consts::OS
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Settings {
    if app.state::<DesktopState>().startup_allowed {
        if let Ok(enabled) = startup::enabled(&app) { app.state::<DesktopState>().settings.lock().unwrap().autostart = enabled; }
    }
    settings(&app)
}

#[tauri::command]
async fn set_mode(app: tauri::AppHandle, mode: String) -> Result<Settings, String> {
    if games::is_active(&app) { return Err("请先退出游戏空间，再切换桌面模式".into()); }
    let state = app.state::<DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    if !["floating", "wallpaper"].contains(&mode.as_str()) {
        return Err("不支持的显示模式".into());
    }
    let previous = settings(&app);
    let mut next = previous.clone();
    next.mode = mode.clone();
    persist(&app, &next)?;
    if let Err(error) = open_scene(&app, &mode) {
        let _ = persist(&app, &previous);
        return Err(error);
    }
    *state.settings.lock().unwrap() = next;
    publish(&app);
    Ok(settings(&app))
}

#[tauri::command]
fn update_settings(
    app: tauri::AppHandle,
    expression: String,
    size: f64,
    theme: String,
    paused: bool,
    fps: u32,
) -> Result<Settings, String> {
    let state = app.state::<DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    if !preferences::EXPRESSIONS.contains(&expression.as_str()) {
        return Err("不支持的表情".into());
    }
    if !size.is_finite() || !(80.0..=420.0).contains(&size) {
        return Err("尺寸应在 80–420 之间".into());
    }
    if !["night", "dawn"].contains(&theme.as_str()) || ![30, 60].contains(&fps) {
        return Err("不支持的场景设置".into());
    }
    let previous = settings(&app);
    let mut next = previous.clone();
    next.expression = expression;
    next.size = size;
    next.theme = theme;
    next.paused = paused;
    next.fps = fps;
    persist(&app, &next)?;
    if let Some(pet) = app.get_webview_window("pet") {
        if let Err(error) = pet.set_size(tauri::LogicalSize::new(size * 1.5, size * 1.22)) {
            let _ = persist(&app, &previous);
            return Err(error.to_string());
        }
    }
    *state.settings.lock().unwrap() = next;
    publish(&app);
    Ok(settings(&app))
}

/// The desktop-wide particle effect: one effect at a time, applied at once and kept across restarts.
#[tauri::command]
fn set_desktop_effects(app: tauri::AppHandle, effects: serde_json::Value) -> Result<Settings, String> {
    let state = app.state::<DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    let previous = settings(&app);
    let mut next = previous.clone();
    next.effects = effect_choice::EffectsChoice::from_value(&effects);
    if !next.effects.selected.is_empty() && !effects::exists(&app, &next.effects.selected) {
        return Err(format!("没有找到特效“{}”，它可能已被移出特效文件夹", next.effects.selected));
    }
    persist(&app, &next)?;
    *state.settings.lock().unwrap() = next;
    publish(&app);
    Ok(settings(&app))
}

#[tauri::command]
async fn set_companion_enabled(app:tauri::AppHandle,enabled:bool)->Result<Settings,String>{
    if games::is_active(&app){return Err("请先退出游戏空间，再更改小伴显示".into());}
    let state=app.state::<DesktopState>(); let _update=state.update_lock.lock().unwrap();
    let previous=settings(&app); let mut next=previous.clone(); next.companion_enabled=enabled;
    persist(&app,&next)?; *state.settings.lock().unwrap()=next.clone();
    if !enabled {
        if let Some(pet)=app.get_webview_window("pet"){let _=pet.destroy();}
        state.ready_scenes.lock().unwrap().remove("pet");
    } else {
        if let Err(error)=open_scene(&app,&next.mode){*state.settings.lock().unwrap()=previous.clone();let _=persist(&app,&previous);return Err(error);}
    }
    publish(&app);Ok(next)
}

#[tauri::command]
fn request_system_permissions(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    computer_permissions::request(&app)
}

#[tauri::command]
fn set_companion_links(app: tauri::AppHandle, enabled: bool) -> Result<Settings, String> {
    let state = app.state::<DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    let mut next = settings(&app);
    next.companion_links = enabled;
    persist(&app, &next)?;
    *state.settings.lock().unwrap() = next.clone();
    publish(&app);
    Ok(next)
}

fn apply_window_appearance(app: &tauri::AppHandle, mode: &str) {
    if let Some(window) = app.get_webview_window("control") {
        let theme = match mode {
            "dark" => Some(Theme::Dark),
            "light" => Some(Theme::Light),
            _ => None,
        };
        let _ = window.set_theme(theme);
        #[cfg(target_os = "macos")]
        let _ = macos::apply_control_appearance(&window, mode);
    }
}

/// Saves the console's colour scheme and tells every window, so the floating assistant matches it.
#[tauri::command]
fn set_appearance(app: tauri::AppHandle, mode: String) -> Result<Settings, String> {
    if !preferences::APPEARANCES.contains(&mode.as_str()) { return Err(format!("没有这个外观：{mode}")); }
    let state = app.state::<DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    let mut next = settings(&app);
    if next.appearance == mode && next.appearance_kept { return Ok(next); }
    next.appearance = mode;
    next.appearance_kept = true;
    persist(&app, &next)?;
    *state.settings.lock().unwrap() = next.clone();
    publish(&app);
    apply_window_appearance(&app, &next.appearance);
    Ok(next)
}

/// What the host has for the colour scheme, and whether it was ever given one.
#[tauri::command]
fn get_appearance(app: tauri::AppHandle) -> serde_json::Value {
    let s = settings(&app);
    serde_json::json!({ "appearance": s.appearance, "appearanceKept": s.appearance_kept })
}

#[tauri::command]
fn set_companion_skin(app: tauri::AppHandle, skin: String) -> Result<Settings, String> {
    if !preferences::SKINS.contains(&skin.as_str()) { return Err(format!("没有这个外壳：{skin}")); }
    let state = app.state::<DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    let mut next = settings(&app);
    next.companion_skin = skin;
    persist(&app, &next)?;
    *state.settings.lock().unwrap() = next.clone();
    publish(&app);
    Ok(next)
}

#[tauri::command]
fn poke(app: tauri::AppHandle) {
    let _ = app.emit("pet-poke", ());
    for label in ["media-a", "media-b"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.eval("window.dispatchEvent(new Event('content-poke'));".to_string());
        }
    }
}

#[tauri::command]
fn scene_ready(window: tauri::WebviewWindow, report: Option<serde_json::Value>) -> Result<(), String> {
    if content::media_label(window.label()) || window.label() == effects::LABEL {
        if let Some(report) = report {
            if report.to_string().len() > 16384 { return Err("场景报告过大".into()); }
            let failed = report.get("error").is_some();
            window.app_handle().state::<content::ContentState>().reports.lock().unwrap().insert(window.label().into(), report);
            if failed { return Ok(()); }
        }
    }
    window.show().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    if window.label() == "pet" {
        windows::configure_companion(&window, settings(window.app_handle()).mode == "wallpaper")?;
        content::sync(window.app_handle());
    }
    #[cfg(windows)]
    windows::order_desktop_layers(window.app_handle())?;
    #[cfg(target_os = "macos")]
    macos::configure_scene(
        &window,
        (window.label() == "pet" && settings(window.app_handle()).mode == "wallpaper") || ["media-a", "media-b", effects::LABEL].contains(&window.label()),
        ["media-a", "media-b", effects::LABEL].contains(&window.label()),
    )?;
    if window.label() == "pet" || window.label() == float_assistant::LABEL || widgets::is_widget(window.label()) || pets::is_pet(window.label()) {
        let default = if widgets::is_widget(window.label()) || (window.label() == "pet" && settings(window.app_handle()).mode == "wallpaper") { "desktop" } else { "topmost" };
        window_control::apply_saved(window.app_handle(), &window, default)?;
    }
    window
        .app_handle()
        .state::<DesktopState>()
        .ready_scenes
        .lock()
        .unwrap()
        .insert(window.label().into());
    Ok(())
}

#[tauri::command]
fn set_hit_test(window: tauri::WebviewWindow, hit: bool) -> Result<(), String> {
    if window.label() != "pet" {
        return Ok(());
    }
    #[cfg(windows)]
    if window_control::layer(window.app_handle(), "pet", if settings(window.app_handle()).mode == "wallpaper" { "desktop" } else { "topmost" }) == "desktop" { return Ok(()); }
    #[cfg(windows)]
    if windows::left_button_down() {
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    if macos::left_button_down() {
        return Ok(());
    }
    window
        .set_ignore_cursor_events(!hit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn start_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "pet" {
        return Err("只有小伴窗口可以拖动".into());
    }
    #[cfg(windows)]
    if window_control::layer(window.app_handle(), "pet", if settings(window.app_handle()).mode == "wallpaper" { "desktop" } else { "topmost" }) == "desktop" {
        return windows::drag_companion(&window);
    }
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_control(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("control").ok_or("没有找到设置面板")?.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn show_control(app: tauri::AppHandle) -> Result<(), String> {
    present_control(&app)
}

pub(crate) fn present_control(app: &tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    app.show().map_err(|error| error.to_string())?;
    let window = app
        .get_webview_window("control")
        .ok_or("没有找到小伴设置面板")?;
    apply_window_appearance(app, &settings(app).appearance);
    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    get_settings(app.clone());
    publish(app);
    Ok(())
}

#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn get_diagnostics(app: tauri::AppHandle) -> serde_json::Value {
    #[cfg(windows)]
    let mut report = windows::diagnostics(&app);
    #[cfg(target_os = "macos")]
    let mut report = macos::diagnostics(&app);
    #[cfg(not(any(windows, target_os = "macos")))]
    let mut report =
        serde_json::json!({"platform": std::env::consts::OS, "wallpaperSupported": false});
    let state = app.state::<DesktopState>();
    let ready = state.ready_scenes.lock().unwrap();
    for (key, label) in [("floating", "pet"), ("wallpaper", "wallpaper")] {
        if let Some(scene) = report.get_mut(key).filter(|value| value.is_object()) {
            scene["frontendReady"] = serde_json::json!(ready.contains(label));
        }
    }
    report["contentReports"] = serde_json::to_value(app.state::<content::ContentState>().reports.lock().unwrap().clone()).unwrap();
    report["games"] = games::get_game_status(app.clone());
    report
}

fn wait_for_scene(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        let ready = app
            .state::<DesktopState>()
            .ready_scenes
            .lock()
            .unwrap()
            .contains(label);
        if ready
            && app
                .get_webview_window(label)
                .is_some_and(|window| window.is_visible().unwrap_or(false))
        {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!("场景 {label} 没有在 20 秒内完成前端挂载并显示"))
}

fn wait_control(app: &tauri::AppHandle, visible: bool) -> Result<serde_json::Value, String> {
    let window = app.get_webview_window("control").ok_or("没有找到设置面板")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let shown = window.is_visible().map_err(|e| e.to_string())?;
        let minimized = window.is_minimized().map_err(|e| e.to_string())?;
        if shown == visible && (!visible || !minimized) {
            return Ok(serde_json::json!({"visible": shown, "minimized": minimized,
                "focused": window.is_focused().unwrap_or(false)}));
        }
        if Instant::now() >= deadline { return Err("设置面板没有完成隐藏或恢复".into()); }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn smoke_lifecycle(app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
    let background_start = std::env::args().any(|arg| arg == "--autostart");
    let initial_control = wait_control(app, !background_start)?;
    if app.tray_by_id("xiaoban-tray").is_none() { return Err("没有找到托盘图标".into()); }
    // Launch a real second process. It must focus this process rather than create another pet.
    let mut process = std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .arg("--smoke-single-instance").spawn().map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = process.try_wait().map_err(|e| e.to_string())? {
            if !status.success() { return Err("第二个小伴进程未正常退出".into()); }
            break;
        }
        if Instant::now() >= deadline {
            let _ = process.kill(); let _ = process.wait();
            return Err("单实例检查超时".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let second_instance = wait_control(app, true)?;
    app.get_webview_window("control").ok_or("没有找到设置面板")?.close().map_err(|e| e.to_string())?;
    let after_close = wait_control(app, false)?;
    let pet = app.get_webview_window("pet").ok_or("关闭设置时销毁了桌宠")?;
    if !pet.is_visible().map_err(|e| e.to_string())? { return Err("关闭设置时隐藏了桌宠".into()); }
    present_control(app)?;
    let after_restore = wait_control(app, true)?;
    let initially_enabled = startup::enabled(app)?;
    if !initially_enabled { return Err("首次启动没有默认开启自启动".into()); }
    set_autostart(app.clone(), false)?;
    let path = app.state::<DesktopState>().preferences_path.lock().unwrap().clone().ok_or("没有设置路径")?;
    let disabled = preferences::load(&path).map_err(|e| e.to_string())?;
    if disabled.settings.autostart || !disabled.autostart_initialized || startup::enabled(app)? {
        return Err("关闭自启动后没有保存或移除系统启动项".into());
    }
    set_autostart(app.clone(), true)?;
    if !startup::enabled(app)? { return Err("无法重新开启自启动".into()); }
    set_autostart(app.clone(), false)?;
    let previous = settings(app);
    update_settings(app.clone(), "happy".into(), 300.0, "dawn".into(), false, 30)?;
    let restored = preferences::load(&path).map_err(|e| e.to_string())?;
    if restored.settings != settings(app) { return Err("场景设置没有正确持久化".into()); }
    update_settings(app.clone(), previous.expression, previous.size, previous.theme, previous.paused, previous.fps)?;
    Ok(serde_json::json!({"initialControl": initial_control, "secondInstance": second_instance,
        "afterClose": after_close, "afterRestore": after_restore, "trayPresent": true,
        "petPreserved": true, "autostartDefaultEnabled": initially_enabled,
        "autostartToggleAndPersistence": true, "scenePersistence": true,
        "testStartupName": app.state::<DesktopState>().startup_name}))
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::{
        menu::{CheckMenuItem, Menu, MenuItem},
        tray::TrayIconBuilder,
    };
    let control = MenuItem::with_id(app, "control", "打开小伴设置", true, None::<&str>)?;
    let assistant = MenuItem::with_id(app, "assistant", "呼出超级小伴", true, None::<&str>)?;
    let poke = MenuItem::with_id(app, "poke", "戳戳小伴", true, None::<&str>)?;
    let games = MenuItem::with_id(app, "games", "玩一玩", true, None::<&str>)?;
    let stop_desk = MenuItem::with_id(app, "stop-desk-game", "停止桌面游戏", true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(app, "autostart", "开机自动启动",
        app.state::<DesktopState>().startup_allowed, settings(app.handle()).autostart, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "退出小伴", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&control, &assistant, &games, &stop_desk, &poke, &autostart, &exit])?;
    *app.state::<DesktopState>().startup_menu.lock().unwrap() = Some(autostart);
    // The app icon at 64x64 as raw RGBA (icons/tray.rgba), so the tray needs no image decoder.
    const TRAY_ICON: &[u8] = include_bytes!("../icons/tray.rgba");
    let builder = TrayIconBuilder::with_id("xiaoban-tray")
        .icon(tauri::image::Image::new(TRAY_ICON, 64, 64))
        .tooltip("小伴 · 桌面陪伴")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "control" => {
                let _ = present_control(app);
            }
            "assistant" => float_assistant::toggle(app),
            "poke" => {
                let _ = app.emit("pet-poke", ());
            }
            "games" => {
                let handle=app.clone();
                tauri::async_runtime::spawn(async move { if let Err(error)=games::open(&handle).await { let _=handle.emit("desktop-error",error); } });
            }
            "stop-desk-game" => {
                let handle=app.clone();
                tauri::async_runtime::spawn(async move { if let Err(error)=desk_game::close(&handle).await { let _=handle.emit("desktop-error",error); } });
            }
            "autostart" => {
                if let Err(error) = set_autostart(app.clone(), !get_settings(app.clone()).autostart) {
                    let _ = present_control(app);
                    let _ = app.emit("desktop-error", error);
                }
            }
            "exit" => app.exit(0),
            _ => {}
        });
    #[cfg(windows)]
    let builder = builder.show_menu_on_left_click(false).on_tray_icon_event(|tray, event| {
        if matches!(event, tauri::tray::TrayIconEvent::Click {
            button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, ..
        }) { let _ = present_control(tray.app_handle()); }
    });
    builder.build(app)?;
    Ok(())
}

fn main() {
    // This must run before Builder::build: even the settings window needs WebView2.
    #[cfg(windows)]
    if !runtime::windows::before_start() {
        return;
    }
    let isolated = std::env::args().any(|argument| ["--smoke-test", "--smoke-single-instance"].contains(&argument.as_str()));
    let startup_id = std::env::var("XIAOBAN_TEST_STARTUP_ID").ok()
        .filter(|value| !value.is_empty() && value.bytes().all(|c| c.is_ascii_digit()))
        .unwrap_or_else(|| std::process::id().to_string());
    let startup_name = if isolated { format!("XiaoBanStartupSmoke-{startup_id}") }
        else { "XiaoBanDesktop".into() };
    let _test_cleanup = startup::TestEntryGuard(isolated.then(|| startup_name.clone()));
    let data_root = storage::location(isolated).expect("无法找到用户数据目录");
    let state = DesktopState { startup_name: startup_name.clone(), isolated,
        data_root: Mutex::new(Some(data_root.clone())),
        startup_allowed: true, ..Default::default() };
    let mut context = tauri::generate_context!();
    if isolated { context.config_mut().identifier.push_str(".smoke"); }
    else if cfg!(debug_assertions) { context.config_mut().identifier.push_str(".development"); }
    #[cfg(windows)]
    for window in &mut context.config_mut().app.windows {
        window.data_directory = Some(storage::webview(&data_root));
    }
    #[cfg(target_os = "macos")]
    for window in &mut context.config_mut().app.windows { window.incognito = true; }
    let builder = tauri::Builder::default()
        .plugin(tauri::plugin::Builder::<tauri::Wry, ()>::new("app-context-menu")
            .js_init_script_on_all_frames(include_str!("webview_context_menu.js"))
            .build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(float_assistant::plugin())
        .plugin(tauri_plugin_single_instance::init(|app, arguments, _| {
            if !arguments.iter().any(|arg| arg == "--autostart") { let _ = present_control(app); }
        }));
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_plugin_autostart::Builder::new().app_name(startup_name).arg("--autostart").build());
    let command_handler: Box<dyn Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync> = Box::new(tauri::generate_handler![resource_packs::export_resource_pack, resource_packs::import_resource_pack, get_platform, get_settings, set_mode, update_settings, set_desktop_effects, effects::list_desktop_effects, effects::open_effects_directory, effects::import_effect_pack, effects::delete_desktop_effect, set_companion_enabled, set_appearance, get_appearance, set_companion_links, set_companion_skin, poke,
            scene_ready, set_hit_test, start_drag, hide_control, show_control, quit, get_diagnostics, ui_smoke_report,
            get_startup_status, set_autostart, content::get_wallpaper_library, content::import_wallpaper_catalog, content::delete_local_wallpaper,
            desk_game::open_desktop_game, desk_game::close_desktop_game, desk_game::desk_set_regions,
            games::open_game_space, games::close_game_space, games::get_game_catalog, games::load_game_save,
            games::save_game_save, games::get_game_status, games::set_game_desktop_double_click,
            games::game_smoke_report, games::list_games, games::delete_user_game, games::open_user_games_folder, rituals::open_ritual_space, rituals::close_ritual_space, rituals::read_ritual_journal, rituals::write_ritual_journal,
            pets::get_pet_catalog, pets::add_pet, pets::update_pet, pets::remove_pet, pets::save_pet_theme, pets::delete_pet_theme, pets::export_pet_theme, pets::get_pet_instance, pets::move_pet_instance, pets::set_pet_hit_test, pets::manage_pet_instance,
             proactivity::get_proactivity_settings, proactivity::get_proactivity_context, proactivity::configure_proactivity, proactivity::pet_proactive_hover, proactivity::pet_proactive_poke, proactivity::get_companion_greeting, proactivity::respond_companion_greeting,
            widgets::get_desktop_widgets, widgets::add_desktop_widget, widgets::update_desktop_widget, widgets::remove_desktop_widget,
            widgets::set_widgets_editing, widgets::get_desktop_widget, widgets::move_desktop_widget, widgets::show_widget_settings, widgets::resize_desktop_widget, widgets::get_widget_settings, widgets::update_widget_settings, widgets::remove_widget_settings, widgets::close_widget_settings, widgets::size_widget_settings, widgets::search_widget_cities, widgets::get_widget_weather, widgets::get_widget_forecast, widgets::get_widget_air_quality, widgets::get_widget_stock_watchlist, widgets::get_widget_stocks, widgets::save_desktop_widget_content,
            widgets::list_custom_widgets, widgets::save_custom_widget, widgets::remove_custom_widget,
            reminders::get_care_settings, reminders::configure_care, reminders::upsert_care_rule, reminders::delete_care_rule,
            reminders::preview_care_rule, reminders::get_care_delivery, reminders::care_ready, reminders::dismiss_care,
            reminders::snooze_care, reminders::care_synthesize, reminders::search_care_cities, reminders::detect_care_city, reminders::check_care_weather,
            content::subscribe_wallpapers, content::apply_wallpaper, content::get_app_info,
            content::check_updates, content::open_download_url, local_media::import_local_wallpaper, storage::open_data_directory,
            website::set_automatic_updates, website::open_official_site, website::fetch_site_wallpapers, website::fetch_site_categories,
            website::select_site_wallpaper, website::download_site_wallpaper, website::cancel_site_download, website::check_website, website::get_website_status, website::dismiss_website_message,
            account::send_account_code, account::login_account_with_code, account::login_account_with_password,
            account::register_account, account::reset_account_password, account::check_account_username, account::get_signup_methods,
            account::get_account_session, account::logout_account,
            tts::tts_get_settings, tts::tts_save_settings, tts::tts_list_voices, tts::tts_synthesize, tts::tts_cancel,
            asr::asr_get_status, asr::asr_set_enabled, asr::asr_prepare, asr::asr_transcribe, asr::asr_report_microphone,
            ai_gateway::ai_gateway_balance, ai_gateway::ai_gateway_redeem, ai_gateway::ai_gateway_tiers, ai_gateway::use_ai_gateway,
            ai_catalogue::refresh_ai_providers, ai_config::get_ai_config, ai_config::save_ai_config, ai_config::pick_workspace, ai_config::fetch_ai_models,
            ai_config::save_ai_provider, ai_config::switch_ai_provider, ai_config::delete_ai_provider, ai_config::save_ai_preferences, ai_config::set_context_tokens, request_system_permissions,
            chat::chat_overview, chat::chat_load_session, chat::chat_new_session, chat::chat_send, chat::chat_cancel, chat::chat_delete_session, chat::chat_run_history, chat::chat_verify_file_outcome, chat::chat_reflect_run, chat::chat_review_experience,
            chat::chat_context, chat::chat_compact, chat::chat_rename_session, chat::chat_move_session, chat::chat_create_group, chat::chat_rename_group, chat::chat_delete_group,
            chat::chat_answer_approval, chat::chat_answer_question, chat::chat_set_permission_mode,
            chat::store::ai_store_installed, chat::store::ai_store_browse, chat::store::ai_store_inspect, chat::store::ai_store_install, chat::store::ai_store_uninstall,
            chat::store::ai_store_catalog, chat::store::ai_store_experts_installed, chat::store::ai_store_expert_review, chat::store::ai_store_expert_install, chat::store::ai_store_expert_remove, chat::store::ai_store_connector_detail, chat::store::ai_store_connector_status, chat::store::ai_store_connector_connect, chat::store::ai_store_connector_disconnect, chat::store::ai_store_connector_reconnect,
            chat::store::ai_store_connector_auth_status, chat::store::ai_store_connector_auth_cancel,
            float_assistant::assistant_status, float_assistant::assistant_configure, float_assistant::assistant_ready, float_assistant::assistant_hide,
            assistant_hit_test::assistant_set_regions, capture_editor::capture_frame, capture_editor::capture_show, capture_editor::capture_finish,
            float_assistant::assistant_screenshot, float_assistant::assistant_fit, float_assistant::assistant_drag, float_assistant::assistant_unfold,
             assistant_capture::reset_screen_capture_permission, assistant_capture::open_screen_capture_settings,
            float_assistant::assistant_transition_ready, float_assistant::assistant_prefs, float_assistant::assistant_try, float_assistant::assistant_open_chat, float_assistant::assistant_open_settings]);
    let app = builder.manage(state).manage(content::ContentState::default()).manage(website::WebsiteState::default())
        .manage(games::GameState::default()).manage(desk_game::DeskState::default()).manage(rituals::RitualState::default()).manage(widgets::WidgetState::default()).manage(pets::PetState::default()).manage(reminders::ReminderState::default()).manage(proactivity::ProactivityState::default()).manage(account::AccountState::default()).manage(ai_config::AiConfigState::default()).manage(tts::TtsState::default()).manage(asr::AsrState::default()).manage(ai_catalogue::CatalogueState::default()).manage(float_assistant::AssistantState::default()).manage(capture_editor::CaptureState::default()).manage(assistant_hit_test::HitState::default()).manage(window_control::WindowState::default())
        .register_uri_scheme_protocol("wallpaper-asset", |context, request| library::serve(context.app_handle(), request))
        .register_uri_scheme_protocol("game-pack", |context, request| game_packs::serve(context.app_handle(), request))
        .invoke_handler(move |invoke| {
            let label=invoke.message.webview_ref().label(); let command=invoke.message.command();
            if command.starts_with("asr_") && label != "control" && label != float_assistant::LABEL {
                invoke.resolver.reject("只能在聊天和声音设置中使用语音识别");
                return true;
            }
            let allowed=capture_editor::permitted(label,command).or_else(||pets::permitted(label,command)).or_else(||float_assistant::permitted(label,command)).or_else(|| desk_game::permitted(label,command)).or_else(|| widgets::permitted(label,command)).or_else(|| rituals::permitted(label,command)).or_else(|| games::permitted(label,command)).unwrap_or_else(|| {
                if matches!(command, "reset_screen_capture_permission" | "open_screen_capture_settings") { return label=="control"; }
                 if command=="ui_smoke_report" {return label=="control";}
                if command=="set_appearance" || command=="get_appearance" || command=="set_companion_enabled" || command=="set_companion_links" || command=="set_companion_skin" || command=="set_desktop_effects" || command=="list_desktop_effects" || command=="open_effects_directory" || command=="import_effect_pack" {return label=="control";}
                if label=="care" { return ["get_care_delivery","care_ready","dismiss_care","snooze_care","care_synthesize"].contains(&command); }
                if command.contains("care") { return label=="control"; }
                content::permitted(label,command)
            });
            if !allowed {
                invoke.resolver.reject("壁纸内容没有桌面控制或系统操作权限");
                return true;
            }
            command_handler(invoke)
        })
        .setup(|app| {
            initialize_preferences(app.handle()).map_err(std::io::Error::other)?;
            if let Some(session) = account::load_session(app.handle()) {
                *app.state::<account::AccountState>().session.lock().unwrap() = Some(session);
            }
            app.manage(chat::ChatState::new(app.handle()));
            float_assistant::start(app.handle());
            asr::start(app.handle());
            games::start(app.handle());
            setup_tray(app)?;
            website::start(app.handle());
            ai_catalogue::start(app.handle());
            let initial_mode = settings(app.handle()).mode;
            let _media_active = match content::restore(app.handle()) {
                Ok(active) => active,
                Err(error) => {
                    *app.state::<DesktopState>().startup_error.lock().unwrap() = Some(error);
                    app.state::<DesktopState>().settings.lock().unwrap().wallpaper_id = "companion".into();
                    false
                }
            };
            pets::start(app.handle());
            open_scene(app.handle(), &initial_mode).map_err(std::io::Error::other)?;
            apply_window_appearance(app.handle(), &settings(app.handle()).appearance);
            effects::sync(app.handle());
            widgets::start(app.handle());
            proactivity::start(app.handle());
            reminders::start(app.handle());
            if !std::env::args().any(|argument| argument == "--autostart") {
                present_control(app.handle()).map_err(std::io::Error::other)?;
            }
            account::activated(app.handle());
            for (label, win) in app.handle().webview_windows() {
                eprintln!(">>> WebviewWindow label={label}, visible={:?}, size={:?}, pos={:?}", win.is_visible(), win.outer_size(), win.outer_position());
            }
            #[cfg(windows)] windows::start_cursor_bridge(app.handle().clone());
            #[cfg(target_os = "macos")] macos::start_cursor_bridge(app.handle().clone());
            if std::env::args().any(|a| a == "--smoke-test") {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Synchronize on the actual frontend handshake, including
                    // cold WKWebView startup on a CI Mac.
                    let startup_result = wait_for_scene(&handle, "pet");
                    let lifecycle = smoke_lifecycle(&handle);
                    let floating_before = get_diagnostics(handle.clone());
                    let wallpaper_result = set_mode(handle.clone(), "wallpaper".into()).await
                        .and_then(|_| wait_for_scene(&handle, "pet"));
                    let wallpaper = get_diagnostics(handle.clone());
                    #[cfg(target_os = "macos")]
                    let dock_reopen = macos::smoke_dock_reopen(&handle);
                    let media = if std::env::var_os("XIAOBAN_WALLPAPER_TEST_URL").is_some() {
                        Some(content::smoke(&handle).await)
                    } else { None };
                    let website = if std::env::var_os("XIAOBAN_WEBSITE_TEST_URL").is_some() {
                        Some(website::smoke(&handle).await)
                    } else { None };
                    let floating_result = set_mode(handle.clone(), "floating".into()).await
                        .and_then(|_| wait_for_scene(&handle, "pet"));
                    let floating_after = get_diagnostics(handle.clone());
                    let games_report=if std::env::var_os("XIAOBAN_GAMES_SMOKE").is_some(){Some(games::smoke(&handle).await)}else{None};
                    let care_report=if std::env::var_os("XIAOBAN_CARE_SMOKE").is_some(){Some(reminders::smoke(&handle).await)}else{None};
                    let ui_report=if ui_smoke_enabled(&handle){Some(wait_ui_smoke(&handle).await)}else{None};
                    let assistant_report=float_assistant::smoke(&handle).await;
                    let asr_report=asr::smoke(&handle).await;
                    #[allow(unused_mut)]
                    let mut report = serde_json::json!({"floatingBefore": floating_before, "wallpaper": wallpaper,
                        "floatingAfter": floating_after, "wallpaperResult": wallpaper_result.map(|_| "ok"),
                        "floatingResult": floating_result.map(|_| "ok"), "startupResult": startup_result.map(|_| "ok"),
                        "lifecycle": lifecycle,"media":media,"website":website,"games":games_report,"care":care_report,"ui":ui_report,"assistant":assistant_report,"asr":asr_report});
                    #[cfg(target_os = "macos")]
                    { report["dockReopen"] = serde_json::to_value(dock_reopen).unwrap(); }
                    if let Ok(path) = std::env::var("XIAOBAN_SMOKE_REPORT") {
                        let _ = std::fs::write(path, serde_json::to_string_pretty(&report).unwrap());
                    }
                    handle.exit(0);
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(target_os="macos")]
            if widgets::is_widget(window.label())&&matches!(event,tauri::WindowEvent::Focused(true)){
                widgets::restore_window_layer(window.app_handle(),window.label());
            }

            if matches!(window.label(), "control" | "assistant") && matches!(event, tauri::WindowEvent::Focused(true)) {
                account::activated(window.app_handle());
            }
            if window.label() == float_assistant::LABEL && matches!(event, tauri::WindowEvent::Moved(_)) { float_assistant::moved(window.app_handle()); }
            if window.label()=="care" && matches!(event,tauri::WindowEvent::Destroyed){reminders::window_closed(window.app_handle());}
            if window.label()=="games" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    // Ask the trusted host to flush pending saves before requesting native close.
                    if window.app_handle().get_webview_window("games").is_none_or(|webview|webview.eval("window.dispatchEvent(new Event('game-space-close'));".to_string()).is_err()) {
                        let handle=window.app_handle().clone(); tauri::async_runtime::spawn(async move { let _=games::close(&handle).await; });
                    }
                }
                if matches!(event,tauri::WindowEvent::Destroyed) && games::is_active(window.app_handle()) {
                    let handle=window.app_handle().clone(); tauri::async_runtime::spawn(async move { let _=games::close(&handle).await; });
                }
            }
            if window.label() == "control" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close(); let _ = hide_control(window.app_handle().clone());
                }
            }
        })
        .build(context);
    #[cfg(windows)]
    let app = match app {
        Ok(app) => app,
        Err(error) => {
            runtime::windows::show_startup_error(&error.to_string());
            return;
        }
    };
    #[cfg(not(windows))]
    let app = app.expect("小伴启动失败");
    app.run(|_handle, _event| {
        if matches!(_event, tauri::RunEvent::Exit) { chat::shutdown_connectors(_handle); }
        if matches!(_event, tauri::RunEvent::Exit) && _handle.state::<DesktopState>().isolated {
            let _ = startup::set(_handle, false);
        }
        #[cfg(target_os = "macos")]
        if matches!(_event, tauri::RunEvent::Reopen { .. }) {
            account::activated(_handle);
            // A visible wallpaper also makes has_visible_windows true. Dock
            // clicks must restore the settings panel regardless of that flag.
            _handle
                .state::<DesktopState>()
                .reopen_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if let Err(error) = present_control(_handle) {
                eprintln!("无法打开小伴设置面板：{error}");
            }
        }
    });
}

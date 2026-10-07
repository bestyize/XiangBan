#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(any(target_os = "macos", test))]
mod coordinates;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    mode: String,
    expression: String,
    size: f64,
    theme: String,
    paused: bool,
    fps: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: "floating".into(),
            expression: "idle".into(),
            size: 260.0,
            theme: "night".into(),
            paused: false,
            fps: 60,
        }
    }
}

#[derive(Default)]
struct DesktopState {
    settings: Mutex<Settings>,
    wallpaper_parent: Mutex<Option<usize>>,
    ready_scenes: Mutex<HashSet<String>>,
    #[cfg(target_os = "macos")]
    reopen_count: std::sync::atomic::AtomicU32,
}

fn settings(app: &tauri::AppHandle) -> Settings {
    app.state::<DesktopState>().settings.lock().unwrap().clone()
}

fn publish(app: &tauri::AppHandle) {
    let _ = app.emit("settings-changed", settings(app));
}

fn open_scene(app: &tauri::AppHandle, mode: &str) -> Result<(), String> {
    let wallpaper = mode == "wallpaper";
    let label = if wallpaper { "wallpaper" } else { "pet" };
    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|e| e.to_string())?;
        return Ok(());
    }
    let monitor = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("没有找到主显示器")?;
    let scale = monitor.scale_factor();
    let width = monitor.size().width as f64 / scale;
    let height = monitor.size().height as f64 / scale;
    let s = settings(app);
    let scene_w = if wallpaper { width } else { s.size * 1.5 };
    let scene_h = if wallpaper { height } else { s.size * 1.22 };
    #[cfg(windows)]
    let parent = if wallpaper {
        Some(windows::desktop_parent()?)
    } else {
        None
    };
    let builder = WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::App(format!("index.html?scene={mode}").into()),
    )
    .title(if wallpaper {
        "小伴 · 动态壁纸"
    } else {
        "小伴 · 悬浮桌宠"
    })
    .decorations(false)
    .shadow(false)
    .resizable(false)
    .skip_taskbar(true)
    .always_on_top(!wallpaper)
    .transparent(!wallpaper)
    .visible(false)
    .focused(false)
    .inner_size(scene_w, scene_h)
    .position(
        monitor.position().x as f64 / scale
            + if wallpaper {
                0.0
            } else {
                width - scene_w - 70.0
            },
        monitor.position().y as f64 / scale
            + if wallpaper {
                0.0
            } else {
                height - scene_h - 100.0
            },
    );
    #[cfg(target_os = "macos")]
    let builder =
        builder.background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    // Register the parent with Tao itself. Reparenting only with SetParent would
    // be undone when Tao rewrites window styles on show/cursor-state changes.
    #[cfg(windows)]
    let builder = if let Some(parent) = parent {
        builder.parent_raw(::windows::Win32::Foundation::HWND(parent as _))
    } else {
        builder
    };
    let window = builder.build().map_err(|e| e.to_string())?;

    if wallpaper {
        #[cfg(windows)]
        {
            match windows::attach_wallpaper(&window, &monitor, parent.unwrap()) {
                Ok(parent) => {
                    *app.state::<DesktopState>().wallpaper_parent.lock().unwrap() = Some(parent)
                }
                Err(error) => {
                    let _ = window.destroy();
                    return Err(error);
                }
            }
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            let _ = window.destroy();
            return Err("壁纸桌面接入支持 Windows 和 macOS".into());
        }
        window
            .set_ignore_cursor_events(true)
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    if let Err(error) = macos::configure_scene(&window, wallpaper, true) {
        let _ = window.destroy();
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
fn get_platform() -> &'static str {
    std::env::consts::OS
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Settings {
    settings(&app)
}

#[tauri::command]
async fn set_mode(app: tauri::AppHandle, mode: String) -> Result<Settings, String> {
    if !["floating", "wallpaper"].contains(&mode.as_str()) {
        return Err("不支持的显示模式".into());
    }
    open_scene(&app, &mode)?;
    let old_label = if mode == "wallpaper" {
        "pet"
    } else {
        "wallpaper"
    };
    if let Some(old) = app.get_webview_window(old_label) {
        old.destroy().map_err(|e| e.to_string())?;
    }
    app.state::<DesktopState>()
        .ready_scenes
        .lock()
        .unwrap()
        .remove(old_label);
    if mode == "floating" {
        *app.state::<DesktopState>().wallpaper_parent.lock().unwrap() = None;
    }
    app.state::<DesktopState>().settings.lock().unwrap().mode = mode;
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
    let expressions = [
        "idle",
        "happy",
        "excited",
        "laughing",
        "proud",
        "curious",
        "surprised",
        "shy",
        "love",
        "sad",
        "crying",
        "annoyed",
        "angry",
        "sleepy",
        "sleeping",
        "thinking",
        "speaking",
        "listening",
        "waiting",
        "confused",
        "greeting",
        "worried",
        "charging",
        "offline",
        "embarrassed",
    ];
    if !expressions.contains(&expression.as_str()) {
        return Err("不支持的表情".into());
    }
    if !size.is_finite() || !(160.0..=420.0).contains(&size) {
        return Err("尺寸应在 160–420 之间".into());
    }
    if !["night", "dawn"].contains(&theme.as_str()) || ![30, 60].contains(&fps) {
        return Err("不支持的场景设置".into());
    }
    {
        let state = app.state::<DesktopState>();
        let mut s = state.settings.lock().unwrap();
        s.expression = expression;
        s.size = size;
        s.theme = theme;
        s.paused = paused;
        s.fps = fps;
    }
    if let Some(pet) = app.get_webview_window("pet") {
        pet.set_size(tauri::LogicalSize::new(size * 1.5, size * 1.22))
            .map_err(|e| e.to_string())?;
    }
    publish(&app);
    Ok(settings(&app))
}

#[tauri::command]
fn poke(app: tauri::AppHandle) {
    let _ = app.emit("pet-poke", ());
}

#[tauri::command]
fn scene_ready(window: tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    macos::configure_scene(
        &window,
        window.label() == "wallpaper",
        window.label() == "wallpaper",
    )?;
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
        return Err("壁纸模式中的位置由场景决定".into());
    }
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn minimize_control(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("control") {
        let _ = w.minimize();
    }
}

#[tauri::command]
fn show_control(app: tauri::AppHandle) -> Result<(), String> {
    present_control(&app)
}

fn present_control(app: &tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    app.show().map_err(|error| error.to_string())?;
    let window = app
        .get_webview_window("control")
        .ok_or("没有找到小伴设置面板")?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
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

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::{
        menu::{Menu, MenuItem},
        tray::TrayIconBuilder,
    };
    let control = MenuItem::with_id(app, "control", "打开小伴控制台", true, None::<&str>)?;
    let poke = MenuItem::with_id(app, "poke", "戳戳小伴", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "退出小伴", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&control, &poke, &exit])?;
    // A small built-in icon keeps this prototype independent of image assets.
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 3usize..29 {
        for x in 3usize..29 {
            let idx = (y * 32 + x) * 4;
            let screen = (7..25).contains(&x) && (11..24).contains(&y);
            let eye = (11..14).contains(&y) && ((10..13).contains(&x) || (19..22).contains(&x));
            let color = if eye {
                [234, 249, 255, 255]
            } else if screen {
                [9, 10, 14, 255]
            } else {
                [246, 235, 219, 255]
            };
            rgba[idx..idx + 4].copy_from_slice(&color);
        }
    }
    TrayIconBuilder::new()
        .icon(tauri::image::Image::new_owned(rgba, 32, 32))
        .tooltip("小伴 · 桌面陪伴")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "control" => {
                let _ = present_control(app);
            }
            "poke" => {
                let _ = app.emit("pet-poke", ());
            }
            "exit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn main() {
    let mut state = DesktopState::default();
    if std::env::args().any(|argument| argument == "--wallpaper") {
        state.settings.get_mut().unwrap().mode = "wallpaper".into();
    }
    let app = tauri::Builder::default().manage(state)
        .invoke_handler(tauri::generate_handler![get_platform, get_settings, set_mode, update_settings, poke,
            scene_ready, set_hit_test, start_drag, minimize_control, show_control, quit, get_diagnostics])
        .setup(|app| {
            setup_tray(app)?;
            let initial_mode = settings(app.handle()).mode;
            open_scene(app.handle(), &initial_mode).map_err(std::io::Error::other)?;
            #[cfg(windows)] windows::start_cursor_bridge(app.handle().clone());
            #[cfg(target_os = "macos")] macos::start_cursor_bridge(app.handle().clone());
            if std::env::args().any(|a| a == "--smoke-test") {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Synchronize on the actual frontend handshake, including
                    // cold WKWebView startup on a CI Mac.
                    let startup_result = wait_for_scene(&handle, "pet");
                    let floating_before = get_diagnostics(handle.clone());
                    let wallpaper_result = set_mode(handle.clone(), "wallpaper".into()).await
                        .and_then(|_| wait_for_scene(&handle, "wallpaper"));
                    let wallpaper = get_diagnostics(handle.clone());
                    #[cfg(target_os = "macos")]
                    let dock_reopen = macos::smoke_dock_reopen(&handle);
                    let floating_result = set_mode(handle.clone(), "floating".into()).await
                        .and_then(|_| wait_for_scene(&handle, "pet"));
                    let floating_after = get_diagnostics(handle.clone());
                    #[allow(unused_mut)]
                    let mut report = serde_json::json!({"floatingBefore": floating_before, "wallpaper": wallpaper,
                        "floatingAfter": floating_after, "wallpaperResult": wallpaper_result.map(|_| "ok"),
                        "floatingResult": floating_result.map(|_| "ok"), "startupResult": startup_result.map(|_| "ok")});
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
            if window.label() == "control" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close(); window.app_handle().exit(0);
                }
            }
        })
        .build(tauri::generate_context!()).expect("小伴启动失败");
    app.run(|_handle, _event| {
        #[cfg(target_os = "macos")]
        if matches!(_event, tauri::RunEvent::Reopen { .. }) {
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

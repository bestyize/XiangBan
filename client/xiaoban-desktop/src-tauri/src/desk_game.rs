//! Desktop-layer games: one transparent window above the other windows that runs a single sandboxed game page.
//!
//! The page receives only what its manifest asked for (window rectangles, the pointer) and tells the host which
//! rectangles should take the mouse. Everywhere else a click falls through to whatever is underneath, so the
//! layer never gets in the way of the user's work. The game itself stays in a sandboxed frame with no native access.

use crate::desk_windows::{self, Pointer, Rect};
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "deskgame";
const TICK: Duration = Duration::from_millis(33);

#[derive(Default)]
pub struct DeskState {
    operation: tauri::async_runtime::Mutex<()>,
    /// Changes whenever a session starts or ends; a sampler whose number no longer matches stops by itself.
    generation: AtomicU64,
    session: Mutex<Option<Session>>,
}

struct Session {
    id: String,
    capabilities: Vec<String>,
    /// The primary display in CSS pixels, from the origin the system reports window positions in.
    screen: Rect,
    /// Physical pixels per CSS pixel; windows report positions in physical pixels, macOS in points.
    scale: f64,
    /// Where the game wants the mouse, from the screen's top-left corner.
    regions: Vec<Rect>,
}

pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if label == LABEL {
        return Some(["close_desktop_game", "desk_set_regions", "load_game_save", "save_game_save"].contains(&command));
    }
    if ["open_desktop_game", "close_desktop_game", "desk_set_regions"].contains(&command) {
        return Some(label == "control" && command != "desk_set_regions");
    }
    None
}

pub fn active_id(app: &tauri::AppHandle) -> Option<String> {
    app.state::<DeskState>().session.lock().ok()?.as_ref().map(|session| session.id.clone())
}

/// Starts a desktop-layer game from the catalog, replacing one that is already running.
pub async fn open(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    #[cfg(not(any(windows, target_os = "macos")))]
    return Err(format!("此平台暂不支持桌面层游戏（{id}）"));
    #[cfg(any(windows, target_os = "macos"))]
    {
        let state = app.state::<DeskState>();
        let _operation = state.operation.lock().await;
        if crate::games::is_active(app) {
            return Err("游戏空间正在运行，请先退出游戏空间".into());
        }
        let game = crate::games::catalog(app)?
            .games
            .into_iter()
            .find(|game| game["id"] == id)
            .ok_or("没有找到这款本地游戏")?;
        if game["layer"] != "desktop" {
            return Err("这不是桌面层游戏".into());
        }
        stop(app).await?;

        let capabilities: Vec<String> = game["capabilities"].as_array().map(|list| list.iter().filter_map(|c| c.as_str().map(str::to_string)).collect()).unwrap_or_default();
        let monitor = app.primary_monitor().map_err(|e| e.to_string())?.ok_or("没有找到主显示器")?;
        let scale = monitor.scale_factor();
        let screen = Rect::new(monitor.position().x as f64 / scale, monitor.position().y as f64 / scale, monitor.size().width as f64 / scale, monitor.size().height as f64 / scale);
        let token = crate::games::issue_token(app)?;
        let init = json!({"manifest": game, "screen": {"width": screen.w, "height": screen.h}, "capabilities": capabilities});
        let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
        *state.session.lock().unwrap_or_else(|e| e.into_inner()) = Some(Session { id: id.to_string(), capabilities, screen, scale, regions: Vec::new() });

        let builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?scene=desk".into()))
            .initialization_script(format!("if(window===window.top){{window.__XIAOBAN_GAME_TOKEN__={};window.__XIAOBAN_DESK__={init};}}", json!(token)))
            .title("小伴 · 桌面游戏")
            .decorations(false).shadow(false).resizable(false).skip_taskbar(true)
            .always_on_top(true).transparent(true).visible(true).focused(false)
            .inner_size(screen.w, screen.h)
            .position(screen.x, screen.y);
        #[cfg(target_os = "macos")]
        let builder = builder.incognito(true).background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
        #[cfg(windows)]
        let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
        let started = (|| -> Result<(), String> {
            let window = builder.build().map_err(|e| e.to_string())?;
            window.set_ignore_cursor_events(true).map_err(|e| e.to_string())?;
            #[cfg(windows)]
            crate::windows::make_overlay(&window)?;
            #[cfg(target_os = "macos")]
            crate::macos::configure_scene(&window, false, false)?;
            Ok(())
        })();
        if let Err(error) = started {
            let _ = stop(app).await;
            return Err(format!("桌面游戏无法启动：{error}"));
        }
        let sampler = app.clone();
        std::thread::Builder::new().name("desk-game".into()).spawn(move || sample(sampler, generation)).map_err(|e| e.to_string())?;
        let _ = app.emit("desk-game-active", id);
        Ok(())
    }
}

/// Ends the running desktop game, if any, and waits until its window is gone.
pub async fn close(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<DeskState>();
    let _operation = state.operation.lock().await;
    stop(app).await
}

async fn stop(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<DeskState>();
    state.generation.fetch_add(1, Ordering::SeqCst);
    state.session.lock().unwrap_or_else(|e| e.into_inner()).take();
    let _ = app.emit("desk-game-active", Value::Null);
    if let Some(window) = app.get_webview_window(LABEL) {
        window.destroy().map_err(|e| e.to_string())?;
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while app.get_webview_window(LABEL).is_some() {
        if std::time::Instant::now() >= deadline {
            return Err("桌面游戏窗口没有及时关闭，请重试".into());
        }
        tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(30))).await.ok();
    }
    Ok(())
}

#[cfg(any(windows, target_os = "macos"))]
fn raw_windows(scale: f64) -> Vec<desk_windows::Raw> {
    #[cfg(target_os = "macos")]
    {
        let _ = scale;
        crate::macos::window_rects()
    }
    #[cfg(windows)]
    {
        crate::windows::window_rects()
            .into_iter()
            .map(|window| desk_windows::Raw { rect: Rect::new(window.rect.x / scale, window.rect.y / scale, window.rect.w / scale, window.rect.h / scale), ..window })
            .collect()
    }
}

#[cfg(any(windows, target_os = "macos"))]
fn pointer_now(app: &tauri::AppHandle, screen: Rect, scale: f64) -> Option<Pointer> {
    #[cfg(target_os = "macos")]
    {
        let _ = scale;
        let (x, y) = crate::macos::widget_cursor(app).ok()?;
        Some(desk_windows::pointer_on(screen, x, y, crate::macos::left_button_down()))
    }
    #[cfg(windows)]
    {
        let _ = app;
        let (x, y) = crate::windows::cursor_position()?;
        Some(desk_windows::pointer_on(screen, x / scale, y / scale, crate::windows::left_button_down()))
    }
}

/// Runs until this session ends: reads the screen, sends the game what it asked for, and decides whether the
/// desktop layer should take the mouse right now.
#[cfg(any(windows, target_os = "macos"))]
fn sample(app: tauri::AppHandle, generation: u64) {
    let mut capturing = false;
    loop {
        std::thread::sleep(TICK);
        let state = app.state::<DeskState>();
        if state.generation.load(Ordering::SeqCst) != generation {
            break;
        }
        let (screen, scale, capabilities, regions) = match state.session.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            Some(session) => (session.screen, session.scale, session.capabilities.clone(), session.regions.clone()),
            None => break,
        };
        let Some(window) = app.get_webview_window(LABEL) else { break };
        let Some(pointer) = pointer_now(&app, screen, scale) else { continue };

        // While a button is held on the game, keep the mouse so a drag is not dropped when it leaves the rectangle.
        let wanted = desk_windows::over_region(&regions, &pointer) || (capturing && pointer.down);
        if wanted != capturing && window.set_ignore_cursor_events(!wanted).is_ok() {
            capturing = wanted;
        }

        let windows = if capabilities.iter().any(|c| c == "windows") { desk_windows::visible(&raw_windows(scale), screen) } else { Vec::new() };
        let payload = desk_windows::frame(&windows, &pointer, &capabilities);
        if payload.as_object().is_some_and(|fields| !fields.is_empty()) {
            let _ = window.eval(format!("window.dispatchEvent(new CustomEvent('desk-frame',{{detail:{payload}}}));"));
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn sample(_: tauri::AppHandle, _: u64) {}

fn deskgame_only(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != LABEL {
        return Err("只有桌面游戏窗口可以设置鼠标区域".into());
    }
    Ok(())
}

/// The game page reports the rectangles that should take the mouse.
#[tauri::command]
pub fn desk_set_regions(app: tauri::AppHandle, window: tauri::WebviewWindow, rects: Value, host_token: String) -> Result<(), String> {
    deskgame_only(&window)?;
    crate::games::authorize(&app, &host_token)?;
    let state = app.state::<DeskState>();
    let mut guard = state.session.lock().unwrap_or_else(|e| e.into_inner());
    let session = guard.as_mut().ok_or("桌面游戏没有在运行")?;
    session.regions = desk_windows::regions_from(&rects, session.screen.w, session.screen.h);
    Ok(())
}

#[tauri::command]
pub async fn open_desktop_game(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    if window.label() != "control" {
        return Err("只有设置面板可以启动桌面游戏".into());
    }
    open(&app, &id).await
}

/// Stops the running desktop game, from the settings panel, the tray, or the game itself.
#[tauri::command]
pub async fn close_desktop_game(app: tauri::AppHandle, window: tauri::WebviewWindow, host_token: Option<String>) -> Result<(), String> {
    if window.label() == LABEL {
        crate::games::authorize(&app, host_token.as_deref().unwrap_or(""))?;
    }
    close(&app).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_settings_panel_starts_a_desktop_game_and_only_the_game_sets_regions() {
        assert_eq!(permitted("control", "open_desktop_game"), Some(true));
        assert_eq!(permitted("control", "close_desktop_game"), Some(true));
        assert_eq!(permitted("control", "desk_set_regions"), Some(false));
        for label in ["pet", "wallpaper", "media-a", "games", "widget-1", "effects", "care"] {
            for command in ["open_desktop_game", "close_desktop_game", "desk_set_regions"] {
                assert_eq!(permitted(label, command), Some(false), "{label} must not reach {command}");
            }
        }
    }

    #[test]
    fn the_game_window_reaches_its_own_few_commands_and_nothing_else() {
        for command in ["close_desktop_game", "desk_set_regions", "load_game_save", "save_game_save"] {
            assert_eq!(permitted(LABEL, command), Some(true), "{command}");
        }
        for command in ["open_desktop_game", "get_settings", "quit", "set_mode", "open_game_space", "get_game_catalog", "game_smoke_report", "list_games", "delete_user_game", "chat_send"] {
            assert_eq!(permitted(LABEL, command), Some(false), "{command}");
        }
    }
}

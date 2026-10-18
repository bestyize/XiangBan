//! The floating assistant: a small always-on-top capsule that a global shortcut brings up for a quick spoken or
//! typed question. It is an ordinary local page (`?scene=assistant`) that talks to the same chat, speech and
//! recognition commands as the control window, with a narrower set of permissions.

use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub const LABEL: &str = "assistant";
/// The key combinations the user can pick from. A fixed list keeps every saved value parseable.
pub const SHORTCUTS: &[&str] = &["Alt+Space", "Ctrl+Alt+Space", "CmdOrCtrl+Shift+Space", "Alt+Shift+Space"];
pub const DEFAULT_SHORTCUT: &str = "Alt+Space";

const WIDTH: f64 = 560.0;
const MIN_HEIGHT: f64 = 92.0;
const MAX_HEIGHT: f64 = 620.0;
/// Space kept between the capsule and the bottom of the usable screen (above the Dock or taskbar).
const BOTTOM_GAP: f64 = 56.0;

/// A rectangle in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Area {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Area {
    fn bottom(&self) -> i32 {
        self.y + self.h
    }
    fn right(&self) -> i32 {
        self.x + self.w
    }
}

/// Where the capsule sits on a screen, relative to the default spot (bottom centre of the usable area), in logical
/// pixels. Dragging it somewhere changes this, and it is applied again on whichever screen the pointer is on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Offset {
    dx: f64,
    dy: f64,
}

#[derive(Default)]
pub struct AssistantState {
    registered: Mutex<Option<String>>,
    error: Mutex<String>,
    moved: Mutex<Offset>,
    /// The page has loaded and listens for `assistant-open`.
    ready: AtomicBool,
}

const WINDOW_COMMANDS: &[&str] = &["assistant_ready", "assistant_hide", "assistant_fit", "assistant_drag", "assistant_open_chat", "assistant_open_settings"];
const CONTROL_COMMANDS: &[&str] = &["assistant_status", "assistant_configure"];
/// What the capsule may ask of the host besides its own window commands: ask and answer, speak, listen.
/// Deleting, renaming, compacting or changing the trust mode of conversations stays with the control window.
const SHARED_COMMANDS: &[&str] = &[
    "chat_overview", "chat_load_session", "chat_new_session", "chat_send", "chat_cancel", "chat_context",
    "chat_answer_approval", "chat_answer_question",
    "tts_get_settings", "tts_synthesize", "tts_cancel",
    "asr_get_status", "asr_prepare", "asr_transcribe",
    "open_download_url",
];

/// `Some(answer)` when this module decides for the window, `None` to let the other rules decide.
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if command.starts_with("assistant_") {
        return Some(if WINDOW_COMMANDS.contains(&command) {
            label == LABEL
        } else if CONTROL_COMMANDS.contains(&command) {
            label == "control"
        } else {
            false
        });
    }
    (label == LABEL).then(|| SHARED_COMMANDS.contains(&command))
}

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                toggle(app);
            }
        })
        .build()
}

pub fn start(app: &tauri::AppHandle) {
    let settings = crate::settings(app);
    if let Err(error) = apply(app, settings.assistant_enabled, &settings.assistant_shortcut) {
        eprintln!("悬浮助手快捷键没有注册：{error}");
    }
}

fn shortcut_label(shortcut: &str) -> String {
    let mac = cfg!(target_os = "macos");
    shortcut
        .replace("CmdOrCtrl", if mac { "⌘" } else { "Ctrl" })
        .replace("Alt", if mac { "⌥" } else { "Alt" })
        .replace("Ctrl", if mac { "⌃" } else { "Ctrl" })
        .replace("Shift", if mac { "⇧" } else { "Shift" })
        .replace('+', if mac { " " } else { "+" })
}

/// Registers `shortcut` (and drops the one registered before). On failure the previous shortcut stays in force.
pub fn apply(app: &tauri::AppHandle, enabled: bool, shortcut: &str) -> Result<(), String> {
    let state = app.state::<AssistantState>();
    let mut current = state.registered.lock().unwrap();
    if enabled && current.as_deref() == Some(shortcut) {
        return Ok(());
    }
    let previous = current.take();
    if let Some(old) = &previous {
        let _ = app.global_shortcut().unregister(old.as_str());
    }
    if !enabled {
        state.error.lock().unwrap().clear();
        return Ok(());
    }
    match app.global_shortcut().register(shortcut) {
        Ok(()) => {
            *current = Some(shortcut.to_string());
            state.error.lock().unwrap().clear();
            Ok(())
        }
        Err(error) => {
            if let Some(old) = previous {
                if app.global_shortcut().register(old.as_str()).is_ok() {
                    *current = Some(old);
                }
            }
            let message = format!("快捷键 {} 无法使用，可能已被其他应用占用（{error}）", shortcut_label(shortcut));
            *state.error.lock().unwrap() = message.clone();
            Err(message)
        }
    }
}

fn status(app: &tauri::AppHandle) -> Value {
    let settings = crate::settings(app);
    let state = app.state::<AssistantState>();
    let registered = state.registered.lock().unwrap().is_some();
    let error = state.error.lock().unwrap().clone();
    json!({
        "enabled": settings.assistant_enabled,
        "shortcut": settings.assistant_shortcut,
        "shortcuts": SHORTCUTS.iter().map(|s| json!({ "value": s, "label": shortcut_label(s) })).collect::<Vec<_>>(),
        "registered": registered,
        "error": error,
    })
}

fn build(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    let builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?scene=assistant".into()))
        .title("小伴 · 助手")
        .decorations(false).shadow(false).resizable(false).skip_taskbar(true)
        .always_on_top(true).transparent(true).visible(false).focused(false)
        .inner_size(WIDTH, MIN_HEIGHT);
    #[cfg(windows)]
    let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
    #[cfg(target_os = "macos")]
    let builder = builder.background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    let window = builder.build().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    crate::macos::configure_scene(&window, false, false)?;
    Ok(window)
}

fn area_of(monitor: &tauri::Monitor) -> Area {
    let area = monitor.work_area();
    Area { x: area.position.x, y: area.position.y, w: area.size.width as i32, h: area.size.height as i32 }
}

/// The left edge and the bottom edge of a capsule `width` wide, for the given usable area and offset. It always stays
/// fully on the screen, so a spot saved on a bigger screen still works on a smaller one.
fn anchor(area: Area, scale: f64, width: i32, min_height: i32, offset: Offset) -> (i32, i32) {
    let gap = (BOTTOM_GAP * scale).round() as i32;
    let x = area.x + (area.w - width) / 2 + (offset.dx * scale).round() as i32;
    let bottom = area.bottom() - gap + (offset.dy * scale).round() as i32;
    (x.clamp(area.x, (area.right() - width).max(area.x)), bottom.clamp(area.y + min_height, area.bottom()))
}

/// Where the top edge goes when the capsule changes to `height` with its bottom edge held in place; if that would
/// push it off the top of the screen it is pulled back down instead.
fn top_for(area: Area, bottom: i32, height: i32) -> i32 {
    (bottom - height).clamp(area.y, (area.bottom() - height).max(area.y))
}

/// How far the capsule's bottom centre is from the default spot of the screen it is on.
fn offset_of(area: Area, scale: f64, x: i32, width: i32, bottom: i32) -> Offset {
    let gap = (BOTTOM_GAP * scale).round() as i32;
    let dx = (x + width / 2) - (area.x + area.w / 2);
    let dy = bottom - (area.bottom() - gap);
    Offset { dx: f64::from(dx) / scale, dy: f64::from(dy) / scale }
}

/// Puts the capsule on the screen the pointer is on, at its default spot or where the user last left it.
fn place(app: &tauri::AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|point| app.monitor_from_point(point.x, point.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or("没有找到显示器")?;
    let scale = monitor.scale_factor();
    let width = (WIDTH * scale).round() as i32;
    let height = (MIN_HEIGHT * scale).round() as i32;
    let offset = *app.state::<AssistantState>().moved.lock().unwrap();
    let (x, bottom) = anchor(area_of(&monitor), scale, width, height, offset);
    // Move first so a window arriving on another screen takes that screen's scale before it is sized.
    let top = bottom - height;
    window.set_position(PhysicalPosition::new(x, top)).map_err(|e| e.to_string())?;
    window.set_size(PhysicalSize::new(width as u32, height as u32)).map_err(|e| e.to_string())?;
    window.set_position(PhysicalPosition::new(x, top)).map_err(|e| e.to_string())
}

/// Sets the height in logical pixels. The bottom edge stays wherever the capsule is now (the user may have dragged
/// it), so it grows upward.
fn resize(window: &WebviewWindow, height: f64) -> Result<(), String> {
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let target = (height.clamp(MIN_HEIGHT, MAX_HEIGHT) * scale).round() as i32;
    let bottom = position.y + size.height as i32;
    let top = match window.current_monitor().map_err(|e| e.to_string())? {
        Some(monitor) => top_for(area_of(&monitor), bottom, target),
        None => bottom - target,
    };
    window.set_position(PhysicalPosition::new(position.x, top)).map_err(|e| e.to_string())?;
    window.set_size(PhysicalSize::new(size.width, target as u32)).map_err(|e| e.to_string())?;
    window.set_position(PhysicalPosition::new(position.x, top)).map_err(|e| e.to_string())
}

/// Remembers where the user left the capsule, so the next time it opens it is there and not back at the default.
fn remember(app: &tauri::AppHandle, window: &WebviewWindow) {
    let (Ok(position), Ok(size), Ok(Some(monitor)), Ok(scale)) =
        (window.outer_position(), window.outer_size(), window.current_monitor(), window.scale_factor())
    else {
        return;
    };
    let offset = offset_of(area_of(&monitor), scale, position.x, size.width as i32, position.y + size.height as i32);
    *app.state::<AssistantState>().moved.lock().unwrap() = offset;
}

pub fn toggle(app: &tauri::AppHandle) {
    if !crate::settings(app).assistant_enabled {
        return;
    }
    let visible = app.get_webview_window(LABEL).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    let result = if visible { hide(app) } else { show(app) };
    if let Err(error) = result {
        eprintln!("悬浮助手：{error}");
    }
}

pub fn show(app: &tauri::AppHandle) -> Result<(), String> {
    let window = match app.get_webview_window(LABEL) {
        Some(window) => window,
        None => build(app)?,
    };
    place(app, &window)?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    // A page that is still loading announces itself with `assistant_ready` and is opened then.
    if app.state::<AssistantState>().ready.load(Ordering::SeqCst) {
        let _ = window.emit("assistant-open", ());
    }
    Ok(())
}

pub fn hide(app: &tauri::AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(LABEL) else { return Ok(()) };
    if window.is_visible().unwrap_or(false) {
        remember(app, &window);
    }
    let _ = window.emit("assistant-close", ());
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn assistant_status(app: tauri::AppHandle) -> Value {
    status(&app)
}

#[tauri::command]
pub fn assistant_configure(app: tauri::AppHandle, enabled: Option<bool>, shortcut: Option<String>) -> Result<Value, String> {
    if let Some(shortcut) = &shortcut {
        if !SHORTCUTS.contains(&shortcut.as_str()) {
            return Err(format!("不支持的快捷键：{shortcut}"));
        }
    }
    let state = app.state::<crate::DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    let mut next = crate::settings(&app);
    if let Some(enabled) = enabled {
        next.assistant_enabled = enabled;
    }
    if let Some(shortcut) = shortcut {
        next.assistant_shortcut = shortcut;
    }
    apply(&app, next.assistant_enabled, &next.assistant_shortcut)?;
    if !next.assistant_enabled {
        let _ = hide(&app);
    }
    crate::persist(&app, &next)?;
    *state.settings.lock().unwrap() = next;
    crate::publish(&app);
    Ok(status(&app))
}

#[tauri::command]
pub fn assistant_ready(app: tauri::AppHandle) {
    app.state::<AssistantState>().ready.store(true, Ordering::SeqCst);
    if let Some(window) = app.get_webview_window(LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.emit("assistant-open", ());
        }
    }
}

#[tauri::command]
pub fn assistant_hide(app: tauri::AppHandle) -> Result<(), String> {
    hide(&app)
}

#[tauri::command]
pub fn assistant_fit(window: WebviewWindow, height: f64) -> Result<(), String> {
    if !height.is_finite() {
        return Err("高度不对".into());
    }
    // The page asks to grow while the user may already have dismissed it.
    if !window.is_visible().unwrap_or(false) {
        return Ok(());
    }
    resize(&window, height)
}

/// Lets the user drag the capsule by its empty parts. The page calls this on the mouse press.
#[tauri::command]
pub fn assistant_drag(window: WebviewWindow) -> Result<(), String> {
    if window.label() != LABEL {
        return Err("只有助手窗口可以拖动".into());
    }
    window.start_dragging().map_err(|e| e.to_string())
}

/// Brings the control window forward on the chat page, with the conversation the capsule was using.
#[tauri::command]
pub fn assistant_open_chat(app: tauri::AppHandle, session: Option<String>) -> Result<(), String> {
    hide(&app)?;
    crate::present_control(&app)?;
    app.emit_to("control", "assistant-open-chat", json!({ "session": session })).map_err(|e| e.to_string())
}

/// Brings the control window forward on a settings tab ("ai" for the model, "voice" for speech).
#[tauri::command]
pub fn assistant_open_settings(app: tauri::AppHandle, tab: String) -> Result<(), String> {
    if !["ai", "voice"].contains(&tab.as_str()) {
        return Err("没有这个设置页".into());
    }
    hide(&app)?;
    crate::present_control(&app)?;
    app.emit_to("control", "assistant-open-settings", json!({ "tab": tab })).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri_plugin_global_shortcut::Shortcut;

    /// A 1440x900 screen with a 25px menu bar, and the same kind of screen at 2x.
    const SCREEN: Area = Area { x: 0, y: 25, w: 1440, h: 875 };
    const RETINA: Area = Area { x: 0, y: 50, w: 2880, h: 1750 };

    #[test]
    fn by_default_the_capsule_sits_at_the_bottom_centre_of_the_usable_area() {
        assert_eq!(anchor(SCREEN, 1.0, 560, 112, Offset::default()), (440, 900 - 56));
        assert_eq!(anchor(RETINA, 2.0, 1120, 224, Offset::default()), (880, 1800 - 112));
    }

    #[test]
    fn a_dragged_spot_comes_back_and_is_kept_on_screen_when_the_screen_is_smaller() {
        assert_eq!(anchor(SCREEN, 1.0, 560, 112, Offset { dx: -300.0, dy: -200.0 }), (140, 900 - 56 - 200));
        // far to the right and below the bottom edge: pulled back inside
        assert_eq!(anchor(SCREEN, 1.0, 560, 112, Offset { dx: 5000.0, dy: 5000.0 }), (1440 - 560, 900));
        // far to the left and above the top edge
        assert_eq!(anchor(SCREEN, 1.0, 560, 112, Offset { dx: -5000.0, dy: -5000.0 }), (0, 25 + 112));
    }

    #[test]
    fn where_the_user_leaves_it_is_what_is_remembered_and_the_default_is_zero() {
        let (x, bottom) = anchor(RETINA, 2.0, 1120, 224, Offset::default());
        assert_eq!(offset_of(RETINA, 2.0, x, 1120, bottom), Offset::default());
        let dragged = offset_of(RETINA, 2.0, x + 200, 1120, bottom - 400);
        assert_eq!(dragged, Offset { dx: 100.0, dy: -200.0 });
        assert_eq!(anchor(RETINA, 2.0, 1120, 224, dragged), (x + 200, bottom - 400));
        // a spot saved on the retina screen lands in the same place, in logical terms, on the plain one
        assert_eq!(anchor(SCREEN, 1.0, 560, 112, dragged), (440 + 100, 900 - 56 - 200));
    }

    #[test]
    fn growing_keeps_the_bottom_edge_and_never_leaves_the_screen() {
        assert_eq!(top_for(SCREEN, 800, 300), 500);
        // taller than the room above the bottom edge: it stops at the top of the usable area
        assert_eq!(top_for(SCREEN, 300, 620), 25);
        // dragged to the very top: it grows downward instead of going off the screen
        assert_eq!(top_for(SCREEN, 137, 700), 25);
        // and never past the bottom edge
        assert_eq!(top_for(SCREEN, 899, 600), 299);
        assert_eq!(top_for(SCREEN, 880, 100), 780);
    }

    #[test]
    fn only_the_assistant_window_can_start_a_drag() {
        assert_eq!(permitted(LABEL, "assistant_drag"), Some(true));
        assert_eq!(permitted("control", "assistant_drag"), Some(false));
        assert_eq!(permitted("pet", "assistant_drag"), Some(false));
    }

    #[test]
    fn every_offered_shortcut_is_a_valid_key_combination() {
        for shortcut in SHORTCUTS {
            assert!(shortcut.parse::<Shortcut>().is_ok(), "{shortcut}");
        }
        assert!(SHORTCUTS.contains(&DEFAULT_SHORTCUT));
    }

    #[test]
    fn the_capsule_may_chat_speak_and_listen_but_not_manage_conversations_or_voice_settings() {
        for command in ["chat_send", "chat_cancel", "chat_answer_approval", "tts_synthesize", "asr_transcribe", "assistant_fit"] {
            assert_eq!(permitted(LABEL, command), Some(true), "{command}");
        }
        for command in ["chat_delete_session", "chat_set_permission_mode", "chat_compact", "asr_set_enabled", "tts_save_settings", "get_ai_config", "assistant_configure", "quit"] {
            assert_eq!(permitted(LABEL, command), Some(false), "{command}");
        }
    }

    #[test]
    fn only_the_right_window_may_use_each_assistant_command() {
        assert_eq!(permitted("control", "assistant_configure"), Some(true));
        assert_eq!(permitted("control", "assistant_status"), Some(true));
        assert_eq!(permitted("control", "assistant_hide"), Some(false));
        assert_eq!(permitted("pet", "assistant_status"), Some(false));
        assert_eq!(permitted("media-a", "assistant_hide"), Some(false));
        assert_eq!(permitted("control", "assistant_unknown"), Some(false));
        assert_eq!(permitted("control", "chat_send"), None);
        assert_eq!(permitted("media-a", "chat_send"), None);
    }
}

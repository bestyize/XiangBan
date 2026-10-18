//! The floating assistant: a small always-on-top ball that opens into a capsule for a quick spoken or typed
//! question, or a capsule alone that a global shortcut brings up. It is an ordinary local page (`?scene=assistant`)
//! that talks to the same chat, speech and recognition commands as the control window, with a narrower set of
//! permissions. One window serves both shapes; the host decides which one is showing.

use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub const LABEL: &str = "assistant";
/// The key combinations the user can pick from. A fixed list keeps every saved value parseable.
pub const SHORTCUTS: &[&str] = &["Alt+Space", "Ctrl+Alt+Space", "CmdOrCtrl+Shift+Space", "Alt+Shift+Space"];
pub const DEFAULT_SHORTCUT: &str = "Alt+Space";
/// What the capsule does when it opens.
pub const OPEN_MODES: &[&str] = &["voice", "type"];
/// When answers are read aloud.
pub const READ_MODES: &[&str] = &["voice", "always", "never"];

const WIDTH: f64 = 480.0;
const MIN_HEIGHT: f64 = 92.0;
const MAX_HEIGHT: f64 = 620.0;
/// Space kept between the capsule and the bottom of the usable screen (above the Dock or taskbar).
const BOTTOM_GAP: f64 = 56.0;
/// The ball's window, in logical pixels. The ball itself is drawn a little smaller inside it.
const BALL_WINDOW: f64 = 72.0;
/// Where the ball starts: close to the right edge, a little above the Dock.
const BALL_RIGHT_GAP: f64 = 16.0;
const BALL_BOTTOM_GAP: f64 = 160.0;
/// The ball's position is saved this long after the last time it moved.
const SAVE_AFTER_MOVE: Duration = Duration::from_millis(600);

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

/// What the one assistant window is showing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Hidden,
    Ball,
    Capsule,
}

#[derive(Default)]
pub struct AssistantState {
    registered: Mutex<Option<String>>,
    error: Mutex<String>,
    /// Where the capsule was left, for this run.
    moved: Mutex<Offset>,
    mode: Mutex<Mode>,
    /// The ball's top-left corner in physical pixels as it was last seen, for this run. The saved offset in the
    /// settings decides the first spot after a start.
    ball_at: Mutex<Option<(i32, i32)>>,
    move_tick: AtomicU64,
    saving: AtomicBool,
    /// The page has loaded and listens for the assistant events.
    ready: AtomicBool,
}

const WINDOW_COMMANDS: &[&str] = &[
    "assistant_ready", "assistant_hide", "assistant_fit", "assistant_drag", "assistant_unfold", "assistant_prefs",
    "assistant_open_chat", "assistant_open_settings",
];
const CONTROL_COMMANDS: &[&str] = &["assistant_status", "assistant_configure", "assistant_try"];
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
        eprintln!("超级小伴快捷键没有注册：{error}");
    }
    if let Err(error) = sync(app) {
        eprintln!("超级小伴：{error}");
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
        "shortcutLabel": shortcut_label(&settings.assistant_shortcut),
        "ball": settings.assistant_ball,
        "open": settings.assistant_open,
        "read": settings.assistant_read,
        "hintSeen": settings.assistant_hint_seen,
        "shortcuts": SHORTCUTS.iter().map(|s| json!({ "value": s, "label": shortcut_label(s) })).collect::<Vec<_>>(),
        "registered": registered,
        "error": error,
    })
}

fn build(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    let builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?scene=assistant".into()))
        .title("小伴 · 超级小伴")
        .decorations(false).shadow(false).resizable(false).skip_taskbar(true)
        .always_on_top(true).transparent(true).visible(false).focused(false)
        // The ball sits over other apps, which are the active ones: the click that lands on it must reach the page.
        .accept_first_mouse(true)
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

/// Keeps a square window of `size` fully inside the usable area.
fn keep_inside(area: Area, x: i32, y: i32, size: i32) -> (i32, i32) {
    (x.clamp(area.x, (area.right() - size).max(area.x)), y.clamp(area.y, (area.bottom() - size).max(area.y)))
}

/// The ball's top-left corner on a screen before any offset: near the right edge, above the Dock.
fn ball_default(area: Area, scale: f64, size: i32) -> (i32, i32) {
    (
        area.right() - size - (BALL_RIGHT_GAP * scale).round() as i32,
        area.bottom() - size - (BALL_BOTTOM_GAP * scale).round() as i32,
    )
}

/// Where the ball goes for a saved offset, kept on the screen.
fn ball_anchor(area: Area, scale: f64, size: i32, offset: Offset) -> (i32, i32) {
    let (x, y) = ball_default(area, scale, size);
    keep_inside(area, x + (offset.dx * scale).round() as i32, y + (offset.dy * scale).round() as i32, size)
}

/// The offset that puts the ball where it is now.
fn ball_offset(area: Area, scale: f64, size: i32, x: i32, y: i32) -> Offset {
    let (default_x, default_y) = ball_default(area, scale, size);
    Offset { dx: f64::from(x - default_x) / scale, dy: f64::from(y - default_y) / scale }
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

pub(crate) fn remember_position(app: &tauri::AppHandle, window: &WebviewWindow) { remember(app, window); }

fn mode(app: &tauri::AppHandle) -> Mode {
    *app.state::<AssistantState>().mode.lock().unwrap()
}

fn set_mode(app: &tauri::AppHandle, mode: Mode) {
    *app.state::<AssistantState>().mode.lock().unwrap() = mode;
}

fn window(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    match app.get_webview_window(LABEL) {
        Some(window) => Ok(window),
        None => build(app),
    }
}

/// Tells the page what to draw. A page that is still loading asks again with `assistant_ready`.
fn announce(app: &tauri::AppHandle, window: &WebviewWindow) {
    if !app.state::<AssistantState>().ready.load(Ordering::SeqCst) {
        return;
    }
    let _ = window.emit("assistant-ball", mode(app) == Mode::Ball);
    if mode(app) == Mode::Capsule {
        let _ = window.emit("assistant-open", ());
    }
}

/// Where the ball goes: where it was last seen on a screen that still exists, else the saved offset on the main screen.
fn ball_target(app: &tauri::AppHandle) -> Result<(i32, i32, i32), String> {
    let seen = *app.state::<AssistantState>().ball_at.lock().unwrap();
    if let Some((x, y)) = seen {
        if let Ok(Some(monitor)) = app.monitor_from_point(f64::from(x) + 1.0, f64::from(y) + 1.0) {
            let size = (BALL_WINDOW * monitor.scale_factor()).round() as i32;
            let (x, y) = keep_inside(area_of(&monitor), x, y, size);
            return Ok((x, y, size));
        }
    }
    let monitor = app.primary_monitor().ok().flatten().ok_or("没有找到显示器")?;
    let scale = monitor.scale_factor();
    let size = (BALL_WINDOW * scale).round() as i32;
    let settings = crate::settings(app);
    let offset = Offset { dx: settings.assistant_ball_dx, dy: settings.assistant_ball_dy };
    let (x, y) = ball_anchor(area_of(&monitor), scale, size, offset);
    Ok((x, y, size))
}

fn show_ball(app: &tauri::AppHandle) -> Result<(), String> {
    let window = window(app)?;
    let (x, y, size) = ball_target(app)?;
    set_mode(app, Mode::Ball);
    // Move first so a window arriving on another screen takes that screen's scale before it is sized.
    window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    window.set_size(PhysicalSize::new(size as u32, size as u32)).map_err(|e| e.to_string())?;
    window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    // Shown without taking the focus: the ball appears over whatever the user is doing.
    crate::window_control::apply_saved(app, &window, "topmost")?;
    window.show().map_err(|e| e.to_string())?;
    announce(app, &window);
    Ok(())
}

fn hide_window(app: &tauri::AppHandle) -> Result<(), String> {
    let was = mode(app);
    set_mode(app, Mode::Hidden);
    let Some(window) = app.get_webview_window(LABEL) else { return Ok(()) };
    if was == Mode::Capsule {
        let _ = window.emit("assistant-close", ());
    }
    let _ = window.emit("assistant-ball", false);
    window.hide().map_err(|e| e.to_string())
}

pub(crate) fn hide_surface(app: &tauri::AppHandle) -> Result<(), String> { hide_window(app) }

/// Opens the capsule, from the ball or from nothing.
pub fn expand(app: &tauri::AppHandle) -> Result<(), String> {
    if !crate::settings(app).assistant_enabled {
        return Ok(());
    }
    let window = window(app)?;
    if mode(app) == Mode::Ball {
        // The ball may have been moved a moment ago; keep that spot for when it comes back.
        save_ball(app, &window);
    }
    place(app, &window)?;
    set_mode(app, Mode::Capsule);
    crate::window_control::apply_saved(app, &window, "topmost")?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    announce(app, &window);
    Ok(())
}

/// Puts the capsule away: back to the ball when there is one, otherwise out of sight.
pub fn collapse(app: &tauri::AppHandle) -> Result<(), String> {
    if mode(app) != Mode::Capsule {
        return Ok(());
    }
    let Some(window) = app.get_webview_window(LABEL) else { return Ok(()) };
    if window.is_visible().unwrap_or(false) {
        remember(app, &window);
    }
    let _ = window.emit("assistant-close", ());
    let settings = crate::settings(app);
    if settings.assistant_enabled && settings.assistant_ball {
        show_ball(app)
    } else {
        hide_window(app)
    }
}

/// Makes the window match the settings: a ball when it is wanted and nothing else is showing, nothing when the
/// assistant or the ball was turned off.
pub fn sync(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = crate::settings(app);
    match mode(app) {
        Mode::Hidden if settings.assistant_enabled && settings.assistant_ball => show_ball(app),
        Mode::Ball if !(settings.assistant_enabled && settings.assistant_ball) => hide_window(app),
        Mode::Capsule if !settings.assistant_enabled => hide_window(app),
        _ => Ok(()),
    }
}

pub fn toggle(app: &tauri::AppHandle) {
    if !crate::settings(app).assistant_enabled {
        return;
    }
    let result = if mode(app) == Mode::Capsule { collapse(app) } else { expand(app) };
    if let Err(error) = result {
        eprintln!("超级小伴：{error}");
    }
}

fn persist_ball(app: &tauri::AppHandle, offset: Offset) {
    let state = app.state::<crate::DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    let mut next = crate::settings(app);
    if (next.assistant_ball_dx - offset.dx).abs() < 0.5 && (next.assistant_ball_dy - offset.dy).abs() < 0.5 {
        return;
    }
    next.assistant_ball_dx = offset.dx;
    next.assistant_ball_dy = offset.dy;
    if crate::persist(app, &next).is_ok() {
        *state.settings.lock().unwrap() = next;
    }
}

/// Notes where the ball is now, for this run and for the next one.
fn save_ball(app: &tauri::AppHandle, window: &WebviewWindow) {
    if mode(app) != Mode::Ball {
        return;
    }
    let (Ok(position), Ok(size), Ok(Some(monitor)), Ok(scale)) =
        (window.outer_position(), window.outer_size(), window.current_monitor(), window.scale_factor())
    else {
        return;
    };
    *app.state::<AssistantState>().ball_at.lock().unwrap() = Some((position.x, position.y));
    persist_ball(app, ball_offset(area_of(&monitor), scale, size.width as i32, position.x, position.y));
}

/// The window moved. A ball being dragged moves many times a second; its spot is saved once it has been still.
pub fn moved(app: &tauri::AppHandle) {
    if mode(app) != Mode::Ball {
        return;
    }
    let state = app.state::<AssistantState>();
    state.move_tick.fetch_add(1, Ordering::SeqCst);
    if state.saving.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let state = app.state::<AssistantState>();
        let mut seen = state.move_tick.load(Ordering::SeqCst);
        loop {
            std::thread::sleep(SAVE_AFTER_MOVE);
            let now = state.move_tick.load(Ordering::SeqCst);
            if now == seen {
                break;
            }
            seen = now;
        }
        if let Some(window) = app.get_webview_window(LABEL) {
            save_ball(&app, &window);
        }
        state.saving.store(false, Ordering::SeqCst);
        // A move that came in while the spot was being saved still has to be saved.
        if state.move_tick.load(Ordering::SeqCst) != seen {
            moved(&app);
        }
    });
}

#[tauri::command]
pub fn assistant_status(app: tauri::AppHandle) -> Value {
    status(&app)
}

/// What the control window may change. Absent fields stay as they are.
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Patch {
    enabled: Option<bool>,
    shortcut: Option<String>,
    ball: Option<bool>,
    open: Option<String>,
    read: Option<String>,
    hint_seen: Option<bool>,
    /// Puts the ball and the capsule back at their default spots.
    reset_position: Option<bool>,
}

#[tauri::command]
pub fn assistant_configure(app: tauri::AppHandle, patch: Patch) -> Result<Value, String> {
    if let Some(shortcut) = &patch.shortcut {
        if !SHORTCUTS.contains(&shortcut.as_str()) {
            return Err(format!("不支持的快捷键：{shortcut}"));
        }
    }
    if patch.open.as_deref().is_some_and(|value| !OPEN_MODES.contains(&value)) {
        return Err("不支持的打开方式".into());
    }
    if patch.read.as_deref().is_some_and(|value| !READ_MODES.contains(&value)) {
        return Err("不支持的朗读方式".into());
    }
    let state = app.state::<crate::DesktopState>();
    let _update = state.update_lock.lock().unwrap();
    let mut next = crate::settings(&app);
    if let Some(enabled) = patch.enabled {
        next.assistant_enabled = enabled;
    }
    if let Some(shortcut) = patch.shortcut {
        next.assistant_shortcut = shortcut;
    }
    if let Some(ball) = patch.ball {
        next.assistant_ball = ball;
    }
    if let Some(open) = patch.open {
        next.assistant_open = open;
    }
    if let Some(read) = patch.read {
        next.assistant_read = read;
    }
    if let Some(seen) = patch.hint_seen {
        next.assistant_hint_seen = seen;
    }
    let reset = patch.reset_position == Some(true);
    if reset {
        next.assistant_ball_dx = 0.0;
        next.assistant_ball_dy = 0.0;
    }
    apply(&app, next.assistant_enabled, &next.assistant_shortcut)?;
    crate::persist(&app, &next)?;
    *state.settings.lock().unwrap() = next;
    if reset {
        *app.state::<AssistantState>().ball_at.lock().unwrap() = None;
        *app.state::<AssistantState>().moved.lock().unwrap() = Offset::default();
        match mode(&app) {
            Mode::Ball => show_ball(&app)?,
            Mode::Capsule => {
                if let Some(window) = app.get_webview_window(LABEL) {
                    place(&app, &window)?;
                }
            }
            Mode::Hidden => {}
        }
    }
    sync(&app)?;
    crate::publish(&app);
    Ok(status(&app))
}

/// Opens the capsule now, so the settings page can show what the shortcut does.
#[tauri::command]
pub fn assistant_try(app: tauri::AppHandle) -> Result<(), String> {
    expand(&app)
}

/// What the capsule needs to know before it opens.
#[tauri::command]
pub fn assistant_prefs(app: tauri::AppHandle) -> Value {
    let settings = crate::settings(&app);
    json!({ "open": settings.assistant_open, "read": settings.assistant_read, "appearance": settings.appearance })
}

/// The ball was clicked.
#[tauri::command]
pub fn assistant_unfold(app: tauri::AppHandle) -> Result<(), String> {
    expand(&app)
}

#[tauri::command]
pub fn assistant_ready(app: tauri::AppHandle) {
    app.state::<AssistantState>().ready.store(true, Ordering::SeqCst);
    if let Some(window) = app.get_webview_window(LABEL) {
        announce(&app, &window);
    }
}

#[tauri::command]
pub fn assistant_hide(app: tauri::AppHandle) -> Result<(), String> {
    collapse(&app)
}

#[tauri::command]
pub fn assistant_fit(window: WebviewWindow, height: f64) -> Result<(), String> {
    if !height.is_finite() {
        return Err("高度不对".into());
    }
    // The page asks to grow while the user may already have put it away, or while the ball is showing.
    if mode(window.app_handle()) != Mode::Capsule || !window.is_visible().unwrap_or(false) {
        return Ok(());
    }
    resize(&window, height)
}

/// Lets the user drag the capsule by its empty parts, or the ball by itself. The page calls this once the pointer
/// has moved.
#[tauri::command]
pub fn assistant_drag(window: WebviewWindow) -> Result<(), String> {
    if window.label() != LABEL {
        return Err("只有超级小伴的窗口可以拖动".into());
    }
    window.start_dragging().map_err(|e| e.to_string())
}

/// Brings the control window forward on the chat page, with the conversation the capsule was using.
#[tauri::command]
pub fn assistant_open_chat(app: tauri::AppHandle, session: Option<String>) -> Result<(), String> {
    collapse(&app)?;
    crate::present_control(&app)?;
    app.emit_to("control", "assistant-open-chat", json!({ "session": session })).map_err(|e| e.to_string())
}

/// Brings the control window forward on the model settings ("ai") or on the assistant's own page ("assistant").
#[tauri::command]
pub fn assistant_open_settings(app: tauri::AppHandle, tab: String) -> Result<(), String> {
    if !["ai", "assistant"].contains(&tab.as_str()) {
        return Err("没有这个设置页".into());
    }
    collapse(&app)?;
    crate::present_control(&app)?;
    app.emit_to("control", "assistant-open-settings", json!({ "tab": tab })).map_err(|e| e.to_string())
}

/// For the smoke test: the ball shows, opens into the capsule and comes back to the same spot.
pub async fn smoke(app: &tauri::AppHandle) -> Value {
    let wait = || tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(400)));
    let describe = |name: &str| -> Value {
        let Some(window) = app.get_webview_window(LABEL) else { return json!({ "step": name, "window": false }) };
        let scale = window.scale_factor().unwrap_or(1.0);
        let size = window.outer_size().map(|s| (f64::from(s.width) / scale, f64::from(s.height) / scale)).unwrap_or((0.0, 0.0));
        let position = window.outer_position().map(|p| (p.x, p.y)).unwrap_or((0, 0));
        #[cfg(target_os = "macos")]
        let level = crate::macos::window_level(&window).ok();
        #[cfg(not(target_os = "macos"))]
        let level: Option<i64> = None;
        json!({ "step": name, "window": true, "mode": format!("{:?}", mode(app)), "visible": window.is_visible().unwrap_or(false),
            "width": size.0, "height": size.1, "x": position.0, "y": position.1, "level": level })
    };
    let mut steps = Vec::new();
    let _ = sync(app);
    let _ = wait().await;
    steps.push(describe("start"));
    let opened = expand(app);
    let _ = wait().await;
    steps.push(describe("expanded"));
    let closed = collapse(app);
    let _ = wait().await;
    steps.push(describe("collapsed"));
    json!({ "steps": steps, "expand": opened.err(), "collapse": closed.err() })
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
    fn the_ball_starts_near_the_right_edge_above_the_dock_and_stays_on_the_screen() {
        // 72px window on a 1440x875 usable area: 16px from the right edge, 160px above the bottom
        assert_eq!(ball_anchor(SCREEN, 1.0, 72, Offset::default()), (1440 - 72 - 16, 900 - 72 - 160));
        assert_eq!(ball_anchor(RETINA, 2.0, 144, Offset::default()), (2880 - 144 - 32, 1800 - 144 - 320));
        // pushed past any edge: pulled back so the whole ball stays visible
        assert_eq!(ball_anchor(SCREEN, 1.0, 72, Offset { dx: 9999.0, dy: 9999.0 }), (1440 - 72, 900 - 72));
        assert_eq!(ball_anchor(SCREEN, 1.0, 72, Offset { dx: -9999.0, dy: -9999.0 }), (0, 25));
    }

    #[test]
    fn where_the_user_leaves_the_ball_is_what_comes_back() {
        let (x, y) = ball_anchor(RETINA, 2.0, 144, Offset::default());
        assert_eq!(ball_offset(RETINA, 2.0, 144, x, y), Offset::default());
        let dragged = ball_offset(RETINA, 2.0, 144, x - 600, y + 100);
        assert_eq!(dragged, Offset { dx: -300.0, dy: 50.0 });
        assert_eq!(ball_anchor(RETINA, 2.0, 144, dragged), (x - 600, y + 100));
        // the same logical spot on the plain screen
        assert_eq!(ball_anchor(SCREEN, 1.0, 72, dragged), (1440 - 72 - 16 - 300, 900 - 72 - 160 + 50));
    }

    #[test]
    fn the_new_commands_belong_to_the_right_windows() {
        for command in ["assistant_unfold", "assistant_prefs"] {
            assert_eq!(permitted(LABEL, command), Some(true), "{command}");
            assert_eq!(permitted("control", command), Some(false), "{command}");
        }
        assert_eq!(permitted("control", "assistant_try"), Some(true));
        assert_eq!(permitted(LABEL, "assistant_try"), Some(false));
        assert_eq!(permitted(LABEL, "assistant_configure"), Some(false));
    }

    #[test]
    fn the_chosen_values_are_the_ones_the_settings_page_offers() {
        assert_eq!(OPEN_MODES, ["voice", "type"]);
        assert_eq!(READ_MODES, ["voice", "always", "never"]);
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

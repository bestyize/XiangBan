//! The floating assistant: a small always-on-top ball that opens into a capsule for a quick spoken or typed
//! question, or a capsule alone that a global shortcut brings up. It is an ordinary local page (`?scene=assistant`)
//! that talks to the same chat, speech and recognition commands as the control window, with a narrower set of
//! permissions. One window serves both shapes; the host decides which one is showing.

mod transition;

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
// Reserve toolbar headroom from the start; hover must never resize the native window.
const MIN_HEIGHT: f64 = 156.0;
// The reply card keeps a fixed body height and scrolls, so the capsule only needs room for its
// tallest state (answer card, question block, attachment tray) — no streaming growth beyond it.
const MAX_HEIGHT: f64 = 600.0;
/// Space kept between the capsule and the bottom of the usable screen (above the Dock or taskbar).
const BOTTOM_GAP: f64 = 56.0;
/// The ball's window, in logical pixels. The ball itself is drawn a little smaller inside it.
const BALL_WINDOW: f64 = 72.0;
/// Extra width the ball's window takes while a one-line notice ("已完成…") shows beside the ball.
const BUBBLE_WIDTH: f64 = 340.0;
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
    dragging: AtomicBool,
    capturing: AtomicBool,
    drag_epoch: AtomicU64,
    pending_height: Mutex<Option<f64>>,
    transition: transition::State,
    /// The ball's window is widened for a notice: `Some(true)` when the notice sits to the ball's left.
    bubble: Mutex<Option<bool>>,
    /// The shortcut is held down. Key repeat sends more presses while it is; they are ignored.
    held: AtomicBool,
    /// Counts presses, so a hold timer from an earlier press does nothing.
    press: AtomicU64,
    /// The capsule was already open when the shortcut went down: a short press closes it.
    open_at_press: AtomicBool,
    /// The shortcut has been held long enough to talk; releasing it sends what was said.
    talking: AtomicBool,
}

/// Holding the shortcut at least this long talks (push to talk); a shorter press opens or closes the capsule.
const HOLD_TO_TALK: Duration = Duration::from_millis(350);

fn shortcut_pressed(app: &tauri::AppHandle) {
    let state = app.state::<AssistantState>();
    if state.held.swap(true, Ordering::SeqCst) { return; }
    if state.capturing.load(Ordering::SeqCst) || !crate::settings(app).assistant_enabled { return; }
    let press = state.press.fetch_add(1, Ordering::SeqCst) + 1;
    let open = mode(app) == Mode::Capsule;
    state.open_at_press.store(open, Ordering::SeqCst);
    // Opening does not wait for the release: the capsule is up by the time a hold turns into talking.
    if !open { toggle(app); }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(HOLD_TO_TALK);
        let state = app.state::<AssistantState>();
        if !state.held.load(Ordering::SeqCst) || state.press.load(Ordering::SeqCst) != press { return; }
        state.talking.store(true, Ordering::SeqCst);
        let main = app.clone();
        let _ = app.run_on_main_thread(move || {
            if mode(&main) != Mode::Capsule { let _ = expand(&main); }
            if let Some(window) = main.get_webview_window(LABEL) { let _ = window.emit("assistant-ptt", "start"); }
        });
    });
}

fn shortcut_released(app: &tauri::AppHandle) {
    let state = app.state::<AssistantState>();
    if !state.held.swap(false, Ordering::SeqCst) { return; }
    if state.talking.swap(false, Ordering::SeqCst) {
        if let Some(window) = app.get_webview_window(LABEL) { let _ = window.emit("assistant-ptt", "end"); }
    } else if state.open_at_press.load(Ordering::SeqCst) {
        toggle(app);
    }
}

const WINDOW_COMMANDS: &[&str] = &[
    "assistant_ready", "assistant_hide", "assistant_fit", "assistant_drag", "assistant_unfold", "assistant_prefs",
    "assistant_open_chat", "assistant_open_settings", "assistant_screenshot", "assistant_set_regions", "assistant_transition_ready",
    "assistant_ball_bubble",
];
const CONTROL_COMMANDS: &[&str] = &["assistant_status", "assistant_configure", "assistant_try"];
/// What the capsule may ask of the host besides its own window commands: ask and answer, speak, listen.
/// Deleting, renaming, compacting or changing the trust mode of conversations stays with the control window.
const SHARED_COMMANDS: &[&str] = &[
    "chat_overview", "chat_load_session", "chat_new_session", "chat_send", "chat_cancel", "chat_context",
    "chat_answer_approval", "chat_answer_question",
    "tts_get_settings", "tts_synthesize", "tts_synthesize_part", "tts_cancel",
    "asr_get_status", "asr_prepare", "asr_transcribe", "asr_report_microphone",
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
        .with_handler(|app, _shortcut, event| match event.state {
            tauri_plugin_global_shortcut::ShortcutState::Pressed => shortcut_pressed(app),
            tauri_plugin_global_shortcut::ShortcutState::Released => shortcut_released(app),
        })
        .build()
}

pub(crate) fn pointer_active(app:&tauri::AppHandle)->bool {
    let state=app.state::<AssistantState>();
    mode(app)!=Mode::Hidden && !state.capturing.load(Ordering::SeqCst)
}

pub fn start(app: &tauri::AppHandle) {
    crate::assistant_hit_test::start(app);
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
    let mut current = state.registered.lock().unwrap_or_else(|e| e.into_inner());
    if enabled && current.as_deref() == Some(shortcut) {
        return Ok(());
    }
    let previous = current.take();
    if let Some(old) = &previous {
        let _ = app.global_shortcut().unregister(old.as_str());
    }
    if !enabled {
        state.error.lock().unwrap_or_else(|e| e.into_inner()).clear();
        return Ok(());
    }
    match app.global_shortcut().register(shortcut) {
        Ok(()) => {
            *current = Some(shortcut.to_string());
            state.error.lock().unwrap_or_else(|e| e.into_inner()).clear();
            Ok(())
        }
        Err(error) => {
            if let Some(old) = previous {
                if app.global_shortcut().register(old.as_str()).is_ok() {
                    *current = Some(old);
                }
            }
            let message = format!("快捷键 {} 无法使用，可能已被其他应用占用（{error}）", shortcut_label(shortcut));
            *state.error.lock().unwrap_or_else(|e| e.into_inner()) = message.clone();
            Err(message)
        }
    }
}

fn status(app: &tauri::AppHandle) -> Value {
    let settings = crate::settings(app);
    let state = app.state::<AssistantState>();
    let registered = state.registered.lock().unwrap_or_else(|e| e.into_inner()).is_some();
    let error = state.error.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
        // Let the capsule receive HTML file drops (including on WebView2).
        .disable_drag_drop_handler()
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

/// A monitor's full rectangle uses physical screen coordinates (not logical
/// points or the work area, which excludes the Dock and menu bar).
fn physical_monitor_area(monitor: &tauri::Monitor) -> Area {
    let position = monitor.position();
    let size = monitor.size();
    Area { x: position.x, y: position.y, w: size.width as i32, h: size.height as i32 }
}

fn contains_physical_point(area: Area, x: f64, y: f64) -> bool {
    x >= f64::from(area.x) && x < f64::from(area.x) + f64::from(area.w)
        && y >= f64::from(area.y) && y < f64::from(area.y) + f64::from(area.h)
}

/// Tauri's monitor_from_point can return None for valid macOS/Retina physical
/// coordinates. Enumerate physical monitor rectangles instead.
fn monitor_at_physical_point(app: &tauri::AppHandle, x: f64, y: f64) -> Result<Option<tauri::Monitor>, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    Ok(monitors.into_iter().find(|monitor| contains_physical_point(physical_monitor_area(monitor), x, y)))
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

/// Expand inward from the ball, clamped to the same screen's usable area.
fn capsule_from_ball(area: Area, scale: f64, x: i32, y: i32, size: i32) -> (i32, i32, i32, i32) {
    let width = ((WIDTH * scale).round() as i32).min(area.w).max(1);
    let height = ((MIN_HEIGHT * scale).round() as i32).min(area.h).max(1);
    let center = x + size / 2;
    let inset = (36.0 * scale).round() as i32;
    let start = if center >= area.x + area.w / 2 { center - width + inset } else { center - inset };
    let top = y + size / 2 + (44.0 * scale).round() as i32 - height;
    (start.clamp(area.x, (area.right() - width).max(area.x)), top.clamp(area.y, (area.bottom() - height).max(area.y)), width, height)
}
fn place_from_ball(app: &tauri::AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let (x, y, size) = ball_target(app)?;
    let monitor = monitor_at_physical_point(app, f64::from(x + size / 2), f64::from(y + size / 2))
        .ok().flatten()
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or("没有找到可用显示器")?;
    let (x, y, width, height) = capsule_from_ball(area_of(&monitor), monitor.scale_factor(), x, y, size);
    window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    window.set_size(PhysicalSize::new(width as u32, height as u32)).map_err(|e| e.to_string())?;
    window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())
}

/// Puts the capsule on the screen the pointer is on, at its default spot or where the user last left it.
fn place(app: &tauri::AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|point| monitor_at_physical_point(app, point.x, point.y).ok().flatten())
        .or_else(|| app.get_webview_window(LABEL).and_then(|window| window.current_monitor().ok().flatten()))
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or("没有找到显示器")?;
    let scale = monitor.scale_factor();
    let width = (WIDTH * scale).round() as i32;
    let height = (MIN_HEIGHT * scale).round() as i32;
    let offset = *app.state::<AssistantState>().moved.lock().unwrap_or_else(|e| e.into_inner());
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
    let monitor = window.current_monitor().map_err(|e| e.to_string())?;
    let available = monitor.as_ref().map(|m| area_of(m).h).unwrap_or(i32::MAX);
    let target = ((height.clamp(MIN_HEIGHT, MAX_HEIGHT) * scale).round() as i32).min(available).max(1);
    if size.height == target as u32 { return Ok(()); }
    let bottom = position.y + size.height as i32;
    let top = match monitor {
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
    *app.state::<AssistantState>().moved.lock().unwrap_or_else(|e| e.into_inner()) = offset;
}

pub(crate) fn remember_position(app: &tauri::AppHandle, window: &WebviewWindow) { remember(app, window); }

fn mode(app: &tauri::AppHandle) -> Mode {
    *app.state::<AssistantState>().mode.lock().unwrap_or_else(|e| e.into_inner())
}

fn set_mode(app: &tauri::AppHandle, mode: Mode) {
    *app.state::<AssistantState>().mode.lock().unwrap_or_else(|e| e.into_inner()) = mode;
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
    let current = mode(app);
    let _ = window.emit("assistant-ball", current == Mode::Ball);
    match current {
        Mode::Capsule => { let _ = window.emit("assistant-open", ()); }
        Mode::Ball => { let _ = window.emit("assistant-close", ()); }
        Mode::Hidden => {}
    }
}

/// Where the ball goes: where it was last seen on a screen that still exists, else the saved offset on the main screen.
fn ball_target(app: &tauri::AppHandle) -> Result<(i32, i32, i32), String> {
    let seen = *app.state::<AssistantState>().ball_at.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((x, y)) = seen {
        if let Ok(Some(monitor)) = monitor_at_physical_point(app, f64::from(x) + 1.0, f64::from(y) + 1.0) {
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

fn show_ball(app: &tauri::AppHandle) -> Result<(), String> { show_ball_surface(app, true) }

fn show_ball_surface(app: &tauri::AppHandle, visible: bool) -> Result<(), String> {
    let window = window(app)?;
    let (x, y, size) = ball_target(app)?;
    set_mode(app, Mode::Ball);
    *app.state::<AssistantState>().bubble.lock().unwrap_or_else(|e| e.into_inner()) = None;
    // Move first so a window arriving on another screen takes that screen's scale before it is sized.
    window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    window.set_size(PhysicalSize::new(size as u32, size as u32)).map_err(|e| e.to_string())?;
    window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    // Shown without taking the focus: the ball appears over whatever the user is doing.
    crate::window_control::apply_saved(app, &window, "topmost")?;
    if visible { window.show().map_err(|e| e.to_string())?; }
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
    // The morph starts from the ball's own square, never from a window widened for a notice.
    if let Some(window) = app.get_webview_window(LABEL) { let _ = set_bubble(app, &window, false); }
    if mode(app) == Mode::Ball && crate::settings(app).assistant_enabled && transition::begin(app, "capsule")? { return Ok(()); }
    expand_surface(app, true)
}
fn expand_surface(app: &tauri::AppHandle, visible: bool) -> Result<(), String> {
    if !crate::settings(app).assistant_enabled {
        return Ok(());
    }
    let window = window(app)?;
    if mode(app) == Mode::Ball {
        // The ball may have been moved a moment ago; keep that spot for when it comes back.
        save_ball(app, &window);
    }
    if mode(app) == Mode::Ball { place_from_ball(app, &window)?; }
    else { place(app, &window)?; }
    set_mode(app, Mode::Capsule);
    crate::window_control::apply_saved(app, &window, "topmost")?;
    if visible {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    announce(app, &window);
    Ok(())
}

/// Puts the capsule away: back to the ball when there is one, otherwise out of sight.
pub fn collapse(app: &tauri::AppHandle) -> Result<(), String> {
    if mode(app) == Mode::Capsule && transition::begin(app, "ball")? { return Ok(()); }
    collapse_surface(app, true)
}
fn collapse_surface(app: &tauri::AppHandle, visible: bool) -> Result<(), String> {
    if mode(app) != Mode::Capsule {
        return Ok(());
    }
    let Some(window) = app.get_webview_window(LABEL) else { return Ok(()) };
    remember(app, &window);
    let _ = window.emit("assistant-close", ());
    let settings = crate::settings(app);
    if settings.assistant_enabled && settings.assistant_ball {
        show_ball_surface(app, visible)
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
    if app.state::<AssistantState>().capturing.load(Ordering::SeqCst) { return; }
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
    let _update = state.update_lock.lock().unwrap_or_else(|e| e.into_inner());
    let mut next = crate::settings(app);
    if (next.assistant_ball_dx - offset.dx).abs() < 0.5 && (next.assistant_ball_dy - offset.dy).abs() < 0.5 {
        return;
    }
    next.assistant_ball_dx = offset.dx;
    next.assistant_ball_dy = offset.dy;
    if crate::persist(app, &next).is_ok() {
        *state.settings.lock().unwrap_or_else(|e| e.into_inner()) = next;
    }
}

/// Notes where the ball is now, for this run and for the next one.
fn save_ball(app: &tauri::AppHandle, window: &WebviewWindow) {
    if mode(app) != Mode::Ball {
        return;
    }
    let (Some((x, y, size)), Ok(Some(monitor)), Ok(scale)) = (ball_rect(app, window), window.current_monitor(), window.scale_factor())
    else {
        return;
    };
    *app.state::<AssistantState>().ball_at.lock().unwrap_or_else(|e| e.into_inner()) = Some((x, y));
    persist_ball(app, ball_offset(area_of(&monitor), scale, size, x, y));
}

/// The ball's own square in physical pixels, also while its window is widened for a notice.
fn ball_rect(app: &tauri::AppHandle, window: &WebviewWindow) -> Option<(i32, i32, i32)> {
    let (position, size) = (window.outer_position().ok()?, window.outer_size().ok()?);
    let ball = size.height as i32;
    let left = *app.state::<AssistantState>().bubble.lock().unwrap_or_else(|e| e.into_inner()) == Some(true);
    Some((if left { position.x + size.width as i32 - ball } else { position.x }, position.y, ball))
}

/// Widens the ball's window toward the middle of the screen for a notice, or narrows it back; the ball stays put.
/// Returns whether the notice sits to the left of the ball, or `None` when nothing is widened.
fn set_bubble(app: &tauri::AppHandle, window: &WebviewWindow, open: bool) -> Result<Option<bool>, String> {
    let state = app.state::<AssistantState>();
    let current = *state.bubble.lock().unwrap_or_else(|e| e.into_inner());
    if mode(app) != Mode::Ball || current.is_some() == open { return Ok(if open { current } else { None }); }
    let (x, y, size) = ball_rect(app, window).ok_or("没有找到悬浮球位置")?;
    if !open {
        *state.bubble.lock().unwrap_or_else(|e| e.into_inner()) = None;
        window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
        window.set_size(PhysicalSize::new(size as u32, size as u32)).map_err(|e| e.to_string())?;
        window.set_position(PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
        return Ok(None);
    }
    let monitor = monitor_at_physical_point(app, f64::from(x + size / 2), f64::from(y + size / 2))?
        .or_else(|| window.current_monitor().ok().flatten())
        .ok_or("没有找到显示器")?;
    let area = area_of(&monitor);
    let left = x + size / 2 >= area.x + area.w / 2;
    let wide = size + (BUBBLE_WIDTH * monitor.scale_factor()).round() as i32;
    let start = if left { x + size - wide } else { x };
    *state.bubble.lock().unwrap_or_else(|e| e.into_inner()) = Some(left);
    window.set_position(PhysicalPosition::new(start, y)).map_err(|e| e.to_string())?;
    window.set_size(PhysicalSize::new(wide as u32, size as u32)).map_err(|e| e.to_string())?;
    window.set_position(PhysicalPosition::new(start, y)).map_err(|e| e.to_string())?;
    Ok(Some(left))
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
    let _update = state.update_lock.lock().unwrap_or_else(|e| e.into_inner());
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
    *state.settings.lock().unwrap_or_else(|e| e.into_inner()) = next;
    if reset {
        *app.state::<AssistantState>().ball_at.lock().unwrap_or_else(|e| e.into_inner()) = None;
        *app.state::<AssistantState>().moved.lock().unwrap_or_else(|e| e.into_inner()) = Offset::default();
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
    json!({ "open": settings.assistant_open, "read": settings.assistant_read, "appearance": settings.appearance, "accent": settings.accent })
}

/// The ball was clicked.
#[tauri::command]
pub fn assistant_transition_ready(window: WebviewWindow, id: u64, step: String) -> Result<(), String> {
    transition::ready(&window, id, &step)
}

/// The page shows (or clears) a one-line notice beside the ball.
#[tauri::command]
pub fn assistant_ball_bubble(window: WebviewWindow, open: bool) -> Result<Option<bool>, String> {
    if window.label() != LABEL { return Err("只有超级小伴可以显示提示".into()); }
    if window.app_handle().state::<AssistantState>().dragging.load(Ordering::SeqCst) { return Ok(None); }
    set_bubble(window.app_handle(), &window, open)
}

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
    if transition::active(window.app_handle()) && !transition::fitting_allowed(window.app_handle()) { return Ok(()); }
    if !height.is_finite() {
        return Err("高度不对".into());
    }
    // The page asks to grow while the user may already have put it away, or while the ball is showing.
    if mode(window.app_handle()) != Mode::Capsule || (!window.is_visible().unwrap_or(false) && !transition::fitting_allowed(window.app_handle())) {
        return Ok(());
    }
    let state=window.app_handle().state::<AssistantState>();
    if state.dragging.load(Ordering::SeqCst){*state.pending_height.lock().unwrap_or_else(|e| e.into_inner())=Some(height);return Ok(());}
    resize(&window, height)
}

/// Native dragging owns the pointer until release, so WebView pointerup is not reliable.
/// Defer streamed-response resizing until the native mouse button is released.
#[tauri::command]
pub fn assistant_drag(window: WebviewWindow) -> Result<(), String> {
    if window.label() != LABEL { return Err("只有超级小伴的窗口可以拖动".into()); }
    let state=window.app_handle().state::<AssistantState>();
    if state.dragging.swap(true,Ordering::SeqCst){return Ok(());}
    let epoch=state.drag_epoch.fetch_add(1,Ordering::SeqCst)+1;
    let target=window.clone();
    std::thread::spawn(move||{
        let app=target.app_handle();let state=app.state::<AssistantState>();
        loop{
            std::thread::sleep(Duration::from_millis(16));
            if state.drag_epoch.load(Ordering::SeqCst)!=epoch{return;}
            #[cfg(target_os="macos")] let down=crate::macos::left_button_down();
            #[cfg(windows)] let down=crate::windows::left_button_down();
            #[cfg(not(any(windows,target_os="macos")))] let down=false;
            if !down||!state.dragging.load(Ordering::SeqCst)||mode(app)==Mode::Hidden{break;}
        }
        state.dragging.store(false,Ordering::SeqCst);
        match mode(app){Mode::Capsule=>remember(app,&target),Mode::Ball=>save_ball(app,&target),_=>{}}
        let pending=state.pending_height.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(height)=pending{if mode(app)==Mode::Capsule{let _=resize(&target,height);}}
        let _=target.eval("window.dispatchEvent(new Event('assistant-drag-end'));".to_string());
    });
    let result=window.start_dragging().map_err(|e|e.to_string());
    if result.is_err(){state.dragging.store(false,Ordering::SeqCst);}
    result
}

/// Capture is user-initiated, preserves the capsule's draft and never sends a message by itself.
#[tauri::command]
pub async fn assistant_screenshot(window: WebviewWindow) -> Result<Option<String>, String> {
    if window.label() != LABEL { return Err("只有超级小伴可以发起截图".into()); }
    let app = window.app_handle().clone();
    if mode(&app) != Mode::Capsule || !window.is_visible().unwrap_or(false) { return Err("请先打开超级小伴再截图".into()); }
    let state = app.state::<AssistantState>();
    if state.capturing.swap(true, Ordering::SeqCst) { return Err("正在截图，请稍候。".into()); }
    let result = async {
        #[cfg(target_os = "macos")]
        {
            // TCC's user interaction belongs on the app's main thread, while its window
            // is still visible. A cached preflight result must not veto the real capture.
            let permission_app = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let (tx, rx) = std::sync::mpsc::channel();
                permission_app.run_on_main_thread(move || {
                    crate::assistant_capture::request_macos_permission();
                    let _ = tx.send(());
                }).map_err(|e| e.to_string())?;
                rx.recv_timeout(Duration::from_secs(120)).map_err(|_| "屏幕录制授权未完成，请重试。".to_string())
            }).await.map_err(|e| e.to_string())??;
        }
        window.hide().map_err(|e| e.to_string())?;
        let capture_app = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            std::thread::sleep(Duration::from_millis(180));
            let frames = crate::assistant_capture::capture(&capture_app)?;
            crate::capture_editor::edit(&capture_app, frames)
        }).await.map_err(|e| format!("截图失败：{e}")).and_then(|result| result)
    }.await;
    // No assistant-open event: that would reset the draft and turn the microphone back on.
    let restore = if mode(&app) == Mode::Capsule && crate::settings(&app).assistant_enabled {
        window.show().and_then(|_| window.set_focus()).map_err(|e| e.to_string())
    } else { Ok(()) };
    state.capturing.store(false, Ordering::SeqCst);
    restore?;
    result
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
    fn only_the_assistant_can_request_a_screen_capture() {
        assert_eq!(permitted(LABEL, "assistant_screenshot"), Some(true));
        for label in ["control", "pet", "widget-example", "content"] { assert_eq!(permitted(label, "assistant_screenshot"), Some(false)); }
    }

    #[test]
    fn physical_display_lookup_handles_retina_and_multiple_monitors() {
        // Regression: Tauri's monitor_from_point returned None on macOS for
        // physical coordinates on a Retina external monitor.
        let external = Area { x: 0, y: 0, w: 3840, h: 2160 };
        let laptop = Area { x: 350, y: 2160, w: 3024, h: 1964 };
        let left = Area { x: -1920, y: 0, w: 1920, h: 1080 };
        assert!(contains_physical_point(external, 1512.0, 534.0));
        assert!(contains_physical_point(external, 3839.0, 2159.0));
        assert!(!contains_physical_point(external, 3840.0, 10.0));
        assert!(contains_physical_point(laptop, 500.0, 2774.0));
        assert!(!contains_physical_point(external, 500.0, 2774.0));
        assert!(contains_physical_point(left, -1.0, 300.0));
        assert!(!contains_physical_point(left, 0.0, 300.0));
    }

    #[test]
    fn expanding_from_any_edge_stays_on_the_balls_screen() {
        for area in [SCREEN, RETINA, Area { x: -1920, y: -1080, w: 1920, h: 1040 }, Area { x: 0, y: 0, w: 320, h: 140 }] {
            for scale in [1.0, 2.0] {
                let size = (BALL_WINDOW * scale) as i32;
                for bx in [area.x, area.right() - size] {
                    for by in [area.y, area.bottom() - size] {
                        let (x, y, width, height) = capsule_from_ball(area, scale, bx, by, size);
                        assert!(x >= area.x && y >= area.y);
                        assert!(x + width <= area.right() && y + height <= area.bottom());
                    }
                }
            }
        }
    }

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


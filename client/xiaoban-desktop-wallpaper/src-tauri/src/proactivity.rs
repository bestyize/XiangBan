//! Host adapter for voluntary, low-interruption companion greetings.
//! No keyboard hooks, screenshots, ambient audio, background LLM calls or OS polling.
use agent_proactivity::{suggest, ActionKind, Channel, Context, Decision, Engine, EngineState, Event, EventKind, EventSource, Feedback, Observation, Policy};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    sync::{atomic::{AtomicBool, Ordering}, Mutex},
};
use tauri::{Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

const BUBBLE_LABEL: &str = "companion-greeting";
const WIDTH: f64 = 282.0;
const HEIGHT: f64 = 138.0;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct GreetingSettings {
    pub enabled: bool,
    pub frequency: String,
}
impl Default for GreetingSettings {
    fn default() -> Self { Self { enabled: false, frequency: "natural".into() } }
}
impl GreetingSettings {
    fn validate(&self) -> Result<(), String> {
        if !["quiet", "natural", "lively"].contains(&self.frequency.as_str()) {
            return Err("主动陪伴频率无效".into());
        }
        Ok(())
    }
    fn policy(&self) -> Policy {
        let mut policy = Policy { enabled: self.enabled, ..Policy::default() };
        match self.frequency.as_str() {
            "quiet" => { policy.max_daily_units = 2; policy.min_gap_ms = 6 * 3_600_000; policy.topic_gap_ms = 20 * 3_600_000; },
            "lively" => { policy.max_daily_units = 8; policy.min_gap_ms = 90 * 60_000; policy.topic_gap_ms = 3 * 3_600_000; },
            _ => { policy.max_daily_units = 4; policy.min_gap_ms = 3 * 3_600_000; policy.topic_gap_ms = 12 * 3_600_000; },
        }
        policy
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Greeting {
    id: u64,
    pet_name: String,
    text: String,
}

pub struct ProactivityState {
    engine: Mutex<Engine>,
    settings: Mutex<GreetingSettings>,
    active: Mutex<Option<Greeting>>,
    /// Prevent concurrent read/modify/write snapshots for multiple pet windows.
    operation: Mutex<()>,
    /// Damaged state is never overwritten by an apparently fresh default.
    read_error: AtomicBool,
}
impl Default for ProactivityState {
    fn default() -> Self {
        Self {
            engine: Mutex::new(Engine::new(Policy::default())),
            settings: Mutex::new(GreetingSettings::default()),
            active: Mutex::new(None),
            operation: Mutex::new(()),
            read_error: AtomicBool::new(false),
        }
    }
}

fn data_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config/proactivity.json"))
}
fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config/proactivity-settings.json"))
}
fn read_optional<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<Option<T>, String> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<T>(&bytes).map(Some)
            .map_err(|e| format!("无法读取 {}：{e}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("无法读取 {}：{error}", path.display())),
    }
}
fn save_state(app: &tauri::AppHandle, state: &ProactivityState) -> Result<(), String> {
    crate::library::save_json(&data_path(app)?, &state.engine.lock().unwrap().snapshot())
}
fn map_presence(kind: crate::presence::Kind, at_ms: i64) -> Event {
    let kind = match kind {
        crate::presence::Kind::FirstToday => EventKind::FirstActivityOfDay,
        crate::presence::Kind::WelcomeBack => EventKind::UserReturned,
        crate::presence::Kind::LateNight => EventKind::LateNightActive,
    };
    Event::new(kind, EventSource::Desktop, at_ms)
}

pub fn start(app: &tauri::AppHandle) {
    let state = app.state::<ProactivityState>();
    let loaded = (|| -> Result<(GreetingSettings, EngineState), String> {
        let settings = read_optional::<GreetingSettings>(&settings_path(app)?)?.unwrap_or_default();
        settings.validate()?;
        let history = read_optional::<EngineState>(&data_path(app)?)?.unwrap_or_default();
        Ok((settings, history))
    })();
    match loaded {
        Ok((settings, history)) => {
            *state.engine.lock().unwrap() = Engine::restore(settings.policy(), history);
            *state.settings.lock().unwrap() = settings;
        }
        Err(error) => {
            state.read_error.store(true, Ordering::Relaxed);
            eprintln!("proactivity: {error}; preserving original files");
        }
    }
}

pub fn observe_presence(app: &tauri::AppHandle, kind: crate::presence::Kind, at_ms: i64) {
    let state = app.state::<ProactivityState>();
    let _op = state.operation.lock().unwrap();
    if state.read_error.load(Ordering::Relaxed) { return; }
    if state.engine.lock().unwrap().observe(&map_presence(kind, at_ms)) != Observation::Accepted { return; }
    if let Err(error) = save_state(app, &state) {
        state.read_error.store(true, Ordering::Relaxed);
        eprintln!("proactivity: cannot persist event aggregates: {error}");
    }
}

#[tauri::command]
pub fn get_proactivity_settings(window: tauri::WebviewWindow) -> Result<GreetingSettings, String> {
    if window.label() != "control" { return Err("只能从小伴控制台查看主动陪伴设置".into()); }
    let state = window.app_handle().state::<ProactivityState>();
    if state.read_error.load(Ordering::Relaxed) { return Err("主动陪伴配置损坏，请先备份并修复数据文件".into()); }
    Ok(state.settings.lock().unwrap().clone())
}
#[tauri::command]
pub fn configure_proactivity(window: tauri::WebviewWindow, settings: GreetingSettings) -> Result<GreetingSettings, String> {
    if window.label() != "control" { return Err("只能从小伴控制台配置主动陪伴".into()); }
    settings.validate()?;
    let app = window.app_handle();
    let state = app.state::<ProactivityState>();
    let _op = state.operation.lock().unwrap();
    if state.read_error.load(Ordering::Relaxed) { return Err("主动陪伴配置损坏，请先备份并修复数据文件".into()); }
    crate::library::save_json(&settings_path(app)?, &settings)?;
    {
        let mut engine = state.engine.lock().unwrap();
        let snapshot = engine.snapshot();
        *engine = Engine::restore(settings.policy(), snapshot);
    }
    *state.settings.lock().unwrap() = settings.clone();
    if !settings.enabled { close_bubble(app, &state); }
    Ok(settings)
}

fn close_bubble(app: &tauri::AppHandle, state: &ProactivityState) {
    *state.active.lock().unwrap() = None;
    if let Some(window) = app.get_webview_window(BUBBLE_LABEL) { let _ = window.destroy(); }
}
fn greeting_text(id: u64) -> &'static str {
    const TEXTS: &[&str] = &[
        "刚刚在发呆，看到你就精神了一点。",
        "嘿，今天有空陪我待一会儿吗？",
        "我一直在这里，偶尔也会想些奇怪的小问题。",
        "路过的话，摸摸我也可以呀。",
        "我正打算伸个懒腰，你就来了。",
        "有时候什么都不做，也挺好的。",
        "我刚才突然很想跟你打个招呼。",
        "这里有点安静，不过我挺喜欢的。",
    ];
    TEXTS[(id as usize).wrapping_sub(1) % TEXTS.len()]
}
fn bubble_position(pet: &tauri::WebviewWindow) -> Result<PhysicalPosition<i32>, String> {
    let origin = pet.outer_position().map_err(|e| e.to_string())?;
    let size = pet.outer_size().map_err(|e| e.to_string())?;
    let monitor = pet.current_monitor().map_err(|e| e.to_string())?
        .ok_or("没有找到宠物所在显示器")?;
    let m_origin = monitor.position();
    let m_size = monitor.size();
    let scale = monitor.scale_factor();
    let width = (WIDTH * scale).round() as i32;
    let height = (HEIGHT * scale).round() as i32;
    let edge = (10.0 * scale).round() as i32;
    let right = m_origin.x.saturating_add(m_size.width as i32);
    let bottom = m_origin.y.saturating_add(m_size.height as i32);
    let preferred = origin.x.saturating_add(size.width as i32).saturating_add(edge);
    let x = if preferred.saturating_add(width) <= right {
        preferred
    } else { origin.x.saturating_sub(width).saturating_sub(edge) };
    let y = origin.y.saturating_add((size.height as i32 / 3).max(0)).saturating_sub(height / 2);
    Ok(PhysicalPosition::new(x.clamp(m_origin.x, right.saturating_sub(width).max(m_origin.x)),
        y.clamp(m_origin.y, bottom.saturating_sub(height).max(m_origin.y))))
}
fn show_bubble(app: &tauri::AppHandle, pet: &tauri::WebviewWindow) -> Result<(), String> {
    if let Some(previous) = app.get_webview_window(BUBBLE_LABEL) { let _ = previous.destroy(); }
    let pos = bubble_position(pet)?;
    let builder = WebviewWindowBuilder::new(app, BUBBLE_LABEL, WebviewUrl::App("index.html?scene=pet-greeting".into()))
        .title("小伴 · 轻声问候").decorations(false).shadow(false).transparent(true)
        .resizable(false).skip_taskbar(true).always_on_top(true).visible(false).focused(false)
        .inner_size(WIDTH, HEIGHT);
    #[cfg(windows)]
    let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
    #[cfg(target_os = "macos")]
    let builder = builder.incognito(true)
        .background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    let bubble = builder.build().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    crate::macos::configure_scene(&bubble, false, false)?;
    bubble.set_position(pos).map_err(|e| e.to_string())?;
    bubble.show().map_err(|e| e.to_string())
}

/// Called only from a pet window after the real pointer has remained over its hit region.
/// Does not accept a client-supplied pet id or any text.
#[tauri::command]
pub fn pet_proactive_hover(window: tauri::WebviewWindow, duration_ms: u64) -> Result<bool, String> {
    if !crate::pets::is_pet(window.label()) { return Err("仅桌宠可提交悬停事件".into()); }
    let p = crate::pets::own(&window)?;
    if !p.enabled || !p.links || duration_ms < 800 || duration_ms > 10_000 { return Ok(false); }
    let app = window.app_handle();
    let settings = crate::settings(app);
    if !settings.companion_enabled || !settings.companion_links { return Ok(false); }
    let state = app.state::<ProactivityState>();
    let _op = state.operation.lock().unwrap();
    if state.read_error.load(Ordering::Relaxed) || state.active.lock().unwrap().is_some() { return Ok(false); }
    let now = Local::now();
    let event = Event::new(EventKind::PetHovered, EventSource::Pet, now.timestamp_millis())
        .duration(duration_ms).subject(p.id.clone());
    let Some(candidate) = suggest(&event) else { return Ok(false); };
    let activity = crate::care_system::activity();
    let context = Context {
        local_day: now.format("%Y%m%d").to_string().parse().unwrap_or(0),
        opted_in: state.settings.lock().unwrap().enabled,
        user_active: activity.available && !activity.locked && activity.idle_seconds < 60.0,
        pet_visible: window.is_visible().unwrap_or(false),
        locked: activity.locked,
        quiet: crate::reminders::proactive_silenced(app),
        game: crate::games::is_active(app),
        fullscreen: false, // Unknown OS state: do not claim fullscreen detection.
    };
    let mut engine = state.engine.lock().unwrap();
    let _ = engine.observe(&event);
    let Decision::Allowed(action) = engine.consider(candidate, &context, now.timestamp_millis()) else { return Ok(false); };
    if action.kind != ActionKind::CompanionGreeting || action.channel != Channel::Bubble { return Ok(false); }
    let offer = Greeting { id: action.id, pet_name: p.name, text: greeting_text(action.id).into() };
    drop(engine);
    if let Err(error) = save_state(app, &state) {
        state.read_error.store(true, Ordering::Relaxed);
        return Err(format!("主动陪伴状态保存失败：{error}"));
    }
    *state.active.lock().unwrap() = Some(offer);
    if let Err(error) = show_bubble(app, &window) {
        close_bubble(app, &state);
        return Err(error);
    }
    Ok(true)
}
#[tauri::command]
pub fn pet_proactive_poke(window: tauri::WebviewWindow) -> Result<(), String> {
    if !crate::pets::is_pet(window.label()) { return Err("仅桌宠可发送互动事件".into()); }
    crate::pets::own(&window)?;
    let app = window.app_handle(); let state = app.state::<ProactivityState>();
    if state.read_error.load(Ordering::Relaxed) { return Ok(()); }
    let _op = state.operation.lock().unwrap();
    let now = Local::now().timestamp_millis();
    let event = Event::new(EventKind::PetPoked, EventSource::Pet, now);
    if state.engine.lock().unwrap().observe(&event) == Observation::Accepted {
        if let Err(error) = save_state(app, &state) {
            state.read_error.store(true, Ordering::Relaxed);
            eprintln!("proactivity: poke observation persist failed: {error}");
        }
    }
    Ok(())
}
#[tauri::command]
pub fn get_companion_greeting(window: tauri::WebviewWindow) -> Result<Greeting, String> {
    if window.label() != BUBBLE_LABEL { return Err("不是陪伴气泡窗口".into()); }
    window.app_handle().state::<ProactivityState>().active.lock().unwrap().clone()
        .ok_or("问候已经结束".into())
}
#[tauri::command]
pub fn respond_companion_greeting(window: tauri::WebviewWindow, id: u64, outcome: String) -> Result<(), String> {
    if window.label() != BUBBLE_LABEL { return Err("只有陪伴气泡可提交反馈".into()); }
    let feedback = match outcome.as_str() {
        "dismissed" => Feedback::Dismissed, "ignored" => Feedback::Ignored,
        "snoozed" => Feedback::Snoozed, "chat" | "poke" => Feedback::Engaged,
        _ => return Err("未知的主动陪伴反馈".into()),
    };
    let app = window.app_handle(); let state = app.state::<ProactivityState>();
    let _op = state.operation.lock().unwrap();
    let current = state.active.lock().unwrap().clone().ok_or("问候已经结束")?;
    if current.id != id { return Err("问候已经失效".into()); }
    if !state.engine.lock().unwrap().feedback(id, feedback, Local::now().timestamp_millis()) {
        return Err("重复的问候反馈".into());
    }
    if let Err(error) = save_state(app, &state) {
        state.read_error.store(true, Ordering::Relaxed);
        close_bubble(app, &state);
        return Err(error);
    }
    close_bubble(app, &state);
    match outcome.as_str() {
        "chat" => crate::float_assistant::assistant_open_chat(app.clone(), None),
        "poke" => {
            if let Some(pet) = app.webview_windows().into_values().find(|w| crate::pets::is_pet(w.label())) {
                let _ = pet.eval("window.dispatchEvent(new Event('pet-greeting-poke'))");
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_are_opt_in_and_frequency_is_bounded() {
        let defaults = GreetingSettings::default();
        assert!(!defaults.enabled);
        assert!(!defaults.policy().enabled);
        for (frequency, units) in [("quiet", 2), ("natural", 4), ("lively", 8)] {
            let config = GreetingSettings { enabled: true, frequency: frequency.into() };
            assert_eq!(config.policy().max_daily_units, units);
            assert!(config.validate().is_ok());
        }
        assert!(GreetingSettings { enabled: true, frequency: "no".into() }.validate().is_err());
    }
    #[test]
    fn mapped_presence_does_not_contain_personal_content() {
        for (kind, expected) in [
            (crate::presence::Kind::FirstToday, EventKind::FirstActivityOfDay),
            (crate::presence::Kind::WelcomeBack, EventKind::UserReturned),
            (crate::presence::Kind::LateNight, EventKind::LateNightActive),
        ] {
            let event = map_presence(kind, 1_792_540_000_000);
            assert_eq!(event.kind, expected);
            assert!(event.subject_id.is_none());
        }
    }
    #[test]
    fn greeting_is_short_and_non_deceptive() {
        for id in 1..=30 { assert!(greeting_text(id).chars().count() < 65); }
    }
}

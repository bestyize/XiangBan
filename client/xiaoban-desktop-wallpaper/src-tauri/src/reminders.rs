//! Care scheduling runs in the native host, independently of WebView visibility.
use crate::{care_system, care_weather::{self, City, Source, Weather}};
use chrono::{DateTime, Datelike, Duration as CalendarDuration, Local, NaiveDateTime, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashSet}, sync::{Mutex, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const VALID_SECONDS: i64 = 30 * 60;
const BUBBLE_SECONDS: i64 = 10;
const SNOOZE_SECONDS: i64 = 10 * 60;
/// A scheduled agent task's result may wait this long for the user (away, quiet hours) before it is dropped; the
/// whole answer stays in its conversation either way.
const AGENT_VALID_SECONDS: i64 = 12 * 3600;
/// A one-time reminder is something the user must not miss: it is still delivered this long after its time (the
/// screen was locked, the app was closed), and its bubble stays up until it is closed or snoozed.
const ONCE_VALID_SECONDS: i64 = 12 * 3600;
const STICKY_SECONDS: i64 = 12 * 3600;
/// What one agent task may ask of the assistant.
const AGENT_PROMPT_CHARS: usize = 1000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CareConfig {
    pub enabled: bool,
    pub sound: bool,
    /// One-time reminders sound even when `sound` is off: they are the ones that must not be missed.
    pub once_sound: bool,
    pub voice: bool,
    /// One-time reminders are read aloud even when `voice` is off.
    pub once_voice: bool,
    pub quiet_enabled: bool,
    pub quiet_start: String,
    pub quiet_end: String,
    pub weather_source: Source,
}
impl Default for CareConfig {
    fn default() -> Self { Self { enabled: true, sound: false, once_sound: true, voice: false, once_voice: true, quiet_enabled: false, quiet_start: "22:00".into(), quiet_end: "08:00".into(), weather_source: Source::default() } }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub revision: u64,
    pub enabled: bool,
    pub kind: String,
    pub repeat: String,
    pub time: String,
    pub once_at: String,
    /// Monday=1, Sunday=7; never infer a user's working week.
    pub weekdays: Vec<u32>,
    pub interval_minutes: u32,
    pub text: String,
    pub city: Option<City>,
    pub not_before: i64,
}
impl Default for Rule {
    fn default() -> Self { Self { id: String::new(), revision: 0, enabled: true, kind: "custom".into(), repeat: "daily".into(), time: "15:00".into(), once_at: String::new(), weekdays: vec![], interval_minutes: 60, text: String::new(), city: None, not_before: 0 } }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pending {
    pub key: String,
    pub rule_id: String,
    pub revision: u64,
    pub kind: String,
    pub text: String,
    pub due_at: i64,
    pub available_at: i64,
    pub expires_at: i64,
    pub preview: bool,
    pub weather_ready: bool,
    #[serde(default)]
    pub weather_at: Option<i64>,
    /// A one-time reminder: its bubble does not go away by itself.
    #[serde(default)]
    pub sticky: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Store {
    config: CareConfig,
    rules: Vec<Rule>,
    pending: Vec<Pending>,
    seen: BTreeMap<String, i64>,
    rest_seconds: BTreeMap<String, f64>,
    weather: Option<Weather>,
    weather_error: Option<String>,
    error: Option<String>,
    /// By rule id.
    agent_runs: BTreeMap<String, AgentRun>,
}
/// The last run of a scheduled agent task, shown on the care page.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentRun {
    pub at: i64,
    pub ok: bool,
    pub summary: String,
    pub session: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Runs so far, this one included.
    pub count: u64,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    pub id: String,
    pub items: Vec<Pending>,
    pub sound: bool,
    pub voice: bool,
    pub display_seconds: i64,
}
#[derive(Clone, Default)]
struct Runtime {
    active: Option<Delivery>,
    shown_at: Option<i64>,
    ready: bool,
    focus: bool,
    weather_attempt: i64,
    city_attempt: i64,
    locate_attempt: i64,
    read_error: bool,
    /// Scheduled agent tasks running now, by pending key, so a tick never starts one twice.
    agent_running: HashSet<String>,
}
#[derive(Default)]
pub struct ReminderState {
    store: Mutex<Store>,
    runtime: Mutex<Runtime>,
    game: AtomicBool,
    started: AtomicBool,
    smoke_running: AtomicBool,
    presence: Mutex<crate::presence::Presence>,
}

fn control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" { return Err("只能从小伴设置修改关怀提醒".into()); }
    Ok(())
}
fn bubble(window: &tauri::WebviewWindow) -> Result<(), String> {
    if !["care", "control"].contains(&window.label()) { return Err("当前窗口没有关怀提示操作权限".into()); }
    Ok(())
}
fn save(app: &tauri::AppHandle, store: &Store) -> Result<(), String> {
    if app.state::<ReminderState>().runtime.lock().unwrap_or_else(|e| e.into_inner()).read_error {
        return Err("原关怀配置无法读取，已保留原文件；请先在用户数据目录备份并修复 care-reminders.json".into());
    }
    crate::library::save_json(&crate::storage::root(app)?.join("config/care-reminders.json"), store)
}
fn time(value: &str) -> Result<NaiveTime, String> {
    if value.len() != 5 { return Err("时间应为 HH:mm".into()); }
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|_| "时间应为 HH:mm".into())
}
fn validate(rule: &Rule) -> Result<(), String> {
    if rule.id.len() > 100 || !rule.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') { return Err("提醒标识无效".into()); }
    if !["custom", "off-work", "rest", "weather", "agent"].contains(&rule.kind.as_str()) { return Err("提醒类型无效".into()); }
    if rule.kind == "agent" {
        if rule.text.chars().count() > AGENT_PROMPT_CHARS { return Err(format!("任务说明最多 {AGENT_PROMPT_CHARS} 个字")); }
        if rule.text.trim().is_empty() { return Err("请写下要小伴定时做的事".into()); }
    } else if rule.text.chars().count() > 240 { return Err("提醒文案最多 240 个字".into()); }
    if rule.kind != "weather" && rule.text.trim().is_empty() { return Err("请填写提醒文案".into()); }
    if rule.kind == "rest" {
        if !(5..=480).contains(&rule.interval_minutes) { return Err("休息间隔应为 5–480 分钟".into()); }
    } else {
        time(&rule.time)?;
        match rule.repeat.as_str() {
            "once" => { NaiveDateTime::parse_from_str(&rule.once_at, "%Y-%m-%dT%H:%M").map_err(|_| "请选择一次提醒的日期和时间")?; },
            "daily" => {},
            "weekdays" if !rule.weekdays.is_empty() && rule.weekdays.iter().all(|day| (1..=7).contains(day)) => {},
            _ => return Err("请选择每天、一次或有效的星期".into()),
        }
    }
    if rule.kind == "weather" { rule.city.as_ref().ok_or("天气提醒需要手动选择城市")?.validate()?; }
    Ok(())
}
fn quiet(config: &CareConfig, now: NaiveTime) -> bool {
    if !config.quiet_enabled { return false; }
    let (Ok(start), Ok(end)) = (time(&config.quiet_start), time(&config.quiet_end)) else { return false; };
    if start == end { return true; }
    if start < end { now >= start && now < end } else { now >= start || now < end }
}
/// Whether a bubble is read aloud: when voice is on, or when it holds a one-time reminder and those are set to.
fn speaks(config: &CareConfig, sticky: bool) -> bool { config.voice || (sticky && config.once_voice) }
/// What is said for a bubble: the reminders themselves, led in by a word so a bare "买花" is understood as a reminder.
fn spoken(items: &[Pending]) -> String {
    let texts: Vec<&str> = items.iter().map(|item| item.text.trim()).filter(|text| !text.is_empty()).collect();
    let body = if texts.len() > 2 { format!("{}。还有 {} 条到期提醒。", texts[..2].join("。"), texts.len() - 2) } else { texts.join("。") };
    let own_words = items.iter().all(|item| matches!(item.kind.as_str(), "custom" | "off-work" | "rest"));
    if body.is_empty() || !own_words { body } else { format!("提醒：{body}") }
}
/// Whether a bubble beeps: when sounds are on, or when it holds a one-time reminder and those are set to.
fn plays_sound(config: &CareConfig, sticky: bool) -> bool { config.sound || (sticky && config.once_sound) }
/// Whether a rule is a one-time reminder, which is kept up until the user answers it.
fn lasts(rule: &Rule) -> bool { rule.repeat == "once" && rule.kind == "custom" }
fn local_due(rule: &Rule, now: DateTime<Local>) -> Vec<(String, i64)> {
    if rule.kind == "rest" { return vec![]; }
    let mut candidates = Vec::new();
    if rule.repeat == "once" {
        if let Ok(value) = NaiveDateTime::parse_from_str(&rule.once_at, "%Y-%m-%dT%H:%M") { candidates.push(value); }
    } else if let Ok(clock) = time(&rule.time) {
        for offset in [0, 1] {
            let date = now.date_naive() - CalendarDuration::days(offset);
            if rule.repeat == "weekdays" && !rule.weekdays.contains(&date.weekday().number_from_monday()) { continue; }
            candidates.push(date.and_time(clock));
        }
    }
    candidates.into_iter().filter_map(|date| {
        // On the repeated DST hour choose the first occurrence. The dedup key
        // deliberately excludes UTC offset, so a clock/timezone rollback cannot replay it.
        let due = Local.from_local_datetime(&date).earliest()?.timestamp();
        let valid = if lasts(rule) { ONCE_VALID_SECONDS } else { VALID_SECONDS };
        if due > now.timestamp() || now.timestamp() - due > valid || due < rule.not_before { return None; }
        Some((format!("{}:{}:{}", rule.id, rule.revision, date.format("%Y-%m-%dT%H:%M")), due))
    }).collect()
}
fn collect(store:&mut Store,now:DateTime<Local>,active_seconds:f64)->bool{collect_with_widgets(store,now,active_seconds,&[])}
fn collect_with_widgets(store: &mut Store, now: DateTime<Local>, active_seconds: f64,widgets:&[Rule]) -> bool {
    let rules:Vec<Rule>=store.rules.iter().chain(widgets).cloned().collect();
    let timestamp = now.timestamp();
    let before = store.pending.len();
    store.pending.retain(|item| item.expires_at >= timestamp && (item.preview || rules.iter().any(|r| r.enabled && r.id == item.rule_id && r.revision == item.revision)));
    for item in &mut store.pending {
        if item.kind == "weather" && item.weather_ready && item.weather_at.is_none_or(|at| timestamp - at > 7200 || at - timestamp > 1800) {
            item.weather_ready = false;
            item.text = store.rules.iter().find(|r| r.id == item.rule_id).map(|r| r.text.clone()).unwrap_or_default();
        }
    }
    let mut changed = before != store.pending.len();
    if !store.config.enabled { return changed; }
    for rule in rules.into_iter().filter(|r| r.enabled) {
        let dues = if rule.kind == "rest" {
            let elapsed = store.rest_seconds.entry(rule.id.clone()).or_default();
            *elapsed += active_seconds;
            if *elapsed >= rule.interval_minutes as f64 * 60.0 {
                *elapsed = 0.0;
                vec![(format!("{}:{}:rest:{}", rule.id, rule.revision, timestamp), timestamp)]
            } else { vec![] }
        } else { local_due(&rule, now) };
        for (key, due) in dues {
            if store.seen.contains_key(&key) { continue; }
            store.seen.insert(key.clone(), timestamp);
            if rule.kind == "rest" && store.pending.iter().any(|item| item.kind == "rest") { changed = true; continue; }
            // Weather and agent items wait for their text (`weather_ready`); an agent's result may wait longer to be seen.
            let expires_at = due + if rule.kind == "agent" { AGENT_VALID_SECONDS } else if lasts(&rule) { ONCE_VALID_SECONDS } else { VALID_SECONDS };
            store.pending.push(Pending { key, rule_id: rule.id.clone(), revision: rule.revision, kind: rule.kind.clone(), text: rule.text.clone(),
                due_at: due, available_at: due, expires_at, preview: false, weather_ready: !matches!(rule.kind.as_str(), "weather" | "agent"), weather_at: None, sticky: lasts(&rule) });
            changed = true;
        }
    }
    // Keep a bounded historical ledger well beyond the catch-up window.
    store.seen.retain(|_, at| timestamp - *at < 90 * 86400 || *at > timestamp);
    while store.seen.len() > 8192 { if let Some(key) = store.seen.iter().min_by_key(|(_, at)| *at).map(|(key, _)| key.clone()) { store.seen.remove(&key); } }
    changed
}
fn status(app: &tauri::AppHandle) -> serde_json::Value {
    let state = app.state::<ReminderState>();
    let store = state.store.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let activity = care_system::activity();
    serde_json::json!({"config":store.config,"rules":store.rules,"pendingCount":store.pending.len(),"weather":store.weather,
        "weatherError":store.weather_error,"error":store.error,"agentRuns":store.agent_runs,"systemNotificationsAvailable":false,"activityAvailable":activity.available})
}
fn publish(app: &tauri::AppHandle) { let _ = app.emit_to("control", "care-settings-changed", status(app)); }

#[tauri::command]
pub fn get_care_settings(window: tauri::WebviewWindow) -> Result<serde_json::Value, String> { control(&window)?; Ok(status(window.app_handle())) }

#[tauri::command]
pub fn configure_care(window: tauri::WebviewWindow, config: CareConfig) -> Result<serde_json::Value, String> {
    control(&window)?; time(&config.quiet_start)?; time(&config.quiet_end)?; config.weather_source.validate()?;
    let app = window.app_handle(); let state = app.state::<ReminderState>();
    let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner()); let mut next = store.clone();
    if next.config.weather_source != config.weather_source {
        next.weather = None; next.weather_error = None;
        next.pending.retain(|p| p.kind != "weather");
    }
    next.config = config;
    if !next.config.enabled { next.pending.clear(); }
    save(app, &next)?; *store = next; drop(store);
    if !state.store.lock().unwrap_or_else(|e| e.into_inner()).config.enabled { close_active(app); }
    publish(app); Ok(status(app))
}

#[tauri::command]
pub fn upsert_care_rule(window: tauri::WebviewWindow, mut rule: Rule) -> Result<serde_json::Value, String> {
    control(&window)?; let app = window.app_handle(); let state = app.state::<ReminderState>();
    let now = Local::now().timestamp();
    let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner()); let mut next = store.clone();
    if rule.id.is_empty() {
        if next.rules.len() >= 50 { return Err("最多保存 50 条提醒".into()); }
        rule.id = format!("care-{}-{}", now, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos());
    }
    let old = next.rules.iter().find(|r| r.id == rule.id);
    rule.revision = old.map_or(1, |r| r.revision.saturating_add(1));
    rule.not_before = now;
    rule.text = rule.text.trim().to_string();
    rule.weekdays.sort_unstable(); rule.weekdays.dedup(); validate(&rule)?;
    if rule.kind == "weather" && rule.enabled && next.config.weather_source.provider == "disabled" { return Err("请先在提醒偏好中开启并保存天气来源".into()); }
    if rule.repeat == "once" && rule.kind != "rest" && rule.enabled {
        let date = NaiveDateTime::parse_from_str(&rule.once_at, "%Y-%m-%dT%H:%M").map_err(|_| "提醒日期无效")?;
        if Local.from_local_datetime(&date).earliest().is_none_or(|d| d.timestamp() <= now) { return Err("一次提醒请选择未来的有效本地时间".into()); }
    }
    next.pending.retain(|p| p.rule_id != rule.id); next.rest_seconds.remove(&rule.id);
    next.rules.retain(|r| r.id != rule.id); next.rules.push(rule.clone());
    save(app, &next)?; *store = next; drop(store);
    cancel_active_rule(app, &rule.id); publish(app); Ok(status(app))
}
#[tauri::command]
pub fn delete_care_rule(window: tauri::WebviewWindow, id: String) -> Result<serde_json::Value, String> {
    control(&window)?; let app = window.app_handle(); let state = app.state::<ReminderState>();
    let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner()); let mut next = store.clone();
    next.rules.retain(|r| r.id != id); next.pending.retain(|p| p.rule_id != id); next.rest_seconds.remove(&id);
    save(app, &next)?; *store = next; drop(store); cancel_active_rule(app, &id); publish(app); Ok(status(app))
}
fn companion_visible(app: &tauri::AppHandle) -> bool {
    let settings = crate::settings(app);
    if !settings.companion_enabled { return false; }
    if settings.mode == "wallpaper" && crate::content::composing(app) {
        return ["media-a", "media-b"].iter().any(|label| app.get_webview_window(label)
            .is_some_and(|window| window.is_visible().unwrap_or(false)));
    }
    app.get_webview_window("pet").is_some_and(|w| w.is_visible().unwrap_or(false))
}
/// Reuse care quiet-hours, game and focus gates for separately opted-in companion greetings.
/// A pending care bubble has priority over optional greetings.
pub fn proactive_silenced(app: &tauri::AppHandle) -> bool {
    let state = app.state::<ReminderState>();
    let is_quiet = quiet(&state.store.lock().unwrap_or_else(|e| e.into_inner()).config, Local::now().time());
    let runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner());
    is_quiet || runtime.focus || runtime.active.is_some() || state.game.load(Ordering::Relaxed)
}
fn blocked(app: &tauri::AppHandle, activity: &care_system::Activity) -> bool {
    let state = app.state::<ReminderState>();
    let store = state.store.lock().unwrap_or_else(|e| e.into_inner());
    !store.config.enabled || quiet(&store.config, Local::now().time()) || activity.locked
        || state.game.load(Ordering::Relaxed) || state.runtime.lock().unwrap_or_else(|e| e.into_inner()).focus
}
fn position(app: &tauri::AppHandle, window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(windows)] let point = care_system::foreground_point();
    #[cfg(not(windows))] let point = app.cursor_position().ok().map(|point| (point.x as i32, point.y as i32));
    let monitor = point.and_then(|(x, y)| app.available_monitors().ok()?.into_iter().find(|monitor| {
        let origin = monitor.position(); let size = monitor.size();
        x >= origin.x && y >= origin.y && (x as i64) < origin.x as i64 + size.width as i64 && (y as i64) < origin.y as i64 + size.height as i64
    })).or_else(|| app.get_webview_window("control").and_then(|w| w.current_monitor().ok().flatten()))
        .or_else(|| app.primary_monitor().ok().flatten()).ok_or("没有可用显示器")?;
    let scale = monitor.scale_factor();
    let origin = monitor.position(); let size = monitor.size();
    let margin = (20.0 * scale).round() as i32;
    let width = (340.0 * scale).round() as i32;
    let x = origin.x + (size.width as i32 - width - margin).max(margin);
    let y = origin.y + margin;
    window.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())
}
fn open_bubble(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("care") {
        position(app, &window)?;
        let state = app.state::<ReminderState>();
        let ready = { let runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()); runtime.ready && runtime.active.is_some() };
        if ready && !blocked(app, &care_system::activity()) { care_system::show_bubble(&window)?; }
        return Ok(());
    }
    let builder = WebviewWindowBuilder::new(app, "care", WebviewUrl::App("index.html?scene=care".into()))
        .title("小伴关怀").decorations(false).shadow(false).resizable(false).skip_taskbar(true)
        .always_on_top(true).transparent(true).visible(false).focused(false).focusable(false).inner_size(340.0, 180.0);
    #[cfg(windows)] let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
    #[cfg(target_os = "macos")] let builder = builder.incognito(true).background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    let window = builder.build().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    crate::macos::configure_scene(&window, false, false)?;
    position(app, &window)
}
fn close_active(app: &tauri::AppHandle) {
    let state = app.state::<ReminderState>();
    let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()); runtime.active = None; runtime.shown_at = None; drop(runtime);
    if let Some(window) = app.get_webview_window("care") { let _ = care_system::hide_bubble(&window); let _ = window.emit("care-delivery", Option::<Delivery>::None); }
}
fn remove_active_rule(runtime: &mut Runtime, id: &str) -> bool {
    let Some(delivery) = runtime.active.as_mut() else { return false; };
    let before = delivery.items.len();
    delivery.items.retain(|item| item.rule_id != id);
    if delivery.items.len() == before { return false; }
    if delivery.items.is_empty() {
        runtime.active = None;
        runtime.shown_at = None;
    }
    true
}
fn cancel_active_rule(app: &tauri::AppHandle, id: &str) {
    let state = app.state::<ReminderState>();
    let next = {
        let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner());
        if !remove_active_rule(&mut runtime, id) { return; }
        runtime.active.clone()
    };
    // Keep the same bubble and display deadline for unrelated reminders. They
    // have already left the pending queue and must not be discarded or replayed.
    // Release the runtime guard before performing native window operations.
    if let Some(window) = app.get_webview_window("care") {
        if next.is_none() { let _ = care_system::hide_bubble(&window); }
        let _ = window.emit("care-delivery", next);
    }
}
fn show(app: &tauri::AppHandle, items: Vec<Pending>) -> Result<(), String> {
    let state = app.state::<ReminderState>();
    let config = state.store.lock().unwrap_or_else(|e| e.into_inner()).config.clone();
    let sticky = items.iter().any(|item| item.sticky && !item.preview);
    let delivery = Delivery { id: format!("bubble-{}", Local::now().timestamp_millis()), items, sound: plays_sound(&config, sticky), voice: speaks(&config, sticky),
        display_seconds: if sticky { STICKY_SECONDS } else if config.voice { 30 } else { BUBBLE_SECONDS } };
    { let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()); runtime.active = Some(delivery.clone()); runtime.shown_at = if runtime.ready { Some(Local::now().timestamp()) } else { None }; }
    if let Err(error) = open_bubble(app) { close_active(app); return Err(error); }
    if blocked(app, &care_system::activity()) { defer_active(app); return Ok(()); }
    if state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.as_ref().is_none_or(|d| d.id != delivery.id) { return Ok(()); }
    if let Some(window) = app.get_webview_window("care") { let _ = window.emit("care-delivery", &delivery); }
    if state.runtime.lock().unwrap_or_else(|e| e.into_inner()).ready && delivery.sound { care_system::play_sound(); }
    // Events into low-permission media scenes are deliberately browser-local,
    // produced only by native trusted code, and carry no caller-provided JS.
    for label in ["pet", "wallpaper", "media-a", "media-b"] {
        if let Some(window) = app.get_webview_window(label) { let _ = window.eval("window.dispatchEvent(new CustomEvent('care-expression',{detail:'happy'}))"); }
    }
    Ok(())
}
#[tauri::command]
pub fn care_ready(window: tauri::WebviewWindow) -> Result<Option<Delivery>, String> {
    if window.label() != "care" { return Err("只有关怀气泡可确认就绪".into()); }
    let app = window.app_handle(); let state = app.state::<ReminderState>();
    let delivery = state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.clone();
    if delivery.is_some() && !blocked(app, &care_system::activity()) {
        care_system::show_bubble(&window)?;
        if delivery.as_ref().is_some_and(|d| d.sound) { care_system::play_sound(); }
    }
    // Readiness is published only after the native show call completes. The
    // smoke observer and scheduler must not see a ready-but-still-hidden window.
    { let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()); runtime.ready = true;
      if runtime.active.is_some() { runtime.shown_at = Some(Local::now().timestamp()); } }
    Ok(delivery)
}
#[tauri::command]
pub fn get_care_delivery(window: tauri::WebviewWindow) -> Result<Option<Delivery>, String> {
    bubble(&window)?; Ok(window.app_handle().state::<ReminderState>().runtime.lock().unwrap_or_else(|e| e.into_inner()).active.clone())
}
#[tauri::command]
pub async fn care_synthesize(window: tauri::WebviewWindow, id: String) -> Result<tauri::ipc::Response, String> {
    if window.label() != "care" { return Err("只有关怀气泡可以播报提醒".into()); }
    let app = window.app_handle().clone();
    let delivery = app.state::<ReminderState>().runtime.lock().unwrap_or_else(|e| e.into_inner()).active.clone()
        .filter(|delivery| delivery.id == id && delivery.voice).ok_or("提醒已关闭或未开启语音播报")?;
    if blocked(&app, &care_system::activity()) { return Err("当前不适合播报提醒".into()); }
    let text = spoken(&delivery.items);
    if text.is_empty() { return Err("提醒没有可播报的文字".into()); }
    let audio = crate::tts::tts_synthesize(app.clone(), text, None).await?;
    if app.state::<ReminderState>().runtime.lock().unwrap_or_else(|e| e.into_inner()).active.as_ref().is_none_or(|active| active.id != id)
        || blocked(&app, &care_system::activity()) { return Err("提醒已关闭".into()); }
    let _ = app.emit_to("control", "care-voice-start", ());
    Ok(audio)
}
#[tauri::command]
pub fn dismiss_care(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    bubble(&window)?; let app = window.app_handle();
    if app.state::<ReminderState>().runtime.lock().unwrap_or_else(|e| e.into_inner()).active.as_ref().is_some_and(|d| d.id == id) { close_active(app); }
    Ok(())
}
#[tauri::command]
pub fn snooze_care(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    bubble(&window)?; let app = window.app_handle(); let state = app.state::<ReminderState>();
    let delivery = state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.clone().filter(|d| d.id == id).ok_or("这条提醒已关闭")?;
    let now = Local::now().timestamp(); let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner()); let mut next = store.clone();
    for mut item in delivery.items.into_iter().filter(|p| !p.preview) {
        item.available_at = now + SNOOZE_SECONDS; item.expires_at = item.available_at + if item.sticky { ONCE_VALID_SECONDS } else { VALID_SECONDS };
        next.pending.retain(|p| p.key != item.key); next.pending.push(item);
    }
    save(app, &next)?; *store = next; drop(store); close_active(app); publish(app); Ok(())
}
#[tauri::command]
pub async fn preview_care_rule(window: tauri::WebviewWindow, mut rule: Rule) -> Result<(), String> {
    control(&window)?; validate(&rule)?; let app = window.app_handle().clone();
    if blocked(&app, &care_system::activity()) { return Err("当前处于免打扰、游戏、锁屏或小伴隐藏状态，预览已暂缓；不会创建到期任务".into()); }
    if rule.kind == "weather" {
        let city = rule.city.clone().ok_or("请选择城市")?;
        let worker = app.clone();
        let weather = tauri::async_runtime::spawn_blocking(move || weather_for(&worker, &city)).await.map_err(|e| e.to_string())??;
        rule.text = format!("{} {}", rule.text, weather.text);
    }
    if blocked(&app, &care_system::activity()) { return Err("预览期间进入免打扰状态，提示已取消".into()); }
    // A preview shows the task, not its result: running it here would spend model calls on every preview.
    if rule.kind == "agent" { rule.text = format!("到时小伴会去做：{}", rule.text.chars().take(80).collect::<String>()); }
    let now = Local::now().timestamp();
    show(&app, vec![Pending { key: "preview".into(), rule_id: rule.id, revision: rule.revision, kind: rule.kind, text: rule.text,
        due_at: now, available_at: now, expires_at: now + VALID_SECONDS, preview: true, weather_ready: true, weather_at: None, sticky: false }])
}
#[tauri::command]
pub async fn search_care_cities(window: tauri::WebviewWindow, query: String) -> Result<Vec<City>, String> {
    control(&window)?; let app = window.app_handle(); let state = app.state::<ReminderState>();
    let now = Local::now().timestamp();
    { let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()); if now - runtime.city_attempt < 2 { return Err("城市查询太快，请稍后再试".into()); } runtime.city_attempt = now; }
    let source = state.store.lock().unwrap_or_else(|e| e.into_inner()).config.weather_source.clone();
    tauri::async_runtime::spawn_blocking(move || care_weather::search(&source, &query)).await.map_err(|e| e.to_string())?
}
/// Pre-fills the city of a weather reminder from the network location. Only the control window may ask,
/// and only while a weather source is on, so a user who switched weather off makes no extra request.
#[tauri::command]
pub async fn detect_care_city(window: tauri::WebviewWindow) -> Result<City, String> {
    control(&window)?;
    let state = window.app_handle().state::<ReminderState>();
    if state.store.lock().unwrap_or_else(|e| e.into_inner()).config.weather_source.provider == "disabled" { return Err("请先开启天气来源".into()); }
    let now = chrono::Utc::now().timestamp();
    { let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()); if now - runtime.locate_attempt < 5 { return Err("定位太快，请稍后再试".into()); } runtime.locate_attempt = now; }
    tauri::async_runtime::spawn_blocking(care_weather::locate_by_ip).await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn check_care_weather(window: tauri::WebviewWindow, city: City) -> Result<serde_json::Value, String> {
    control(&window)?; let app = window.app_handle().clone();
    let worker = app.clone();
    tauri::async_runtime::spawn_blocking(move || weather_for(&worker, &city)).await.map_err(|e| e.to_string())??;
    publish(&app); Ok(status(&app))
}
fn weather_for(app: &tauri::AppHandle, city: &City) -> Result<Weather, String> {
    let state = app.state::<ReminderState>(); let now = Local::now().timestamp();
    let source;
    { let store = state.store.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(weather) = store.weather.as_ref().filter(|w| w.city == *city && (0..1800).contains(&(now - w.updated_at)) && now - w.model_at < 7200) { return Ok(weather.clone()); }
        source = store.config.weather_source.clone();
    }
    { let mut runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()); if now - runtime.weather_attempt < 60 { return Err("天气查询每分钟最多一次，请稍后再试".into()); } runtime.weather_attempt = now; }
    let result = care_weather::fetch(&source, city, now);
    let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner());
    if store.config.weather_source != source { return Err("天气来源已改变，请重新查询".into()); }
    match &result { Ok(weather) => { store.weather = Some(weather.clone()); store.weather_error = None; }, Err(error) => { store.weather_error = Some(error.clone()); } }
    if let Err(error) = save(app, &store) { store.error = Some(error); }
    result
}
/// Read-only weather sample; no extra network or IP-based location requests.
pub fn cached_weather(app: &tauri::AppHandle, now: i64) -> Option<Weather> {
    let state = app.state::<ReminderState>();
    let store = state.store.lock().ok()?;
    if store.config.weather_source.provider == "disabled" { return None; }
    store.weather.clone().filter(|w|
        (0..1800).contains(&(now - w.updated_at)) &&
        (0..7200).contains(&(now - w.model_at))
    )
}
pub fn widget_weather_source(app: &tauri::AppHandle) -> care_weather::Source {
    app.state::<ReminderState>().store.lock().unwrap_or_else(|e| e.into_inner()).config.weather_source.clone()
}
pub fn set_game_active(app: &tauri::AppHandle, active: bool) {
    app.state::<ReminderState>().game.store(active, Ordering::Relaxed);
    if active { defer_active(app); }
}
/// Called if the OS closes the bubble; future due reminders can create it again.
pub fn window_closed(app: &tauri::AppHandle) {
    close_active(app);
    app.state::<ReminderState>().runtime.lock().unwrap_or_else(|e| e.into_inner()).ready = false;
}
/// Reserved for native focus state; a low-permission scene cannot set this flag.
#[allow(dead_code)]
pub fn set_focus_active(app: &tauri::AppHandle, active: bool) {
    app.state::<ReminderState>().runtime.lock().unwrap_or_else(|e| e.into_inner()).focus = active;
    if active { defer_active(app); }
}
fn defer_active(app: &tauri::AppHandle) {
    let state = app.state::<ReminderState>();
    let active = state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.clone();
    if let Some(active) = active {
        let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner());
        for item in active.items.into_iter().filter(|p| !p.preview) {
            if !store.pending.iter().any(|p| p.key == item.key) { store.pending.push(item); }
        }
        if let Err(error) = save(app, &store) { store.error = Some(error); }
    }
    close_active(app);
}
fn prune_widget_delivery(app:&tauri::AppHandle,rules:&[Rule]){
 let state=app.state::<ReminderState>();
 let mut runtime=state.runtime.lock().unwrap_or_else(|e| e.into_inner());let Some(active)=runtime.active.as_mut() else{return;};
 let before=active.items.len();
 active.items.retain(|p|!p.rule_id.starts_with("widget-")||rules.iter().any(|r|r.id==p.rule_id&&r.revision==p.revision));
 if active.items.len()==before{return;}
 let empty=active.items.is_empty();let delivery=active.clone();drop(runtime);
 if empty{close_active(app);}else if let Some(window)=app.get_webview_window("care"){let _=window.emit("care-delivery",delivery);}
}
fn tick(app: &tauri::AppHandle, elapsed: f64, checkpoint: bool) {
    let now = Local::now(); let activity = care_system::activity(); let state = app.state::<ReminderState>();
    let active_seconds = if activity.available && !activity.locked && activity.idle_seconds < 60.0 { elapsed } else { 0.0 };
    let widget_rules=crate::widgets::reminder_rules(app,now);
    prune_widget_delivery(app,&widget_rules);
    let changed;
    { let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner()); changed = collect_with_widgets(&mut store, now, active_seconds,&widget_rules);
        if changed || checkpoint { if let Err(error) = save(app, &store) { store.error = Some(error); } }
    }
    // Agent tasks run at their time even in quiet hours or while the user is away; only the bubble waits.
    start_agent_tasks(app);
    if blocked(app, &activity) { defer_active(app); if changed { publish(app); } return; }
    let active = state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.clone();
    if let Some(active) = active {
        // A bubble that stays up must not hold back a reminder that has come due since: fold it back and show both.
        let waiting = active.items.iter().any(|item| item.sticky && !item.preview)
            && state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.iter().any(|p| p.available_at <= now.timestamp() && !active.items.iter().any(|item| item.key == p.key));
        if waiting { defer_active(app); } else {
            let shown_at = state.runtime.lock().unwrap_or_else(|e| e.into_inner()).shown_at;
            if shown_at.is_some_and(|at| now.timestamp() - at >= active.display_seconds) { close_active(app); }
            else if let Some(window) = app.get_webview_window("care") { let _ = position(app, &window); }
            return;
        }
    }
    let pending = state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.iter().find(|p| p.available_at <= now.timestamp()).cloned();
    if let Some(pending) = pending.filter(|p| p.kind == "weather" && !p.weather_ready) {
        let rule = state.store.lock().unwrap_or_else(|e| e.into_inner()).rules.iter().find(|r| r.id == pending.rule_id && r.revision == pending.revision).cloned();
        if let Some(city) = rule.as_ref().and_then(|r| r.city.as_ref()) {
            let result = weather_for(app, city);
            let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner());
            match result {
                Ok(weather) => if let Some(item) = store.pending.iter_mut().find(|p| p.key == pending.key) {
                    item.text = format!("{} {}", item.text, weather.text); item.weather_ready = true; item.weather_at = Some(weather.model_at);
                },
                Err(error) => { store.weather_error = Some(error); store.pending.retain(|p| p.key != pending.key); },
            }
            if let Err(error) = save(app, &store) { store.error = Some(error); }
            drop(store); publish(app);
        }
        return;
    }
    // Network calls and native window creation can overlap with game entry or
    // editing. Recheck both suppression and the expiry before selecting a summary.
    if blocked(app, &care_system::activity()) { return; }
    let mut items = Vec::new();
    { let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner());
        let mut rest = false;
        store.pending.retain(|p| {
            if p.expires_at < Local::now().timestamp() { return false; }
            if p.available_at > now.timestamp() || !p.weather_ready { return true; }
            if p.kind == "rest" && rest { return false; }
            rest |= p.kind == "rest"; items.push(p.clone()); false
        });
        if !items.is_empty() { if let Err(error) = save(app, &store) { store.error = Some(error); store.pending.extend(items.drain(..)); } }
    }
    if !items.is_empty() {
        if let Err(error) = show(app, items.clone()) {
            let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner()); store.pending.extend(items); store.error = Some(error); let _ = save(app, &store);
        }
        publish(app);
    } else if changed { publish(app); }
}
/// Starts each due agent task that is not running yet, on its own thread; its result becomes the bubble's text.
fn start_agent_tasks(app: &tauri::AppHandle) {
    let state = app.state::<ReminderState>();
    let due: Vec<(String, String, String)> = {
        let store = state.store.lock().unwrap_or_else(|e| e.into_inner());
        store.pending.iter().filter(|p| p.kind == "agent" && !p.weather_ready && !p.preview)
            .filter_map(|p| store.rules.iter().find(|r| r.id == p.rule_id && r.revision == p.revision).map(|r| (p.key.clone(), r.id.clone(), r.text.clone())))
            .collect()
    };
    for (key, rule_id, prompt) in due {
        if !state.runtime.lock().unwrap_or_else(|e| e.into_inner()).agent_running.insert(key.clone()) { continue; }
        let (app, thread_key) = (app.clone(), key.clone());
        let started = std::thread::Builder::new().name(format!("care-agent-{rule_id}")).spawn(move || {
            let key = thread_key;
            let outcome = crate::chat::scheduled::run(&app, &rule_id, &prompt);
            let text = outcome.bubble.clone();
            let state = app.state::<ReminderState>();
            {
                let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner());
                record_agent_run(&mut store, &rule_id, &outcome, Local::now().timestamp());
                // Edited or deleted meanwhile: the item is gone, and the answer stays in the conversation only.
                if let Some(item) = store.pending.iter_mut().find(|p| p.key == key) {
                    item.text = text;
                    item.weather_ready = true;
                }
                if let Err(error) = save(&app, &store) { store.error = Some(error); }
            }
            state.runtime.lock().unwrap_or_else(|e| e.into_inner()).agent_running.remove(&key);
            publish(&app);
        });
        if started.is_err() {
            state.runtime.lock().unwrap_or_else(|e| e.into_inner()).agent_running.remove(&key);
        }
    }
}
fn record_agent_run(store: &mut Store, rule_id: &str, outcome: &crate::chat::scheduled::Outcome, at: i64) {
    let ids: HashSet<&str> = store.rules.iter().filter(|r| r.kind == "agent").map(|r| r.id.as_str()).collect();
    let mut runs = std::mem::take(&mut store.agent_runs);
    runs.retain(|id, _| ids.contains(id.as_str()));
    if ids.contains(rule_id) {
        let count = runs.get(rule_id).map_or(0, |run| run.count) + 1;
        runs.insert(rule_id.to_string(), AgentRun {
            at, ok: outcome.ok, summary: outcome.bubble.clone(), session: outcome.session.clone(),
            input_tokens: outcome.usage.prompt_tokens.into(), output_tokens: outcome.usage.completion_tokens.into(), count,
        });
    }
    store.agent_runs = runs;
}
fn presence_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    crate::storage::root(app).ok().map(|root| root.join("config/presence.json"))
}
/// Reuses the care idle/lock sample and its quiet/game gates; the only output is one event to the pet window.
fn presence_tick(app: &tauri::AppHandle, elapsed: f64) {
    let state = app.state::<ReminderState>(); let activity = care_system::activity(); let now = Local::now();
    let silenced = { let store = state.store.lock().unwrap_or_else(|e| e.into_inner()); !store.config.enabled || quiet(&store.config, now.time()) }
        || state.runtime.lock().unwrap_or_else(|e| e.into_inner()).focus || !companion_visible(app);
    let input = crate::presence::Input { local: now.naive_local(), unix: now.timestamp(), idle_seconds: activity.idle_seconds, elapsed,
        locked: activity.locked, available: activity.available, quiet: silenced, game: state.game.load(Ordering::Relaxed) };
    let (kind, greeted) = {
        let mut presence = state.presence.lock().unwrap_or_else(|e| e.into_inner()); let before = presence.first_today_date();
        let kind = presence.step(input); (kind, presence.first_today_date().filter(|day| Some(*day) != before))
    };
    if let (Some(day), Some(path)) = (greeted, presence_path(app)) { let _ = crate::presence::save_first_today(&path, day); }
    if let Some(kind) = kind {
        // Record an aggregated, non-content event. No extra greeting or model call is emitted.
        crate::proactivity::observe_presence(app, kind, now.timestamp_millis());
        let payload = serde_json::json!({"kind": kind.as_str()});
        let _ = app.emit_to("pet", "companion-presence", payload.clone());
        for (label,window) in app.webview_windows(){if crate::pets::is_pet(&label){let _=window.eval(format!("window.dispatchEvent(new CustomEvent('pet-presence',{{detail:{payload}}}));"));}}
        if crate::content::composing(app) {
            for label in ["media-a", "media-b"] {
                if let Some(window) = app.get_webview_window(label) {
                    let _ = window.eval(format!("window.dispatchEvent(new CustomEvent('companion-presence',{{detail:{payload}}}))"));
                }
            }
        }
    }
    // Reuse the existing OS activity sample; do not sample keyboard/lock state twice.
    crate::proactivity::observe_environment(app, &activity);
}
pub fn start(app: &tauri::AppHandle) {
    let state = app.state::<ReminderState>();
    if state.started.swap(true, Ordering::Relaxed) { return; }
    match crate::storage::root(app).and_then(|root| {
        match std::fs::read(root.join("config/care-reminders.json")) {
            Ok(bytes) => serde_json::from_slice::<Store>(&bytes).map_err(|e| format!("关怀配置读取失败，原文件已保留：{e}")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Store::default()),
            Err(e) => Err(format!("关怀配置无法读取，原文件已保留：{e}")),
        }
    }) {
        Ok(mut store) => {
            store.rules.retain(|r| validate(r).is_ok());
            let mut ids = HashSet::new(); store.rules.retain(|r| ids.insert(r.id.clone())); store.rules.truncate(50);
            if time(&store.config.quiet_start).is_err() { store.config.quiet_start = "22:00".into(); }
            if time(&store.config.quiet_end).is_err() { store.config.quiet_end = "08:00".into(); }
            if store.config.weather_source.validate().is_err() { store.config.weather_source = Source::default(); }
            *state.store.lock().unwrap_or_else(|e| e.into_inner()) = store;
        },
        Err(error) => {
            let mut store = state.store.lock().unwrap_or_else(|e| e.into_inner()); store.config.enabled = false; store.error = Some(error);
            state.runtime.lock().unwrap_or_else(|e| e.into_inner()).read_error = true;
        },
    }
    if let Some(path) = presence_path(app) {
        *state.presence.lock().unwrap_or_else(|e| e.into_inner()) = crate::presence::Presence::with_first_today(crate::presence::load_first_today(&path, Local::now().date_naive()));
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let mut previous = Instant::now(); let mut wall = Local::now().timestamp(); let mut checkpoint = Instant::now();
        loop {
            let instant = Instant::now(); let timestamp = Local::now().timestamp(); let seconds = instant.duration_since(previous).as_secs_f64();
            // Do not count a sleep/resume gap, clock jump, or delayed background
            // scheduling as active computer usage. Timed rules still catch up within 30m.
            let elapsed = if seconds <= 15.0 && ((timestamp - wall) as f64 - seconds).abs() < 15.0 { seconds } else { 0.0 };
            previous = instant; wall = timestamp;
            let save_elapsed = checkpoint.elapsed() >= Duration::from_secs(60);
            if !app.state::<ReminderState>().smoke_running.load(Ordering::Relaxed) {
                tick(&app, elapsed, save_elapsed);
                presence_tick(&app, elapsed);
            }
            if save_elapsed { checkpoint = Instant::now(); }
            std::thread::sleep(Duration::from_secs(5));
        }
    });
}

/// Exercises native delivery and disk state only in an isolated smoke profile.
/// No real weather requests, OS notifications, or user rules are used.
pub async fn smoke(app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
    if !app.state::<crate::DesktopState>().isolated { return Err("关怀验证只能使用隔离测试数据目录".into()); }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || smoke_sync(&app)).await.map_err(|e| e.to_string())?
}
fn smoke_sync(app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
    let state = app.state::<ReminderState>();
    state.smoke_running.store(true, Ordering::Relaxed);
    let original_store = state.store.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let original_runtime = state.runtime.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let original_game = state.game.swap(false, Ordering::Relaxed);
    close_active(app);
    let result = (|| {
        let control = app.get_webview_window("control").ok_or("缺少隔离设置窗口")?;
        *state.store.lock().unwrap_or_else(|e| e.into_inner()) = Store::default();
        *state.runtime.lock().unwrap_or_else(|e| e.into_inner()) = Runtime { ready: app.get_webview_window("care").is_some(), ..Runtime::default() };
        let now = Local::now();
        let rule = Rule { id: "smoke-care-rule".into(), text: "隔离验证：喝水".into(), time: now.format("%H:%M").to_string(), ..Rule::default() };
        upsert_care_rule(control.clone(), rule.clone())?;
        let disk_path = crate::storage::root(app)?.join("config/care-reminders.json");
        let disk: Store = serde_json::from_slice(&std::fs::read(&disk_path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        if disk.rules.len() != 1 || disk.rules[0].text != rule.text { return Err("关怀规则没有正确持久化".into()); }
        let pending = |preview: bool| Pending { key: if preview { "smoke-preview".into() } else { "smoke-due".into() }, rule_id: rule.id.clone(), revision: disk.rules[0].revision,
            kind: "custom".into(), text: rule.text.clone(), due_at: now.timestamp(), available_at: now.timestamp(), expires_at: now.timestamp() + VALID_SECONDS, preview, weather_ready: true, weather_at: None, sticky: false };
        show(app, vec![pending(true)])?;
        let deadline = Instant::now() + Duration::from_secs(20);
        while !state.runtime.lock().unwrap_or_else(|e| e.into_inner()).ready {
            if Instant::now() > deadline { return Err("关怀前端未完成原生就绪握手".into()); }
            std::thread::sleep(Duration::from_millis(30));
        }
        let care = app.get_webview_window("care").ok_or("关怀窗口未创建")?;
        let visibility_deadline = Instant::now() + Duration::from_secs(2);
        while !care.is_visible().map_err(|e| e.to_string())? && Instant::now() < visibility_deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        // Keep the actual native focus assertion, including the foreground HWND
        // on Windows. A short settling delay only accommodates OS visibility.
        std::thread::sleep(Duration::from_millis(100));
        let bubble_state = care_system::bubble_diagnostics(&care)?;
        if bubble_state["visible"] != true || bubble_state["focused"] != false {
            return Err(format!("关怀气泡可见/焦点检查失败：{bubble_state}"));
        }
        #[cfg(windows)]
        if bubble_state["nativeVisible"] != true || bubble_state["foregroundIsBubble"] != false || bubble_state["noActivate"] != true || bubble_state["topmost"] != true {
            return Err(format!("关怀气泡原生不激活检查失败：{bubble_state}"));
        }
        let preview = get_care_delivery(care.clone())?.ok_or("没有预览气泡")?;
        if !preview.items[0].preview || !state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.is_empty() { return Err("预览错误地创建了真实待展示任务".into()); }
        dismiss_care(care.clone(), preview.id)?;
        let hide_deadline = Instant::now() + Duration::from_secs(2);
        while care.is_visible().map_err(|e| e.to_string())? && Instant::now() < hide_deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        let after_dismiss = care_system::bubble_diagnostics(&care)?;
        let remaining_delivery = get_care_delivery(care.clone())?;
        if remaining_delivery.is_some() || after_dismiss["visible"] != false {
            return Err(format!("气泡关闭没有生效：window={after_dismiss}, delivery={}", serde_json::to_value(remaining_delivery).map_err(|e| e.to_string())?));
        }
        #[cfg(windows)]
        if after_dismiss["nativeVisible"] != false { return Err(format!("气泡原生窗口关闭没有生效：{after_dismiss}")); }

        show(app, vec![pending(false)])?;
        let delivery = get_care_delivery(care.clone())?.ok_or("没有到期提示")?;
        snooze_care(care.clone(), delivery.id)?;
        let disk: Store = serde_json::from_slice(&std::fs::read(&disk_path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        if disk.pending.len() != 1 || disk.pending[0].available_at < now.timestamp() + SNOOZE_SECONDS { return Err("稍后提醒没有按 10 分钟持久化".into()); }
        upsert_care_rule(control.clone(), Rule { enabled: false, ..rule.clone() })?;
        if !state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.is_empty() { return Err("关闭规则没有取消稍后提醒".into()); }

        upsert_care_rule(control.clone(), rule.clone())?;
        let revision = state.store.lock().unwrap_or_else(|e| e.into_inner()).rules[0].revision;
        let mut item = pending(false); item.revision = revision;
        state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.push(item.clone());
        set_game_active(app, true);
        tick(app, 0.0, true);
        if state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.is_some() || state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.len() != 1 { return Err("游戏期间提醒没有延后".into()); }
        set_game_active(app, false);
        tick(app, 0.0, true);
        if state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.is_none() { return Err("游戏退出后有效提示没有恢复".into()); }
        let active = get_care_delivery(care.clone())?.ok_or("没有恢复气泡")?;
        dismiss_care(care.clone(), active.id)?;
        let mut expired = item.clone(); expired.key = "smoke-expired".into(); expired.expires_at = Local::now().timestamp() - 1;
        state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.push(expired);
        tick(app, 0.0, true);
        if !state.store.lock().unwrap_or_else(|e| e.into_inner()).pending.is_empty() || state.runtime.lock().unwrap_or_else(|e| e.into_inner()).active.is_some() { return Err("过期提醒被补发".into()); }
        delete_care_rule(control, rule.id.clone())?;
        if !state.store.lock().unwrap_or_else(|e| e.into_inner()).rules.is_empty() { return Err("删除规则没有生效".into()); }
        let mut fixture = Store { rules: vec![Rule { id: "dedup".into(), revision: 1, kind: "custom".into(), repeat: "once".into(), once_at: (now - CalendarDuration::minutes(1)).format("%Y-%m-%dT%H:%M").to_string(), text: "fixture".into(), ..Rule::default() }], ..Store::default() };
        collect(&mut fixture, now, 0.0);
        let mut restored: Store = serde_json::from_slice(&serde_json::to_vec(&fixture).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        collect(&mut restored, now, 0.0);
        if restored.pending.len() != 1 || restored.seen.len() != 1 { return Err("重启状态去重失败".into()); }
        Ok(serde_json::json!({"persistence":true,"previewNoTask":true,"bubbleReady":true,"noFocus":true,"dismiss":true,"snooze":true,
            "disableCancels":true,"delete":true,"gameDeferred":true,"gameResumeSummary":true,"expiry":true,"restartDedup":true,"weatherNetworkUsed":false,"bubbleState":bubble_state,"afterDismiss":after_dismiss}))
    })();
    close_active(app);
    *state.store.lock().unwrap_or_else(|e| e.into_inner()) = original_store.clone();
    *state.runtime.lock().unwrap_or_else(|e| e.into_inner()) = Runtime { ready: app.get_webview_window("care").is_some(), active: None, shown_at: None, ..original_runtime };
    state.game.store(original_game, Ordering::Relaxed);
    let restore = save(app, &original_store);
    state.smoke_running.store(false, Ordering::Relaxed);
    restore?;
    result
}

#[cfg(test)]
mod tests {
    #[test] fn widget_reminders_persist_dedup_and_cancel_when_removed(){
        let now=chrono::Local.with_ymd_and_hms(2026,10,19,10,0,0).unwrap();
        let rule=Rule{id:"widget-one-task".into(),revision:1,repeat:"once".into(),once_at:"2026-10-19T10:00".into(),time:"10:00".into(),text:"读书".into(),..Rule::default()};
        let mut store=Store::default();assert!(collect_with_widgets(&mut store,now,0.,&[rule.clone()]));assert_eq!(store.pending.len(),1);
        let bytes=serde_json::to_vec(&store).unwrap();let mut restarted:Store=serde_json::from_slice(&bytes).unwrap();
        assert!(!collect_with_widgets(&mut restarted,now,0.,&[rule.clone()]));assert_eq!(restarted.pending.len(),1);
        assert!(collect_with_widgets(&mut restarted,now,0.,&[]));assert!(restarted.pending.is_empty());
    }
    use super::*;
    #[test]
    fn old_care_config_keeps_voice_off() {
        let config: CareConfig = serde_json::from_str(r#"{"enabled":true,"sound":true}"#).unwrap();
        assert!(config.sound);
        assert!(!config.voice);
    }
    fn date() -> DateTime<Local> { Local.with_ymd_and_hms(2026, 10, 9, 15, 10, 0).earliest().unwrap() }
    fn scheduled() -> Rule { Rule { id: "test".into(), text: "喝水".into(), revision: 1, ..Rule::default() } }
    #[test] fn due_once_deduplicates_clock_rollback_and_restart() {
        let mut store = Store { rules: vec![scheduled()], ..Store::default() };
        assert!(collect(&mut store, date(), 0.0)); assert_eq!(store.pending.len(), 1);
        let bytes = serde_json::to_vec(&store).unwrap(); let mut restored: Store = serde_json::from_slice(&bytes).unwrap();
        collect(&mut restored, date() - CalendarDuration::minutes(5), 0.0); assert_eq!(restored.pending.len(), 1);
    }
    #[test] fn expired_reminders_do_not_replay_days_later() {
        let mut store = Store { rules: vec![scheduled()], ..Store::default() };
        collect(&mut store, date() + CalendarDuration::minutes(31), 0.0); assert!(store.pending.is_empty());
    }
    #[test] fn explicit_workdays_are_respected() {
        let mut rule = scheduled(); rule.repeat = "weekdays".into(); rule.weekdays = vec![1, 3]; assert!(local_due(&rule, date()).is_empty());
        rule.weekdays = vec![5]; assert_eq!(local_due(&rule, date()).len(), 1);
    }
    #[test] fn a_one_time_reminder_is_kept_for_hours_and_its_bubble_stays_until_answered() {
        let mut rule = scheduled(); rule.repeat = "once".into(); rule.once_at = "2026-10-09T15:05".into();
        let hours_later = Local.with_ymd_and_hms(2026, 10, 9, 21, 0, 0).earliest().unwrap();
        assert_eq!(local_due(&rule, hours_later).len(), 1, "still delivered after a long absence");
        assert!(local_due(&rule, hours_later + CalendarDuration::hours(7)).is_empty(), "but not a day later");
        let mut store = Store { rules: vec![rule.clone()], ..Store::default() };
        assert!(collect(&mut store, hours_later, 0.0));
        let item = &store.pending[0];
        assert!(item.sticky && item.expires_at - item.due_at == ONCE_VALID_SECONDS);
        // A repeating reminder is as before: shown briefly, and only for half an hour.
        rule.repeat = "daily".into(); rule.time = "15:05".into();
        assert!(local_due(&rule, hours_later).is_empty());
        let mut daily = Store { rules: vec![rule], ..Store::default() };
        assert!(collect(&mut daily, date(), 0.0) && !daily.pending[0].sticky);
    }
    #[test] fn one_time_reminders_are_read_aloud_by_default_and_led_in_by_a_word() {
        let config = CareConfig::default();
        assert!(!config.voice && config.once_voice);
        assert!(speaks(&config, true) && !speaks(&config, false));
        assert!(!speaks(&CareConfig { once_voice: false, ..CareConfig::default() }, true));
        assert!(speaks(&CareConfig { voice: true, once_voice: false, ..CareConfig::default() }, false));
        let old: CareConfig = serde_json::from_str(r#"{"enabled":true,"voice":false}"#).unwrap();
        assert!(old.once_voice);
        let item = |kind: &str, text: &str| Pending { key: text.into(), rule_id: "r".into(), revision: 1, kind: kind.into(), text: text.into(), due_at: 0, available_at: 0, expires_at: 0, preview: false, weather_ready: true, weather_at: None, sticky: true };
        assert_eq!(spoken(&[item("custom", "买花")]), "提醒：买花");
        assert_eq!(spoken(&[item("custom", "买花"), item("custom", "喝水"), item("rest", "站起来")]), "提醒：买花。喝水。还有 1 条到期提醒。");
        assert_eq!(spoken(&[item("agent", "今天有 3 封新邮件")]), "今天有 3 封新邮件", "a task's result is not led in as a reminder");
        assert_eq!(spoken(&[item("custom", "  ")]), "");
    }
    #[test] fn one_time_reminders_beep_by_default_and_everything_else_stays_quiet() {
        let config = CareConfig::default();
        assert!(!config.sound && config.once_sound);
        assert!(plays_sound(&config, true) && !plays_sound(&config, false));
        assert!(!plays_sound(&CareConfig { once_sound: false, ..CareConfig::default() }, true), "it can be switched off");
        assert!(plays_sound(&CareConfig { sound: true, once_sound: false, ..CareConfig::default() }, false));
        // A settings file written before this option existed gets the default, on.
        let old: CareConfig = serde_json::from_str(r#"{"enabled":true,"sound":false,"voice":false}"#).unwrap();
        assert!(old.once_sound);
    }
    #[test] fn once_and_midnight_catchup_work() {
        let mut rule = scheduled(); rule.repeat = "once".into(); rule.once_at = "2026-10-09T15:05".into(); assert_eq!(local_due(&rule, date()).len(), 1);
        rule.repeat = "daily".into(); rule.time = "23:55".into(); let next = Local.with_ymd_and_hms(2026,10,10,0,5,0).earliest().unwrap(); assert_eq!(local_due(&rule, next).len(), 1);
    }
    #[test] fn rest_counts_only_supplied_active_time_and_merges() {
        let mut rule = scheduled(); rule.kind = "rest".into(); rule.interval_minutes = 5;
        let mut store = Store { rules: vec![rule], ..Store::default() }; collect(&mut store, date(), 299.0); assert!(store.pending.is_empty());
        collect(&mut store, date(), 0.0); assert!(store.pending.is_empty()); collect(&mut store, date(), 1.0); assert_eq!(store.pending.len(), 1);
        collect(&mut store, date() + CalendarDuration::minutes(5), 300.0); assert_eq!(store.pending.len(), 1);
    }
    #[test] fn an_agent_task_waits_for_its_result_and_may_wait_longer_to_be_seen() {
        let rule = Rule { id: "task".into(), kind: "agent".into(), text: "看看下载文件夹里有什么新文件".into(), revision: 1, ..Rule::default() };
        assert!(validate(&rule).is_ok());
        assert!(validate(&Rule { text: "长".repeat(AGENT_PROMPT_CHARS + 1), ..rule.clone() }).is_err());
        assert!(validate(&Rule { text: "长".repeat(500), ..rule.clone() }).is_ok(), "longer than an ordinary reminder's text");
        let mut store = Store { rules: vec![rule], ..Store::default() };
        assert!(collect(&mut store, date(), 0.0));
        let item = &store.pending[0];
        assert!(!item.weather_ready, "not shown before the task has run");
        assert_eq!(item.expires_at - item.due_at, AGENT_VALID_SECONDS);
    }
    #[test] fn the_last_run_of_each_scheduled_task_is_kept_with_its_cost() {
        let task = Rule { id: "task".into(), kind: "agent".into(), text: "总结".into(), ..Rule::default() };
        let mut store = Store { rules: vec![task, scheduled()], ..Store::default() };
        store.agent_runs.insert("deleted".into(), AgentRun::default());
        let done = crate::chat::scheduled::Outcome { ok: true, bubble: "三个文件改过".into(), session: Some("s1".into()),
            usage: agent_kernel::Usage { prompt_tokens: 1200, completion_tokens: 80, ..Default::default() } };
        record_agent_run(&mut store, "task", &done, 100);
        record_agent_run(&mut store, "task", &crate::chat::scheduled::Outcome { bubble: "定时任务没有完成：没有模型".into(), ..Default::default() }, 200);
        let run = &store.agent_runs["task"];
        assert_eq!((run.at, run.ok, run.count, run.input_tokens), (200, false, 2, 0));
        assert!(run.summary.contains("没有完成"));
        assert!(!store.agent_runs.contains_key("deleted"), "runs of deleted tasks are dropped");
        record_agent_run(&mut store, &scheduled().id, &done, 300);
        assert_eq!(store.agent_runs.len(), 1, "only scheduled tasks are recorded");
    }
    #[test] fn quiet_hours_wrap_midnight() { let config = CareConfig { quiet_enabled: true, ..CareConfig::default() }; assert!(quiet(&config,time("23:00").unwrap())); assert!(quiet(&config,time("07:59").unwrap())); assert!(!quiet(&config,time("08:00").unwrap())); }
    #[test] fn disabled_or_edited_rules_cancel_queued_items() {
        let mut store = Store { rules: vec![scheduled()], ..Store::default() }; collect(&mut store,date(),0.0); store.rules[0].revision += 1;
        store.rules[0].enabled = false; collect(&mut store,date(),0.0); assert!(store.pending.is_empty());
    }
    #[test] fn deleting_or_editing_one_rule_preserves_other_already_delivered_reminder() {
        for delete in [true, false] {
            let first = scheduled();
            let second = Rule { id: "second".into(), text: "下班".into(), ..scheduled() };
            let mut store = Store { rules: vec![first, second], ..Store::default() };
            collect(&mut store, date(), 0.0);
            assert_eq!(store.pending.len(), 2);
            let remaining_key = store.pending.iter().find(|item| item.rule_id == "second").unwrap().key.clone();
            let mut runtime = Runtime { active: Some(Delivery { id: "combined".into(), items: std::mem::take(&mut store.pending), sound: false, voice: false, display_seconds: BUBBLE_SECONDS }),
                shown_at: Some(date().timestamp()), ..Runtime::default() };
            if delete { store.rules.retain(|rule| rule.id != "test"); }
            else { store.rules[0].revision += 1; }
            assert!(remove_active_rule(&mut runtime, "test"));
            let delivery = runtime.active.as_ref().unwrap();
            assert_eq!(delivery.id, "combined");
            assert_eq!(delivery.items.len(), 1);
            assert_eq!(delivery.items[0].rule_id, "second");
            assert_eq!(delivery.items[0].key, remaining_key);
            assert_eq!(delivery.items[0].text, "下班");
            assert!(store.seen.contains_key(&remaining_key));
            assert!(store.pending.is_empty());
            assert_eq!(runtime.shown_at, Some(date().timestamp()));
            assert!(!remove_active_rule(&mut runtime, "test"));
            assert!(remove_active_rule(&mut runtime, "second"));
            assert!(runtime.active.is_none());
            assert!(runtime.shown_at.is_none());
        }
    }
}

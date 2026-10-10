//! Care scheduling runs in the native host, independently of WebView visibility.
use crate::{care_system, care_weather::{self, City, Source, Weather}};
use chrono::{DateTime, Datelike, Duration as CalendarDuration, Local, NaiveDateTime, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashSet}, sync::{Mutex, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const VALID_SECONDS: i64 = 30 * 60;
const BUBBLE_SECONDS: i64 = 10;
const SNOOZE_SECONDS: i64 = 10 * 60;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CareConfig {
    pub enabled: bool,
    pub sound: bool,
    pub quiet_enabled: bool,
    pub quiet_start: String,
    pub quiet_end: String,
    pub weather_source: Source,
}
impl Default for CareConfig {
    fn default() -> Self { Self { enabled: true, sound: false, quiet_enabled: false, quiet_start: "22:00".into(), quiet_end: "08:00".into(), weather_source: Source::default() } }
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
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    pub id: String,
    pub items: Vec<Pending>,
    pub sound: bool,
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
    if app.state::<ReminderState>().runtime.lock().unwrap().read_error {
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
    if !["custom", "off-work", "rest", "weather"].contains(&rule.kind.as_str()) { return Err("提醒类型无效".into()); }
    if rule.text.chars().count() > 240 { return Err("提醒文案最多 240 个字".into()); }
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
        if due > now.timestamp() || now.timestamp() - due > VALID_SECONDS || due < rule.not_before { return None; }
        Some((format!("{}:{}:{}", rule.id, rule.revision, date.format("%Y-%m-%dT%H:%M")), due))
    }).collect()
}
fn collect(store: &mut Store, now: DateTime<Local>, active_seconds: f64) -> bool {
    let timestamp = now.timestamp();
    let before = store.pending.len();
    store.pending.retain(|item| item.expires_at >= timestamp && (item.preview || store.rules.iter().any(|r| r.enabled && r.id == item.rule_id && r.revision == item.revision)));
    for item in &mut store.pending {
        if item.kind == "weather" && item.weather_ready && item.weather_at.is_none_or(|at| timestamp - at > 7200 || at - timestamp > 1800) {
            item.weather_ready = false;
            item.text = store.rules.iter().find(|r| r.id == item.rule_id).map(|r| r.text.clone()).unwrap_or_default();
        }
    }
    let mut changed = before != store.pending.len();
    if !store.config.enabled { return changed; }
    for rule in store.rules.clone().into_iter().filter(|r| r.enabled) {
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
            store.pending.push(Pending { key, rule_id: rule.id.clone(), revision: rule.revision, kind: rule.kind.clone(), text: rule.text.clone(),
                due_at: due, available_at: due, expires_at: due + VALID_SECONDS, preview: false, weather_ready: rule.kind != "weather", weather_at: None });
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
    let store = state.store.lock().unwrap().clone();
    let activity = care_system::activity();
    serde_json::json!({"config":store.config,"rules":store.rules,"pendingCount":store.pending.len(),"weather":store.weather,
        "weatherError":store.weather_error,"error":store.error,"systemNotificationsAvailable":false,"activityAvailable":activity.available})
}
fn publish(app: &tauri::AppHandle) { let _ = app.emit_to("control", "care-settings-changed", status(app)); }

#[tauri::command]
pub fn get_care_settings(window: tauri::WebviewWindow) -> Result<serde_json::Value, String> { control(&window)?; Ok(status(window.app_handle())) }

#[tauri::command]
pub fn configure_care(window: tauri::WebviewWindow, config: CareConfig) -> Result<serde_json::Value, String> {
    control(&window)?; time(&config.quiet_start)?; time(&config.quiet_end)?; config.weather_source.validate()?;
    let app = window.app_handle(); let state = app.state::<ReminderState>();
    let mut store = state.store.lock().unwrap(); let mut next = store.clone();
    if next.config.weather_source != config.weather_source {
        next.weather = None; next.weather_error = None;
        next.pending.retain(|p| p.kind != "weather");
    }
    next.config = config;
    if !next.config.enabled { next.pending.clear(); }
    save(app, &next)?; *store = next; drop(store);
    if !state.store.lock().unwrap().config.enabled { close_active(app); }
    publish(app); Ok(status(app))
}

#[tauri::command]
pub fn upsert_care_rule(window: tauri::WebviewWindow, mut rule: Rule) -> Result<serde_json::Value, String> {
    control(&window)?; let app = window.app_handle(); let state = app.state::<ReminderState>();
    let now = Local::now().timestamp();
    let mut store = state.store.lock().unwrap(); let mut next = store.clone();
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
    let mut store = state.store.lock().unwrap(); let mut next = store.clone();
    next.rules.retain(|r| r.id != id); next.pending.retain(|p| p.rule_id != id); next.rest_seconds.remove(&id);
    save(app, &next)?; *store = next; drop(store); cancel_active_rule(app, &id); publish(app); Ok(status(app))
}
fn companion_visible(app: &tauri::AppHandle) -> bool {
    let settings = crate::settings(app);
    if !settings.companion_enabled { return false; }
    app.get_webview_window("pet").is_some_and(|w| w.is_visible().unwrap_or(false))
}
fn blocked(app: &tauri::AppHandle, activity: &care_system::Activity) -> bool {
    let state = app.state::<ReminderState>();
    let store = state.store.lock().unwrap();
    !store.config.enabled || quiet(&store.config, Local::now().time()) || activity.locked
        || state.game.load(Ordering::Relaxed) || state.runtime.lock().unwrap().focus || !companion_visible(app)
}
fn position(app: &tauri::AppHandle, window: &tauri::WebviewWindow) -> Result<(), String> {
    let pet = app.get_webview_window("pet");
    let monitor = pet.as_ref().and_then(|w| w.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten()).ok_or("没有可用显示器")?;
    let scale = monitor.scale_factor();
    let origin = monitor.position(); let size = monitor.size();
    let left = origin.x as f64 / scale; let top = origin.y as f64 / scale;
    let width = size.width as f64 / scale; let height = size.height as f64 / scale;
    let (anchor_x, anchor_y) = if let Some(pet) = pet.filter(|_| crate::settings(app).mode == "floating") {
        let p = pet.outer_position().map_err(|e| e.to_string())?;
        let s = pet.outer_size().map_err(|e| e.to_string())?;
        (p.x as f64 / scale + s.width as f64 / scale / 2.0, p.y as f64 / scale)
    } else { (left + width * 0.7, top + height * 0.62) };
    let x = (anchor_x - 170.0).clamp(left + 8.0, (left + width - 348.0).max(left + 8.0));
    let y = if anchor_y - 188.0 >= top + 8.0 { anchor_y - 188.0 } else { anchor_y + crate::settings(app).size * 1.1 };
    let y = y.clamp(top + 8.0, (top + height - 188.0).max(top + 8.0));
    window.set_position(tauri::LogicalPosition::new(x, y)).map_err(|e| e.to_string())
}
fn open_bubble(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("care") {
        position(app, &window)?;
        let state = app.state::<ReminderState>();
        let ready = { let runtime = state.runtime.lock().unwrap(); runtime.ready && runtime.active.is_some() };
        if ready && !blocked(app, &care_system::activity()) { care_system::show_bubble(&window)?; }
        return Ok(());
    }
    let builder = WebviewWindowBuilder::new(app, "care", WebviewUrl::App("index.html?scene=care".into()))
        .title("小伴关怀").decorations(false).shadow(false).resizable(false).skip_taskbar(true)
        .always_on_top(true).transparent(true).visible(false).focused(false).focusable(false).inner_size(340.0, 180.0);
    #[cfg(windows)] let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
    #[cfg(target_os = "macos")] let builder = builder.incognito(true).background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    let window = builder.build().map_err(|e| e.to_string())?;
    position(app, &window)
}
fn close_active(app: &tauri::AppHandle) {
    let state = app.state::<ReminderState>();
    let mut runtime = state.runtime.lock().unwrap(); runtime.active = None; runtime.shown_at = None; drop(runtime);
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
        let mut runtime = state.runtime.lock().unwrap();
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
    let sound = state.store.lock().unwrap().config.sound;
    let delivery = Delivery { id: format!("bubble-{}", Local::now().timestamp_millis()), items, sound, display_seconds: BUBBLE_SECONDS };
    { let mut runtime = state.runtime.lock().unwrap(); runtime.active = Some(delivery.clone()); runtime.shown_at = if runtime.ready { Some(Local::now().timestamp()) } else { None }; }
    if let Err(error) = open_bubble(app) { close_active(app); return Err(error); }
    if blocked(app, &care_system::activity()) { defer_active(app); return Ok(()); }
    if state.runtime.lock().unwrap().active.as_ref().is_none_or(|d| d.id != delivery.id) { return Ok(()); }
    if let Some(window) = app.get_webview_window("care") { let _ = window.emit("care-delivery", &delivery); }
    if state.runtime.lock().unwrap().ready && delivery.sound { care_system::play_sound(); }
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
    let delivery = state.runtime.lock().unwrap().active.clone();
    if delivery.is_some() && !blocked(app, &care_system::activity()) {
        care_system::show_bubble(&window)?;
        if delivery.as_ref().is_some_and(|d| d.sound) { care_system::play_sound(); }
    }
    // Readiness is published only after the native show call completes. The
    // smoke observer and scheduler must not see a ready-but-still-hidden window.
    { let mut runtime = state.runtime.lock().unwrap(); runtime.ready = true;
      if runtime.active.is_some() { runtime.shown_at = Some(Local::now().timestamp()); } }
    Ok(delivery)
}
#[tauri::command]
pub fn get_care_delivery(window: tauri::WebviewWindow) -> Result<Option<Delivery>, String> {
    bubble(&window)?; Ok(window.app_handle().state::<ReminderState>().runtime.lock().unwrap().active.clone())
}
#[tauri::command]
pub fn dismiss_care(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    bubble(&window)?; let app = window.app_handle();
    if app.state::<ReminderState>().runtime.lock().unwrap().active.as_ref().is_some_and(|d| d.id == id) { close_active(app); }
    Ok(())
}
#[tauri::command]
pub fn snooze_care(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    bubble(&window)?; let app = window.app_handle(); let state = app.state::<ReminderState>();
    let delivery = state.runtime.lock().unwrap().active.clone().filter(|d| d.id == id).ok_or("这条提醒已关闭")?;
    let now = Local::now().timestamp(); let mut store = state.store.lock().unwrap(); let mut next = store.clone();
    for mut item in delivery.items.into_iter().filter(|p| !p.preview) {
        item.available_at = now + SNOOZE_SECONDS; item.expires_at = item.available_at + VALID_SECONDS;
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
    let now = Local::now().timestamp();
    show(&app, vec![Pending { key: "preview".into(), rule_id: rule.id, revision: rule.revision, kind: rule.kind, text: rule.text,
        due_at: now, available_at: now, expires_at: now + VALID_SECONDS, preview: true, weather_ready: true, weather_at: None }])
}
#[tauri::command]
pub async fn search_care_cities(window: tauri::WebviewWindow, query: String) -> Result<Vec<City>, String> {
    control(&window)?; let app = window.app_handle(); let state = app.state::<ReminderState>();
    let now = Local::now().timestamp();
    { let mut runtime = state.runtime.lock().unwrap(); if now - runtime.city_attempt < 2 { return Err("城市查询太快，请稍后再试".into()); } runtime.city_attempt = now; }
    let source = state.store.lock().unwrap().config.weather_source.clone();
    tauri::async_runtime::spawn_blocking(move || care_weather::search(&source, &query)).await.map_err(|e| e.to_string())?
}
/// Pre-fills the city of a weather reminder from the network location. Only the control window may ask,
/// and only while a weather source is on, so a user who switched weather off makes no extra request.
#[tauri::command]
pub async fn detect_care_city(window: tauri::WebviewWindow) -> Result<City, String> {
    control(&window)?;
    let state = window.app_handle().state::<ReminderState>();
    if state.store.lock().unwrap().config.weather_source.provider == "disabled" { return Err("请先开启天气来源".into()); }
    let now = chrono::Utc::now().timestamp();
    { let mut runtime = state.runtime.lock().unwrap(); if now - runtime.locate_attempt < 5 { return Err("定位太快，请稍后再试".into()); } runtime.locate_attempt = now; }
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
    { let store = state.store.lock().unwrap();
        if let Some(weather) = store.weather.as_ref().filter(|w| w.city == *city && (0..1800).contains(&(now - w.updated_at)) && now - w.model_at < 7200) { return Ok(weather.clone()); }
        source = store.config.weather_source.clone();
    }
    { let mut runtime = state.runtime.lock().unwrap(); if now - runtime.weather_attempt < 60 { return Err("天气查询每分钟最多一次，请稍后再试".into()); } runtime.weather_attempt = now; }
    let result = care_weather::fetch(&source, city, now);
    let mut store = state.store.lock().unwrap();
    if store.config.weather_source != source { return Err("天气来源已改变，请重新查询".into()); }
    match &result { Ok(weather) => { store.weather = Some(weather.clone()); store.weather_error = None; }, Err(error) => { store.weather_error = Some(error.clone()); } }
    if let Err(error) = save(app, &store) { store.error = Some(error); }
    result
}
pub fn set_game_active(app: &tauri::AppHandle, active: bool) {
    app.state::<ReminderState>().game.store(active, Ordering::Relaxed);
    if active { defer_active(app); }
}
/// Called if the OS closes the bubble; future due reminders can create it again.
pub fn window_closed(app: &tauri::AppHandle) {
    close_active(app);
    app.state::<ReminderState>().runtime.lock().unwrap().ready = false;
}
/// Reserved for native focus state; a low-permission scene cannot set this flag.
#[allow(dead_code)]
pub fn set_focus_active(app: &tauri::AppHandle, active: bool) {
    app.state::<ReminderState>().runtime.lock().unwrap().focus = active;
    if active { defer_active(app); }
}
fn defer_active(app: &tauri::AppHandle) {
    let state = app.state::<ReminderState>();
    let active = state.runtime.lock().unwrap().active.clone();
    if let Some(active) = active {
        let mut store = state.store.lock().unwrap();
        for item in active.items.into_iter().filter(|p| !p.preview) {
            if !store.pending.iter().any(|p| p.key == item.key) { store.pending.push(item); }
        }
        if let Err(error) = save(app, &store) { store.error = Some(error); }
    }
    close_active(app);
}
fn tick(app: &tauri::AppHandle, elapsed: f64, checkpoint: bool) {
    let now = Local::now(); let activity = care_system::activity(); let state = app.state::<ReminderState>();
    let active_seconds = if activity.available && !activity.locked && activity.idle_seconds < 60.0 { elapsed } else { 0.0 };
    let changed;
    { let mut store = state.store.lock().unwrap(); changed = collect(&mut store, now, active_seconds);
        if changed || checkpoint { if let Err(error) = save(app, &store) { store.error = Some(error); } }
    }
    if blocked(app, &activity) { defer_active(app); if changed { publish(app); } return; }
    let active = state.runtime.lock().unwrap().active.clone();
    if active.is_some() {
        let shown_at = state.runtime.lock().unwrap().shown_at;
        if shown_at.is_some_and(|at| now.timestamp() - at >= BUBBLE_SECONDS) { close_active(app); }
        else if let Some(window) = app.get_webview_window("care") { let _ = position(app, &window); }
        return;
    }
    let pending = state.store.lock().unwrap().pending.iter().find(|p| p.available_at <= now.timestamp()).cloned();
    if let Some(pending) = pending.filter(|p| p.kind == "weather" && !p.weather_ready) {
        let rule = state.store.lock().unwrap().rules.iter().find(|r| r.id == pending.rule_id && r.revision == pending.revision).cloned();
        if let Some(city) = rule.as_ref().and_then(|r| r.city.as_ref()) {
            let result = weather_for(app, city);
            let mut store = state.store.lock().unwrap();
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
    { let mut store = state.store.lock().unwrap();
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
            let mut store = state.store.lock().unwrap(); store.pending.extend(items); store.error = Some(error); let _ = save(app, &store);
        }
        publish(app);
    } else if changed { publish(app); }
}
fn presence_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    crate::storage::root(app).ok().map(|root| root.join("config/presence.json"))
}
/// Reuses the care idle/lock sample and its quiet/game gates; the only output is one event to the pet window.
fn presence_tick(app: &tauri::AppHandle, elapsed: f64) {
    let state = app.state::<ReminderState>(); let activity = care_system::activity(); let now = Local::now();
    let silenced = { let store = state.store.lock().unwrap(); !store.config.enabled || quiet(&store.config, now.time()) }
        || state.runtime.lock().unwrap().focus || !companion_visible(app);
    let input = crate::presence::Input { local: now.naive_local(), unix: now.timestamp(), idle_seconds: activity.idle_seconds, elapsed,
        locked: activity.locked, available: activity.available, quiet: silenced, game: state.game.load(Ordering::Relaxed) };
    let (kind, greeted) = {
        let mut presence = state.presence.lock().unwrap(); let before = presence.first_today_date();
        let kind = presence.step(input); (kind, presence.first_today_date().filter(|day| Some(*day) != before))
    };
    if let (Some(day), Some(path)) = (greeted, presence_path(app)) { let _ = crate::presence::save_first_today(&path, day); }
    if let Some(kind) = kind { let _ = app.emit_to("pet", "companion-presence", serde_json::json!({"kind": kind.as_str()})); }
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
            *state.store.lock().unwrap() = store;
        },
        Err(error) => {
            let mut store = state.store.lock().unwrap(); store.config.enabled = false; store.error = Some(error);
            state.runtime.lock().unwrap().read_error = true;
        },
    }
    if let Some(path) = presence_path(app) {
        *state.presence.lock().unwrap() = crate::presence::Presence::with_first_today(crate::presence::load_first_today(&path, Local::now().date_naive()));
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
            if !app.state::<ReminderState>().smoke_running.load(Ordering::Relaxed) { tick(&app, elapsed, save_elapsed); presence_tick(&app, elapsed); }
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
    let original_store = state.store.lock().unwrap().clone();
    let original_runtime = state.runtime.lock().unwrap().clone();
    let original_game = state.game.swap(false, Ordering::Relaxed);
    close_active(app);
    let result = (|| {
        let control = app.get_webview_window("control").ok_or("缺少隔离设置窗口")?;
        *state.store.lock().unwrap() = Store::default();
        *state.runtime.lock().unwrap() = Runtime { ready: app.get_webview_window("care").is_some(), ..Runtime::default() };
        let now = Local::now();
        let rule = Rule { id: "smoke-care-rule".into(), text: "隔离验证：喝水".into(), time: now.format("%H:%M").to_string(), ..Rule::default() };
        upsert_care_rule(control.clone(), rule.clone())?;
        let disk_path = crate::storage::root(app)?.join("config/care-reminders.json");
        let disk: Store = serde_json::from_slice(&std::fs::read(&disk_path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        if disk.rules.len() != 1 || disk.rules[0].text != rule.text { return Err("关怀规则没有正确持久化".into()); }
        let pending = |preview: bool| Pending { key: if preview { "smoke-preview".into() } else { "smoke-due".into() }, rule_id: rule.id.clone(), revision: disk.rules[0].revision,
            kind: "custom".into(), text: rule.text.clone(), due_at: now.timestamp(), available_at: now.timestamp(), expires_at: now.timestamp() + VALID_SECONDS, preview, weather_ready: true, weather_at: None };
        show(app, vec![pending(true)])?;
        let deadline = Instant::now() + Duration::from_secs(20);
        while !state.runtime.lock().unwrap().ready {
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
        if bubble_state["nativeVisible"] != true || bubble_state["foregroundIsBubble"] != false || bubble_state["noActivate"] != true {
            return Err(format!("关怀气泡原生不激活检查失败：{bubble_state}"));
        }
        let preview = get_care_delivery(care.clone())?.ok_or("没有预览气泡")?;
        if !preview.items[0].preview || !state.store.lock().unwrap().pending.is_empty() { return Err("预览错误地创建了真实待展示任务".into()); }
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
        if !state.store.lock().unwrap().pending.is_empty() { return Err("关闭规则没有取消稍后提醒".into()); }

        upsert_care_rule(control.clone(), rule.clone())?;
        let revision = state.store.lock().unwrap().rules[0].revision;
        let mut item = pending(false); item.revision = revision;
        state.store.lock().unwrap().pending.push(item.clone());
        set_game_active(app, true);
        tick(app, 0.0, true);
        if state.runtime.lock().unwrap().active.is_some() || state.store.lock().unwrap().pending.len() != 1 { return Err("游戏期间提醒没有延后".into()); }
        set_game_active(app, false);
        tick(app, 0.0, true);
        if state.runtime.lock().unwrap().active.is_none() { return Err("游戏退出后有效提示没有恢复".into()); }
        let active = get_care_delivery(care.clone())?.ok_or("没有恢复气泡")?;
        dismiss_care(care.clone(), active.id)?;
        let mut expired = item.clone(); expired.key = "smoke-expired".into(); expired.expires_at = Local::now().timestamp() - 1;
        state.store.lock().unwrap().pending.push(expired);
        tick(app, 0.0, true);
        if !state.store.lock().unwrap().pending.is_empty() || state.runtime.lock().unwrap().active.is_some() { return Err("过期提醒被补发".into()); }
        delete_care_rule(control, rule.id.clone())?;
        if !state.store.lock().unwrap().rules.is_empty() { return Err("删除规则没有生效".into()); }
        let mut fixture = Store { rules: vec![Rule { id: "dedup".into(), revision: 1, kind: "custom".into(), repeat: "once".into(), once_at: (now - CalendarDuration::minutes(1)).format("%Y-%m-%dT%H:%M").to_string(), text: "fixture".into(), ..Rule::default() }], ..Store::default() };
        collect(&mut fixture, now, 0.0);
        let mut restored: Store = serde_json::from_slice(&serde_json::to_vec(&fixture).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        collect(&mut restored, now, 0.0);
        if restored.pending.len() != 1 || restored.seen.len() != 1 { return Err("重启状态去重失败".into()); }
        Ok(serde_json::json!({"persistence":true,"previewNoTask":true,"bubbleReady":true,"noFocus":true,"dismiss":true,"snooze":true,
            "disableCancels":true,"delete":true,"gameDeferred":true,"gameResumeSummary":true,"expiry":true,"restartDedup":true,"weatherNetworkUsed":false,"bubbleState":bubble_state,"afterDismiss":after_dismiss}))
    })();
    close_active(app);
    *state.store.lock().unwrap() = original_store.clone();
    *state.runtime.lock().unwrap() = Runtime { ready: app.get_webview_window("care").is_some(), active: None, shown_at: None, ..original_runtime };
    state.game.store(original_game, Ordering::Relaxed);
    let restore = save(app, &original_store);
    state.smoke_running.store(false, Ordering::Relaxed);
    restore?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
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
            let mut runtime = Runtime { active: Some(Delivery { id: "combined".into(), items: std::mem::take(&mut store.pending), sound: false, display_seconds: BUBBLE_SECONDS }),
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

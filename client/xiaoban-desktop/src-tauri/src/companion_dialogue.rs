//! Explicit, user-initiated companion conversations. Shares no proactive interruption budget.
pub(crate) mod core;
use self::core::{Corpus, Data, Reply};
use chrono::{Datelike, Local, Timelike};
use std::{fs, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex, OnceLock}, time::{Duration, Instant}};
use tauri::Manager;

#[derive(Default)]
pub struct DialogueState {
    data: Mutex<Option<Data>>,
    /// At most one model request across companions. A timed-out request retains this gate until it exits.
    model: Mutex<Option<(String, u64, Arc<AtomicBool>)>>,
    last_model: Mutex<Option<Instant>>,
}
fn corpus() -> &'static Corpus { static CORPUS: OnceLock<Corpus> = OnceLock::new(); CORPUS.get_or_init(Corpus::bundled) }
fn path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> { Ok(crate::storage::root(app)?.join("config/companion-dialogue.json")) }
fn edit<T>(app: &tauri::AppHandle, apply: impl FnOnce(&mut Data) -> Result<T, String>) -> Result<T, String> {
    let state = app.state::<DialogueState>();
    let mut data = state.data.lock().unwrap_or_else(|e| e.into_inner());
    if data.is_none() {
        let file = path(app)?;
        let loaded = match fs::metadata(&file) {
            Ok(meta) if meta.len() > 512 * 1024 => return Err("伙伴互动记录过大，请先备份修复".into()),
            Ok(_) => serde_json::from_slice::<Data>(&fs::read(file).map_err(|e| e.to_string())?).map_err(|_| "伙伴互动记录暂不可用，请先备份修复".to_string())?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Data { next_id: Local::now().timestamp_millis().max(1) as u64, ..Data::default() },
            Err(e) => return Err(e.to_string()),
        };
        if !loaded.valid() { return Err("伙伴互动记录校验失败，请先备份修复".into()); }
        *data = Some(loaded);
    }
    let mut next = data.as_ref().expect("loaded above").clone();
    let result = apply(&mut next)?;
    crate::library::save_json(&path(app)?, &next)?;
    *data = Some(next);
    Ok(result)
}
fn identity(window: &tauri::WebviewWindow) -> Result<(String, String, String), String> {
    if crate::pets::is_pet(window.label()) {
        let pet = crate::pets::own(window)?;
        if !pet.enabled { return Err("这个伙伴已隐藏".into()); }
        let variant = pet.pet_id.bytes().fold(0usize, |value, byte| value.wrapping_mul(31).wrapping_add(byte as usize));
        let tone = if pet.expression == "sleepy" { "quiet" } else { ["warm", "playful", "quiet", "proud"][variant % 4] };
        return Ok((format!("pet:{}", pet.id), pet.name.chars().take(32).collect(), tone.into()));
    }
    if window.label() == "companion-greeting" {
        let (key, name, _) = crate::proactivity::dialogue_identity(window.app_handle())?;
        let state = window.app_handle().state::<DialogueState>();
        let tone = state.data.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
            .and_then(|data| data.records.get(&key)).map(|record| record.personality.clone()).unwrap_or_else(|| "warm".into());
        return Ok((key, name, tone));
    }
    if matches!(window.label(), "control" | "assistant") { return Ok(("composer".into(), "小伴".into(), "warm".into())); }
    Err("这个窗口不能发起伙伴对话".into())
}
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if !command.starts_with("companion_dialogue_") { return None; }
    let known = ["companion_dialogue_click", "companion_dialogue_choose", "companion_dialogue_model", "companion_dialogue_continue", "companion_dialogue_close"].contains(&command);
    Some(known && (matches!(label, "control" | "assistant" | "companion-greeting") || crate::pets::is_pet(label)))
}
pub fn cancel(app: &tauri::AppHandle, key: &str) {
    if let Some((current, _, flag)) = app.state::<DialogueState>().model.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        if current == key { flag.store(true, Ordering::Relaxed); }
    }
}
fn interact(window: &tauri::WebviewWindow, selected: Option<&str>, id: Option<u64>) -> Result<Reply, String> {
    let (key, name, tone) = identity(window)?;
    let app = window.app_handle(); cancel(app, &key);
    let now = Local::now();
    let reply = edit(app, |data| data.interact(corpus(), &key, &name, &tone, now.timestamp(), now.hour(), now.weekday().number_from_monday() >= 6, selected, id))?;
    if crate::pets::is_pet(window.label()) { crate::proactivity::show_dialogue(app, window, key, reply.clone())?; }
    Ok(reply)
}
#[tauri::command]
pub fn companion_dialogue_click(window: tauri::WebviewWindow) -> Result<Reply, String> { interact(&window, None, None) }
#[tauri::command]
pub fn companion_dialogue_choose(window: tauri::WebviewWindow, id: u64, choice: String) -> Result<Reply, String> {
    if choice.len() > 32 { return Err("没有这个选项".into()); }
    interact(&window, Some(&choice), Some(id))
}

#[tauri::command]
pub async fn companion_dialogue_model(window: tauri::WebviewWindow, id: u64) -> Result<Option<Reply>, String> {
    let (key, _, _) = identity(&window)?;
    let app = window.app_handle().clone();
    let snapshot = edit(&app, |data| data.records.get(&key).filter(|r| r.current.as_ref().is_some_and(|v| v.id == id && !v.model)).cloned().ok_or("伙伴已在聊下一句".to_string()))?;
    let config = crate::ai_config::current(&app)?;
    if !config.is_complete() || crate::ai_catalogue::usable(&app, &config).is_err() { return Ok(None); }
    let state = app.state::<DialogueState>();
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut running = state.model.lock().unwrap_or_else(|e| e.into_inner());
        let mut last = state.last_model.lock().unwrap_or_else(|e| e.into_inner());
        if running.is_some() || last.is_some_and(|at| at.elapsed() < Duration::from_secs(6)) { return Ok(None); }
        *running = Some((key.clone(), id, cancel.clone())); *last = Some(Instant::now());
    }
    // The native provider is synchronous. Keep it off the UI thread and bound how long the bubble waits.
    let (tx, rx) = std::sync::mpsc::channel();
    let worker_app = app.clone(); let worker_key = key.clone(); let worker_cancel = cancel.clone();
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::chat::companion_line(&worker_app, config, &worker_key, &snapshot, &worker_cancel)
        })).ok().and_then(Result::ok);
        let result = if !worker_cancel.load(Ordering::Relaxed) {
            result.and_then(|text| edit(&worker_app, |data| Ok(if worker_cancel.load(Ordering::Relaxed) { None } else { data.replace(&worker_key, id, &text) })).ok().flatten())
        } else { None };
        *worker_app.state::<DialogueState>().model.lock().unwrap_or_else(|e| e.into_inner()) = None;
        let _ = tx.send(result);
    });
    tauri::async_runtime::spawn_blocking(move || {
        match rx.recv_timeout(Duration::from_secs(18)) {
            Ok(reply) => reply,
            Err(_) => { cancel.store(true, Ordering::Relaxed); None },
        }
    }).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn companion_dialogue_continue(window: tauri::WebviewWindow, id: u64) -> Result<String, String> {
    let (key, name, tone) = identity(&window)?;
    let app = window.app_handle(); cancel(app, &key);
    // Serialize handoffs with clicks, so a double click cannot create two sessions or duplicate the transcript.
    let session = edit(app, |data| {
        let record = data.records.get(&key).filter(|r| r.current.as_ref().is_some_and(|v| v.id == id)).ok_or("这句话已经过去了，再点一下伙伴吧")?;
        crate::chat::companion_session(app, &key, &name, &tone, id, &record.history)
    })?;
    crate::float_assistant::open_companion_session(app, session.clone())?;
    if window.label() == "companion-greeting" { crate::proactivity::close_dialogue(app); }
    Ok(session)
}
#[tauri::command]
pub fn companion_dialogue_close(window: tauri::WebviewWindow, id: u64) -> Result<(), String> {
    let (key, _, _) = identity(&window)?;
    let current = edit(window.app_handle(), |data| Ok(data.records.get(&key).and_then(|r| r.current.as_ref()).is_some_and(|r| r.id == id)))?;
    if !current { return Ok(()); }
    cancel(window.app_handle(), &key);
    if window.label() == "companion-greeting" { crate::proactivity::close_dialogue(window.app_handle()); }
    Ok(())
}

mod engine;
use serde::{Deserialize, Serialize};
use std::{sync::{Mutex, atomic::{AtomicBool, AtomicU64, Ordering}}, time::Duration};
use tauri::{Emitter, Manager};

#[derive(Default)]
pub struct AsrState {
    microphone_error: Mutex<String>,
    settings_io: Mutex<()>,
    prepare: Mutex<()>,
    generation: AtomicU64,
    engine_ready: AtomicBool,
    /// The model is being fetched or loaded in the background, without anyone asking at that moment.
    warming: AtomicBool,
    /// A page is asking for the engine right now: a download that nobody asked for steps aside so that page sees progress.
    urgent: AtomicBool,
    recognizer: Mutex<Option<engine::AsrProcess>>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Settings { enabled: bool }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status { enabled: bool, model_ready: bool, engine_ready: bool, warming: bool, microphone_error: String }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress<'a> { request_id: &'a str, phase: &'a str, file: &'a str, downloaded: u64, total: u64 }
fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> { Ok(crate::storage::root(app)?.join("config/asr.json")) }
fn settings(app: &tauri::AppHandle) -> Result<Settings, String> {
    match std::fs::read(settings_path(app)?) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| "语音识别设置无法读取".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(format!("语音识别设置读取失败：{e}")),
    }
}
fn check(app: &tauri::AppHandle, generation: u64) -> Result<(), String> {
    if app.state::<AsrState>().generation.load(Ordering::SeqCst) != generation { Err("语音识别已关闭".into()) } else { Ok(()) }
}
#[tauri::command]
pub fn asr_get_status(app: tauri::AppHandle) -> Result<Status, String> {
    let state = app.state::<AsrState>();
    let _io = state.settings_io.lock().unwrap_or_else(|e| e.into_inner());
    let microphone_error = state.microphone_error.lock().unwrap_or_else(|e| e.into_inner()).clone();
    Ok(Status { enabled: settings(&app)?.enabled, model_ready: engine::present(&crate::inference::directory(&app)?), engine_ready: state.engine_ready.load(Ordering::SeqCst), warming: state.warming.load(Ordering::SeqCst), microphone_error })
}
#[tauri::command]
pub fn asr_report_microphone(app: tauri::AppHandle, error: String) -> Result<(), String> {
    if error.len() > 4096 { return Err("麦克风状态过长".into()); }
    *app.state::<AsrState>().microphone_error.lock().unwrap_or_else(|e| e.into_inner()) = error;
    announce(&app);
    Ok(())
}
/// Tells every window what the speech engine is up to, so a page does not have to ask again.
fn announce(app: &tauri::AppHandle) {
    if let Ok(status) = asr_get_status(app.clone()) { let _ = app.emit("asr-state", status); }
}
/// Makes the engine ready: the model is fetched when it is missing, then loaded. With a `request_id` the progress is
/// reported to the page that asked; without one it is silent.
fn prepare_blocking(handle: &tauri::AppHandle, generation: u64, request_id: Option<&str>, locked: impl FnOnce()) -> Result<(), String> {
    let state = handle.state::<AsrState>();
    let _prepare = state.prepare.lock().unwrap_or_else(|e| e.into_inner());
    locked();
    check(handle, generation)?;
    if !settings(handle)?.enabled { return Err("请先开启语音识别".into()); }
    if state.engine_ready.load(Ordering::SeqCst) { return Ok(()); }
    let directory = crate::inference::directory(handle)?;
    engine::ensure_model(&directory, || check(handle, generation), |file, downloaded| {
        if let Some(request_id) = request_id {
            let _ = handle.emit_to("control", "asr-progress", Progress { request_id, phase: "downloading", file, downloaded, total: engine::TOTAL_BYTES });
        }
    })?;
    check(handle, generation)?;
    if let Some(request_id) = request_id {
        let _ = handle.emit_to("control", "asr-progress", Progress { request_id, phase: "loading", file: "", downloaded: engine::TOTAL_BYTES, total: engine::TOTAL_BYTES });
    }
    // Serialize model creation, decoding and disposal; never run two recognizers concurrently.
    let mut slot = state.recognizer.lock().unwrap_or_else(|e| e.into_inner());
    let recognizer = engine::load(&directory)?;
    let _io = state.settings_io.lock().unwrap_or_else(|e| e.into_inner());
    check(handle, generation)?;
    *slot = Some(recognizer);
    state.engine_ready.store(true, Ordering::SeqCst);
    Ok(())
}
/// For the smoke test: how long after a start the model is on the disk (speech off) or the engine is ready (speech on),
/// without anyone asking. `wait_seconds` bounds the wait; a first download needs far longer than a warm start.
pub async fn smoke(app: &tauri::AppHandle) -> serde_json::Value {
    let limit = std::env::var("XIAOBAN_ASR_SMOKE_SECONDS").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(30);
    let started = std::time::Instant::now();
    loop {
        let status = asr_get_status(app.clone());
        let ready = status.as_ref().is_ok_and(|s| if s.enabled { s.engine_ready } else { s.model_ready });
        if ready || started.elapsed() > Duration::from_secs(limit) {
            return match status {
                Ok(s) => serde_json::json!({ "enabled": s.enabled, "modelReady": s.model_ready, "engineReady": s.engine_ready, "seconds": started.elapsed().as_secs_f64() }),
                Err(error) => serde_json::json!({ "error": error }),
            };
        }
        let _ = tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(200))).await;
    }
}
/// What the background pass has left to do.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    /// Nothing: the engine is ready, or speech is off and the model is already on disk.
    Done,
    /// Speech is off, the model is not on disk yet: fetch it, so it is there when speech is turned on.
    Fetch,
    /// Speech is on: fetch the model if it is missing, then load the engine.
    Prepare,
}
fn next_step(enabled: bool, model_present: bool, engine_ready: bool) -> Step {
    match (enabled, model_present, engine_ready) {
        (_, _, true) => Step::Done,
        (true, _, false) => Step::Prepare,
        (false, false, false) => Step::Fetch,
        (false, true, false) => Step::Done,
    }
}
/// Downloads the model without loading it, for someone who has not turned speech on. It gives way at once if a page
/// asks for the engine in the meantime, and the page then downloads with its own progress.
fn fetch_blocking(handle: &tauri::AppHandle) -> Result<(), String> {
    let state = handle.state::<AsrState>();
    let _prepare = state.prepare.lock().unwrap_or_else(|e| e.into_inner());
    let directory = crate::inference::directory(handle)?;
    engine::ensure_model(&directory, || {
        if state.urgent.load(Ordering::SeqCst) { Err("有页面正在使用语音识别，后台下载让路".into()) } else { Ok(()) }
    }, |_, _| {})
}
/// Seconds to wait before each background attempt: the first right after start-up has settled, then slower and slower
/// while the network or the disk is the problem.
const BACKGROUND_ATTEMPTS: [u64; 5] = [3, 20, 60, 300, 900];
/// After a start, gets the speech model onto the disk for everyone, and loaded into the engine for those who have
/// speech on, so the first question is not held up by checking, downloading or loading it. Silent: a failure is
/// retried later, never shown.
pub fn start(app: &tauri::AppHandle) {
    let handle = app.clone();
    std::thread::spawn(move || warm_up(&handle, &BACKGROUND_ATTEMPTS));
}
fn warm_up(app: &tauri::AppHandle, waits: &[u64]) {
    for (attempt, wait) in waits.iter().enumerate() {
        std::thread::sleep(Duration::from_secs(*wait));
        let state = app.state::<AsrState>();
        let enabled = settings(app).map(|s| s.enabled).unwrap_or(false);
        let present = crate::inference::directory(app).map(|dir| engine::present(&dir)).unwrap_or(false);
        let step = next_step(enabled, present, state.engine_ready.load(Ordering::SeqCst));
        if step == Step::Done { return; }
        let generation = state.generation.load(Ordering::SeqCst);
        state.warming.store(true, Ordering::SeqCst);
        announce(app);
        let result = if step == Step::Fetch { fetch_blocking(app) } else { prepare_blocking(app, generation, None, || {}) };
        state.warming.store(false, Ordering::SeqCst);
        announce(app);
        match result {
            Ok(()) => {
                // Speech may have been turned on while the model was coming down: there is still the engine to load.
                if step == Step::Fetch && settings(app).map(|s| s.enabled).unwrap_or(false) { continue_with_engine(app); }
                return;
            }
            Err(error) => eprintln!("语音模型后台准备失败（第 {} 次）：{error}", attempt + 1),
        }
    }
}
fn continue_with_engine(app: &tauri::AppHandle) {
    let state = app.state::<AsrState>();
    let generation = state.generation.load(Ordering::SeqCst);
    state.warming.store(true, Ordering::SeqCst);
    announce(app);
    if let Err(error) = prepare_blocking(app, generation, None, || {}) { eprintln!("语音引擎后台加载失败：{error}"); }
    state.warming.store(false, Ordering::SeqCst);
    announce(app);
}
#[tauri::command]
pub async fn asr_set_enabled(app: tauri::AppHandle, enabled: bool) -> Result<Status, String> {
    {
        let state = app.state::<AsrState>();
        let _io = state.settings_io.lock().unwrap_or_else(|e| e.into_inner());
        crate::library::save_json(&settings_path(&app)?, &Settings { enabled })?;
        state.generation.fetch_add(1, Ordering::SeqCst);
        if !enabled { state.engine_ready.store(false, Ordering::SeqCst); }
    }
    // Publish before waiting for an in-flight decode to release the engine so
    // every window drops its microphone as soon as the saved switch is off.
    announce(&app);
    if !enabled {
        let handle = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            // A decode already in progress may finish, but its result is fenced by generation.
            let state = handle.state::<AsrState>();
            let mut recognizer = state.recognizer.lock().unwrap_or_else(|e| e.into_inner());
            if !settings(&handle).map(|s| s.enabled).unwrap_or(false) { recognizer.take(); }
        }).await.map_err(|_| "语音引擎释放失败")?;
    }
    asr_get_status(app)
}
#[tauri::command]
pub async fn asr_prepare(app: tauri::AppHandle, request_id: String) -> Result<Status, String> {
    if request_id.len() > 100 { return Err("语音请求标识无效".into()); }
    let generation = app.state::<AsrState>().generation.load(Ordering::SeqCst);
    let handle = app.clone();
    app.state::<AsrState>().urgent.store(true, Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AsrState>();
        let result = prepare_blocking(&handle, generation, Some(&request_id), || state.urgent.store(false, Ordering::SeqCst));
        announce(&handle);
        result
    }).await.map_err(|_| "语音引擎加载任务失败")??;
    asr_get_status(app)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everyone_gets_the_model_but_only_those_with_speech_on_get_the_engine() {
        // speech off, no model yet: fetch it, so it is there the day they turn speech on
        assert_eq!(next_step(false, false, false), Step::Fetch);
        // speech off, model on disk: nothing more to do, and no engine held in memory
        assert_eq!(next_step(false, true, false), Step::Done);
        // speech on: fetch if missing, then load
        assert_eq!(next_step(true, false, false), Step::Prepare);
        assert_eq!(next_step(true, true, false), Step::Prepare);
        // already running
        assert_eq!(next_step(true, true, true), Step::Done);
        assert_eq!(next_step(false, true, true), Step::Done);
    }
}

#[tauri::command]
pub async fn asr_transcribe(app: tauri::AppHandle, samples: Vec<f32>) -> Result<String, String> {
    engine::validate_audio(&samples)?;
    let generation = app.state::<AsrState>().generation.load(Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AsrState>();
        if !settings(&app)?.enabled { return Err("语音识别未开启".into()); }
        let mut slot = state.recognizer.lock().unwrap_or_else(|e| e.into_inner());
        check(&app, generation)?;
        let recognizer = slot.as_mut().ok_or("语音引擎尚未加载，请重试")?;
        let text = engine::transcribe(recognizer, &samples)?;
        check(&app, generation)?;
        Ok(text)
    }).await.map_err(|_| "语音识别任务失败")?
}


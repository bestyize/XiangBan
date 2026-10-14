mod engine;
use serde::{Deserialize, Serialize};
use sherpa_onnx::OfflineRecognizer;
use std::sync::{Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};
use tauri::{Emitter, Manager};

#[derive(Default)]
pub struct AsrState {
    settings_io: Mutex<()>,
    prepare: Mutex<()>,
    generation: AtomicU64,
    engine_ready: AtomicBool,
    recognizer: Mutex<Option<OfflineRecognizer>>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Settings { enabled: bool }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status { enabled: bool, model_ready: bool, engine_ready: bool }
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
    let _io = state.settings_io.lock().unwrap();
    Ok(Status { enabled: settings(&app)?.enabled, model_ready: engine::present(&crate::storage::root(&app)?.join("asr")), engine_ready: state.engine_ready.load(Ordering::SeqCst) })
}
#[tauri::command]
pub async fn asr_set_enabled(app: tauri::AppHandle, enabled: bool) -> Result<Status, String> {
    {
        let state = app.state::<AsrState>();
        let _io = state.settings_io.lock().unwrap();
        crate::library::save_json(&settings_path(&app)?, &Settings { enabled })?;
        state.generation.fetch_add(1, Ordering::SeqCst);
        if !enabled { state.engine_ready.store(false, Ordering::SeqCst); }
    }
    if !enabled {
        let handle = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            // A decode already in progress may finish, but its result is fenced by generation.
            let state = handle.state::<AsrState>();
            let mut recognizer = state.recognizer.lock().unwrap();
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
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AsrState>();
        let _prepare = state.prepare.lock().unwrap();
        check(&handle, generation)?;
        if !settings(&handle)?.enabled { return Err("请先开启语音识别".into()); }
        if state.engine_ready.load(Ordering::SeqCst) { return Ok(()); }
        let directory = crate::storage::root(&handle)?.join("asr");
        engine::ensure_model(&directory, || check(&handle, generation), |file, downloaded| {
            let _ = handle.emit_to("control", "asr-progress", Progress { request_id: &request_id, phase: "downloading", file, downloaded, total: engine::TOTAL_BYTES });
        })?;
        check(&handle, generation)?;
        let _ = handle.emit_to("control", "asr-progress", Progress { request_id: &request_id, phase: "loading", file: "", downloaded: engine::TOTAL_BYTES, total: engine::TOTAL_BYTES });
        // Serialize model creation, decoding and disposal; never run two recognizers concurrently.
        let mut slot = state.recognizer.lock().unwrap();
        let recognizer = engine::load(&directory)?;
        let _io = state.settings_io.lock().unwrap();
        check(&handle, generation)?;
        *slot = Some(recognizer);
        state.engine_ready.store(true, Ordering::SeqCst);
        Ok::<(), String>(())
    }).await.map_err(|_| "语音引擎加载任务失败")??;
    asr_get_status(app)
}
#[tauri::command]
pub async fn asr_transcribe(app: tauri::AppHandle, samples: Vec<f32>) -> Result<String, String> {
    engine::validate_audio(&samples)?;
    let generation = app.state::<AsrState>().generation.load(Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AsrState>();
        if !settings(&app)?.enabled { return Err("语音识别未开启".into()); }
        let slot = state.recognizer.lock().unwrap();
        check(&app, generation)?;
        let recognizer = slot.as_ref().ok_or("语音引擎尚未加载，请重试")?;
        let text = engine::transcribe(recognizer, &samples)?;
        check(&app, generation)?;
        Ok(text)
    }).await.map_err(|_| "语音识别任务失败")?
}

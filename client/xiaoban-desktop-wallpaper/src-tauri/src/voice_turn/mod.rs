//! Turn detection for voice input: Silero VAD ("is someone speaking") and Smart Turn v3 ("have they finished").
//! The xiaoban-asr runtime runs both in a second process (`--turn`) on the onnxruntime it already contains, so the
//! app links no onnxruntime of its own. When that runtime or its models are not here yet, the commands fail and the
//! page keeps judging pauses by loudness.
use crate::asr::engine::TurnProcess;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Default)]
pub struct VoiceTurnState { process: Mutex<Option<TurnProcess>> }

/// Runs `work` on the turn process, starting it first if needed. A failed request drops the process, so the next
/// call starts a fresh one.
fn with_process<T>(app: &tauri::AppHandle, work: impl FnOnce(&mut TurnProcess) -> Result<T, String>) -> Result<T, String> {
    let state = app.state::<VoiceTurnState>();
    let mut slot = state.process.lock().unwrap_or_else(|e| e.into_inner());
    if slot.is_none() { *slot = Some(TurnProcess::spawn(&crate::storage::root(app)?.join("asr"))?); }
    let result = work(slot.as_mut().expect("started above"));
    if result.is_err() { *slot = None; }
    result
}

/// Starts turn detection (once). `false` means it is unavailable and the page keeps loudness endpointing.
#[tauri::command]
pub async fn voice_turn_ready(app: tauri::AppHandle) -> bool {
    tauri::async_runtime::spawn_blocking(move || match with_process(&app, |_| Ok(())) {
        Ok(()) => true,
        Err(error) => { eprintln!("智能判停不可用：{error}"); false }
    }).await.unwrap_or(false)
}

/// Speech probabilities for the next 16 kHz samples (16-bit) of a listener; one per 512-sample window.
#[tauri::command]
pub async fn voice_vad(app: tauri::AppHandle, stream: String, pcm: Vec<i16>) -> Result<Vec<f32>, String> {
    if stream.is_empty() || stream.len() > 64 || pcm.len() > 32000 { return Err("语音数据无效".into()); }
    tauri::async_runtime::spawn_blocking(move || with_process(&app, |p| p.vad(&stream, &pcm)))
        .await.map_err(|_| "语音检测任务失败".to_string())?
}

/// Forgets a listener's VAD state (it stopped listening).
#[tauri::command]
pub async fn voice_vad_reset(app: tauri::AppHandle, stream: String) {
    let _ = tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<VoiceTurnState>();
        let mut slot = state.process.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(process) = slot.as_mut() { let _ = process.reset(&stream); }
    }).await;
}

/// The probability, 0..1, that the speaker has finished their turn, from up to 20 s of their audio (16-bit).
#[tauri::command]
pub async fn voice_turn_complete(app: tauri::AppHandle, pcm: Vec<i16>) -> Result<f32, String> {
    if pcm.len() < 1600 || pcm.len() > 16000 * 20 { return Err("语音数据无效".into()); }
    tauri::async_runtime::spawn_blocking(move || with_process(&app, |p| p.complete(&pcm)))
        .await.map_err(|_| "判停任务失败".to_string())?
}

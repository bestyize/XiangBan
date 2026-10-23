//! Turn detection for voice input: Silero VAD ("is someone speaking") and Smart Turn v3 ("have they finished"),
//! served by the inference engine's `turn` capability in a process of its own, so recognition never holds up
//! per-frame speech detection. The models are fetched with the recognition model. While the engine cannot serve
//! them the commands fail and the page judges pauses by loudness.
use crate::inference::{Engine, ModelFile};
use std::sync::Mutex;
use tauri::Manager;

/// Silero VAD v5 (MIT) and Smart Turn v3.2 CPU (BSD-2), as shipped in pipecat-ai 1.13.0, pinned by hash.
pub const TURN_FILES: [ModelFile; 2] = [
    ModelFile { name: "silero_vad.onnx", bytes: 2327524, sha256: "1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3" },
    ModelFile { name: "smart-turn-v3.2.onnx", bytes: 8679182, sha256: "2bb026316b14a660486a75b1733cd3fbab8c2fd0314dc9af7be49f8cca967e4f" },
];

pub struct TurnProcess(Engine);

impl TurnProcess {
    fn start(app: &tauri::AppHandle) -> Result<Self, String> {
        let directory = crate::inference::directory(app)?;
        if !crate::inference::present(&directory, &[&TURN_FILES[0], &TURN_FILES[1]]) { return Err("判停模型尚未下载".into()); }
        Engine::start(&directory, "turn").map(Self)
    }
    fn vad(&mut self, stream: &str, pcm: &[i16]) -> Result<Vec<f32>, String> {
        let response = self.0.request(&serde_json::json!({ "type": "vad", "stream": stream, "pcm": pcm }))?;
        Ok(response.get("values").and_then(|v| v.as_array()).map(|v| v.iter().filter_map(|p| p.as_f64()).map(|p| p as f32).collect()).unwrap_or_default())
    }
    fn reset(&mut self, stream: &str) -> Result<(), String> {
        self.0.request(&serde_json::json!({ "type": "vadReset", "stream": stream })).map(|_| ())
    }
    fn complete(&mut self, pcm: &[i16]) -> Result<f32, String> {
        let response = self.0.request(&serde_json::json!({ "type": "turn", "pcm": pcm }))?;
        response.get("value").and_then(|v| v.as_f64()).map(|v| v as f32).ok_or_else(|| "判停结果无效".to_string())
    }
}

#[derive(Default)]
pub struct VoiceTurnState { process: Mutex<Option<TurnProcess>> }

/// Runs `work` on the turn process, starting it first if needed. A failed request drops the process, so the next
/// call starts a fresh one.
fn with_process<T>(app: &tauri::AppHandle, work: impl FnOnce(&mut TurnProcess) -> Result<T, String>) -> Result<T, String> {
    let state = app.state::<VoiceTurnState>();
    let mut slot = state.process.lock().unwrap_or_else(|e| e.into_inner());
    if slot.is_none() { *slot = Some(TurnProcess::start(app)?); }
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

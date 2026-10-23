//! Turn detection for voice input, run natively: Silero VAD ("is someone speaking") and Smart Turn v3
//! ("have they finished"). Recognition itself stays with SenseVoice in the xiaoban-asr companion; these two
//! small models only decide when an utterance ends. Both ship as resources under `voice/`.
mod features;

use ort::{session::Session, value::Tensor};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
use tauri::Manager;

/// Silero works on 512-sample windows at 16 kHz, with 64 samples of context and a 2×1×128 recurrent state.
const WINDOW: usize = 512;
const CONTEXT: usize = 64;
const STATE: usize = 2 * 128;
/// At most this many listeners keep VAD state at once (one per window that is listening, normally one).
const MAX_STREAMS: usize = 8;

struct Models { silero: Mutex<Session>, turn: Mutex<Session> }

struct Stream { state: Vec<f32>, context: Vec<f32>, pending: Vec<f32>, used: u64 }

#[derive(Default)]
pub struct VoiceTurnState {
    models: Mutex<Option<std::sync::Arc<Models>>>,
    streams: Mutex<HashMap<String, Stream>>,
    tick: std::sync::atomic::AtomicU64,
}

fn model_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().resource_dir().map(|root| root.join("voice")).map_err(|e| format!("无法定位语音判停模型：{e}"))
}

fn session(path: PathBuf) -> Result<Session, String> {
    Session::builder()
        // Two threads halve Smart Turn's time; more add little and compete with recognition.
        .and_then(|b| b.with_intra_threads(2))
        .and_then(|b| b.commit_from_file(&path))
        .map_err(|e| format!("语音判停模型加载失败（{}）：{e}", path.display()))
}

fn models(app: &tauri::AppHandle) -> Result<std::sync::Arc<Models>, String> {
    let state = app.state::<VoiceTurnState>();
    let mut slot = state.models.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(models) = slot.as_ref() { return Ok(models.clone()); }
    let dir = model_dir(app)?;
    let loaded = std::sync::Arc::new(Models {
        silero: Mutex::new(session(dir.join("silero_vad.onnx"))?),
        turn: Mutex::new(session(dir.join("smart-turn-v3.2.onnx"))?),
    });
    *slot = Some(loaded.clone());
    Ok(loaded)
}

/// Runs Silero over whole windows of `stream`, returning one speech probability per window.
fn vad(models: &Models, stream: &mut Stream, samples: &[f32]) -> Result<Vec<f32>, String> {
    stream.pending.extend_from_slice(samples);
    let mut out = Vec::new();
    let mut session = models.silero.lock().unwrap_or_else(|e| e.into_inner());
    while stream.pending.len() >= WINDOW {
        let mut input = Vec::with_capacity(CONTEXT + WINDOW);
        input.extend_from_slice(&stream.context);
        input.extend(stream.pending.drain(..WINDOW));
        let mut run = || -> ort::Result<(f32, Vec<f32>)> {
            let outputs = session.run(ort::inputs![
                "input" => Tensor::from_array(([1usize, CONTEXT + WINDOW], input.clone()))?,
                "state" => Tensor::from_array(([2usize, 1, 128], stream.state.clone()))?,
                "sr" => Tensor::from_array(((), vec![16000i64]))?,
            ])?;
            let probability = outputs["output"].try_extract_tensor::<f32>()?.1[0];
            let state = outputs["stateN"].try_extract_tensor::<f32>()?.1.to_vec();
            Ok((probability, state))
        };
        let (probability, state) = run().map_err(|e| format!("语音检测失败：{e}"))?;
        stream.state = state;
        stream.context = input[input.len() - CONTEXT..].to_vec();
        out.push(probability);
    }
    Ok(out)
}

fn turn(models: &Models, audio: &[f32]) -> Result<f32, String> {
    let features = features::log_mel(&features::last_eight_seconds(audio));
    let mut session = models.turn.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = || -> ort::Result<f32> {
        let outputs = session.run(ort::inputs![
            "input_features" => Tensor::from_array(([1usize, features::N_MELS, features::FRAMES], features.clone()))?,
        ])?;
        Ok(outputs[0].try_extract_tensor::<f32>()?.1[0])
    };
    run().map_err(|e| format!("判停失败：{e}"))
}

/// Audio arrives as 16-bit samples to keep the IPC payload small.
fn samples(pcm: &[i16]) -> Vec<f32> { pcm.iter().map(|s| *s as f32 / 32768.0).collect() }

/// Loads both models (once). `false` means turn detection is unavailable and the page keeps loudness endpointing.
#[tauri::command]
pub async fn voice_turn_ready(app: tauri::AppHandle) -> bool {
    tauri::async_runtime::spawn_blocking(move || match models(&app) {
        Ok(_) => true,
        Err(error) => { eprintln!("{error}"); false }
    }).await.unwrap_or(false)
}

/// Speech probabilities for the next 16 kHz samples of a listener; one per 512-sample window.
#[tauri::command]
pub async fn voice_vad(app: tauri::AppHandle, stream: String, pcm: Vec<i16>) -> Result<Vec<f32>, String> {
    if stream.is_empty() || stream.len() > 64 || pcm.len() > 16000 * 2 { return Err("语音数据无效".into()); }
    tauri::async_runtime::spawn_blocking(move || {
        let models = models(&app)?;
        let state = app.state::<VoiceTurnState>();
        let used = state.tick.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut streams = state.streams.lock().unwrap_or_else(|e| e.into_inner());
        if !streams.contains_key(&stream) && streams.len() >= MAX_STREAMS {
            if let Some(oldest) = streams.iter().min_by_key(|(_, s)| s.used).map(|(k, _)| k.clone()) { streams.remove(&oldest); }
        }
        let entry = streams.entry(stream).or_insert_with(|| Stream { state: vec![0.0; STATE], context: vec![0.0; CONTEXT], pending: Vec::new(), used });
        entry.used = used;
        vad(&models, entry, &samples(&pcm))
    }).await.map_err(|_| "语音检测任务失败".to_string())?
}

/// Forgets a listener's VAD state (it stopped listening).
#[tauri::command]
pub fn voice_vad_reset(app: tauri::AppHandle, stream: String) {
    app.state::<VoiceTurnState>().streams.lock().unwrap_or_else(|e| e.into_inner()).remove(&stream);
}

/// The probability, 0..1, that the speaker has finished their turn, judged from the last 8 s of their audio.
#[tauri::command]
pub async fn voice_turn_complete(app: tauri::AppHandle, pcm: Vec<i16>) -> Result<f32, String> {
    if pcm.len() < 1600 || pcm.len() > 16000 * 20 { return Err("语音数据无效".into()); }
    tauri::async_runtime::spawn_blocking(move || turn(&*models(&app)?, &samples(&pcm)))
        .await.map_err(|_| "判停任务失败".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled(name: &str) -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/voice").join(name) }
    fn loaded() -> Option<Models> {
        // Needs onnxruntime at test time; skipped where it was not linked for this platform.
        Some(Models { silero: Mutex::new(session(bundled("silero_vad.onnx")).ok()?), turn: Mutex::new(session(bundled("smart-turn-v3.2.onnx")).ok()?) })
    }
    fn tone(seconds: f32, amplitude: f32) -> Vec<f32> {
        (0..(seconds * 16000.0) as usize).map(|i| amplitude * (2.0 * std::f32::consts::PI * 220.0 * i as f32 / 16000.0).sin()).collect()
    }

    #[test]
    #[ignore = "needs onnxruntime; run with ORT_LIB_LOCATION set"]
    fn silero_reports_silence_as_not_speech() {
        let Some(models) = loaded() else { return };
        let mut stream = Stream { state: vec![0.0; STATE], context: vec![0.0; CONTEXT], pending: Vec::new(), used: 0 };
        let quiet = vad(&models, &mut stream, &vec![0.0; 16000]).unwrap();
        assert_eq!(quiet.len(), 16000 / WINDOW);
        assert!(quiet.iter().all(|p| *p < 0.5));
        assert_eq!(stream.pending.len(), 16000 % WINDOW);
    }

    #[test]
    #[ignore = "needs onnxruntime; run with ORT_LIB_LOCATION set"]
    fn smart_turn_gives_a_probability_quickly() {
        let Some(models) = loaded() else { return };
        let _ = turn(&models, &tone(1.0, 0.3)).unwrap();
        let started = std::time::Instant::now();
        let p = turn(&models, &tone(3.0, 0.3)).unwrap();
        assert!((0.0..=1.0).contains(&p));
        eprintln!("smart turn: {p:.4} in {:?}", started.elapsed());
    }

    #[test]
    fn decodes_16_bit_audio() {
        assert_eq!(samples(&[0, 16384, -32768]), vec![0.0, 0.5, -1.0]);
    }
}

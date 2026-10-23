//! Turn detection (`--turn` mode): Silero VAD ("is someone speaking") and Smart Turn v3 ("have they finished").
//! Both run on the onnxruntime that sherpa-onnx already links into this program, through `ort`'s
//! alternative-backend mode, so there is one onnxruntime and nothing extra to ship.
use crate::features;
use ort::{session::Session, value::Tensor};
use std::{collections::HashMap, path::Path};

/// Silero works on 512-sample windows at 16 kHz, with 64 samples of context and a 2×1×128 recurrent state.
pub const WINDOW: usize = 512;
const CONTEXT: usize = 64;
const STATE: usize = 2 * 128;
/// At most this many listeners keep VAD state at once (normally one per window that is listening).
const MAX_STREAMS: usize = 8;
pub const SILERO_FILE: &str = "silero_vad.onnx";
pub const SMART_TURN_FILE: &str = "smart-turn-v3.2.onnx";

/// Points `ort` at the onnxruntime linked in through sherpa-onnx. Must run before any session is created.
pub fn use_linked_onnxruntime() -> Result<(), String> {
    unsafe {
        let base = ort::sys::OrtGetApiBase();
        if base.is_null() { return Err("onnxruntime 不可用".into()); }
        let api = ((*base).GetApi)(ort::sys::ORT_API_VERSION);
        if api.is_null() { return Err("onnxruntime 版本过旧".into()); }
        ort::set_api(*api);
    }
    Ok(())
}

struct Stream { state: Vec<f32>, context: Vec<f32>, pending: Vec<f32>, used: u64 }

pub struct Turn { silero: Session, smart_turn: Session, streams: HashMap<String, Stream>, tick: u64 }

fn session(path: &Path) -> Result<Session, String> {
    if !path.is_file() { return Err(format!("找不到判停模型：{}", path.display())); }
    Session::builder()
        // Two threads halve Smart Turn's time; more add little and compete with recognition.
        .and_then(|b| b.with_intra_threads(2))
        .and_then(|b| b.commit_from_file(path))
        .map_err(|e| format!("判停模型加载失败（{}）：{e}", path.display()))
}

impl Turn {
    pub fn load(directory: &Path) -> Result<Self, String> {
        Ok(Self { silero: session(&directory.join(SILERO_FILE))?, smart_turn: session(&directory.join(SMART_TURN_FILE))?, streams: HashMap::new(), tick: 0 })
    }

    /// Speech probabilities for the next 16 kHz samples of a listener; one per 512-sample window.
    pub fn vad(&mut self, stream: &str, samples: &[f32]) -> Result<Vec<f32>, String> {
        self.tick += 1;
        if !self.streams.contains_key(stream) && self.streams.len() >= MAX_STREAMS {
            if let Some(oldest) = self.streams.iter().min_by_key(|(_, s)| s.used).map(|(k, _)| k.clone()) { self.streams.remove(&oldest); }
        }
        let tick = self.tick;
        let entry = self.streams.entry(stream.to_string()).or_insert_with(|| Stream { state: vec![0.0; STATE], context: vec![0.0; CONTEXT], pending: Vec::new(), used: tick });
        entry.used = tick;
        entry.pending.extend_from_slice(samples);
        let mut out = Vec::new();
        while entry.pending.len() >= WINDOW {
            let mut input = Vec::with_capacity(CONTEXT + WINDOW);
            input.extend_from_slice(&entry.context);
            input.extend(entry.pending.drain(..WINDOW));
            let mut run = || -> ort::Result<(f32, Vec<f32>)> {
                let outputs = self.silero.run(ort::inputs![
                    "input" => Tensor::from_array(([1usize, CONTEXT + WINDOW], input.clone()))?,
                    "state" => Tensor::from_array(([2usize, 1, 128], entry.state.clone()))?,
                    "sr" => Tensor::from_array(((), vec![16000i64]))?,
                ])?;
                let probability = outputs["output"].try_extract_tensor::<f32>()?.1[0];
                let state = outputs["stateN"].try_extract_tensor::<f32>()?.1.to_vec();
                Ok((probability, state))
            };
            let (probability, state) = run().map_err(|e| format!("语音检测失败：{e}"))?;
            entry.state = state;
            entry.context = input[input.len() - CONTEXT..].to_vec();
            out.push(probability);
        }
        Ok(out)
    }

    pub fn reset(&mut self, stream: &str) { self.streams.remove(stream); }

    /// The probability, 0..1, that the speaker has finished their turn, judged from the last 8 s of audio.
    pub fn complete(&mut self, audio: &[f32]) -> Result<f32, String> {
        let input = features::log_mel(&features::last_eight_seconds(audio));
        let mut run = || -> ort::Result<f32> {
            let outputs = self.smart_turn.run(ort::inputs![
                "input_features" => Tensor::from_array(([1usize, features::N_MELS, features::FRAMES], input.clone()))?,
            ])?;
            Ok(outputs[0].try_extract_tensor::<f32>()?.1[0])
        };
        run().map_err(|e| format!("判停失败：{e}"))
    }
}

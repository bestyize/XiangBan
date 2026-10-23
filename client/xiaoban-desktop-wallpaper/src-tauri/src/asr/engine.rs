//! Speech recognition with SenseVoice Small (INT8), served by the inference engine's `asr` capability.
use crate::inference::{self, Engine, ModelFile};
use std::path::Path;

/// ModelScope's sherpa-compatible export of iic/SenseVoiceSmall, pinned by hash.
pub const FILES: [ModelFile; 2] = [
    ModelFile { name: "model.int8.onnx", bytes: 239233841, sha256: "c71f0ce00bec95b07744e116345e33d8cbbe08cef896382cf907bf4b51a2cd51" },
    ModelFile { name: "tokens.txt", bytes: 315894, sha256: "f449eb28dc567533d7fa59be34e2abca8784f771850c78a47fb731a31429a1dc" },
];
pub const MAX_SAMPLES: usize = 20 * 16000;

/// Everything voice input needs: recognition and turn detection are fetched together.
pub fn voice_files() -> [&'static ModelFile; 4] {
    let turn = &crate::voice_turn::TURN_FILES;
    [&FILES[0], &FILES[1], &turn[0], &turn[1]]
}
pub const TOTAL_BYTES: u64 = FILES[0].bytes + FILES[1].bytes + crate::voice_turn::TURN_FILES[0].bytes + crate::voice_turn::TURN_FILES[1].bytes;

pub fn present(directory: &Path) -> bool { inference::present(directory, &voice_files()) }

pub fn ensure_model(directory: &Path, check: impl Fn() -> Result<(), String>, progress: impl Fn(&str, u64)) -> Result<(), String> {
    inference::ensure(directory, &voice_files(), check, progress)
}

pub struct AsrProcess(Engine);

impl AsrProcess {
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String, String> {
        validate_audio(samples)?;
        let response = self.0.request(&serde_json::json!({ "type": "transcribe", "samples": samples }))?;
        Ok(response.get("text").and_then(|t| t.as_str()).unwrap_or_default().to_string())
    }
}

pub fn load(directory: &Path) -> Result<AsrProcess, String> {
    Engine::start(directory, "asr").map(AsrProcess)
}

pub fn validate_audio(samples: &[f32]) -> Result<(), String> {
    if samples.len() < 1600 || samples.len() > MAX_SAMPLES || samples.iter().any(|s| !s.is_finite() || s.abs() > 1.01) {
        return Err("语音数据无效，请使用 0.1–20 秒、16kHz 单声道 PCM 音频".into());
    }
    Ok(())
}

pub fn transcribe(recognizer: &mut AsrProcess, samples: &[f32]) -> Result<String, String> {
    recognizer.transcribe(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_invalid_pcm() {
        assert!(validate_audio(&vec![0.1; 1600]).is_ok());
        assert!(validate_audio(&vec![0.0; MAX_SAMPLES + 1]).is_err());
        assert!(validate_audio(&[f32::NAN; 1600]).is_err());
        assert!(validate_audio(&[f32::INFINITY; 1600]).is_err());
        assert!(validate_audio(&[2.0; 1600]).is_err());
        assert!(validate_audio(&[]).is_err());
    }
}

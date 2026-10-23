//! `asr`: speech recognition with SenseVoice Small (INT8) through sherpa-onnx.
use crate::protocol::{Request, Response, MAX_SAMPLES};
use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig};
use std::path::Path;

pub struct Asr { recognizer: OfflineRecognizer }

impl Asr {
    pub fn load(directory: &Path) -> Result<Self, String> {
        let model_path = directory.join("model.int8.onnx");
        let tokens_path = directory.join("tokens.txt");
        if !model_path.is_file() { return Err(format!("找不到模型文件：{}", model_path.display())); }
        if !tokens_path.is_file() { return Err(format!("找不到词表文件：{}", tokens_path.display())); }
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
            model: Some(model_path.to_str().ok_or("模型路径编码错误")?.into()),
            language: Some("auto".into()),
            use_itn: true,
        };
        config.model_config.tokens = Some(tokens_path.to_str().ok_or("词表路径编码错误")?.into());
        config.model_config.num_threads = 2;
        config.model_config.provider = Some("cpu".into());
        let recognizer = OfflineRecognizer::create(&config).ok_or("SenseVoice 引擎初始化失败")?;
        Ok(Self { recognizer })
    }

    pub fn handle(&mut self, request: Request) -> Response {
        let Request::Transcribe { samples } = request else { return Response::error("语音识别进程不支持该请求") };
        if let Err(e) = validate_audio(&samples) { return Response::error(e); }
        let stream = self.recognizer.create_stream();
        stream.accept_waveform(16000, &samples);
        self.recognizer.decode(&stream);
        match stream.get_result() {
            Some(result) => Response::Ok { text: clean_text(&result.text) },
            None => Response::error("语音识别未返回结果"),
        }
    }
}

fn validate_audio(samples: &[f32]) -> Result<(), String> {
    if samples.len() < 1600 || samples.len() > MAX_SAMPLES || samples.iter().any(|s| !s.is_finite() || s.abs() > 1.01) {
        return Err("语音数据无效，请使用 0.1–20 秒、16kHz 单声道 PCM 音频".into());
    }
    Ok(())
}

/// Drops SenseVoice's `<|zh|><|NEUTRAL|>…` tags.
fn clean_text(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::new();
    while let Some(start) = remaining.find("<|") {
        output.push_str(&remaining[..start]);
        match remaining[start + 2..].find("|>") {
            Some(end) => remaining = &remaining[start + 2 + end + 2..],
            None => { remaining = &remaining[start..]; break; }
        }
    }
    output.push_str(remaining);
    output.trim().into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn strips_sense_voice_tags() {
        assert_eq!(super::clean_text("<|zh|><|NEUTRAL|><|Speech|><|withitn|>你好。"), "你好。");
        assert_eq!(super::clean_text("没有标签"), "没有标签");
    }
}

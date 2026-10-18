use serde::{Deserialize, Serialize};
use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig};
use std::{
    env,
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

const MAX_SAMPLES: usize = 20 * 16000;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Request {
    Ping,
    Transcribe { samples: Vec<f32> },
    Exit,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum Response {
    Ready,
    Pong,
    Ok { text: String },
    Error { error: String },
}

fn validate_audio(samples: &[f32]) -> Result<(), String> {
    if samples.len() < 1600 || samples.len() > MAX_SAMPLES || samples.iter().any(|s| !s.is_finite() || s.abs() > 1.01) {
        return Err("语音数据无效，请使用 0.1–20 秒、16kHz 单声道 PCM 音频".into());
    }
    Ok(())
}

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

fn load_recognizer(directory: &Path) -> Result<OfflineRecognizer, String> {
    let model_path = directory.join("model.int8.onnx");
    let tokens_path = directory.join("tokens.txt");

    if !model_path.is_file() {
        return Err(format!("找不到模型文件：{}", model_path.display()));
    }
    if !tokens_path.is_file() {
        return Err(format!("找不到词表文件：{}", tokens_path.display()));
    }

    let mut config = OfflineRecognizerConfig::default();
    config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
        model: Some(model_path.to_str().ok_or("模型路径编码错误")?.into()),
        language: Some("auto".into()),
        use_itn: true,
    };
    config.model_config.tokens = Some(tokens_path.to_str().ok_or("词表路径编码错误")?.into());
    config.model_config.num_threads = 2;
    config.model_config.provider = Some("cpu".into());

    OfflineRecognizer::create(&config).ok_or_else(|| "SenseVoice 引擎初始化失败".into())
}

fn send_response(resp: &Response) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if let Ok(line) = serde_json::to_string(resp) {
        let _ = writeln!(handle, "{}", line);
        let _ = handle.flush();
    }
}

fn resolve_model_dir() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        if (args[i] == "--dir" || args[i] == "-d") && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
    }
    // Default to exe directory
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join("model.int8.onnx").is_file() {
                return parent.to_path_buf();
            }
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() {
    let model_dir = resolve_model_dir();
    let recognizer = match load_recognizer(&model_dir) {
        Ok(r) => r,
        Err(e) => {
            send_response(&Response::Error { error: e });
            std::process::exit(1);
        }
    };

    send_response(&Response::Ready);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: Request = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                send_response(&Response::Error { error: format!("无效请求 JSON：{e}") });
                continue;
            }
        };

        match req {
            Request::Ping => {
                send_response(&Response::Pong);
            }
            Request::Exit => {
                break;
            }
            Request::Transcribe { samples } => {
                if let Err(e) = validate_audio(&samples) {
                    send_response(&Response::Error { error: e });
                    continue;
                }
                let stream = recognizer.create_stream();
                stream.accept_waveform(16000, &samples);
                recognizer.decode(&stream);
                let result = stream.get_result();
                match result {
                    Some(res) => {
                        let text = clean_text(&res.text);
                        send_response(&Response::Ok { text });
                    }
                    None => {
                        send_response(&Response::Error { error: "语音识别未返回结果".into() });
                    }
                }
            }
        }
    }
}

mod features;
mod turn;

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
    /// `--turn` mode: speech probabilities for the next 16 kHz samples (16-bit PCM) of a listener.
    Vad { stream: String, pcm: Vec<i16> },
    /// `--turn` mode: forget a listener's VAD state.
    VadReset { stream: String },
    /// `--turn` mode: probability that the speaker finished, from up to 20 s of 16-bit PCM.
    Turn { pcm: Vec<i16> },
    Exit,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum Response {
    /// `version` and `features` let the app tell this build from older ones, which sent a bare `ready`.
    Ready { version: u32, features: Vec<&'static str> },
    Pong,
    Ok { text: String },
    Probabilities { values: Vec<f32> },
    Probability { value: f32 },
    Error { error: String },
}

/// Bumped whenever the app needs to know a request is understood.
const PROTOCOL_VERSION: u32 = 2;

fn pcm(samples: &[i16]) -> Vec<f32> { samples.iter().map(|s| *s as f32 / 32768.0).collect() }

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

fn turn_mode() -> bool { env::args().any(|a| a == "--turn") }

/// Turn detection only: loaded in its own process so recognition never holds up per-frame VAD.
fn serve_turn(model_dir: &Path) {
    let mut detector = match turn::use_linked_onnxruntime().and_then(|_| turn::Turn::load(model_dir)) {
        Ok(detector) => detector,
        Err(error) => { send_response(&Response::Error { error }); std::process::exit(1); }
    };
    send_response(&Response::Ready { version: PROTOCOL_VERSION, features: vec!["turn"] });
    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() { continue; }
        let response = match serde_json::from_str::<Request>(line.trim()) {
            Err(e) => Response::Error { error: format!("无效请求 JSON：{e}") },
            Ok(Request::Exit) => break,
            Ok(Request::Ping) => Response::Pong,
            Ok(Request::Vad { stream, pcm: samples }) if !stream.is_empty() && stream.len() <= 64 && samples.len() <= 32000 =>
                detector.vad(&stream, &pcm(&samples)).map(|values| Response::Probabilities { values }).unwrap_or_else(|error| Response::Error { error }),
            Ok(Request::VadReset { stream }) => { detector.reset(&stream); Response::Pong }
            Ok(Request::Turn { pcm: samples }) if (1600..=MAX_SAMPLES).contains(&samples.len()) =>
                detector.complete(&pcm(&samples)).map(|value| Response::Probability { value }).unwrap_or_else(|error| Response::Error { error }),
            Ok(_) => Response::Error { error: "这个进程不支持该请求".into() },
        };
        send_response(&response);
    }
}

fn main() {
    let model_dir = resolve_model_dir();
    if turn_mode() { serve_turn(&model_dir); return; }
    let recognizer = match load_recognizer(&model_dir) {
        Ok(r) => r,
        Err(e) => {
            send_response(&Response::Error { error: e });
            std::process::exit(1);
        }
    };

    send_response(&Response::Ready { version: PROTOCOL_VERSION, features: vec!["transcribe"] });

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
            Request::Vad { .. } | Request::VadReset { .. } | Request::Turn { .. } => {
                send_response(&Response::Error { error: "判停请使用 --turn 进程".into() });
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

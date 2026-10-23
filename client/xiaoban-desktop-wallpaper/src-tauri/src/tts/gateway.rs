//! Speech from the 小伴 AI gateway (`server-ai`, `POST /v1/audio/speech`). The gateway picks the provider and
//! voice and bills the signed-in account; the app never holds a provider key.
use std::time::Duration;

const MAX_AUDIO: u64 = 8 * 1024 * 1024;

pub fn speak(app: &tauri::AppHandle, text: &str, rate: i32) -> Result<Vec<u8>, String> {
    let token = tauri::async_runtime::block_on(crate::account::fresh_access_token(app)).map_err(|f| f.message)?;
    let client = crate::network::blocking_builder(reqwest::blocking::Client::builder().connect_timeout(Duration::from_secs(8)).timeout(Duration::from_secs(30)))?.build().map_err(|_| "无法连接小伴 AI 语音")?;
    let speed = (1.0 + rate as f64 / 100.0).clamp(0.5, 2.0);
    let response = client.post(format!("{}/v1/audio/speech", crate::ai_gateway::ai_base_url()))
        .bearer_auth(token)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(serde_json::json!({ "input": text, "speed": speed, "response_format": "mp3" }).to_string())
        .send().map_err(|_| "小伴 AI 语音连接失败")?;
    if !response.status().is_success() {
        let body = response.text().unwrap_or_default();
        return Err(crate::ai_gateway::readable_error(&body));
    }
    if response.content_length().is_some_and(|n| n > MAX_AUDIO) { return Err("语音过长".into()); }
    let bytes = response.bytes().map_err(|_| "小伴 AI 语音读取失败")?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_AUDIO { return Err("小伴 AI 语音没有返回音频".into()); }
    Ok(bytes.to_vec())
}

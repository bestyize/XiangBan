pub(crate) mod drm;
mod gateway;
mod protocol;
mod system;
use protocol::{Options, HOST, MAX_AUDIO};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, net::{TcpStream, ToSocketAddrs}, sync::{Mutex, atomic::{AtomicU64, Ordering}}, time::{Duration, Instant}};
use tauri::{Emitter, Manager};
use tungstenite::{client::IntoClientRequest, stream::MaybeTlsStream, Message};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub auto_read: bool,
    /// Where speech comes from: `edge` (default), `gateway` (小伴 AI) or `system`. Online voices fall back to
    /// the system voice when they fail.
    pub provider: String,
    #[serde(flatten)] pub options: Options,
}
#[derive(Default)]
pub struct TtsState {
    io: Mutex<()>,
    generation: AtomicU64,
    cache: Mutex<VecDeque<(String, Vec<u8>)>>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Voice { pub short_name: String, pub gender: String, pub locale: String, pub friendly_name: String }
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VoiceDto { short_name: String, gender: String, locale: String, friendly_name: String }
const PROVIDERS: &[&str] = &["edge", "gateway", "system"];
fn path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> { Ok(crate::storage::root(app)?.join("config/tts.json")) }
fn now() -> i64 { chrono::Utc::now().timestamp() }
#[tauri::command]
pub fn tts_get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let state = app.state::<TtsState>(); let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    match std::fs::read(path(&app)?) {
        Ok(bytes) => {
            let mut value: Settings = serde_json::from_slice(&bytes).map_err(|_| "语音设置无法读取，请重新保存")?;
            value.options.validate()?;
            if !PROVIDERS.contains(&value.provider.as_str()) { value.provider = "edge".into(); }
            Ok(value)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings { provider: "edge".into(), ..Settings::default() }),
        Err(_) => Err("语音设置读取失败".into()),
    }
}
#[tauri::command]
pub fn tts_save_settings(app: tauri::AppHandle, mut settings: Settings) -> Result<Settings, String> {
    settings.options.validate()?;
    if settings.provider.is_empty() { settings.provider = "edge".into(); }
    if !PROVIDERS.contains(&settings.provider.as_str()) { return Err("没有这个语音来源".into()); }
    let state = app.state::<TtsState>(); let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    crate::library::save_json(&path(&app)?, &settings)?;
    let _ = app.emit_to("control", "tts-settings-changed", &settings);
    Ok(settings)
}
#[tauri::command]
pub fn tts_cancel(app: tauri::AppHandle) { app.state::<TtsState>().generation.fetch_add(1, Ordering::SeqCst); }
fn cancelled(app: &tauri::AppHandle, id: u64) -> Result<(), String> {
    if app.state::<TtsState>().generation.load(Ordering::SeqCst) != id { Err("朗读已停止".into()) } else { Ok(()) }
}
#[tauri::command]
pub async fn tts_list_voices() -> Result<Vec<Voice>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let client = crate::network::blocking_builder(reqwest::blocking::Client::builder().timeout(Duration::from_secs(15)))?.build().map_err(|_| "无法创建语音连接")?;
        let response = client.get(protocol::voices_url(now())).header("User-Agent", protocol::USER_AGENT)
            .header("Accept-Language", "en-US,en;q=0.9").header("Accept", "*/*")
            .header("Cookie", format!("muid={};", protocol::random_id()?))
            .send().map_err(|_| "音色列表拉取失败，请检查网络")?;
        if !response.status().is_success() { return Err("音色服务暂不可用，请稍后重试".into()); }
        let text = response.text().map_err(|_| "音色列表读取失败")?;
        if text.len() > 2 * 1024 * 1024 { return Err("音色列表过大".into()); }
        let voices: Vec<VoiceDto> = serde_json::from_str(&text).map_err(|_| "音色列表格式不支持")?;
        Ok(voices.into_iter().map(|v| Voice { short_name:v.short_name,gender:v.gender,locale:v.locale,friendly_name:v.friendly_name }).collect())
    }).await.map_err(|_| "音色列表任务失败")?
}
/// Speech from the chosen provider; an online voice that fails is replaced by the system voice, so reading
/// carries on (the cached audio of either is reused).
fn synthesize(app: &tauri::AppHandle, id: u64, text: &str, options: &Options) -> Result<Vec<u8>, String> {
    options.validate()?;
    let provider = tts_get_settings(app.clone()).map(|s| s.provider).unwrap_or_default();
    let online = match provider.as_str() {
        "system" => Err(String::new()),
        "gateway" => cached(app, &("gateway", text, options.rate), || gateway::speak(app, text, options.rate)),
        _ => synthesize_edge(app, id, text, options),
    };
    cancelled(app, id)?;
    match online {
        Ok(audio) => Ok(audio),
        Err(error) => cached(app, &("system", text, options.rate), || system::speak(text, options.rate))
            .map_err(|fallback| if error.is_empty() { fallback } else { error }),
    }
}

pub(crate) fn synthesize_raw(app: &tauri::AppHandle, text: &str, options: Option<&Options>) -> Result<Vec<u8>, String> {
    let default_options = tts_get_settings(app.clone()).map(|s| s.options).unwrap_or_default();
    let opts = options.unwrap_or(&default_options);
    let id = app.state::<TtsState>().generation.load(Ordering::SeqCst);
    synthesize(app, id, text, opts)
}
fn cached(app: &tauri::AppHandle, key: &impl serde::Serialize, make: impl FnOnce() -> Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {
    let key = format!("{:x}", Sha256::digest(serde_json::to_vec(key).map_err(|_| "语音参数无效")?));
    if let Some((_, audio)) = app.state::<TtsState>().cache.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(k, _)| *k == key) { return Ok(audio.clone()); }
    let audio = make()?;
    remember(app, key, &audio);
    Ok(audio)
}
fn remember(app: &tauri::AppHandle, key: String, audio: &[u8]) {
    let state = app.state::<TtsState>(); let mut cache = state.cache.lock().unwrap_or_else(|e| e.into_inner());
    if audio.len() <= 8 * 1024 * 1024 {
        cache.push_back((key, audio.to_vec()));
        while cache.len() > 4 || cache.iter().map(|(_, v)| v.len()).sum::<usize>() > 8 * 1024 * 1024 { cache.pop_front(); }
    }
}
fn synthesize_edge(app: &tauri::AppHandle, id: u64, text: &str, options: &Options) -> Result<Vec<u8>, String> {
    let parts = protocol::chunks(text)?;
    let key = format!("{:x}", Sha256::digest(serde_json::to_vec(&(text, options)).map_err(|_| "语音参数无效")?));
    cancelled(app, id)?;
    if let Some((_, audio)) = app.state::<TtsState>().cache.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(k, _)| *k == key) { return Ok(audio.clone()); }
    let mut output = Vec::new();
    for text in parts {
        cancelled(app, id)?;
        let connection = protocol::random_id()?;
        let mut request = protocol::ws_url(&connection, now()).into_client_request().map_err(|_| "语音连接地址无效")?;
        for (name, value) in [("User-Agent", protocol::USER_AGENT.to_string()), ("Origin", protocol::ORIGIN.to_string()),
            ("Accept", "*/*".into()), ("Accept-Language", "en-US,en;q=0.9".into()),
            ("Pragma", "no-cache".into()), ("Cache-Control", "no-cache".into()), ("Cookie", format!("muid={};", protocol::random_id()?))] {
            request.headers_mut().insert(tungstenite::http::HeaderName::from_bytes(name.as_bytes()).map_err(|_| "语音请求头无效")?,
                value.parse().map_err(|_| "语音请求头无效")?);
        }
        let addresses = (HOST, 443).to_socket_addrs().map_err(|_| "语音服务域名解析失败")?;
        let mut stream = None;
        for address in addresses.take(3) {
            cancelled(app, id)?;
            if let Ok(tcp) = TcpStream::connect_timeout(&address, Duration::from_secs(5)) { stream = Some(tcp); break; }
        }
        let tcp = stream.ok_or("无法连接语音服务，请检查网络")?;
        tcp.set_read_timeout(Some(Duration::from_secs(15))).map_err(|_| "语音连接设置失败")?;
        tcp.set_write_timeout(Some(Duration::from_secs(15))).map_err(|_| "语音连接设置失败")?;
        let (mut socket, _) = tungstenite::client_tls_with_config(request, tcp, None, None).map_err(|e| {
            eprintln!("[TTS] WebSocket handshake failed: {e}");
            "语音服务握手失败，请稍后重试"
        })?;
        match socket.get_mut() {
            MaybeTlsStream::NativeTls(tls) => tls.get_mut().set_read_timeout(Some(Duration::from_secs(2))),
            MaybeTlsStream::Plain(tcp) => tcp.set_read_timeout(Some(Duration::from_secs(2))),
            _ => return Err("当前语音 TLS 连接不支持".into()),
        }.map_err(|_| "语音连接设置失败")?;
        socket.send(Message::Text(protocol::config(now()).into())).map_err(|_| "语音请求发送失败")?;
        socket.send(Message::Text(protocol::ssml(&protocol::random_id()?, options, &text, now()).into())).map_err(|_| "语音文本发送失败")?;
        let deadline = Instant::now() + Duration::from_secs(60);
        let initial_size = output.len();
        let mut ended = false;
        while Instant::now() < deadline {
            cancelled(app, id)?;
            match socket.read() {
                Ok(Message::Text(frame)) => match protocol::text_path(&frame)?.as_str() {
                    "turn.end" => { ended = true; break; }
                    "turn.start" | "response" | "audio.metadata" => {}
                    _ => return Err("语音服务响应协议已变化".into()),
                },
                Ok(Message::Binary(frame)) => {
                    let bytes = protocol::audio_frame(&frame)?;
                    if output.len() + bytes.len() > MAX_AUDIO { return Err("合成音频过长，请分段朗读".into()); }
                    output.extend_from_slice(bytes);
                }
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) => {}
                Err(_) => return Err("语音合成连接中断，请重试".into()),
            }
        }
        let _ = socket.close(None);
        if !ended || output.len() == initial_size { return Err("语音服务未返回完整音频，请稍后重试".into()); }
    }
    cancelled(app, id)?;
    remember(app, key, &output);
    Ok(output)
}
#[tauri::command]
pub async fn tts_synthesize(app: tauri::AppHandle, text: String, options: Option<Options>) -> Result<tauri::ipc::Response, String> {
    let options = options.unwrap_or(tts_get_settings(app.clone())?.options);
    let id = app.state::<TtsState>().generation.fetch_add(1, Ordering::SeqCst) + 1;
    tauri::async_runtime::spawn_blocking(move || synthesize(&app, id, &text, &options).map(tauri::ipc::Response::new))
        .await.map_err(|_| "语音合成任务失败")?
}

/// One sentence of a reading that follows a streamed answer. Unlike `tts_synthesize` it does not cancel the
/// sentences already being synthesised for the same reading; `tts_cancel` still stops them all at once.
#[tauri::command]
pub async fn tts_synthesize_part(app: tauri::AppHandle, text: String, options: Option<Options>) -> Result<tauri::ipc::Response, String> {
    let options = options.unwrap_or(tts_get_settings(app.clone())?.options);
    let id = app.state::<TtsState>().generation.load(Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || synthesize(&app, id, &text, &options).map(tauri::ipc::Response::new))
        .await.map_err(|_| "语音合成任务失败")?
}

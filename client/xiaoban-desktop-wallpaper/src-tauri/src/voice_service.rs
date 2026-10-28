//! Local voice service integration (P4): SenseVoice ASR, Edge TTS, and speech broadcast for external agents.
//!
//! Exposes an OpenAI-compatible audio API and an MCP voice service on 127.0.0.1,
//! powered by Xiaoban's local SenseVoice recognizer and Edge speech synthesizer.
//! Also subscribes to delegated agent events to trigger speech announcements.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{Emitter, Manager};
use xiaoban_voice_service::{
    AudioBytes, Backends, Capability, ClientInfo, Consent, Pcm, RunningServer, ServerConfig,
    Synthesizer, Transcriber, VoiceError, VoiceInfo,
};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct VoiceServiceConfig {
    /// Whether the local loopback voice service is enabled.
    pub enabled: bool,
    /// Whether to announce delegated agent events (task done, approval waiting).
    pub speech_broadcast: bool,
    /// Preferred port (default: 58139).
    pub port: u16,
    /// Secret Bearer token.
    pub token: String,
}

impl Default for VoiceServiceConfig {
    fn default() -> Self {
        let mut random = [0u8; 16];
        let _ = getrandom::fill(&mut random);
        let token: String = random.iter().map(|b| format!("{b:02x}")).collect();
        Self {
            enabled: false,
            speech_broadcast: true,
            port: 58139,
            token,
        }
    }
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config").join("voice-service.json"))
}

pub fn load_config(app: &tauri::AppHandle) -> VoiceServiceConfig {
    let Ok(path) = config_path(app) else { return VoiceServiceConfig::default() };
    let Ok(bytes) = fs::read(&path) else { return VoiceServiceConfig::default() };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save_config(app: &tauri::AppHandle, cfg: &VoiceServiceConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&path, bytes).map_err(|e| e.to_string())
}

// ───────────── 后端适配器 ─────────────

struct DesktopTranscriber {
    app: tauri::AppHandle,
}

#[async_trait]
impl Transcriber for DesktopTranscriber {
    async fn transcribe(&self, pcm: &Pcm, _language: Option<&str>) -> Result<String, VoiceError> {
        let samples = pcm.samples.clone();
        let app = self.app.clone();
        crate::asr::asr_transcribe(app, samples).await
            .map_err(|e| VoiceError::Unavailable(e))
    }
}

struct DesktopSynthesizer {
    app: tauri::AppHandle,
}

#[async_trait]
impl Synthesizer for DesktopSynthesizer {
    async fn synthesize(
        &self,
        text: &str,
        voice: Option<&str>,
        speed: Option<f32>,
    ) -> Result<AudioBytes, VoiceError> {
        let text_owned = text.to_string();
        let voice_owned = voice.map(str::to_string);
        let app = self.app.clone();

        let bytes = tauri::async_runtime::spawn_blocking(move || {
            let mut options = crate::tts::tts_get_settings(app.clone())
                .map(|s| s.options)
                .unwrap_or_default();
            if let Some(v) = voice_owned {
                options.voice = v;
            }
            if let Some(s) = speed {
                options.rate = ((s - 1.0) * 100.0).round().clamp(-50.0, 100.0) as i32;
            }
            crate::tts::synthesize_raw(&app, &text_owned, Some(&options))
        })
        .await
        .map_err(|e| VoiceError::Internal(e.to_string()))?
        .map_err(|e| VoiceError::Unavailable(e))?;

        Ok(AudioBytes {
            bytes,
            mime: "audio/mpeg".into(),
        })
    }

    async fn voices(&self) -> Result<Vec<VoiceInfo>, VoiceError> {
        let list = crate::tts::tts_list_voices().await.unwrap_or_default();
        if list.is_empty() {
            Ok(vec![VoiceInfo {
                id: "zh-CN-XiaoxiaoNeural".into(),
                name: "晓晓（中文女声）".into(),
                locale: "zh-CN".into(),
                gender: "Female".into(),
            }])
        } else {
            Ok(list.into_iter().map(|v| VoiceInfo {
                id: v.short_name,
                name: v.friendly_name,
                locale: v.locale,
                gender: v.gender,
            }).collect())
        }
    }
}

struct DesktopConsent;

#[async_trait]
impl Consent for DesktopConsent {
    async fn allow(&self, _client: &ClientInfo, _capability: Capability) -> bool {
        // 本地服务：只要用户在设置中开启该服务，回环请求即被允许
        true
    }
}

// ───────────── 运行时状态 ─────────────

#[derive(Default)]
pub struct VoiceServiceState {
    server: Mutex<Option<RunningServer>>,
    actual_addr: Mutex<Option<SocketAddr>>,
}

/// Start the background voice service if enabled by the user.
pub fn start(app: &tauri::AppHandle) {
    let cfg = load_config(app);
    if cfg.enabled {
        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = start_server(&handle).await;
        });
    }
    start_broadcast_listener(app);
}

/// Subscribe to delegated agent events and emit speech announcements to the frontend.
fn start_broadcast_listener(app: &tauri::AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        // 等待 ChatState 初始化完成
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let Some(chat_state) = handle.try_state::<crate::chat::ChatState>() else { return };
        let mut rx = chat_state.delegation.hub().subscribe();
        while let Ok(event) = rx.recv().await {
            if let Some(announcement) = event.announcement() {
                let cfg = load_config(&handle);
                if cfg.speech_broadcast {
                    let _ = handle.emit("speech-announcement", json!({
                        "text": announcement,
                        "event": event,
                    }));
                }
            }
        }
    });
}

async fn start_server(app: &tauri::AppHandle) -> Result<SocketAddr, String> {
    let state = app.state::<VoiceServiceState>();
    if let Some(addr) = *lock(&state.actual_addr) {
        return Ok(addr);
    }

    let cfg = load_config(app);
    let bind_addr: SocketAddr = format!("127.0.0.1:{}", cfg.port)
        .parse()
        .map_err(|e| format!("绑定地址无效：{e}"))?;

    let srv_cfg = ServerConfig {
        token: cfg.token.clone(),
        bind: bind_addr,
        ..Default::default()
    };

    let backends = Backends {
        transcriber: Arc::new(DesktopTranscriber { app: app.clone() }),
        synthesizer: Arc::new(DesktopSynthesizer { app: app.clone() }),
        microphone: None,
        speaker: None,
        consent: Arc::new(DesktopConsent),
    };

    let running = xiaoban_voice_service::start(srv_cfg, backends).await
        .map_err(|e| format!("启动语音服务失败：{e}"))?;

    let addr = running.addr;
    *lock(&state.actual_addr) = Some(addr);
    *lock(&state.server) = Some(running);

    let _ = app.emit("voice-service-status-changed", ());
    Ok(addr)
}

fn stop_server(app: &tauri::AppHandle) {
    let state = app.state::<VoiceServiceState>();
    let mut old_srv = lock(&state.server);
    let mut old_addr = lock(&state.actual_addr);
    *old_addr = None;
    if let Some(srv) = old_srv.take() {
        tauri::async_runtime::spawn(async move {
            let _ = srv.shutdown().await;
        });
    }
    let _ = app.emit("voice-service-status-changed", ());
}

// ───────────── Tauri 命令 ─────────────

#[tauri::command]
pub fn voice_service_status(app: tauri::AppHandle) -> Result<Value, String> {
    let cfg = load_config(&app);
    let state = app.state::<VoiceServiceState>();
    let addr = *lock(&state.actual_addr);
    let port = addr.map(|a| a.port()).unwrap_or(cfg.port);
    let base_url = format!("http://127.0.0.1:{port}/v1");
    let mcp_url = format!("http://127.0.0.1:{port}/mcp");

    Ok(json!({
        "enabled": cfg.enabled,
        "running": addr.is_some(),
        "speechBroadcast": cfg.speech_broadcast,
        "port": port,
        "token": cfg.token,
        "baseUrl": base_url,
        "mcpUrl": mcp_url,
        "mcpConfigSnippet": {
            "mcpServers": {
                "xiaoban-voice": {
                    "url": mcp_url,
                    "headers": {
                        "Authorization": format!("Bearer {}", cfg.token)
                    }
                }
            }
        }
    }))
}

#[tauri::command]
pub async fn voice_service_set_enabled(
    app: tauri::AppHandle,
    enabled: bool,
    speech_broadcast: Option<bool>,
) -> Result<Value, String> {
    let mut cfg = load_config(&app);
    cfg.enabled = enabled;
    if let Some(b) = speech_broadcast {
        cfg.speech_broadcast = b;
    }
    save_config(&app, &cfg)?;

    if enabled {
        let _ = start_server(&app).await?;
    } else {
        stop_server(&app);
    }

    voice_service_status(app)
}

#[tauri::command]
pub fn voice_service_reset_token(app: tauri::AppHandle) -> Result<Value, String> {
    let mut cfg = load_config(&app);
    let mut random = [0u8; 16];
    let _ = getrandom::fill(&mut random);
    cfg.token = random.iter().map(|b| format!("{b:02x}")).collect();
    save_config(&app, &cfg)?;

    if cfg.enabled {
        stop_server(&app);
        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = start_server(&handle).await;
        });
    }

    voice_service_status(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_and_token_generation() {
        let cfg = VoiceServiceConfig::default();
        assert!(!cfg.enabled);
        assert!(cfg.speech_broadcast);
        assert_eq!(cfg.port, 58139);
        assert_eq!(cfg.token.len(), 32);
    }
}

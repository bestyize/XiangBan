//! Run with XIAOBAN_VOICE_TOKEN set (24+ characters); optional XIAOBAN_VOICE_BIND.
use async_trait::async_trait;
use std::sync::Arc;
use xiaoban_voice_service::*;
#[path = "../tests/fixtures.rs"]
mod fixtures;
struct Mock;
#[async_trait]
impl Transcriber for Mock {
    async fn transcribe(&self, _: &Pcm, _: Option<&str>) -> Result<String, VoiceError> {
        Ok("你好，小伴".into())
    }
}
#[async_trait]
impl Synthesizer for Mock {
    async fn synthesize(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<f32>,
    ) -> Result<AudioBytes, VoiceError> {
        Ok(AudioBytes {
            bytes: fixtures::fixture("mp3"),
            mime: "audio/mpeg".into(),
        })
    }
    async fn voices(&self) -> Result<Vec<VoiceInfo>, VoiceError> {
        Ok(vec![VoiceInfo {
            id: "zh-CN-XiaoxiaoNeural".into(),
            name: "Mock Xiaoxiao (440 Hz tone)".into(),
            locale: "zh-CN".into(),
            gender: "Female".into(),
        }])
    }
}
#[async_trait]
impl Consent for Mock {
    async fn allow(&self, _: &ClientInfo, _: Capability) -> bool {
        true
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        token: std::env::var("XIAOBAN_VOICE_TOKEN")?,
        bind: std::env::var("XIAOBAN_VOICE_BIND")
            .unwrap_or_else(|_| "127.0.0.1:0".into())
            .parse()?,
        ..ServerConfig::default()
    };
    let mock = Arc::new(Mock);
    let server = start(
        config,
        Backends {
            transcriber: mock.clone(),
            synthesizer: mock.clone(),
            microphone: None,
            speaker: None,
            consent: mock,
        },
    )
    .await?;
    println!(
        "http://{} (mock consents automatically; speech is a 440 Hz tone)",
        server.addr
    );
    tokio::signal::ctrl_c().await?;
    server.shutdown().await?;
    Ok(())
}

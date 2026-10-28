#![allow(dead_code)]
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use tokio::sync::Semaphore;
use xiaoban_voice_service::*;
#[path = "../fixtures.rs"]
pub mod fixtures;
pub const TOKEN: &str = "test-secret-at-least-24-characters";
type SynthesisCall = (String, Option<String>, Option<f32>);
pub struct Fake {
    pub seen: Mutex<Vec<Pcm>>,
    pub synthesis: Mutex<Vec<SynthesisCall>>,
    pub approvals: Mutex<Vec<(ClientInfo, Capability)>>,
    pub allow: AtomicBool,
    pub error: Mutex<Option<VoiceError>>,
    pub max: usize,
    pub block_asr: AtomicBool,
    pub block_mic: AtomicBool,
    pub asr_entered: AtomicUsize,
    pub mic_entered: AtomicUsize,
    pub gate: Semaphore,
    pub played: AtomicUsize,
    pub mic_seconds: AtomicUsize,
    pub captured: Mutex<Pcm>,
    pub block_speaker: AtomicBool,
}
impl Default for Fake {
    fn default() -> Self {
        Self {
            seen: Mutex::new(Vec::new()),
            synthesis: Mutex::new(Vec::new()),
            approvals: Mutex::new(Vec::new()),
            allow: AtomicBool::new(true),
            error: Mutex::new(None),
            max: 320_000,
            block_asr: AtomicBool::new(false),
            block_mic: AtomicBool::new(false),
            asr_entered: AtomicUsize::new(0),
            mic_entered: AtomicUsize::new(0),
            gate: Semaphore::new(0),
            played: AtomicUsize::new(0),
            mic_seconds: AtomicUsize::new(0),
            captured: Mutex::new(Pcm {
                samples: vec![0.1; 3200],
            }),
            block_speaker: AtomicBool::new(false),
        }
    }
}
#[async_trait]
impl Transcriber for Fake {
    fn max_samples(&self) -> usize {
        self.max
    }
    async fn transcribe(&self, pcm: &Pcm, _: Option<&str>) -> Result<String, VoiceError> {
        self.asr_entered.fetch_add(1, Ordering::SeqCst);
        if self.block_asr.load(Ordering::SeqCst) {
            self.gate.acquire().await.unwrap().forget();
        }
        if let Some(error) = self.error.lock().unwrap().clone() {
            return Err(error);
        }
        self.seen.lock().unwrap().push(pcm.clone());
        Ok("你好".into())
    }
}
#[async_trait]
impl Synthesizer for Fake {
    async fn synthesize(
        &self,
        text: &str,
        voice: Option<&str>,
        speed: Option<f32>,
    ) -> Result<AudioBytes, VoiceError> {
        self.synthesis
            .lock()
            .unwrap()
            .push((text.into(), voice.map(str::to_owned), speed));
        Ok(AudioBytes {
            bytes: fixtures::fixture("mp3"),
            mime: "audio/mpeg".into(),
        })
    }
    async fn voices(&self) -> Result<Vec<VoiceInfo>, VoiceError> {
        Ok(["zh-CN-XiaoxiaoNeural", "second"]
            .iter()
            .map(|id| VoiceInfo {
                id: id.to_string(),
                name: id.to_string(),
                locale: "zh-CN".into(),
                gender: "Female".into(),
            })
            .collect())
    }
}
#[async_trait]
impl Consent for Fake {
    async fn allow(&self, client: &ClientInfo, capability: Capability) -> bool {
        self.approvals
            .lock()
            .unwrap()
            .push((client.clone(), capability));
        self.allow.load(Ordering::SeqCst)
    }
}
#[async_trait]
impl Microphone for Fake {
    async fn listen(&self, max_secs: u32) -> Result<Pcm, VoiceError> {
        self.mic_seconds.store(max_secs as usize, Ordering::SeqCst);
        self.mic_entered.fetch_add(1, Ordering::SeqCst);
        if self.block_mic.load(Ordering::SeqCst) {
            self.gate.acquire().await.unwrap().forget();
        }
        if let Some(error) = self.error.lock().unwrap().clone() {
            return Err(error);
        }
        Ok(self.captured.lock().unwrap().clone())
    }
}
#[async_trait]
impl Speaker for Fake {
    async fn play(&self, _: AudioBytes) -> Result<(), VoiceError> {
        self.played.fetch_add(1, Ordering::SeqCst);
        if self.block_speaker.load(Ordering::SeqCst) {
            self.gate.acquire().await.unwrap().forget();
        }
        Ok(())
    }
}
pub fn backends(fake: &Arc<Fake>, devices: bool) -> Backends {
    Backends {
        transcriber: fake.clone(),
        synthesizer: fake.clone(),
        microphone: devices.then(|| fake.clone() as Arc<dyn Microphone>),
        speaker: devices.then(|| fake.clone() as Arc<dyn Speaker>),
        consent: fake.clone(),
    }
}
pub struct TestServer {
    pub running: RunningServer,
    pub fake: Arc<Fake>,
    pub client: Client,
}
impl TestServer {
    pub async fn new() -> Self {
        Self::with(Arc::new(Fake::default()), ServerConfig::default(), false).await
    }
    pub async fn with(fake: Arc<Fake>, mut config: ServerConfig, devices: bool) -> Self {
        config.token = TOKEN.into();
        Self {
            running: start(config, backends(&fake, devices)).await.unwrap(),
            fake,
            client: Client::builder().no_proxy().build().unwrap(),
        }
    }
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.running.addr, path)
    }
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.client.get(self.url(path)).bearer_auth(TOKEN)
    }
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.client.post(self.url(path)).bearer_auth(TOKEN)
    }
    pub fn transcribe(
        &self,
        file: Vec<u8>,
        format: &str,
        language: Option<&str>,
    ) -> RequestBuilder {
        let mut form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(file).file_name("ignored.bin"),
            )
            .text("model", "anything")
            .text("response_format", format.to_owned());
        if let Some(language) = language {
            form = form.text("language", language.to_owned());
        }
        self.post("/v1/audio/transcriptions").multipart(form)
    }
}
pub fn wav(samples: &[f32], rate: u32, channels: u16, bits: u16, float: bool) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(
            &mut cursor,
            hound::WavSpec {
                channels,
                sample_rate: rate,
                bits_per_sample: bits,
                sample_format: if float {
                    hound::SampleFormat::Float
                } else {
                    hound::SampleFormat::Int
                },
            },
        )
        .unwrap();
        for sample in samples {
            if float {
                writer.write_sample(*sample).unwrap();
            } else {
                writer
                    .write_sample((sample * 2_f32.powi(bits as i32 - 1)) as i32)
                    .unwrap();
            }
        }
        writer.finalize().unwrap();
    }
    cursor.into_inner()
}
pub fn tone() -> Vec<u8> {
    wav(&vec![0.2; 3200], 16_000, 1, 16, false)
}
pub async fn entered(counter: &AtomicUsize, target: usize) {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while counter.load(Ordering::SeqCst) < target {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}
pub async fn assert_error(
    response: reqwest::Response,
    status: u16,
    code: &str,
) -> serde_json::Value {
    assert_eq!(response.status().as_u16(), status);
    if status == 429 {
        assert_eq!(response.headers()["retry-after"], "2");
    }
    let value: serde_json::Value = response.json().await.unwrap();
    assert_eq!(value["error"]["type"], "invalid_request_error");
    assert_eq!(value["error"]["code"], code);
    assert!(value["error"]["message"].is_string());
    value
}

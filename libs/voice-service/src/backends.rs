//! Host-provided voice capabilities, independent of devices and GUI frameworks.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

/// Backend failure. Error detail strings are never exposed in HTTP responses or logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceError {
    /// Resource is currently occupied.
    Busy,
    /// Backend is temporarily unavailable.
    Unavailable(String),
    /// Unsupported audio or operation.
    Unsupported(String),
    /// Audio exceeds the service duration limit.
    TooLong,
    /// Invalid backend input.
    InvalidInput(String),
    /// Operation was cancelled.
    Cancelled,
    /// Unexpected backend failure.
    Internal(String),
}
impl fmt::Display for VoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Busy => "resource busy",
            Self::Unavailable(_) => "backend unavailable",
            Self::Unsupported(_) => "unsupported audio; supported: WAV, MP3, FLAC, OGG Vorbis",
            Self::TooLong => "audio exceeds 10 minutes",
            Self::InvalidInput(_) => "invalid audio or input",
            Self::Cancelled => "operation cancelled",
            Self::Internal(_) => "internal backend error",
        })
    }
}
impl std::error::Error for VoiceError {}

/// 16 kHz mono samples, finite and within -1.0..=1.0.
#[derive(Clone, Debug, Default)]
pub struct Pcm {
    /// Consecutive samples without a container header.
    pub samples: Vec<f32>,
}
/// Encoded audio owned by the caller.
#[derive(Clone, Debug)]
pub struct AudioBytes {
    /// Complete encoded file in memory.
    pub bytes: Vec<u8>,
    /// MIME type, normally `audio/mpeg` for synthesis.
    pub mime: String,
}
/// Available synthesizer voice.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceInfo {
    /// Stable backend identifier.
    pub id: String,
    /// User-visible name.
    pub name: String,
    /// BCP-47 locale.
    pub locale: String,
    /// Backend-defined gender label.
    pub gender: String,
}
/// Converts PCM into text. Implementations must be cancellation-safe.
#[async_trait]
pub trait Transcriber: Send + Sync {
    /// Hard maximum per backend call; the library splits longer input.
    fn max_samples(&self) -> usize {
        20 * 16_000
    }
    /// Recognize one segment, optionally using a language hint.
    async fn transcribe(&self, pcm: &Pcm, language: Option<&str>) -> Result<String, VoiceError>;
}
/// Produces MP3 bytes without playing them.
#[async_trait]
pub trait Synthesizer: Send + Sync {
    /// Synthesize using a resolved voice and optional speed in 0.25..=4.0.
    async fn synthesize(
        &self,
        text: &str,
        voice: Option<&str>,
        speed: Option<f32>,
    ) -> Result<AudioBytes, VoiceError>;
    /// Enumerate voices, default voice first.
    async fn voices(&self) -> Result<Vec<VoiceInfo>, VoiceError>;
}
/// Captures one utterance, including end-of-speech detection in the host.
#[async_trait]
pub trait Microphone: Send + Sync {
    /// Capture for at most `max_secs`; return Busy when occupied.
    async fn listen(&self, max_secs: u32) -> Result<Pcm, VoiceError>;
}
/// Host playback, cancellation-safe and returning only when playback finishes.
#[async_trait]
pub trait Speaker: Send + Sync {
    /// Play encoded audio on the local machine.
    async fn play(&self, audio: AudioBytes) -> Result<(), VoiceError>;
}
/// Separately consented capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Uploaded or captured audio recognition.
    Transcribe,
    /// Text synthesis.
    Synthesize,
    /// Local microphone capture.
    Listen,
    /// Local speaker playback.
    Speak,
}
/// Self-reported client identity; headers are labels, not authentication.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClientInfo {
    /// X-Client-Name, otherwise User-Agent, otherwise `unknown`.
    pub name: String,
    /// Optional original User-Agent.
    pub user_agent: Option<String>,
}
/// Host UI approval. Only allowed decisions are cached; unanswered after 30s is denied.
#[async_trait]
pub trait Consent: Send + Sync {
    /// Ask the host user to approve this capability for the client label.
    async fn allow(&self, client: &ClientInfo, capability: Capability) -> bool;
}
/// Capability implementations injected into a server instance.
#[derive(Clone)]
pub struct Backends {
    /// Recognition backend.
    pub transcriber: Arc<dyn Transcriber>,
    /// MP3 synthesis backend.
    pub synthesizer: Arc<dyn Synthesizer>,
    /// Optional local microphone.
    pub microphone: Option<Arc<dyn Microphone>>,
    /// Optional local speaker.
    pub speaker: Option<Arc<dyn Speaker>>,
    /// User approval backend.
    pub consent: Arc<dyn Consent>,
}

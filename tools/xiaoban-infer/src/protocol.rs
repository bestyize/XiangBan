//! One JSON object per line on stdin and stdout. Every capability shares these messages; a capability answers
//! the requests it understands and refuses the rest.
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

/// Bumped when a message changes in a way the app must know about.
pub const PROTOCOL_VERSION: u32 = 1;
/// The longest audio any request may carry: 20 s at 16 kHz.
pub const MAX_SAMPLES: usize = 20 * 16000;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Request {
    Ping,
    Exit,
    /// `asr`: text of 0.1–20 s of 16 kHz audio.
    Transcribe { samples: Vec<f32> },
    /// `turn`: speech probabilities for the next 16 kHz samples (16-bit PCM) of a listener.
    Vad { stream: String, pcm: Vec<i16> },
    /// `turn`: forget a listener's VAD state.
    VadReset { stream: String },
    /// `turn`: probability that the speaker finished, from up to 20 s of 16-bit PCM.
    Turn { pcm: Vec<i16> },
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Response {
    /// First line after start: the protocol version and the capability this process serves.
    Ready { version: u32, capabilities: Vec<&'static str> },
    Pong,
    Ok { text: String },
    Probabilities { values: Vec<f32> },
    Probability { value: f32 },
    Error { error: String },
}

impl Response {
    pub fn error(message: impl Into<String>) -> Self { Self::Error { error: message.into() } }
}

pub fn send(response: &Response) {
    let mut out = io::stdout().lock();
    if let Ok(line) = serde_json::to_string(response) {
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }
}

/// 16-bit PCM to samples in -1..1.
pub fn pcm(samples: &[i16]) -> Vec<f32> { samples.iter().map(|s| *s as f32 / 32768.0).collect() }

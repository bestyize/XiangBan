//! HTTP transport shared by the streaming providers: POST a request, map failures, read
//! server-sent events on a helper thread so cancel and stall checks never wait on the socket.

use super::stream::SseParser;
use crate::error::HarnessError;
use reqwest::blocking::Client;
use std::io::{BufRead, BufReader, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

const ERROR_BODY_LIMIT: u64 = 2000;
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(90);

pub(crate) struct Call {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// One protocol's reading of the event stream.
pub(crate) trait Decoder {
    /// Handles one event payload. `Ok(true)` means the terminal event arrived and reading can stop.
    fn push(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError>;
    /// Whether the stream may end here without a terminal event and still count as complete.
    fn complete(&self) -> bool;
}

pub(crate) fn new_client() -> Result<Client, HarnessError> {
    // The blocking client defaults to a 30s total timeout, which would cut long streams.
    // Stalls are caught by STREAM_IDLE_TIMEOUT; this is only a backstop for leaked reader threads.
    Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(900))
        .build()
        .map_err(|e| HarnessError::Config(format!("http client: {e}")))
}

/// `extra` replaces defaults of the same name (case-insensitively) instead of adding a duplicate.
pub(crate) fn merge_headers(mut headers: Vec<(String, String)>, extra: &[(String, String)]) -> Vec<(String, String)> {
    for (name, value) in extra {
        headers.retain(|(existing, _)| !existing.eq_ignore_ascii_case(name));
        headers.push((name.clone(), value.clone()));
    }
    headers
}

fn status_is_retryable(status: u16) -> bool {
    matches!(status, 408 | 425 | 429) || status >= 500
}

pub(crate) fn post_stream(
    client: &Client,
    call: Call,
    decoder: &mut dyn Decoder,
    on_text: &mut dyn FnMut(&str),
    cancel: &AtomicBool,
) -> Result<(), HarnessError> {
    let mut builder = client.post(call.url);
    for (name, value) in &call.headers {
        builder = builder.header(name.as_str(), value.as_str());
    }
    let response = builder
        .body(call.body)
        .send()
        .map_err(|e| HarnessError::provider(format!("request failed: {e}"), true))?;

    let status = response.status();
    if !status.is_success() {
        let mut raw = Vec::new();
        let _ = response.take(ERROR_BODY_LIMIT).read_to_end(&mut raw);
        let detail = String::from_utf8_lossy(&raw);
        let message = format!("HTTP {}: {}", status.as_u16(), detail.trim());
        return Err(HarnessError::provider(message, status_is_retryable(status.as_u16())));
    }

    let (tx, rx) = mpsc::channel::<Result<String, String>>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(response);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) if tx.send(Ok(line)).is_err() => break,
                Ok(_) => {}
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    break;
                }
            }
        }
    });

    let mut parser = SseParser::default();
    let mut last_activity = Instant::now();
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(HarnessError::Cancelled);
        }
        let (event, at_end) = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(line)) => {
                last_activity = Instant::now();
                (parser.push_line(&line), false)
            }
            Ok(Err(e)) => return Err(HarnessError::provider(format!("stream read failed: {e}"), true)),
            Err(RecvTimeoutError::Timeout) => {
                if last_activity.elapsed() > STREAM_IDLE_TIMEOUT {
                    return Err(HarnessError::provider("stream stalled", true));
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => (parser.flush(), true),
        };
        if let Some(payload) = event {
            if decoder.push(&payload, on_text)? {
                return Ok(());
            }
        }
        if at_end {
            break;
        }
    }
    if decoder.complete() {
        Ok(())
    } else {
        Err(HarnessError::provider("stream ended before completion", true))
    }
}

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
const RESPONSE_HEADER_TIMEOUT: Duration = Duration::from_secs(45);
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
    /// Only protocol-supplied displayable text goes to this independent channel.
    fn push_with_reasoning(
        &mut self, payload: &str, on_text: &mut dyn FnMut(&str),
        _on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) -> Result<bool, HarnessError> { self.push(payload, on_text) }
    /// Whether the stream may end here without a terminal event and still count as complete.
    fn complete(&self) -> bool;
}

pub(crate) fn new_client() -> Result<Client, HarnessError> {
    // Allow long reasoning streams, but bound a blocking worker that no longer has a consumer.
    // The first response header and SSE idle have separate, shorter watchdogs below.
    Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300))
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

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn post_stream(
    client: &Client,
    call: Call,
    decoder: &mut dyn Decoder,
    on_text: &mut dyn FnMut(&str),
    cancel: &AtomicBool,
) -> Result<(), HarnessError> {
    post_stream_with_reasoning(client, call, decoder, on_text, &mut |_, _| {}, cancel)
}

pub(crate) fn post_stream_with_reasoning(
    client: &Client,
    call: Call,
    decoder: &mut dyn Decoder,
    on_text: &mut dyn FnMut(&str),
    on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    cancel: &AtomicBool,
) -> Result<(), HarnessError> {
    let mut builder = client.post(call.url);
    for (name, value) in &call.headers {
        builder = builder.header(name.as_str(), value.as_str());
    }
    if cancel.load(Ordering::Relaxed) { return Err(HarnessError::Cancelled); }
    // Sending and waiting for response headers must be on the helper too. A model
    // can spend minutes thinking before it writes even its first response header.
    enum Incoming { Started, Line(String), Failed(String, bool) }
    let (tx, rx) = mpsc::sync_channel::<Incoming>(64);
    std::thread::spawn(move || {
        let response = match builder.body(call.body).send() {
            Ok(response) => response,
            Err(error) => { let _ = tx.send(Incoming::Failed(format!("request failed: {error}"), true)); return; }
        };
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let mut raw = Vec::new();
            let _ = response.take(ERROR_BODY_LIMIT).read_to_end(&mut raw);
            let message = if status == 413 {
                "HTTP 413: request_too_large: 请求体积超过服务端限制，请减少图片、降低截图尺寸或压缩会话后继续。".to_string()
            } else {
                format!("HTTP {status}: {}", String::from_utf8_lossy(&raw).trim())
            };
            let _ = tx.send(Incoming::Failed(message, status_is_retryable(status)));
            return;
        }
        if tx.send(Incoming::Started).is_err() { return; }
        let mut reader = BufReader::new(response);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) if tx.send(Incoming::Line(line)).is_err() => break,
                Ok(_) => {}
                Err(e) => {
                    let _ = tx.send(Incoming::Failed(format!("stream read failed: {e}"), true));
                    break;
                }
            }
        }
    });

    let mut parser = SseParser::default();
    let requested_at = Instant::now();
    let mut last_activity = requested_at;
    let mut started = false;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(HarnessError::Cancelled);
        }
        let (event, at_end) = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Incoming::Started) => { started = true; last_activity = Instant::now(); continue; }
            Ok(Incoming::Line(line)) => {
                last_activity = Instant::now();
                (parser.push_line(&line), false)
            }
            Ok(Incoming::Failed(message, retryable)) => return Err(HarnessError::provider(message, retryable)),
            Err(RecvTimeoutError::Timeout) => {
                // A server can accept TCP/TLS but never send an HTTP response. The idle
                // SSE timeout cannot catch that case until Incoming::Started arrives.
                if !started && requested_at.elapsed() > RESPONSE_HEADER_TIMEOUT {
                    return Err(HarnessError::provider("模型服务在 45 秒内没有响应，请检查供应商状态或代理连接", true));
                }
                if started && last_activity.elapsed() > STREAM_IDLE_TIMEOUT {
                    return Err(HarnessError::provider("stream stalled", true));
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => (parser.flush(), true),
        };
        if let Some(payload) = event {
            if decoder.push_with_reasoning(&payload, on_text, on_reasoning)? {
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

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::{io::Write, net::TcpListener, sync::Arc};
    struct NeverDone;
    impl Decoder for NeverDone {
        fn push(&mut self, _: &str, _: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> { Ok(false) }
        fn complete(&self) -> bool { false }
    }
    #[test]
    fn cancel_does_not_wait_for_the_first_response_headers() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            accepted_tx.send(()).unwrap();
            let _ = release_rx.recv_timeout(Duration::from_secs(5));
            let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        });
        let cancel = Arc::new(AtomicBool::new(false)); let flag = cancel.clone();
        let (result_tx, result_rx) = mpsc::channel();
        let run = std::thread::spawn(move || {
            let client = Client::builder().no_proxy().timeout(Duration::from_secs(5)).build().unwrap();
            let result = post_stream(&client, Call { url, headers: vec![], body: vec![] }, &mut NeverDone, &mut |_| {}, &flag);
            let _ = result_tx.send(result);
        });
        accepted_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        cancel.store(true, Ordering::SeqCst);
        // Release the server after observing the result, so the former blocking send
        // deterministically fails this check instead of accidentally passing on headers.
        let result = result_rx.recv_timeout(Duration::from_secs(1));
        let _ = release_tx.send(()); let _ = server.join(); let _ = run.join();
        assert!(matches!(result, Ok(Err(HarnessError::Cancelled))));
    }
}

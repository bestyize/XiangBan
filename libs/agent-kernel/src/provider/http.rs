//! HTTP transport shared by the streaming providers: POST a request, map failures, read
//! server-sent events with cancellable asynchronous I/O.

use super::stream::SseParser;
use crate::error::HarnessError;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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

pub(crate) fn new_client(profile: &super::wire::ProviderProfile) -> Result<Client, HarnessError> {
    // Bound each request as well as header and stream idle time.
    // The first response header and SSE idle have separate, shorter watchdogs below.
    let builder = Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300));
    let builder = match profile.proxy_mode.as_str() {
        "direct" => builder.no_proxy(),
        "manual" => builder.no_proxy().proxy(reqwest::Proxy::all(profile.proxy_url.as_str())
            .map_err(|e| HarnessError::Config(format!("invalid model proxy: {e}")))?),
        _ => builder,
    };
    builder.build().map_err(|e| HarnessError::Config(format!("http client: {e}")))
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
    let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e|HarnessError::Config(e.to_string()))?;
    rt.block_on(async {
        let mut builder=client.post(call.url);
        for (name,value) in call.headers {builder=builder.header(name,value);}
        let mut response=crate::net::wait(builder.body(call.body).send(),||cancel.load(Ordering::Relaxed),RESPONSE_HEADER_TIMEOUT).await?
            .map_err(|e|HarnessError::provider(format!("request failed: {e}"),true))?;
        if !response.status().is_success(){
            let status=response.status().as_u16();let mut raw=Vec::new();
            while raw.len()<ERROR_BODY_LIMIT as usize {
                let chunk=crate::net::wait(response.chunk(),||cancel.load(Ordering::Relaxed),Duration::from_secs(10)).await?.map_err(|e|HarnessError::provider(e.to_string(),true))?;
                let Some(chunk)=chunk else{break;};let n=chunk.len().min(ERROR_BODY_LIMIT as usize-raw.len());raw.extend_from_slice(&chunk[..n]);
            }
            return Err(HarnessError::provider(if status==413 { "HTTP 413: request_too_large: 请求体积超过服务端限制，请减少图片、降低截图尺寸或压缩会话后继续。".into() } else {format!("HTTP {status}: {}",String::from_utf8_lossy(&raw))},status_is_retryable(status)));
        }
        let mut parser=SseParser::default();let mut buffer=Vec::new();let mut total=0usize;let mut event_bytes=0usize;
        loop {
            let chunk=crate::net::wait(response.chunk(),||cancel.load(Ordering::Relaxed),STREAM_IDLE_TIMEOUT).await?.map_err(|e|HarnessError::provider(e.to_string(),true))?;
            let Some(chunk)=chunk else{break;};total=total.saturating_add(chunk.len());
            if total>64*1024*1024 {return Err(HarnessError::provider("model stream exceeds 64 MiB",false));}
            buffer.extend_from_slice(&chunk);
            while let Some(end)=buffer.iter().position(|b|*b==b'\n') {
                let line:Vec<_>=buffer.drain(..=end).collect();event_bytes+=line.len();
                if line.len()>2*1024*1024 || event_bytes>8*1024*1024 {return Err(HarnessError::provider("SSE line/event exceeds limit",false));}
                let line=std::str::from_utf8(&line).map_err(|_|HarnessError::provider("invalid SSE UTF-8",false))?;
                if line.trim().is_empty(){event_bytes=0;}
                if let Some(payload)=parser.push_line(line){if decoder.push_with_reasoning(&payload,on_text,on_reasoning)?{return Ok(());}}
            }
            if buffer.len()>2*1024*1024{return Err(HarnessError::provider("SSE line exceeds limit",false));}
        }
        if !buffer.is_empty(){let line=std::str::from_utf8(&buffer).map_err(|_|HarnessError::provider("invalid SSE UTF-8",false))?;if let Some(p)=parser.push_line(line){if decoder.push_with_reasoning(&p,on_text,on_reasoning)?{return Ok(());}}}
        if let Some(p)=parser.flush(){if decoder.push_with_reasoning(&p,on_text,on_reasoning)?{return Ok(());}}
        if decoder.complete(){Ok(())}else{Err(HarnessError::provider("stream ended before terminal event",true))}
    })
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::sync::{Arc,mpsc};
    use std::{io::Write, net::TcpListener};
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

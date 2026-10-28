//! HTTP transport shared by the streaming providers: POST a request, map failures, read
//! server-sent events with cancellable asynchronous I/O.

use super::stream::SseParser;
use crate::error::HarnessError;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const ERROR_BODY_LIMIT: u64 = 2000;

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
    /// The tool call being written out right now, if any, for progress reports while its arguments stream in.
    fn pending_tool(&self) -> Option<super::ToolProgress> { None }
    /// Whether the stream may end here without a terminal event and still count as complete.
    fn complete(&self) -> bool;
}

pub(crate) fn new_client(profile: &super::wire::ProviderProfile) -> Result<Client, HarnessError> {
    // Do not set a total request timeout: reqwest includes streaming body reads in
    // that deadline, so a healthy long reasoning/output stream would be cut off.
    // Header and per-chunk idle watchdogs below still bound stalled requests.
    profile.timeouts.validate()?;
    let builder = Client::builder()
        .connect_timeout(Duration::from_secs(15));
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

/// A tool call's streaming arguments are reported about once a second while they grow, sooner (but at most four times
/// a second) for every 4 KB: enough to show a long silent write is moving, without an event per chunk.
fn progress_due(last: &mut (Instant, usize, bool), bytes: usize) -> bool {
    let (at, seen, any) = *last;
    if bytes == seen && any { return false; }
    let due = !any || at.elapsed() >= Duration::from_secs(1)
        || (bytes.saturating_sub(seen) >= 4096 && at.elapsed() >= Duration::from_millis(250));
    if due { *last = (Instant::now(), bytes, true); }
    due
}

fn status_is_retryable(status: u16) -> bool {
    matches!(status, 408 | 425 | 429) || status >= 500
}

async fn wait_for_io<F: std::future::Future>(
    future: F, cancel: &AtomicBool, timeout: Duration, phase: &str,
) -> Result<F::Output, HarnessError> {
    crate::net::wait(future, || cancel.load(Ordering::Relaxed), timeout).await
        .map_err(|error| match error {
            HarnessError::Provider { .. } => HarnessError::provider(
                format!("model HTTP timeout {phase}: no data for {}s", timeout.as_secs()), true),
            other => other,
        })
}

fn transport_error(error: reqwest::Error, phase: &str) -> HarnessError {
    let kind = if error.is_timeout() { "timed out" } else { "failed" };
    // reqwest may label a body timeout as a decode error. Keep its source chain
    // for diagnostics while showing the timeout and transport phase explicitly.
    let mut detail = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        detail.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    HarnessError::provider(format!("model HTTP {phase} {kind}: {detail}"), true)
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
    post_stream_with_policy(client, call, decoder, on_text, on_reasoning, &mut |_| {}, cancel, &super::wire::ModelTimeouts::default())
}

pub(crate) fn post_stream_with_policy(
    client: &Client, call: Call, decoder: &mut dyn Decoder,
    on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    on_tool: &mut dyn FnMut(&super::ToolProgress), cancel: &AtomicBool, policy: &super::wire::ModelTimeouts,
) -> Result<(), HarnessError> {
    policy.validate()?;
    let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e|HarnessError::Config(e.to_string()))?;
    rt.block_on(async {
        let mut builder=client.post(call.url);
        for (name,value) in call.headers {builder=builder.header(name,value);}
        let mut response=wait_for_io(builder.body(call.body).send(),cancel,Duration::from_secs(policy.response_header_seconds),"waiting for response headers").await?
            .map_err(|e|transport_error(e,"request"))?;
        if !response.status().is_success(){
            let status=response.status().as_u16();let mut raw=Vec::new();
            while raw.len()<ERROR_BODY_LIMIT as usize {
                let chunk=wait_for_io(response.chunk(),cancel,Duration::from_secs(10),"reading error response").await?.map_err(|e|transport_error(e,"error response read"))?;
                let Some(chunk)=chunk else{break;};let n=chunk.len().min(ERROR_BODY_LIMIT as usize-raw.len());raw.extend_from_slice(&chunk[..n]);
            }
            return Err(HarnessError::provider(if status==413 { "HTTP 413: request_too_large: 请求体积超过服务端限制，请减少图片、降低截图尺寸或压缩会话后继续。".into() } else {format!("HTTP {status}: {}",String::from_utf8_lossy(&raw))},status_is_retryable(status)));
        }
        let mut parser=SseParser::default();let mut buffer=Vec::new();let mut total=0usize;let mut event_bytes=0usize;let mut received_event=false;let mut reported=(Instant::now(),0usize,false);
        loop {
            let chunk=wait_for_io(response.chunk(),cancel,Duration::from_secs(if received_event {policy.stream_idle_seconds} else {policy.first_event_seconds}),if received_event {"waiting for stream data"} else {"waiting for first model event"}).await?.map_err(|e|transport_error(e,"stream read"))?;
            let Some(chunk)=chunk else{break;};total=total.saturating_add(chunk.len());
            if total>64*1024*1024 {return Err(HarnessError::provider("model stream exceeds 64 MiB",false));}
            buffer.extend_from_slice(&chunk);
            while let Some(end)=buffer.iter().position(|b|*b==b'\n') {
                let line:Vec<_>=buffer.drain(..=end).collect();event_bytes+=line.len();
                if line.len()>2*1024*1024 || event_bytes>8*1024*1024 {return Err(HarnessError::provider("SSE line/event exceeds limit",false));}
                let line=std::str::from_utf8(&line).map_err(|_|HarnessError::provider("invalid SSE UTF-8",false))?;
                if line.trim().is_empty(){event_bytes=0;}
                if let Some(payload)=parser.push_line(line){received_event=true;if decoder.push_with_reasoning(&payload,on_text,on_reasoning)?{return Ok(());}}
            }
            if buffer.len()>2*1024*1024{return Err(HarnessError::provider("SSE line exceeds limit",false));}
            if let Some(progress)=decoder.pending_tool() {
                if progress_due(&mut reported,progress.bytes) {on_tool(&progress);}
            }
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
    use std::{io::{Read, Write}, net::TcpListener};
    struct NeverDone;
    impl Decoder for NeverDone {
        fn push(&mut self, _: &str, _: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> { Ok(false) }
        fn complete(&self) -> bool { false }
    }

    // Release the final event only after the client has advanced its clock.
    fn held_stream() -> (String, mpsc::Sender<()>, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (release, wait) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                if socket.read(&mut byte).unwrap_or(0) == 0 { return; }
                request.push(byte[0]);
            }
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: thinking\n\n").unwrap();
            let _ = wait.recv_timeout(Duration::from_secs(5));
            let _ = socket.write_all(b"data: [DONE]\n\n");
        });
        (url, release, server)
    }

    #[test]
    fn model_stream_can_read_past_the_former_total_deadline() {
        let (url, release, server) = held_stream();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let result = rt.block_on(async {
            let mut profile = super::super::wire::ProviderProfile::new(&url, "", "test");
            profile.proxy_mode = "direct".into();
            let client = new_client(&profile).unwrap();
            let mut response = client.get(&url).send().await.unwrap();
            assert_eq!(response.chunk().await.unwrap().unwrap().as_ref(), b"data: thinking\n\n");
            tokio::time::pause();
            // Exercise reqwest's actual deadline without a five-minute wall-clock test.
            tokio::time::advance(Duration::from_secs(301)).await;
            tokio::time::resume();
            release.send(()).unwrap();
            response.chunk().await
        });
        server.join().unwrap();
        assert_eq!(result.unwrap().unwrap().as_ref(), b"data: [DONE]\n\n");
    }

    #[test]
    fn body_timeout_is_reported_as_timeout_instead_of_just_decode_error() {
        let (url, release, server) = held_stream();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let error = rt.block_on(async {
            let client = Client::builder().no_proxy().timeout(Duration::from_secs(300)).build().unwrap();
            let mut response = client.get(url).send().await.unwrap();
            response.chunk().await.unwrap().unwrap();
            tokio::time::pause();
            tokio::time::advance(Duration::from_secs(301)).await;
            tokio::time::resume();
            let error = response.chunk().await.unwrap_err();
            let _ = release.send(());
            transport_error(error, "stream read")
        });
        server.join().unwrap();
        assert!(matches!(error, HarnessError::Provider { message, retryable: true }
            if message.contains("stream read timed out")), "body timeout must retain its cause");
    }

    #[test]
    fn io_watchdogs_report_the_phase_and_still_honor_cancellation() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            for phase in ["waiting for response headers", "waiting for stream data"] {
                let error = wait_for_io(std::future::pending::<()>(), &AtomicBool::new(false),
                    Duration::ZERO, phase).await.unwrap_err();
                assert!(matches!(error, HarnessError::Provider { message, retryable: true }
                    if message.contains(phase) && message.contains("timeout")));
            }
            let error = wait_for_io(std::future::pending::<()>(), &AtomicBool::new(true),
                Duration::from_secs(300), "waiting for stream data").await.unwrap_err();
            assert_eq!(error, HarnessError::Cancelled);
        });
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

#[cfg(test)]
mod phase_timeout_tests {
    use super::*;
    use std::{io::{Read,Write},net::TcpListener,sync::mpsc};
    struct DecoderStub;
    impl Decoder for DecoderStub {
        fn push(&mut self,payload:&str,_:&mut dyn FnMut(&str))->Result<bool,HarnessError>{Ok(payload=="[DONE]")}
        fn complete(&self)->bool{false}
    }
    #[test]
    fn custom_header_first_event_and_idle_watchdogs_are_independent() {
        for (prefix,phase) in [
            ("","waiting for response headers"),
            ("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n","waiting for first model event"),
            ("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: started\n\n","waiting for stream data"),
        ] {
            let listener=TcpListener::bind("127.0.0.1:0").unwrap();let url=format!("http://{}",listener.local_addr().unwrap());let(release,wait)=mpsc::channel();
            let server=std::thread::spawn(move||{let(mut socket,_)=listener.accept().unwrap();socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();let mut raw=Vec::new();let mut byte=[0];while !raw.ends_with(b"\r\n\r\n"){if socket.read(&mut byte).unwrap_or(0)==0{return;}raw.push(byte[0]);}let _=socket.write_all(prefix.as_bytes());let _=wait.recv_timeout(Duration::from_secs(3));});
            let policy=super::super::wire::ModelTimeouts{response_header_seconds:1,first_event_seconds:1,stream_idle_seconds:1};
            let error=post_stream_with_policy(&Client::builder().no_proxy().build().unwrap(),Call{url,headers:vec![],body:vec![]},&mut DecoderStub,&mut |_|{},&mut |_,_|{},&mut |_|{},&AtomicBool::new(false),&policy).unwrap_err();
            let _=release.send(());server.join().unwrap();
            assert!(matches!(error,HarnessError::Provider{message,..}if message.contains(phase)),"wrong phase for {phase}");
        }
    }
}

#[cfg(test)]
mod progress_tests {
    use super::*;

    #[test]
    fn progress_is_reported_first_then_on_growth_but_never_per_chunk() {
        let mut last = (Instant::now(), 0usize, false);
        assert!(progress_due(&mut last, 10), "the first sign of a tool call is reported at once");
        assert!(!progress_due(&mut last, 10), "no growth, no report");
        assert!(!progress_due(&mut last, 3000), "a small step right after a report waits");
        assert!(!progress_due(&mut last, 9000), "so does a big one inside the minimum gap");
        last.0 = Instant::now() - Duration::from_millis(300);
        assert!(progress_due(&mut last, 9000), "4 KB of growth after the minimum gap");
        last.0 = Instant::now() - Duration::from_millis(1100);
        assert!(progress_due(&mut last, 9100), "a second of silence with any growth");
    }

    #[test]
    fn the_target_path_is_read_from_half_written_arguments() {
        use super::super::path_hint;
        assert_eq!(path_hint(r#"{"path":"a/b.js","content":"xx"#).as_deref(), Some("a/b.js"));
        assert_eq!(path_hint(r#"{ "path" : "with \"quote\".js""#).as_deref(), Some("with \"quote\".js"));
        assert_eq!(path_hint(r#"{"path":"unfinished"#), None, "a path that is still being written is not shown");
        assert_eq!(path_hint(r#"{"content":"x"}"#), None);
    }
}

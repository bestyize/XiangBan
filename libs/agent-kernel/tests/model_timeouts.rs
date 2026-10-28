//! A model gateway that stops answering must fail fast with a retryable error, so the run retries on its own
//! instead of hanging until the user presses stop.
#![cfg(feature = "anthropic")]
mod common;

use agent_kernel::provider::anthropic::AnthropicMessages;
use agent_kernel::provider::wire::{ModelTimeouts, ProviderProfile};
use agent_kernel::*;
use common::http::*;
use std::io::Write;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

const ANSWER: [(&str, &str); 6] = [
    ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":5,"output_tokens":1}}}"#),
    ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#),
    ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"done"}}"#),
    ("content_block_stop", r#"{"type":"content_block_stop","index":0}"#),
    ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":3}}"#),
    ("message_stop", r#"{"type":"message_stop"}"#),
];

#[test]
fn defaults_bound_a_silent_gateway_to_about_a_minute() {
    let t = ModelTimeouts::default();
    assert!(t.response_header_seconds <= 60, "headers: {}s", t.response_header_seconds);
    assert!(t.first_event_seconds <= 120, "first event: {}s", t.first_event_seconds);
    assert!(t.stream_idle_seconds <= 120, "idle: {}s", t.stream_idle_seconds);
}

/// The first connection is accepted and the request read, but no response header is ever written, like the
/// gateway in the field (request bytes out, nothing in). The second connection answers normally.
#[test]
fn a_gateway_that_never_sends_headers_times_out_and_the_run_retries() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let (release, held) = mpsc::channel::<()>();
    let server = std::thread::spawn(move || {
        let (mut silent, _) = listener.accept().unwrap();
        let first = read_request(&mut silent);
        let (mut second, _) = listener.accept().unwrap();
        let retry = read_request(&mut second);
        let _ = second.write_all(sse_events(&ANSWER).as_bytes());
        let _ = held.recv_timeout(Duration::from_secs(1)); // `silent` stays open until here
        (first, retry)
    });

    let mut profile = ProviderProfile::new(&url, "sk-test", "claude-test");
    profile.proxy_mode = "direct".into();
    profile.timeouts = ModelTimeouts { response_header_seconds: 1, first_event_seconds: 30, stream_idle_seconds: 30 };
    let agent = Agent::builder(Arc::new(AnthropicMessages::new(profile).unwrap()), Arc::new(MemoryStore::new()))
        .retry(RetryPolicy { max_retries: 2, base_delay: Duration::from_millis(10), max_delay: Duration::from_millis(10) })
        .build()
        .unwrap();

    let mut retries = Vec::new();
    let started = Instant::now();
    let summary = agent
        .run("s", "go", &mut |event| if let Event::Retrying { message, .. } = event { retries.push(message) }, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    let elapsed = started.elapsed();
    let _ = release.send(());
    let (first, retry) = server.join().unwrap();

    assert_eq!(summary.text, "done");
    assert_eq!(first, retry, "the retry sends the identical request");
    assert_eq!(retries.len(), 1, "{retries:?}");
    assert!(retries[0].contains("waiting for response headers"), "{}", retries[0]);
    assert!(elapsed >= Duration::from_secs(1) && elapsed < Duration::from_secs(10), "{elapsed:?}");
}

/// Only the budgets of a stalled phase apply: a stream that keeps producing events for longer than any one budget
/// is healthy, however long it runs in total.
#[test]
fn a_slow_but_steady_stream_outlives_every_single_phase_budget() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        read_request(&mut socket);
        let events = sse_events(&ANSWER);
        let (head, body) = events.split_once("\r\n\r\n").unwrap();
        socket.write_all(format!("{head}\r\n\r\n").as_bytes()).unwrap();
        for part in body.split_inclusive("\n\n") {
            socket.write_all(part.as_bytes()).unwrap();
            socket.flush().unwrap();
            std::thread::sleep(Duration::from_millis(700));
        }
    });

    let mut profile = ProviderProfile::new(&url, "sk-test", "claude-test");
    profile.proxy_mode = "direct".into();
    profile.timeouts = ModelTimeouts { response_header_seconds: 1, first_event_seconds: 1, stream_idle_seconds: 1 };
    let provider = AnthropicMessages::new(profile).unwrap();
    let started = Instant::now();
    let reply = provider
        .chat(&ChatRequest { messages: vec![Message::user_text("go")], tools: vec![] }, &mut |_| {}, &AtomicBool::new(false))
        .unwrap();
    server.join().unwrap();
    assert_eq!(reply.message.text(), "done");
    assert!(started.elapsed() > Duration::from_secs(3), "the stream outlasted three 1s budgets: {:?}", started.elapsed());
}

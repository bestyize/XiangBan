#![cfg(feature = "openai")]
use agent_kernel::provider::openai::OpenAiCompat;
use agent_kernel::provider::wire::ProviderProfile;
use agent_kernel::*;
mod common;

use common::http::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn provider(url: &str) -> OpenAiCompat {
    OpenAiCompat::new(ProviderProfile::new(url, "sk-test", "m1")).unwrap()
}

fn request(text: &str) -> ChatRequest {
    ChatRequest { messages: vec![Message::user_text(text)], tools: vec![] }
}

const HELLO: [&str; 4] = [
    r#"{"choices":[{"delta":{"content":"Hel"}}]}"#,
    r#"{"choices":[{"delta":{"content":"lo"},"finish_reason":"stop"}]}"#,
    r#"{"choices":[],"usage":{"prompt_tokens":7,"completion_tokens":2}}"#,
    "[DONE]",
];

#[test]
fn streams_a_reply_and_sends_auth_and_body() {
    let (url, server) = serve(vec![sse(&HELLO)]);
    let mut streamed = String::new();
    let reply = provider(&url).chat(&request("hi"), &mut |t| streamed.push_str(t), &AtomicBool::new(false)).unwrap();
    assert_eq!((streamed.as_str(), reply.message.text().as_str()), ("Hello", "Hello"));
    assert_eq!(reply.usage, Some(Usage { prompt_tokens: 7, completion_tokens: 2, ..Default::default() }));
    let seen = server.join().unwrap().remove(0);
    assert!(seen.starts_with("POST /v1/chat/completions"));
    assert!(seen.to_lowercase().contains("authorization: bearer sk-test"));
    assert!(seen.contains(r#""model":"m1""#) && seen.contains(r#""stream":true"#));
}

#[test]
fn a_key_source_is_asked_again_for_every_request() {
    let (url, server) = serve(vec![sse(&HELLO), sse(&HELLO), sse(&HELLO)]);
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = calls.clone();
    let source: Arc<dyn Fn() -> String + Send + Sync> = Arc::new(move || match counter.fetch_add(1, Ordering::SeqCst) {
        0 => "token-1".into(),
        1 => "token-2".into(),
        _ => String::new(),
    });
    let p = OpenAiCompat::new(ProviderProfile::new(&url, "static-key", "m1").with_api_key_source(source)).unwrap();
    let cancel = AtomicBool::new(false);
    for _ in 0..3 {
        p.chat(&request("hi"), &mut |_| {}, &cancel).unwrap();
    }
    let seen: Vec<String> = server.join().unwrap().into_iter().map(|s| s.to_lowercase()).collect();
    assert!(seen[0].contains("authorization: bearer token-1"), "{}", seen[0]);
    assert!(seen[1].contains("authorization: bearer token-2"), "{}", seen[1]);
    assert!(seen[2].contains("authorization: bearer static-key"), "an empty answer falls back to the fixed key: {}", seen[2]);
}

#[test]
fn http_errors_map_to_retryable_or_not() {
    let status = |line: &str| format!("HTTP/1.1 {line}\r\nConnection: close\r\nContent-Length: 4\r\n\r\nnope");
    let (url, server) = serve(vec![status("429 Too Many Requests"), status("401 Unauthorized"), status("503 Unavailable")]);
    let p = provider(&url);
    let cancel = AtomicBool::new(false);
    let retryable: Vec<bool> = (0..3)
        .map(|_| match p.chat(&request("x"), &mut |_| {}, &cancel) {
            Err(HarnessError::Provider { retryable, message }) => {
                assert!(message.contains("nope") && !message.contains("sk-test"));
                retryable
            }
            other => panic!("expected provider error, got {other:?}"),
        })
        .collect();
    server.join().unwrap();
    assert_eq!(retryable, vec![true, false, true]);
}

#[test]
fn truncated_stream_is_a_retryable_error() {
    let (url, server) = serve(vec![sse(&[r#"{"choices":[{"delta":{"content":"cut"}}]}"#])]);
    let err = provider(&url).chat(&request("x"), &mut |_| {}, &AtomicBool::new(false)).unwrap_err();
    server.join().unwrap();
    assert!(matches!(err, HarnessError::Provider { retryable: true, .. }));
}

#[test]
fn cancel_returns_promptly_while_the_server_stalls() {
    let first = format!("{SSE}data: {}\n\n", r#"{"choices":[{"delta":{"content":"a"}}]}"#);
    let (url, server) = serve_holding(vec![first], Duration::from_secs(2));
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = cancel.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        trigger.store(true, Ordering::SeqCst);
    });
    let started = Instant::now();
    let err = provider(&url).chat(&request("x"), &mut |_| {}, &cancel).unwrap_err();
    assert_eq!(err, HarnessError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(5));
    drop(server);
}

#[test]
fn agent_runs_a_tool_round_trip_over_http() {
    let call = r#"{"choices":[{"delta":{"reasoning_content":"opaque thought state","tool_calls":[{"index":0,"id":"c1","function":{"name":"echo","arguments":"{\"x\":1}"}}]},"finish_reason":"tool_calls"}]}"#;
    let (url, server) = serve(vec![sse(&[call, "[DONE]"]), sse(&HELLO)]);
    struct Echo(ToolSpec);
    impl Tool for Echo {
        fn spec(&self) -> &ToolSpec {
            &self.0
        }
        fn execute(&self, args: serde_json::Value, _: &ToolCtx) -> Result<String, String> {
            Ok(format!("echo {}", args["x"]))
        }
    }
    let spec = ToolSpec::new("echo", "echoes", serde_json::json!({"type": "object"}));
    let agent = Agent::builder(Arc::new(provider(&url)), Arc::new(MemoryStore::new())).tool(Echo(spec)).build().unwrap();
    let summary = agent.run("s", "go", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!((summary.text.as_str(), summary.turns), ("Hello", 2));
    let second = server.join().unwrap().remove(1);
    assert!(second.contains(r#""tool_call_id":"c1""#) && second.contains("echo 1"));
    assert!(second.contains(r#""reasoning_content":"opaque thought state""#));
}

#[test]fn cancellation_releases_the_stream_socket(){
 use std::{net::TcpListener,io::{Read,Write}};
 let l=TcpListener::bind("127.0.0.1:0").unwrap();let url=format!("http://{}/v1",l.local_addr().unwrap());let(ready,rx)=std::sync::mpsc::channel();
 let server=thread::spawn(move||{let(mut s,_)=l.accept().unwrap();s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();read_request(&mut s);s.write_all(SSE.as_bytes()).unwrap();ready.send(()).unwrap();let mut byte=[0];assert_eq!(s.read(&mut byte).unwrap(),0,"cancel must close the socket");});
 let cancel=Arc::new(AtomicBool::new(false));let c=cancel.clone();let call=thread::spawn(move||provider(&url).chat(&request("x"),&mut |_|{},&c));rx.recv_timeout(Duration::from_secs(2)).unwrap();cancel.store(true,Ordering::SeqCst);assert_eq!(call.join().unwrap().unwrap_err(),HarnessError::Cancelled);server.join().unwrap();
}

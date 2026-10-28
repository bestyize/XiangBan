//! While a model streams the arguments of a big tool call (a 32 KB write_file), nothing else happens for minutes:
//! the host is told how far it got, about once a second, so the page can show it is moving.
#![cfg(all(feature = "anthropic", feature = "openai"))]
mod common;

use agent_kernel::provider::anthropic::AnthropicMessages;
use agent_kernel::provider::responses::OpenAiResponses;
use agent_kernel::provider::wire::ProviderProfile;
use agent_kernel::*;
use common::http::*;
use std::io::Write;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

const PIECES: usize = 6;
const PIECE_BYTES: usize = 5000;

/// Writes `events` one by one, pausing `gap` before each piece of the arguments, like a slow model.
fn slow_server(head: Vec<String>, pieces: Vec<String>, tail: Vec<String>, gap: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        read_request(&mut socket);
        socket.write_all(SSE.as_bytes()).unwrap();
        for part in &head { socket.write_all(part.as_bytes()).unwrap(); }
        socket.flush().unwrap();
        for piece in &pieces { std::thread::sleep(gap); socket.write_all(piece.as_bytes()).unwrap(); socket.flush().unwrap(); }
        for part in &tail { socket.write_all(part.as_bytes()).unwrap(); }
    });
    url
}

fn event(name: &str, data: serde_json::Value) -> String { format!("event: {name}\ndata: {data}\n\n") }

fn request() -> ChatRequest { ChatRequest { messages: vec![Message::user_text("write it")], tools: vec![] } }

fn profile(url: &str) -> ProviderProfile {
    let mut p = ProviderProfile::new(url, "sk-test", "m");
    p.proxy_mode = "direct".into();
    p
}

fn content() -> Vec<String> { (0..PIECES).map(|n| format!("{}{}", "x".repeat(PIECE_BYTES - 1), n)).collect() }

fn check(reports: &[ToolProgress], reply: Reply) {
    assert!(reports.len() >= 3, "{reports:?}");
    assert!(reports.windows(2).all(|w| w[0].bytes < w[1].bytes), "progress only moves forward: {reports:?}");
    assert!(reports.len() <= PIECES + 1, "no event per chunk: {reports:?}");
    assert_eq!((reports[0].name.as_str(), reports[0].path.as_deref()), ("write_file", Some("game-drafts/wind-relics-extension.js")));
    assert!(reports.last().unwrap().bytes >= (PIECES - 1) * PIECE_BYTES, "progress follows the stream to near its end: {reports:?}");
    let call = &reply.message.tool_calls[0];
    assert_eq!(call.name, "write_file");
    assert!(serde_json::from_str::<serde_json::Value>(&call.arguments).unwrap()["content"].as_str().unwrap().len() >= PIECES * PIECE_BYTES);
}

#[test]
fn a_slow_anthropic_tool_input_reports_progress_with_the_target_path() {
    let head = vec![
        event("message_start", serde_json::json!({"type":"message_start","message":{"usage":{"input_tokens":5,"output_tokens":1}}})),
        event("content_block_start", serde_json::json!({"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_1","name":"write_file","input":{}}})),
        event("content_block_delta", serde_json::json!({"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"path\":\"game-drafts/wind-relics-extension.js\",\"content\":\""}})),
    ];
    let pieces = content().into_iter().map(|p| event("content_block_delta", serde_json::json!({"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":p}}))).collect();
    let tail = vec![
        event("content_block_delta", serde_json::json!({"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"\"}"}})),
        event("content_block_stop", serde_json::json!({"type":"content_block_stop","index":0})),
        event("message_delta", serde_json::json!({"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":9}})),
        event("message_stop", serde_json::json!({"type":"message_stop"})),
    ];
    let url = slow_server(head, pieces, tail, Duration::from_millis(300));
    let provider = AnthropicMessages::new(profile(&url)).unwrap();
    let mut reports = Vec::new();
    let reply = provider.chat_with_progress(&request(), &mut |_| {}, &mut |_, _| {}, &mut |p| reports.push(p.clone()), &AtomicBool::new(false)).unwrap();
    check(&reports, reply);
}

#[test]
fn a_slow_responses_tool_input_reports_progress_with_the_target_path() {
    let head = vec![
        event("response.output_item.added", serde_json::json!({"type":"response.output_item.added","output_index":0,"item":{"type":"function_call","id":"fc_1","call_id":"call_1","name":"write_file","arguments":""}})),
        event("response.function_call_arguments.delta", serde_json::json!({"type":"response.function_call_arguments.delta","output_index":0,"item_id":"fc_1","delta":"{\"path\":\"game-drafts/wind-relics-extension.js\",\"content\":\""})),
    ];
    let pieces = content().into_iter().map(|p| event("response.function_call_arguments.delta", serde_json::json!({"type":"response.function_call_arguments.delta","output_index":0,"item_id":"fc_1","delta":p}))).collect();
    let full = format!("{{\"path\":\"game-drafts/wind-relics-extension.js\",\"content\":\"{}\"}}", content().concat());
    let tail = vec![
        event("response.output_item.done", serde_json::json!({"type":"response.output_item.done","output_index":0,"item":{"type":"function_call","id":"fc_1","call_id":"call_1","name":"write_file","arguments":full}})),
        event("response.completed", serde_json::json!({"type":"response.completed","response":{"usage":{"input_tokens":4,"output_tokens":1}}})),
    ];
    let url = slow_server(head, pieces, tail, Duration::from_millis(300));
    let provider = OpenAiResponses::new(profile(&url)).unwrap();
    let mut reports = Vec::new();
    let reply = provider.chat_with_progress(&request(), &mut |_| {}, &mut |_, _| {}, &mut |p| reports.push(p.clone()), &AtomicBool::new(false)).unwrap();
    check(&reports, reply);
}

/// Through the agent: the progress arrives as `tool_input` events in the run, before the tool call itself starts.
#[test]
fn the_run_emits_tool_input_events_before_the_tool_starts() {
    let head = vec![
        event("response.output_item.added", serde_json::json!({"type":"response.output_item.added","output_index":0,"item":{"type":"function_call","id":"fc_1","call_id":"call_1","name":"write_file","arguments":""}})),
        event("response.function_call_arguments.delta", serde_json::json!({"type":"response.function_call_arguments.delta","output_index":0,"item_id":"fc_1","delta":"{\"path\":\"a.js\",\"content\":\""})),
    ];
    let pieces = content().into_iter().map(|p| event("response.function_call_arguments.delta", serde_json::json!({"type":"response.function_call_arguments.delta","output_index":0,"item_id":"fc_1","delta":p}))).collect();
    let full = format!("{{\"path\":\"a.js\",\"content\":\"{}\"}}", content().concat());
    let tail = vec![
        event("response.output_item.done", serde_json::json!({"type":"response.output_item.done","output_index":0,"item":{"type":"function_call","id":"fc_1","call_id":"call_1","name":"write_file","arguments":full}})),
        event("response.completed", serde_json::json!({"type":"response.completed","response":{"usage":{"input_tokens":4,"output_tokens":1}}})),
    ];
    let url = slow_server(head, pieces, tail, Duration::from_millis(300));
    let agent = Agent::builder(std::sync::Arc::new(OpenAiResponses::new(profile(&url)).unwrap()), std::sync::Arc::new(MemoryStore::new()))
        .max_turns(1).build().unwrap();
    let mut events = Vec::new();
    let _ = agent.run("s", "go", &mut |e| events.push(e), &std::sync::Arc::new(AtomicBool::new(false)));
    let inputs: Vec<_> = events.iter().filter(|e| matches!(e, Event::ToolInput { .. })).collect();
    let start = events.iter().position(|e| matches!(e, Event::ToolStart { .. }));
    assert!(inputs.len() >= 3, "{events:?}");
    assert!(matches!(inputs[0], Event::ToolInput { name, path: Some(path), .. } if name == "write_file" && path == "a.js"));
    if let Some(start) = start { assert!(events.iter().rposition(|e| matches!(e, Event::ToolInput { .. })).unwrap() < start); }
}

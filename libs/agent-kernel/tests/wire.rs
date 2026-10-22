use agent_kernel::provider::stream::{SseParser, StreamAccumulator};
use agent_kernel::provider::wire::{build_request_body, ProviderProfile};
use agent_kernel::*;
use serde_json::{json, Value};

fn feed(acc: &mut StreamAccumulator, chunks: &[Value]) -> String {
    let mut streamed = String::new();
    for chunk in chunks {
        acc.push(chunk, &mut |t| streamed.push_str(t)).unwrap();
    }
    streamed
}

#[test]
fn sse_parser_joins_data_lines_and_ignores_noise() {
    let mut p = SseParser::default();
    let lines = [": keep-alive\n", "event: message\n", "data: {\"a\":\n", "data: 1}\n", "\n", "data:[DONE]\n"];
    let events: Vec<_> = lines.iter().filter_map(|l| p.push_line(l)).collect();
    assert_eq!(events, vec!["{\"a\":\n1}".to_string()]);
    assert_eq!(p.flush().as_deref(), Some("[DONE]"), "unterminated final event is released by flush");
    assert_eq!(p.flush(), None);
}

#[test]
fn accumulator_assembles_fragmented_tool_calls_and_trailing_usage() {
    let mut acc = StreamAccumulator::default();
    let streamed = feed(&mut acc, &[
        json!({"choices": [{"delta": {"role": "assistant", "content": "Hi "}}]}),
        json!({"choices": [{"delta": {"content": "there"}}]}),
        json!({"choices": [{"delta": {"tool_calls": [{"index": 0, "id": "call_a", "function": {"name": "get", "arguments": "{\"ci"}}]}}]}),
        json!({"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"name": "get", "arguments": "ty\":\"X\"}"}}]}}]}),
        json!({"choices": [{"delta": {"tool_calls": [{"index": 1, "id": "call_b", "function": {"name": "ping", "arguments": "{}"}}]}}]}),
        json!({"choices": [{"delta": {}, "finish_reason": "tool_calls"}]}),
        json!({"choices": [], "usage": {"prompt_tokens": 10, "completion_tokens": 5}}),
    ]);
    assert_eq!(streamed, "Hi there");
    assert!(acc.is_finished());
    let reply = acc.finish();
    assert_eq!(reply.message.text(), "Hi there");
    assert_eq!(reply.usage, Some(Usage { prompt_tokens: 10, completion_tokens: 5 }));
    let calls = reply.message.tool_calls;
    assert_eq!((calls[0].id.as_str(), calls[0].name.as_str(), calls[0].arguments.as_str()), ("call_a", "get", r#"{"city":"X"}"#));
    assert_eq!((calls[1].id.as_str(), calls[1].name.as_str()), ("call_b", "ping"));
}

#[test]
fn reasoning_content_is_streamed_separately_and_never_mixed_with_answer() {
    let mut acc = StreamAccumulator::default();
    let mut answer = String::new();
    let mut reasoning = String::new();
    for chunk in [
        json!({"choices":[{"delta":{"reasoning_content":"Plan "}}]}),
        json!({"choices":[{"delta":{"reasoning_content":"first","content":"Done."},"finish_reason":"stop"}]}),
    ] {
        acc.push_with_reasoning(
            &chunk, &mut |text| answer.push_str(text),
            &mut |kind, text| { assert_eq!(kind, ReasoningKind::Text); reasoning.push_str(text); }
        ).unwrap();
    }
    assert_eq!(reasoning, "Plan first");
    assert_eq!(answer, "Done.");
    assert_eq!(acc.finish().message.text(), "Done.");
}

#[test]
fn displayable_reasoning_round_trips_without_exposing_opaque_protocol_state() {
    let mut message = Message::assistant("result", vec![]);
    message.reasoning.push(ReasoningSegment { kind: ReasoningKind::Summary, text: "User-visible summary".into() });
    message.provider_state = Some(json!({"opaque":"state"}));
    let saved = serde_json::to_string(&message).unwrap();
    let loaded: Message = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.reasoning, message.reasoning);
    assert_eq!(loaded.provider_state, message.provider_state);
    assert_eq!(serde_json::to_value(Event::ReasoningDelta {
        kind: ReasoningKind::Summary, text: "x".into()
    }).unwrap(), json!({"type":"reasoning_delta","kind":"summary","text":"x"}));
    let legacy: Message = serde_json::from_value(json!({"role":"assistant","parts":[{"type":"text","text":"old"}]})).unwrap();
    assert!(legacy.reasoning.is_empty());
}

#[test]
fn accumulator_tolerates_missing_index_id_and_arguments() {
    let mut acc = StreamAccumulator::default();
    feed(&mut acc, &[json!({"choices": [{"delta": {"tool_calls": [{"function": {"name": "ping", "arguments": ""}}]}, "finish_reason": "stop"}]})]);
    let call = acc.finish().message.tool_calls.remove(0);
    assert_eq!((call.id.as_str(), call.arguments.as_str()), ("call_0", "{}"));
}

#[test]
fn accumulator_surfaces_in_stream_errors_and_incomplete_streams() {
    let mut acc = StreamAccumulator::default();
    let err = acc.push(&json!({"error": {"message": "quota exceeded"}}), &mut |_| {}).unwrap_err();
    assert_eq!(err, HarnessError::provider("quota exceeded", false));
    let mut acc = StreamAccumulator::default();
    feed(&mut acc, &[json!({"choices": [{"delta": {"content": "cut off"}}]})]);
    assert!(!acc.is_finished());
}

#[test]
fn request_body_follows_the_chat_completions_shape() {
    let mut profile = ProviderProfile::new("http://x/v1", "sk-secret", "m1");
    profile.extra_body.insert("temperature".into(), json!(0.2));
    let call = ToolCall { id: "c1".into(), name: "get".into(), arguments: "{}".into() };
    let request = ChatRequest {
        messages: vec![
            Message::system("sys"),
            Message::user(vec![Part::Text { text: "look".into() }, Part::Image { url: "data:image/png;base64,AA".into() }]),
            Message::assistant("", vec![call]),
            Message::tool("c1", "result"),
        ],
        tools: vec![ToolSchema { name: "get".into(), description: "d".into(), parameters: json!({"type": "object"}) }],
    };
    let body = build_request_body(&profile, &request);
    assert_eq!((body["model"].clone(), body["stream"].clone(), body["temperature"].clone()), (json!("m1"), json!(true), json!(0.2)));
    assert_eq!(body["stream_options"], json!({"include_usage": true}));
    assert_eq!(body["messages"][0], json!({"role": "system", "content": "sys"}));
    assert_eq!(body["messages"][1]["content"][1]["image_url"]["url"], "data:image/png;base64,AA");
    assert_eq!(body["messages"][2]["content"], Value::Null);
    assert_eq!(body["messages"][2]["tool_calls"][0]["function"]["name"], "get");
    assert_eq!(body["messages"][3], json!({"role": "tool", "tool_call_id": "c1", "content": "result"}));
    assert_eq!(body["tools"][0]["function"]["name"], "get");

    profile.stream_usage = false;
    profile.supports_tools = false;
    let body = build_request_body(&profile, &request);
    assert!(body.get("stream_options").is_none() && body.get("tools").is_none());
    assert!(!format!("{profile:?}").contains("sk-secret"));
}

#[test]
fn events_and_messages_round_trip_through_json() {
    let delta = serde_json::to_value(Event::TextDelta { text: "x".into() }).unwrap();
    assert_eq!(delta, json!({"type": "text_delta", "text": "x"}));
    assert_eq!(serde_json::to_value(Event::Done).unwrap(), json!({"type": "done"}));
    let message = Message::assistant("hi", vec![ToolCall { id: "1".into(), name: "n".into(), arguments: "{}".into() }]);
    let back: Message = serde_json::from_str(&serde_json::to_string(&message).unwrap()).unwrap();
    assert_eq!(back, message);
}

fn chat_finish(reason: Option<&str>, with_call: bool) -> FinishReason {
    let mut acc = StreamAccumulator::default();
    let mut delta = json!({"content": "x"});
    if with_call {
        delta["tool_calls"] = json!([{"index": 0, "id": "c", "function": {"name": "t", "arguments": "{}"}}]);
    }
    let finish = reason.map_or(Value::Null, |r| json!(r));
    feed(&mut acc, &[json!({"choices": [{"delta": delta, "finish_reason": finish}]})]);
    acc.mark_done();
    acc.finish().finish
}

#[test]
fn chat_finish_reasons_are_mapped_and_settled_against_the_calls() {
    use FinishReason::*;
    let cases = [
        (Some("stop"), false, Stop), (Some("stop"), true, ToolCalls), (Some("tool_calls"), true, ToolCalls),
        (Some("function_call"), true, ToolCalls), (Some("tool_calls"), false, Stop), (Some("length"), false, Length),
        (Some("length"), true, Length), (Some("content_filter"), false, ContentFilter), (Some("insufficient_system_resource"), false, Other),
        (None, false, Stop), (None, true, ToolCalls),
    ];
    for (reason, with_call, expected) in cases {
        assert_eq!(chat_finish(reason, with_call), expected, "finish_reason={reason:?} calls={with_call}");
    }
}

#[test]
fn thinking_chat_tool_history_is_preserved_without_becoming_visible_text() {
    let mut acc = StreamAccumulator::default();
    let visible = feed(&mut acc, &[
        json!({"choices":[{"delta":{"reasoning_content":"first "}}]}),
        json!({"choices":[{"delta":{"reasoning_content":"second", "content":"Checking"}}]}),
        json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"c1","function":{"name":"ping","arguments":"{}"}}]},"finish_reason":"tool_calls"}]}),
    ]);
    assert_eq!(visible, "Checking");
    let message = acc.finish().message;
    assert_eq!(message.text(), "Checking");
    let message: Message = serde_json::from_value(serde_json::to_value(&message).unwrap()).unwrap();
    let req = ChatRequest {messages:vec![Message::user_text("ping"),message.clone(),Message::tool("c1","ok")],tools:vec![]};
    let mut p = ProviderProfile::new("https://proxy.example/v1", "", "kimi-k3");
    p.extra_body.insert("reasoning_effort".into(), json!("high"));
    let body = build_request_body(&p, &req);
    assert_eq!(body["reasoning_effort"], "high");
    assert_eq!(body["messages"][1]["reasoning_content"], "first second");
    assert_eq!(body["messages"][2]["tool_call_id"], "c1");
    let mut other = message;
    other.provider_state.as_mut().unwrap()["source"] = json!({"base_url":"https://other.example/v1","model":"kimi-k3"});
    let body = build_request_body(&p, &ChatRequest {messages:vec![other],tools:vec![]});
    assert!(body["messages"][0].get("reasoning_content").is_none());
}

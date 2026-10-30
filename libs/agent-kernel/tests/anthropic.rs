mod common;

use agent_kernel::provider::anthropic::{build_request_body, endpoint_url, model_max_output_tokens, next_max_tokens_from_error, request_prefix, AnthropicStream};
use agent_kernel::provider::wire::ProviderProfile;
use agent_kernel::*;
use serde_json::{json, Value};

fn profile() -> ProviderProfile {
    ProviderProfile::new("https://api.anthropic.com", "sk-ant-secret", "claude-test")
}

fn call(id: &str, name: &str, args: &str) -> ToolCall {
    ToolCall { id: id.into(), name: name.into(), arguments: args.into() }
}

fn request(messages: Vec<Message>) -> ChatRequest {
    ChatRequest { messages, tools: vec![] }
}

fn feed(stream: &mut AnthropicStream, events: &[Value]) -> String {
    let mut streamed = String::new();
    for event in events {
        stream.push_event(event, &mut |t| streamed.push_str(t)).unwrap();
    }
    streamed
}

#[test]
fn endpoint_follows_the_anthropic_sdk_convention() {
    assert_eq!(endpoint_url("https://api.anthropic.com"), "https://api.anthropic.com/v1/messages");
    assert_eq!(endpoint_url("https://api.anthropic.com/"), "https://api.anthropic.com/v1/messages");
    assert_eq!(endpoint_url("https://api.anthropic.com/v1"), "https://api.anthropic.com/v1/messages");
    assert_eq!(endpoint_url(" https://gateway.example.com/anthropic/ "), "https://gateway.example.com/anthropic/v1/messages");
}

#[test]
fn the_request_has_the_required_fields_and_tools_in_messages_format() {
    let mut p = profile();
    p.extra_body.insert("temperature".into(), json!(0.2));
    let mut req = request(vec![Message::user_text("hi")]);
    req.tools = vec![ToolSchema { name: "get".into(), description: "d".into(), parameters: json!({"type": "object", "properties": {}}) }];
    let body = build_request_body(&p, &req);
    assert_eq!((body["model"].clone(), body["max_tokens"].clone(), body["stream"].clone()), (json!("claude-test"), json!(128000), json!(true)));
    assert_eq!(body["temperature"], json!(0.2));
    assert_eq!(body["tools"], json!([{"name": "get", "description": "d", "input_schema": {"type": "object", "properties": {}}}]));
    assert_eq!(body["messages"], json!([{"role": "user", "content": [{"type": "text", "text": "hi"}]}]));
    assert!(body.get("system").is_none(), "no system field without system messages");

    p.max_output_tokens = Some(1024);
    p.supports_tools = false;
    let body = build_request_body(&p, &req);
    assert_eq!(body["max_tokens"], json!(1024));
    assert!(body.get("tools").is_none());
}

#[test]
fn prompt_cache_marks_tools_system_and_the_newest_block_only_when_enabled() {
    let mut req = request(vec![Message::system("rules"), Message::user_text("first"), Message::assistant("ok", vec![]), Message::user_text("second")]);
    req.tools = vec![
        ToolSchema { name: "a".into(), description: "d".into(), parameters: json!({"type": "object"}) },
        ToolSchema { name: "b".into(), description: "d".into(), parameters: json!({"type": "object"}) },
    ];
    let plain = build_request_body(&profile(), &req);
    assert!(!plain.to_string().contains("cache_control"), "off by default");
    assert_eq!(plain["system"], json!("rules"));

    let mut p = profile();
    p.prompt_cache = true;
    let body = build_request_body(&p, &req);
    let ephemeral = json!({"type": "ephemeral"});
    assert_eq!(body["system"], json!([{"type": "text", "text": "rules", "cache_control": ephemeral}]));
    assert!(body["tools"][0].get("cache_control").is_none());
    assert_eq!(body["tools"][1]["cache_control"], ephemeral);
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.last().unwrap()["content"][0]["cache_control"], ephemeral);
    assert_eq!(body.to_string().matches("cache_control").count(), 3, "at most four breakpoints are allowed");
}

#[test]
fn anthropic_uses_endpoint_metadata_or_a_model_agnostic_protocol_default() {
    for model in ["claude-3-5-sonnet-20241022", "claude-opus-4-6", "new-model-2030", "MiniMax-M3"] {
        assert_eq!(model_max_output_tokens(model), 128_000,
            "model names must not select stale built-in output limits");
    }
    let mut p = profile();
    p.model = "new-model-2030".into();
    assert_eq!(build_request_body(&p, &request(vec![Message::user_text("hi")]))["max_tokens"], json!(128_000));
    p.max_output_tokens = Some(262_144);
    assert_eq!(build_request_body(&p, &request(vec![Message::user_text("hi")]))["max_tokens"], json!(262_144),
        "declared metadata may exceed the protocol fallback");
}

#[test]
fn output_limit_errors_reduce_only_the_generation_cap() {
    let cap = "HTTP 400: max_tokens: 128000 > 64000, which is the maximum allowed number of output tokens";
    assert_eq!(next_max_tokens_from_error(cap, 128_000), Some(63_999));
    assert_eq!(next_max_tokens_from_error("max_tokens cannot exceed the output limit of 16384", 128_000), Some(16_383));
    assert_eq!(next_max_tokens_from_error("range of max_tokens should be [1, 65536]", 128_000), Some(65_535));
    assert_eq!(next_max_tokens_from_error("max_tokens is too large", 128_000), Some(64_000));
    assert_eq!(next_max_tokens_from_error("input token count exceeds context; max_tokens too large", 128_000), None);
    assert_eq!(next_max_tokens_from_error("max_tokens must be greater than zero", 128_000), None);
}

#[test]
fn system_messages_anywhere_are_hoisted_into_the_system_field() {
    let body = build_request_body(&profile(), &request(vec![
        Message::system("You are brief."),
        Message::user_text("hi"),
        Message::assistant("hello", vec![]),
        Message::system("   "),
        Message::system("Extra rule."),
    ]));
    assert_eq!(body["system"], json!("You are brief.\n\nExtra rule."));
    let roles: Vec<&str> = body["messages"].as_array().unwrap().iter().map(|m| m["role"].as_str().unwrap()).collect();
    assert_eq!(roles, vec!["user", "assistant"]);
}

#[test]
fn tool_results_are_merged_into_one_user_message_ahead_of_new_text() {
    let body = build_request_body(&profile(), &request(vec![
        Message::user_text("do two things"),
        Message::assistant("on it", vec![call("a", "t1", r#"{"x":1}"#), call("b", "t2", "{}")]),
        Message::tool("a", "result a"),
        Message::tool("b", ""),
        Message::user_text("and now?"),
    ]));
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[1]["content"], json!([
        {"type": "text", "text": "on it"},
        {"type": "tool_use", "id": "a", "name": "t1", "input": {"x": 1}},
        {"type": "tool_use", "id": "b", "name": "t2", "input": {}},
    ]));
    assert_eq!(messages[2]["role"], "user");
    assert_eq!(messages[2]["content"], json!([
        {"type": "tool_result", "tool_use_id": "a", "content": "result a"},
        {"type": "tool_result", "tool_use_id": "b"},
        {"type": "text", "text": "and now?"},
    ]), "tool results come first, an empty result omits content");
}

#[test]
fn blank_text_invalid_arguments_and_images_are_handled() {
    let body = build_request_body(&profile(), &request(vec![
        Message::user(vec![
            Part::Text { text: "  \n".into() },
            Part::Image { url: "data:image/png;base64,AAAA".into() },
            Part::Image { url: "https://example.com/cat.jpg".into() },
            Part::Text { text: "what is this?".into() },
        ]),
        Message::assistant("", vec![call("a", "t", "{not json"), call("b", "t", "[1,2]")]),
        Message::tool("a", "err"),
        Message::tool("b", "err"),
        Message::assistant("", vec![]),
        Message::user_text(""),
    ]));
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 3, "empty assistant and empty user messages disappear: {messages:#?}");
    assert_eq!(messages[0]["content"], json!([
        {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "AAAA"}},
        {"type": "image", "source": {"type": "url", "url": "https://example.com/cat.jpg"}},
        {"type": "text", "text": "what is this?"},
    ]));
    assert_eq!(messages[1]["content"], json!([
        {"type": "tool_use", "id": "a", "name": "t", "input": {}},
        {"type": "tool_use", "id": "b", "name": "t", "input": {}},
    ]), "arguments that are not a JSON object are replaced by an empty object");
}

#[test]
fn a_full_stream_with_text_and_two_fragmented_tool_calls_is_assembled() {
    let mut stream = AnthropicStream::default();
    let streamed = feed(&mut stream, &[
        json!({"type": "message_start", "message": {"id": "m", "role": "assistant", "content": [], "usage": {"input_tokens": 25, "cache_creation_input_tokens": 100, "cache_read_input_tokens": 50, "output_tokens": 1}}}),
        json!({"type": "ping"}),
        json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Let me "}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "check."}}),
        json!({"type": "content_block_stop", "index": 0}),
        json!({"type": "content_block_start", "index": 1, "content_block": {"type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": {}}}),
        json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": ""}}),
        json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "{\"city\": \"Pa"}}),
        json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "ris\"}"}}),
        json!({"type": "content_block_stop", "index": 1}),
        json!({"type": "content_block_start", "index": 2, "content_block": {"type": "tool_use", "id": "toolu_2", "name": "get_time", "input": {}}}),
        json!({"type": "content_block_delta", "index": 2, "delta": {"type": "input_json_delta", "partial_json": "{}"}}),
        json!({"type": "content_block_stop", "index": 2}),
        json!({"type": "message_delta", "delta": {"stop_reason": "tool_use", "stop_sequence": null}, "usage": {"output_tokens": 42}}),
    ]);
    assert_eq!(streamed, "Let me check.");
    assert!(stream.is_complete(), "a stop reason is enough if the connection closes before message_stop");
    assert_eq!(stream.push_event(&json!({"type": "message_stop"}), &mut |_| {}), Ok(true));
    let reply = stream.finish();
    assert_eq!(reply.message.text(), "Let me check.");
    let calls = reply.message.tool_calls;
    assert_eq!((calls[0].id.as_str(), calls[0].name.as_str(), calls[0].arguments.as_str()), ("toolu_1", "get_weather", "{\"city\": \"Paris\"}"));
    assert_eq!((calls[1].id.as_str(), calls[1].arguments.as_str()), ("toolu_2", "{}"));
    assert_eq!(reply.usage, Some(Usage { prompt_tokens: 175, completion_tokens: 42, cached_prompt_tokens: 50 }), "prompt tokens include cache reads and writes");
}

#[test]
fn unusual_streams_are_tolerated() {
    let mut stream = AnthropicStream::default();
    feed(&mut stream, &[
        json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": ""}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "hmm"}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "signature_delta", "signature": "sig"}}),
        json!({"type": "content_block_delta", "index": 1, "delta": {"type": "text_delta", "text": "no start event"}}),
        json!({"type": "content_block_start", "index": 2, "content_block": {"type": "tool_use", "id": "", "name": "whole", "input": {"a": 1}}}),
        json!({"type": "content_block_start", "index": 3, "content_block": {"type": "tool_use", "id": "t4", "name": "bare", "input": {}}}),
        json!({"type": "content_block_start", "index": 4, "content_block": {"type": "tool_use", "id": "x", "name": "", "input": {}}}),
        json!({"type": "content_block_start", "index": 100000, "content_block": {"type": "text", "text": "dropped"}}),
        json!({"type": "content_block_start", "content_block": {"type": "text", "text": "no index"}}),
        json!({"type": "something_new", "whatever": true}),
    ]);
    let reply = stream.finish();
    assert_eq!(reply.message.text(), "no start event", "thinking blocks are ignored");
    let calls = reply.message.tool_calls;
    assert_eq!(calls.len(), 2);
    assert_eq!((calls[0].id.as_str(), calls[0].arguments.as_str()), ("call_2", r#"{"a":1}"#), "input sent whole at block start is used when no deltas follow");
    assert_eq!((calls[1].id.as_str(), calls[1].arguments.as_str()), ("t4", "{}"));
    assert_eq!(reply.usage, None);
}

#[test]
fn error_events_are_classified_for_retry() {
    for (kind, retryable) in [("overloaded_error", true), ("api_error", true), ("rate_limit_error", true), ("invalid_request_error", false), ("authentication_error", false)] {
        let mut stream = AnthropicStream::default();
        let err = stream.push_event(&json!({"type": "error", "error": {"type": kind, "message": "boom"}}), &mut |_| {}).unwrap_err();
        assert_eq!(err, HarnessError::provider("boom", retryable), "{kind}");
    }
    let mut stream = AnthropicStream::default();
    assert!(matches!(stream.push_payload("not json", &mut |_| {}), Err(HarnessError::Provider { retryable: false, .. })));
    assert!(!stream.is_complete());
}

#[cfg(feature = "anthropic")]
mod over_http {
    use super::*;
    use agent_kernel::provider::anthropic::AnthropicMessages;
    use common::http::*;
    use common::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    const TEXT_REPLY: [(&str, &str); 6] = [
        ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":9,"output_tokens":1}}}"#),
        ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#),
        ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hi "}}"#),
        ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"there"}}"#),
        ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":4}}"#),
        ("message_stop", r#"{"type":"message_stop"}"#),
    ];

    fn chat(profile: ProviderProfile, responses: Vec<String>) -> (Result<Reply, HarnessError>, Vec<String>, String) {
        let (url, server) = serve(responses);
        let mut profile = profile;
        profile.base_url = url.trim_end_matches("/v1").to_string();
        let mut streamed = String::new();
        let req = ChatRequest { messages: vec![Message::system("be brief"), Message::user_text("hi")], tools: vec![] };
        let result = AnthropicMessages::new(profile).unwrap().chat(&req, &mut |t| streamed.push_str(t), &AtomicBool::new(false));
        (result, server.join().unwrap(), streamed)
    }

    #[test]
    fn a_reply_streams_with_the_headers_and_path_claude_endpoints_expect() {
        let (result, seen, streamed) = chat(profile(), vec![sse_events(&TEXT_REPLY)]);
        let reply = result.unwrap();
        assert_eq!((streamed.as_str(), reply.message.text().as_str()), ("Hi there", "Hi there"));
        assert_eq!(reply.usage, Some(Usage { prompt_tokens: 9, completion_tokens: 4, ..Default::default() }));
        let req = &seen[0];
        assert_eq!(request_path(req), "/v1/messages");
        assert_eq!(header_values(req, "x-api-key"), vec!["sk-ant-secret"]);
        assert_eq!(header_values(req, "anthropic-version"), vec!["2023-06-01"]);
        assert!(header_values(req, "authorization").is_empty());
        assert_eq!(body_json(req)["system"], "be brief");
    }

    #[test]
    fn authentication_cache_is_scoped_to_endpoint_and_key() {
        use agent_kernel::provider::anthropic::auth::{self, Header};
        let endpoint = "https://auth-cache-scope.example/v1";
        auth::remember(endpoint, "key-one", Header::Bearer);
        assert_eq!(auth::preferred(endpoint, "key-one"), Header::Bearer);
        assert_eq!(auth::preferred(endpoint, "key-two"), Header::ApiKey);
        assert_eq!(auth::preferred("https://other-cache-scope.example/v1", "key-one"), Header::ApiKey);
    }

    #[test]
    fn unauthorized_retries_with_bearer_and_reuses_the_successful_header() {
        let (url, server) = serve(vec![
            status("401 Unauthorized", r#"{"error":{"message":"unsupported key header"}}"#),
            sse_events(&TEXT_REPLY), sse_events(&TEXT_REPLY),
        ]);
        let mut p = profile(); p.base_url = url;
        let provider = AnthropicMessages::new(p).unwrap();
        let req = ChatRequest { messages: vec![Message::user_text("hi")], tools: vec![] };
        for _ in 0..2 { provider.chat(&req, &mut |_| {}, &AtomicBool::new(false)).unwrap(); }
        let seen = server.join().unwrap();
        assert_eq!(header_values(&seen[0], "x-api-key"), vec!["sk-ant-secret"]);
        for req in &seen[1..] {
            assert_eq!(header_values(req, "authorization"), vec!["Bearer sk-ant-secret"]);
            assert!(header_values(req, "x-api-key").is_empty());
        }
    }

    #[test]
    fn two_rejected_credentials_stop_after_one_alternate() {
        let (result, seen, _) = chat(profile(), vec![
            status("401 Unauthorized", r#"{"error":{"message":"bad key"}}"#),
            status("401 Unauthorized", r#"{"error":{"message":"bad key"}}"#),
        ]);
        assert!(result.is_err());
        assert_eq!(seen.len(), 2);
        assert_eq!(header_values(&seen[0], "x-api-key"), vec!["sk-ant-secret"]);
        assert_eq!(header_values(&seen[1], "authorization"), vec!["Bearer sk-ant-secret"]);
    }

    #[test]
    fn forbidden_does_not_switch_authentication() {
        let (result, seen, _) = chat(profile(), vec![status("403 Forbidden", r#"{"error":{"message":"forbidden"}}"#)]);
        assert!(result.is_err());
        assert_eq!(seen.len(), 1);
        assert_eq!(header_values(&seen[0], "x-api-key"), vec!["sk-ant-secret"]);
    }

    #[test]
    fn explicit_authentication_headers_are_not_retried() {
        let mut p = profile(); p.extra_headers.push(("Authorization".into(), "Bearer gateway-token".into()));
        let (result, seen, _) = chat(p, vec![status("401 Unauthorized", r#"{"error":{"message":"bad key"}}"#)]);
        assert!(result.is_err());
        assert_eq!(seen.len(), 1);
        assert_eq!(header_values(&seen[0], "authorization"), vec!["Bearer gateway-token"]);
    }

    #[test]
    fn extra_headers_replace_defaults_and_an_empty_key_sends_no_key_header() {
        let mut p = profile();
        p.extra_headers = vec![("Anthropic-Version".into(), "2099-01-01".into()), ("anthropic-beta".into(), "tools-2099".into())];
        let (result, seen, _) = chat(p, vec![sse_events(&TEXT_REPLY)]);
        assert!(result.is_ok());
        assert_eq!(header_values(&seen[0], "anthropic-version"), vec!["2099-01-01"], "replaced, not duplicated");
        assert_eq!(header_values(&seen[0], "anthropic-beta"), vec!["tools-2099"]);

        let mut p = ProviderProfile::new("unused", "", "claude-test");
        p.extra_headers = vec![("Authorization".into(), "Bearer gateway-token".into())];
        let (result, seen, _) = chat(p, vec![sse_events(&TEXT_REPLY)]);
        assert!(result.is_ok());
        assert!(header_values(&seen[0], "x-api-key").is_empty());
        assert_eq!(header_values(&seen[0], "authorization"), vec!["Bearer gateway-token"]);
    }

    #[test]
    fn http_errors_are_classified_and_never_echo_the_key() {
        let error = |line: &str| status(line, r#"{"type":"error","error":{"type":"x","message":"details here"}}"#);
        for (line, retryable) in [("429 Too Many Requests", true), ("529 Overloaded", true), ("400 Bad Request", false), ("401 Unauthorized", false)] {
            let responses = if line.starts_with("401 ") { vec![error(line), error(line)] } else { vec![error(line)] };
            let (result, _, _) = chat(profile(), responses);
            match result {
                Err(HarnessError::Provider { message, retryable: r }) => {
                    assert_eq!(r, retryable, "{line}");
                    assert!(message.contains("details here") && !message.contains("sk-ant-secret"), "{message}");
                }
                other => panic!("{line}: {other:?}"),
            }
        }
    }

    #[test]
    fn streams_that_end_early_or_report_errors_are_failures() {
        let (result, _, streamed) = chat(profile(), vec![sse_events(&TEXT_REPLY[..4])]);
        assert!(matches!(result, Err(HarnessError::Provider { retryable: true, .. })), "closed before any stop signal");
        assert_eq!(streamed, "Hi there", "text before the cut was still delivered");
        let overloaded = sse_events(&[("error", r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#)]);
        let (result, _, _) = chat(profile(), vec![overloaded]);
        assert_eq!(result.unwrap_err(), HarnessError::provider("Overloaded", true));
    }

    #[test]
    fn an_agent_completes_a_tool_round_trip_over_the_messages_protocol() {
        let first = sse_events(&[
            ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":5,"output_tokens":1}}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_9","name":"echo","input":{}}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"x\":"}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"7}"}}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":3}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        let (url, server) = serve(vec![first, sse_events(&TEXT_REPLY)]);
        let mut p = profile();
        p.base_url = url;
        let echo = FnTool::new(spec("echo"), |args, _| Ok(format!("echoed {}", args["x"])));
        let agent = Agent::builder(Arc::new(AnthropicMessages::new(p).unwrap()), Arc::new(MemoryStore::new()))
            .system_prompt("be brief")
            .tool(echo)
            .build()
            .unwrap();
        let summary = agent.run("s", "go", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!((summary.text.as_str(), summary.turns), ("Hi there", 2));

        let seen = server.join().unwrap();
        assert_eq!(body_json(&seen[0])["tools"][0]["name"], "echo");
        let second = body_json(&seen[1]);
        let messages = second["messages"].as_array().unwrap();
        assert_eq!(messages[1], json!({"role": "assistant", "content": [{"type": "tool_use", "id": "toolu_9", "name": "echo", "input": {"x": 7}}]}));
        assert_eq!(messages[2], json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": "toolu_9", "content": "echoed 7"}]}));
    }
}

#[test]
fn cumulative_usage_never_goes_backwards() {
    let mut stream = AnthropicStream::default();
    feed(&mut stream, &[
        json!({"type": "message_start", "message": {"usage": {"input_tokens": 10, "cache_read_input_tokens": 90, "output_tokens": 1}}}),
        json!({"type": "message_delta", "delta": {"stop_reason": "end_turn"}, "usage": {"input_tokens": 4000, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0, "output_tokens": 55}}),
        json!({"type": "message_delta", "delta": {}, "usage": {"output_tokens": 20}}),
    ]);
    let reply = stream.finish();
    assert_eq!(reply.usage, Some(Usage { prompt_tokens: 4090, completion_tokens: 55, cached_prompt_tokens: 90 }), "input grows, cache is kept, output never shrinks");
}

fn anthropic_finish(stop_reason: &str, with_tool: bool) -> FinishReason {
    let mut stream = AnthropicStream::default();
    let mut events = vec![json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}})];
    if with_tool {
        events.push(json!({"type": "content_block_start", "index": 1, "content_block": {"type": "tool_use", "id": "t", "name": "get", "input": {}}}));
        events.push(json!({"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "{\"city\": \"Pa"}}));
    }
    events.push(json!({"type": "message_delta", "delta": {"stop_reason": stop_reason}, "usage": {"output_tokens": 5}}));
    feed(&mut stream, &events);
    stream.finish().finish
}

#[test]
fn stop_reasons_are_mapped_to_finish_reasons() {
    use FinishReason::*;
    let cases = [
        ("end_turn", false, Stop), ("stop_sequence", false, Stop), ("tool_use", true, ToolCalls), ("max_tokens", false, Length),
        ("max_tokens", true, Length), ("model_context_window_exceeded", false, Length), ("refusal", false, ContentFilter), ("pause_turn", false, Other),
    ];
    for (reason, with_tool, expected) in cases {
        assert_eq!(anthropic_finish(reason, with_tool), expected, "stop_reason={reason} tool={with_tool}");
    }
    assert_eq!(AnthropicStream::default().finish().finish, Stop, "no stop reason at all counts as a normal stop");
}

#[cfg(feature = "anthropic")]
mod truncated_over_http {
    use super::*;
    use agent_kernel::provider::anthropic::AnthropicMessages;
    use common::http::*;
    use common::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn rejected_large_max_tokens_retries_with_server_declared_limit() {
        let bad = status("400 Bad Request",
            r#"{"error":{"message":"max_tokens: 128000 > 64000, which is the maximum allowed number of output tokens"}}"#);
        let success = sse_events(&[
            ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":5,"output_tokens":1}}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":"ok"}}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":2}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        let (url, server) = serve(vec![bad, success]);
        let provider = AnthropicMessages::new(ProviderProfile::new(url, "k", "unknown-future-model")).unwrap();
        let mut streamed = String::new();
        let reply = provider.chat(&request(vec![Message::user_text("hello")]),
            &mut |text| streamed.push_str(text), &AtomicBool::new(false)).unwrap();
        let seen = server.join().unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(body_json(&seen[0])["max_tokens"], json!(128_000));
        assert_eq!(body_json(&seen[1])["max_tokens"], json!(63_999));
        assert_eq!(streamed, "ok");
        assert_eq!(reply.finish, FinishReason::Stop);
    }

    #[test]
    fn a_provider_reuses_an_observed_cap_on_later_calls_in_the_same_run() {
        let bad = status("400 Bad Request",
            r#"{"error":{"message":"max_tokens: 128000 > 64000, which is the maximum allowed number of output tokens"}}"#);
        let success = || sse_events(&[
            ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":5,"output_tokens":1}}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":"ok"}}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":2}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        let (url, server) = serve(vec![bad, success(), success()]);
        let provider = AnthropicMessages::new(ProviderProfile::new(url, "k", "unknown-model")).unwrap();
        for _ in 0..2 {
            let reply = provider.chat(&request(vec![Message::user_text("hello")]),
                &mut |_| {}, &AtomicBool::new(false)).unwrap();
            assert_eq!(reply.finish, FinishReason::Stop);
        }
        let requests = server.join().unwrap();
        let caps: Vec<_> = requests.iter().map(|r| body_json(r)["max_tokens"].as_u64().unwrap()).collect();
        assert_eq!(caps, vec![128_000, 63_999, 63_999]);
    }

    #[test]
    fn a_tool_call_cut_off_by_max_tokens_is_not_run_and_history_stays_valid() {
        let cut = sse_events(&[
            ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":5,"output_tokens":1}}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_cut","name":"echo","input":{}}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"text\": \"a very lo"}}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"max_tokens"},"usage":{"output_tokens":64}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        let done = sse_events(&[
            ("message_start", r#"{"type":"message_start","message":{"usage":{"input_tokens":9,"output_tokens":1}}}"#),
            ("content_block_start", r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#),
            ("content_block_delta", r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Shorter now."}}"#),
            ("message_delta", r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":3}}"#),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ]);
        let (url, server) = serve(vec![cut, done]);
        let mut p = ProviderProfile::new(url, "k", "claude-test");
        p.max_output_tokens = Some(64);
        let runs = Arc::new(AtomicUsize::new(0));
        let counter = runs.clone();
        let echo = FnTool::new(spec("echo"), move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok("echoed".into())
        });
        let agent = Agent::builder(Arc::new(AnthropicMessages::new(p).unwrap()), Arc::new(MemoryStore::new())).tool(echo).build().unwrap();
        let mut events = Vec::new();
        let summary = agent.run("s", "write it", &mut |e| events.push(e), &Arc::new(AtomicBool::new(false))).unwrap();

        assert_eq!((summary.text.as_str(), summary.finish, summary.turns), ("Shorter now.", FinishReason::Stop, 2));
        assert_eq!(runs.load(Ordering::SeqCst), 0, "the half-written call never ran");
        assert!(!events.contains(&Event::Truncated));
        assert!(events.contains(&Event::Continuing { reason: "tool_call_length".into() }));

        let second = body_json(&server.join().unwrap()[1]);
        let messages = second["messages"].as_array().unwrap();
        assert_eq!(messages[1], json!({"role": "assistant", "content": [{"type": "tool_use", "id": "toolu_cut", "name": "echo", "input": {}}]}));
        let result = &messages[2]["content"][0];
        assert_eq!((result["type"].clone(), result["tool_use_id"].clone()), (json!("tool_result"), json!("toolu_cut")));
        assert!(result["content"].as_str().unwrap().starts_with("Not run: your reply hit the output length limit"));
    }
}

#[test]
fn thinking_streams_as_reasoning_and_goes_back_first_with_its_signature() {
    let mut stream = AnthropicStream::default();
    let mut reasoning = String::new();
    for event in [
        json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": "", "signature": ""}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "先读文件，"}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "再改。"}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "signature_delta", "signature": "sig-abc"}}),
        json!({"type": "content_block_start", "index": 1, "content_block": {"type": "redacted_thinking", "data": "opaque"}}),
        json!({"type": "content_block_start", "index": 2, "content_block": {"type": "tool_use", "id": "t1", "name": "read_file", "input": {}}}),
        json!({"type": "content_block_delta", "index": 2, "delta": {"type": "input_json_delta", "partial_json": "{\"path\":\"a\"}"}}),
        json!({"type": "message_delta", "delta": {"stop_reason": "tool_use"}}),
        json!({"type": "message_stop"}),
    ] {
        stream.push_event_with_reasoning(&event, &mut |_| {}, &mut |_, text| reasoning.push_str(text)).unwrap();
    }
    assert_eq!(reasoning, "先读文件，再改。");
    let mut reply = stream.finish();
    assert_eq!(reply.message.text(), "", "thinking is not answer text");
    assert_eq!(reply.message.provider_state.as_ref().unwrap()["blocks"], json!([
        {"type": "thinking", "thinking": "先读文件，再改。", "signature": "sig-abc"},
        {"type": "redacted_thinking", "data": "opaque"},
    ]));

    // Bound to the endpoint, the model and the request that produced it, the way the client does it.
    let asked = request(vec![Message::user_text("改一下 a")]);
    let state = reply.message.provider_state.as_mut().unwrap();
    state["source"] = json!({"base_url": "https://api.anthropic.com", "model": "claude-test"});
    state["prefix"] = json!(request_prefix(&profile(), &asked));
    let req = request(vec![Message::user_text("改一下 a"), reply.message.clone(), Message::tool("t1", "内容")]);
    let body = build_request_body(&profile(), &req);
    let assistant = &body["messages"][1]["content"];
    assert_eq!(assistant[0]["type"], "thinking");
    assert_eq!(assistant[0]["signature"], "sig-abc");
    assert_eq!(assistant[1]["type"], "redacted_thinking");
    assert_eq!(assistant[2]["type"], "tool_use");

    // An edited history before the reply (trimming, compaction) would be refused with the blocks: they are left out.
    let edited = request(vec![Message::user_text("改一下 a（已压缩）"), reply.message.clone(), Message::tool("t1", "内容")]);
    assert_eq!(build_request_body(&profile(), &edited)["messages"][1]["content"][0]["type"], "tool_use");

    // Same endpoint and model can still be routed to a different provider/account.
    let mut routed = profile(); routed.state_scope = Some("provider-a".into());
    let mut scoped = req.clone();
    scoped.messages[1].provider_state.as_mut().unwrap()["source"]["scope"] = json!("provider-a");
    assert_eq!(build_request_body(&routed, &scoped)["messages"][1]["content"][0]["type"], "thinking");
    routed.state_scope = Some("provider-b".into());
    let switched = build_request_body(&routed, &scoped);
    assert_eq!(switched["messages"][1]["content"][0]["type"], "tool_use");
    assert_eq!(switched["messages"][2]["content"][0]["type"], "tool_result");
    scoped.messages[1].provider_state.as_mut().unwrap().as_object_mut().unwrap().remove("source");
    assert_eq!(build_request_body(&routed, &scoped)["messages"][1]["content"][0]["type"], "tool_use");

    // Another model cannot verify those signatures: its requests leave the blocks out.
    let other = ProviderProfile::new("https://api.anthropic.com", "k", "claude-other");
    let body = build_request_body(&other, &req);
    assert_eq!(body["messages"][1]["content"][0]["type"], "tool_use");
}

#[test]
fn thinking_without_a_signature_is_not_kept() {
    let mut stream = AnthropicStream::default();
    for event in [
        json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": "想了想"}}),
        json!({"type": "content_block_start", "index": 1, "content_block": {"type": "text", "text": "好"}}),
        json!({"type": "message_stop"}),
    ] {
        stream.push_event(&event, &mut |_| {}).unwrap();
    }
    let reply = stream.finish();
    assert_eq!(reply.message.text(), "好");
    assert!(reply.message.provider_state.is_none(), "an unsigned block would be rejected if sent back");
}

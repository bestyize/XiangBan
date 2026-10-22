mod common;

use agent_kernel::provider::responses::{build_request_body, endpoint_url, ResponsesStream};
use agent_kernel::provider::wire::ProviderProfile;
use agent_kernel::*;
use serde_json::{json, Value};

fn profile() -> ProviderProfile {
    ProviderProfile::new("https://api.openai.com/v1", "sk-secret", "gpt-test")
}

fn call(id: &str, name: &str, args: &str) -> ToolCall {
    ToolCall { id: id.into(), name: name.into(), arguments: args.into() }
}

fn feed(stream: &mut ResponsesStream, events: &[Value]) -> String {
    let mut streamed = String::new();
    for event in events {
        stream.push_event(event, &mut |t| streamed.push_str(t)).unwrap();
    }
    streamed
}

#[test]
fn stream_reasoning_summary_delta_done_and_final_reconciliation_do_not_duplicate() {
    let mut stream = ResponsesStream::default();
    let mut visible = String::new();
    let mut answer = String::new();
    let events = [
        json!({"type":"response.reasoning_summary_text.delta","output_index":0,"summary_index":0,"delta":"分析"}),
        json!({"type":"response.reasoning_summary_text.delta","output_index":0,"summary_index":0,"delta":"完成"}),
        json!({"type":"response.reasoning_summary_text.done","output_index":0,"summary_index":0,"text":"分析完成"}),
        json!({"type":"response.output_item.done","output_index":0,"item":{
            "type":"reasoning","status":"completed","id":"rs_test",
            "summary":[{"type":"summary_text","text":"分析完成"}]}}),
        json!({"type":"response.output_text.delta","delta":"回答"}),
        json!({"type":"response.completed","response":{"output":[{
            "type":"reasoning","status":"completed","id":"rs_test",
            "summary":[{"type":"summary_text","text":"分析完成"}]}]}}),
    ];
    for event in &events {
        stream.push_event_with_reasoning(event,
            &mut |t| answer.push_str(t),
            &mut |kind, t| { assert_eq!(kind, ReasoningKind::Summary); visible.push_str(t); }
        ).unwrap();
    }
    assert_eq!(visible, "分析完成");
    assert_eq!(answer, "回答");
    assert!(stream.finish().message.provider_state.is_some());
}

#[test]
fn final_only_responses_reasoning_summary_is_still_displayed() {
    let mut stream = ResponsesStream::default();
    let mut visible = String::new();
    stream.push_event_with_reasoning(&json!({"type":"response.completed","response":{"output":[{
        "type":"reasoning","status":"completed",
        "summary":[{"type":"summary_text","text":"只在最终响应中返回"}]}]}}),
        &mut |_| {}, &mut |kind, t| { assert_eq!(kind, ReasoningKind::Summary); visible.push_str(t); }
    ).unwrap();
    assert_eq!(visible, "只在最终响应中返回");
}

#[test]
fn failed_response_keeps_an_explicit_overflow_code_even_with_a_localized_message() {
    let error = ResponsesStream::default().push_event(&json!({"type":"response.failed", "response":{"error":{"code":"context_length_exceeded", "message":"请压缩会话"}}}), &mut |_| {}).unwrap_err();
    assert!(error.is_context_limit());
}

#[test]
fn endpoint_appends_responses_to_the_base() {
    assert_eq!(endpoint_url("https://api.openai.com/v1"), "https://api.openai.com/v1/responses");
    assert_eq!(endpoint_url("https://gw.example.com/v1/"), "https://gw.example.com/v1/responses");
}

#[test]
fn the_conversation_becomes_typed_input_items() {
    let mut p = profile();
    p.max_output_tokens = Some(2048);
    p.extra_body.insert("reasoning".into(), json!({"effort": "low"}));
    let req = ChatRequest {
        messages: vec![
            Message::system("You are brief."),
            Message::user(vec![Part::Text { text: "look".into() }, Part::Image { url: "data:image/png;base64,AA".into() }, Part::Text { text: " ".into() }]),
            Message::assistant("checking", vec![call("c1", "get", r#"{"a":1}"#), call("c2", "ping", "")]),
            Message::tool("c1", "result one"),
            Message::tool("c2", ""),
            Message::assistant("", vec![]),
            Message::user_text("thanks"),
        ],
        tools: vec![ToolSchema { name: "get".into(), description: "d".into(), parameters: json!({"type": "object", "properties": {"a": {"type": "integer"}}}) }],
    };
    let body = build_request_body(&p, &req);
    assert_eq!((body["model"].clone(), body["stream"].clone(), body["store"].clone()), (json!("gpt-test"), json!(true), json!(false)));
    assert_eq!((body["max_output_tokens"].clone(), body["reasoning"].clone()), (json!(2048), json!({"effort": "low"})));
    assert_eq!(body["input"], json!([
        {"role": "system", "content": "You are brief."},
        {"role": "user", "content": [{"type": "input_text", "text": "look"}, {"type": "input_image", "image_url": "data:image/png;base64,AA", "detail": "auto"}]},
        {"role": "assistant", "content": "checking"},
        {"type": "function_call", "call_id": "c1", "name": "get", "arguments": "{\"a\":1}"},
        {"type": "function_call", "call_id": "c2", "name": "ping", "arguments": "{}"},
        {"type": "function_call_output", "call_id": "c1", "output": "result one"},
        {"type": "function_call_output", "call_id": "c2", "output": ""},
        {"role": "user", "content": [{"type": "input_text", "text": "thanks"}]},
    ]));
    assert_eq!(body["tools"], json!([{"type": "function", "name": "get", "description": "d", "parameters": {"type": "object", "properties": {"a": {"type": "integer"}}}, "strict": false}]));
}

#[test]
fn screenshots_stay_image_inputs_even_for_gateways_that_stringify_tool_outputs() {
    let image_a = format!("data:image/png;base64,{}", "ABCD".repeat(10_000));
    let image_b = "data:image/png;base64,EFGH";
    for trailing_reply in [false, true] {
        let mut first = Message::tool("screen_a", "first screenshot");
        first.parts.push(Part::Image { url: image_a.clone() });
        let mut second = Message::tool("screen_b", "second screenshot");
        second.parts.push(Part::Image { url: image_b.into() });
        let mut messages = vec![
            Message::assistant("", vec![call("screen_a", "screenshot", "{}"), call("screen_b", "screenshot", "{}"), call("text", "read", "{}")]),
            first,
            second,
            Message::tool("text", "file contents"),
        ];
        if trailing_reply {
            messages.push(Message::assistant("screens checked", vec![]));
        }
        let body = build_request_body(&profile(), &ChatRequest { messages, tools: vec![] });
        let input = body["input"].as_array().unwrap();
        // A legacy converter may JSON-stringify any output value. No screenshot bytes may
        // reach that path, and all results must precede the added image message.
        let legacy_tool_text: String = input[3..6].iter().map(|item| {
            assert_eq!(item["type"], "function_call_output");
            assert!(item["output"].is_string());
            item["output"].to_string()
        }).collect();
        assert!(!legacy_tool_text.contains("base64"));
        assert!(legacy_tool_text.len() < 100);
        assert_eq!(input[6]["role"], "user");
        let media = input[6]["content"].as_array().unwrap();
        assert_eq!(media.len(), 4);
        assert!(media[0]["text"].as_str().unwrap().contains("screen_a"));
        assert_eq!(media[1], json!({"type":"input_image", "image_url": image_a, "detail":"auto"}));
        assert!(media[2]["text"].as_str().unwrap().contains("screen_b"));
        assert_eq!(media[3]["image_url"], image_b);
        assert_eq!(input.len(), if trailing_reply { 8 } else { 7 });
        if trailing_reply {
            assert_eq!(input[7]["role"], "assistant");
        }
    }
}

#[test]
fn optional_settings_are_left_out_and_extra_body_can_override_store() {
    let req = ChatRequest { messages: vec![Message::user_text("hi")], tools: vec![] };
    let body = build_request_body(&profile(), &req);
    assert!(body.get("max_output_tokens").is_none() && body.get("tools").is_none());
    let mut p = profile();
    p.extra_body.insert("store".into(), json!(true));
    p.supports_tools = false;
    let with_tools = ChatRequest { tools: vec![ToolSchema { name: "t".into(), description: String::new(), parameters: json!({}) }], ..req };
    let body = build_request_body(&p, &with_tools);
    assert_eq!(body["store"], json!(true));
    assert!(body.get("tools").is_none());
}

#[test]
fn text_and_fragmented_function_calls_are_assembled_with_usage() {
    let mut stream = ResponsesStream::default();
    let streamed = feed(&mut stream, &[
        json!({"type": "response.created", "response": {"id": "resp_1", "status": "in_progress"}}),
        json!({"type": "response.output_item.added", "output_index": 0, "item": {"type": "reasoning", "id": "rs_1"}}),
        json!({"type": "response.reasoning_summary_text.delta", "output_index": 0, "delta": "thinking about it"}),
        json!({"type": "response.output_item.added", "output_index": 1, "item": {"type": "message", "id": "msg_1", "role": "assistant", "content": []}}),
        json!({"type": "response.output_text.delta", "output_index": 1, "content_index": 0, "delta": "Let me "}),
        json!({"type": "response.output_text.delta", "output_index": 1, "content_index": 0, "delta": "look."}),
        json!({"type": "response.output_text.done", "output_index": 1, "text": "Let me look."}),
        json!({"type": "response.output_item.added", "output_index": 2, "item": {"type": "function_call", "id": "fc_1", "call_id": "call_abc", "name": "get_weather", "arguments": ""}}),
        json!({"type": "response.function_call_arguments.delta", "output_index": 2, "item_id": "fc_1", "delta": "{\"city\":"}),
        json!({"type": "response.function_call_arguments.delta", "output_index": 2, "item_id": "fc_1", "delta": "\"Paris\"}"}),
        json!({"type": "response.function_call_arguments.done", "output_index": 2, "item_id": "fc_1", "arguments": "{\"city\":\"Paris\"}"}),
        json!({"type": "response.output_item.done", "output_index": 2, "item": {"type": "function_call", "id": "fc_1", "call_id": "call_abc", "name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}}),
        json!({"type": "response.output_item.added", "output_index": 3, "item": {"type": "function_call", "id": "fc_2", "call_id": "call_def", "name": "get_time", "arguments": ""}}),
        json!({"type": "response.output_item.done", "output_index": 3, "item": {"type": "function_call", "id": "fc_2", "call_id": "call_def", "name": "get_time", "arguments": "{}"}}),
    ]);
    assert_eq!(streamed, "Let me look.", "reasoning summaries are not shown as answer text");
    assert!(!stream.is_complete());
    let done = stream.push_event(&json!({"type": "response.completed", "response": {"status": "completed", "usage": {"input_tokens": 120, "output_tokens": 30, "total_tokens": 150}}}), &mut |_| {});
    assert_eq!(done, Ok(true));
    let reply = stream.finish();
    assert_eq!(reply.message.text(), "Let me look.");
    let calls = reply.message.tool_calls;
    assert_eq!((calls[0].id.as_str(), calls[0].name.as_str(), calls[0].arguments.as_str()), ("call_abc", "get_weather", r#"{"city":"Paris"}"#));
    assert_eq!((calls[1].id.as_str(), calls[1].arguments.as_str()), ("call_def", "{}"));
    assert_eq!(reply.usage, Some(Usage { prompt_tokens: 120, completion_tokens: 30 }));
}

#[test]
fn the_final_item_overrides_partial_arguments_and_missing_added_events_are_tolerated() {
    let mut stream = ResponsesStream::default();
    feed(&mut stream, &[
        json!({"type": "response.function_call_arguments.delta", "output_index": 0, "item_id": "fc_9", "delta": "{\"half"}),
        json!({"type": "response.output_item.done", "output_index": 0, "item": {"type": "function_call", "id": "fc_9", "call_id": "", "name": "search", "arguments": "{\"q\":\"full\"}"}}),
        json!({"type": "response.output_item.done", "output_index": 1, "item": {"type": "function_call", "name": "bare"}}),
        json!({"type": "response.output_item.added", "output_index": 100000, "item": {"type": "function_call", "name": "dropped"}}),
        json!({"type": "response.refusal.delta", "delta": "I can't help with that."}),
        json!({"type": "response.some_future_event", "x": 1}),
    ]);
    let reply = stream.finish();
    assert_eq!(reply.message.text(), "I can't help with that.", "refusals are delivered as text");
    let calls = reply.message.tool_calls;
    assert_eq!(calls.len(), 2);
    assert_eq!((calls[0].id.as_str(), calls[0].arguments.as_str()), ("fc_9", r#"{"q":"full"}"#), "the item id stands in when there is no call_id");
    assert_eq!((calls[1].id.as_str(), calls[1].arguments.as_str()), ("call_1", "{}"));
}

#[test]
fn a_gateway_that_only_sends_the_final_response_still_works() {
    let mut stream = ResponsesStream::default();
    let streamed = feed(&mut stream, &[json!({"type": "response.completed", "response": {"usage": {"input_tokens": 3, "output_tokens": 4}, "output": [
        {"type": "reasoning", "id": "rs"},
        {"type": "message", "content": [{"type": "output_text", "text": "Hello "}, {"type": "output_text", "text": "world"}]},
        {"type": "function_call", "id": "fc", "call_id": "call_z", "name": "t", "arguments": "{\"a\":1}"},
    ]}})]);
    assert_eq!(streamed, "Hello world");
    let reply = stream.finish();
    assert_eq!(reply.message.tool_calls[0].id, "call_z");
    assert_eq!(reply.usage, Some(Usage { prompt_tokens: 3, completion_tokens: 4 }));
}

#[test]
fn incomplete_responses_keep_what_was_produced_and_failures_are_errors() {
    let mut stream = ResponsesStream::default();
    feed(&mut stream, &[json!({"type": "response.output_text.delta", "delta": "partial"})]);
    let done = stream.push_event(&json!({"type": "response.incomplete", "response": {"incomplete_details": {"reason": "max_output_tokens"}}}), &mut |_| {});
    assert_eq!(done, Ok(true));
    assert_eq!(stream.finish().message.text(), "partial");

    for (code, retryable) in [("server_error", true), ("rate_limit_exceeded", true), ("invalid_prompt", false), ("", false)] {
        let mut stream = ResponsesStream::default();
        let event = json!({"type": "response.failed", "response": {"error": {"code": code, "message": "it failed"}}});
        assert_eq!(stream.push_event(&event, &mut |_| {}).unwrap_err(), HarnessError::provider("it failed", retryable), "{code}");
    }
    let mut stream = ResponsesStream::default();
    let event = json!({"type": "error", "code": "server_error", "message": "stream error"});
    assert_eq!(stream.push_event(&event, &mut |_| {}).unwrap_err(), HarnessError::provider("stream error", true));
    assert!(matches!(stream.push_payload("{oops", &mut |_| {}), Err(HarnessError::Provider { retryable: false, .. })));
}

#[cfg(feature = "openai")]
mod over_http {
    use super::*;
    use agent_kernel::provider::responses::OpenAiResponses;
    use common::http::*;
    use common::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    const TEXT_REPLY: [(&str, &str); 4] = [
        ("response.created", r#"{"type":"response.created","response":{"status":"in_progress"}}"#),
        ("response.output_text.delta", r#"{"type":"response.output_text.delta","output_index":0,"delta":"Hi "}"#),
        ("response.output_text.delta", r#"{"type":"response.output_text.delta","output_index":0,"delta":"there"}"#),
        ("response.completed", r#"{"type":"response.completed","response":{"status":"completed","usage":{"input_tokens":6,"output_tokens":2}}}"#),
    ];

    fn chat(profile: ProviderProfile, responses: Vec<String>) -> (Result<Reply, HarnessError>, Vec<String>, String) {
        let (url, server) = serve(responses);
        let mut profile = profile;
        profile.base_url = url;
        let mut streamed = String::new();
        let req = ChatRequest { messages: vec![Message::system("be brief"), Message::user_text("hi")], tools: vec![] };
        let result = OpenAiResponses::new(profile).unwrap().chat(&req, &mut |t| streamed.push_str(t), &AtomicBool::new(false));
        (result, server.join().unwrap(), streamed)
    }

    #[test]
    fn a_reply_streams_to_the_responses_path_with_a_bearer_token() {
        let (result, seen, streamed) = chat(profile(), vec![sse_events(&TEXT_REPLY)]);
        let reply = result.unwrap();
        assert_eq!((streamed.as_str(), reply.message.text().as_str()), ("Hi there", "Hi there"));
        assert_eq!(reply.usage, Some(Usage { prompt_tokens: 6, completion_tokens: 2 }));
        assert_eq!(request_path(&seen[0]), "/v1/responses");
        assert_eq!(header_values(&seen[0], "authorization"), vec!["Bearer sk-secret"]);
        assert_eq!(body_json(&seen[0])["input"][0], json!({"role": "system", "content": "be brief"}));
    }

    #[test]
    fn extra_headers_replace_the_default_authorization() {
        let mut p = profile();
        p.extra_headers = vec![("authorization".into(), "Bearer other".into()), ("OpenAI-Organization".into(), "org-1".into())];
        let (result, seen, _) = chat(p, vec![sse_events(&TEXT_REPLY)]);
        assert!(result.is_ok());
        assert_eq!(header_values(&seen[0], "authorization"), vec!["Bearer other"]);
        assert_eq!(header_values(&seen[0], "openai-organization"), vec!["org-1"]);
    }

    #[test]
    fn http_errors_and_cut_streams_are_classified() {
        for (line, retryable) in [("429 Too Many Requests", true), ("503 Unavailable", true), ("400 Bad Request", false)] {
            let (result, _, _) = chat(profile(), vec![status(line, r#"{"error":{"message":"details here"}}"#)]);
            match result {
                Err(HarnessError::Provider { message, retryable: r }) => {
                    assert_eq!(r, retryable, "{line}");
                    assert!(message.contains("details here") && !message.contains("sk-secret"));
                }
                other => panic!("{line}: {other:?}"),
            }
        }
        let (result, _, _) = chat(profile(), vec![sse_events(&TEXT_REPLY[..3])]);
        assert!(matches!(result, Err(HarnessError::Provider { retryable: true, .. })), "no response.completed");
        let (result, _, _) = chat(profile(), vec![status("413 Payload Too Large", "<html>request entity too large</html>")]);
        let error = result.unwrap_err().to_string();
        assert!(error.contains("request_too_large") && error.contains("减少图片") && !error.contains("<html>"));
    }

    #[test]
    fn an_agent_completes_a_tool_round_trip_over_the_responses_protocol() {
        let first = sse_events(&[
            ("response.output_item.done", r#"{"type":"response.output_item.done","output_index":0,"item":{"type":"reasoning","id":"rs_77","summary":[],"encrypted_content":"encrypted-test-state"}}"#),
            ("response.output_item.added", r#"{"type":"response.output_item.added","output_index":1,"item":{"type":"function_call","id":"fc_1","call_id":"call_77","name":"echo","arguments":""}}"#),
            ("response.function_call_arguments.delta", r#"{"type":"response.function_call_arguments.delta","output_index":1,"item_id":"fc_1","delta":"{\"x\":"}"#),
            ("response.function_call_arguments.delta", r#"{"type":"response.function_call_arguments.delta","output_index":1,"item_id":"fc_1","delta":"5}"}"#),
            ("response.output_item.done", r#"{"type":"response.output_item.done","output_index":1,"item":{"type":"function_call","id":"fc_1","call_id":"call_77","name":"echo","arguments":"{\"x\":5}"}}"#),
            ("response.completed", r#"{"type":"response.completed","response":{"usage":{"input_tokens":4,"output_tokens":1}}}"#),
        ]);
        let (url, server) = serve(vec![first, sse_events(&TEXT_REPLY)]);
        let mut p = profile();
        p.base_url = url;
        let echo = FnTool::new(spec("echo"), |args, _| Ok(format!("echoed {}", args["x"])));
        let agent = Agent::builder(Arc::new(OpenAiResponses::new(p).unwrap()), Arc::new(MemoryStore::new())).tool(echo).build().unwrap();
        let summary = agent.run("s", "go", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!((summary.text.as_str(), summary.turns), ("Hi there", 2));

        let seen = server.join().unwrap();
        let tool = &body_json(&seen[0])["tools"][0];
        assert_eq!((tool["name"].clone(), tool["strict"].clone()), (json!("echo"), json!(false)));
        let input = body_json(&seen[1])["input"].clone();
        assert_eq!(input[1]["encrypted_content"], "encrypted-test-state");
        assert_eq!(input[2], json!({"type": "function_call", "call_id": "call_77", "name": "echo", "arguments": "{\"x\":5}"}));
        assert_eq!(input[3], json!({"type": "function_call_output", "call_id": "call_77", "output": "echoed 5"}));
    }
}

fn responses_finish(event: Value, with_call: bool) -> FinishReason {
    let mut stream = ResponsesStream::default();
    if with_call {
        feed(&mut stream, &[json!({"type": "response.output_item.done", "output_index": 0, "item": {"type": "function_call", "id": "fc", "call_id": "c", "name": "t", "arguments": "{}"}})]);
    }
    feed(&mut stream, &[event]);
    stream.finish().finish
}

#[test]
fn response_statuses_are_mapped_to_finish_reasons() {
    use FinishReason::*;
    let completed = || json!({"type": "response.completed", "response": {}});
    let incomplete = |reason: &str| json!({"type": "response.incomplete", "response": {"incomplete_details": {"reason": reason}}});
    assert_eq!(responses_finish(completed(), false), Stop);
    assert_eq!(responses_finish(completed(), true), ToolCalls);
    assert_eq!(responses_finish(incomplete("max_output_tokens"), false), Length);
    assert_eq!(responses_finish(incomplete("max_output_tokens"), true), Length);
    assert_eq!(responses_finish(incomplete("content_filter"), false), ContentFilter);
    assert_eq!(responses_finish(incomplete("something_else"), false), Other);
    assert_eq!(responses_finish(json!({"type": "response.incomplete", "response": {}}), false), Other);
}

#[test]
fn reasoning_items_survive_streaming_storage_and_the_next_tool_request() {
    let reasoning = json!({"type":"reasoning","id":"rs_1","summary":[],"encrypted_content":"opaque-state","status":"completed"});
    let mut stream = ResponsesStream::default();
    let mut text = String::new();
    stream.push_event(&json!({"type":"response.output_item.done","output_index":0,"item":reasoning}), &mut |s|text.push_str(s)).unwrap();
    stream.push_event(&json!({"type":"response.output_item.done","output_index":1,"item":{"type":"function_call","id":"fc_1","call_id":"c1","name":"ping","arguments":"{}"}}), &mut |_|{}).unwrap();
    // A final response repeats the same reasoning item: it must not be duplicated.
    stream.push_event(&json!({"type":"response.completed","response":{"output":[reasoning],"status":"completed"}}), &mut |_|{}).unwrap();
    let message = stream.finish().message;
    assert!(text.is_empty());
    let message: Message = serde_json::from_value(serde_json::to_value(message).unwrap()).unwrap();
    let request = ChatRequest {messages:vec![Message::user_text("ping"),message,Message::tool("c1","ok")], tools:vec![]};
    let body = build_request_body(&profile(), &request);
    assert_eq!(body["input"][1], reasoning);
    assert_eq!(body["input"][2]["type"], "function_call");
    assert_eq!(body["input"][3]["type"], "function_call_output");
    assert_eq!(body["input"].as_array().unwrap().iter().filter(|v|v["type"] == "reasoning").count(), 1);
    let mut other = request.clone();
    other.messages[1].provider_state.as_mut().unwrap()["source"] = json!({"base_url":"https://elsewhere.example","model":"other"});
    assert_eq!(build_request_body(&profile(), &other)["input"][1]["type"], "function_call");
}

#[test]
fn final_only_reasoning_and_foreign_protocol_state_are_handled() {
    let reasoning = json!({"type":"reasoning","content":[{"type":"reasoning_text","text":"provider state"}]});
    let mut stream = ResponsesStream::default();
    stream.push_event(&json!({"type":"response.completed","response":{"output":[reasoning,{"type":"message","content":[{"type":"output_text","text":"answer"}]}]}}), &mut |_|{}).unwrap();
    let mut message = stream.finish().message;
    assert_eq!(message.text(), "answer");
    assert_eq!(message.provider_state.as_ref().unwrap()["items"][0], reasoning);
    message.provider_state = Some(json!({"protocol":"chat","reasoning_content":"other protocol"}));
    let body = build_request_body(&profile(), &ChatRequest {messages:vec![message],tools:vec![]});
    assert_eq!(body["input"], json!([{"role":"assistant","content":"answer"}]));
}

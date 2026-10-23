mod common;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use serde_json::json;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

fn deferred(counter: Arc<AtomicUsize>) -> Arc<dyn Tool> {
    Arc::new(FnTool::new(ToolSpec::new("mcp_ext_calendar_list", "查询日历会议 calendar events", json!({"type":"object","properties":{"day":{"type":"string"}},"required":["day"],"additionalProperties":false})).risk(Risk::Read), move |args, _| {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(format!("events for {}", args["day"]))
    }))
}

#[test]
fn search_loads_real_schemas_for_the_next_call_and_reuses_them() {
    let count = Arc::new(AtomicUsize::new(0));
    let h = harness(vec![
        Step::tool_call("s", "tool_search", r#"{"query":"日历"}"#),
        Step::tool_call("a", "mcp_ext_calendar_list", r#"{"day":"Monday"}"#),
        Step::tool_call("b", "mcp_ext_calendar_list", r#"{"day":"Tuesday"}"#),
        Step::text("done"),
    ], |b| b.tool(FnTool::new(spec("read_file"), |_, _| Ok("file".into()))).deferred_tool_arc(deferred(count.clone())));
    assert!(run(&h.agent, "查询日历").0.is_ok());
    let requests = h.provider.requests();
    assert!(!requests[0].tools.iter().any(|s| s.name == "mcp_ext_calendar_list"));
    assert!(requests[0].tools.iter().any(|s| s.name == "read_file"));
    assert!(requests[0].messages[0].text().contains("mcp_ext_calendar_list"));
    let loaded = requests[1].tools.iter().find(|s| s.name == "mcp_ext_calendar_list").unwrap();
    assert_eq!(loaded.parameters["required"], json!(["day"]));
    assert_eq!(requests[1].tools, requests[2].tools);
    assert_eq!(count.load(Ordering::SeqCst), 2);
    assert!(!h.store.load("s1").unwrap().iter().any(|m| m.text().contains("## Additional tools")));
    // The same discovered function schema is serializable through all three wire formats;
    // no provider-specific tool_search or generic execution proxy is required.
    let profile = agent_kernel::provider::wire::ProviderProfile::new("https://example.test/v1", "", "test");
    for request in &requests[0..2] {
        let visible = request.tools.iter().any(|s| s.name == "mcp_ext_calendar_list");
        let chat = agent_kernel::provider::wire::build_request_body(&profile, request);
        let responses = agent_kernel::provider::responses::build_request_body(&profile, request);
        let anthropic = agent_kernel::provider::anthropic::build_request_body(&profile, request);
        for body in [chat, responses, anthropic] {
            let names: Vec<_> = body["tools"].as_array().unwrap().iter().map(|tool| tool.get("function").unwrap_or(tool)["name"].as_str().unwrap()).collect();
            assert_eq!(names.contains(&"mcp_ext_calendar_list"), visible);
            assert!(!body.to_string().contains("defer_loading"));
        }
    }
}

#[test]
fn exact_loading_preserves_validation_and_real_permission_gate() {
    let count = Arc::new(AtomicUsize::new(0));
    let seen = Arc::new(AtomicUsize::new(0));
    struct Deny(Arc<AtomicUsize>);
    impl Middleware for Deny {
        fn before_tool(&self, _: &ToolCtx, call: &ToolCall, _: &serde_json::Value, spec: &ToolSpec) -> ToolGate {
            if call.name == "mcp_ext_calendar_list" {
                assert_eq!(spec.name, call.name);
                self.0.fetch_add(1, Ordering::SeqCst);
                return ToolGate::Block("user denied the real MCP tool".into());
            }
            ToolGate::Proceed
        }
    }
    let h = harness(vec![
        Step::tool_call("l", "tool_load", r#"{"names":["mcp_ext_calendar_list"]}"#),
        Step::tool_call("bad", "mcp_ext_calendar_list", "{}"),
        Step::tool_call("ok", "mcp_ext_calendar_list", r#"{"day":"Monday"}"#),
        Step::text("done"),
    ], |b| b.deferred_tool_arc(deferred(count.clone())).middleware(Deny(seen.clone())));
    assert!(run(&h.agent, "go").0.is_ok());
    let messages = tool_messages(&h);
    assert!(messages[1].1.contains("invalid arguments"));
    assert!(messages[2].1.contains("user denied"));
    assert_eq!(seen.load(Ordering::SeqCst), 1);
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[test]
fn guesses_and_unknown_batch_names_do_not_execute_or_partially_activate() {
    let count = Arc::new(AtomicUsize::new(0));
    let h = harness(vec![
        Step::tool_call("guess", "mcp_ext_calendar_list", r#"{"day":"Monday"}"#),
        Step::tool_call("bad", "tool_load", r#"{"names":["mcp_ext_calendar_list","unregistered"]}"#),
        Step::text("done"),
    ], |b| b.deferred_tool_arc(deferred(count.clone())));
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(count.load(Ordering::SeqCst), 0);
    assert!(tool_messages(&h)[0].1.contains("deferred"));
    assert!(h.provider.requests().iter().all(|r| !r.tools.iter().any(|s| s.name == "mcp_ext_calendar_list")));
}

#[test]
fn activation_is_isolated_between_sessions_and_cleared_at_run_end() {
    let h = harness(vec![
        Step::tool_call("l", "tool_load", r#"{"names":["mcp_ext_calendar_list"]}"#),
        Step::text("done"), Step::text("another session"), Step::text("new task"),
    ], |b| b.deferred_tool_arc(deferred(Arc::new(AtomicUsize::new(0)))));
    assert!(run(&h.agent, "first").0.is_ok());
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    h.agent.run("s2", "second", &mut |_| {}, &cancel).unwrap();
    assert!(run(&h.agent, "third").0.is_ok());
    let requests = h.provider.requests();
    for i in [0, 2, 3] { assert!(!requests[i].tools.iter().any(|s| s.name == "mcp_ext_calendar_list")); }
    assert!(requests[1].tools.iter().any(|s| s.name == "mcp_ext_calendar_list"));
}

#[test]
fn catalog_overflow_remains_searchable_and_no_match_is_explicit() {
    let h = harness(vec![
        Step::tool_call("s", "tool_search", r#"{"query":"unique_late_capability"}"#),
        Step::tool_call("n", "tool_search", r#"{"query":"not_in_this_catalog"}"#),
        Step::text("done"),
    ], |mut b| {
        for i in 0..150 {
            let name = format!("tool_{i:03}");
            let desc = if i == 149 { "unique_late_capability".into() } else { "x".repeat(100) };
            b = b.deferred_tool_arc(Arc::new(FnTool::new(ToolSpec::new(name, desc, json!({"type":"object"})).risk(Risk::Read), |_, _| Ok("ok".into()))));
        }
        b
    });
    assert!(h.agent.tool_catalog().contains("omitted"));
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(h.provider.requests()[1].tools.iter().any(|s| s.name == "tool_149"));
    let no_match: serde_json::Value = serde_json::from_str(&tool_messages(&h)[1].1).unwrap();
    assert_eq!(no_match["total"], 0);
}

#[test]
fn eager_only_agents_are_unchanged_and_discovery_names_cannot_collide() {
    let h = harness(vec![Step::text("done")], |b| b.tool(FnTool::new(spec("read_file"), |_, _| Ok("ok".into()))));
    assert_eq!(h.agent.tool_schemas(), h.agent.initial_tool_schemas());
    assert!(h.agent.tool_catalog().is_empty());
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(h.provider.requests()[0].tools.len(), 1);
    let provider = Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(vec![]));
    let result = Agent::builder(provider, Arc::new(MemoryStore::new()))
        .tool(FnTool::new(spec("tool_search"), |_, _| Ok("collision".into())))
        .deferred_tool_arc(deferred(Arc::new(AtomicUsize::new(0)))).build();
    assert!(matches!(result, Err(HarnessError::Config(message)) if message.contains("duplicate")));
}

#[test]
fn deferred_schemas_reduce_initial_budget_for_a_large_catalog() {
    let h = harness(vec![], |mut b| {
        for i in 0..60 {
            b = b.deferred_tool_arc(Arc::new(FnTool::new(ToolSpec::new(format!("desktop_{i}"), "desktop appearance action", json!({"type":"object","properties":{"content":{"type":"string","description":"detailed parameter documentation ".repeat(40)}}})).risk(Risk::Read), |_, _| Ok("ok".into()))));
        }
        b
    });
    let full = estimate_tool_tokens(&h.agent.tool_schemas());
    let initial = estimate_tool_tokens(&h.agent.initial_tool_schemas()) + estimate_tokens(&[Message::system(h.agent.tool_catalog())]);
    assert!(initial * 3 < full, "initial={initial}; full={full}");
}

#[test]
fn provider_failure_and_cancellation_clear_activation_before_the_next_task() {
    for error in [HarnessError::provider("offline", false), HarnessError::Cancelled] {
        let h = harness(vec![
            Step::tool_call("l", "tool_load", r#"{"names":["mcp_ext_calendar_list"]}"#),
            Step::error(error), Step::text("recovered"),
        ], |b| b.deferred_tool_arc(deferred(Arc::new(AtomicUsize::new(0)))));
        assert!(run(&h.agent, "first").0.is_err());
        assert!(run(&h.agent, "next").0.is_ok());
        let requests = h.provider.requests();
        assert!(requests[1].tools.iter().any(|s| s.name == "mcp_ext_calendar_list"));
        assert!(!requests[2].tools.iter().any(|s| s.name == "mcp_ext_calendar_list"));
    }
}

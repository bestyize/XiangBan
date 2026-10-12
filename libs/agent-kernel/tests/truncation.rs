mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn counting_tool(runs: &Arc<AtomicUsize>) -> FnTool {
    let runs = runs.clone();
    FnTool::new(spec("echo"), move |_, _| {
        runs.fetch_add(1, Ordering::SeqCst);
        Ok("echoed".into())
    })
}

#[test]
fn a_cut_off_answer_is_returned_and_flagged() {
    let h = harness(vec![Step::text("The answer begins").finish(FinishReason::Length)], |b| b);
    let (result, events) = run(&h.agent, "hi");
    let summary = result.unwrap();
    assert_eq!((summary.text.as_str(), summary.finish), ("The answer begins", FinishReason::Length));
    let kinds: Vec<&Event> = events.iter().filter(|e| matches!(e, Event::Truncated | Event::Done)).collect();
    assert_eq!(kinds, vec![&Event::Truncated, &Event::Done]);
}

#[test]
fn normal_replies_report_stop_and_emit_no_truncation() {
    let h = harness(vec![Step::text("complete")], |b| b);
    let (result, events) = run(&h.agent, "hi");
    assert_eq!(result.unwrap().finish, FinishReason::Stop);
    assert!(!events.contains(&Event::Truncated));
    for reason in [FinishReason::ContentFilter, FinishReason::Other] {
        let h = harness(vec![Step::text("x").finish(reason)], |b| b);
        let (result, events) = run(&h.agent, "hi");
        assert_eq!(result.unwrap().finish, reason);
        assert!(!events.contains(&Event::Truncated), "{reason:?} is not a length cut-off");
    }
}

#[test]
fn tool_calls_in_a_cut_off_reply_are_not_run_and_the_model_can_retry() {
    let runs = Arc::new(AtomicUsize::new(0));
    let steps = vec![
        Step::text("Let me write that").and_tool_call("a", "echo", r#"{"complete": true}"#).and_tool_call("b", "echo", "{\"text\": \"a very long").finish(FinishReason::Length),
        Step::tool_call("c", "echo", r#"{"short": true}"#),
        Step::text("recovered"),
    ];
    let h = harness(steps, |b| b.tool(counting_tool(&runs)));
    let (result, events) = run(&h.agent, "go");
    let summary = result.unwrap();
    assert_eq!((summary.text.as_str(), summary.finish, summary.turns), ("recovered", FinishReason::Stop, 3));
    assert_eq!(runs.load(Ordering::SeqCst), 1, "only the retried call ran; the two from the cut-off reply did not");

    assert_eq!(events.iter().filter(|e| matches!(e, Event::Truncated)).count(), 1);
    let rejected: Vec<&Event> = events.iter().filter(|e| matches!(e, Event::ToolEnd { is_error: true, .. })).collect();
    assert_eq!(rejected.len(), 2, "both calls are reported as failed");

    let history = h.store.load("s1").unwrap();
    let assistant = &history[1];
    assert_eq!(assistant.tool_calls[0].arguments, r#"{"complete": true}"#, "complete arguments are kept as sent");
    assert_eq!(assistant.tool_calls[1].arguments, "{}", "half-written arguments are replaced so history stays valid JSON");
    let results: Vec<(Option<&str>, String)> = history.iter().filter(|m| m.role == Role::Tool).map(|m| (m.tool_call_id.as_deref(), m.text())).collect();
    assert_eq!(results.len(), 3);
    for (id, text) in &results[..2] {
        assert!(matches!(id, Some("a" | "b")) && text.starts_with("Not run: your reply hit the output length limit"), "{text}");
    }
    assert_eq!(results[2], (Some("c"), "echoed".to_string()));
}

#[test]
fn a_truncated_reply_with_calls_still_gets_the_tool_middleware_treatment() {
    struct Mark;
    impl Middleware for Mark {
        fn after_tool(&self, _session: &str, _call: &ToolCall, result: &mut ToolResult) {
            result.output.push_str(" [seen]");
        }
    }
    let runs = Arc::new(AtomicUsize::new(0));
    let steps = vec![Step::tool_call("a", "echo", "{\"cut").finish(FinishReason::Length), Step::text("ok")];
    let h = harness(steps, |b| b.tool(counting_tool(&runs)).middleware(Mark));
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(tool_messages(&h)[0].1.ends_with("[seen]"));
    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

#[test]
fn a_model_that_keeps_getting_cut_off_ends_at_the_turn_limit() {
    let runs = Arc::new(AtomicUsize::new(0));
    let cut = || Step::tool_call("x", "echo", "{\"").finish(FinishReason::Length);
    let h = harness(vec![cut(), cut(), cut()], |b| b.tool(counting_tool(&runs)).max_turns(3));
    assert_eq!(run(&h.agent, "go").0.unwrap_err(), HarnessError::MaxTurns(3));
    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

#[test]
fn wire_forms_are_stable() {
    assert_eq!(serde_json::to_value(Event::Truncated).unwrap(), json!({"type": "truncated"}));
    let reasons = [(FinishReason::Stop, "stop"), (FinishReason::ToolCalls, "tool_calls"), (FinishReason::Length, "length"), (FinishReason::ContentFilter, "content_filter"), (FinishReason::Other, "other")];
    for (reason, text) in reasons {
        assert_eq!(serde_json::to_value(reason).unwrap(), json!(text));
        assert_eq!(serde_json::from_value::<FinishReason>(json!(text)).unwrap(), reason);
    }
}

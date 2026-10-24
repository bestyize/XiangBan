mod common;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

fn counted(count: Arc<AtomicUsize>) -> impl FnOnce(AgentBuilder) -> AgentBuilder {
    move |b| b.tool(FnTool::new(spec("write_big_file"), move |_, _| { count.fetch_add(1, Ordering::SeqCst); Ok("written".into()) }))
}

#[test]
fn arguments_cut_off_mid_json_are_treated_as_a_length_limit_even_when_the_provider_says_tool_calls() {
    let count = Arc::new(AtomicUsize::new(0));
    // Default finish for a scripted tool call is ToolCalls: the provider did not report the cut-off.
    let h = harness(vec![
        Step::tool_call("big", "write_big_file", r#"{"path":"a.html","content":"<html><body>never closed"#),
        Step::tool_call("small", "write_big_file", r#"{"path":"a.html"}"#),
        Step::text("done"),
    ], counted(count.clone()));
    let (result, events) = run(&h.agent, "make a game");
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(count.load(Ordering::SeqCst), 1, "the truncated call never ran; the complete retry did");
    let messages = tool_messages(&h);
    assert!(messages[0].1.contains("output length limit"), "{:?}", messages[0]);
    assert!(events.iter().any(|e| matches!(e, Event::Continuing { reason } if reason == "tool_call_length")));
}

#[test]
fn a_model_that_keeps_sending_cut_off_calls_is_paused_not_looped_forever() {
    let count = Arc::new(AtomicUsize::new(0));
    let cut = r#"{"path":"a.html","content":"<html"#;
    let h = harness(
        (0..8).map(|i| Step::tool_call(&format!("c{i}"), "write_big_file", cut)).collect(),
        counted(count.clone()),
    );
    let (result, _) = run(&h.agent, "make a game");
    assert!(matches!(result, Err(HarnessError::Paused(ref r)) if r == "output_length_limit"), "{result:?}");
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

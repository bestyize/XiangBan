mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Gate {
    ran: Arc<AtomicBool>,
}

impl Middleware for Gate {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        turn.messages.push(Message::system("injected for this request only"));
        Ok(())
    }

    fn after_model(&self, _turn: &Turn, reply: &mut Message) -> Result<(), HarnessError> {
        if reply.tool_calls.is_empty() {
            reply.parts = vec![Part::Text { text: reply.text().to_uppercase() }];
        }
        Ok(())
    }

    fn before_tool(&self, _ctx: &ToolCtx, call: &ToolCall, _args: &Value, spec: &ToolSpec) -> ToolGate {
        if spec.risk == Risk::Exec {
            return ToolGate::Block(format!("`{}` needs approval", call.name));
        }
        self.ran.store(true, Ordering::SeqCst);
        ToolGate::Proceed
    }

    fn after_tool(&self, _session: &str, _call: &ToolCall, result: &mut ToolResult) {
        result.output.push_str(" [checked]");
    }
}

#[test]
fn middleware_can_block_tools_and_reshape_requests_and_replies() {
    let ran = Arc::new(AtomicBool::new(false));
    let danger = FnTool::new(spec("danger").risk(Risk::Exec), |_, _| Ok("should not run".into()));
    let safe = FnTool::new(spec("safe"), |_, _| Ok("fine".into()));
    let steps = vec![Step::tool_call("a", "danger", "{}").and_tool_call("b", "safe", "{}"), Step::text("done")];
    let h = harness(steps, |b| b.tool(danger).tool(safe).middleware(Gate { ran: ran.clone() }));
    let (result, _) = run(&h.agent, "go");
    assert_eq!(result.unwrap().text, "DONE");
    let messages = tool_messages(&h);
    assert_eq!(messages[0].1, "`danger` needs approval [checked]");
    assert_eq!(messages[1].1, "fine [checked]");
    assert!(ran.load(Ordering::SeqCst));
    let injected = |m: &Message| m.text() == "injected for this request only";
    assert!(h.provider.requests().iter().all(|r| r.messages.iter().any(injected)));
    assert!(!h.store.load("s1").unwrap().iter().any(injected), "per-request edits are not persisted");
    assert_eq!(h.store.load("s1").unwrap().last().unwrap().text(), "DONE");
}

fn retryable(msg: &str) -> Step {
    Step::error(HarnessError::provider(msg, true))
}

#[test]
fn explicit_context_overflow_recovers_without_replaying_actions_and_keeps_the_smaller_budget() {
    use std::sync::atomic::AtomicUsize;
    let executed = Arc::new(AtomicUsize::new(0));
    let counter = executed.clone();
    let call = |id: &str| ToolCall { id: id.into(), name: "act".into(), arguments: "{}".into() };
    let h = harness(vec![
        Step::tool_call("a", "act", "{}"),
        Step::error(HarnessError::provider("HTTP 400: context_length_exceeded", false)),
        Step::tool_call("b", "act", "{}"),
        Step::text("done"),
    ], |b| b.tool(FnTool::new(spec("act"), move |_, _| { counter.fetch_add(1, Ordering::SeqCst); Ok("performed".into()) })));
    let old = vec![Message::user_text("old question"), Message::assistant("", vec![call("old")]), Message::tool("old", "x".repeat(80_000)), Message::assistant("old done", vec![])];
    h.store.append("s1", &old).unwrap();
    let (result, events) = run(&h.agent, "continue");
    assert_eq!(result.unwrap().text, "done");
    assert_eq!(executed.load(Ordering::SeqCst), 2, "each newly requested action runs exactly once");
    let requests = h.provider.requests();
    assert!(estimate_tokens(&requests[2].messages) < estimate_tokens(&requests[1].messages));
    assert!(requests[3].messages.iter().all(|m| m.text().len() < 80_000), "later tool rounds keep the reduced budget");
    assert_eq!(h.store.load("s1").unwrap()[2], old[2], "stored history is intact");
    assert_eq!(events.iter().filter(|e| matches!(e, Event::Retrying { delay_ms: 0, .. })).count(), 1);
    for request in requests {
        let calls: Vec<_> = request.messages.iter().flat_map(|m| m.tool_calls.iter().map(|c| &c.id)).collect();
        let results: Vec<_> = request.messages.iter().filter_map(|m| m.tool_call_id.as_ref()).collect();
        assert_eq!(calls, results);
    }
}

#[test]
fn overflow_recovery_preserves_user_requests_and_never_retries_generic_400_or_partial_answers() {
    let fail = || Step::error(HarnessError::provider("prompt is too long", false));
    let h = harness(vec![fail(), fail(), fail(), Step::text("must not run")], |b| b);
    for _ in 0..4 { h.store.append("s1", &[Message::user_text("x".repeat(20_000)), Message::assistant("old answer", vec![])]).unwrap(); }
    let (result, _) = run(&h.agent, "continue");
    assert!(result.unwrap_err().is_context_limit());
    // Old user requests are pinned as constraints, each cut to its start when very long, so a recovery really
    // shrinks the request. An unchanged request is never sent again, and the error is reported when recoveries run out.
    let requests = h.provider.requests();
    assert_eq!(requests.len(), 3, "two recoveries, then the error is reported");
    let sizes: Vec<usize> = requests.iter().map(|r| estimate_tokens(&r.messages)).collect();
    assert!(sizes[0] > sizes[1] && sizes[1] > sizes[2], "every retry is smaller than the one before: {sizes:?}");
    for step in [Step::error(HarnessError::provider("HTTP 400: bad_request", false)), Step::text("partial").then_fail(HarnessError::provider("context_length_exceeded", false))] {
        let h = harness(vec![step, Step::text("must not run")], |b| b);
        assert!(run(&h.agent, "continue").0.is_err());
        assert_eq!(h.provider.requests().len(), 1);
    }
}

#[test]
fn retryable_errors_are_retried_with_events() {
    let h = harness(vec![retryable("busy"), retryable("busy"), Step::text("ok")], |b| b);
    let (result, events) = run(&h.agent, "hi");
    assert_eq!(result.unwrap().text, "ok");
    let attempts: Vec<u32> = events.iter().filter_map(|e| if let Event::Retrying { attempt, .. } = e { Some(*attempt) } else { None }).collect();
    assert_eq!(attempts, vec![1, 2]);
    assert_eq!(h.provider.requests().len(), 3);
}

#[test]
fn non_retryable_errors_and_exhausted_retries_fail() {
    let h = harness(vec![Step::error(HarnessError::provider("bad key", false)), Step::text("ok")], |b| b);
    let (result, events) = run(&h.agent, "hi");
    assert!(matches!(result, Err(HarnessError::Provider { retryable: false, .. })));
    assert!(matches!(events.last(), Some(Event::Error { message }) if message.contains("bad key")));
    assert_eq!(h.provider.requests().len(), 1);

    let h = harness((0..5).map(|_| retryable("busy")).collect(), |b| b);
    assert!(run(&h.agent, "hi").0.is_err());
    assert_eq!(h.provider.requests().len(), 4, "first try plus three retries");
}

#[test]
fn interrupted_text_is_preserved_and_continued_instead_of_replayed() {
    let drop = Step::text("par").then_fail(HarnessError::provider("connection dropped", true));
    let h = harness(vec![drop, Step::text("ok")], |b| b);
    let (result, events) = run(&h.agent, "hi");
    assert_eq!(result.unwrap().text, "ok");
    let requests = h.provider.requests();
    assert_eq!(requests.len(), 2);
    assert!(requests[1].messages.iter().any(|m| m.role == Role::Assistant && m.text() == "par"));
    assert!(events.iter().any(|e| matches!(e, Event::Continuing { reason } if reason == "provider_interrupted")));
    assert!(events.contains(&Event::TextDelta { text: "par".into() }));
}

#[test]
fn interrupted_streams_have_a_run_wide_limit_and_never_replay_finished_tools() {
    let executed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = executed.clone();
    let mut steps = vec![Step::tool_call("saved", "write", "{}")];
    steps.extend((0..5).map(|_| Step::text("working").then_fail(HarnessError::provider("idle timeout", true))));
    let h = harness(steps, |b| b.tool(FnTool::new(spec("write"), move |_, _| {
        count.fetch_add(1, Ordering::SeqCst); Ok("saved".into())
    })));
    let (result, events) = run(&h.agent, "finish");
    assert!(result.is_err());
    assert_eq!(executed.load(Ordering::SeqCst), 1);
    assert_eq!(h.provider.requests().len(), 5, "one tool request, four stream attempts");
    assert_eq!(events.iter().filter(|e| matches!(e, Event::Continuing {reason} if reason == "provider_interrupted")).count(), 3);
    let saved = h.store.load("s1").unwrap();
    assert!(saved.last().unwrap().structured_content.as_ref().unwrap()["interrupted"] == true);
}

#[test]
fn interrupted_stream_continuation_keeps_the_original_token_budget() {
    let h = harness(vec![Step::text("partial").then_fail(HarnessError::provider("timeout", true)), Step::text("must not run")], |b| b);
    let control = RunControl::bounded(Duration::from_secs(5), 1);
    let result = h.agent.run_parts_observed("s1", vec![Part::Text { text: "finish".into() }],
        RunOptions {control:Some(control.clone()), ..Default::default()}, &mut |_| Ok(()), &Arc::new(AtomicBool::new(false)));
    assert!(matches!(result, Err(HarnessError::BudgetExceeded(_))));
    assert_eq!(h.provider.requests().len(), 1);
    assert!(control.reported_tokens() > 1);
}

#[test]
fn cancel_interrupts_retry_backoff() {
    let slow = RetryPolicy { max_retries: 3, base_delay: Duration::from_secs(30), max_delay: Duration::from_secs(30) };
    let h = harness(vec![retryable("busy"), Step::text("ok")], |b| b.retry(slow));
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = cancel.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        trigger.store(true, Ordering::SeqCst);
    });
    let started = Instant::now();
    let (result, _) = run_with_cancel(&h.agent, "hi", &cancel);
    canceller.join().unwrap();
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(5));
}

/// A model that thinks aloud and then goes quiet: the first call streams some reasoning and stalls, the next answers.
struct StallsAfterThinking {
    calls: std::sync::atomic::AtomicUsize,
    stall_after_text: bool,
}

impl ModelProvider for StallsAfterThinking {
    fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
        self.chat_with_reasoning(request, on_text, &mut |_, _| {}, cancel)
    }

    fn chat_with_reasoning(
        &self, _request: &ChatRequest, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(ReasoningKind, &str), _cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        if self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
            on_reasoning(ReasoningKind::Summary, "先想想怎么写脚本");
            if self.stall_after_text {
                on_text("部分回答");
            }
            return Err(HarnessError::provider("HTTP transport timed out", true));
        }
        on_text("写好了");
        Ok(Reply { message: Message::assistant("写好了", vec![]), usage: None, finish: FinishReason::Stop })
    }
}

#[test]
fn a_stall_after_reasoning_but_before_any_text_is_retried_and_only_the_successful_attempt_is_kept() {
    let provider = Arc::new(StallsAfterThinking { calls: Default::default(), stall_after_text: false });
    let store = Arc::new(MemoryStore::new());
    let agent = Agent::builder(provider.clone(), store.clone()).retry(fast_retry()).build().unwrap();
    let (result, events) = run(&agent, "hi");
    assert_eq!(result.unwrap().text, "写好了");
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert!(events.iter().any(|e| matches!(e, Event::Retrying { attempt: 1, .. })), "the page is told, so it can drop the reasoning shown");
    let saved = store.load("s1").unwrap();
    let answer = saved.last().unwrap();
    assert_eq!(answer.text(), "写好了");
    assert!(answer.reasoning.is_empty(), "the failed attempt's reasoning is not stored: {:?}", answer.reasoning);
}

#[test]
fn a_stall_after_answer_text_continues_from_saved_output() {
    let provider = Arc::new(StallsAfterThinking { calls: Default::default(), stall_after_text: true });
    let agent = Agent::builder(provider.clone(), Arc::new(MemoryStore::new())).retry(fast_retry()).build().unwrap();
    let (result, _) = run(&agent, "hi");
    assert_eq!(result.unwrap().text, "写好了");
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
}

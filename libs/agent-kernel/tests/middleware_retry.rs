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
fn no_retry_once_text_has_been_streamed() {
    let drop = Step::text("par").then_fail(HarnessError::provider("connection dropped", true));
    let h = harness(vec![drop, Step::text("ok")], |b| b);
    let (result, events) = run(&h.agent, "hi");
    assert!(result.is_err());
    assert_eq!(h.provider.requests().len(), 1);
    assert!(events.contains(&Event::TextDelta { text: "par".into() }));
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

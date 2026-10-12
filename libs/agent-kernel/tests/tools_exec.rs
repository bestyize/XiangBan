mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A tool that records how many copies of it ran at the same time.
fn probe(name: &str, serial: bool, active: &Arc<AtomicUsize>, peak: &Arc<AtomicUsize>) -> FnTool {
    let (active, peak, label) = (active.clone(), peak.clone(), name.to_string());
    let mut s = spec(name);
    if serial {
        s = s.serial();
    }
    FnTool::new(s, move |_, _| {
        let now = active.fetch_add(1, Ordering::SeqCst) + 1;
        peak.fetch_max(now, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(150));
        active.fetch_sub(1, Ordering::SeqCst);
        Ok(label.clone())
    })
}

fn concurrency(serial: bool) -> (usize, Vec<(String, String)>) {
    let (active, peak) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let steps = vec![Step::tool_call("c1", "a", "{}").and_tool_call("c2", "b", "{}"), Step::text("ok")];
    let h = harness(steps, |b| b.tool(probe("a", serial, &active, &peak)).tool(probe("b", serial, &active, &peak)));
    assert!(run(&h.agent, "go").0.is_ok());
    (peak.load(Ordering::SeqCst), tool_messages(&h))
}

#[test]
fn parallel_safe_tools_overlap_and_keep_call_order() {
    let (peak, messages) = concurrency(false);
    assert_eq!(peak, 2);
    assert_eq!(messages, vec![("c1".into(), "a".into()), ("c2".into(), "b".into())]);
}

#[test]
fn serial_tools_never_overlap() {
    assert_eq!(concurrency(true).0, 1);
}

#[test]
fn timeout_cancels_the_call_and_reports_an_error() {
    let stopped = Arc::new(AtomicBool::new(false));
    let flag = stopped.clone();
    let tool = FnTool::new(spec("hang").timeout(Duration::from_millis(100)), move |_, ctx| {
        while !ctx.is_cancelled() {
            std::thread::sleep(Duration::from_millis(10));
        }
        flag.store(true, Ordering::SeqCst);
        Ok("late".into())
    });
    let h = harness(vec![Step::tool_call("c", "hang", "{}"), Step::text("ok")], |b| b.tool(tool));
    let started = Instant::now();
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(tool_messages(&h)[0].1.contains("timed out"));
    std::thread::sleep(Duration::from_millis(100));
    assert!(stopped.load(Ordering::SeqCst), "tool saw its cancellation flag");
}

#[test]
fn panicking_tool_becomes_an_error_result() {
    let tool = FnTool::new(spec("boom"), |_, _| panic!("tool bug"));
    let h = harness(vec![Step::tool_call("c", "boom", "{}"), Step::text("ok")], |b| b.tool(tool));
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(tool_messages(&h)[0].1.contains("crashed"));
}

#[test]
fn cancel_during_a_tool_leaves_a_paired_history() {
    let tool = FnTool::new(spec("wait"), |_, ctx| {
        while !ctx.is_cancelled() {
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok("stopped".into())
    });
    let h = harness(vec![Step::tool_call("c", "wait", "{}"), Step::text("never")], |b| b.tool(tool));
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = cancel.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        trigger.store(true, Ordering::SeqCst);
    });
    let (result, events) = run_with_cancel(&h.agent, "go", &cancel);
    canceller.join().unwrap();
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert_eq!(events.last(), Some(&Event::Cancelled));
    let history = h.store.load("s1").unwrap();
    let last = history.last().unwrap();
    assert_eq!((last.role, last.tool_call_id.as_deref()), (Role::Tool, Some("c")));
    assert_eq!(h.provider.requests().len(), 1, "no further model call after cancel");
}

#[test]
fn oversized_output_keeps_head_and_tail_on_char_boundaries() {
    let ascii = FnTool::new(spec("big").max_output(1000), |_, _| Ok(format!("HEAD{}TAIL", "x".repeat(10_000))));
    let cjk = FnTool::new(spec("cjk").max_output(1001), |_, _| Ok("你好".repeat(1000)));
    let steps = vec![Step::tool_call("a", "big", "{}").and_tool_call("b", "cjk", "{}"), Step::text("ok")];
    let h = harness(steps, |b| b.tool(ascii).tool(cjk));
    assert!(run(&h.agent, "go").0.is_ok());
    let messages = tool_messages(&h);
    assert!(messages[0].1.starts_with("HEAD") && messages[0].1.ends_with("TAIL"));
    assert!(messages[0].1.contains("bytes truncated") && messages[0].1.len() < 1100);
    assert!(messages[1].1.contains("bytes truncated") && messages[1].1.len() < 1100);
}

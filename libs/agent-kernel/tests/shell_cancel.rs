#![cfg(feature = "shell")]
mod common;

use agent_kernel::builtin::Shell;
use agent_kernel::provider::scripted::Step;
use agent_kernel::{Role, SessionStore};
use common::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(windows)]
const LONG_COMMAND: &str = "ping -n 60 127.0.0.1 > nul";
#[cfg(unix)]
const LONG_COMMAND: &str = "sleep 60";

/// Stopping a run kills the command's process tree, which takes longer than the default cancel window
/// (taskkill alone can need a few hundred milliseconds). The model must be told the command was killed,
/// not that its outcome is unknown, and the run must not be paused for review.
#[test]
fn a_cancelled_shell_command_reports_that_it_was_killed_not_an_unknown_outcome() {
    let dir = TempDir::new("shell-cancel");
    let args = serde_json::json!({ "command": LONG_COMMAND }).to_string();
    let h = harness(vec![Step::tool_call("c", "shell", &args), Step::text("ok")], |b| b.tool(Shell::new(&dir.0)));
    let cancel = Arc::new(AtomicBool::new(false));
    let stopper = {
        let cancel = cancel.clone();
        std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(600)); cancel.store(true, Ordering::SeqCst); })
    };
    let started = Instant::now();
    let _ = run_with_cancel(&h.agent, "go", &cancel);
    stopper.join().unwrap();
    assert!(started.elapsed() < Duration::from_secs(10), "took {:?}", started.elapsed());
    // A cancelled run ends before the next model request, so read the stored tool result directly.
    let stored = h.store.load("s1").unwrap();
    let output = stored.iter().find(|m| m.role == Role::Tool).expect("the tool result was recorded").text();
    assert!(output.contains("was killed"), "{output}");
    assert!(!output.contains("outcome unknown"), "{output}");
}

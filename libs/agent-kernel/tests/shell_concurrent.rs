#![cfg(feature = "shell")]
mod common;

use agent_kernel::builtin::Shell;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// The lease table is process-wide, so these tests must not overlap.
static SERIAL: Mutex<()> = Mutex::new(());

fn options() -> RunOptions {
    RunOptions { control: Some(RunControl::bounded(Duration::from_secs(30), 100_000)), ..Default::default() }
}

/// Another conversation holding the shared mutation lease must not stop this one from running commands.
#[test]
fn a_shell_command_runs_while_another_conversation_holds_the_shared_lease() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let owner = RunControl::bounded(Duration::from_secs(30), 100_000);
    owner.claim().expect("the first conversation takes the shared lease");

    let dir = TempDir::new("shell-concurrent");
    let args = serde_json::json!({ "command": "echo from-the-second-conversation" }).to_string();
    let h = harness(vec![Step::tool_call("c", "shell", &args), Step::text("done")], |b| b.tool(Shell::new(&dir.0)));
    let result = h.agent.run_parts_observed("s2", vec![Part::Text { text: "go".into() }], options(), &mut |_| Ok(()), &Arc::new(AtomicBool::new(false)));
    assert!(result.is_ok(), "{result:?}");
    let output = tool_messages(&h).remove(0).1;
    assert!(output.contains("from-the-second-conversation"), "{output}");
    drop(owner);
}

/// Other state-changing tools keep the shared lease: only commands were released.
#[test]
fn other_mutating_tools_still_wait_for_the_shared_lease() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let owner = RunControl::bounded(Duration::from_secs(30), 100_000);
    owner.claim().unwrap();
    let h = harness(vec![Step::tool_call("w", "change_state", "{}"), Step::text("done")], |b| {
        b.tool(FnTool::new(ToolSpec::new("change_state", "x", serde_json::json!({"type": "object"})).risk(Risk::Write), |_, _| Ok("changed".into())))
    });
    let result = h.agent.run_parts_observed("s3", vec![Part::Text { text: "go".into() }], options(), &mut |_| Ok(()), &Arc::new(AtomicBool::new(false)));
    assert!(matches!(result, Err(HarnessError::Paused(ref r)) if r == "resources_busy"), "{result:?}");
    drop(owner);
}

#![cfg(all(feature = "process", unix))]
mod common;

use agent_kernel::{
    process::{ProcessConfig, ProcessManager, ProcessScope, ProcessState},
    provider::scripted::Step,
    *,
};
use common::*;
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

#[test]
fn a_process_accepts_pipe_input_reports_exit_status_and_reads_logs_by_cursor() {
    let dir = TempDir::new("process-input");
    let manager = ProcessManager::new(&dir.0).unwrap();
    let ctx = ToolCtx::detached("s");
    let started = manager
        .start(
            &ctx,
            "printf 'ready\\n'; read line; printf 'got:%s\\n' \"$line\"; exit 7",
            ProcessScope::Run,
            Duration::from_secs(5),
        )
        .unwrap();
    let first = manager
        .read(&ctx, &started.id, 0, 32000, Duration::from_millis(100))
        .unwrap();
    assert_eq!(first.output, "ready\n");
    assert_eq!(first.state, ProcessState::Running);
    manager.write(&ctx, &started.id, "hello\n", false).unwrap();
    let last = manager
        .read(
            &ctx,
            &started.id,
            first.next_cursor,
            32000,
            Duration::from_secs(1),
        )
        .unwrap();
    assert_eq!(last.output, "got:hello\n");
    assert_eq!(last.state, ProcessState::Exited);
    assert_eq!(last.exit_code, Some(7));
    let empty = manager
        .read(&ctx, &started.id, last.next_cursor, 32000, Duration::ZERO)
        .unwrap();
    assert_eq!(empty.output, "");
}

#[test]
fn handles_are_session_scoped_and_workspace_scoped_and_cannot_be_guessed() {
    let dir = TempDir::new("process-owner");
    let manager = ProcessManager::new(&dir.0).unwrap();
    let owner = ToolCtx::detached("owner");
    let process = manager
        .start(
            &owner,
            "sleep 30",
            ProcessScope::Session,
            Duration::from_secs(40),
        )
        .unwrap();
    let other = ToolCtx::detached("other");
    assert!(manager
        .read(&other, &process.id, 0, 100, Duration::ZERO)
        .is_err());
    assert!(manager.write(&other, &process.id, "x", true).is_err());
    assert!(manager.stop(&other, &process.id).is_err());
    let another = ProcessManager::new(&dir.0).unwrap();
    assert!(another.stop(&owner, &process.id).is_err());
    assert_eq!(
        manager.stop(&owner, &process.id).unwrap().state,
        ProcessState::Stopped
    );
}

#[test]
fn timeouts_kill_managed_descendants_and_do_not_leave_a_delayed_side_effect() {
    let dir = TempDir::new("process-timeout");
    let manager = ProcessManager::new(&dir.0).unwrap();
    let ctx = ToolCtx::detached("s");
    let process = manager
        .start(
            &ctx,
            "(sleep 0.5; touch should-not-exist) & wait",
            ProcessScope::Session,
            Duration::from_millis(100),
        )
        .unwrap();
    let result = manager
        .read(&ctx, &process.id, 0, 100, Duration::from_secs(1))
        .unwrap();
    assert_eq!(result.state, ProcessState::TimedOut);
    std::thread::sleep(Duration::from_millis(550));
    assert!(!dir.path("should-not-exist").exists());
}

#[test]
fn output_is_bounded_and_utf8_cursor_pages_do_not_lose_characters() {
    let dir = TempDir::new("process-output");
    let manager = ProcessManager::with_config(
        &dir.0,
        ProcessConfig {
            output_bytes: 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let ctx = ToolCtx::detached("s");
    let process = manager
        .start(
            &ctx,
            "printf '%05000d' 0",
            ProcessScope::Run,
            Duration::from_secs(2),
        )
        .unwrap();
    let result = manager
        .read(&ctx, &process.id, 0, 32000, Duration::from_secs(1))
        .unwrap();
    assert_eq!(result.next_cursor, 5000);
    assert_eq!(result.output.len(), 1024);
    assert_eq!(result.dropped_bytes, 3976);
    assert!(manager
        .read(&ctx, &process.id, 5001, 100, Duration::ZERO)
        .is_err());
    let process = manager
        .start(
            &ctx,
            "printf '你好世界'",
            ProcessScope::Run,
            Duration::from_secs(2),
        )
        .unwrap();
    let mut cursor = 0;
    let mut text = String::new();
    for _ in 0..4 {
        let page = manager
            .read(&ctx, &process.id, cursor, 4, Duration::from_secs(1))
            .unwrap();
        text.push_str(&page.output);
        assert!(page.next_cursor > cursor);
        cursor = page.next_cursor;
    }
    assert_eq!(text, "你好世界");
    assert_eq!(cursor, 12);
}

fn start_agent(manager: &Arc<ProcessManager>, keep_alive: bool) -> Harness {
    harness(
        vec![
            Step::tool_call(
                "c",
                "process_start",
                &json!({"command":"sleep 30","keep_alive":keep_alive}).to_string(),
            ),
            Step::text("started"),
        ],
        |mut builder| {
            for tool in manager.tools() {
                builder = builder.tool_arc(tool);
            }
            builder
        },
    )
}

fn process_id(h: &Harness) -> String {
    h.store
        .load("s1")
        .unwrap()
        .iter()
        .find(|m| m.role == Role::Tool)
        .unwrap()
        .structured_content
        .as_ref()
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}

#[test]
fn run_cleanup_stops_default_children_and_explicit_session_children_survive_until_closed() {
    let dir = TempDir::new("process-lifetime");
    let manager = Arc::new(ProcessManager::new(&dir.0).unwrap());
    let ctx = ToolCtx::detached("s1");
    let default = start_agent(&manager, false);
    run(&default.agent, "go").0.unwrap();
    assert_eq!(
        manager
            .read(&ctx, &process_id(&default), 0, 100, Duration::ZERO)
            .unwrap()
            .state,
        ProcessState::Stopped
    );
    let retained = start_agent(&manager, true);
    run(&retained.agent, "go").0.unwrap();
    let id = process_id(&retained);
    assert_eq!(
        manager
            .read(&ctx, &id, 0, 100, Duration::ZERO)
            .unwrap()
            .state,
        ProcessState::Running
    );
    manager.stop_session("s1").unwrap();
    assert_eq!(
        manager
            .read(&ctx, &id, 0, 100, Duration::ZERO)
            .unwrap()
            .state,
        ProcessState::Stopped
    );
}

#[test]
fn cancellation_also_stops_retained_processes_before_the_run_returns() {
    let dir = TempDir::new("process-cancel");
    let manager = Arc::new(ProcessManager::new(&dir.0).unwrap());
    let h = start_agent(&manager, true);
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = cancel.clone();
    let result = h.agent.run(
        "s1",
        "go",
        &mut |event| {
            if matches!(event, Event::ToolEnd { .. }) {
                trigger.store(true, Ordering::SeqCst);
            }
        },
        &cancel,
    );
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    let process = manager
        .read(
            &ToolCtx::detached("s1"),
            &process_id(&h),
            0,
            100,
            Duration::ZERO,
        )
        .unwrap();
    assert_ne!(process.state, ProcessState::Running);
}

#[test]
fn process_limits_and_manager_shutdown_prevent_further_starts() {
    let dir = TempDir::new("process-limit");
    let manager = ProcessManager::with_config(
        &dir.0,
        ProcessConfig {
            max_running: 1,
            max_retained: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let ctx = ToolCtx::detached("s");
    manager
        .start(
            &ctx,
            "sleep 30",
            ProcessScope::Session,
            Duration::from_secs(40),
        )
        .unwrap();
    assert!(manager
        .start(&ctx, "true", ProcessScope::Run, Duration::from_secs(1))
        .is_err());
    manager.shutdown().unwrap();
    assert!(manager
        .start(&ctx, "true", ProcessScope::Run, Duration::from_secs(1))
        .is_err());
}

#[test]
fn process_tools_use_execution_policy_and_can_be_rejected_before_start() {
    let dir = TempDir::new("process-policy");
    let manager = Arc::new(ProcessManager::new(&dir.0).unwrap());
    let h = harness(
        vec![
            Step::tool_call("c", "process_start", r#"{"command":"touch forbidden"}"#),
            Step::text("denied"),
        ],
        |mut builder| {
            for tool in manager.tools() {
                builder = builder.tool_arc(tool);
            }
            builder.middleware(Permission::new(
                PermissionMode::ReadOnly,
                |_: &ApprovalRequest<'_>| panic!("must not ask"),
            ))
        },
    );
    run(&h.agent, "go").0.unwrap();
    assert!(!dir.path("forbidden").exists());
    assert!(
        h.store
            .load("s1")
            .unwrap()
            .iter()
            .find(|m| m.role == Role::Tool)
            .unwrap()
            .is_error
    );
}

#[test]
fn delegated_session_processes_are_closed_even_when_keep_alive_was_requested() {
    use agent_kernel::{builtin::Subagent, provider::scripted::ScriptedProvider};
    let dir = TempDir::new("process-child");
    let manager = Arc::new(ProcessManager::new(&dir.0).unwrap());
    let store = Arc::new(MemoryStore::new());
    let provider = Arc::new(ScriptedProvider::new(vec![
        Step::tool_call(
            "c",
            "process_start",
            r#"{"command":"sleep 30","keep_alive":true}"#,
        ),
        Step::text("done"),
    ]));
    let mut child = Agent::builder(provider, store.clone());
    for tool in manager.tools() {
        child = child.tool_arc(tool);
    }
    let sub = Subagent::new(
        "task",
        "delegate",
        Arc::new(child.build().unwrap()),
        Risk::Exec,
    );
    let parent = harness(
        vec![
            Step::tool_call("c", "task", r#"{"task":"work"}"#),
            Step::text("done"),
        ],
        |b| b.tool(sub),
    );
    run(&parent.agent, "go").0.unwrap();
    let session = store.session_ids().remove(0);
    let history = store.load(&session).unwrap();
    let result = history.iter().find(|m| m.role == Role::Tool).unwrap();
    let id = result.structured_content.as_ref().unwrap()["id"]
        .as_str()
        .unwrap();
    assert!(manager
        .read(&ToolCtx::detached(&session), id, 0, 100, Duration::ZERO)
        .is_err());
}

#[test]
fn a_timed_out_pipe_write_stops_the_process_and_unblocks_the_writer() {
    let dir = TempDir::new("process-pipe-timeout");
    let manager = Arc::new(ProcessManager::new(&dir.0).unwrap());
    let process = manager
        .start(
            &ToolCtx::detached("s1"),
            "sleep 30",
            ProcessScope::Session,
            Duration::from_secs(40),
        )
        .unwrap();
    let writer = manager.clone();
    let id = process.id.clone();
    let h = harness(
        vec![Step::tool_call("c", "fill", "{}"), Step::text("done")],
        |b| {
            b.tool(FnTool::new(
                spec("fill").timeout(Duration::from_millis(100)),
                move |_, ctx| {
                    for _ in 0..1000 {
                        writer.write(ctx, &id, &"x".repeat(8192), false)?;
                    }
                    Ok("written".into())
                },
            ))
        },
    );
    run(&h.agent, "go").0.unwrap();
    let status = manager
        .read(
            &ToolCtx::detached("s1"),
            &process.id,
            0,
            100,
            Duration::from_secs(1),
        )
        .unwrap();
    assert_eq!(status.state, ProcessState::Cancelled);
}

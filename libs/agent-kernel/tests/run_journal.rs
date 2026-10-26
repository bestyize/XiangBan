#![cfg(feature = "journal")]
mod common;

use agent_kernel::{journal::JsonlRunLog, provider::scripted::Step, *};
use common::*;
use std::io::Write;
use std::sync::{atomic::AtomicBool, Arc};

fn record(sequence: u64, body: RunEventBody) -> RunEvent {
    RunEvent {
        run_id: "r-test".into(),
        session_id: "s".into(),
        parent_run_id: None,
        sequence,
        timestamp_ms: 0,
        body,
    }
}

#[test]
fn a_run_is_observable_after_reopening_and_can_be_paged_without_repeating_events() {
    let dir = TempDir::new("run-journal");
    let log = JsonlRunLog::new(&dir.0);
    let h = harness(
        vec![Step::tool_call("c", "echo", "{}"), Step::text("done")],
        |b| b.tool(FnTool::new(spec("echo"), |_, _| Ok("ok".into()))),
    );
    let summary = h
        .agent
        .run_parts_observed(
            "s",
            vec![Part::Text { text: "go".into() }],
            RunOptions::default(),
            &mut |event| log.append(&event),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    drop(log);
    let reopened = JsonlRunLog::new(&dir.0);
    let first = reopened.read_after(&summary.run_id, 0, 2).unwrap();
    let second = reopened
        .read_after(&summary.run_id, first.last().unwrap().sequence, 100)
        .unwrap();
    assert_eq!(
        second.first().unwrap().sequence,
        first.last().unwrap().sequence + 1
    );
    let status = reopened.inspect(&summary.run_id).unwrap();
    assert!(
        status.terminal
            && status.state == Some(RunState::Completed)
            && status.incomplete_tools.is_empty()
    );
    assert!(first.iter().chain(&second).any(|r| matches!(
        r.body,
        RunEventBody::Event {
            event: Event::ToolDispatched { .. }
        }
    )));
}

#[test]
fn a_dispatched_action_without_a_result_is_reported_without_replaying_it() {
    let dir = TempDir::new("run-interrupted");
    let log = JsonlRunLog::new(&dir.0);
    log.append(&record(
        1,
        RunEventBody::State {
            state: RunState::Running,
            reason: None,
        },
    ))
    .unwrap();
    log.append(&record(
        2,
        RunEventBody::Event {
            event: Event::ToolStart {
                id: "c".into(),
                name: "publish".into(),
                arguments: "{}".into(),
            },
        },
    ))
    .unwrap();
    log.append(&record(
        3,
        RunEventBody::Event {
            event: Event::ToolDispatched { id: "c".into() },
        },
    ))
    .unwrap();
    let path = dir.path("run-r-test.jsonl");
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(b"{\"partial\":")
        .unwrap();
    let reopened = JsonlRunLog::new(&dir.0);
    let status = reopened.inspect("r-test").unwrap();
    assert!(!status.terminal && status.last_sequence == 3);
    assert_eq!(status.incomplete_tools.len(), 1);
    assert!(status.incomplete_tools[0].dispatched);
    assert_eq!(status.incomplete_tools[0].name, "publish");
}

#[test]
fn competing_writers_invalid_paths_and_sequence_gaps_are_rejected() {
    let dir = TempDir::new("run-writer");
    let log = JsonlRunLog::new(&dir.0);
    let start = record(
        1,
        RunEventBody::State {
            state: RunState::Running,
            reason: None,
        },
    );
    log.append(&start).unwrap();
    assert!(JsonlRunLog::new(&dir.0).append(&start).is_err());
    let mut gap = start.clone();
    gap.sequence = 3;
    assert!(log.append(&gap).is_err());
    let mut unsafe_id = start;
    unsafe_id.run_id = "../outside".into();
    assert!(log.append(&unsafe_id).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.path("run-r-test.jsonl"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn terminal_state_allows_only_its_notification_or_a_failed_commit_correction() {
    let dir=TempDir::new("terminal-tail");let log=JsonlRunLog::new(&dir.0);
    log.append(&record(1,RunEventBody::State{state:RunState::Running,reason:None})).unwrap();
    log.append(&record(2,RunEventBody::State{state:RunState::Completed,reason:None})).unwrap();
    assert!(log.append(&record(3,RunEventBody::Event{event:Event::TextDelta{text:"late output".into()}})).is_err());
    log.append(&record(3,RunEventBody::State{state:RunState::Failed,reason:Some("goal commit failed".into())})).unwrap();
    log.append(&record(4,RunEventBody::Event{event:Event::Error{message:"goal commit failed".into()}})).unwrap();
    assert_eq!(log.inspect("r-test").unwrap().state,Some(RunState::Failed));
}

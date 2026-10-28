//! A model that keeps looking at the same unchanged thing is warned plainly, given one more round to explain itself,
//! and only then paused; identical reads of an unchanged target stop being run from the third time on.
#![cfg(feature = "fs")]
mod common;

use agent_kernel::builtin::fs::fs_tools;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use serde_json::json;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const LIST: &str = r#"{"path":"."}"#;

fn fs_agent(dir: &TempDir, steps: Vec<Step>) -> Harness {
    let roots = vec![dir.0.clone()];
    harness(steps, move |b| fs_tools(roots).unwrap().into_iter().fold(b.unlimited_turns(), |b, tool| b.tool_arc(tool)))
}

fn run_observed(h: &Harness, limit: usize) -> (Result<RunSummary, HarnessError>, Vec<Event>) {
    let mut events = Vec::new();
    let result = h.agent.run_parts_observed(
        "s1", vec![Part::Text { text: "go".into() }],
        RunOptions { max_no_progress_rounds: Some(limit), ..Default::default() },
        &mut |record| { if let RunEventBody::Event { event } = record.body { events.push(event); } Ok(()) },
        &Arc::new(AtomicBool::new(false)),
    );
    (result, events)
}

fn same_list_dir(rounds: usize) -> Vec<Step> {
    (0..rounds).map(|n| Step::tool_call(&format!("c{n}"), "list_dir", LIST)).collect()
}

#[test]
fn a_model_looping_on_one_directory_is_warned_then_given_one_round_then_paused() {
    let dir = TempDir::new("stall-pause");
    dir.write("a.txt", "x");
    let h = fs_agent(&dir, same_list_dir(12));
    let (result, events) = run_observed(&h, 3);

    assert_eq!(result, Err(HarnessError::Paused("no_progress_tool_rounds".into())));
    let warnings: Vec<_> = events.iter().enumerate().filter(|(_, e)| matches!(e, Event::Continuing { reason } if reason == "no_progress_warning")).collect();
    assert_eq!(warnings.len(), 1, "{events:?}");
    let error_at = events.iter().position(|e| matches!(e, Event::Error { message } if message.contains("no_progress_tool_rounds"))).expect("the pause is reported");
    assert!(warnings[0].0 < error_at, "the warning comes before the pause");
    // Warned after round 4 (progress, then three idle rounds); the very next round was the last one.
    assert_eq!(h.provider.requests().len(), 5);
    let last = h.provider.requests().pop().unwrap();
    let reminder = last.messages.iter().find(|m| m.synthetic).expect("the warning reached the model");
    assert!(reminder.text().contains("Stop calling tools now") && reminder.text().contains("what you are waiting for"), "{}", reminder.text());
}

#[test]
fn explaining_after_the_warning_ends_the_run_normally() {
    let dir = TempDir::new("stall-explain");
    dir.write("a.txt", "x");
    let mut steps = same_list_dir(4);
    steps.push(Step::text("我在等你确认目录里是不是还缺文件。"));
    let h = fs_agent(&dir, steps);
    let (result, events) = run_observed(&h, 3);
    assert_eq!(result.unwrap().text, "我在等你确认目录里是不是还缺文件。");
    assert!(events.iter().any(|e| matches!(e, Event::Continuing { reason } if reason == "no_progress_warning")));
    assert!(!events.iter().any(|e| matches!(e, Event::Error { .. })));
}

#[test]
fn the_third_identical_read_of_an_unchanged_directory_is_not_run() {
    let dir = TempDir::new("stall-skip");
    dir.write("a.txt", "x");
    let h = fs_agent(&dir, { let mut s = same_list_dir(3); s.push(Step::text("done")); s });
    let (result, _) = run_observed(&h, 50);
    assert!(result.is_ok());
    let tools = tool_messages(&h);
    assert!(tools[0].1.contains("a.txt") && tools[1].1.contains("a.txt"), "{tools:?}");
    assert!(tools[2].1.starts_with("Not run:") && tools[2].1.contains("has not changed since"), "{tools:?}");
}

#[test]
fn a_write_or_a_command_in_between_resets_the_count_so_write_then_look_still_works() {
    let dir = TempDir::new("stall-reset");
    dir.write("a.txt", "x");
    let root = dir.0.clone();
    let steps = vec![
        Step::tool_call("c0", "list_dir", LIST),
        Step::tool_call("c1", "list_dir", LIST),
        Step::tool_call("c2", "touch", r#"{}"#),
        Step::tool_call("c3", "list_dir", LIST),
        Step::tool_call("c4", "list_dir", LIST),
        Step::text("done"),
    ];
    let tools = fs_tools(vec![dir.0.clone()]).unwrap();
    let h = harness(steps, move |b| {
        let b = tools.into_iter().fold(b.unlimited_turns(), |b, tool| b.tool_arc(tool));
        b.tool(FnTool::new(ToolSpec::new("touch", "writes", json!({"type":"object"})).risk(Risk::Write).serial(),
            move |_, _| { std::fs::write(root.join("b.txt"), "y").map(|_| "ok".to_string()).map_err(|e| e.to_string()) }))
    });
    let (result, _) = run_observed(&h, 50);
    assert!(result.is_ok());
    let outputs: Vec<String> = h.provider.requests().pop().unwrap().messages.iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect();
    assert_eq!(outputs.len(), 5);
    assert!(outputs.iter().all(|o| !o.starts_with("Not run:")), "nothing was refused: {outputs:?}");
    assert!(outputs[3].contains("b.txt"), "the new file shows up after the write");
}

#[test]
fn a_changed_file_is_read_again_and_an_unchanged_one_is_not() {
    let dir = TempDir::new("stall-file");
    dir.write("f.txt", "one\ntwo\n");
    let path = dir.path("f.txt");
    let args = r#"{"path":"f.txt"}"#;
    let steps = vec![
        Step::tool_call("c0", "read_file", args), Step::tool_call("c1", "read_file", args), Step::tool_call("c2", "read_file", args),
        Step::text("done"),
    ];
    let h = fs_agent(&dir, steps);
    let (result, _) = run_observed(&h, 50);
    assert!(result.is_ok());
    assert!(tool_messages(&h)[2].1.starts_with("Not run:"));
    // The same three calls with the file changing before the third: all of them run.
    let h = fs_agent(&dir, vec![
        Step::tool_call("d0", "read_file", args), Step::tool_call("d1", "read_file", args),
        Step::tool_call("d2", "read_file", args), Step::text("done"),
    ]);
    let mut seen = 0;
    let result = h.agent.run_parts_observed("s2", vec![Part::Text { text: "go".into() }], RunOptions::default(), &mut |record| {
        if matches!(record.body, RunEventBody::Event { event: Event::ToolEnd { .. } }) { seen += 1; if seen == 2 { std::fs::write(&path, "one\ntwo\nthree\n").unwrap(); } }
        Ok(())
    }, &Arc::new(AtomicBool::new(false)));
    assert!(result.is_ok());
    assert!(tool_messages(&h)[2].1.contains("three"), "{:?}", tool_messages(&h));
}

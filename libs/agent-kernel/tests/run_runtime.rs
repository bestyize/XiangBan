mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

#[test]
fn observed_runs_have_unique_ids_monotonic_events_and_explicit_terminal_states() {
    let h = harness(vec![Step::text("one"), Step::text("two")], |b| b);
    let mut records = Vec::new();
    let options = RunOptions {
        parent_run_id: Some("parent-run".into()),
        ..Default::default()
    };
    let first = h
        .agent
        .run_parts_observed(
            "s1",
            vec![Part::Text { text: "hi".into() }],
            options,
            &mut |event| {
                records.push(event);
                Ok(())
            },
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(matches!(
        records.first().unwrap().body,
        RunEventBody::State {
            state: RunState::Running,
            ..
        }
    ));
    assert!(matches!(
        records.last().unwrap().body,
        RunEventBody::State {
            state: RunState::Completed,
            ..
        }
    ));
    for (index, event) in records.iter().enumerate() {
        assert_eq!(event.sequence, index as u64 + 1);
        assert_eq!(event.run_id, first.run_id);
        assert_eq!(event.session_id, "s1");
        assert_eq!(event.parent_run_id.as_deref(), Some("parent-run"));
    }
    let second = run(&h.agent, "again").0.unwrap();
    assert_ne!(first.run_id, second.run_id);
}

#[test]
fn a_journal_failure_at_dispatch_prevents_the_side_effect_and_runs_cleanup() {
    struct Managed {
        spec: ToolSpec,
        writes: Arc<AtomicUsize>,
        cleaned: Arc<AtomicUsize>,
    }
    impl Tool for Managed {
        fn spec(&self) -> &ToolSpec {
            &self.spec
        }
        fn execute(&self, _: serde_json::Value, ctx: &ToolCtx) -> Result<String, String> {
            assert!(ctx.run_id().starts_with("r-"));
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok("written".into())
        }
        fn on_run_end(&self, end: &RunEnd) -> Result<(), String> {
            assert_eq!(end.state, RunState::Failed);
            self.cleaned.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    let writes = Arc::new(AtomicUsize::new(0));
    let cleaned = Arc::new(AtomicUsize::new(0));
    let h = harness(vec![Step::tool_call("c", "managed", "{}")], |b| {
        b.tool(Managed {
            spec: spec("managed"),
            writes: writes.clone(),
            cleaned: cleaned.clone(),
        })
    });
    let result = h.agent.run_parts_observed(
        "s1",
        vec![Part::Text { text: "go".into() }],
        RunOptions::default(),
        &mut |record| {
            if matches!(
                record.body,
                RunEventBody::Event {
                    event: Event::ToolDispatched { .. }
                }
            ) {
                return Err(HarnessError::Store("disk full".into()));
            }
            Ok(())
        },
        &Arc::new(AtomicBool::new(false)),
    );
    assert_eq!(result.unwrap_err(), HarnessError::Store("disk full".into()));
    assert_eq!(writes.load(Ordering::SeqCst), 0);
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    let history = h.store.load("s1").unwrap();
    assert!(history.last().unwrap().is_error);
}

#[test]
fn a_tool_batch_over_budget_is_not_partially_executed_and_pauses() {
    let writes = Arc::new(AtomicUsize::new(0));
    let count = writes.clone();
    let h = harness(
        vec![Step::tool_call("a", "write", "{}").and_tool_call("b", "write", "{}")],
        |b| {
            b.tool(FnTool::new(spec("write"), move |_, _| {
                count.fetch_add(1, Ordering::SeqCst);
                Ok("ok".into())
            }))
        },
    );
    let mut last = None;
    let result = h.agent.run_parts_observed(
        "s1",
        vec![Part::Text { text: "go".into() }],
        RunOptions {
            max_tool_calls: Some(1),
            ..Default::default()
        },
        &mut |record| {
            last = Some(record.body);
            Ok(())
        },
        &Arc::new(AtomicBool::new(false)),
    );
    assert!(matches!(result, Err(HarnessError::BudgetExceeded(_))));
    assert_eq!(writes.load(Ordering::SeqCst), 0);
    assert!(matches!(
        last,
        Some(RunEventBody::State {
            state: RunState::Paused,
            ..
        })
    ));
    assert_eq!(
        h.store
            .load("s1")
            .unwrap()
            .iter()
            .filter(|m| m.role == Role::Tool && m.is_error)
            .count(),
        2
    );
}

#[test]
fn rejected_calls_never_emit_dispatch_and_legacy_events_remain_compatible() {
    let h = harness(
        vec![
            Step::tool_call("c", "checked", r#"{"value":1}"#),
            Step::text("done"),
        ],
        |b| {
            b.tool(FnTool::new(
                ToolSpec::new(
                    "checked",
                    "d",
                    json!({"type":"object","properties":{"value":{"type":"string"}}}),
                ),
                |_, _| panic!("invalid"),
            ))
        },
    );
    let mut records = Vec::new();
    h.agent
        .run_parts_observed(
            "s1",
            vec![Part::Text { text: "go".into() }],
            RunOptions::default(),
            &mut |record| {
                records.push(record);
                Ok(())
            },
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(!records.iter().any(|r| matches!(
        r.body,
        RunEventBody::Event {
            event: Event::ToolDispatched { .. }
        }
    )));
    let h = harness(
        vec![Step::tool_call("c", "ok", "{}"), Step::text("done")],
        |b| b.tool(FnTool::new(spec("ok"), |_, _| Ok("ok".into()))),
    );
    let (_, events) = run(&h.agent, "go");
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::ToolDispatched { .. })));
    assert_eq!(events.last(), Some(&Event::Done));
}

#[test]
fn ignored_polling_tools_remain_repeatable_but_still_consume_budget() {
    let steps = (0..8)
        .map(|i| Step::tool_call(&format!("c{i}"), "poll", "{}"))
        .chain([Step::text("done")])
        .collect();
    let h = harness(steps, |b| {
        b.tool(FnTool::new(spec("poll"), |_, _| Ok("running".into())))
            .middleware(RepeatGuard::new().ignore_tool("poll"))
    });
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(h
        .store
        .load("s1")
        .unwrap()
        .iter()
        .filter(|m| m.role == Role::Tool)
        .all(|m| !m.is_error && m.text() == "running"));
}

#[test]
fn delegated_run_records_keep_parent_identity_and_use_temporary_session_resources() {
    use agent_kernel::{builtin::Subagent, provider::scripted::ScriptedProvider};
    use std::sync::Mutex;
    let child = Arc::new(
        Agent::builder(
            Arc::new(ScriptedProvider::new(vec![Step::text("child done")])),
            Arc::new(MemoryStore::new()),
        )
        .build()
        .unwrap(),
    );
    let records = Arc::new(Mutex::new(Vec::new()));
    let seen = records.clone();
    let sub = Subagent::new("task", "delegate", child, Risk::Read).observe_run(move |event| {
        seen.lock().unwrap().push(event.clone());
        Ok(())
    });
    let h = harness(
        vec![
            Step::tool_call("c", "task", r#"{"task":"work"}"#),
            Step::text("done"),
        ],
        |b| b.tool(sub),
    );
    let summary = run(&h.agent, "go").0.unwrap();
    let records = records.lock().unwrap();
    assert!(!records.is_empty());
    assert!(records.iter().all(
        |record| record.parent_run_id.as_deref() == Some(&summary.run_id)
            && record.run_id != summary.run_id
    ));
}

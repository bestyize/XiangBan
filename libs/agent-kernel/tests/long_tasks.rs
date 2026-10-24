use agent_kernel::{
    builtin::Todo,
    provider::scripted::{ScriptedProvider, Step},
    *,
};
use serde_json::json;
use std::sync::{atomic::AtomicBool, Arc};

struct Work(ToolSpec);
impl Work {
    fn new() -> Self {
        Self(ToolSpec::new("work", "A deterministic work step", json!({"type":"object","properties":{"step":{"type":"integer"}},"required":["step"]})).risk(Risk::Read))
    }
}
impl Tool for Work {
    fn spec(&self) -> &ToolSpec {
        &self.0
    }
    fn execute(&self, args: serde_json::Value, _: &ToolCtx) -> Result<String, String> {
        Ok(format!("finished {}", args["step"]))
    }
}

#[test]
fn productive_work_can_cross_the_old_sixty_turn_boundary_and_finite_limits_still_work() {
    let steps = || {
        (0..300)
            .map(|i| Step::tool_call(&format!("s{i}"), "work", &json!({"step":i}).to_string()))
            .chain([Step::text("all 300 steps finished")])
            .collect::<Vec<_>>()
    };
    let agent = Agent::builder(
        Arc::new(ScriptedProvider::new(steps())),
        Arc::new(MemoryStore::new()),
    )
    .tool(Work::new())
    .middleware(ContinueGuard::new())
    .unlimited_turns()
    .build()
    .unwrap();
    let summary = agent
        .run_parts_observed(
            "long",
            vec![Part::Text {
                text: "complete all steps".into(),
            }],
            RunOptions {
                max_no_progress_rounds: Some(6),
                ..Default::default()
            },
            &mut |_| Ok(()),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(summary.turns, 301);
    assert_eq!(summary.text, "all 300 steps finished");
    let bounded = Agent::builder(
        Arc::new(ScriptedProvider::new(steps())),
        Arc::new(MemoryStore::new()),
    )
    .tool(Work::new())
    .max_turns(2)
    .build()
    .unwrap();
    assert!(matches!(
        bounded.run(
            "limited",
            "work",
            &mut |_| {},
            &Arc::new(AtomicBool::new(false))
        ),
        Err(HarnessError::MaxTurns(2))
    ));
}

#[test]
fn progress_and_evidence_survive_a_new_agent_and_a_different_chat_history() {
    let root = std::env::temp_dir().join(format!("agent-progress-{}", std::process::id()));
    let checkpoint = json!({"todos":[{"content":"Build report","status":"in_progress","evidence":"sources.json checked"}],"handoff":"Next: validate report; never send it"});
    {
        let todo = Arc::new(Todo::new().persistent(&root));
        let agent = Agent::builder(
            Arc::new(ScriptedProvider::new(vec![
                Step::tool_call("plan", "todo", &checkpoint.to_string()),
                Step::text("paused"),
            ])),
            Arc::new(MemoryStore::new()),
        )
        .tool_arc(todo.clone())
        .middleware(todo)
        .build()
        .unwrap();
        agent
            .run(
                "session",
                "start",
                &mut |_| {},
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
    }
    let todo = Arc::new(Todo::new().persistent(&root));
    let provider = Arc::new(ScriptedProvider::new(vec![Step::text(
        "resumed from durable progress",
    )]));
    let agent = Agent::builder(provider.clone(), Arc::new(MemoryStore::new()))
        .tool_arc(todo.clone())
        .middleware(todo.clone())
        .build()
        .unwrap();
    agent
        .run(
            "session",
            "continue",
            &mut |_| {},
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let request = &provider.requests()[0];
    assert!(request
        .messages
        .iter()
        .any(|m| m.text().contains("sources.json checked") && m.text().contains("never send it")));
    assert!(todo.snapshot("other-session").is_empty());
    todo.remove_session("session").unwrap();
    assert!(Todo::new().persistent(&root).snapshot("session").is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_unwritable_progress_store_never_acknowledges_a_saved_plan() {
    let root = std::env::temp_dir().join(format!("agent-progress-file-{}", std::process::id()));
    std::fs::write(&root, b"not a directory").unwrap();
    let store = Arc::new(MemoryStore::new());
    let agent = Agent::builder(
        Arc::new(ScriptedProvider::new(vec![
            Step::tool_call(
                "p",
                "todo",
                r#"{"todos":[{"content":"work","status":"pending"}]}"#,
            ),
            Step::text("could not save"),
        ])),
        store.clone(),
    )
    .tool(Todo::new().persistent(&root))
    .build()
    .unwrap();
    agent
        .run("s", "work", &mut |_| {}, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert!(store
        .load("s")
        .unwrap()
        .iter()
        .any(|m| m.role == Role::Tool && m.is_error));
    std::fs::remove_file(root).unwrap();
}

#[test]
fn quiet_long_running_processes_can_wait_but_busy_polling_still_pauses() {
    struct Waiting(ToolSpec);
    impl Tool for Waiting {
        fn spec(&self) -> &ToolSpec {
            &self.0
        }
        fn execute_result(&self, _: serde_json::Value, _: &ToolCtx) -> ToolResult {
            ToolResult::ok("")
                .with_structured_content(json!({"state":"running","id":"owned-process"}))
        }
    }
    for (wait, succeeds) in [(60_000, true), (0, false)] {
        let steps = (0..12)
            .map(|i| {
                Step::tool_call(
                    &format!("wait{i}"),
                    "process_read",
                    &json!({"wait_ms":wait}).to_string(),
                )
            })
            .chain([Step::text("finished waiting")])
            .collect();
        let agent = Agent::builder(
            Arc::new(ScriptedProvider::new(steps)),
            Arc::new(MemoryStore::new()),
        )
        .tool(Waiting(
            ToolSpec::new(
                "process_read",
                "Owned process wait",
                json!({"type":"object"}),
            )
            .risk(Risk::Read),
        ))
        .unlimited_turns()
        .build()
        .unwrap();
        let result = agent.run_parts_observed(
            "wait",
            vec![Part::Text {
                text: "wait for build".into(),
            }],
            RunOptions {
                max_no_progress_rounds: Some(3),
                ..Default::default()
            },
            &mut |_| Ok(()),
            &Arc::new(AtomicBool::new(false)),
        );
        if succeeds {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(HarnessError::Paused(_))));
        }
    }
}

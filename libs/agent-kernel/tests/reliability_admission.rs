mod common;
use agent_kernel::*;
use common::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
struct Malformed(FinishReason, bool);
impl ModelProvider for Malformed {
    fn chat(
        &self,
        _: &ChatRequest,
        _: &mut dyn FnMut(&str),
        _: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        let call = ToolCall {
            id: "same".into(),
            name: "act".into(),
            arguments: "{}".into(),
        };
        Ok(Reply {
            message: Message::assistant(
                "",
                if self.1 {
                    vec![call.clone(), call]
                } else {
                    vec![call]
                },
            ),
            usage: None,
            finish: self.0,
        })
    }
}
#[test]
fn filtered_unknown_or_duplicate_batches_never_execute() {
    for (finish, duplicate) in [
        (FinishReason::ContentFilter, false),
        (FinishReason::Other, false),
        (FinishReason::ToolCalls, true),
    ] {
        let count = Arc::new(AtomicUsize::new(0));
        let n = count.clone();
        let a = Agent::builder(
            Arc::new(Malformed(finish, duplicate)),
            Arc::new(MemoryStore::new()),
        )
        .tool(FnTool::new(spec("act"), move |_, _| {
            n.fetch_add(1, Ordering::SeqCst);
            Ok("changed".into())
        }))
        .build()
        .unwrap();
        assert!(matches!(
            a.run("s", "act", &mut |_| {}, &Arc::new(AtomicBool::new(false))),
            Err(HarnessError::Paused(_))
        ));
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }
}
#[test]
#[cfg(all(unix, feature = "shell"))]
fn shell_reaps_redirected_descendants_on_success() {
    use agent_kernel::builtin::Shell;
    let dir = std::env::temp_dir().join(format!("agent-child-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let shell = Shell::new(&dir);
    shell
        .execute(
            serde_json::json!({"command":"(sleep 0.3; echo leaked > marker) >/dev/null 2>&1 &"}),
            &ToolCtx::detached("test"),
        )
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(!dir.join("marker").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn rejected_duplicate_ids_do_not_poison_the_next_authorized_turn() {
    use agent_kernel::provider::scripted::{ScriptedProvider, Step};
    let provider = Arc::new(ScriptedProvider::new(vec![
        Step::tool_call("same", "act", "{}").and_tool_call("same", "act", "{}"),
        Step::text("checked"),
    ]));
    let store = Arc::new(MemoryStore::new());
    let a = Agent::builder(provider.clone(), store.clone())
        .tool(FnTool::new(spec("act"), |_, _| {
            panic!("rejected calls must not run")
        }))
        .build()
        .unwrap();
    let c = Arc::new(AtomicBool::new(false));
    assert!(matches!(
        a.run("s", "act", &mut |_| {}, &c),
        Err(HarnessError::Paused(_))
    ));
    assert_eq!(
        a.run("s", "check before resuming", &mut |_| {}, &c)
            .unwrap()
            .text,
        "checked"
    );
    let history = store.load("s").unwrap();
    let m = history.iter().find(|m| m.tool_calls.len() == 2).unwrap();
    assert_ne!(m.tool_calls[0].id, m.tool_calls[1].id);
    assert!(m
        .structured_content
        .as_ref()
        .unwrap()
        .get("rejected_model_calls")
        .is_some());
    for call in &m.tool_calls {
        assert!(history
            .iter()
            .any(|m| m.tool_call_id.as_deref() == Some(call.id.as_str()) && m.is_error));
    }
}

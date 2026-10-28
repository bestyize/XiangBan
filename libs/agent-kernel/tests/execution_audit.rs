use agent_kernel::provider::scripted::{ScriptedProvider, Step};
use agent_kernel::{Agent, Event, HarnessError, MemoryStore, Part, Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use serde_json::{json, Value};
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};

struct ObservedRead {
    spec: ToolSpec,
    calls: Arc<AtomicUsize>,
    fail: bool,
}
impl Tool for ObservedRead {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn observe(&self, _: &Value) -> Option<String> { Some("unchanged".into()) }
    fn execute_result(&self, _: Value, _: &ToolCtx) -> ToolResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail { ToolResult::error("cannot read this file") } else { ToolResult::ok("file contents") }
    }
}

#[test]
fn failed_reads_are_never_cached_as_successful_observations() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = Arc::new(ScriptedProvider::new(vec![
        Step::tool_call("a", "read", "{}"), Step::tool_call("b", "read", "{}"),
        Step::tool_call("c", "read", "{}"), Step::text("blocked"),
    ]));
    let agent = Agent::builder(provider, Arc::new(MemoryStore::new())).tool(ObservedRead {
        spec: ToolSpec::new("read", "read", json!({"type":"object"})).risk(Risk::Read), calls: calls.clone(), fail: true,
    }).build().unwrap();
    let mut errors = vec![];
    agent.run("s", "read", &mut |event| if let Event::ToolEnd { is_error, .. } = event { errors.push(is_error); },
        &Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(errors, [true, true, true]);
}

#[test]
fn successful_reads_are_still_deduplicated_after_two_actual_results() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = Arc::new(ScriptedProvider::new(vec![
        Step::tool_call("a", "read", ""), Step::tool_call("b", "read", "{}"),
        Step::tool_call("c", "read", "{}"), Step::text("done"),
    ]));
    let agent = Agent::builder(provider, Arc::new(MemoryStore::new())).tool(ObservedRead {
        spec: ToolSpec::new("read", "read", json!({"type":"object"})).risk(Risk::Read), calls: calls.clone(), fail: false,
    }).build().unwrap();
    let mut skipped = vec![];
    agent.run("s", "read", &mut |event| if let Event::ToolEnd { id, structured_content: Some(value), .. } = event {
        if value["harness_status"] == "unchanged_repeat" { skipped.push(id); }
    }, &Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(skipped, ["c"]);
}

#[test]
fn pending_parallel_reads_are_not_mistaken_for_successful_results() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = Arc::new(ScriptedProvider::new(vec![
        Step::tool_call("a", "read", "{}").and_tool_call("b", "read", "{}").and_tool_call("c", "read", "{}"),
        Step::text("done"),
    ]));
    let agent = Agent::builder(provider, Arc::new(MemoryStore::new())).tool(ObservedRead {
        spec: ToolSpec::new("read", "read", json!({"type":"object"})).risk(Risk::Read).parallel(), calls: calls.clone(), fail: false,
    }).build().unwrap();
    agent.run("s", "read", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

struct BarrierTool { spec: ToolSpec, status: &'static str, media: bool }
impl Tool for BarrierTool {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute_result(&self, _: Value, _: &ToolCtx) -> ToolResult {
        let mut result = ToolResult::error("inspect state before retrying").with_structured_content(json!({
            "harness_status":self.status, "details":if self.media { String::new() } else { "x".repeat(40_000) }
        }));
        if self.media { result = result.with_part(Part::Image { url: "x".repeat(6 * 1024 * 1024 + 1) }); }
        result
    }
}
struct Mutation { spec: ToolSpec, calls: Arc<AtomicUsize> }
impl Tool for Mutation {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute_result(&self, _: Value, _: &ToolCtx) -> ToolResult {
        self.calls.fetch_add(1, Ordering::SeqCst); ToolResult::ok("changed")
    }
}

#[test]
fn oversized_results_keep_execution_barriers_and_reject_later_mutations() {
    for status in ["outcome_unknown", "resource_busy"] {
        for media in [false, true] {
            let calls = Arc::new(AtomicUsize::new(0));
            let provider = Arc::new(ScriptedProvider::new(vec![
                Step::tool_call("a", "barrier", "{}").and_tool_call("b", "mutate", "{}"), Step::text("must not finish"),
            ]));
            let agent = Agent::builder(provider.clone(), Arc::new(MemoryStore::new()))
                .tool(BarrierTool { spec: ToolSpec::new("barrier", "barrier", json!({"type":"object"})), status, media })
                .tool(Mutation { spec: ToolSpec::new("mutate", "mutate", json!({"type":"object"})), calls: calls.clone() })
                .build().unwrap();
            let mut barrier = None;
            let result = agent.run("s", "act", &mut |event| if let Event::ToolEnd { id, structured_content, parts, .. } = event {
                if id == "a" { assert!(parts.is_empty()); barrier = structured_content; }
            }, &Arc::new(AtomicBool::new(false)));
            assert!(matches!(result, Err(HarnessError::Paused(_))), "{status}, media={media}");
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert_eq!(provider.requests().len(), 1);
            assert_eq!(barrier.unwrap(), json!({"harness_status":status}));
        }
    }
}

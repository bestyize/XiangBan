#![allow(dead_code)]
pub mod http;
use agent_kernel::provider::scripted::{ScriptedProvider, Step};
use agent_kernel::*;
use serde_json::{json, Value};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

type ToolFn = Box<dyn Fn(Value, &ToolCtx) -> Result<String, String> + Send + Sync>;

pub struct FnTool {
    spec: ToolSpec,
    run: ToolFn,
}

impl FnTool {
    pub fn new(
        spec: ToolSpec,
        run: impl Fn(Value, &ToolCtx) -> Result<String, String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            spec,
            run: Box::new(run),
        }
    }
}

impl Tool for FnTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        (self.run)(args, ctx)
    }
}

pub fn spec(name: &str) -> ToolSpec {
    ToolSpec::new(name, "test tool", json!({ "type": "object" }))
        .risk(Risk::Read)
        .parallel()
}

pub struct Harness {
    pub agent: Agent,
    pub provider: Arc<ScriptedProvider>,
    pub store: Arc<MemoryStore>,
}

pub fn fast_retry() -> RetryPolicy {
    RetryPolicy {
        max_retries: 3,
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(5),
    }
}

pub fn harness(steps: Vec<Step>, configure: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Harness {
    let provider = Arc::new(ScriptedProvider::new(steps));
    let store = Arc::new(MemoryStore::new());
    let builder = Agent::builder(provider.clone(), store.clone()).retry(fast_retry());
    let agent = configure(builder).build().expect("agent builds");
    Harness {
        agent,
        provider,
        store,
    }
}

pub fn run(agent: &Agent, input: &str) -> (Result<RunSummary, HarnessError>, Vec<Event>) {
    run_with_cancel(agent, input, &Arc::new(AtomicBool::new(false)))
}

pub fn run_with_cancel(
    agent: &Agent,
    input: &str,
    cancel: &Arc<AtomicBool>,
) -> (Result<RunSummary, HarnessError>, Vec<Event>) {
    let mut events = Vec::new();
    let result = agent.run("s1", input, &mut |e| events.push(e), cancel);
    (result, events)
}

/// The tool messages of the last request the model received, as (call id, text).
pub fn tool_messages(h: &Harness) -> Vec<(String, String)> {
    let requests = h.provider.requests();
    let last = requests.last().expect("at least one request");
    last.messages
        .iter()
        .filter(|m| m.role == Role::Tool)
        .map(|m| (m.tool_call_id.clone().unwrap_or_default(), m.text()))
        .collect()
}

pub struct TempDir(pub std::path::PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "agent-kernel-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub fn path(&self, rel: &str) -> std::path::PathBuf {
        self.0.join(rel)
    }

    pub fn write(&self, rel: &str, text: &str) {
        let path = self.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

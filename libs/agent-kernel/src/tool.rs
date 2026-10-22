use serde_json::Value;
use crate::types::{Message, Part};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    Read,
    Write,
    Network,
    Exec,
}

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// JSON Schema for the arguments object.
    pub schema: Value,
    pub risk: Risk,
    pub parallel_safe: bool,
    pub timeout: Duration,
    /// Output beyond this many bytes is cut (head and tail kept) before the model sees it.
    pub max_output: usize,
    /// Output comes from outside (web pages, search results). `Permission` then stops
    /// auto-approving state-changing tools for the rest of that session.
    pub untrusted_output: bool,
    /// Always ask the user, in every mode, and never remember an approval.
    pub always_ask: bool,
    /// Changes only this app's own state (its windows, appearance, settings or saved content), never the
    /// user's files, other applications or the system. `Ask` mode runs it without asking and a session that
    /// read external content is not asked again for it; `ReadOnly` still blocks it, and a `danger_check`
    /// reason still asks.
    pub self_managed: bool,
}

impl ToolSpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>, schema: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schema,
            risk: Risk::Exec,
            parallel_safe: false,
            timeout: Duration::from_secs(30),
            max_output: DEFAULT_MAX_OUTPUT,
            untrusted_output: false,
            always_ask: false,
            self_managed: false,
        }
    }

    pub fn untrusted_output(mut self) -> Self {
        self.untrusted_output = true;
        self
    }

    pub fn always_ask(mut self) -> Self {
        self.always_ask = true;
        self
    }

    pub fn self_managed(mut self) -> Self {
        self.self_managed = true;
        self
    }

    pub fn risk(mut self, risk: Risk) -> Self {
        self.risk = risk;
        self
    }

    pub fn serial(mut self) -> Self {
        self.parallel_safe = false;
        self
    }

    /// Opt in only when overlapping calls cannot race on shared state.
    pub fn parallel(mut self) -> Self {
        self.parallel_safe = true;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_output(mut self, bytes: usize) -> Self {
        self.max_output = bytes;
        self
    }
}

pub const DEFAULT_MAX_OUTPUT: usize = 32_000;

/// Handed to a running tool. Long-running tools should poll `is_cancelled`.
pub struct ToolCtx {
    pub(crate) session: String,
    pub(crate) run_id: String,
    pub(crate) run_cancel: Arc<AtomicBool>,
    pub(crate) call_cancel: Arc<AtomicBool>,
    pub(crate) budget: Option<Arc<crate::ToolBudget>>,
}

impl ToolCtx {
    pub fn session(&self) -> &str {
        &self.session
    }

    pub fn run_id(&self) -> &str { &self.run_id }

    /// The run's tool-call budget, when it has one; a tool that starts a run of its own passes it on.
    pub fn tool_budget(&self) -> Option<&Arc<crate::ToolBudget>> { self.budget.as_ref() }

    /// A context that is never cancelled, for calling a tool directly, for example in a host's tests.
    pub fn detached(session: &str) -> Self {
        Self { session: session.to_string(), run_id: format!("detached-{session}"), run_cancel: Arc::new(AtomicBool::new(false)), call_cancel: Arc::new(AtomicBool::new(false)), budget: None }
    }

    /// True after the user cancels the run or this call hits its timeout.
    pub fn is_cancelled(&self) -> bool {
        self.run_cancel.load(Ordering::Relaxed) || self.call_cancel.load(Ordering::Relaxed)
    }

    /// Waits for a value from `rx` (for example the user's answer from a dialog) and gives up with
    /// `None` when the run is cancelled, the call times out, or the sender is dropped. Code that
    /// blocks on a person should wait this way, or the run cannot be cancelled while it waits.
    pub fn wait_for<T>(&self, rx: &Receiver<T>) -> Option<T> {
        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(value) => return Some(value),
                Err(RecvTimeoutError::Timeout) if self.is_cancelled() => return None,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return None,
            }
        }
    }
}

pub trait Tool: Send + Sync {
    fn spec(&self) -> &ToolSpec;
    /// `Ok` and `Err` both go back to the model; `Err` is flagged as an error result.
    fn execute(&self, _args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        Err("this tool requires the structured execute_result entry point".into())
    }

    /// The harness calls this entry point. Text-only tools need no changes; tools returning
    /// images or structured data override it and keep `execute` as a legacy text projection.
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        match self.execute(args, ctx) {
            Ok(output) => ToolResult::ok(output),
            Err(error) => ToolResult::error(error),
        }
    }

    /// Release run-owned resources before the terminal event. Runs once per registered tool;
    /// implementations sharing a resource manager should designate one tool for cleanup.
    /// Must be idempotent: finalization failure may trigger a second cleanup with Failed state.
    fn on_run_end(&self, _end: &crate::RunEnd) -> Result<(), String> { Ok(()) }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
    /// Additional media. Text stays in `output` so existing middleware can edit/spill it.
    pub parts: Vec<Part>,
    pub structured_content: Option<Value>,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self { output: output.into(), is_error: false, parts: Vec::new(), structured_content: None }
    }

    pub fn error(output: impl Into<String>) -> Self {
        Self { is_error: true, ..Self::ok(output) }
    }

    pub fn with_part(mut self, part: Part) -> Self {
        self.parts.push(part);
        self
    }

    pub fn with_structured_content(mut self, value: Value) -> Self {
        self.structured_content = Some(value);
        self
    }

    /// Preserves error state and media in persistent history.
    pub fn message(&self, call_id: impl Into<String>) -> Message {
        let mut message = Message::tool(call_id, &self.output);
        message.parts.extend(self.parts.clone());
        message.is_error = self.is_error;
        message.structured_content = self.structured_content.clone();
        message
    }
}

pub enum ToolGate {
    Proceed,
    /// The reason is returned to the model as an error result.
    Block(String),
}

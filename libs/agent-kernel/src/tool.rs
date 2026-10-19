use serde_json::Value;
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
            risk: Risk::Read,
            parallel_safe: true,
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
    pub(crate) run_cancel: Arc<AtomicBool>,
    pub(crate) call_cancel: Arc<AtomicBool>,
}

impl ToolCtx {
    pub fn session(&self) -> &str {
        &self.session
    }

    /// A context that is never cancelled, for calling a tool directly, for example in a host's tests.
    pub fn detached(session: &str) -> Self {
        Self { session: session.to_string(), run_cancel: Arc::new(AtomicBool::new(false)), call_cancel: Arc::new(AtomicBool::new(false)) }
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
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self { output: output.into(), is_error: false }
    }

    pub fn error(output: impl Into<String>) -> Self {
        Self { output: output.into(), is_error: true }
    }
}

pub enum ToolGate {
    Proceed,
    /// The reason is returned to the model as an error result.
    Block(String),
}

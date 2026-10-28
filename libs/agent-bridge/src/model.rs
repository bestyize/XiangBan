//! Common interface and event types.
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

/// Supported local agent programs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    /// OpenAI Codex CLI.
    Codex,
    /// Anthropic Claude Code.
    ClaudeCode,
}
/// Result of probing a local executable.
#[derive(Debug, Clone)]
pub struct Availability {
    /// Whether an executable was found.
    pub installed: bool,
    /// Executable path, if found.
    pub path: Option<PathBuf>,
    /// Version returned by the executable.
    pub version: Option<String>,
    /// Non-sensitive diagnostic notes.
    pub notes: Vec<String>,
}
/// Permission mode. Neither mode grants blanket approval.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PermissionProfile {
    /// Most restrictive available settings.
    #[default]
    Strict,
    /// Allow reads while asking for other operations.
    AskForWrites,
}
/// Request to create a session and start its first turn.
#[derive(Debug, Clone)]
pub struct StartRequest {
    /// Working directory.
    pub cwd: PathBuf,
    /// First user instruction.
    pub prompt: String,
    /// Permission settings.
    pub profile: PermissionProfile,
    /// Optional model.
    pub model: Option<String>,
    /// Optional display name.
    pub name: Option<String>,
}
/// Metadata returned by an agent's supported listing interface.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Agent-assigned identifier.
    pub id: String,
    /// Optional title.
    pub title: Option<String>,
    /// Working directory.
    pub cwd: Option<PathBuf>,
    /// Updated timestamp, in seconds.
    pub updated_at: Option<i64>,
    /// Agent implementation.
    pub kind: AgentKind,
}
/// Type of approval requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalKind {
    /// Command execution.
    Command,
    /// File modification.
    FileChange,
    /// Additional permissions.
    Permission,
    /// User input.
    Question,
}
/// Approximate risk classification, not a security guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    /// No high-risk indicators were found.
    Low,
    /// A high-risk indicator was found.
    High,
}
/// An option shown for an approval.
#[derive(Debug, Clone)]
pub struct ApprovalOption {
    /// Machine-readable option identifier.
    pub id: String,
    /// Display label.
    pub label: String,
}
/// A decision awaiting the caller.
#[derive(Debug, Clone)]
pub struct Approval {
    /// Identifier for this request.
    pub id: String,
    /// Category.
    pub kind: ApprovalKind,
    /// Short title.
    pub title: String,
    /// Details provided by the agent.
    pub detail: String,
    /// Heuristic risk.
    pub risk: Risk,
    /// Available choices.
    pub options: Vec<ApprovalOption>,
}
/// Caller's decision.
#[derive(Debug, Clone)]
pub enum Decision {
    /// Allow once.
    Allow,
    /// Allow for the current session where supported.
    AllowForSession,
    /// Decline.
    Deny,
    /// Answer a question.
    Answer(String),
}
/// Usage information reported by the agent.
#[derive(Debug, Clone, Default)]
pub struct Usage {
    /// Input tokens, if reported.
    pub input_tokens: Option<u64>,
    /// Output tokens, if reported.
    pub output_tokens: Option<u64>,
}
/// Events produced by a session.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Agent session identifier is available.
    SessionStarted {
        /// Identifier.
        session: String,
    },
    /// Activity changed.
    Status {
        /// Whether a turn is running.
        busy: bool,
    },
    /// Incremental assistant text.
    TextDelta {
        /// New text.
        text: String,
    },
    /// A complete assistant message.
    Message {
        /// Message text.
        text: String,
    },
    /// A tool began.
    ToolStarted {
        /// Tool call identifier.
        id: String,
        /// Tool name.
        name: String,
        /// Summary.
        summary: String,
    },
    /// A tool ended.
    ToolFinished {
        /// Tool call identifier.
        id: String,
        /// Whether successful.
        ok: bool,
        /// Truncated output.
        preview: String,
    },
    /// A file was changed.
    FileChanged {
        /// Path.
        path: String,
    },
    /// A decision is required.
    ApprovalRequested(Approval),
    /// A pending decision was resolved elsewhere.
    ApprovalResolved {
        /// Request identifier.
        id: String,
    },
    /// Turn ended.
    TurnCompleted {
        /// Final assistant text if available.
        final_text: Option<String>,
        /// Token counts if available.
        usage: Option<Usage>,
    },
    /// Error without credentials or file contents.
    Error {
        /// Stable category.
        code: String,
        /// Sanitized description.
        message: String,
    },
    /// Process closed.
    Closed,
}
/// Bridge failures. Messages deliberately do not include raw agent output.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    /// Program was not found.
    #[error("agent executable is not installed")]
    NotInstalled,
    /// Login is needed.
    #[error("agent is not logged in")]
    NotLoggedIn,
    /// Malformed or unexpected protocol data.
    #[error("agent protocol error")]
    Protocol,
    /// Process exited or failed to communicate.
    #[error("agent process exited")]
    ProcessExited,
    /// A request timed out.
    #[error("agent request timed out")]
    Timeout,
    /// Feature unavailable.
    #[error("operation is unsupported")]
    Unsupported,
    /// Invalid caller input.
    #[error("invalid caller argument")]
    InvalidArgument,
}
/// Backend for one kind of external agent.
#[async_trait]
pub trait ExternalAgent: Send + Sync {
    /// Agent kind.
    fn kind(&self) -> AgentKind;
    /// Probe without starting a conversation.
    async fn availability(&self) -> Availability;
    /// List sessions available through a supported interface.
    async fn list_sessions(&self, limit: usize) -> Result<Vec<SessionInfo>, BridgeError>;
    /// Start a session.
    async fn start(&self, req: StartRequest) -> Result<Box<dyn Session>, BridgeError>;
    /// Resume an existing session.
    async fn resume(
        &self,
        id: &str,
        cwd: &Path,
        profile: PermissionProfile,
    ) -> Result<Box<dyn Session>, BridgeError>;
}
/// A running agent session. Event receiver can be taken once.
#[async_trait]
pub trait Session: Send {
    /// Session identifier.
    fn id(&self) -> &str;
    /// Take the receiver. Subsequent calls return a closed receiver.
    fn events(&mut self) -> mpsc::Receiver<AgentEvent>;
    /// Start a new turn.
    async fn send(&self, text: &str) -> Result<(), BridgeError>;
    /// Append instruction to a running turn.
    async fn steer(&self, text: &str) -> Result<(), BridgeError>;
    /// Interrupt the active turn.
    async fn interrupt(&self) -> Result<(), BridgeError>;
    /// Resolve a pending approval.
    async fn answer(&self, approval_id: &str, decision: Decision) -> Result<(), BridgeError>;
    /// Close and reap the child.
    async fn close(self: Box<Self>) -> Result<(), BridgeError>;
}
/// Classify obvious destructive, network, installation, and outside-workspace actions.
/// This heuristic cannot establish safety; callers must inspect the full request.
pub fn classify_risk(detail: &str, cwd: &Path) -> Risk {
    let text = detail.to_lowercase();
    let dangerous = [
        "rm ",
        "rm-",
        "rmdir",
        "del ",
        "remove-item",
        "overwrite",
        "delete",
        "truncate",
        "curl ",
        "wget ",
        "http://",
        "https://",
        "npm install",
        "npm i ",
        "pip install",
        "cargo install",
        "apt install",
        "brew install",
        "git push",
        " > ",
        " >> ",
    ];
    if dangerous.iter().any(|s| text.contains(s)) {
        return Risk::High;
    }
    let mut parts = vec![detail.to_owned()];
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(detail) {
        fn collect(value: &serde_json::Value, parts: &mut Vec<String>) {
            match value {
                serde_json::Value::String(s) => parts.push(s.clone()),
                serde_json::Value::Array(v) => v.iter().for_each(|v| collect(v, parts)),
                serde_json::Value::Object(v) => v.values().for_each(|v| collect(v, parts)),
                _ => {}
            }
        }
        collect(&value, &mut parts);
    }
    for word in parts.iter().flat_map(|part| {
        part.split(|c: char| {
            c.is_whitespace() || matches!(c, '"' | '\'' | ',' | ';' | '{' | '}' | '[' | ']')
        })
    }) {
        if word.starts_with("../") || word.starts_with("..\\") {
            return Risk::High;
        }
        let p = Path::new(word);
        if (p.is_absolute() || word.starts_with('/')) && !p.starts_with(cwd) {
            return Risk::High;
        }
    }
    Risk::Low
}

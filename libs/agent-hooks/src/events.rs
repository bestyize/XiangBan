//! Event and response types.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Unique identifier for a received event within a server instance.
pub type EventId = u64;
/// An event with common and documented event-specific fields. Unknown fields are preserved.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HookEvent {
    /// Receiver-generated identifier; not part of the incoming payload.
    #[serde(skip)]
    pub id: EventId,
    /// Session identifier.
    #[serde(default)]
    pub session_id: String,
    /// Transcript path, if provided (the library never reads it).
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Working directory.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Event name in the payload.
    #[serde(default)]
    pub hook_event_name: String,
    /// Current permission mode.
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Prompt identifier, if present.
    #[serde(default)]
    pub prompt_id: Option<String>,
    /// Tool name, if applicable.
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Tool input, if applicable.
    #[serde(default)]
    pub tool_input: Option<Value>,
    /// Tool use identifier, if applicable.
    #[serde(default)]
    pub tool_use_id: Option<String>,
    /// Tool response, if applicable.
    #[serde(default)]
    pub tool_response: Option<Value>,
    /// Suggested permission updates.
    #[serde(default)]
    pub permission_suggestions: Option<Value>,
    /// User prompt.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Notification type.
    #[serde(default)]
    pub notification_type: Option<String>,
    /// Notification message.
    #[serde(default)]
    pub message: Option<String>,
    /// Session end reason.
    #[serde(default)]
    pub reason: Option<String>,
    /// Recursion guard supplied by Claude Code.
    #[serde(default)]
    pub stop_hook_active: bool,
    /// Unknown fields retained for forward compatibility.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
/// Decision for a permission request.
#[derive(Clone, Debug)]
pub enum HookDecision {
    /// Allow the pending tool invocation.
    Allow,
    /// Deny with an optional explanation.
    Deny { message: Option<String> },
    /// Leave the decision to Claude Code.
    NoOpinion,
}

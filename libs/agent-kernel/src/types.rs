use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

/// Bump when an event or message field changes incompatibly.
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    Image { url: String },
}

/// Only provider-exposed displayable reasoning; opaque continuation state stays in provider_state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningKind { Summary, Text }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningSegment {
    pub kind: ReasoningKind,
    pub text: String,
}

/// `arguments` stays the raw JSON string from the model so history round-trips exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Host-authored context record, never inferred from text supplied by the user or a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextSummary {
    pub keep_from: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    #[serde(default)]
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Written by the harness rather than the person, for example a nudge to keep working. The model
    /// sees it as an ordinary user message; hosts usually leave it out of the visible conversation.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub synthetic: bool,
    /// Tool errors remain distinguishable after saving/reloading. Old histories default to false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_summary: Option<ContextSummary>,
    /// Opaque protocol continuation state. It is persisted, but is not visible assistant text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_state: Option<serde_json::Value>,
    /// Persist only text the endpoint explicitly exposed for display, never opaque reasoning items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasoning: Vec<ReasoningSegment>,
}

impl Message {
    fn with(role: Role, parts: Vec<Part>) -> Self {
        Self { role, parts, tool_calls: Vec::new(), tool_call_id: None, synthetic: false, is_error: false, structured_content: None, context_summary: None, provider_state: None, reasoning: Vec::new() }
    }

    /// A user-role message from the harness itself (see `synthetic`).
    pub fn reminder(text: impl Into<String>) -> Self {
        Self { synthetic: true, ..Self::user_text(text) }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self::with(Role::System, vec![Part::Text { text: text.into() }])
    }

    pub fn user(parts: Vec<Part>) -> Self {
        Self::with(Role::User, parts)
    }

    pub fn user_text(text: impl Into<String>) -> Self {
        Self::user(vec![Part::Text { text: text.into() }])
    }

    pub fn assistant(text: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        let text = text.into();
        let parts = if text.is_empty() { Vec::new() } else { vec![Part::Text { text }] };
        Self { tool_calls, ..Self::with(Role::Assistant, parts) }
    }

    pub fn tool(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            tool_call_id: Some(call_id.into()),
            ..Self::with(Role::Tool, vec![Part::Text { text: output.into() }])
        }
    }

    /// Concatenated text parts; images are skipped.
    pub fn text(&self) -> String {
        self.parts
            .iter()
            .filter_map(|p| match p {
                Part::Text { text } => Some(text.as_str()),
                Part::Image { .. } => None,
            })
            .collect()
    }

    /// JSON is kept separate in storage and appended only when constructing a model request.
    pub fn tool_text(&self) -> String {
        let mut text = self.text();
        if let Some(value) = &self.structured_content {
            if !text.is_empty() { text.push('\n'); }
            text.push_str(&value.to_string());
        }
        text
    }

    /// Protocols without a native tool error flag get an explicit text marker.
    pub fn tool_text_with_error(&self) -> String {
        let text = self.tool_text();
        if self.is_error { format!("Error: {text}") } else { text }
    }
}

/// Why the model stopped. `Length` means the reply may be cut off mid-way, which for a tool call
/// means its arguments may be incomplete.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// The model finished on its own.
    #[default]
    Stop,
    /// The model stopped to call tools.
    ToolCalls,
    /// Generation hit the output or context limit.
    Length,
    /// The provider's safety system ended or replaced the reply.
    ContentFilter,
    Other,
}

impl FinishReason {
    /// Providers disagree on how a reply that contains tool calls is labelled. Tool calls win over
    /// a plain `Stop`, and a `ToolCalls` label without any call is just `Stop`.
    pub(crate) fn settle(self, has_calls: bool) -> Self {
        match (self, has_calls) {
            (Self::Stop, true) => Self::ToolCalls,
            (Self::ToolCalls, false) => Self::Stop,
            (other, _) => other,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

impl AddAssign for Usage {
    fn add_assign(&mut self, rhs: Self) {
        self.prompt_tokens = self.prompt_tokens.saturating_add(rhs.prompt_tokens);
        self.completion_tokens = self.completion_tokens.saturating_add(rhs.completion_tokens);
    }
}

/// Streamed to the host UI or the stdio adapter; wire form is `{"type": "...", ...}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    TextDelta { text: String },
    ReasoningDelta { kind: ReasoningKind, text: String },
    ToolStart { id: String, name: String, arguments: String },
    /// Emitted after validation and approval, immediately before dispatch. Observers may fail
    /// closed at this point. A crash after this event leaves the action's outcome uncertain.
    ToolDispatched { id: String },
    ToolEnd {
        id: String, output: String, is_error: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        parts: Vec<Part>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        structured_content: Option<serde_json::Value>,
    },
    Usage { prompt_tokens: u32, completion_tokens: u32 },
    Retrying { attempt: u32, delay_ms: u64, message: String },
    /// A reply ended because it hit the length limit. Tool calls in it are not run.
    Truncated,
    /// The model stopped although the work looked unfinished, so the harness asked it to go on.
    /// `reason` is a short code from the middleware, such as `unfinished_todos` or `length`.
    Continuing { reason: String },
    Done,
    Cancelled,
    Error { message: String },
}

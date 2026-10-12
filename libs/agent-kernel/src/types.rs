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

/// `arguments` stays the raw JSON string from the model so history round-trips exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
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
}

impl Message {
    fn with(role: Role, parts: Vec<Part>) -> Self {
        Self { role, parts, tool_calls: Vec::new(), tool_call_id: None }
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
    ToolStart { id: String, name: String, arguments: String },
    ToolEnd { id: String, output: String, is_error: bool },
    Usage { prompt_tokens: u32, completion_tokens: u32 },
    Retrying { attempt: u32, delay_ms: u64, message: String },
    /// A reply ended because it hit the length limit. Tool calls in it are not run.
    Truncated,
    Done,
    Cancelled,
    Error { message: String },
}

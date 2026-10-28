use crate::error::HarnessError;
use crate::types::{FinishReason, Message, Usage};
use serde_json::Value;
use std::sync::atomic::AtomicBool;

pub mod anthropic;
pub mod responses;
pub mod scripted;
pub mod stream;
pub mod wire;
mod budget;

#[cfg(any(feature = "openai", feature = "anthropic"))]
mod http;

#[cfg(feature = "openai")]
pub mod openai;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatRequest {
    /// Includes the system message, if any, as the first entry.
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSchema>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub message: Message,
    pub usage: Option<Usage>,
    pub finish: FinishReason,
}

/// A tool call whose arguments are still being streamed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolProgress {
    pub id: String,
    pub name: String,
    /// The `path` argument, once the model has written it out completely.
    pub path: Option<String>,
    /// Argument bytes received so far.
    pub bytes: usize,
}

/// The value of the first `"path"` string in a possibly half-written JSON arguments text.
pub(crate) fn path_hint(partial: &str) -> Option<String> {
    let after = &partial[partial.find("\"path\"")? + 6..];
    let after = after.trim_start().strip_prefix(':')?.trim_start().strip_prefix('"')?;
    let mut value = String::new();
    let mut chars = after.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(value),
            '\\' => value.push(chars.next()?),
            c => value.push(c),
        }
    }
    None
}

pub trait ModelProvider: Send + Sync {
    /// Streams text through `on_text` and returns the assembled reply.
    /// Implementations should check `cancel` between reads and return `Cancelled`.
    /// Mark an error `retryable` only when repeating the identical request is safe.
    fn chat(
        &self,
        request: &ChatRequest,
        on_text: &mut dyn FnMut(&str),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError>;

    /// Like `chat_with_reasoning`, and `on_tool` is told how far a tool call's arguments have streamed (see `ToolProgress`),
    /// so a long silent write can be shown as progress. Providers that cannot tell keep this default.
    fn chat_with_progress(
        &self,
        request: &ChatRequest,
        on_text: &mut dyn FnMut(&str),
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
        _on_tool: &mut dyn FnMut(&ToolProgress),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        self.chat_with_reasoning(request, on_text, on_reasoning, cancel)
    }

    /// A display-only reasoning channel. Older/custom providers may implement
    /// just `chat`; the default preserves binary-free source compatibility.
    fn chat_with_reasoning(
        &self,
        request: &ChatRequest,
        on_text: &mut dyn FnMut(&str),
        _on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        self.chat(request, on_text, cancel)
    }
}

/// For auxiliary calls outside Agent::run (e.g. compression). Root/child loops already meter themselves.
pub struct MeteredProvider{pub provider:std::sync::Arc<dyn ModelProvider>,pub control:crate::RunControl}
impl ModelProvider for MeteredProvider{
 fn chat(&self,r:&ChatRequest,text:&mut dyn FnMut(&str),cancel:&AtomicBool)->Result<Reply,HarnessError>{self.control.check()?;let reply=self.provider.chat(r,text,cancel)?;if let Some(u)=reply.usage{self.control.record(u);}self.control.check()?;Ok(reply)}
}

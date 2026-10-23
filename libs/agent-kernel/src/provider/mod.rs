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

use super::{ChatRequest, ModelProvider, Reply};
use crate::error::HarnessError;
use crate::types::{FinishReason, Message, ToolCall, Usage};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// One scripted model response. Hosts can use `ScriptedProvider` in their own tests too.
#[derive(Debug, Clone, Default)]
pub struct Step {
    chunks: Vec<String>,
    tool_calls: Vec<ToolCall>,
    usage: Option<Usage>,
    finish: Option<FinishReason>,
    fail: Option<HarnessError>,
}

impl Step {
    pub fn text(text: &str) -> Self {
        Self { chunks: vec![text.to_string()], ..Self::default() }
    }

    pub fn chunks(chunks: &[&str]) -> Self {
        Self { chunks: chunks.iter().map(|c| c.to_string()).collect(), ..Self::default() }
    }

    pub fn tool_call(id: &str, name: &str, arguments: &str) -> Self {
        let call = ToolCall { id: id.into(), name: name.into(), arguments: arguments.into() };
        Self { tool_calls: vec![call], ..Self::default() }
    }

    pub fn error(error: HarnessError) -> Self {
        Self { fail: Some(error), ..Self::default() }
    }

    pub fn and_tool_call(mut self, id: &str, name: &str, arguments: &str) -> Self {
        self.tool_calls.push(ToolCall { id: id.into(), name: name.into(), arguments: arguments.into() });
        self
    }

    pub fn with_usage(mut self, prompt_tokens: u32, completion_tokens: u32) -> Self {
        self.usage = Some(Usage { prompt_tokens, completion_tokens, ..Default::default() });
        self
    }

    /// Overrides the finish reason, which otherwise is `ToolCalls` when there are calls and `Stop` if not.
    pub fn finish(mut self, reason: FinishReason) -> Self {
        self.finish = Some(reason);
        self
    }

    /// Emit the text chunks first, then fail, like a stream that drops mid-way.
    pub fn then_fail(mut self, error: HarnessError) -> Self {
        self.fail = Some(error);
        self
    }
}

pub struct ScriptedProvider {
    steps: Mutex<VecDeque<Step>>,
    requests: Mutex<Vec<ChatRequest>>,
}

impl ScriptedProvider {
    pub fn new(steps: Vec<Step>) -> Self {
        Self { steps: Mutex::new(steps.into()), requests: Mutex::new(Vec::new()) }
    }

    /// Every request received so far, for assertions.
    pub fn requests(&self) -> Vec<ChatRequest> {
        self.requests.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl ModelProvider for ScriptedProvider {
    fn chat(
        &self,
        request: &ChatRequest,
        on_text: &mut dyn FnMut(&str),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(HarnessError::Cancelled);
        }
        self.requests.lock().unwrap_or_else(|e| e.into_inner()).push(request.clone());
        let step = self.steps.lock().unwrap_or_else(|e| e.into_inner()).pop_front();
        let Some(step) = step else {
            return Err(HarnessError::provider("script exhausted", false));
        };
        for chunk in &step.chunks {
            on_text(chunk);
        }
        if let Some(error) = step.fail {
            return Err(error);
        }
        let finish = step.finish.unwrap_or(if step.tool_calls.is_empty() { FinishReason::Stop } else { FinishReason::ToolCalls });
        let message = Message::assistant(step.chunks.concat(), step.tool_calls);
        Ok(Reply { message, usage: step.usage, finish })
    }
}

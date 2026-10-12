use super::Reply;
use crate::error::HarnessError;
use crate::types::{FinishReason, Message, ToolCall, Usage};
use serde_json::Value;

const MAX_TOOL_CALLS: usize = 128;

/// Minimal server-sent-events reader: feed lines, get one payload per completed event.
#[derive(Default)]
pub struct SseParser {
    data: Vec<String>,
}

impl SseParser {
    pub fn push_line(&mut self, line: &str) -> Option<String> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            return self.flush();
        }
        if let Some(rest) = line.strip_prefix("data:") {
            self.data.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
        }
        None
    }

    /// Call at end of input to release an event that lacked a trailing blank line.
    pub fn flush(&mut self) -> Option<String> {
        if self.data.is_empty() {
            return None;
        }
        Some(std::mem::take(&mut self.data).join("\n"))
    }
}

#[derive(Default)]
struct PartialCall {
    id: String,
    name: String,
    arguments: String,
}

/// Rebuilds one assistant reply from OpenAI-style streaming chunks.
#[derive(Default)]
pub struct StreamAccumulator {
    text: String,
    calls: Vec<PartialCall>,
    usage: Option<Usage>,
    finish_reason: Option<String>,
    finished: bool,
}

fn chat_finish(reason: Option<&str>) -> FinishReason {
    match reason {
        Some("stop") | None => FinishReason::Stop,
        Some("tool_calls" | "function_call") => FinishReason::ToolCalls,
        Some("length") => FinishReason::Length,
        Some("content_filter") => FinishReason::ContentFilter,
        Some(_) => FinishReason::Other,
    }
}

impl StreamAccumulator {
    pub fn push(&mut self, chunk: &Value, on_text: &mut dyn FnMut(&str)) -> Result<(), HarnessError> {
        if let Some(error) = chunk.get("error").filter(|e| !e.is_null()) {
            let message = error.get("message").and_then(Value::as_str).map_or_else(|| error.to_string(), str::to_string);
            return Err(HarnessError::provider(message, false));
        }
        if let Some(usage) = chunk.get("usage").filter(|u| u.is_object()) {
            let read = |key: &str| usage.get(key).and_then(Value::as_u64).unwrap_or(0).min(u32::MAX as u64) as u32;
            self.usage = Some(Usage { prompt_tokens: read("prompt_tokens"), completion_tokens: read("completion_tokens") });
        }
        let Some(choice) = chunk.get("choices").and_then(|c| c.get(0)) else { return Ok(()) };
        if let Some(delta) = choice.get("delta") {
            if let Some(text) = delta.get("content").and_then(Value::as_str).filter(|t| !t.is_empty()) {
                self.text.push_str(text);
                on_text(text);
            }
            for call in delta.get("tool_calls").and_then(Value::as_array).into_iter().flatten() {
                self.merge_call(call);
            }
        }
        if let Some(reason) = choice.get("finish_reason").filter(|r| !r.is_null()) {
            self.finished = true;
            self.finish_reason = reason.as_str().map(str::to_string);
        }
        Ok(())
    }

    fn merge_call(&mut self, call: &Value) {
        let id = call.get("id").and_then(Value::as_str).filter(|s| !s.is_empty());
        let index = match call.get("index").and_then(Value::as_u64) {
            Some(i) => i as usize,
            None if id.is_some() => self.calls.len(),
            None => self.calls.len().saturating_sub(1),
        };
        if index >= MAX_TOOL_CALLS {
            return;
        }
        while self.calls.len() <= index {
            self.calls.push(PartialCall::default());
        }
        let slot = &mut self.calls[index];
        if let (Some(id), true) = (id, slot.id.is_empty()) {
            slot.id = id.to_string();
        }
        let function = call.get("function");
        // Some endpoints repeat the name in every fragment, so only the first one counts.
        if let (Some(name), true) = (function.and_then(|f| f.get("name")).and_then(Value::as_str), slot.name.is_empty()) {
            slot.name = name.to_string();
        }
        if let Some(args) = function.and_then(|f| f.get("arguments")).and_then(Value::as_str) {
            slot.arguments.push_str(args);
        }
    }

    /// Mark the stream complete when the server sends `[DONE]`.
    pub fn mark_done(&mut self) {
        self.finished = true;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn finish(self) -> Reply {
        let tool_calls: Vec<ToolCall> = self
            .calls
            .into_iter()
            .enumerate()
            .filter(|(_, c)| !c.name.is_empty())
            .map(|(i, c)| ToolCall {
                id: if c.id.is_empty() { format!("call_{i}") } else { c.id },
                name: c.name,
                arguments: if c.arguments.trim().is_empty() { "{}".to_string() } else { c.arguments },
            })
            .collect();
        let finish = chat_finish(self.finish_reason.as_deref()).settle(!tool_calls.is_empty());
        Reply { message: Message::assistant(self.text, tool_calls), usage: self.usage, finish }
    }
}

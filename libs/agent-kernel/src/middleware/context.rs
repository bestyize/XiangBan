use super::{Middleware, Turn};
use crate::error::HarnessError;
use crate::types::{Message, Part, Role};
use crate::provider::ToolSchema;

const CLEARED: &str = "[older tool output cleared to save context]";
const OMITTED: &str = "[Earlier messages in this conversation were omitted to fit the context window.]";
// A conservative, size-sensitive allowance, not a provider tokenizer. Compressed images
// and remote URLs can still cost more; explicit upstream overflow has a bounded recovery.
const IMAGE_TOKENS: usize = 4096;
// A dropped question is kept word for word so a constraint the user set is not lost, but only up to a size: a long
// pasted document in an early message must not keep the conversation over budget for good.
const PINNED_REQUEST_CHARS: usize = 2_000;
const PINNED_TOTAL_CHARS: usize = 8_000;
const MESSAGE_OVERHEAD: usize = 4;

/// Rough token count: ASCII at about four bytes per token, everything else at one per character.
fn estimate_text(text: &str) -> usize {
    let ascii = text.bytes().filter(u8::is_ascii).count();
    ascii.div_ceil(4) + (text.chars().count() - ascii)
}

fn estimate_message(message: &Message) -> usize {
    let parts: usize = message
        .parts
        .iter()
        .map(|p| match p {
            Part::Text { text } => estimate_text(text),
            Part::Image { url } => IMAGE_TOKENS + if url.starts_with("data:") { url.len().div_ceil(128) } else { 0 },
        })
        .sum();
    let calls: usize = message.tool_calls.iter().map(|c| estimate_text(&c.name) + estimate_text(&c.arguments)).sum();
    let structured = message.structured_content.as_ref().map_or(0, |v| estimate_text(&v.to_string()));
    let state = message.provider_state.as_ref().map_or(0, |v| estimate_text(&v.to_string()));
    MESSAGE_OVERHEAD + parts + calls + structured + state
}

/// The start of a long question; what was left out is counted so the model knows the text is cut.
fn shortened(text: &str) -> String {
    let total = text.chars().count();
    if total <= PINNED_REQUEST_CHARS { return text.to_string(); }
    format!("{}…[this request was longer; {} more characters omitted]", text.chars().take(PINNED_REQUEST_CHARS).collect::<String>(), total - PINNED_REQUEST_CHARS)
}

/// The requests that fit the allowance, newest first kept: when there are too many, the oldest are named as omitted.
fn pinned_text(requests: &[String]) -> String {
    let mut kept = 0;
    let mut chars = 0;
    for request in requests.iter().rev() {
        chars += request.chars().count();
        if kept > 0 && chars > PINNED_TOTAL_CHARS { break; }
        kept += 1;
    }
    let skipped = requests.len() - kept;
    let note = if skipped > 0 { format!("({skipped} earlier requests omitted)\n\n") } else { String::new() };
    format!("{OMITTED}\nOriginal user requests, in order; later corrections take precedence:\n{note}{}", requests[skipped..].join("\n\n"))
}

pub fn estimate_tokens(messages: &[Message]) -> usize {
    messages.iter().map(estimate_message).sum()
}

pub fn estimate_tool_tokens(tools: &[ToolSchema]) -> usize {
    tools.iter().map(|tool| {
        MESSAGE_OVERHEAD + estimate_text(&tool.name) + estimate_text(&tool.description) + estimate_text(&tool.parameters.to_string())
    }).sum()
}

/// Keeps requests under a token budget without calling a model: first clears old tool outputs,
/// then drops whole oldest exchanges. The stored session is never modified.
pub struct ContextTrim {
    max_tokens: usize,
    keep_recent: usize,
    reserve_output: usize,
}

impl ContextTrim {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens, keep_recent: 6, reserve_output: 0 }
    }

    /// Reserve room for output and estimation error within the model's total context window.
    pub fn reserve_output(mut self, tokens: usize) -> Self {
        self.reserve_output = tokens;
        self
    }

    /// How many of the newest messages keep their tool outputs intact.
    pub fn keep_recent(mut self, messages: usize) -> Self {
        self.keep_recent = messages;
        self
    }
}

impl Middleware for ContextTrim {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let tool_tokens = estimate_tool_tokens(&turn.tools);
        let budget = self.max_tokens.saturating_sub(self.reserve_output).saturating_sub(tool_tokens);
        let messages = &mut turn.messages;
        let mut total = estimate_tokens(messages);
        if total <= budget && self.reserve_output.saturating_add(tool_tokens) <= self.max_tokens {
            return Ok(());
        }

        let placeholder_cost = MESSAGE_OVERHEAD + estimate_text(CLEARED);
        let protect_from = messages.len().saturating_sub(self.keep_recent);
        for message in &mut messages[..protect_from] {
            if total <= budget {
                break;
            }
            if message.role != Role::Tool {
                continue;
            }
            let cost = estimate_message(message);
            if cost <= placeholder_cost {
                continue;
            }
            message.parts = vec![Part::Text { text: format!("{CLEARED}; is_error={}; call_id={}",message.is_error,message.tool_call_id.as_deref().unwrap_or("unknown")) }];
            message.structured_content = None;
            total = total - cost + estimate_message(message);
        }

        let first = messages.iter().position(|m| m.role != Role::System).unwrap_or(messages.len());
        let mut dropped = false;
        let mut requests=Vec::new();
        let pinned = |requests:&[String]| {
            let mut m=Message::reminder(pinned_text(requests));
            m.structured_content=Some(serde_json::json!({"harness_context":"user_requests"}));m
        };
        while total.saturating_add(if dropped {estimate_message(&pinned(&requests))}else{0})>budget {
            let Some(offset)=messages.get(first+1..).and_then(|rest|rest.iter().position(|m|m.role==Role::User&&!m.synthetic))else{break;};
            let removed:usize=messages.drain(first..first+1+offset).map(|m|{
                if m.role==Role::User&&!m.synthetic{requests.push(shortened(&m.text()));}
                else if m.role==Role::User&&m.structured_content.as_ref().is_some_and(|v|v["harness_context"]=="user_requests"){requests.push(m.text());}
                estimate_message(&m)
            }).sum();total=total.saturating_sub(removed);dropped=true;
        }
        if dropped{messages.insert(first,pinned(&requests));}
        let used = estimate_tokens(messages).saturating_add(tool_tokens).saturating_add(self.reserve_output);
        if used > self.max_tokens {
            return Err(HarnessError::Middleware(format!(
                "context budget exceeded: estimated {used} tokens including tool definitions and output reserve, limit {}. Split the current input, reduce tools, or compact the conversation.", self.max_tokens
            )));
        }
        Ok(())
    }
}

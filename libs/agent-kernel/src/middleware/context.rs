use super::{Middleware, Turn};
use crate::error::HarnessError;
use crate::types::{Message, Part, Role};

const CLEARED: &str = "[older tool output cleared to save context]";
const OMITTED: &str = "[Earlier messages in this conversation were omitted to fit the context window.]";
const IMAGE_TOKENS: usize = 800;
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
            Part::Image { .. } => IMAGE_TOKENS,
        })
        .sum();
    let calls: usize = message.tool_calls.iter().map(|c| estimate_text(&c.name) + estimate_text(&c.arguments)).sum();
    MESSAGE_OVERHEAD + parts + calls
}

pub fn estimate_tokens(messages: &[Message]) -> usize {
    messages.iter().map(estimate_message).sum()
}

/// Keeps requests under a token budget without calling a model: first clears old tool outputs,
/// then drops whole oldest exchanges. The stored session is never modified.
pub struct ContextTrim {
    max_tokens: usize,
    keep_recent: usize,
}

impl ContextTrim {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens, keep_recent: 6 }
    }

    /// How many of the newest messages keep their tool outputs intact.
    pub fn keep_recent(mut self, messages: usize) -> Self {
        self.keep_recent = messages;
        self
    }
}

impl Middleware for ContextTrim {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let messages = &mut turn.messages;
        let mut total = estimate_tokens(messages);
        if total <= self.max_tokens {
            return Ok(());
        }

        let placeholder_cost = MESSAGE_OVERHEAD + estimate_text(CLEARED);
        let protect_from = messages.len().saturating_sub(self.keep_recent);
        for message in &mut messages[..protect_from] {
            if total <= self.max_tokens {
                break;
            }
            if message.role != Role::Tool {
                continue;
            }
            let cost = estimate_message(message);
            if cost <= placeholder_cost {
                continue;
            }
            message.parts = vec![Part::Text { text: CLEARED.to_string() }];
            total = total - cost + placeholder_cost;
        }

        let first = messages.iter().position(|m| m.role != Role::System).unwrap_or(messages.len());
        let mut dropped = false;
        while total > self.max_tokens {
            // Cut only at user messages so a tool call is never separated from its result.
            let Some(offset) = messages.get(first + 1..).and_then(|rest| rest.iter().position(|m| m.role == Role::User)) else {
                break;
            };
            let removed: usize = messages.drain(first..first + 1 + offset).map(|m| estimate_message(&m)).sum();
            total = total.saturating_sub(removed);
            dropped = true;
        }
        if dropped {
            messages.insert(first, Message::user_text(OMITTED));
        }
        Ok(())
    }
}

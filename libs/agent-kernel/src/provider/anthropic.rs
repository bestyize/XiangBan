//! Anthropic Messages protocol (`/v1/messages`) with streaming.
//!
//! Request and stream formats are written from the public protocol description and checked only
//! against local fake servers, not against a live endpoint. Not supported: extended thinking
//! (thinking blocks are not carried between turns, so do not enable it in `extra_body` when tools
//! are in use), server-side tools, prompt-cache markers.

use super::wire::ProviderProfile;
use super::{ChatRequest, Reply};
use crate::error::HarnessError;
use crate::types::{FinishReason, Message, Part, Role, ToolCall, Usage};
use serde_json::{json, Value};

// Deprecated compatibility surface; the policy and recovery are now shared
// through `agent_kernel::model` rather than implemented separately by this provider.
pub const DEFAULT_MAX_TOKENS: u32 = 128_000;

pub fn model_max_output_tokens(model: &str) -> u32 {
    crate::model::ModelCapabilities::resolve(
        crate::model::ModelProtocol::AnthropicMessages, model, None, None
    ).request_output_limit().unwrap_or(DEFAULT_MAX_TOKENS)
}

pub fn next_max_tokens_from_error(message: &str, requested: u32) -> Option<u32> {
    crate::model::recovery::next_output_limit(message, requested)
}

pub const DEFAULT_VERSION: &str = "2023-06-01";
const MAX_BLOCKS: usize = 128;

/// Anthropic's SDKs take the host without `/v1` and add `/v1/messages`; a base that already ends
/// in `/v1` only gets `/messages`.
pub fn endpoint_url(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.ends_with("/v1") { format!("{base}/messages") } else { format!("{base}/v1/messages") }
}

fn text_block(text: &str) -> Option<Value> {
    (!text.trim().is_empty()).then(|| json!({ "type": "text", "text": text }))
}

fn image_block(url: &str) -> Value {
    if let Some((meta, data)) = url.strip_prefix("data:").and_then(|rest| rest.split_once(',')) {
        if let Some(media_type) = meta.strip_suffix(";base64") {
            return json!({ "type": "image", "source": { "type": "base64", "media_type": media_type, "data": data } });
        }
    }
    json!({ "type": "image", "source": { "type": "url", "url": url } })
}

/// Consecutive messages of one role are merged: the API wants all results of one assistant turn
/// in a single user message, with tool results first.
fn push_turn(turns: &mut Vec<(&'static str, Vec<Value>)>, role: &'static str, blocks: Vec<Value>) {
    if blocks.is_empty() {
        return;
    }
    match turns.last_mut() {
        Some((last, existing)) if *last == role => existing.extend(blocks),
        _ => turns.push((role, blocks)),
    }
}

/// System messages anywhere in the history are joined into the top-level `system` field.
/// Empty text is dropped because the API rejects empty text blocks.
pub fn build_request_body(profile: &ProviderProfile, request: &ChatRequest) -> Value {
    let mut system: Vec<String> = Vec::new();
    let mut turns: Vec<(&'static str, Vec<Value>)> = Vec::new();
    for message in &request.messages {
        match message.role {
            Role::System => {
                let text = message.text();
                if !text.trim().is_empty() {
                    system.push(text);
                }
            }
            Role::User => {
                let blocks = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        Part::Text { text } => text_block(text),
                        Part::Image { url } => Some(image_block(url)),
                    })
                    .collect();
                push_turn(&mut turns, "user", blocks);
            }
            Role::Assistant => {
                let mut blocks: Vec<Value> = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        Part::Text { text } => text_block(text),
                        Part::Image { .. } => None,
                    })
                    .collect();
                for call in &message.tool_calls {
                    let input = serde_json::from_str::<Value>(&call.arguments).ok().filter(Value::is_object).unwrap_or_else(|| json!({}));
                    blocks.push(json!({ "type": "tool_use", "id": call.id, "name": call.name, "input": input }));
                }
                push_turn(&mut turns, "assistant", blocks);
            }
            Role::Tool => {
                let mut block = json!({ "type": "tool_result", "tool_use_id": message.tool_call_id.clone().unwrap_or_default() });
                let text = message.tool_text();
                let images: Vec<Value> = message.parts.iter().filter_map(|p| match p {
                    Part::Image { url } => Some(image_block(url)),
                    _ => None,
                }).collect();
                if images.is_empty() {
                    if !text.is_empty() { block["content"] = Value::String(text); }
                } else {
                    let mut content = Vec::new();
                    if let Some(text) = text_block(&text) { content.push(text); }
                    content.extend(images);
                    block["content"] = json!(content);
                }
                if message.is_error { block["is_error"] = json!(true); }
                push_turn(&mut turns, "user", vec![block]);
            }
        }
    }

    let mut body = json!({
        "model": profile.model,
        "max_tokens": profile.max_output_tokens.unwrap_or_else(|| model_max_output_tokens(&profile.model)),
        "stream": true,
        "messages": turns.into_iter().map(|(role, content)| json!({ "role": role, "content": content })).collect::<Vec<_>>(),
    });
    if !system.is_empty() {
        body["system"] = Value::String(system.join("\n\n"));
    }
    if profile.supports_tools && !request.tools.is_empty() {
        body["tools"] = request
            .tools
            .iter()
            .map(|t| json!({ "name": t.name, "description": t.description, "input_schema": t.parameters }))
            .collect();
    }
    for (key, value) in &profile.extra_body {
        body[key] = value.clone();
    }
    body
}

enum Block {
    Text(String),
    Tool { id: String, name: String, partial: String, initial: String },
    Ignored,
}

/// Rebuilds one assistant reply from Messages stream events.
#[derive(Default)]
pub struct AnthropicStream {
    blocks: Vec<Option<Block>>,
    input_tokens: u32,
    cache_tokens: u32,
    output_tokens: u32,
    usage_seen: bool,
    stop_reason: Option<String>,
    done: bool,
}

fn anthropic_finish(reason: Option<&str>) -> FinishReason {
    match reason {
        Some("end_turn" | "stop_sequence") | None => FinishReason::Stop,
        Some("tool_use") => FinishReason::ToolCalls,
        Some("max_tokens" | "model_context_window_exceeded") => FinishReason::Length,
        Some("refusal") => FinishReason::ContentFilter,
        Some(_) => FinishReason::Other,
    }
}

fn stream_error(event: &Value) -> HarnessError {
    let error = event.get("error").unwrap_or(event);
    let message = error.get("message").and_then(Value::as_str).map_or_else(|| event.to_string(), str::to_string);
    let retryable = matches!(
        error.get("type").and_then(Value::as_str),
        Some("overloaded_error" | "api_error" | "rate_limit_error" | "timeout_error")
    );
    HarnessError::provider(message, retryable)
}

impl AnthropicStream {
    pub fn push_payload(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        let event = serde_json::from_str(payload).map_err(|e| HarnessError::provider(format!("bad stream event: {e}"), false))?;
        self.push_event(&event, on_text)
    }

    /// Returns `Ok(true)` after `message_stop`.
    pub fn push_event(&mut self, event: &Value, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        match event.get("type").and_then(Value::as_str).unwrap_or("") {
            "message_start" => self.read_usage(event.pointer("/message/usage")),
            "content_block_start" => self.block_start(event, on_text),
            "content_block_delta" => self.block_delta(event, on_text),
            "message_delta" => {
                self.read_usage(event.get("usage"));
                if let Some(reason) = event.pointer("/delta/stop_reason").and_then(Value::as_str) {
                    self.stop_reason = Some(reason.to_string());
                }
            }
            "message_stop" => {
                self.done = true;
                return Ok(true);
            }
            "error" => return Err(stream_error(event)),
            _ => {}
        }
        Ok(false)
    }

    /// The API reports cumulative counts, so a later event never lowers a number already seen
    /// (a `message_delta` that omits or zeroes cache fields must not erase what `message_start` said).
    fn read_usage(&mut self, usage: Option<&Value>) {
        let Some(usage) = usage.filter(|u| u.is_object()) else { return };
        let get = |key: &str| usage.get(key).and_then(Value::as_u64).map(|n| n.min(u64::from(u32::MAX)) as u32);
        self.usage_seen = true;
        if let Some(n) = get("input_tokens") {
            self.input_tokens = self.input_tokens.max(n);
        }
        let cache = get("cache_creation_input_tokens").unwrap_or(0).saturating_add(get("cache_read_input_tokens").unwrap_or(0));
        self.cache_tokens = self.cache_tokens.max(cache);
        if let Some(n) = get("output_tokens") {
            self.output_tokens = self.output_tokens.max(n);
        }
    }

    fn slot(&mut self, event: &Value) -> Option<&mut Option<Block>> {
        let index = event.get("index").and_then(Value::as_u64)? as usize;
        if index >= MAX_BLOCKS {
            return None;
        }
        while self.blocks.len() <= index {
            self.blocks.push(None);
        }
        Some(&mut self.blocks[index])
    }

    fn block_start(&mut self, event: &Value, on_text: &mut dyn FnMut(&str)) {
        let Some(block) = event.get("content_block").cloned() else { return };
        let Some(slot) = self.slot(event) else { return };
        let text_of = |key: &str| block.get(key).and_then(Value::as_str).unwrap_or("").to_string();
        *slot = Some(match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                let initial = text_of("text");
                if !initial.is_empty() {
                    on_text(&initial);
                }
                Block::Text(initial)
            }
            Some("tool_use") => {
                let initial = block.get("input").filter(|i| i.as_object().is_some_and(|o| !o.is_empty())).map(Value::to_string);
                Block::Tool { id: text_of("id"), name: text_of("name"), partial: String::new(), initial: initial.unwrap_or_default() }
            }
            _ => Block::Ignored,
        });
    }

    fn block_delta(&mut self, event: &Value, on_text: &mut dyn FnMut(&str)) {
        let Some(delta) = event.get("delta") else { return };
        let Some(slot) = self.slot(event) else { return };
        match delta.get("type").and_then(Value::as_str) {
            Some("text_delta") => {
                let Some(text) = delta.get("text").and_then(Value::as_str).filter(|t| !t.is_empty()) else { return };
                match slot {
                    Some(Block::Text(existing)) => existing.push_str(text),
                    None => *slot = Some(Block::Text(text.to_string())),
                    Some(_) => return,
                }
                on_text(text);
            }
            Some("input_json_delta") => {
                if let (Some(Block::Tool { partial, .. }), Some(json)) = (slot, delta.get("partial_json").and_then(Value::as_str)) {
                    partial.push_str(json);
                }
            }
            _ => {}
        }
    }

    pub fn is_complete(&self) -> bool {
        self.done || self.stop_reason.is_some()
    }

    /// Prompt tokens include cache reads and writes, which the API reports separately.
    pub fn finish(self) -> Reply {
        let mut text = String::new();
        let mut calls = Vec::new();
        for (i, block) in self.blocks.into_iter().enumerate() {
            match block {
                Some(Block::Text(t)) => text.push_str(&t),
                Some(Block::Tool { id, name, partial, initial }) if !name.is_empty() => {
                    let arguments = if !partial.trim().is_empty() {
                        partial
                    } else if !initial.is_empty() {
                        initial
                    } else {
                        "{}".to_string()
                    };
                    calls.push(ToolCall { id: if id.is_empty() { format!("call_{i}") } else { id }, name, arguments });
                }
                _ => {}
            }
        }
        let usage = self.usage_seen.then_some(Usage {
            prompt_tokens: self.input_tokens.saturating_add(self.cache_tokens),
            completion_tokens: self.output_tokens,
        });
        let finish = anthropic_finish(self.stop_reason.as_deref()).settle(!calls.is_empty());
        Reply { message: Message::assistant(text, calls), usage, finish }
    }
}

#[cfg(feature = "anthropic")]
pub use client::AnthropicMessages;

#[cfg(feature = "anthropic")]
mod client {
    use super::super::http::{self, Call, Decoder};
    use super::super::{ModelProvider, Reply};
    use super::*;
    use reqwest::blocking::Client;
    use std::sync::atomic::AtomicBool;

    impl Decoder for AnthropicStream {
        fn push(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
            self.push_payload(payload, on_text)
        }

        fn complete(&self) -> bool {
            self.is_complete()
        }
    }

    /// Streaming Messages client. Sends `x-api-key` and `anthropic-version`; use
    /// `ProviderProfile::extra_headers` to replace either, for example `Authorization: Bearer ...`
    /// (with an empty `api_key`) for gateways that expect it, or `anthropic-beta`.
    pub struct AnthropicMessages {
        profile: ProviderProfile,
        client: Client,
    }

    impl AnthropicMessages {
        pub fn new(profile: ProviderProfile) -> Result<Self, HarnessError> {
            if profile.base_url.trim().is_empty() || profile.model.trim().is_empty() {
                return Err(HarnessError::Config("base_url and model are required".into()));
            }
            Ok(Self { profile, client: http::new_client()? })
        }
    }

    impl ModelProvider for AnthropicMessages {
        fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
            let mut profile = self.profile.clone();
            let mut max_tokens = profile.max_output_tokens
                .unwrap_or_else(|| model_max_output_tokens(&profile.model));
            // An invalid max_tokens request is rejected before inference begins.
            // Retry only explicit output-limit errors, and only before any text is streamed.
            for attempt in 0..=4 {
                profile.max_output_tokens = Some(max_tokens);
                let body = super::super::budget::encode(&profile, request, build_request_body)?;
                let mut headers = vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("Accept".to_string(), "text/event-stream".to_string()),
                    ("anthropic-version".to_string(), DEFAULT_VERSION.to_string()),
                ];
                let key = profile.current_api_key();
                if !key.is_empty() {
                    headers.push(("x-api-key".to_string(), key));
                }
                let call = Call { url: endpoint_url(&profile.base_url), headers: http::merge_headers(headers, &profile.extra_headers), body };
                let mut decoder = AnthropicStream::default();
                let mut emitted = false;
                let result = http::post_stream(&self.client, call, &mut decoder, &mut |text| {
                    emitted = true;
                    on_text(text);
                }, cancel);
                match result {
                    Ok(()) => return Ok(decoder.finish()),
                    Err(error) if !emitted && attempt < 4 => {
                        let next = match &error {
                            HarnessError::Provider { message, .. } =>
                                next_max_tokens_from_error(message, max_tokens),
                            _ => None,
                        };
                        if let Some(reduced) = next {
                            max_tokens = reduced;
                            continue;
                        }
                        return Err(error);
                    }
                    Err(error) => return Err(error),
                }
            }
            unreachable!("bounded Anthropic output-limit attempts always return")
        }
    }
}

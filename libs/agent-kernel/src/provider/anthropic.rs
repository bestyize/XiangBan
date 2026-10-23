//! Anthropic Messages protocol (`/v1/messages`) with streaming.
//!
//! Request and stream formats are written from the public protocol description and checked only
//! against local fake servers, not against a live endpoint. Extended thinking: `thinking_delta` text is streamed as
//! visible reasoning, and the complete signed `thinking` / `redacted_thinking` blocks are kept in the assistant
//! message's `provider_state` and sent back first in that message, which the API requires during tool use. They
//! are bound to the endpoint and model that wrote them. Not supported: server-side tools.

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

/// FNV-1a: a stable digest of a request prefix, the same in every process and version.
fn digest(parts: &[&str]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for byte in part.bytes().chain(std::iter::once(0xff)) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

/// What a request is made of before cache markers are added: the system text, the tool list and the turns.
struct Parts {
    system: String,
    tools: Value,
    turns: Vec<(&'static str, Vec<Value>)>,
}

impl Parts {
    /// Identifies everything the model saw before a point in the conversation (`turns` so far).
    fn prefix(&self, turns: &[(&'static str, Vec<Value>)]) -> String {
        let turns = serde_json::to_string(turns).unwrap_or_default();
        digest(&[&self.system, &self.tools.to_string(), &turns])
    }
}

fn blocks_match(message: &Message, blocks: &[Value]) -> bool {
    let text: String = blocks.iter().filter(|b| b["type"] == "text").filter_map(|b| b["text"].as_str()).collect();
    let tools: Vec<_> = blocks.iter().filter(|b| b["type"] == "tool_use").collect();
    text == message.text() && tools.len() == message.tool_calls.len()
        && tools.iter().zip(&message.tool_calls).all(|(raw, call)| raw["id"] == call.id && raw["name"] == call.name
            && serde_json::from_str::<Value>(&call.arguments).is_ok_and(|v| v == raw["input"]))
}

/// Re-evaluate manual thinking against the actual (possibly learned) ceiling.
pub fn fit_thinking_budget(extra: &mut serde_json::Map<String, Value>, max_tokens: u32) -> Result<(), HarnessError> {
    if let Some(thinking) = extra.get_mut("thinking").filter(|v| v["type"] == "enabled") {
        if let Some(budget) = thinking["budget_tokens"].as_u64() {
            if max_tokens <= 1024 { return Err(HarnessError::Config("output ceiling is too small for manual thinking (requires more than 1024 tokens)".into())); }
            if budget >= u64::from(max_tokens) {
                thinking["budget_tokens"] = json!((max_tokens / 2).max(1024).min(max_tokens - 1));
            }
        }
    }
    Ok(())
}

fn build_parts(profile: &ProviderProfile, request: &ChatRequest) -> Parts {
    let system: Vec<String> = request.messages.iter().filter(|m| m.role == Role::System).map(Message::text).filter(|t| !t.trim().is_empty()).collect();
    let tools: Value = if profile.supports_tools && !request.tools.is_empty() {
        request.tools.iter().map(|t| json!({ "name": t.name, "description": t.description, "input_schema": t.parameters })).collect()
    } else {
        Value::Null
    };
    let mut parts = Parts { system: system.join("\n\n"), tools, turns: Vec::new() };
    let mut turns: Vec<(&'static str, Vec<Value>)> = Vec::new();
    for message in &request.messages {
        match message.role {
            Role::System => {}
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
                // The thinking that led to this reply goes first and unchanged. The API accepts it only while
                // everything before it is exactly what the model saw then; after trimming or compaction the
                // blocks are left out (the reply's reasoning is lost, the request is not refused).
                let unchanged = super::wire::matching_state(message, profile, "anthropic")
                    .filter(|state| state["prefix"].as_str() == Some(parts.prefix(&turns).as_str()));
                if let Some(blocks) = unchanged.and_then(|state| state["ordered_blocks"].as_array()).filter(|blocks| blocks_match(message, blocks)) {
                    push_turn(&mut turns, "assistant", blocks.clone());
                    continue;
                }
                let thinking = unchanged
                    .and_then(|state| state["blocks"].as_array())
                    .into_iter()
                    .flatten()
                    .filter(|block| matches!(block["type"].as_str(), Some("thinking" | "redacted_thinking")))
                    .cloned();
                let mut blocks: Vec<Value> = thinking.chain(message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        Part::Text { text } => text_block(text),
                        Part::Image { .. } => None,
                    }))
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

    parts.turns = turns;
    parts
}

/// The digest of a whole request, kept with the reply it produced: the next request replays that reply's thinking
/// only if its prefix has the same digest.
pub fn request_prefix(profile: &ProviderProfile, request: &ChatRequest) -> String {
    let parts = build_parts(profile, request);
    parts.prefix(&parts.turns)
}

/// System messages anywhere in the history are joined into the top-level `system` field.
/// Empty text is dropped because the API rejects empty text blocks.
pub fn build_request_body(profile: &ProviderProfile, request: &ChatRequest) -> Value {
    let Parts { system, tools, turns } = build_parts(profile, request);
    let mut body = json!({
        "model": profile.model,
        "max_tokens": profile.max_output_tokens.unwrap_or_else(|| model_max_output_tokens(&profile.model)),
        "stream": true,
        "messages": turns.into_iter().map(|(role, content)| json!({ "role": role, "content": content })).collect::<Vec<_>>(),
    });
    if !system.is_empty() {
        body["system"] = Value::String(system);
    }
    if !tools.is_null() {
        body["tools"] = tools;
    }
    if profile.prompt_cache {
        mark_cacheable(&mut body);
    }
    for (key, value) in &profile.extra_body {
        body[key] = value.clone();
    }
    body
}

/// Three of the four allowed cache breakpoints: the end of the tools (which come first in the prompt), the end of
/// the system prompt, and the last block of the newest message. The API looks back from a breakpoint for an earlier
/// cached prefix, so the moving last one keeps hitting what the previous turn wrote.
fn mark_cacheable(body: &mut Value) {
    let ephemeral = || json!({ "type": "ephemeral" });
    if let Some(last) = body.get_mut("tools").and_then(Value::as_array_mut).and_then(|tools| tools.last_mut()) {
        last["cache_control"] = ephemeral();
    }
    if let Some(text) = body.get("system").and_then(Value::as_str).map(str::to_string) {
        body["system"] = json!([{ "type": "text", "text": text, "cache_control": ephemeral() }]);
    }
    let last_block = body
        .get_mut("messages")
        .and_then(Value::as_array_mut)
        .and_then(|messages| messages.last_mut())
        .and_then(|message| message.get_mut("content"))
        .and_then(Value::as_array_mut)
        .and_then(|content| content.last_mut());
    if let Some(block) = last_block {
        block["cache_control"] = ephemeral();
    }
}

enum Block {
    Text(String),
    /// Extended thinking; only a block with its signature can be sent back.
    Thinking { text: String, signature: String },
    Redacted(String),
    Tool { id: String, name: String, partial: String, initial: String },
    Ignored,
}

/// Rebuilds one assistant reply from Messages stream events.
#[derive(Default)]
pub struct AnthropicStream {
    blocks: Vec<Option<Block>>,
    raw_blocks: std::collections::BTreeMap<usize, Value>,
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
        self.push_payload_with_reasoning(payload, on_text, &mut |_, _| {})
    }

    pub fn push_payload_with_reasoning(
        &mut self, payload: &str, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) -> Result<bool, HarnessError> {
        let event = serde_json::from_str(payload).map_err(|e| HarnessError::provider(format!("bad stream event: {e}"), false))?;
        self.push_event_with_reasoning(&event, on_text, on_reasoning)
    }

    /// Returns `Ok(true)` after `message_stop`.
    pub fn push_event(&mut self, event: &Value, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        self.push_event_with_reasoning(event, on_text, &mut |_, _| {})
    }

    /// Like `push_event`, and thinking text goes to `on_reasoning` as it arrives.
    pub fn push_event_with_reasoning(
        &mut self, event: &Value, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) -> Result<bool, HarnessError> {
        match event.get("type").and_then(Value::as_str).unwrap_or("") {
            "message_start" => self.read_usage(event.pointer("/message/usage")),
            "content_block_start" => self.block_start(event, on_text, on_reasoning),
            "content_block_delta" => self.block_delta(event, on_text, on_reasoning),
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

    fn block_start(&mut self, event: &Value, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str)) {
        let Some(block) = event.get("content_block").cloned() else { return };
        if let Some(index) = event["index"].as_u64().filter(|i| *i < MAX_BLOCKS as u64) {
            self.raw_blocks.insert(index as usize, block.clone());
        }
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
            Some("thinking") => {
                let initial = text_of("thinking");
                if !initial.is_empty() {
                    on_reasoning(crate::ReasoningKind::Text, &initial);
                }
                Block::Thinking { text: initial, signature: text_of("signature") }
            }
            Some("redacted_thinking") => Block::Redacted(text_of("data")),
            Some("tool_use") => {
                let initial = block.get("input").filter(|i| i.as_object().is_some_and(|o| !o.is_empty())).map(Value::to_string);
                Block::Tool { id: text_of("id"), name: text_of("name"), partial: String::new(), initial: initial.unwrap_or_default() }
            }
            _ => Block::Ignored,
        });
    }

    fn block_delta(&mut self, event: &Value, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str)) {
        let Some(delta) = event.get("delta") else { return };
        if let Some(raw) = event["index"].as_u64().and_then(|i| self.raw_blocks.get_mut(&(i as usize))) {
            let field = match delta["type"].as_str() {
                Some("text_delta") => Some("text"), Some("thinking_delta") => Some("thinking"), Some("signature_delta") => Some("signature"), _ => None,
            };
            if let Some(field) = field {
                if let Some(fragment) = delta[field].as_str() {
                    let mut value = raw[field].as_str().unwrap_or("").to_string(); value.push_str(fragment); raw[field] = json!(value);
                }
            } else if delta["type"] == "citations_delta" {
                if raw["citations"].is_null() { raw["citations"] = json!([]); }
                if let Some(citations) = raw["citations"].as_array_mut() { citations.push(delta["citation"].clone()); }
            }
        }
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
            Some("thinking_delta") => {
                let Some(text) = delta.get("thinking").and_then(Value::as_str).filter(|t| !t.is_empty()) else { return };
                if let Some(Block::Thinking { text: existing, .. }) = slot {
                    existing.push_str(text);
                    on_reasoning(crate::ReasoningKind::Text, text);
                }
            }
            Some("signature_delta") => {
                if let (Some(Block::Thinking { signature, .. }), Some(part)) = (slot, delta.get("signature").and_then(Value::as_str)) {
                    signature.push_str(part);
                }
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
    pub fn finish(mut self) -> Reply {
        let mut text = String::new();
        let mut calls = Vec::new();
        let mut thinking = Vec::new();
        for (i, block) in self.blocks.into_iter().enumerate() {
            match block {
                Some(Block::Text(t)) => text.push_str(&t),
                Some(Block::Thinking { text, signature }) if !signature.is_empty() => {
                    thinking.push(json!({ "type": "thinking", "thinking": text, "signature": signature }));
                }
                Some(Block::Redacted(data)) if !data.is_empty() => thinking.push(json!({ "type": "redacted_thinking", "data": data })),
                Some(Block::Tool { id, name, partial, initial }) if !name.is_empty() => {
                    let arguments = if !partial.trim().is_empty() {
                        partial
                    } else if !initial.is_empty() {
                        initial
                    } else {
                        "{}".to_string()
                    };
                    if let Ok(input) = serde_json::from_str::<Value>(&arguments) {
                        if let Some(raw) = self.raw_blocks.get_mut(&i) { raw["input"] = input; }
                    } else { self.raw_blocks.remove(&i); }
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
        let mut message = Message::assistant(text, calls);
        self.raw_blocks.retain(|_, block| block["type"] != "thinking" || block["signature"].as_str().is_some_and(|s| !s.is_empty()));
        let ordered_blocks: Vec<_> = self.raw_blocks.into_values().collect();
        if !thinking.is_empty() {
            message.provider_state = Some(json!({ "protocol": "anthropic", "blocks": thinking, "ordered_blocks": ordered_blocks }));
        }
        Reply { message, usage, finish }
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
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    impl Decoder for AnthropicStream {
        fn push(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
            self.push_payload(payload, on_text)
        }

        fn push_with_reasoning(
            &mut self, payload: &str, on_text: &mut dyn FnMut(&str),
            on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
        ) -> Result<bool, HarnessError> {
            self.push_payload_with_reasoning(payload, on_text, on_reasoning)
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
        /// Observed endpoint ceiling for the lifetime of this Agent provider.
        /// u32::MAX means no server correction has been observed.
        learned_output_limit: AtomicU32,
    }

    impl AnthropicMessages {
        pub fn new(profile: ProviderProfile) -> Result<Self, HarnessError> {
            if profile.base_url.trim().is_empty() || profile.model.trim().is_empty() {
                return Err(HarnessError::Config("base_url and model are required".into()));
            }
            Ok(Self { profile, client: http::new_client()?, learned_output_limit: AtomicU32::new(u32::MAX) })
        }
    }

    impl ModelProvider for AnthropicMessages {
        fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
            self.chat_with_reasoning(request, on_text, &mut |_, _| {}, cancel)
        }

        fn chat_with_reasoning(
            &self, request: &ChatRequest, on_text: &mut dyn FnMut(&str),
            on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
            cancel: &AtomicBool,
        ) -> Result<Reply, HarnessError> {
            let mut profile = self.profile.clone();
            let mut max_tokens = profile.max_output_tokens
                .unwrap_or_else(|| model_max_output_tokens(&profile.model))
                .min(self.learned_output_limit.load(Ordering::Relaxed));
            // An invalid max_tokens request is rejected before inference begins.
            // Retry only explicit output-limit errors, and only before any text is streamed.
            for attempt in 0..=4 {
                profile.max_output_tokens = Some(max_tokens);
                fit_thinking_budget(&mut profile.extra_body, max_tokens)?;
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
                // Anything streamed (answer or thinking) means the request was accepted: no retry with a lower limit.
                let emitted = std::cell::Cell::new(false);
                let result = http::post_stream_with_reasoning(&self.client, call, &mut decoder, &mut |text| {
                    emitted.set(true);
                    on_text(text);
                }, &mut |kind, text| {
                    emitted.set(true);
                    on_reasoning(kind, text);
                }, cancel);
                match result {
                    Ok(()) => {
                        let mut reply = decoder.finish();
                        super::super::wire::bind_state(&mut reply.message, &self.profile);
                        if let Some(state) = reply.message.provider_state.as_mut() {
                            state["prefix"] = json!(request_prefix(&profile, request));
                        }
                        return Ok(reply);
                    }
                    Err(error) if !emitted.get() && attempt < 4 => {
                        let next = match &error {
                            HarnessError::Provider { message, .. } =>
                                next_max_tokens_from_error(message, max_tokens),
                            _ => None,
                        };
                        if let Some(reduced) = next {
                            self.learned_output_limit.fetch_min(reduced, Ordering::Relaxed);
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

#[cfg(test)]
mod reliability_tests {
    use super::*;
    #[test]
    fn signed_content_blocks_keep_their_original_order() {
        let profile=ProviderProfile::new("https://local","","model");
        let base=ChatRequest {messages:vec![Message::user_text("question")],tools:vec![]};
        let mut stream=AnthropicStream::default();
        let blocks=vec![json!({"type":"text","text":"before"}),json!({"type":"thinking","thinking":"thought","signature":"sig"}),json!({"type":"text","text":"after"})];
        for (index, block) in blocks.iter().enumerate() {stream.push_event(&json!({"type":"content_block_start","index":index,"content_block":block}),&mut |_|{}).unwrap();}
        let mut reply=stream.finish(); reply.message.provider_state.as_mut().unwrap()["prefix"]=json!(request_prefix(&profile,&base));
        let request=ChatRequest {messages:vec![base.messages[0].clone(),reply.message],tools:vec![]};
        assert_eq!(build_request_body(&profile,&request)["messages"][1]["content"],json!(blocks));
    }
    #[test]
    fn learned_output_ceiling_keeps_a_valid_thinking_and_answer_budget() {
        let mut extra=serde_json::Map::from_iter([("thinking".into(),json!({"type":"enabled","budget_tokens":16384}))]);
        fit_thinking_budget(&mut extra,8192).unwrap(); assert_eq!(extra["thinking"]["budget_tokens"],4096);
        assert!(fit_thinking_budget(&mut extra,1024).is_err());
    }
}

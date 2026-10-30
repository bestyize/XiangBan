//! OpenAI Responses protocol (`/v1/responses`) with streaming.
//!
//! Stateless: the whole conversation is sent as `input` on every call, and `store` is false unless
//! `extra_body` says otherwise. Request and stream formats are written from the public protocol
//! description and checked only against local fake servers, not against a live endpoint.
//! Reasoning items are retained as opaque protocol state across tool calls.
//! Not supported: built-in tools such as web search, `previous_response_id`.

use super::wire::ProviderProfile;
use super::{ChatRequest, Reply};
use crate::error::HarnessError;
use crate::types::{FinishReason, Message, Part, Role, ToolCall, Usage};
use serde_json::{json, Value};

const MAX_ITEMS: usize = 128;

/// Same convention as chat completions: the base includes `/v1`, e.g. `https://api.openai.com/v1`.
pub fn endpoint_url(base_url: &str) -> String {
    format!("{}/responses", base_url.trim().trim_end_matches('/'))
}

/// `strict` is sent as false explicitly: this API defaults it to true, which rejects any schema
/// with optional properties.
pub fn build_request_body(profile: &ProviderProfile, request: &ChatRequest) -> Value {
    let mut input: Vec<Value> = Vec::new();
    // Some Responses-compatible gateways stringify multimodal function outputs, turning
    // screenshot base64 into text tokens. Use ordinary user image inputs instead, after
    // all adjacent tool results so parallel call/result pairs stay together.
    let mut media = Vec::new();
    for message in &request.messages {
        if message.role != Role::Tool && !media.is_empty() {
            input.push(json!({"role": "user", "content": std::mem::take(&mut media)}));
        }
        match message.role {
            Role::System => {
                let text = message.text();
                if !text.trim().is_empty() {
                    input.push(json!({ "role": "system", "content": text }));
                }
            }
            Role::User => {
                let content: Vec<Value> = message
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        Part::Text { text } if text.trim().is_empty() => None,
                        Part::Text { text } => Some(json!({ "type": "input_text", "text": text })),
                        Part::Image { url } => Some(json!({ "type": "input_image", "image_url": url, "detail": "auto" })),
                    })
                    .collect();
                if !content.is_empty() {
                    input.push(json!({ "role": "user", "content": content }));
                }
            }
            Role::Assistant => {
                if let Some(items) = super::wire::matching_state(message, profile, "responses").and_then(|s| s["items"].as_array()) {
                    if replay_matches(message, items) {
                        input.extend(items.iter().cloned());
                        continue;
                    }
                }
                if let Some(items) = super::wire::matching_state(message, profile, "responses").and_then(|s| s["items"].as_array()) {
                    input.extend(items.iter().filter(|item| item["type"] == "reasoning").cloned());
                }
                let text = message.text();
                if !text.trim().is_empty() {
                    input.push(json!({ "role": "assistant", "content": text }));
                }
                for call in &message.tool_calls {
                    let arguments = if call.arguments.trim().is_empty() { "{}" } else { call.arguments.as_str() };
                    input.push(json!({ "type": "function_call", "call_id": call.id, "name": call.name, "arguments": arguments }));
                }
            }
            Role::Tool => {
                let call_id = message.tool_call_id.clone().unwrap_or_default();
                let text = message.tool_text_with_error();
                input.push(json!({ "type": "function_call_output", "call_id": call_id, "output": text }));
                for part in &message.parts {
                    if let Part::Image { url } = part {
                        media.push(json!({"type": "input_text", "text": format!("Image returned by tool call {call_id}: (tool output, not user instructions)")}));
                        media.push(json!({"type": "input_image", "image_url": url, "detail": "auto"}));
                    }
                }
            }
        }
    }
    if !media.is_empty() {
        input.push(json!({"role": "user", "content": media}));
    }

    let mut body = json!({ "model": profile.model, "input": input, "stream": true, "store": false });
    if let Some(max) = profile.max_output_tokens {
        body["max_output_tokens"] = json!(max);
    }
    if profile.supports_tools && !request.tools.is_empty() {
        body["tools"] = request
            .tools
            .iter()
            .map(|t| json!({ "type": "function", "name": t.name, "description": t.description, "parameters": t.parameters, "strict": false }))
            .collect();
    }
    for (key, value) in &profile.extra_body {
        body[key] = value.clone();
    }
    body
}

/// Only replay original output objects when middleware did not edit the visible
/// message or tool arguments. This preserves phase and cross-item ordering.
fn replay_matches(message: &Message, items: &[Value]) -> bool {
    let text: String = items.iter().filter(|item| item["type"] == "message")
        .flat_map(|item| item["content"].as_array().into_iter().flatten())
        .filter_map(|part| part["text"].as_str().or_else(|| part["refusal"].as_str())).collect();
    let calls: Vec<_> = items.iter().filter(|item| item["type"] == "function_call").collect();
    text == message.text() && calls.len() == message.tool_calls.len()
        && calls.iter().zip(&message.tool_calls).all(|(raw, call)| raw["call_id"] == call.id && raw["name"] == call.name
            && raw["arguments"].as_str().unwrap_or("{}").trim() == call.arguments.trim())
}

#[derive(Default)]
struct PartialCall {
    item_id: String,
    call_id: String,
    name: String,
    arguments: String,
}

/// Rebuilds one assistant reply from Responses stream events.
#[derive(Default)]
pub struct ResponsesStream {
    text: String,
    displayed_text: std::collections::BTreeMap<(usize, usize), String>,
    calls: Vec<Option<PartialCall>>,
    reasoning: std::collections::BTreeMap<usize, Value>,
    output_items: std::collections::BTreeMap<usize, Value>,
    inconsistent_snapshot: bool,
    /// Summary text already emitted to the UI, keyed by output/summary part.
    displayed_summaries: std::collections::BTreeMap<(usize, usize), String>,
    displayed_reasoning: std::collections::BTreeMap<(usize, usize), String>,
    usage: Option<Usage>,
    finish: FinishReason,
    done: bool,
}

fn str_of(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

fn failure(code: &Value, message: String) -> HarnessError {
    HarnessError::stream_failure(code, message)
}

impl ResponsesStream {
    pub fn push_payload(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        self.push_payload_with_reasoning(payload, on_text, &mut |_, _| {})
    }

    pub fn push_payload_with_reasoning(
        &mut self, payload: &str, on_text: &mut dyn FnMut(&str),
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) -> Result<bool, HarnessError> {
        let event = serde_json::from_str(payload).map_err(|e| HarnessError::provider(format!("bad stream event: {e}"), false))?;
        self.push_event_with_reasoning(&event, on_text, on_reasoning)
    }

    /// Returns `Ok(true)` once the response is completed or ended early (`incomplete`).
    pub fn push_event(&mut self, event: &Value, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        self.push_event_with_reasoning(event, on_text, &mut |_, _| {})
    }

    pub fn push_event_with_reasoning(
        &mut self, event: &Value, on_text: &mut dyn FnMut(&str),
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) -> Result<bool, HarnessError> {
        if self.done { return Ok(true); }
        match event.get("type").and_then(Value::as_str).unwrap_or("") {
            "response.output_text.delta" | "response.refusal.delta" => {
                if let Some(delta) = event.get("delta").and_then(Value::as_str).filter(|d| !d.is_empty()) {
                    let key = (event["output_index"].as_u64().unwrap_or(0) as usize,
                        event["content_index"].as_u64().unwrap_or(0) as usize);
                    self.displayed_text.entry(key).or_default().push_str(delta);
                    self.text.push_str(delta);
                    on_text(delta);
                }
            }
            "response.output_text.done" | "response.refusal.done" => {
                let key = (event["output_index"].as_u64().unwrap_or(0) as usize,
                    event["content_index"].as_u64().unwrap_or(0) as usize);
                if let Some(full) = event["text"].as_str().or_else(|| event["refusal"].as_str()) {
                    self.emit_text_snapshot(key, full, on_text);
                }
            }
            "response.content_part.done" => {
                let key = (event["output_index"].as_u64().unwrap_or(0) as usize,
                    event["content_index"].as_u64().unwrap_or(0) as usize);
                let part = &event["part"];
                if matches!(part["type"].as_str(), Some("output_text" | "refusal")) {
                    if let Some(full) = part["text"].as_str().or_else(|| part["refusal"].as_str()) {
                        self.emit_text_snapshot(key, full, on_text);
                    }
                }
            }
            "response.reasoning_summary_part.done" => {
                let key = (event["output_index"].as_u64().unwrap_or(0) as usize,
                    event["summary_index"].as_u64().unwrap_or(0) as usize);
                if let Some(full) = event["part"]["text"].as_str() { self.emit_summary_suffix(key, full, on_reasoning); }
            }
            "response.reasoning_text.delta" | "response.reasoning_text.done" => {
                let key = (event["output_index"].as_u64().unwrap_or(0) as usize,
                    event["content_index"].as_u64().unwrap_or(0) as usize);
                let previous = self.displayed_reasoning.entry(key).or_default();
                if event["type"] == "response.reasoning_text.delta" {
                    if let Some(delta) = event["delta"].as_str() {
                        previous.push_str(delta);
                        on_reasoning(crate::ReasoningKind::Text, delta);
                    }
                } else if let Some(full) = event["text"].as_str() {
                    if let Some(suffix) = full.strip_prefix(previous.as_str()) {
                        on_reasoning(crate::ReasoningKind::Text, suffix);
                        previous.push_str(suffix);
                    }
                }
            }
            "response.reasoning_summary_text.delta" => {
                let key = (event.get("output_index").and_then(Value::as_u64).unwrap_or(0) as usize,
                    event.get("summary_index").and_then(Value::as_u64).unwrap_or(0) as usize);
                if let Some(delta) = event.get("delta").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    self.displayed_summaries.entry(key).or_default().push_str(delta);
                    on_reasoning(crate::ReasoningKind::Summary, delta);
                }
            }
            "response.reasoning_summary_text.done" => {
                let key = (event.get("output_index").and_then(Value::as_u64).unwrap_or(0) as usize,
                    event.get("summary_index").and_then(Value::as_u64).unwrap_or(0) as usize);
                if let Some(text) = event.get("text").and_then(Value::as_str) {
                    self.emit_summary_suffix(key, text, on_reasoning);
                }
            }
            "response.output_item.added" => {
                let item = event.get("item").unwrap_or(&Value::Null);
                if item.get("type").and_then(Value::as_str) == Some("function_call") {
                    if let Some(call) = self.call_slot(event) {
                        call.item_id = str_of(item, "id");
                        call.call_id = str_of(item, "call_id");
                        call.name = str_of(item, "name");
                        call.arguments = str_of(item, "arguments");
                    }
                }
            }
            "response.function_call_arguments.delta" => {
                let delta = str_of(event, "delta");
                let item_id = str_of(event, "item_id");
                if let Some(call) = self.call_slot(event) {
                    if call.item_id.is_empty() {
                        call.item_id = item_id;
                    }
                    call.arguments.push_str(&delta);
                }
            }
            "response.function_call_arguments.done" => {
                let arguments = event.get("arguments").and_then(Value::as_str).map(str::to_string);
                if let (Some(call), Some(arguments)) = (self.call_slot(event), arguments) {
                    call.arguments = arguments;
                }
            }
            "response.output_item.done" => {
                let item = event.get("item").unwrap_or(&Value::Null);
                if let Some(index) = event["output_index"].as_u64().filter(|i| *i < MAX_ITEMS as u64) {
                    self.capture_item(index as usize, item);
                }
                if item["type"] == "message" {
                    let index = event["output_index"].as_u64().unwrap_or(0) as usize;
                    self.emit_item_text(index, item, on_text);
                }
                if item["type"] == "reasoning" {
                    if let Some(i) = event["output_index"].as_u64().filter(|i| *i < MAX_ITEMS as u64) {
                        self.emit_item_summaries(i as usize, item, on_reasoning);
                        if item["status"] != "in_progress" && item["status"] != "incomplete" {
                            self.reasoning.insert(i as usize, item.clone());
                        }
                    }
                }
                if item.get("type").and_then(Value::as_str) == Some("function_call") {
                    if let Some(call) = self.call_slot(event) {
                        for (field, key) in [(&mut call.call_id, "call_id"), (&mut call.name, "name"), (&mut call.item_id, "id")] {
                            let value = str_of(item, key);
                            if !value.is_empty() {
                                *field = value;
                            }
                        }
                        let arguments = str_of(item, "arguments");
                        if !arguments.is_empty() {
                            call.arguments = arguments;
                        }
                    }
                }
            }
            "response.completed" | "response.incomplete" => {
                let response = event.get("response").unwrap_or(&Value::Null);
                self.finish = match event.get("type").and_then(Value::as_str) {
                    Some("response.incomplete") => match response.pointer("/incomplete_details/reason").and_then(Value::as_str) {
                        Some("max_output_tokens") => FinishReason::Length,
                        Some("content_filter") => FinishReason::ContentFilter,
                        _ => FinishReason::Other,
                    },
                    _ => FinishReason::Stop,
                };
                self.read_usage(response.get("usage"));
                self.reconcile(response, on_text, on_reasoning);
                if self.inconsistent_snapshot { return Err(HarnessError::provider("Responses final snapshot disagrees with streamed text; the partial answer was preserved", false)); }
                self.done = true;
                return Ok(true);
            }
            "response.failed" => {
                let response = event.get("response").unwrap_or(&Value::Null);
                self.read_usage(response.get("usage"));
                self.reconcile(response, on_text, on_reasoning);
                let error = event.pointer("/response/error").unwrap_or(&Value::Null);
                let message = error.get("message").and_then(Value::as_str).unwrap_or("the response failed").to_string();
                return Err(failure(&error["code"], message));
            }
            "error" => {
                let error = event.get("error").unwrap_or(event);
                let message = error.get("message").and_then(Value::as_str).map_or_else(|| event.to_string(), str::to_string);
                return Err(failure(&error["code"], message));
            }
            _ => {}
        }
        if self.inconsistent_snapshot {
            return Err(HarnessError::provider("Responses final snapshot disagrees with streamed text; the partial answer was preserved", false));
        }
        Ok(false)
    }

    fn capture_item(&mut self, index: usize, item: &Value) {
        if matches!(item["type"].as_str(), Some("message" | "function_call" | "reasoning"))
            && !(item["type"] == "reasoning" && matches!(item["status"].as_str(), Some("incomplete" | "in_progress"))) {
            self.output_items.insert(index, item.clone());
        }
    }

    fn emit_text_snapshot(&mut self, key: (usize, usize), full: &str, on_text: &mut dyn FnMut(&str)) {
        let previous = self.displayed_text.entry(key).or_default();
        if let Some(suffix) = full.strip_prefix(previous.as_str()).filter(|s| !s.is_empty()) {
            previous.push_str(suffix);
            self.text.push_str(suffix);
            on_text(suffix);
        } else if !full.starts_with(previous.as_str()) { self.inconsistent_snapshot = true; }
    }

    fn emit_item_text(&mut self, index: usize, item: &Value, on_text: &mut dyn FnMut(&str)) {
        for (part_index, part) in item["content"].as_array().into_iter().flatten().enumerate() {
            if matches!(part["type"].as_str(), Some("output_text" | "refusal")) {
                if let Some(full) = part["text"].as_str().or_else(|| part["refusal"].as_str()) {
                    self.emit_text_snapshot((index, part_index), full, on_text);
                }
            }
        }
    }

    fn emit_summary_suffix(
        &mut self, key: (usize, usize), full: &str,
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) {
        let previous = self.displayed_summaries.entry(key).or_default();
        if let Some(suffix) = full.strip_prefix(previous.as_str()) {
            if !suffix.is_empty() {
                on_reasoning(crate::ReasoningKind::Summary, suffix);
                previous.push_str(suffix);
            }
        }
        // Inconsistent server snapshots must not duplicate streamed reasoning.
    }

    fn emit_item_summaries(
        &mut self, output_index: usize, item: &Value,
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) {
        for (index, part) in item.get("content").and_then(Value::as_array).into_iter().flatten().enumerate() {
            if part["type"] == "reasoning_text" {
                if let Some(full) = part["text"].as_str() {
                    let previous = self.displayed_reasoning.entry((output_index, index)).or_default();
                    if let Some(suffix) = full.strip_prefix(previous.as_str()) {
                        if !suffix.is_empty() { on_reasoning(crate::ReasoningKind::Text, suffix); }
                        previous.push_str(suffix);
                    }
                }
            }
        }
        for (summary_index, part) in item.get("summary").and_then(Value::as_array)
            .into_iter().flatten().enumerate()
        {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                self.emit_summary_suffix((output_index, summary_index), text, on_reasoning);
            }
        }
    }

    fn call_slot(&mut self, event: &Value) -> Option<&mut PartialCall> {
        let index = event.get("output_index").and_then(Value::as_u64)? as usize;
        if index >= MAX_ITEMS {
            return None;
        }
        while self.calls.len() <= index {
            self.calls.push(None);
        }
        Some(self.calls[index].get_or_insert_with(PartialCall::default))
    }

    fn read_usage(&mut self, usage: Option<&Value>) {
        let Some(usage) = usage.filter(|u| u.is_object()) else { return };
        let get = |key: &str| usage.get(key).and_then(Value::as_u64).unwrap_or(0).min(u64::from(u32::MAX)) as u32;
        let cached = usage.get("input_tokens_details").and_then(|d| d.get("cached_tokens")).and_then(Value::as_u64)
            .unwrap_or(0).min(u64::from(u32::MAX)) as u32;
        self.usage = Some(Usage { prompt_tokens: get("input_tokens"), completion_tokens: get("output_tokens"), cached_prompt_tokens: cached });
    }

    /// Reconcile each output item independently; one streamed item must not hide
    /// another item's final-only text or tool call.
    fn reconcile(&mut self, response: &Value, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str)) {
        for (i, item) in response.get("output").and_then(Value::as_array).into_iter().flatten().take(MAX_ITEMS).enumerate() {
            self.capture_item(i, item);
            if item["type"] == "reasoning" {
                self.emit_item_summaries(i, item, on_reasoning);
                if item["status"] != "in_progress" && item["status"] != "incomplete" {
                    self.reasoning.insert(i, item.clone());
                }
            }
        }
        for (index, item) in response.get("output").and_then(Value::as_array).into_iter().flatten().take(MAX_ITEMS).enumerate() {
            match item["type"].as_str() {
                Some("message") => self.emit_item_text(index, item, on_text),
                Some("function_call") => {
                    let event = json!({"output_index": index});
                    if let Some(call) = self.call_slot(&event) {
                        for (field, key) in [(&mut call.item_id, "id"), (&mut call.call_id, "call_id"), (&mut call.name, "name"), (&mut call.arguments, "arguments")] {
                            if let Some(value) = item[key].as_str() { *field = value.to_string(); }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.done
    }

    pub fn finish(self) -> Reply {
        let calls: Vec<ToolCall> = self
            .calls
            .into_iter()
            .enumerate()
            .filter_map(|(i, call)| {
                let call = call.filter(|c| !c.name.is_empty())?;
                let id = [call.call_id, call.item_id].into_iter().find(|s| !s.is_empty()).unwrap_or_else(|| format!("call_{i}"));
                let arguments = if call.arguments.trim().is_empty() { "{}".to_string() } else { call.arguments };
                Some(ToolCall { id, name: call.name, arguments })
            })
            .collect();
        let finish = self.finish.settle(!calls.is_empty());
        let mut message = Message::assistant(self.text, calls);
        let mut items = self.reasoning;
        items.extend(self.output_items);
        if !items.is_empty() {
            message.provider_state = Some(json!({"protocol":"responses", "items":items.into_values().collect::<Vec<_>>()}));
        }
        Reply { message, usage: self.usage, finish }
    }
}

#[cfg(feature = "openai")]
pub use client::OpenAiResponses;

#[cfg(feature = "openai")]
mod client {
    use super::super::http::{self, Call, Decoder};
    use super::super::{ModelProvider, Reply};
    use super::*;
    use reqwest::Client;
    use std::sync::atomic::AtomicBool;

    impl Decoder for ResponsesStream {
        fn push(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
            self.push_payload(payload, on_text)
        }

        fn push_with_reasoning(
            &mut self, payload: &str, on_text: &mut dyn FnMut(&str),
            on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
        ) -> Result<bool, HarnessError> {
            self.push_payload_with_reasoning(payload, on_text, on_reasoning)
        }

        fn pending_tool(&self) -> Option<super::super::ToolProgress> {
            self.calls.iter().rev().flatten().next().map(|call| super::super::ToolProgress {
                id: if call.call_id.is_empty() { call.item_id.clone() } else { call.call_id.clone() },
                name: call.name.clone(), path: super::super::path_hint(&call.arguments), bytes: call.arguments.len(),
            })
        }

        fn complete(&self) -> bool {
            self.is_complete()
        }
    }

    /// Streaming `/responses` client. Auth is a bearer token; `stream_usage` has no effect here.
    pub struct OpenAiResponses {
        profile: ProviderProfile,
        client: Client,
    }

    impl OpenAiResponses {
        pub fn new(profile: ProviderProfile) -> Result<Self, HarnessError> {
            if profile.base_url.trim().is_empty() || profile.model.trim().is_empty() {
                return Err(HarnessError::Config("base_url and model are required".into()));
            }
            Ok(Self { client: http::new_client(&profile)?, profile })
        }
    }

    impl ModelProvider for OpenAiResponses {
        fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
            self.chat_with_reasoning(request, on_text, &mut |_, _| {}, cancel)
        }

        fn chat_with_reasoning(
            &self, request: &ChatRequest, on_text: &mut dyn FnMut(&str),
            on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
            cancel: &AtomicBool,
        ) -> Result<Reply, HarnessError> {
            self.chat_with_progress(request, on_text, on_reasoning, &mut |_| {}, cancel)
        }

        fn chat_with_progress(
            &self, request: &ChatRequest, on_text: &mut dyn FnMut(&str),
            on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
            on_tool: &mut dyn FnMut(&super::super::ToolProgress),
            cancel: &AtomicBool,
        ) -> Result<Reply, HarnessError> {
            let body = super::super::budget::encode(&self.profile, request, build_request_body)?;
            let mut headers = vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Accept".to_string(), "text/event-stream".to_string()),
            ];
            let key = self.profile.current_api_key();
            if !key.is_empty() {
                headers.push(("Authorization".to_string(), format!("Bearer {key}")));
            }
            let call = Call { url: endpoint_url(&self.profile.base_url), headers: http::merge_headers(headers, &self.profile.extra_headers), body };
            let mut decoder = ResponsesStream::default();
            http::post_stream_with_policy(&self.client, call, &mut decoder, on_text, on_reasoning, on_tool, cancel, &self.profile.timeouts)?;
            let mut reply = decoder.finish();
            super::super::wire::bind_state(&mut reply.message, &self.profile);
            Ok(reply)
        }
    }
}

#[cfg(test)]
mod protocol_regression_tests {
    use super::*;

    #[test]
    fn deepseek_reasoning_snapshots_do_not_duplicate_deltas() {
        let mut stream = ResponsesStream::default();
        let mut thinking = String::new();
        let mut text = String::new();
        for event in [
            json!({"type":"response.reasoning_text.delta","output_index":0,"content_index":0,"delta":"分析"}),
            json!({"type":"response.reasoning_text.done","output_index":0,"content_index":0,"text":"分析问题"}),
            json!({"type":"response.completed","response":{"output":[{"type":"reasoning","content":[{"type":"reasoning_text","text":"分析问题"}]},{"type":"message","content":[{"type":"output_text","text":"回答"}]}]}}),
        ] {
            stream.push_event_with_reasoning(&event, &mut |s| text.push_str(s), &mut |_, s| thinking.push_str(s)).unwrap();
        }
        assert_eq!(thinking, "分析问题");
        assert_eq!(text, "回答");
        assert!(stream.finish().message.provider_state.is_some());
    }

    #[test]
    fn completed_snapshot_fills_text_suffix_and_missing_parallel_call() {
        let mut stream = ResponsesStream::default();
        let mut text = String::new();
        for event in [
            json!({"type":"response.output_text.delta","output_index":0,"delta":"hello"}),
            json!({"type":"response.output_item.added","output_index":1,"item":{"type":"function_call","call_id":"a","name":"first","arguments":""}}),
            json!({"type":"response.completed","response":{"output":[
                {"type":"message","content":[{"type":"output_text","text":"hello world"}]},
                {"type":"function_call","call_id":"a","name":"first","arguments":"{}"},
                {"type":"function_call","call_id":"b","name":"second","arguments":"{\"x\":1}"}
            ]}}),
        ] { stream.push_event(&event, &mut |s| text.push_str(s)).unwrap(); }
        assert_eq!(text, "hello world");
        let reply = stream.finish();
        assert_eq!(reply.message.tool_calls.len(), 2);
        assert_eq!(reply.message.tool_calls[1].arguments, "{\"x\":1}");
    }
}

#[cfg(test)]
mod final_snapshot_tests {
    use super::*;

    #[test]
    fn text_done_part_done_item_done_and_completed_share_one_dedup_key() {
        let mut stream = ResponsesStream::default();
        let mut text = String::new();
        for event in [
            json!({"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":"hello"}),
            json!({"type":"response.output_text.done","output_index":0,"content_index":0,"text":"hello world"}),
            json!({"type":"response.content_part.done","output_index":0,"content_index":0,"part":{"type":"output_text","text":"hello world"}}),
            json!({"type":"response.output_item.done","output_index":0,"item":{"type":"message","content":[{"type":"output_text","text":"hello world"}]}}),
            json!({"type":"response.completed","response":{"output":[{"type":"message","content":[{"type":"output_text","text":"hello world"}]}]}}),
        ] { stream.push_event(&event, &mut |s| text.push_str(s)).unwrap(); }
        assert_eq!(text, "hello world");
        assert_eq!(stream.finish().message.text(), text);
    }

    #[test]
    fn incomplete_reasoning_is_displayed_but_not_replayed_as_complete_state() {
        let mut stream = ResponsesStream::default();
        let mut thinking = String::new();
        stream.push_event_with_reasoning(&json!({"type":"response.incomplete","response":{
            "incomplete_details":{"reason":"max_output_tokens"},
            "output":[{"type":"reasoning","status":"incomplete","content":[{"type":"reasoning_text","text":"partial thought"}]}]
        }}), &mut |_| {}, &mut |_, s| thinking.push_str(s)).unwrap();
        assert_eq!(thinking, "partial thought");
        let reply = stream.finish();
        assert_eq!(reply.finish, FinishReason::Length);
        assert!(reply.message.provider_state.is_none());
    }
}

#[cfg(test)]
mod reliability_tests {
    use super::*;

    #[test]
    fn ordered_output_and_phase_survive_the_next_tool_request() {
        let mut stream = ResponsesStream::default();
        let output = json!([
            {"type":"message","id":"m1","role":"assistant","phase":"commentary","content":[{"type":"output_text","text":"before"}]},
            {"type":"reasoning","id":"r1","summary":[{"type":"summary_text","text":"checking"}],"encrypted_content":"opaque"},
            {"type":"function_call","id":"f1","call_id":"c1","name":"lookup","arguments":"{}"}
        ]);
        stream.push_event(&json!({"type":"response.completed","response":{"output":output}}), &mut |_| {}).unwrap();
        let mut reply = stream.finish();
        let profile = ProviderProfile::new("https://local/v1", "", "model");
        super::super::wire::bind_state(&mut reply.message, &profile);
        let request = ChatRequest { messages: vec![Message::user_text("question"), reply.message, Message::tool("c1", "result")], tools: vec![] };
        let body = build_request_body(&profile, &request);
        assert_eq!(body["input"][1], output[0]);
        assert_eq!(body["input"][2], output[1]);
        assert_eq!(body["input"][3], output[2]);
        let mut edited = request;
        edited.messages[1].tool_calls[0].arguments = "{\"changed\":true}".into();
        assert_eq!(build_request_body(&profile, &edited)["input"][3]["arguments"], "{\"changed\":true}", "stale raw calls must never override middleware edits");
    }

    #[test]
    fn failed_terminal_snapshot_still_exposes_safe_partial_text() {
        let mut stream = ResponsesStream::default(); let mut text = String::new();
        let result = stream.push_event(&json!({"type":"response.failed","response":{
            "error":{"code":"server_error","message":"failed"},
            "output":[{"type":"message","content":[{"type":"output_text","text":"partial"}]}]
        }}), &mut |s| text.push_str(s));
        assert!(result.is_err()); assert_eq!(text, "partial");
    }

    #[test]
    fn inconsistent_final_snapshot_is_not_silently_reported_as_success() {
        let mut stream = ResponsesStream::default();
        stream.push_event(&json!({"type":"response.output_text.delta","output_index":0,"delta":"wrong"}), &mut |_| {}).unwrap();
        assert!(stream.push_event(&json!({"type":"response.completed","response":{"output":[
            {"type":"message","content":[{"type":"output_text","text":"correct"}]}]}}), &mut |_| {}).is_err());
    }
}

//! OpenAI Responses protocol (`/v1/responses`) with streaming.
//!
//! Stateless: the whole conversation is sent as `input` on every call, and `store` is false unless
//! `extra_body` says otherwise. Request and stream formats are written from the public protocol
//! description and checked only against local fake servers, not against a live endpoint.
//! Not supported: reasoning items (they are not carried between turns), built-in tools such as
//! web search, `previous_response_id`.

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
    for message in &request.messages {
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
                let images: Vec<Value> = message.parts.iter().filter_map(|p| match p {
                    Part::Image { url } => Some(json!({"type": "input_image", "image_url": url, "detail": "auto"})),
                    _ => None,
                }).collect();
                let output = if images.is_empty() { json!(text) } else {
                    let mut content = vec![json!({"type": "input_text", "text": text})];
                    content.extend(images);
                    json!(content)
                };
                input.push(json!({ "type": "function_call_output", "call_id": call_id, "output": output }));
            }
        }
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
    calls: Vec<Option<PartialCall>>,
    usage: Option<Usage>,
    finish: FinishReason,
    done: bool,
}

fn str_of(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

fn failure(code: &str, message: String) -> HarnessError {
    HarnessError::provider(message, matches!(code, "server_error" | "rate_limit_exceeded"))
}

impl ResponsesStream {
    pub fn push_payload(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        let event = serde_json::from_str(payload).map_err(|e| HarnessError::provider(format!("bad stream event: {e}"), false))?;
        self.push_event(&event, on_text)
    }

    /// Returns `Ok(true)` once the response is completed or ended early (`incomplete`).
    pub fn push_event(&mut self, event: &Value, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        match event.get("type").and_then(Value::as_str).unwrap_or("") {
            "response.output_text.delta" | "response.refusal.delta" => {
                if let Some(delta) = event.get("delta").and_then(Value::as_str).filter(|d| !d.is_empty()) {
                    self.text.push_str(delta);
                    on_text(delta);
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
                self.reconcile(response, on_text);
                self.done = true;
                return Ok(true);
            }
            "response.failed" => {
                let error = event.pointer("/response/error").unwrap_or(&Value::Null);
                let message = error.get("message").and_then(Value::as_str).unwrap_or("the response failed").to_string();
                return Err(failure(error.get("code").and_then(Value::as_str).unwrap_or(""), message));
            }
            "error" => {
                let error = event.get("error").unwrap_or(event);
                let message = error.get("message").and_then(Value::as_str).map_or_else(|| event.to_string(), str::to_string);
                return Err(failure(error.get("code").and_then(Value::as_str).unwrap_or(""), message));
            }
            _ => {}
        }
        Ok(false)
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
        self.usage = Some(Usage { prompt_tokens: get("input_tokens"), completion_tokens: get("output_tokens") });
    }

    /// Some gateways only send the final `response` object. If nothing arrived incrementally,
    /// take the content from its `output` list.
    fn reconcile(&mut self, response: &Value, on_text: &mut dyn FnMut(&str)) {
        let has_calls = self.calls.iter().any(Option::is_some);
        if !self.text.is_empty() || has_calls {
            return;
        }
        for item in response.get("output").and_then(Value::as_array).into_iter().flatten() {
            match item.get("type").and_then(Value::as_str) {
                Some("message") => {
                    for part in item.get("content").and_then(Value::as_array).into_iter().flatten() {
                        if let Some(text) = part.get("text").and_then(Value::as_str).filter(|t| !t.is_empty()) {
                            self.text.push_str(text);
                            on_text(text);
                        }
                    }
                }
                Some("function_call") if self.calls.len() < MAX_ITEMS => {
                    self.calls.push(Some(PartialCall {
                        item_id: str_of(item, "id"),
                        call_id: str_of(item, "call_id"),
                        name: str_of(item, "name"),
                        arguments: str_of(item, "arguments"),
                    }));
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
        Reply { message: Message::assistant(self.text, calls), usage: self.usage, finish }
    }
}

#[cfg(feature = "openai")]
pub use client::OpenAiResponses;

#[cfg(feature = "openai")]
mod client {
    use super::super::http::{self, Call, Decoder};
    use super::super::{ModelProvider, Reply};
    use super::*;
    use reqwest::blocking::Client;
    use std::sync::atomic::AtomicBool;

    impl Decoder for ResponsesStream {
        fn push(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
            self.push_payload(payload, on_text)
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
            Ok(Self { profile, client: http::new_client()? })
        }
    }

    impl ModelProvider for OpenAiResponses {
        fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
            let body = serde_json::to_vec(&build_request_body(&self.profile, request))
                .map_err(|e| HarnessError::provider(format!("encode request: {e}"), false))?;
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
            http::post_stream(&self.client, call, &mut decoder, on_text, cancel)?;
            Ok(decoder.finish())
        }
    }
}

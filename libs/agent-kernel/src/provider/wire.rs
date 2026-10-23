use super::ChatRequest;
use crate::types::{Message, Part, Role};
use serde_json::{json, Map, Value};
use std::{fmt, sync::Arc};

/// Everything needed to talk to one OpenAI-compatible endpoint.
#[derive(Clone)]
pub struct ProviderProfile {
    /// Without the `/chat/completions` suffix, e.g. `https://api.openai.com/v1`.
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub extra_headers: Vec<(String, String)>,
    /// Merged into the request body last, e.g. `{"temperature": 0.3}`.
    pub extra_body: Map<String, Value>,
    /// Ask for `stream_options.include_usage`. Turn off for endpoints that reject it.
    pub stream_usage: bool,
    /// Send tool schemas. Turn off for models without function calling.
    pub supports_tools: bool,
    /// Optional output ceiling. Leave unset for automatic Chat Completions and
    /// Responses calls (server default). Anthropic requires max_tokens, so it
    /// resolves model-family capabilities when this field is unset.
    pub max_output_tokens: Option<u32>,
    /// Serialized request cap. Older tool images may be omitted from the request only;
    /// user images and the newest tool image result are preserved. None disables the cap.
    pub max_request_bytes: Option<usize>,
    /// Asked for the key on every request, for credentials that expire while a long run is going.
    /// Empty means "use `api_key`".
    pub api_key_source: Option<Arc<dyn Fn() -> String + Send + Sync>>,
    /// Claude Messages only: mark the system prompt, the tool list and the newest message as cacheable
    /// (`cache_control`), so a long run re-sends its unchanged prefix at the cached rate. Off by default because
    /// some compatible gateways reject unknown fields.
    pub prompt_cache: bool,
}

impl ProviderProfile {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            extra_headers: Vec::new(),
            extra_body: Map::new(),
            stream_usage: true,
            supports_tools: true,
            max_output_tokens: None,
            max_request_bytes: None,
            api_key_source: None,
            prompt_cache: false,
        }
    }

    pub fn with_api_key_source(mut self, source: Arc<dyn Fn() -> String + Send + Sync>) -> Self {
        self.api_key_source = Some(source);
        self
    }

    /// The key to send with the next request.
    pub fn current_api_key(&self) -> String {
        self.api_key_source.as_ref().map(|source| source()).filter(|key| !key.is_empty()).unwrap_or_else(|| self.api_key.clone())
    }
}

impl fmt::Debug for ProviderProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderProfile")
            .field("base_url", &self.base_url)
            .field("api_key", &"<redacted>")
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

/// Bind opaque continuation state to the endpoint/model that produced it, never to credentials.
pub(crate) fn state_source(profile: &ProviderProfile) -> Value {
    json!({"base_url":profile.base_url.trim().trim_end_matches('/'), "model":profile.model})
}
pub(crate) fn matching_state<'a>(message: &'a Message, profile: &ProviderProfile, protocol: &str) -> Option<&'a Value> {
    message.provider_state.as_ref().filter(|s| s["protocol"] == protocol && s.get("source").is_none_or(|source| source == &state_source(profile)))
}
pub(crate) fn bind_state(message: &mut Message, profile: &ProviderProfile) {
    if let Some(state) = message.provider_state.as_mut() { state["source"] = state_source(profile); }
}

pub fn message_to_wire(message: &Message) -> Value {
    match message.role {
        Role::Tool => json!({
            "role": "tool",
            "tool_call_id": message.tool_call_id,
            "content": message.tool_text_with_error(),
        }),
        Role::Assistant => {
            let text = message.text();
            let mut wire = json!({ "role": "assistant", "content": if text.is_empty() { Value::Null } else { Value::String(text) } });
            if let Some(state) = message.provider_state.as_ref().filter(|s| s["protocol"] == "chat") {
                if let Some(refusal) = state["refusal"].as_str() {
                    let content = state["content"].as_str().unwrap_or("");
                    if message.text() == format!("{content}{refusal}") {
                        wire["content"] = state["content"].clone();
                        wire["refusal"] = Value::String(refusal.into());
                    }
                }
                for key in ["reasoning_content", "reasoning", "reasoning_text", "reasoning_details"] {
                    if let Some(value) = state.get(key) { wire[key] = value.clone(); }
                }
            }
            if !message.tool_calls.is_empty() {
                wire["tool_calls"] = message
                    .tool_calls
                    .iter()
                    .map(|c| {
                        let mut call = json!({ "id": c.id, "type": "function", "function": { "name": c.name, "arguments": c.arguments } });
                        if let Some(extra) = message.provider_state.as_ref().filter(|s| s["protocol"] == "chat").and_then(|s| s["tool_extra_content"].get(&c.id)) {
                            call["extra_content"] = extra.clone();
                        }
                        call
                    })
                    .collect();
            }
            wire
        }
        role => {
            let name = if role == Role::System { "system" } else { "user" };
            let has_image = message.parts.iter().any(|p| matches!(p, Part::Image { .. }));
            let content = if has_image {
                message
                    .parts
                    .iter()
                    .map(|p| match p {
                        Part::Text { text } => json!({ "type": "text", "text": text }),
                        Part::Image { url } => json!({ "type": "image_url", "image_url": { "url": url } }),
                    })
                    .collect()
            } else {
                Value::String(message.text())
            };
            json!({ "role": name, "content": content })
        }
    }
}

pub fn build_request_body(profile: &ProviderProfile, request: &ChatRequest) -> Value {
    // Chat Completions tool content is text-only. Attach tool images in a user message AFTER
    // all adjacent tool results, preserving the assistant-call/result pairing required by APIs.
    let mut messages = Vec::new();
    let mut media = Vec::new();
    for message in &request.messages {
        if message.role != Role::Tool && !media.is_empty() {
            messages.push(json!({"role": "user", "content": std::mem::take(&mut media)}));
        }
        let mut scoped = message.clone();
        if matching_state(message, profile, "chat").is_none() { scoped.provider_state = None; }
        let wire = message_to_wire(&scoped);
        messages.push(wire);
        for part in &message.parts {
            if let (Role::Tool, Part::Image { url }) = (message.role, part) {
                media.push(json!({"type": "text", "text": format!("Image returned by tool call {}:", message.tool_call_id.as_deref().unwrap_or_default())}));
                media.push(json!({"type": "image_url", "image_url": {"url": url}}));
            }
        }
    }
    if !media.is_empty() { messages.push(json!({"role": "user", "content": media})); }
    let mut body = json!({
        "model": profile.model,
        "messages": messages,
        "stream": true,
    });
    if profile.stream_usage {
        body["stream_options"] = json!({ "include_usage": true });
    }
    if profile.supports_tools && !request.tools.is_empty() {
        body["tools"] = request
            .tools
            .iter()
            .map(|t| json!({ "type": "function", "function": { "name": t.name, "description": t.description, "parameters": t.parameters } }))
            .collect();
    }
    for (key, value) in &profile.extra_body {
        body[key] = value.clone();
    }
    body
}

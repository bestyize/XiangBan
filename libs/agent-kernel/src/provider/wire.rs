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
    /// Cap on generated tokens. Anthropic requires one (8192 when unset), Responses sends
    /// `max_output_tokens`, and chat completions sends nothing: endpoints disagree on the parameter
    /// name there, so set `max_tokens` or `max_completion_tokens` in `extra_body` instead.
    pub max_output_tokens: Option<u32>,
    /// Asked for the key on every request, for credentials that expire while a long run is going.
    /// Empty means "use `api_key`".
    pub api_key_source: Option<Arc<dyn Fn() -> String + Send + Sync>>,
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
            api_key_source: None,
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

pub fn message_to_wire(message: &Message) -> Value {
    match message.role {
        Role::Tool => json!({
            "role": "tool",
            "tool_call_id": message.tool_call_id,
            "content": message.text(),
        }),
        Role::Assistant => {
            let text = message.text();
            let mut wire = json!({ "role": "assistant", "content": if text.is_empty() { Value::Null } else { Value::String(text) } });
            if !message.tool_calls.is_empty() {
                wire["tool_calls"] = message
                    .tool_calls
                    .iter()
                    .map(|c| json!({ "id": c.id, "type": "function", "function": { "name": c.name, "arguments": c.arguments } }))
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
    let mut body = json!({
        "model": profile.model,
        "messages": request.messages.iter().map(message_to_wire).collect::<Vec<_>>(),
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

use super::http::{self, Call, Decoder};
use super::stream::StreamAccumulator;
use super::wire::{build_request_body, ProviderProfile};
use super::{ChatRequest, ModelProvider, Reply};
use crate::error::HarnessError;
use reqwest::Client;
use std::sync::atomic::AtomicBool;

/// Streaming `/chat/completions` client for any OpenAI-compatible endpoint.
pub struct OpenAiCompat {
    profile: ProviderProfile,
    client: Client,
}

impl OpenAiCompat {
    pub fn new(profile: ProviderProfile) -> Result<Self, HarnessError> {
        if profile.base_url.trim().is_empty() || profile.model.trim().is_empty() {
            return Err(HarnessError::Config("base_url and model are required".into()));
        }
        Ok(Self { client: http::new_client(&profile)?, profile })
    }
}

#[derive(Default)]
struct ChatDecoder(StreamAccumulator);

impl Decoder for ChatDecoder {
    fn push(&mut self, payload: &str, on_text: &mut dyn FnMut(&str)) -> Result<bool, HarnessError> {
        if payload.trim() == "[DONE]" {
            self.0.mark_done();
            return Ok(true);
        }
        let chunk = serde_json::from_str(payload).map_err(|e| HarnessError::provider(format!("bad stream chunk: {e}"), false))?;
        self.0.push(&chunk, on_text)?;
        Ok(false)
    }

    fn push_with_reasoning(
        &mut self, payload: &str, on_text: &mut dyn FnMut(&str),
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) -> Result<bool, HarnessError> {
        if payload.trim() == "[DONE]" {
            self.0.mark_done();
            return Ok(true);
        }
        let chunk = serde_json::from_str(payload).map_err(|e| HarnessError::provider(format!("bad stream chunk: {e}"), false))?;
        self.0.push_with_reasoning(&chunk, on_text, on_reasoning)?;
        Ok(false)
    }

    fn complete(&self) -> bool {
        self.0.is_finished()
    }
}

impl ModelProvider for OpenAiCompat {
    fn chat(
        &self,
        request: &ChatRequest,
        on_text: &mut dyn FnMut(&str),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        self.chat_with_reasoning(request, on_text, &mut |_, _| {}, cancel)
    }

    fn chat_with_reasoning(
        &self,
        request: &ChatRequest,
        on_text: &mut dyn FnMut(&str),
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        let body = super::budget::encode(&self.profile, request, build_request_body)?;
        let mut headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Accept".to_string(), "text/event-stream".to_string()),
        ];
        let key = self.profile.current_api_key();
        if !key.is_empty() {
            headers.push(("Authorization".to_string(), format!("Bearer {key}")));
        }
        let call = Call {
            url: format!("{}/chat/completions", self.profile.base_url.trim_end_matches('/')),
            headers: http::merge_headers(headers, &self.profile.extra_headers),
            body,
        };
        let mut decoder = ChatDecoder::default();
        http::post_stream_with_policy(&self.client, call, &mut decoder, on_text, on_reasoning, cancel, &self.profile.timeouts)?;
        let mut reply = decoder.0.finish();
        super::wire::bind_state(&mut reply.message, &self.profile);
        Ok(reply)
    }
}

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
    extra_content: Option<Value>,
}

/// Rebuilds one assistant reply from OpenAI-style streaming chunks.
#[derive(Default)]
pub struct StreamAccumulator {
    text: String,
    reasoning: String,
    reasoning_channel: Option<String>,
    answer_content: String,
    refusal: String,
    reasoning_fields: serde_json::Map<String, Value>,
    reasoning_details: Vec<Value>,
    calls: Vec<PartialCall>,
    usage: Option<Usage>,
    finish_reason: Option<String>,
    finished: bool,
}

fn merge_metadata(target: &mut Value, incoming: &Value) {
    if let (Some(target), Some(incoming)) = (target.as_object_mut(), incoming.as_object()) {
        for (key, value) in incoming {
            if let Some(previous) = target.get_mut(key) { merge_metadata(previous, value); }
            else { target.insert(key.clone(), value.clone()); }
        }
    } else if !incoming.is_null() { *target = incoming.clone(); }
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
        self.push_with_reasoning(chunk, on_text, &mut |_, _| {})
    }

    pub fn push_with_reasoning(
        &mut self, chunk: &Value, on_text: &mut dyn FnMut(&str),
        on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str),
    ) -> Result<(), HarnessError> {
        if let Some(error) = chunk.get("error").filter(|e| !e.is_null()) {
            let message = error.get("message").and_then(Value::as_str).map_or_else(|| error.to_string(), str::to_string);
            return Err(HarnessError::provider(message, false));
        }
        if let Some(usage) = chunk.get("usage").filter(|u| u.is_object()) {
            let read = |key: &str| usage.get(key).and_then(Value::as_u64).unwrap_or(0).min(u32::MAX as u64) as u32;
            // OpenAI-style `prompt_tokens_details.cached_tokens`, or DeepSeek's `prompt_cache_hit_tokens`.
            let cached = usage.get("prompt_tokens_details").and_then(|d| d.get("cached_tokens")).and_then(Value::as_u64)
                .or_else(|| usage.get("prompt_cache_hit_tokens").and_then(Value::as_u64))
                .unwrap_or(0).min(u32::MAX as u64) as u32;
            self.usage = Some(Usage { prompt_tokens: read("prompt_tokens"), completion_tokens: read("completion_tokens"), cached_prompt_tokens: cached });
        }
        let Some(choice) = chunk.get("choices").and_then(|c| c.get(0)) else { return Ok(()) };
        if let Some(delta) = choice.get("delta") {
            let mut displayed = false;
            for field in ["reasoning_content", "reasoning", "reasoning_text"] {
                if let Some(text) = delta[field].as_str().filter(|s| !s.is_empty()) {
                    let value = self.reasoning_fields.entry(field.to_string()).or_insert_with(|| Value::String(String::new()));
                    if let Some(previous) = value.as_str() { *value = Value::String(format!("{previous}{text}")); }
                    if self.reasoning_channel.is_none() { self.reasoning_channel = Some(field.into()); }
                    if !displayed && self.reasoning_channel.as_deref() == Some(field) {
                        self.reasoning.push_str(text);
                        on_reasoning(crate::ReasoningKind::Text, text);
                        displayed = true;
                    }
                }
            }
            for detail in delta["reasoning_details"].as_array().into_iter().flatten() {
                let index = detail["index"].as_u64();
                let id = detail["id"].as_str().filter(|s| !s.is_empty());
                let encrypted = detail["type"] == "reasoning.encrypted";
                // Encrypted objects are opaque, discrete records. Never concatenate
                // or overwrite their payload based only on a streaming index.
                let slot = if encrypted { None } else { self.reasoning_details.iter().position(|v| {
                    if v["type"] != detail["type"] { return false; }
                    match (id, v["id"].as_str().filter(|s| !s.is_empty())) {
                        (Some(a), Some(b)) => a == b,
                        _ => index.is_some() && v["index"].as_u64() == index,
                    }
                }) };
                let readable = detail["text"].as_str().or_else(|| detail["summary"].as_str()).is_some_and(|s| !s.is_empty());
                if readable && self.reasoning_channel.is_none() { self.reasoning_channel = Some("details".into()); }
                if !displayed && self.reasoning_channel.as_deref() == Some("details") {
                    for field in ["text", "summary"] {
                        if let Some(text) = detail[field].as_str() {
                            on_reasoning(if field == "summary" { crate::ReasoningKind::Summary } else { crate::ReasoningKind::Text }, text);
                        }
                    }
                }
                if let Some(slot) = slot {
                    if let (Some(target), Some(fields)) = (self.reasoning_details[slot].as_object_mut(), detail.as_object()) {
                        for (key, value) in fields {
                            if ["text", "summary"].contains(&key.as_str()) {
                                if let Some(fragment) = value.as_str() {
                                    let previous = target.get(key).and_then(Value::as_str).unwrap_or("");
                                    target.insert(key.clone(), Value::String(format!("{previous}{fragment}")));
                                }
                            } else if !value.is_null() { target.insert(key.clone(), value.clone()); }
                        }
                    }
                } else { self.reasoning_details.push(detail.clone()); }
            }
            if let Some(text) = delta.get("content").and_then(Value::as_str).filter(|t| !t.is_empty()) {
                self.answer_content.push_str(text);
                self.text.push_str(text);
                on_text(text);
            }
            if let Some(text) = delta["refusal"].as_str().filter(|s| !s.is_empty()) {
                self.refusal.push_str(text);
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
            None if id.is_some() => self.calls.iter().position(|c| Some(c.id.as_str()) == id).unwrap_or(self.calls.len()),
            None => self.calls.len().saturating_sub(1),
        };
        if index >= MAX_TOOL_CALLS {
            return;
        }
        while self.calls.len() <= index {
            self.calls.push(PartialCall::default());
        }
        let slot = &mut self.calls[index];
        if let Some(extra) = call.get("extra_content") {
            if let (Some(target), Some(fields)) = (slot.extra_content.as_mut().and_then(Value::as_object_mut), extra.as_object()) {
                for (key, value) in fields {
                    if let Some(existing) = target.get_mut(key) { merge_metadata(existing, value); }
                    else { target.insert(key.clone(), value.clone()); }
                }
            } else { slot.extra_content = Some(extra.clone()); }
        }
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

    pub fn finish(mut self) -> Reply {
        let extras: serde_json::Map<String, Value> = self.calls.iter().enumerate().filter_map(|(i, call)| {
            call.extra_content.clone().map(|extra| (if call.id.is_empty() { format!("call_{i}") } else { call.id.clone() }, extra))
        }).collect();
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
        let mut message = Message::assistant(self.text, tool_calls);
        if !self.reasoning_fields.is_empty() || !self.reasoning_details.is_empty() || !extras.is_empty() || !self.refusal.is_empty() {
            let mut state = serde_json::json!({"protocol":"chat", "tool_extra_content": extras});
            if !self.refusal.is_empty() {
                state["refusal"] = Value::String(self.refusal);
                state["content"] = if self.answer_content.is_empty() { Value::Null } else { Value::String(self.answer_content) };
            }
            for (key, value) in self.reasoning_fields { state[&key] = value; }
            if !self.reasoning_details.is_empty() {
                for detail in &mut self.reasoning_details {
                    if let Some(fields) = detail.as_object_mut() { fields.remove("index"); }
                }
                state["reasoning_details"] = Value::Array(self.reasoning_details);
            }
            message.provider_state = Some(state);
        }
        Reply { message, usage: self.usage, finish }
    }
}

#[cfg(test)]
mod protocol_regression_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reasoning_aliases_and_gemini_signature_round_trip() {
        let mut stream = StreamAccumulator::default();
        let mut thinking = String::new();
        stream.push_with_reasoning(&json!({"choices":[{"delta":{
            "reasoning":"checking", "tool_calls":[{"index":0,"id":"call_a","function":{"name":"lookup","arguments":"{}"},"extra_content":{"google":{"thought_signature":"opaque"}}}]
        }}]}), &mut |_| {}, &mut |_, s| thinking.push_str(s)).unwrap();
        assert_eq!(thinking, "checking");
        let reply = stream.finish();
        let wire = super::super::wire::message_to_wire(&reply.message);
        assert_eq!(wire["reasoning"], "checking");
        assert_eq!(wire["tool_calls"][0]["extra_content"]["google"]["thought_signature"], "opaque");
    }

    #[test]
    fn structured_reasoning_merges_fragments_without_exposing_encrypted_data() {
        let mut stream = StreamAccumulator::default();
        let mut thinking = String::new();
        for details in [
            json!([{"index":0,"type":"reasoning.text","text":"one","id":"r"}]),
            json!([{"index":0,"type":"reasoning.text","text":" two","signature":"sig"}]),
            json!([{"index":1,"type":"reasoning.encrypted","data":"secret","id":"e"}]),
        ] {
            stream.push_with_reasoning(&json!({"choices":[{"delta":{"reasoning_details":details}}]}), &mut |_| {}, &mut |_, s| thinking.push_str(s)).unwrap();
        }
        assert_eq!(thinking, "one two");
        let wire = super::super::wire::message_to_wire(&stream.finish().message);
        assert_eq!(wire["reasoning_details"][0]["text"], "one two");
        assert!(wire["reasoning_details"][0].get("index").is_none());
        assert_eq!(wire["reasoning_details"][1]["data"], "secret");
    }
}

#[cfg(test)]
mod reliability_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn refusal_is_visible_and_replayed_as_a_refusal() {
        let mut stream = StreamAccumulator::default(); let mut text = String::new();
        stream.push(&json!({"choices":[{"delta":{"refusal":"cannot comply"},"finish_reason":"stop"}]}), &mut |s|text.push_str(s)).unwrap();
        let reply = stream.finish(); assert_eq!(text, "cannot comply"); assert_eq!(reply.message.text(), text);
        let wire = super::super::wire::message_to_wire(&reply.message);
        assert_eq!(wire["refusal"], text); assert!(wire["content"].is_null());
    }
    #[test]
    fn missing_indices_merge_text_by_identity_but_keep_encrypted_objects_distinct() {
        let mut stream=StreamAccumulator::default(); let mut thinking=String::new();
        for detail in [json!({"id":"r","type":"reasoning.text","text":"a","signature":"sig"}),
            json!({"id":"r","type":"reasoning.text","text":"b","signature":null}),
            json!({"id":"e1","index":1,"type":"reasoning.encrypted","data":"one"}),
            json!({"id":"e2","index":1,"type":"reasoning.encrypted","data":"two"})] {
            stream.push_with_reasoning(&json!({"choices":[{"delta":{"reasoning_details":[detail]}}]}), &mut |_|{}, &mut |_,s|thinking.push_str(s)).unwrap();
        }
        let reply=stream.finish(); let details=&reply.message.provider_state.unwrap()["reasoning_details"];
        assert_eq!(thinking,"ab"); assert_eq!(details[0]["text"],"ab"); assert_eq!(details[0]["signature"],"sig");
        assert_eq!(details.as_array().unwrap().len(),3); assert_eq!(details[1]["data"],"one"); assert_eq!(details[2]["data"],"two");
    }
    #[test]
    fn tool_metadata_merges_nested_fields_without_losing_signature() {
        let mut target=json!({"google":{"thought_signature":"opaque","first":1}});
        merge_metadata(&mut target,&json!({"google":{"second":2}}));
        assert_eq!(target["google"]["thought_signature"],"opaque"); assert_eq!(target["google"]["second"],2);
    }
}

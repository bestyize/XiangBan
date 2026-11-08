//! Bridges OpenAI `/v1/chat/completions` callers (such as mobile `AiGateway` and `agent_mobile`)
//! onto New API's `/v1/responses`-only Codex subscription channel, without leaking upstream model names.

use crate::{error::AppError, proxy::map_upstream_error};
use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn extract_text_parts(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| match part.get("type").and_then(Value::as_str) {
                Some("text" | "input_text" | "output_text" | "refusal") => {
                    part.get("text").or_else(|| part.get("refusal")).and_then(Value::as_str)
                }
                None => part.get("text").and_then(Value::as_str),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

fn image_url_part(part: &Value) -> Option<Value> {
    match part.get("type").and_then(Value::as_str) {
        Some("image_url") => {
            let image = part.get("image_url")?;
            let url = image.as_str().or_else(|| image.get("url").and_then(Value::as_str))?.trim();
            if url.is_empty() {
                return None;
            }
            let detail = image.get("detail").and_then(Value::as_str).unwrap_or("auto");
            Some(json!({ "type": "input_image", "image_url": url, "detail": detail }))
        }
        Some("input_image") => {
            let url = part.get("image_url").and_then(Value::as_str)?.trim();
            if url.is_empty() {
                return None;
            }
            let detail = part.get("detail").and_then(Value::as_str).unwrap_or("auto");
            Some(json!({ "type": "input_image", "image_url": url, "detail": detail }))
        }
        _ => None,
    }
}

/// Converts a Chat Completions JSON request (whose `model` has already been replaced with the upstream model)
/// into a stateless streaming `/v1/responses` payload, returning `(payload_bytes, client_requested_stream)`.
pub(crate) fn chat_to_responses(request: &Value) -> Result<(Vec<u8>, bool), AppError> {
    let want_stream = request.get("stream").and_then(Value::as_bool).unwrap_or(false);
    let messages = request
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages 需要是数组".into()))?;
    if messages.is_empty() {
        return Err(AppError::BadRequest("messages 不能为空".into()));
    }

    let mut input: Vec<Value> = Vec::new();
    let mut deferred_media: Vec<Value> = Vec::new();
    for msg in messages {
        let role = msg.get("role").and_then(Value::as_str).unwrap_or_default();
        if role != "tool" && !deferred_media.is_empty() {
            input.push(json!({ "role": "user", "content": std::mem::take(&mut deferred_media) }));
        }
        match role {
            "system" | "developer" => {
                let text = msg.get("content").map(extract_text_parts).unwrap_or_default();
                if !text.trim().is_empty() {
                    input.push(json!({ "role": role, "content": text }));
                }
            }
            "user" => {
                let parts: Vec<Value> = match msg.get("content") {
                    Some(Value::String(s)) if !s.trim().is_empty() => vec![json!({ "type": "input_text", "text": s })],
                    Some(Value::Array(items)) => items
                        .iter()
                        .filter_map(|part| {
                            if let Some(img) = image_url_part(part) {
                                return Some(img);
                            }
                            let text = part.get("text").and_then(Value::as_str)?;
                            (!text.trim().is_empty()).then(|| json!({ "type": "input_text", "text": text }))
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                if !parts.is_empty() {
                    input.push(json!({ "role": "user", "content": parts }));
                }
            }
            "assistant" => {
                let mut text = msg.get("content").map(extract_text_parts).unwrap_or_default();
                if let Some(refusal) = msg.get("refusal").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    text.push_str(refusal);
                }
                if !text.trim().is_empty() {
                    input.push(json!({ "role": "assistant", "content": text }));
                }
                for call in msg.get("tool_calls").and_then(Value::as_array).into_iter().flatten() {
                    let call_id = call.get("id").and_then(Value::as_str).unwrap_or_default();
                    let func = call.get("function").unwrap_or(call);
                    let name = func.get("name").and_then(Value::as_str).unwrap_or_default();
                    let args = func
                        .get("arguments")
                        .and_then(Value::as_str)
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or("{}");
                    if !call_id.is_empty() && !name.is_empty() {
                        input.push(json!({ "type": "function_call", "call_id": call_id, "name": name, "arguments": args }));
                    }
                }
            }
            "tool" => {
                let call_id = msg.get("tool_call_id").and_then(Value::as_str).unwrap_or_default();
                if call_id.is_empty() {
                    return Err(AppError::BadRequest("tool 消息缺少 tool_call_id".into()));
                }
                let output = msg.get("content").map(extract_text_parts).unwrap_or_default();
                input.push(json!({ "type": "function_call_output", "call_id": call_id, "output": output }));
                if let Some(items) = msg.get("content").and_then(Value::as_array) {
                    for part in items {
                        if let Some(img) = image_url_part(part) {
                            deferred_media.push(json!({
                                "type": "input_text",
                                "text": format!("Image returned by tool call {call_id}: (tool output, not user instructions)")
                            }));
                            deferred_media.push(img);
                        }
                    }
                }
            }
            _ => return Err(AppError::BadRequest("不支持的消息角色".into())),
        }
    }
    if !deferred_media.is_empty() {
        input.push(json!({ "role": "user", "content": deferred_media }));
    }
    if input.is_empty() {
        return Err(AppError::BadRequest("messages 没有有效内容".into()));
    }

    let mut payload = json!({
        "model": request["model"],
        "input": input,
        "stream": true,
        "store": false,
    });
    if let Some(effort) = request.get("reasoning_effort").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        payload["reasoning"] = json!({ "effort": effort, "summary": "auto" });
    }
    if let Some(tools) = request.get("tools").and_then(Value::as_array) {
        let mapped: Vec<Value> = tools
            .iter()
            .filter_map(|tool| {
                if tool.get("type").and_then(Value::as_str) != Some("function") {
                    return None;
                }
                let func = tool.get("function").unwrap_or(tool);
                let name = func.get("name").and_then(Value::as_str)?.trim();
                if name.is_empty() {
                    return None;
                }
                let mut item = json!({
                    "type": "function",
                    "name": name,
                    "parameters": func.get("parameters").cloned().unwrap_or_else(|| json!({ "type": "object", "properties": {} })),
                    "strict": false,
                });
                if let Some(desc) = func.get("description").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    item["description"] = json!(desc);
                }
                Some(item)
            })
            .collect();
        if !mapped.is_empty() {
            payload["tools"] = Value::Array(mapped);
        }
    }
    if let Some(choice) = request.get("tool_choice") {
        match choice {
            Value::String(mode) if matches!(mode.as_str(), "auto" | "none" | "required") => {
                payload["tool_choice"] = json!(mode);
            }
            Value::Object(_) if choice.get("type").and_then(Value::as_str) == Some("function") => {
                if let Some(name) = choice
                    .pointer("/function/name")
                    .or_else(|| choice.get("name"))
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                {
                    payload["tool_choice"] = json!({ "type": "function", "name": name });
                }
            }
            _ => {}
        }
    }
    if let Some(parallel) = request.get("parallel_tool_calls").and_then(Value::as_bool) {
        payload["parallel_tool_calls"] = json!(parallel);
    }

    let bytes = serde_json::to_vec(&payload).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((bytes, want_stream))
}

#[derive(Default)]
struct ToolSlot {
    chat_index: usize,
    call_id: String,
    name: String,
    arguments: String,
    header_sent: bool,
}

struct ResponsesToChat {
    id: String,
    created: i64,
    tier: String,
    role_sent: bool,
    text: String,
    reasoning: String,
    text_seen: BTreeMap<(usize, usize), String>,
    reasoning_seen: BTreeMap<(usize, usize), String>,
    summary_seen: BTreeMap<(usize, usize), String>,
    calls: BTreeMap<usize, ToolSlot>,
    usage: Option<Value>,
    finish_reason: Option<String>,
    failed: Option<String>,
    done: bool,
}

impl ResponsesToChat {
    fn new(tier: String) -> Self {
        Self {
            id: "chatcmpl-xiangban".into(),
            created: chrono::Utc::now().timestamp(),
            tier,
            role_sent: false,
            text: String::new(),
            reasoning: String::new(),
            text_seen: BTreeMap::new(),
            reasoning_seen: BTreeMap::new(),
            summary_seen: BTreeMap::new(),
            calls: BTreeMap::new(),
            usage: None,
            finish_reason: None,
            failed: None,
            done: false,
        }
    }

    fn chunk(&self, delta: Value, finish_reason: Option<&str>, usage: Option<Value>) -> String {
        let mut obj = json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.tier,
            "choices": [{
                "index": 0,
                "delta": delta,
                "finish_reason": finish_reason,
            }],
        });
        if let Some(u) = usage {
            obj["usage"] = u;
        }
        format!("data: {obj}\n\n")
    }

    fn ensure_role(&mut self, out: &mut String) {
        if !self.role_sent {
            self.role_sent = true;
            out.push_str(&self.chunk(json!({ "role": "assistant", "content": "" }), None, None));
        }
    }

    fn push_text_delta(&mut self, key: (usize, usize), delta: &str, out: &mut String) {
        if delta.is_empty() {
            return;
        }
        self.ensure_role(out);
        self.text_seen.entry(key).or_default().push_str(delta);
        self.text.push_str(delta);
        out.push_str(&self.chunk(json!({ "content": delta }), None, None));
    }

    fn reconcile_text(&mut self, key: (usize, usize), full: &str, out: &mut String) {
        let prev = self.text_seen.entry(key).or_default();
        if let Some(suffix) = full.strip_prefix(prev.as_str()).filter(|s| !s.is_empty()) {
            let suffix = suffix.to_string();
            prev.push_str(&suffix);
            self.ensure_role(out);
            self.text.push_str(&suffix);
            out.push_str(&self.chunk(json!({ "content": suffix }), None, None));
        }
    }

    fn push_reasoning_delta(&mut self, is_summary: bool, key: (usize, usize), delta: &str, out: &mut String) {
        if delta.is_empty() {
            return;
        }
        self.ensure_role(out);
        let map = if is_summary { &mut self.summary_seen } else { &mut self.reasoning_seen };
        map.entry(key).or_default().push_str(delta);
        self.reasoning.push_str(delta);
        out.push_str(&self.chunk(json!({ "reasoning_content": delta }), None, None));
    }

    fn reconcile_reasoning(&mut self, is_summary: bool, key: (usize, usize), full: &str, out: &mut String) {
        let map = if is_summary { &mut self.summary_seen } else { &mut self.reasoning_seen };
        let prev = map.entry(key).or_default();
        if let Some(suffix) = full.strip_prefix(prev.as_str()).filter(|s| !s.is_empty()) {
            let suffix = suffix.to_string();
            prev.push_str(&suffix);
            self.ensure_role(out);
            self.reasoning.push_str(&suffix);
            out.push_str(&self.chunk(json!({ "reasoning_content": suffix }), None, None));
        }
    }

    fn slot_mut(&mut self, output_index: usize) -> &mut ToolSlot {
        let next_chat_index = self.calls.len();
        self.calls.entry(output_index).or_insert_with(|| ToolSlot {
            chat_index: next_chat_index,
            ..ToolSlot::default()
        })
    }

    fn reconcile_item(&mut self, output_index: usize, item: &Value, out: &mut String) {
        match item.get("type").and_then(Value::as_str) {
            Some("message") => {
                for (part_idx, part) in item.get("content").and_then(Value::as_array).into_iter().flatten().enumerate() {
                    if matches!(part.get("type").and_then(Value::as_str), Some("output_text" | "refusal")) {
                        if let Some(full) = part.get("text").or_else(|| part.get("refusal")).and_then(Value::as_str) {
                            self.reconcile_text((output_index, part_idx), full, out);
                        }
                    }
                }
            }
            Some("reasoning") => {
                for (idx, part) in item.get("content").and_then(Value::as_array).into_iter().flatten().enumerate() {
                    if part.get("type").and_then(Value::as_str) == Some("reasoning_text") {
                        if let Some(full) = part.get("text").and_then(Value::as_str) {
                            self.reconcile_reasoning(false, (output_index, idx), full, out);
                        }
                    }
                }
                for (idx, part) in item.get("summary").and_then(Value::as_array).into_iter().flatten().enumerate() {
                    if let Some(full) = part.get("text").and_then(Value::as_str) {
                        self.reconcile_reasoning(true, (output_index, idx), full, out);
                    }
                }
            }
            Some("function_call") => {
                self.ensure_role(out);
                let call_id = item.get("call_id").or_else(|| item.get("id")).and_then(Value::as_str).unwrap_or_default().to_string();
                let name = item.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
                let full_args = item.get("arguments").and_then(Value::as_str).unwrap_or_default();
                let slot = self.slot_mut(output_index);
                if !call_id.is_empty() {
                    slot.call_id = call_id;
                }
                if !name.is_empty() {
                    slot.name = name;
                }
                let chat_index = slot.chat_index;
                if !slot.header_sent {
                    slot.header_sent = true;
                    slot.arguments = full_args.to_string();
                    let delta = json!({
                        "tool_calls": [{
                            "index": chat_index,
                            "id": slot.call_id.clone(),
                            "type": "function",
                            "function": { "name": slot.name.clone(), "arguments": slot.arguments.clone() }
                        }]
                    });
                    out.push_str(&self.chunk(delta, None, None));
                } else if let Some(suffix) = full_args.strip_prefix(slot.arguments.as_str()).filter(|s| !s.is_empty()) {
                    let suffix = suffix.to_string();
                    slot.arguments.push_str(&suffix);
                    let delta = json!({
                        "tool_calls": [{
                            "index": chat_index,
                            "function": { "arguments": suffix }
                        }]
                    });
                    out.push_str(&self.chunk(delta, None, None));
                }
            }
            _ => {}
        }
    }

    fn parse_usage(usage: Option<&Value>) -> Option<Value> {
        let u = usage?.as_object()?;
        let prompt = u.get("input_tokens").or_else(|| u.get("prompt_tokens")).and_then(Value::as_u64)?;
        let completion = u.get("output_tokens").or_else(|| u.get("completion_tokens")).and_then(Value::as_u64).unwrap_or(0);
        let total = u.get("total_tokens").and_then(Value::as_u64).unwrap_or(prompt + completion);
        let cached = u
            .get("input_tokens_details")
            .and_then(|d| d.get("cached_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        Some(json!({
            "prompt_tokens": prompt,
            "completion_tokens": completion,
            "total_tokens": total,
            "prompt_tokens_details": { "cached_tokens": cached },
        }))
    }

    fn finish_response(&mut self, response: &Value, incomplete: bool, out: &mut String) {
        if let Some(id) = response.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()) {
            self.id = if id.starts_with("chatcmpl-") { id.to_string() } else { format!("chatcmpl-{id}") };
        }
        if let Some(ts) = response.get("created_at").and_then(Value::as_i64) {
            self.created = ts;
        }
        for (idx, item) in response.get("output").and_then(Value::as_array).into_iter().flatten().enumerate() {
            self.reconcile_item(idx, item, out);
        }
        self.usage = Self::parse_usage(response.get("usage"));
        let reason = if !self.calls.is_empty() {
            "tool_calls"
        } else if incomplete {
            match response.pointer("/incomplete_details/reason").and_then(Value::as_str) {
                Some("max_output_tokens") => "length",
                Some("content_filter") => "content_filter",
                _ => "stop",
            }
        } else {
            "stop"
        };
        self.ensure_role(out);
        self.finish_reason = Some(reason.to_string());
        let usage = self.usage.clone();
        out.push_str(&self.chunk(json!({}), Some(reason), usage));
        out.push_str("data: [DONE]\n\n");
        self.done = true;
    }

    fn on_event(&mut self, event: &Value) -> String {
        if self.done {
            return String::new();
        }
        let mut out = String::new();
        let kind = event.get("type").and_then(Value::as_str).unwrap_or_default();
        let out_idx = event.get("output_index").and_then(Value::as_u64).unwrap_or(0) as usize;
        let content_idx = event.get("content_index").and_then(Value::as_u64).unwrap_or(0) as usize;
        let summary_idx = event.get("summary_index").and_then(Value::as_u64).unwrap_or(0) as usize;

        match kind {
            "response.created" => {
                if let Some(resp) = event.get("response") {
                    if let Some(id) = resp.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                        self.id = if id.starts_with("chatcmpl-") { id.to_string() } else { format!("chatcmpl-{id}") };
                    }
                    if let Some(ts) = resp.get("created_at").and_then(Value::as_i64) {
                        self.created = ts;
                    }
                }
            }
            "response.output_text.delta" | "response.refusal.delta" => {
                if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                    self.push_text_delta((out_idx, content_idx), delta, &mut out);
                }
            }
            "response.output_text.done" | "response.refusal.done" => {
                if let Some(full) = event.get("text").or_else(|| event.get("refusal")).and_then(Value::as_str) {
                    self.reconcile_text((out_idx, content_idx), full, &mut out);
                }
            }
            "response.content_part.done" => {
                if let Some(part) = event.get("part") {
                    if matches!(part.get("type").and_then(Value::as_str), Some("output_text" | "refusal")) {
                        if let Some(full) = part.get("text").or_else(|| part.get("refusal")).and_then(Value::as_str) {
                            self.reconcile_text((out_idx, content_idx), full, &mut out);
                        }
                    }
                }
            }
            "response.reasoning_text.delta" => {
                if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                    self.push_reasoning_delta(false, (out_idx, content_idx), delta, &mut out);
                }
            }
            "response.reasoning_text.done" => {
                if let Some(full) = event.get("text").and_then(Value::as_str) {
                    self.reconcile_reasoning(false, (out_idx, content_idx), full, &mut out);
                }
            }
            "response.reasoning_summary_text.delta" => {
                if let Some(delta) = event.get("delta").and_then(Value::as_str) {
                    self.push_reasoning_delta(true, (out_idx, summary_idx), delta, &mut out);
                }
            }
            "response.reasoning_summary_text.done" => {
                if let Some(full) = event.get("text").and_then(Value::as_str) {
                    self.reconcile_reasoning(true, (out_idx, summary_idx), full, &mut out);
                }
            }
            "response.reasoning_summary_part.done" => {
                if let Some(full) = event.pointer("/part/text").and_then(Value::as_str) {
                    self.reconcile_reasoning(true, (out_idx, summary_idx), full, &mut out);
                }
            }
            "response.output_item.added" => {
                if let Some(item) = event.get("item") {
                    if item.get("type").and_then(Value::as_str) == Some("function_call") {
                        self.ensure_role(&mut out);
                        let call_id = item.get("call_id").or_else(|| item.get("id")).and_then(Value::as_str).unwrap_or_default().to_string();
                        let name = item.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
                        let args = item.get("arguments").and_then(Value::as_str).unwrap_or_default().to_string();
                        let slot = self.slot_mut(out_idx);
                        slot.call_id = call_id.clone();
                        slot.name = name.clone();
                        slot.arguments = args.clone();
                        slot.header_sent = true;
                        let chat_index = slot.chat_index;
                        let delta = json!({
                            "tool_calls": [{
                                "index": chat_index,
                                "id": call_id,
                                "type": "function",
                                "function": { "name": name, "arguments": args }
                            }]
                        });
                        out.push_str(&self.chunk(delta, None, None));
                    }
                }
            }
            "response.function_call_arguments.delta" => {
                if let Some(delta_args) = event.get("delta").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    self.ensure_role(&mut out);
                    let slot = self.slot_mut(out_idx);
                    slot.arguments.push_str(delta_args);
                    let chat_index = slot.chat_index;
                    let delta = json!({
                        "tool_calls": [{
                            "index": chat_index,
                            "function": { "arguments": delta_args }
                        }]
                    });
                    out.push_str(&self.chunk(delta, None, None));
                }
            }
            "response.output_item.done" => {
                if let Some(item) = event.get("item") {
                    self.reconcile_item(out_idx, item, &mut out);
                }
            }
            "response.completed" => {
                if let Some(resp) = event.get("response") {
                    self.finish_response(resp, false, &mut out);
                }
            }
            "response.incomplete" => {
                if let Some(resp) = event.get("response") {
                    self.finish_response(resp, true, &mut out);
                }
            }
            "response.failed" | "error" => {
                tracing::warn!("Responses 流内报错: {}", event.to_string().chars().take(300).collect::<String>());
                let err = json!({
                    "error": { "message": "AI 服务暂时不可用，请稍后再试", "type": "api_error", "code": "bad_gateway" }
                });
                self.failed = Some("AI 服务暂时不可用，请稍后再试".into());
                self.done = true;
                out.push_str(&format!("data: {err}\n\ndata: [DONE]\n\n"));
            }
            _ if event.get("object").and_then(Value::as_str) == Some("response") => {
                self.finish_response(event, event.get("status").and_then(Value::as_str) == Some("incomplete"), &mut out);
            }
            _ => {}
        }
        out
    }

    fn feed_line(&mut self, line: &str) -> String {
        let line = line.trim_end_matches(['\r', '\n']);
        let Some(rest) = line.strip_prefix("data:") else { return String::new() };
        let payload = rest.trim();
        if payload.is_empty() {
            return String::new();
        }
        if payload == "[DONE]" {
            if !self.done {
                self.done = true;
                let reason = if !self.calls.is_empty() { "tool_calls" } else { "stop" };
                self.finish_reason = Some(reason.into());
                let mut out = String::new();
                self.ensure_role(&mut out);
                let usage = self.usage.clone();
                out.push_str(&self.chunk(json!({}), Some(reason), usage));
                out.push_str("data: [DONE]\n\n");
                return out;
            }
            return String::new();
        }
        let Ok(event) = serde_json::from_str::<Value>(payload) else { return String::new() };
        self.on_event(&event)
    }

    fn into_non_streaming(mut self) -> Result<Value, AppError> {
        if let Some(msg) = self.failed.take() {
            return Err(AppError::BadGateway(msg));
        }
        let tool_calls: Vec<Value> = self
            .calls
            .into_values()
            .map(|slot| {
                json!({
                    "id": if slot.call_id.is_empty() { format!("call_{}", slot.chat_index) } else { slot.call_id },
                    "type": "function",
                    "function": {
                        "name": slot.name,
                        "arguments": if slot.arguments.is_empty() { "{}".into() } else { slot.arguments },
                    }
                })
            })
            .collect();
        let finish = self
            .finish_reason
            .unwrap_or_else(|| if tool_calls.is_empty() { "stop".into() } else { "tool_calls".into() });
        let content = if self.text.is_empty() && !tool_calls.is_empty() {
            Value::Null
        } else {
            Value::String(self.text)
        };
        let mut message = json!({
            "role": "assistant",
            "content": content,
        });
        if !self.reasoning.is_empty() {
            message["reasoning_content"] = json!(self.reasoning);
        }
        if !tool_calls.is_empty() {
            message["tool_calls"] = Value::Array(tool_calls);
        }
        let mut reply = json!({
            "id": self.id,
            "object": "chat.completion",
            "created": self.created,
            "model": self.tier,
            "choices": [{
                "index": 0,
                "message": message,
                "finish_reason": finish,
            }],
        });
        if let Some(usage) = self.usage {
            reply["usage"] = usage;
        }
        Ok(reply)
    }
}

fn adapt_stream<S>(inner: S, tier: String) -> impl Stream<Item = Result<Bytes, std::io::Error>>
where
    S: Stream<Item = reqwest::Result<Bytes>> + Unpin + Send + 'static,
{
    struct State<S> {
        inner: S,
        buffer: Vec<u8>,
        converter: ResponsesToChat,
        finished: bool,
    }
    futures_util::stream::unfold(
        State { inner, buffer: Vec::new(), converter: ResponsesToChat::new(tier), finished: false },
        |mut st| async move {
            if st.finished {
                return None;
            }
            loop {
                match st.inner.next().await {
                    Some(Ok(chunk)) => {
                        st.buffer.extend_from_slice(&chunk);
                        let Some(last_nl) = st.buffer.iter().rposition(|b| *b == b'\n') else { continue };
                        let complete: Vec<u8> = st.buffer.drain(..=last_nl).collect();
                        let mut out = String::new();
                        for line in complete.split_inclusive(|b| *b == b'\n') {
                            if let Ok(text) = std::str::from_utf8(line) {
                                out.push_str(&st.converter.feed_line(text));
                            }
                        }
                        if !out.is_empty() {
                            return Some((Ok(Bytes::from(out)), st));
                        }
                    }
                    Some(Err(e)) => {
                        st.finished = true;
                        return Some((Err(std::io::Error::other(e)), st));
                    }
                    None => {
                        st.finished = true;
                        let mut out = String::new();
                        if !st.buffer.is_empty() {
                            let rest = std::mem::take(&mut st.buffer);
                            if let Ok(text) = std::str::from_utf8(&rest) {
                                out.push_str(&st.converter.feed_line(text));
                            }
                        }
                        if !st.converter.done {
                            out.push_str(&st.converter.feed_line("data: [DONE]"));
                        }
                        return if out.is_empty() { None } else { Some((Ok(Bytes::from(out)), st)) };
                    }
                }
            }
        },
    )
}

pub(crate) async fn respond_as_chat(upstream: reqwest::Response, tier: &str, want_stream: bool) -> Result<Response, AppError> {
    let status = upstream.status();
    if !status.is_success() {
        let text = upstream.text().await.unwrap_or_default();
        tracing::warn!("上游返回 {status}: {}", text.chars().take(300).collect::<String>());
        return Err(map_upstream_error(status, &text));
    }

    if want_stream {
        let stream = adapt_stream(Box::pin(upstream.bytes_stream()), tier.to_string());
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-store")
            .header("x-accel-buffering", "no")
            .body(Body::from_stream(stream))
            .map_err(|e| AppError::Internal(e.to_string()));
    }

    let bytes = upstream.bytes().await.map_err(|_| AppError::BadGateway("AI 服务暂时不可用".into()))?;
    let mut converter = ResponsesToChat::new(tier.to_string());
    if let Ok(json_body) = serde_json::from_slice::<Value>(&bytes) {
        converter.on_event(&json_body);
    } else {
        let text = std::str::from_utf8(&bytes).map_err(|_| AppError::BadGateway("AI 服务返回了无法识别的内容".into()))?;
        for line in text.lines() {
            converter.feed_line(line);
        }
    }
    let reply = converter.into_non_streaming()?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(reply)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_chat_messages_tools_and_reasoning_into_responses_payload() {
        let chat = json!({
            "model": "gpt-6.1-terra",
            "stream": true,
            "reasoning_effort": "high",
            "temperature": 0.7,
            "max_tokens": 4096,
            "messages": [
                { "role": "system", "content": "You are helpful." },
                { "role": "user", "content": [
                    { "type": "text", "text": "Look at this" },
                    { "type": "image_url", "image_url": { "url": "https://example.com/a.png", "detail": "high" } }
                ]},
                { "role": "assistant", "content": null, "tool_calls": [
                    { "id": "call_1", "type": "function", "function": { "name": "read_file", "arguments": "{\"path\":\"a.txt\"}" } }
                ]},
                { "role": "tool", "tool_call_id": "call_1", "content": "hello world" }
            ],
            "tools": [
                { "type": "function", "function": { "name": "read_file", "description": "Read a file", "parameters": { "type": "object" } } }
            ],
            "tool_choice": "auto",
            "parallel_tool_calls": false
        });
        let (bytes, want_stream) = chat_to_responses(&chat).unwrap();
        assert!(want_stream);
        let out: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(out["model"], "gpt-6.1-terra");
        assert_eq!(out["stream"], true);
        assert_eq!(out["store"], false);
        assert!(out.get("temperature").is_none() && out.get("max_tokens").is_none() && out.get("max_output_tokens").is_none());
        assert_eq!(out["reasoning"], json!({ "effort": "high", "summary": "auto" }));
        assert_eq!(out["input"][0], json!({ "role": "system", "content": "You are helpful." }));
        assert_eq!(out["input"][1]["content"][1], json!({ "type": "input_image", "image_url": "https://example.com/a.png", "detail": "high" }));
        assert_eq!(out["input"][2], json!({ "type": "function_call", "call_id": "call_1", "name": "read_file", "arguments": "{\"path\":\"a.txt\"}" }));
        assert_eq!(out["input"][3], json!({ "type": "function_call_output", "call_id": "call_1", "output": "hello world" }));
        assert_eq!(out["tools"][0], json!({ "type": "function", "name": "read_file", "description": "Read a file", "parameters": { "type": "object" }, "strict": false }));
        assert_eq!(out["tool_choice"], "auto");
        assert_eq!(out["parallel_tool_calls"], false);
    }

    #[test]
    fn bridges_responses_sse_with_reasoning_and_dense_tool_indices_without_leaking_model() {
        let mut conv = ResponsesToChat::new("gpt-6-1-terra".into());
        let events = [
            json!({ "type": "response.created", "response": { "id": "resp_abc", "created_at": 1700000000, "model": "gpt-6.1-terra-internal" } }),
            json!({ "type": "response.reasoning_summary_text.delta", "output_index": 0, "summary_index": 0, "delta": "thinking..." }),
            json!({ "type": "response.output_item.added", "output_index": 1, "item": { "id": "fc_1", "type": "function_call", "call_id": "call_99", "name": "get_weather", "arguments": "" } }),
            json!({ "type": "response.function_call_arguments.delta", "output_index": 1, "delta": "{\"city\":" }),
            json!({ "type": "response.function_call_arguments.delta", "output_index": 1, "delta": "\"Shenzhen\"}" }),
            json!({
                "type": "response.completed",
                "response": {
                    "id": "resp_abc",
                    "model": "gpt-6.1-terra-internal",
                    "usage": { "input_tokens": 120, "output_tokens": 30, "total_tokens": 150, "input_tokens_details": { "cached_tokens": 64 } },
                    "output": [
                        { "type": "reasoning", "summary": [{ "type": "summary_text", "text": "thinking..." }] },
                        { "type": "function_call", "call_id": "call_99", "name": "get_weather", "arguments": "{\"city\":\"Shenzhen\"}" }
                    ]
                }
            }),
        ];
        let mut sse = String::new();
        for ev in &events {
            sse.push_str(&conv.feed_line(&format!("data: {ev}")));
        }
        assert!(!sse.contains("gpt-6.1-terra-internal"), "upstream model leaked: {sse}");
        assert!(sse.contains("\"model\":\"gpt-6-1-terra\""));
        assert!(sse.contains("\"index\":0,\"type\":\"function\"") || sse.contains("\"id\":\"call_99\",\"index\":0"));
        assert!(sse.contains("\"finish_reason\":\"tool_calls\""));
        assert!(sse.ends_with("data: [DONE]\n\n"));

        let single = conv.into_non_streaming().unwrap();
        assert_eq!(single["model"], "gpt-6-1-terra");
        assert_eq!(single["choices"][0]["finish_reason"], "tool_calls");
        assert_eq!(single["choices"][0]["message"]["content"], Value::Null);
        assert_eq!(single["choices"][0]["message"]["reasoning_content"], "thinking...");
        assert_eq!(single["choices"][0]["message"]["tool_calls"][0]["id"], "call_99");
        assert_eq!(single["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"], "{\"city\":\"Shenzhen\"}");
        assert_eq!(single["usage"]["prompt_tokens"], 120);
        assert_eq!(single["usage"]["prompt_tokens_details"]["cached_tokens"], 64);
    }

    #[test]
    fn deduplicates_streamed_text_against_completed_snapshot_and_sanitizes_errors() {
        let mut conv = ResponsesToChat::new("gpt-6-1-terra".into());
        let mut sse = String::new();
        sse.push_str(&conv.feed_line(&format!("data: {}", json!({ "type": "response.output_text.delta", "output_index": 0, "content_index": 0, "delta": "你好" }))));
        sse.push_str(&conv.feed_line(&format!("data: {}", json!({ "type": "response.output_text.done", "output_index": 0, "content_index": 0, "text": "你好世界" }))));
        sse.push_str(&conv.feed_line(&format!("data: {}", json!({
            "type": "response.completed",
            "response": { "id": "r1", "model": "secret-codex", "output": [{ "type": "message", "content": [{ "type": "output_text", "text": "你好世界" }] }] }
        }))));
        let single = conv.into_non_streaming().unwrap();
        assert_eq!(single["choices"][0]["message"]["content"], "你好世界");
        assert!(!sse.contains("secret-codex"));

        let mut failed = ResponsesToChat::new("gpt-6-1-terra".into());
        let err_sse = failed.feed_line(&format!("data: {}", json!({ "type": "error", "error": { "message": "channel 7 secret token expired" } })));
        assert!(!err_sse.contains("channel 7") && err_sse.contains("bad_gateway"));
        assert!(failed.into_non_streaming().is_err());
    }

    #[tokio::test]
    async fn respond_as_chat_streams_and_collects_over_http_without_exposing_upstream_model() {
        use axum::{body::to_bytes, routing::post, Router};

        let sse_body = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_live\",\"created_at\":1700000001,\"model\":\"gpt-6.1-terra\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"output_index\":0,\"content_index\":0,\"delta\":\"梧桐山\"}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"output_index\":0,\"content_index\":0,\"delta\":\"秋色\"}\n\n",
            "event: response.completed\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_live\",\"model\":\"gpt-6.1-terra\",\"usage\":{\"input_tokens\":10,\"output_tokens\":4,\"total_tokens\":14},\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"梧桐山秋色\"}]}]}}\n\n",
            "data: [DONE]\n\n"
        );
        let app = Router::new().route(
            "/v1/responses",
            post(move || async move { ([(header::CONTENT_TYPE, "text/event-stream")], sse_body) }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let client = reqwest::Client::builder().no_proxy().build().unwrap();

        // 1. Streaming mode
        let up_stream = client.post(format!("http://{addr}/v1/responses")).send().await.unwrap();
        let chat_stream = respond_as_chat(up_stream, "gpt-6-1-terra", true).await.unwrap();
        assert_eq!(chat_stream.status(), StatusCode::OK);
        assert_eq!(chat_stream.headers()[header::CONTENT_TYPE], "text/event-stream");
        let raw = String::from_utf8(to_bytes(chat_stream.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
        assert!(!raw.contains("gpt-6.1-terra"), "leaked upstream model: {raw}");
        assert!(raw.contains("\"model\":\"gpt-6-1-terra\""));
        assert!(raw.contains("\"content\":\"梧桐山\""));
        assert!(raw.contains("\"content\":\"秋色\""));
        assert!(raw.ends_with("data: [DONE]\n\n"));

        // 2. Non-streaming mode
        let up_json = client.post(format!("http://{addr}/v1/responses")).send().await.unwrap();
        let chat_json = respond_as_chat(up_json, "gpt-6-1-terra", false).await.unwrap();
        assert_eq!(chat_json.status(), StatusCode::OK);
        let parsed: Value = serde_json::from_slice(&to_bytes(chat_json.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(parsed["model"], "gpt-6-1-terra");
        assert_eq!(parsed["choices"][0]["message"]["content"], "梧桐山秋色");
        assert_eq!(parsed["choices"][0]["finish_reason"], "stop");
        assert_eq!(parsed["usage"]["total_tokens"], 14);
    }
}

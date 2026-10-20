use super::{wire::ProviderProfile, ChatRequest};
use crate::{HarnessError, Part, Role};
use serde_json::Value;

/// Enforce the limit on the actual protocol body, including schemas, JSON escaping and media.
/// Keep the saved history intact, all user images, and the newest tool screenshot result.
pub(crate) fn encode(
    profile: &ProviderProfile,
    request: &ChatRequest,
    build: impl Fn(&ProviderProfile, &ChatRequest) -> Value,
) -> Result<Vec<u8>, HarnessError> {
    let serialize = |request: &ChatRequest| serde_json::to_vec(&build(profile, request))
        .map_err(|e| HarnessError::provider(format!("encode request: {e}"), false));
    let mut body = serialize(request)?;
    let Some(limit) = profile.max_request_bytes else { return Ok(body) };
    if body.len() <= limit { return Ok(body); }
    let mut reduced = request.clone();
    let latest = reduced.messages.iter().rposition(|m| m.role == Role::Tool && m.parts.iter().any(|p| matches!(p, Part::Image { .. })));
    for index in 0..latest.unwrap_or(0) {
        let message = &mut reduced.messages[index];
        if message.role != Role::Tool || !message.parts.iter().any(|p| matches!(p, Part::Image { .. })) { continue; }
        message.parts.retain(|p| !matches!(p, Part::Image { .. }));
        message.parts.push(Part::Text { text: "\n[Older tool images omitted from this request to fit the payload limit; capture a fresh screenshot if needed.]".into() });
        body = serialize(&reduced)?;
        if body.len() <= limit { return Ok(body); }
    }
    Err(HarnessError::provider(format!(
        "request_too_large: 请求大小为 {} 字节，上限为 {limit} 字节。旧工具截图已按需省略；请减少当前图片、降低截图尺寸，或压缩会话后继续。", body.len()
    ), false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, ToolCall};
    use super::super::{anthropic, responses, wire};
    use serde_json::json;

    #[test]
    fn all_protocols_trim_old_tool_images_to_the_actual_byte_limit_without_changing_history() {
        let mut profile = ProviderProfile::new("http://local", "", "test");
        profile.max_request_bytes = Some(8 * 1024 * 1024);
        let image = format!("data:image/png;base64,{}", "A".repeat(4_500_000));
        let mut messages = vec![Message::user_text("inspect")];
        for id in ["old", "new"] {
            messages.push(Message::assistant("", vec![ToolCall { id: id.into(), name: "screen".into(), arguments: "{}".into() }]));
            let mut output = Message::tool(id, "screenshot");
            output.parts.push(Part::Image { url: image.clone() });
            messages.push(output);
        }
        let request = ChatRequest { messages, tools: vec![] };
        for build in [wire::build_request_body, responses::build_request_body, anthropic::build_request_body] {
            let raw = encode(&profile, &request, build).unwrap();
            assert!(raw.len() <= profile.max_request_bytes.unwrap());
            let text = String::from_utf8(raw).unwrap();
            assert_eq!(text.matches(&image[22..]).count(), 1);
            assert!(text.contains("Older tool images omitted"));
            assert!(text.contains("old") && text.contains("new"));
        }
        assert_eq!(request.messages.iter().flat_map(|m| &m.parts).filter(|p| matches!(p, Part::Image { .. })).count(), 2);
    }

    #[test]
    fn current_user_images_and_newest_tool_images_are_never_silently_dropped() {
        let mut profile = ProviderProfile::new("http://local", "", "test");
        profile.max_request_bytes = Some(1_000);
        let image = Part::Image { url: format!("data:image/png;base64,{}", "A".repeat(2_000)) };
        for message in [Message::user(vec![image.clone()]), Message { parts: vec![image], ..Message::tool("c", "") }] {
            let request = ChatRequest { messages: vec![message], tools: vec![] };
            assert!(encode(&profile, &request, responses::build_request_body).unwrap_err().to_string().contains("request_too_large"));
        }
    }

    #[test]
    fn escaping_and_tool_schemas_count_toward_the_body_budget() {
        let mut profile = ProviderProfile::new("http://local", "", "test");
        let request = ChatRequest { messages: vec![Message::user_text("\\\"".repeat(100))], tools: vec![super::super::ToolSchema { name: "t".into(), description: "long".repeat(100), parameters: json!({"type":"object"}) }] };
        let bytes = encode(&profile, &request, responses::build_request_body).unwrap();
        profile.max_request_bytes = Some(bytes.len());
        assert_eq!(encode(&profile, &request, responses::build_request_body).unwrap(), bytes);
        profile.max_request_bytes = Some(bytes.len() - 1);
        assert!(encode(&profile, &request, responses::build_request_body).is_err());
    }
}

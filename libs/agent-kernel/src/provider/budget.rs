use super::{wire::ProviderProfile, ChatRequest};
use crate::{HarnessError, Part, Role};
use serde_json::Value;

/// Enforce the limit on the actual protocol body, including schemas, JSON escaping and media.
/// Only edit the request copy. Protect the latest real user message and two latest tool image results.
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
    let images: Vec<_> = reduced.messages.iter().enumerate()
        .filter(|(_, m)| matches!(m.role, Role::User | Role::Tool))
        .flat_map(|(mi, m)| m.parts.iter().enumerate().filter_map(move |(pi, p)|
            matches!(p, Part::Image { .. }).then_some((mi, pi))))
        .collect();
    let current = reduced.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic);
    let latest_tools: Vec<_> = reduced.messages.iter().enumerate().rev().filter(|(_, m)| m.role == Role::Tool && m.parts.iter().any(|p| matches!(p, Part::Image { .. }))).take(2).map(|(mi, _)| mi).collect();
    let last_image = images.last().copied();
    let mut old: Vec<_> = images.iter().copied().filter(|&(mi, pi)|
        Some((mi, pi)) != last_image && match reduced.messages[mi].role {
            Role::Tool => !latest_tools.contains(&mi),
            Role::User => Some(mi) != current,
            _ => false,
        }).collect();
    // Older tool screenshots are less likely to be important than user-provided images.
    old.sort_by_key(|&(mi, pi)| (reduced.messages[mi].role != Role::Tool, mi, pi));
    for (mi, pi) in old {
        let Part::Image { url } = &reduced.messages[mi].parts[pi] else { continue };
        let bytes = url.split_once(',').filter(|(header, _)| header.ends_with(";base64"))
            .map(|(_, data)| (data.len().saturating_mul(3) / 4).saturating_sub(data.bytes().rev().take_while(|b| *b == b'=').count().min(2)));
        let size = bytes.map(|bytes| format!("：原约 {bytes} 字节")).unwrap_or_default();
        reduced.messages[mi].parts[pi] = Part::Text { text: format!("[图片已省略{size}；如需查看请重新提供或读取。]") };
        body = serialize(&reduced)?;
        if body.len() <= limit { return Ok(body); }
    }
    // Downsample remaining images without removing any current-turn images. Multiple
    // passes bound CPU work and keep the largest useful version that fits.
    #[cfg(feature = "image-payload")]
    for (edge, target) in [(1500, 1024 * 1024), (1024, 512 * 1024), (768, 256 * 1024)] {
        let mut changed = false;
        for &(mi, pi) in &images {
            let Part::Image { url } = &reduced.messages[mi].parts[pi] else { continue };
            if let Ok(Some(smaller)) = crate::image_payload::optimize_data_url(url, edge, target) {
                reduced.messages[mi].parts[pi] = Part::Image { url: smaller };
                changed = true;
            }
        }
        if changed {
            body = serialize(&reduced)?;
            if body.len() <= limit { return Ok(body); }
        }
    }
    Err(HarnessError::provider(format!(
        "request_too_large: 请求大小为 {} 字节，上限为 {limit} 字节。旧图片已按需省略；请减少当前图片、降低图片尺寸，或新开会话后继续。", body.len()
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
        let image = format!("data:image/png;base64,{}", "A".repeat(3_000_000));
        let mut messages = vec![Message::user_text("inspect")];
        for id in ["old", "middle", "new"] {
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
            assert_eq!(text.matches(&image[22..]).count(), 2);
            assert!(text.contains("图片已省略"));
            assert!(text.contains("old") && text.contains("new"));
        }
        assert_eq!(request.messages.iter().flat_map(|m| &m.parts).filter(|p| matches!(p, Part::Image { .. })).count(), 3);
    }

    #[test]
    fn sole_user_or_tool_image_is_not_silently_dropped() {
        let mut profile = ProviderProfile::new("http://local", "", "test");
        profile.max_request_bytes = Some(1_000);
        let image = Part::Image { url: format!("data:image/png;base64,{}", "A".repeat(2_000)) };
        for message in [Message::user(vec![image.clone()]), Message { parts: vec![image], ..Message::tool("c", "") }] {
            let request = ChatRequest { messages: vec![message], tools: vec![] };
            assert!(encode(&profile, &request, responses::build_request_body).unwrap_err().to_string().contains("request_too_large"));
        }
    }

    #[test]
    fn old_user_and_tool_images_are_omitted_oldest_first_across_protocols() {
        let mut profile = ProviderProfile::new("http://local", "", "test");
        profile.max_request_bytes = Some(8 * 1024 * 1024);
        let image = |ch: char, size: usize| format!("data:image/png;base64,{}", ch.to_string().repeat(size));
        let old = image('A', 5_600_000);
        let middle = image('B', 2_200_000);
        let recent = image('C', 1_400_000);
        let tool = image('D', 2_200_000);
        let mut result = Message::tool("shot", "screenshot");
        result.parts.push(Part::Image { url: tool.clone() });
        let request = ChatRequest { messages: vec![Message::user(vec![Part::Image { url: old.clone() }]),
            Message::user(vec![Part::Image { url: middle.clone() }, Part::Image { url: recent.clone() }]),
            Message::assistant("", vec![ToolCall { id: "shot".into(), name: "screen".into(), arguments: "{}".into() }]), result], tools: vec![] };
        for build in [wire::build_request_body, responses::build_request_body, anthropic::build_request_body] {
            assert!(serde_json::to_vec(&build(&profile, &request)).unwrap().len() > profile.max_request_bytes.unwrap());
            let raw = encode(&profile, &request, build).unwrap();
            assert!(raw.len() <= profile.max_request_bytes.unwrap());
            let text = String::from_utf8(raw).unwrap();
            assert!(!text.contains(&old[22..]));
            assert!(text.contains(&middle[22..]) && text.contains(&recent[22..]) && text.contains(&tool[22..]));
            assert!(text.contains("图片已省略"));
        }
        assert_eq!(request.messages.iter().flat_map(|m| &m.parts).filter(|p| matches!(p, Part::Image { .. })).count(), 4);
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

#[cfg(all(test, feature = "image-payload"))]
mod image_recovery_tests {
    use super::*;
    use crate::{Message, Part};
    use super::super::{anthropic, responses, wire};
    use base64::{engine::general_purpose::STANDARD, Engine};
    use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
    use std::io::Cursor;

    fn noisy_png() -> String {
        let mut state: u32 = 0x6a5b4c3d;
        let mut pixels = Vec::with_capacity(1200 * 1200 * 3);
        for _ in 0..1200 * 1200 * 3 {
            state ^= state << 13; state ^= state >> 17; state ^= state << 5;
            pixels.push(state as u8);
        }
        let image = DynamicImage::ImageRgb8(ImageBuffer::<Rgb<u8>, _>::from_raw(1200, 1200, pixels).unwrap());
        let mut data = Cursor::new(Vec::new());
        image.write_to(&mut data, ImageFormat::Png).unwrap();
        format!("data:image/png;base64,{}", STANDARD.encode(data.into_inner()))
    }
    fn count_images(v: &serde_json::Value) -> usize {
        match v {
            serde_json::Value::Object(o) => usize::from(matches!(o.get("type").and_then(|v| v.as_str()), Some("image" | "image_url" | "input_image"))) + o.values().map(count_images).sum::<usize>(),
            serde_json::Value::Array(a) => a.iter().map(count_images).sum(),
            _ => 0,
        }
    }
    #[test]
    fn current_multi_image_turn_is_resized_not_omitted_for_all_protocols() {
        let original = noisy_png();
        let request = ChatRequest { messages: vec![Message::user(vec![Part::Text { text: "Compare these images".into() },
            Part::Image { url: original.clone() }, Part::Image { url: original.clone() }, Part::Image { url: original.clone() }])], tools: vec![] };
        let mut profile = ProviderProfile::new("http://local", "", "test");
        profile.max_request_bytes = Some(8 * 1024 * 1024);
        for build in [wire::build_request_body, responses::build_request_body, anthropic::build_request_body] {
            assert!(serde_json::to_vec(&build(&profile, &request)).unwrap().len() > profile.max_request_bytes.unwrap());
            let encoded = encode(&profile, &request, build).unwrap();
            assert!(encoded.len() <= profile.max_request_bytes.unwrap());
            let body: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(count_images(&body), 3);
            assert!(!String::from_utf8(encoded).unwrap().contains("图片已省略"));
        }
        assert_eq!(request.messages[0].parts[1], Part::Image { url: original });
    }
}

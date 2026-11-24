//! Drop byte-identical desktop observations from the model-facing request only.
use agent_kernel::{HarnessError, Middleware, Part, Role, Turn};
use std::collections::HashMap;

pub struct DesktopObservationHistory;

impl Middleware for DesktopObservationHistory {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let mut apps = HashMap::new();
        let mut observations = Vec::new();
        let mut latest = HashMap::new();
        for (index, message) in turn.messages.iter().enumerate() {
            for call in &message.tool_calls {
                if matches!(
                    call.name.as_str(),
                    "get_app_state"
                        | "click"
                        | "drag"
                        | "set_value"
                        | "type_text"
                        | "scroll"
                        | "press_key"
                        | "perform_secondary_action"
                        | "select_text"
                ) {
                    if let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.arguments) {
                        if let Some(app) = args["app"].as_str() {
                            apps.insert(call.id.as_str(), app.to_owned());
                        }
                    }
                } else {
                    apps.remove(call.id.as_str());
                }
            }
            if message.role != Role::Tool || message.is_error {
                continue;
            }
            let Some(app) = message
                .tool_call_id
                .as_deref()
                .and_then(|id| apps.remove(id))
            else {
                continue;
            };
            if !message.parts.iter().any(|p| matches!(p,Part::Text{text} if text.starts_with("App=") || text.starts_with("已顺序派发 "))) { continue; }
            latest.insert(app.clone(), index);
            observations.push((index, app));
        }

        // Keep each distinct tree/frame, plus the latest observation per application
        // in full. Compare bytes, including image data; matching titles alone mean nothing.
        let mut first = HashMap::new();
        let mut repeats = Vec::new();
        for (index, app) in &observations {
            let message = &turn.messages[*index];
            for (part, value) in message.parts.iter().enumerate() {
                let (image, payload) = match value {
                    Part::Image { url } if url.starts_with("data:image/") => (true, url.as_str()),
                    Part::Text { text } if text.len() >= 512 => (false, text.as_str()),
                    _ => continue,
                };
                let source = first
                    .entry((app.as_str(), image, payload))
                    .or_insert(*index);
                if *source != *index && latest.get(app) != Some(index) {
                    repeats.push((*index, part, *source, image));
                }
            }
        }
        for (index, part, source, image) in repeats {
            let id = turn.messages[source]
                .tool_call_id
                .as_deref()
                .unwrap_or_default();
            let kind = if image {
                "截图"
            } else {
                "窗口状态文本"
            };
            turn.messages[index].parts[part] = Part::Text {
                text: format!("此{kind}与较早工具调用 {id} 返回的内容完全相同。"),
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::{Message, ToolCall};
    #[test]
    fn only_identical_desktop_parts_are_collapsed_in_the_request() {
        let frame = |k: &str| Part::Image {
            url: format!("data:image/png;base64,{k}"),
        };
        let mut msgs = vec![Message::user(vec![frame("A")])];
        for (i, (tool, app, im)) in [
            ("click", "lab", "A"),
            ("click", "lab", "A"),
            ("click", "lab", "B"),
            ("click", "other", "A"),
            ("other_tool", "lab", "A"),
            ("click", "lab", "A"),
        ]
        .into_iter()
        .enumerate()
        {
            let id = format!("c{i}");
            msgs.push(Message::assistant(
                "",
                vec![ToolCall {
                    id: id.clone(),
                    name: tool.into(),
                    arguments: serde_json::json!({"app":app}).to_string(),
                }],
            ));
            let mut m = Message::tool(id, format!("App={app}\n{}", "x".repeat(520)));
            m.parts.push(frame(im));
            msgs.push(m);
        }
        let mut turn = Turn {
            index: 0,
            session: "s".into(),
            tools: vec![],
            messages: msgs.clone(),
        };
        DesktopObservationHistory.before_model(&mut turn).unwrap();
        assert_eq!(turn.messages[0], msgs[0]); // user's original attachment
        assert_eq!(turn.messages[2], msgs[2]); // first matching state
        assert!(turn.messages[4]
            .parts
            .iter()
            .all(|p| matches!(p,Part::Text{text} if text.contains("c0"))));
        assert_eq!(turn.messages[6].parts[1], frame("B"));
        assert_eq!(turn.messages[8], msgs[8]); // other application
        assert_eq!(turn.messages[10], msgs[10]); // unrelated tool
        assert_eq!(turn.messages[12], msgs[12]); // latest state
    }
}

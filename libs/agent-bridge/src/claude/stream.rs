use crate::{AgentEvent, Usage};
use serde_json::Value;
fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}
pub(super) fn parse(v: &Value) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    match v.get("type").and_then(Value::as_str).unwrap_or("") {
        "stream_event" => {
            if let Some(t) = v.pointer("/event/delta/text").and_then(Value::as_str) {
                events.push(AgentEvent::TextDelta { text: t.into() });
            }
        }
        "assistant" => {
            if let Some(content) = v.pointer("/message/content").and_then(Value::as_array) {
                for item in content {
                    match item.get("type").and_then(Value::as_str).unwrap_or("") {
                        "text" => events.push(AgentEvent::Message {
                            text: text(item, "text"),
                        }),
                        "tool_use" => events.push(AgentEvent::ToolStarted {
                            id: text(item, "id"),
                            name: text(item, "name"),
                            summary: item.get("input").map(Value::to_string).unwrap_or_default(),
                        }),
                        _ => {}
                    }
                }
            }
        }
        "user" => {
            if let Some(content) = v.pointer("/message/content").and_then(Value::as_array) {
                for item in content {
                    if item.get("type").and_then(Value::as_str) == Some("tool_result") {
                        let preview = item
                            .get("content")
                            .map(|v| {
                                v.as_str()
                                    .map(str::to_owned)
                                    .unwrap_or_else(|| v.to_string())
                            })
                            .unwrap_or_default()
                            .chars()
                            .take(2000)
                            .collect();
                        events.push(AgentEvent::ToolFinished {
                            id: text(item, "tool_use_id"),
                            ok: !item
                                .get("is_error")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                            preview,
                        });
                    }
                }
            }
        }
        "result" => {
            if v.get("is_error").and_then(Value::as_bool).unwrap_or(false) {
                events.push(AgentEvent::Error {
                    code: "agent".into(),
                    message: "Agent reported a turn failure".into(),
                });
            }
            events.push(AgentEvent::TurnCompleted {
                final_text: v.get("result").and_then(Value::as_str).map(str::to_owned),
                usage: v.get("usage").map(|u| Usage {
                    input_tokens: u.get("input_tokens").and_then(Value::as_u64),
                    output_tokens: u.get("output_tokens").and_then(Value::as_u64),
                }),
            });
        }
        _ => {}
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_maps_known_events_and_truncates_tool_output() {
        let events: Vec<_> = include_str!("../../tests/fixtures/claude-stream.jsonl")
            .lines()
            .flat_map(|line| parse(&serde_json::from_str::<Value>(line).unwrap()))
            .collect();
        assert!(matches!(&events[0],AgentEvent::TextDelta{text} if text=="hello"));
        assert!(matches!(&events[1],AgentEvent::Message{text} if text=="reply"));
        assert!(matches!(&events[2],AgentEvent::ToolStarted{id,..} if id=="t1"));
        assert!(matches!(
            &events[3],
            AgentEvent::ToolFinished { ok: true, .. }
        ));
        assert!(
            matches!(&events[4],AgentEvent::TurnCompleted{usage:Some(u),..} if u.output_tokens==Some(4))
        );
        let huge = serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"字".repeat(2500)}]}});
        assert!(
            matches!(&parse(&huge)[0],AgentEvent::ToolFinished{preview,..} if preview.chars().count()==2000)
        );
    }
}

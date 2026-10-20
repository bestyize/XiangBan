//! Persist sequenced kernel execution events before forwarding them to the existing chat UI.
//! RunState::Completed is not evidence of business task success.

use agent_kernel::journal::JsonlRunLog;
use agent_kernel::{Event, HarnessError, RunEvent, RunEventBody};
use serde_json::{json, Value};

fn redact_key(value: &mut Value, key: &str) {
    if key.is_empty() { return; }
    match value {
        Value::String(s) => { if s.contains(key) { *s = s.replace(key, "<hidden>"); } }
        Value::Array(xs) => for x in xs { redact_key(x, key); },
        Value::Object(xs) => for x in xs.values_mut() { redact_key(x, key); },
        _ => {}
    }
}

pub(super) fn forward(
    log: &JsonlRunLog, record: RunEvent, api_key: &str,
    mut notify: impl FnMut(&str, Value),
) -> Result<(), HarnessError> {
    let record = if api_key.is_empty() {
        record
    } else {
        let mut value = serde_json::to_value(&record)
            .map_err(|e| HarnessError::Store(format!("serialize run event: {e}")))?;
        redact_key(&mut value, api_key);
        serde_json::from_value(value)
            .map_err(|e| HarnessError::Store(format!("deserialize run event: {e}")))?
    };
    log.append(&record)?;
    let RunEvent { run_id, session_id, sequence, timestamp_ms, body, .. } = record;
    match body {
        RunEventBody::State { state, reason } => notify("chat-run-state", json!({
            "session": session_id, "runId": run_id, "sequence": sequence,
            "timestampMs": timestamp_ms, "state": state, "reason": reason
        })),
        RunEventBody::Event { event } => {
            if !matches!(event, Event::ToolDispatched { .. }) {
                notify("chat-event", json!({
                    "session": session_id, "runId": run_id, "sequence": sequence,
                    "timestampMs": timestamp_ms, "event": event
                }));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::RunState;

    #[test]
    fn run_events_are_redacted_and_journaled_in_order() {
        let dir = std::env::temp_dir().join(format!("xiaoban-run-history-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let journal = JsonlRunLog::new(&dir);
        let mut seen = Vec::new();
        let records = [
            RunEvent { run_id: "r-test".into(), session_id: "s1".into(), parent_run_id: None,
                sequence: 1, timestamp_ms: 100,
                body: RunEventBody::State { state: RunState::Running, reason: None } },
            RunEvent { run_id: "r-test".into(), session_id: "s1".into(), parent_run_id: None,
                sequence: 2, timestamp_ms: 101,
                body: RunEventBody::Event { event: Event::Error { message: "secret-value".into() } } },
            RunEvent { run_id: "r-test".into(), session_id: "s1".into(), parent_run_id: None,
                sequence: 3, timestamp_ms: 102,
                body: RunEventBody::State { state: RunState::Failed, reason: None } },
        ];
        for record in records {
            forward(&journal, record, "secret-value", |kind, data| seen.push((kind.to_string(), data))).unwrap();
        }
        assert_eq!(seen[1].1["event"]["message"], "<hidden>");
        let stored = journal.read_after("r-test", 0, 100).unwrap();
        assert_eq!(stored.len(), 3);
        assert_eq!(stored[1].sequence, 2);
        assert!(!format!("{stored:?}").contains("secret-value"));
        let status = journal.inspect("r-test").unwrap();
        assert!(status.terminal);
        assert_eq!(status.state, Some(RunState::Failed));
        let _ = std::fs::remove_dir_all(dir);
    }
}

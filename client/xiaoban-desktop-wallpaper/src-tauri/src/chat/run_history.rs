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
    forward_observed(log, record, api_key, |_| Ok(()), notify)
}

pub(super) fn forward_observed(
    log: &JsonlRunLog, record: RunEvent, api_key: &str,
    before_notify: impl FnOnce(&RunEvent) -> Result<(), HarnessError>,
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
    before_notify(&record)?;
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

/// Recover only displayable text from the final model attempt. Tool dispatch boundaries
/// prove the preceding assistant message was already saved; never recreate tool calls,
/// signatures or encrypted state from a partially written run log.
#[derive(Default)]
struct VisibleTail {
    text: String,
    reasoning: Vec<agent_kernel::ReasoningSegment>,
}

impl VisibleTail {
    fn push(&mut self, event: &Event) {
        match event {
            Event::TextDelta { text } => self.text.push_str(text),
            Event::ReasoningDelta { kind, text } => {
                if let Some(last) = self.reasoning.last_mut().filter(|part| part.kind == *kind) {
                    last.text.push_str(text);
                } else if !text.is_empty() {
                    self.reasoning.push(agent_kernel::ReasoningSegment { kind: *kind, text: text.clone() });
                }
            }
            Event::UserMessage { .. } | Event::ToolStart { .. } | Event::ToolEnd { .. } | Event::Continuing { .. } | Event::Done => {
                *self = Self::default();
            }
            _ => {}
        }
    }

    fn message(self, saved: &[agent_kernel::Message]) -> Option<agent_kernel::Message> {
        if self.text.is_empty() && self.reasoning.is_empty() { return None; }
        if saved.last().is_some_and(|last| last.role == agent_kernel::Role::Assistant
            && last.text() == self.text && last.reasoning == self.reasoning) {
            return None;
        }
        let mut message = agent_kernel::Message::assistant(self.text, Vec::new());
        message.reasoning = self.reasoning;
        message.structured_content = Some(json!({"interrupted": true}));
        Some(message)
    }
}

/// Build the final visible tail without inventing calls or continuation state.
/// The caller persists it once, while holding the session lease.
pub(super) fn recover_visible_tail(
    dir: &std::path::Path, session: &str, saved: &[agent_kernel::Message],
) -> Result<Option<agent_kernel::Message>, HarnessError> {
    let Some(run) = latest_run_id(dir, session)? else { return Ok(None); };
    let log = JsonlRunLog::new(dir);
    let mut tail = VisibleTail::default();
    log.visit(&run, |record| {
        if record.session_id != session || record.parent_run_id.is_some() { return; }
        if let RunEventBody::Event { event } = record.body { tail.push(&event); }
    })?;
    let already_saved = saved.iter().enumerate().any(|(i, message)| {
        message.structured_content.as_ref().and_then(|v| v["recovery_run_id"].as_str()) == Some(run.as_str())
            && saved.get(i + 1).and_then(|next| next.structured_content.as_ref())
                .and_then(|v| v["stream_recovery_run_id"].as_str()) != Some(run.as_str())
    });
    if already_saved { return Ok(None); }
    Ok(tail.message(saved).map(|mut message| {
        message.structured_content.as_mut().unwrap()["recovery_run_id"] = json!(run);
        message
    }))
}

/// Caller holds the session lease. Trace corruption must not hide valid session
/// history; only a failed write to the actual session store is fatal.
pub(super) fn persist_visible_tail(
    dir: &std::path::Path, session: &str, store: &impl agent_kernel::SessionStore,
) -> Result<Option<String>, HarnessError> {
    let saved = store.load(session)?;
    match recover_visible_tail(dir, session, &saved) {
        Ok(Some(message)) => { store.append(session, &[message])?; Ok(None) }
        Ok(None) => Ok(None),
        Err(_) => Ok(Some("运行日志无法恢复，已保留正常会话历史。".into())),
    }
}

/// Root runs only: research-child files share this directory and must never
/// replace the parent session's snapshot or recovery cursor.
pub(super) fn latest_run_id(dir: &std::path::Path, session: &str) -> Result<Option<String>, HarnessError> {
    use std::io::{BufRead, Read};
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(HarnessError::Store(error.to_string())),
    };
    let latest = entries.flatten().filter_map(|entry| {
        let name = entry.file_name().to_string_lossy().into_owned();
        let run = name.strip_prefix("run-")?.strip_suffix(".jsonl")?.to_owned();
        let file = std::fs::File::open(entry.path()).ok()?;
        let mut first = String::new();
        std::io::BufReader::new(file).take(64 * 1024).read_line(&mut first).ok()?;
        if !first.ends_with('\n') { return None; }
        let record: Value = serde_json::from_str(&first).ok()?;
        let event = &record["event"];
        if event["session_id"] != session || !event["parent_run_id"].is_null() || event["run_id"] != run { return None; }
        Some((entry.metadata().ok()?.modified().ok()?, run))
    }).max();
    Ok(latest.map(|(_, run)| run))
}

pub(super) fn latest_events(
    dir: &std::path::Path, session: &str,
) -> Result<(Option<String>, u64, Vec<Value>), HarnessError> {
    let Some(run) = latest_run_id(dir, session)? else { return Ok((None, 0, Vec::new())); };
    let mut sequence = 0;
    let mut events = Vec::new();
    JsonlRunLog::new(dir).visit(&run, |record| {
        if record.session_id != session || record.parent_run_id.is_some() { return; }
        sequence = record.sequence;
        if let RunEventBody::Event { event } = record.body {
            events.push(json!({"session":session,"runId":run,"sequence":record.sequence,"event":event}));
        }
    })?;
    Ok((Some(run), sequence, events))
}

pub(super) fn snapshot_projection(
    mut messages: Vec<agent_kernel::Message>, run_id: Option<String>, sequence: u64, mut events: Vec<Value>, running: bool,
) -> Value {
    let terminal = events.iter().any(|v| matches!(v["event"]["type"].as_str(), Some("done" | "cancelled" | "error")));
    let running = running && !terminal;
    let has_output = events.iter().any(|v| matches!(v["event"]["type"].as_str(),
        Some("user_message" | "text_delta" | "reasoning_delta" | "tool_start" | "done" | "cancelled" | "error")));
    // Model callbacks only begin after the user's message is saved. Prior to
    // that boundary the UI keeps waiting and retries on the first model event.
    let inputs: Vec<_> = events.iter().filter(|v|v["event"]["type"] == "user_message").collect();
    let id_of = |m: &agent_kernel::Message| m.structured_content.as_ref().and_then(|v|v["outboxId"].as_str()).map(str::to_string);
    let initial = inputs.iter().find(|v|v["event"]["initial"] == true).and_then(|v|v["event"]["message"]["structured_content"]["outboxId"].as_str());
    let boundary = initial.and_then(|id|messages.iter().position(|m|id_of(m).as_deref() == Some(id)))
        .or_else(|| messages.iter().rposition(|m|m.role == agent_kernel::Role::User && !m.synthetic && !inputs.iter().any(|v|id_of(m).as_deref().is_some_and(|id|v["event"]["message"]["structured_content"]["outboxId"] == id))).map(|i|i+1));
    // A long run may compact its original input after several follow-ups. Resume
    // projection at the earliest retained input instead of waiting for the run to end.
    let retained = if boundary.is_none() { messages.iter().enumerate().find_map(|(at,m)| {
        let id=id_of(m)?;
        events.iter().position(|v|v["event"]["type"] == "user_message" && v["event"]["message"]["structured_content"]["outboxId"] == id).map(|event|(at,event))
    }) } else {None};
    let boundary=boundary.or_else(||retained.map(|(at,_)|at));
    if let Some((_,event))=retained {events=events.into_iter().skip(event).collect();}
    if let Some(boundary) = has_output.then_some(boundary).flatten() {
        messages.truncate(boundary);
        let interrupted = !running && !events.iter().any(|v| matches!(v["event"]["type"].as_str(), Some("done" | "cancelled" | "error")));
        json!({"messages":messages,"runId":run_id,"sequence":sequence,"events":events,"running":running,"ready":true,"interrupted":interrupted})
    } else {
        json!({"messages":messages,"runId":run_id,"sequence":sequence,"events":[],"running":running,"ready":!running})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::RunState;

    #[test]
    fn snapshot_replays_follow_up_in_its_original_position() {
        let mut follow=agent_kernel::Message::user_text("change direction");follow.structured_content=Some(json!({"outboxId":"q1"}));
        let messages=vec![agent_kernel::Message::user_text("original"),agent_kernel::Message::assistant("first",vec![]),follow.clone(),agent_kernel::Message::assistant("second",vec![])];
        let events=vec![json!({"event":{"type":"text_delta","text":"first"}}),json!({"event":{"type":"user_message","initial":false,"message":follow}}),json!({"event":{"type":"text_delta","text":"second"}})];
        let projection=snapshot_projection(messages,Some("r1".into()),4,events,true);
        assert_eq!(projection["messages"].as_array().unwrap().len(),1);assert_eq!(projection["events"].as_array().unwrap().len(),3);
    }
    #[test]
    fn snapshot_stays_live_when_compaction_has_removed_the_original_input() {
        let mut follow=agent_kernel::Message::user_text("latest instruction");follow.structured_content=Some(json!({"outboxId":"q2"}));
        let summary=agent_kernel::Message::reminder("earlier work was compacted");
        let events=vec![json!({"event":{"type":"text_delta","text":"old output"}}),json!({"event":{"type":"user_message","initial":false,"message":follow}}),json!({"event":{"type":"text_delta","text":"new output"}})];
        let p=snapshot_projection(vec![summary,follow],Some("r1".into()),4,events,true);
        assert_eq!(p["messages"].as_array().unwrap().len(),1);assert_eq!(p["events"].as_array().unwrap().len(),2);assert_eq!(p["ready"],true);
    }
    #[test]
    fn snapshot_does_not_duplicate_the_initial_queued_input() {
        let mut initial=agent_kernel::Message::user_text("queued question");initial.structured_content=Some(json!({"outboxId":"q1"}));
        let events=vec![json!({"event":{"type":"user_message","initial":true,"message":initial}})];
        let projection=snapshot_projection(vec![initial],Some("r1".into()),2,events,true);
        assert_eq!(projection["messages"].as_array().unwrap().len(),0);assert!(projection["ready"].as_bool().unwrap());
    }
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

#[cfg(test)]
mod visible_recovery_tests {
    use super::*;
    use agent_kernel::{Message, ReasoningKind};

    #[test]
    fn failed_tail_is_visible_but_not_recovered_twice_or_as_tool_state() {
        let mut tail = VisibleTail::default();
        tail.push(&Event::ReasoningDelta { kind: ReasoningKind::Text, text: "checking".into() });
        tail.push(&Event::TextDelta { text: "answer prefix".into() });
        tail.push(&Event::Error { message: "network".into() });
        let recovered = tail.message(&[]).unwrap();
        assert_eq!(recovered.text(), "answer prefix");
        assert_eq!(recovered.reasoning[0].text, "checking");
        assert!(recovered.tool_calls.is_empty());
        assert!(recovered.provider_state.is_none());
        let mut duplicate = VisibleTail::default();
        duplicate.push(&Event::ReasoningDelta { kind: ReasoningKind::Text, text: "checking".into() });
        duplicate.push(&Event::TextDelta { text: "answer prefix".into() });
        assert!(duplicate.message(&[recovered]).is_none());
    }

    #[test]
    fn saved_tool_round_is_not_recovered_as_another_assistant_message() {
        let mut tail = VisibleTail::default();
        tail.push(&Event::TextDelta { text: "before tool".into() });
        tail.push(&Event::ToolStart { id: "t".into(), name: "lookup".into(), arguments: "{}".into() });
        tail.push(&Event::ReasoningDelta { kind: ReasoningKind::Summary, text: "next round".into() });
        let recovered = tail.message(&[Message::assistant("before tool", vec![])]).unwrap();
        assert!(recovered.text().is_empty());
        assert_eq!(recovered.reasoning[0].text, "next round");
        let mut done = VisibleTail::default();
        done.push(&Event::TextDelta { text: "complete".into() });
        done.push(&Event::Done);
        assert!(done.message(&[]).is_none());
    }

    #[test]
    fn recovery_reads_long_trace_and_ignores_a_partial_final_record() {
        let dir = std::env::temp_dir().join(format!("xiaoban-visible-recovery-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let journal = JsonlRunLog::new(&dir);
        for sequence in 1..=270 {
            journal.append(&RunEvent { run_id: "r".into(), session_id: "s".into(), parent_run_id: None,
                sequence, timestamp_ms: sequence,
                body: RunEventBody::Event { event: Event::TextDelta { text: "x".into() } } }).unwrap();
        }
        use std::io::Write;
        std::fs::OpenOptions::new().append(true).open(dir.join("run-r.jsonl")).unwrap().write_all(b"{unfinished").unwrap();
        let recovered = recover_visible_tail(&dir, "s", &[]).unwrap().unwrap();
        assert_eq!(recovered.text().len(), 270);
        assert!(recover_visible_tail(&dir, "other-session", &[]).unwrap().is_none());
        drop(journal);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod durable_recovery_tests {
    use super::*;
    use agent_kernel::{JsonlStore, Message, SessionStore, RunState};
    #[test]
    fn recovery_is_durable_idempotent_and_ignores_research_children() {
        let dir=std::env::temp_dir().join(format!("durable-chat-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let runs=dir.join("runs"); let store=JsonlStore::new(dir.join("sessions")); store.append("s",&[Message::user_text("question")]).unwrap();
        let log=JsonlRunLog::new(&runs);
        log.append(&RunEvent{run_id:"root".into(),session_id:"s".into(),parent_run_id:None,sequence:1,timestamp_ms:1,body:RunEventBody::Event{event:Event::TextDelta{text:"partial".into()}}}).unwrap();
        log.append(&RunEvent{run_id:"child".into(),session_id:"s#research".into(),parent_run_id:Some("root".into()),sequence:1,timestamp_ms:2,body:RunEventBody::State{state:RunState::Running,reason:None}}).unwrap();
        assert_eq!(latest_run_id(&runs,"s").unwrap().as_deref(),Some("root"));
        assert!(persist_visible_tail(&runs,"s",&store).unwrap().is_none());
        assert!(persist_visible_tail(&runs,"s",&store).unwrap().is_none());
        assert_eq!(store.load("s").unwrap().len(),2); assert_eq!(store.load("s").unwrap()[1].text(),"partial");
        log.append(&RunEvent{run_id:"new".into(),session_id:"s".into(),parent_run_id:None,sequence:1,timestamp_ms:3,body:RunEventBody::State{state:RunState::Running,reason:None}}).unwrap();
        use std::io::Write;
        std::fs::OpenOptions::new().append(true).open(runs.join("run-new.jsonl")).unwrap().write_all(b"{broken}\n").unwrap();
        assert!(persist_visible_tail(&runs,"s",&store).unwrap().is_some());
        assert_eq!(store.load("s").unwrap()[1].text(),"partial");
        drop(log); std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn snapshot_has_one_history_prefix_and_one_cursor_bound_trace() {
        let saved=vec![Message::user_text("old"),Message::assistant("old reply",vec![]),Message::user_text("current"),Message::assistant("new reply",vec![])];
        let events=vec![json!({"session":"s","runId":"r","sequence":2,"event":{"type":"text_delta","text":"new reply"}})];
        let result=snapshot_projection(saved,Some("r".into()),2,events,true);
        assert_eq!(result["messages"].as_array().unwrap().len(),3); assert_eq!(result["events"].as_array().unwrap().len(),1); assert_eq!(result["sequence"],2);
        assert_eq!(snapshot_projection(vec![],Some("new".into()),1,vec![],true)["ready"],false);
        let done=snapshot_projection(vec![Message::user_text("q")],Some("done".into()),3,vec![json!({"event":{"type":"done"}})],true);
        assert_eq!(done["running"],false);
        let failed=snapshot_projection(vec![Message::user_text("q")],Some("failed".into()),3,vec![json!({"event":{"type":"error","message":"network"}})],true);
        assert_eq!(failed["events"].as_array().unwrap().len(),1); assert_eq!(failed["running"],false);
    }
}

#[cfg(test)]
mod terminal_commit_tests {
    use super::*;
    #[test]
    fn failed_goal_commit_is_not_forwarded_as_a_successful_terminal() {
        let root=std::env::temp_dir().join(format!("terminal-commit-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let log=JsonlRunLog::new(&root);let mut sent=false;
        let record=RunEvent{run_id:"r1".into(),session_id:"s1".into(),parent_run_id:None,sequence:1,timestamp_ms:1,body:RunEventBody::State{state:agent_kernel::RunState::Completed,reason:None}};
        assert!(forward_observed(&log,record,"",|_|Err(HarnessError::Store("goal write failed".into())),|_,_|sent=true).is_err());assert!(!sent);
        let _=std::fs::remove_dir_all(root);
    }
}

//! Conservative business outcomes: a completed model loop is never task success.
//! Artifact verification is explicitly scoped to the claimed file, not a generic task score.
use agent_kernel::journal::JsonlRunLog;
use crate::community_media::{bilibili_target, BILIBILI_HTTP_567, DOWNLOAD_TOOL};
use agent_kernel::{RunEventBody, RunState, Event};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all="snake_case")]
pub enum Status { Succeeded, Failed, Partial, Cancelled, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct TaskOutcome {
    pub run_id: String,
    pub session_id: String,
    pub status: Status,
    pub validator: String,
    pub evidence: String,
    pub verified_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub last_tool_call_id: Option<String>,
}

fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 &&
    id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn outcome_path(root: &Path, session: &str, run: &str) -> Result<PathBuf, String> {
    if !safe_id(session) || !safe_id(run) { return Err("invalid task identity".into()); }
    Ok(root.join(session).join(format!("{run}.json")))
}

pub fn load(root: &Path, session: &str, run: &str) -> Result<Option<TaskOutcome>, String> {
    let path = outcome_path(root, session, run)?;
    match fs::read(&path) {
        Ok(data) => {
            let outcome: TaskOutcome = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
            if outcome.run_id != run || outcome.session_id != session { return Err("outcome identity mismatch".into()); }
            Ok(Some(outcome))
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Restore each run at its own tool boundary. A damaged outcome cannot make another task succeed.
pub fn history(root: &Path, runs: &Path, session: &str) -> Vec<TaskOutcome> {
    if !safe_id(session) { return Vec::new(); }
    let Ok(entries) = fs::read_dir(root.join(session)) else { return Vec::new(); };
    let journal = JsonlRunLog::new(runs);
    entries.flatten().filter_map(|entry| {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") { return None; }
        let run = path.file_stem()?.to_str()?;
        let mut outcome = load(root, session, run).ok().flatten()?;
        // Older records predate the anchor field. Resolve it from the trace without replaying anything.
        if outcome.last_tool_call_id.is_none() {
            outcome.last_tool_call_id = inspect(&journal, session, run).ok().and_then(|o| o.last_tool_call_id);
        }
        outcome.last_tool_call_id.as_ref()?;
        Some(outcome)
    }).collect()
}

pub fn save(root: &Path, record: &TaskOutcome) -> Result<(), String> {
    let path = outcome_path(root, &record.session_id, &record.run_id)?;
    fs::create_dir_all(path.parent().ok_or("invalid outcome path")?).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| e.to_string())
}

pub fn inspect(journal: &JsonlRunLog, session: &str, run: &str) -> Result<TaskOutcome, String> {
    inspect_with_recovery(journal, session, run).map(|(outcome, _)| outcome)
}

pub(super) fn inspect_with_recovery(journal: &JsonlRunLog, session: &str, run: &str) -> Result<(TaskOutcome, bool), String> {
    let inspection = journal.inspect(run).map_err(|e| e.to_string())?;
    // Read all tool end statuses using a bounded cursor, not model text.
    let mut after = 0;
    let mut failed_tools = 0usize;
    let mut only_failed_downloads_or_lookups = true;
    let mut failed_download = false;
    let mut recovered_video = false;
    let mut last_tool_call_id = None;
    let mut calls = std::collections::HashMap::new();
    let mut unresolved = Vec::new();
    loop {
        let batch = journal.read_after(run, after, 100).map_err(|e| e.to_string())?;
        if batch.is_empty() { break; }
        for record in &batch {
            if record.session_id != session { return Err("run belongs to another session".into()); }
            if let RunEventBody::Event { event } = &record.body {
                match event {
                    Event::ToolStart { id, name, arguments } => {
                        last_tool_call_id = Some(id.clone());
                        let arguments = serde_json::from_str::<serde_json::Value>(arguments)
                            .unwrap_or_else(|_| serde_json::Value::String(arguments.clone()));
                        calls.insert(id.clone(), (name.clone(), arguments));
                    }
                    Event::ToolEnd { id, is_error, structured_content, output, .. } => {
                        let key = calls.remove(id).unwrap_or_else(|| (id.clone(), serde_json::Value::Null));
                        let download = key.0 == DOWNLOAD_TOOL;
                        failed_download |= download && *is_error;
                        only_failed_downloads_or_lookups &= (download && *is_error)
                            || matches!(key.0.as_str(), "web_search" | "web_fetch" | "web_extract" | "experience_recall" | "todo");
                        if *is_error {
                            failed_tools += 1;
                            let validation = structured_content.as_ref().is_some_and(|v| v["harness_status"] == "invalid_arguments")
                                || output.starts_with("invalid arguments for `");
                            let recoverable = download && structured_content.as_ref()
                                .is_some_and(|meta| meta["recoverable_error"] == BILIBILI_HTTP_567);
                            let failure = (key, validation, recoverable);
                            if !unresolved.contains(&failure) { unresolved.push(failure); }
                        } else if !structured_content.as_ref().is_some_and(|v| v["harness_status"] == "unchanged_repeat") {
                            // A later successful retry of the same operation can repair a
                            // failed attempt. A different tool/target is not proof of repair.
                            if download && key.1["range_retry"] == true && structured_content.as_ref()
                                .is_some_and(|meta| meta["verified_recovery"] == BILIBILI_HTTP_567)
                            {
                                if let Some(target) = key.1["url"].as_str().and_then(bilibili_target) {
                                    let count = unresolved.len();
                                    unresolved.retain(|(old, _, recoverable)| {
                                        !*recoverable || old.1["url"].as_str().and_then(bilibili_target).as_ref() != Some(&target)
                                    });
                                    recovered_video |= count != unresolved.len();
                                }
                            }
                            unresolved.retain(|(old, validation, _)| old != &key && !(*validation && old.0 == key.0
                                && old.1.as_object().is_some_and(|args| args.iter().all(|(name, value)| key.1.get(name) == Some(value)))));
                        }
                    }
                    _ => {}
                }
            }
        }
        after = batch.last().map_or(after, |r| r.sequence);
        if batch.len() < 100 { break; }
    }
    let status = if !inspection.terminal || !inspection.incomplete_tools.is_empty() {
        Status::Unknown
    } else {
        match inspection.state {
            Some(RunState::Cancelled) => Status::Cancelled,
            Some(RunState::Failed) => Status::Failed,
            Some(RunState::Completed) if failed_download && only_failed_downloads_or_lookups => Status::Failed,
            Some(RunState::Completed) if !unresolved.is_empty() => Status::Partial,
            // Even with no tool errors, completion is not proof of user task success.
            _ => Status::Unknown,
        }
    };
    let recovered_video = recovered_video && inspection.terminal && inspection.incomplete_tools.is_empty()
        && matches!(inspection.state, Some(RunState::Completed));
    Ok((TaskOutcome { run_id: run.into(), session_id: session.into(), status,
        validator: "run_trace_only".into(),
        evidence: format!("terminal={}; tool_errors={failed_tools}; unresolved_tool_errors={}; incomplete_tools={}; verified_video_recovery={recovered_video}",
            inspection.terminal, unresolved.len(), inspection.incomplete_tools.len()),
        verified_at_ms: None, last_tool_call_id }, recovered_video))
}

/// Deterministic validator for one explicitly declared file-output objective.
/// Refuses a non-file, symlink escape, empty file, or hash mismatch.
pub fn verify_file(root: &Path, session: &str, run: &str, workspace: &Path,
    relative: &str, expected_sha256: &str) -> Result<TaskOutcome, String> {
    if relative.is_empty() || relative.len() > 512 || Path::new(relative).is_absolute() {
        return Err("invalid relative path".into());
    }
    if expected_sha256.len() != 64 || !expected_sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected SHA256 is required (64 hex chars)".into());
    }
    let base = workspace.canonicalize().map_err(|e| e.to_string())?;
    let dest = workspace.join(relative).canonicalize().map_err(|e| e.to_string())?;
    if !dest.starts_with(&base) || !dest.is_file() { return Err("file outside workspace or missing".into()); }
    let mut file = fs::File::open(&dest).map_err(|e| e.to_string())?;
    let mut digest = Sha256::new();
    let mut buf = [0u8;8192];
    let mut size=0u64;
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n==0 { break; }
        size+=n as u64;
        digest.update(&buf[..n]);
    }
    if size==0 { return Err("file is empty".into()); }
    let actual=format!("{:x}",digest.finalize());
    if !actual.eq_ignore_ascii_case(expected_sha256) { return Err("SHA256 mismatch".into()); }
    let result=TaskOutcome { session_id: session.into(), run_id: run.into(),
        last_tool_call_id: load(root, session, run).ok().flatten().and_then(|old| old.last_tool_call_id),
        status: Status::Succeeded, validator:"expected_file_sha256".into(),
        evidence: format!("verified sha256={actual} bytes={size} path={relative}"),
        verified_at_ms:Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_millis() as u64),
    };
    save(root,&result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn verified_bilibili_retry_reconciles_only_the_original_target() {
    use agent_kernel::{Agent, MemoryStore, Tool, ToolSpec, ToolCtx, ToolResult, Risk, Part, RunOptions};
    use agent_kernel::provider::scripted::{ScriptedProvider, Step};
    use serde_json::{json, Value};
    use std::sync::{Arc, atomic::AtomicBool};

    struct Stub(ToolSpec);
    impl Tool for Stub {
        fn spec(&self) -> &ToolSpec { &self.0 }
        fn execute_result(&self, args: Value, _: &ToolCtx) -> ToolResult {
            if self.0.name == "web_search" {
                return ToolResult::ok("上游问题记录提供了一次 Range 重试的线索");
            }
            if args["range_retry"] == true {
                ToolResult::ok("verified media").with_structured_content(json!({"verified_recovery": BILIBILI_HTTP_567}))
            } else {
                ToolResult::error("HTTP Error 567").with_structured_content(json!({"recoverable_error": BILIBILI_HTTP_567}))
            }
        }
    }

    for (case, retry_url, recovered) in [
        ("same", "https://www.bilibili.com/video/BV15N4V6hEmF/", true),
        ("other-part", "https://www.bilibili.com/video/BV15N4V6hEmF/?p=2", false),
    ] {
        let root = std::env::temp_dir().join(format!("xiaoban-video-recovery-{}-{case}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let provider = Arc::new(ScriptedProvider::new(vec![
            Step::tool_call("original", DOWNLOAD_TOOL, r#"{"url":"https://www.bilibili.com/zh-Hans/video/BV15N4V6hEmF/?trackid=example"}"#),
            Step::tool_call("search", "web_search", r#"{"query":"yt-dlp Bilibili HTTP 567"}"#),
            Step::tool_call("retry", DOWNLOAD_TOOL, &json!({"url":retry_url,"range_retry":true}).to_string()),
            Step::text("交付文件"),
        ]));
        let agent = Agent::builder(provider.clone(), Arc::new(MemoryStore::new()))
            .tool(Stub(ToolSpec::new(DOWNLOAD_TOOL, "download", json!({"type":"object"})).risk(Risk::Read)))
            .tool(Stub(ToolSpec::new("web_search", "search", json!({"type":"object"})).risk(Risk::Read)))
            .middleware(super::super::feature_hints::RecoveryHint)
            .build().unwrap();
        let journal = JsonlRunLog::new(root.join("runs"));
        let run = agent.run_parts_observed("s", vec![Part::Text { text:"下载视频".into() }],
            RunOptions::default(), &mut |record| journal.append(&record), &Arc::new(AtomicBool::new(false))).unwrap();
        let requests = provider.requests();
        assert!(!requests[0].messages.iter().any(|message| message.text().contains("桌面故障排查")));
        let followup = &requests[1].messages;
        assert_eq!(followup[followup.len()-2].role, agent_kernel::Role::Tool);
        assert!(followup.last().unwrap().synthetic && followup.last().unwrap().text().contains("上游问题记录"));
        let (outcome, verified) = inspect_with_recovery(&journal, "s", &run.run_id).unwrap();
        assert_eq!(verified, recovered);
        assert_eq!(outcome.status, if recovered { Status::Unknown } else { Status::Partial });

        if verified {
            let db = super::super::LearningDb::open(&root.join("learning.sqlite")).unwrap();
            super::super::learning::record_bilibili_recovery(&db, &run.run_id).unwrap();
            let candidates = db.list_experiences("personal").unwrap();
            assert_eq!(candidates.len(), 1);
            assert_eq!(candidates[0].0.state, "candidate");
            assert!(db.search_experiences("personal", "Bilibili", 5).unwrap().is_empty());
        }
        fs::remove_dir_all(root).unwrap();
    }
 }

 #[test]
 fn skipping_an_unchanged_read_does_not_repair_failed_execution() {
    use agent_kernel::{Agent, MemoryStore, Tool, ToolSpec, ToolCtx, ToolResult, Risk, Part, RunOptions};
    use agent_kernel::provider::scripted::{ScriptedProvider, Step};
    use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
    struct Read { spec: ToolSpec, calls: AtomicUsize }
    impl Tool for Read {
        fn spec(&self) -> &ToolSpec { &self.spec }
        fn execute_result(&self, _: serde_json::Value, _: &ToolCtx) -> ToolResult {
            match self.calls.fetch_add(1, Ordering::SeqCst) {
                0 => ToolResult::error("read failed"),
                1 => ToolResult::ok("not dispatched").with_structured_content(serde_json::json!({"harness_status":"unchanged_repeat"})),
                _ => ToolResult::ok("actual file contents"),
            }
        }
    }
    for actual_retry in [false, true] {
        let root = std::env::temp_dir().join(format!("xiaoban-skipped-outcome-{}-{actual_retry}",std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let mut steps = vec![Step::tool_call("a","read","{}"),Step::tool_call("b","read","{}")];
        if actual_retry { steps.push(Step::tool_call("c","read","{}")); }
        steps.push(Step::text("report"));
        let agent = Agent::builder(Arc::new(ScriptedProvider::new(steps)), Arc::new(MemoryStore::new()))
            .tool(Read { spec:ToolSpec::new("read","read",serde_json::json!({"type":"object"})).risk(Risk::Read),calls:AtomicUsize::new(0) })
            .build().unwrap();
        let journal = JsonlRunLog::new(&root);
        let run = agent.run_parts_observed("s",vec![Part::Text{text:"read".into()}],RunOptions::default(),
            &mut |record|journal.append(&record), &Arc::new(AtomicBool::new(false))).unwrap();
        let outcome = inspect(&journal,"s",&run.run_id).unwrap();
        assert_eq!(outcome.status, if actual_retry {Status::Unknown} else {Status::Partial});
        fs::remove_dir_all(root).unwrap();
    }
 }
 #[test] fn never_claim_success_from_model_completion() {
    let base=std::env::temp_dir().join(format!("xiaoban-outcome-{}",std::process::id()));
    let _=fs::remove_dir_all(&base);
    let j=JsonlRunLog::new(base.join("runs"));
    use agent_kernel::{RunEvent,RunEventBody,RunState};
    for (sequence,state) in [(1,RunState::Running),(2,RunState::Completed)] {
       j.append(&RunEvent {run_id:"r1".into(),session_id:"s1".into(),parent_run_id:None,
         sequence,timestamp_ms:100,body:RunEventBody::State {state,reason:None}}).unwrap();
    }
    assert_eq!(inspect(&j,"s1","r1").unwrap().status,Status::Unknown);
    let _=fs::remove_dir_all(base);
 }

 #[test]
 fn a_successful_retry_repairs_the_same_failed_operation_but_not_a_different_target() {
    use agent_kernel::{RunEvent, ToolSpec, Tool, ToolCtx, ToolResult, Part, Risk, Agent, MemoryStore, RunOptions};
    use agent_kernel::provider::scripted::{ScriptedProvider, Step};
    use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
    struct RetryTool { spec: ToolSpec, calls: AtomicUsize }
    impl Tool for RetryTool {
        fn spec(&self) -> &ToolSpec { &self.spec }
        fn execute_result(&self, _: serde_json::Value, _: &ToolCtx) -> ToolResult {
            if self.calls.fetch_add(1, Ordering::SeqCst) == 0 { ToolResult::error("temporary failure") }
            else { ToolResult::ok("recovered") }
        }
    }
    for (case, second_args, expected) in [
        ("same", r#"{"path":"a"}"#, Status::Unknown),
        ("different", r#"{"path":"b"}"#, Status::Partial),
    ] {
        let base=std::env::temp_dir().join(format!("xiaoban-outcome-retry-{case}-{}",std::process::id()));
        let _=fs::remove_dir_all(&base);
        let journal=JsonlRunLog::new(base.join("runs"));
        let provider=Arc::new(ScriptedProvider::new(vec![
            Step::tool_call("one","read",r#"{"path":"a"}"#),
            Step::tool_call("two","read",second_args), Step::text("report"),
        ]));
        let agent=Agent::builder(provider, Arc::new(MemoryStore::new())).tool(RetryTool {
            spec:ToolSpec::new("read","read test file",serde_json::json!({"type":"object"})).risk(Risk::Read),
            calls:AtomicUsize::new(0),
        }).build().unwrap();
        let mut observe=|record:RunEvent|journal.append(&record);
        let summary=agent.run_parts_observed("s1",vec![Part::Text {text:"read a".into()}],RunOptions::default(),
            &mut observe,&Arc::new(AtomicBool::new(false))).unwrap();
        let outcome=inspect(&journal,"s1",&summary.run_id).unwrap();
        assert_eq!(outcome.status,expected,"{case}: {}",outcome.evidence);
        assert!(outcome.evidence.contains("tool_errors=1"),"keep the original failure as evidence");
        assert_ne!(outcome.status,Status::Succeeded,"successful tool execution is not business verification");
        let _=fs::remove_dir_all(base);
    }
 }

 #[test]
 fn completing_missing_required_arguments_repairs_prior_validation_failures() {
    use agent_kernel::{RunEvent, ToolSpec, Tool, ToolCtx, ToolResult, Part, Risk, Agent, MemoryStore, RunOptions};
    use agent_kernel::provider::scripted::{ScriptedProvider, Step};
    use std::sync::{Arc, atomic::AtomicBool};
    struct CreateGameTool { spec: ToolSpec }
    impl Tool for CreateGameTool {
        fn spec(&self) -> &ToolSpec { &self.spec }
        fn execute_result(&self, _: serde_json::Value, _: &ToolCtx) -> ToolResult { ToolResult::ok("installed") }
    }
    let base = std::env::temp_dir().join(format!("xiaoban-outcome-validation-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let journal = JsonlRunLog::new(base.join("runs"));
    let provider = Arc::new(ScriptedProvider::new(vec![
        Step::tool_call("c1", "game_create", r#"{"source_dir":"game-drafts/ant"}"#),
        Step::tool_call("c2", "game_create", r#"{"id":"ant","source_dir":"game-drafts/ant"}"#),
        Step::tool_call("c3", "game_create", r#"{"id":"ant","name":"蚂蚁骑车","description":"横版闯关","source_dir":"game-drafts/ant"}"#),
        Step::text("done"),
    ]));
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "name": { "type": "string" },
            "description": { "type": "string" },
            "source_dir": { "type": "string" }
        },
        "required": ["id", "name", "description", "source_dir"]
    });
    let agent = Agent::builder(provider, Arc::new(MemoryStore::new())).tool(CreateGameTool {
        spec: ToolSpec::new("game_create", "install game", schema).risk(Risk::Write),
    }).build().unwrap();
    let mut observe = |record: RunEvent| journal.append(&record);
    let summary = agent.run_parts_observed("s1", vec![Part::Text { text: "create".into() }], RunOptions::default(),
        &mut observe, &Arc::new(AtomicBool::new(false))).unwrap();
    let outcome = inspect(&journal, "s1", &summary.run_id).unwrap();
    assert_eq!(outcome.status, Status::Unknown, "{}", outcome.evidence);
    assert!(outcome.evidence.contains("tool_errors=2; unresolved_tool_errors=0"), "{}", outcome.evidence);
    let _ = fs::remove_dir_all(base);
 }
}

#[cfg(test)]
mod history_regressions {
    use super::*;
    use agent_kernel::{RunEvent, RunEventBody};
    #[test]
    fn restores_legacy_and_current_outcomes_with_their_own_tool_anchors() {
        let root = std::env::temp_dir().join(format!("outcome-history-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let runs = root.join("runs"); let outcomes = root.join("outcomes"); let journal = JsonlRunLog::new(&runs);
        for (run, tool) in [("old", "old-tool"), ("new", "new-tool")] {
            journal.append(&RunEvent { run_id:run.into(), session_id:"s1".into(), parent_run_id:None, sequence:1, timestamp_ms:1,
                body:RunEventBody::Event { event:Event::ToolStart { id:tool.into(), name:"todo".into(), arguments:"{}".into() } } }).unwrap();
            let record = inspect(&journal, "s1", run).unwrap();
            assert_eq!(record.last_tool_call_id.as_deref(), Some(tool)); save(&outcomes, &record).unwrap();
        }
        let old_path = outcomes.join("s1/old.json");
        let mut legacy: serde_json::Value = serde_json::from_slice(&fs::read(&old_path).unwrap()).unwrap();
        legacy.as_object_mut().unwrap().remove("lastToolCallId"); fs::write(old_path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        fs::write(outcomes.join("s1/broken.json"), "{partial").unwrap();
        let records = history(&outcomes, &runs, "s1"); assert_eq!(records.len(), 2);
        assert_eq!(records.iter().find(|o| o.run_id == "old").unwrap().last_tool_call_id.as_deref(), Some("old-tool"));
        assert_eq!(records.iter().find(|o| o.run_id == "new").unwrap().last_tool_call_id.as_deref(), Some("new-tool"));
        assert!(history(&outcomes, &runs, "other").is_empty()); drop(journal); fs::remove_dir_all(root).unwrap();
    }
}

//! Execution policy and deterministic acceptance. No model call or command is run by this module.
use agent_kernel::{Message, Middleware, Nudge, Role, RunControl, Stop, StopDecision};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecutionLimits {
    /// None means no count cap; finite budgets are still independently enforced.
    pub max_turns: Option<usize>,
    pub max_tool_calls: Option<usize>,
    pub max_minutes: Option<u64>,
    pub max_reported_tokens: Option<u64>,
}
impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_turns: None,
            max_tool_calls: None,
            max_minutes: Some(45),
            max_reported_tokens: Some(1_000_000),
        }
    }
}
impl ExecutionLimits {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_turns.is_some_and(|n| n == 0 || n > 1_000_000)
            || self
                .max_tool_calls
                .is_some_and(|n| n == 0 || n > 10_000_000)
            || self.max_minutes.is_some_and(|n| n == 0 || n > 43_200)
            || self
                .max_reported_tokens
                .is_some_and(|n| n == 0 || n > 1_000_000_000)
        {
            return Err("执行预算必须为正数，留空表示不限制".into());
        }
        Ok(())
    }
    pub fn apply(&self, builder: agent_kernel::AgentBuilder) -> agent_kernel::AgentBuilder {
        match self.max_turns {
            Some(n) => builder.max_turns(n),
            None => builder.unlimited_turns(),
        }
    }
    pub fn control(&self, spent_tokens: u64, elapsed_ms: u64) -> Result<RunControl, String> {
        self.validate()?;
        let tokens = self
            .max_reported_tokens
            .map(|limit| limit.saturating_sub(spent_tokens));
        let millis = self
            .max_minutes
            .map(|minutes| minutes * 60_000)
            .map(|limit| limit.saturating_sub(elapsed_ms));
        if tokens == Some(0) || millis == Some(0) {
            return Err("目标累计执行预算已用完；可在安全与权限设置中调整预算后继续".into());
        }
        Ok(RunControl::limits(
            millis.map(Duration::from_millis),
            tokens,
        ))
    }
}

// Unknown tools are conservatively treated as mutations. The command must be rerun after them.
fn read_only(name: &str) -> bool {
    matches!(
        name,
        "read_file"
            | "list_dir"
            | "grep"
            | "web_fetch"
            | "web_search"
            | "time_now"
            | "todo"
            | "ask_user"
            | "research"
            | "tool_search"
            | "tool_load"
            | "skill_find"
            | "load_skill"
            | "memory_read"
            | "process_read"
    )
}

/// Evidence comes only from successful actual tool results, never assistant prose.
/// A verification sharing a batch with another mutation is not fresh evidence.
pub fn command_evidence(messages: &[Message], command: &str) -> Option<String> {
    if command.trim().is_empty() {
        return None;
    }
    let mut candidate: Option<(String, bool)> = None;
    let mut process: Option<String> = None;
    let mut reads = std::collections::HashSet::new();
    let mut evidence = None;
    let mut pending = std::collections::HashSet::new();
    for message in messages {
        let mutations: Vec<_> = message
            .tool_calls
            .iter()
            .filter(|c| !read_only(&c.name))
            .collect();
        if !mutations.is_empty() {
            pending.extend(mutations.iter().map(|c| c.id.clone()));
            candidate = None;
            process = None;
            reads.clear();
            evidence = None;
            if mutations.len() == 1 {
                let call = mutations[0];
                if matches!(call.name.as_str(), "shell" | "process_start")
                    && serde_json::from_str::<serde_json::Value>(&call.arguments)
                        .ok()
                        .and_then(|v| v.get("command")?.as_str().map(str::to_owned))
                        .as_deref()
                        == Some(command)
                {
                    candidate = Some((call.id.clone(), call.name == "process_start"));
                }
            }
        }
        for call in &message.tool_calls {
            if call.name == "process_read"
                && process.is_some()
                && serde_json::from_str::<serde_json::Value>(&call.arguments)
                    .ok()
                    .and_then(|v| v.get("id")?.as_str().map(str::to_owned))
                    == process
            {
                reads.insert(call.id.clone());
            }
        }
        let proof = |id: &str| {
            Some(format!(
                "用户指定验收命令已成功执行：{}\n工具调用：{id}",
                command.chars().take(1500).collect::<String>()
            ))
        };
        let exited_ok = |value: Option<&serde_json::Value>| {
            value.is_some_and(|v| v["state"] == "exited" && v["exit_code"].as_i64() == Some(0))
        };
        let mutation_ended = message.role == Role::Tool
            && message
                .tool_call_id
                .as_ref()
                .is_some_and(|id| pending.remove(id));
        if mutation_ended
            && candidate
                .as_ref()
                .is_some_and(|(id, _)| message.tool_call_id.as_deref() == Some(id))
        {
            if !message.is_error && pending.is_empty() {
                let (id, background) = candidate.as_ref().unwrap();
                if *background {
                    process = message
                        .structured_content
                        .as_ref()
                        .and_then(|v| v["id"].as_str())
                        .map(str::to_owned);
                    if exited_ok(message.structured_content.as_ref()) {
                        evidence = proof(id);
                    }
                } else {
                    evidence = proof(id);
                }
            }
            candidate = None;
        } else if mutation_ended {
            candidate = None;
            process = None;
            reads.clear();
            evidence = None;
        }
        if message.role == Role::Tool
            && message
                .tool_call_id
                .as_ref()
                .is_some_and(|id| reads.remove(id))
        {
            if !message.is_error
                && pending.is_empty()
                && exited_ok(message.structured_content.as_ref())
                && message
                    .structured_content
                    .as_ref()
                    .and_then(|v| v["id"].as_str())
                    == process.as_deref()
            {
                evidence = proof(message.tool_call_id.as_deref().unwrap());
            }
        }
    }
    evidence
}

/// Keeps a goal in the normal tool/permission loop until its user-chosen command was run.
/// This never runs a command behind the user's permission policy.
pub struct VerificationGuard {
    pub command: String,
}
fn permission_denied(stop: &Stop<'_>) -> bool {
    stop.history
        .get(stop.run_start..)
        .unwrap_or_default()
        .iter()
        .any(|m| {
            m.role == Role::Tool
                && m.is_error
                && m.structured_content
                    .as_ref()
                    .is_some_and(|v| v["harness_status"] == "permission_denied")
        })
}
impl Middleware for VerificationGuard {
    fn before_stop(&self, stop: &Stop<'_>) -> StopDecision {
        if permission_denied(stop) {
            return StopDecision::Pause("permission_denied".into());
        }
        if stop.finish != agent_kernel::FinishReason::Stop {
            return StopDecision::Finish;
        }
        if command_evidence(
            stop.history.get(stop.run_start..).unwrap_or_default(),
            &self.command,
        )
        .is_some()
        {
            return StopDecision::Finish;
        }
        if stop.continuations_without_progress >= 3 {
            return StopDecision::Pause("verification_required".into());
        }
        StopDecision::Continue(Nudge { reason: "verification_required".into(), message: format!(
            "目标尚未通过验收。请原样运行用户指定的命令：\n{}\n普通检查使用 shell；长耗时检查用 process_start，再用 process_read 等待实际退出且 exit_code=0。必须在最后一次修改之后验证。失败则修复后再验证；无法继续时使用 ask_user 说明阻塞，不要宣称完成。", self.command) })
    }
}

/// A persisted unfinished plan remains actionable after compaction or a new run.
pub struct GoalProgressGuard(pub std::sync::Arc<agent_kernel::builtin::Todo>);
impl Middleware for GoalProgressGuard {
    fn before_stop(&self, stop: &Stop<'_>) -> StopDecision {
        if permission_denied(stop) {
            return StopDecision::Pause("permission_denied".into());
        }
        if stop.finish != agent_kernel::FinishReason::Stop {
            return StopDecision::Finish;
        }
        let progress = match self.0.load_snapshot(stop.session) {
            Ok(progress) => progress,
            Err(_) => return StopDecision::Pause("progress_store_unavailable".into()),
        };
        if progress
            .todos
            .iter()
            .all(|item| item.status == agent_kernel::builtin::TodoStatus::Completed)
        {
            return StopDecision::Finish;
        }
        if stop.continuations_without_progress >= 3 {
            return StopDecision::Pause("unfinished_milestones".into());
        }
        StopDecision::Continue(Nudge { reason: "unfinished_milestones".into(), message:
            "持久任务清单还有未完成的里程碑。请检查现场，继续当前目标；已完成则用 todo 记录实际证据并更新清单。若计划改变，更新原因和下一步；需要用户信息时用 ask_user，不要把未完成步骤视为已完成。".into() })
    }
}

/// Reconstruct tool calls/results from the durable trace, excluding sibling/child runs.
pub fn trace_evidence(
    log: &agent_kernel::journal::JsonlRunLog,
    session: &str,
    run: &str,
    command: &str,
) -> Result<Option<String>, String> {
    use agent_kernel::{Event, RunEventBody, ToolCall, ToolResult};
    let inspection = log.inspect(run).map_err(|e| e.to_string())?;
    if !inspection.terminal
        || inspection.state != Some(agent_kernel::RunState::Completed)
        || !inspection.incomplete_tools.is_empty()
    {
        return Ok(None);
    }
    let mut messages = Vec::new();
    let mut after = 0;
    loop {
        let records = log.read_after(run, after, 100).map_err(|e| e.to_string())?;
        if records.is_empty() {
            break;
        }
        for record in &records {
            if record.session_id != session || record.run_id != run {
                return Err("验收日志身份不匹配".into());
            }
            if let RunEventBody::Event { event } = &record.body {
                match event {
                    Event::ToolStart {
                        id,
                        name,
                        arguments,
                    } => {
                        let mut message = Message::assistant("", Vec::new());
                        message.tool_calls.push(ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            arguments: arguments.clone(),
                        });
                        messages.push(message);
                    }
                    Event::ToolEnd {
                        id,
                        is_error,
                        structured_content,
                        ..
                    } => {
                        let mut result = if *is_error {
                            ToolResult::error("failed")
                        } else {
                            ToolResult::ok("succeeded")
                        };
                        result.structured_content = structured_content.clone();
                        messages.push(result.message(id));
                    }
                    _ => {}
                }
            }
        }
        after = records.last().unwrap().sequence;
    }
    // ToolStart records precede dispatch and may represent a batch. The candidate is
    // invalidated by any later mutation before its result; never accept interleaved writes.
    Ok(command_evidence(&messages, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::{ToolCall, ToolResult};
    fn call(id: &str, name: &str, command: &str) -> Message {
        let mut m = Message::assistant("", Vec::new());
        m.tool_calls.push(ToolCall {
            id: id.into(),
            name: name.into(),
            arguments: serde_json::json!({"command":command}).to_string(),
        });
        m
    }
    #[test]
    fn only_fresh_exact_successful_commands_count() {
        let pass = vec![
            call("v", "shell", "cargo test"),
            ToolResult::ok("ok").message("v"),
        ];
        assert!(command_evidence(&pass, "cargo test").is_some());
        assert!(command_evidence(&pass, "cargo test --all").is_none());
        let mut stale = pass.clone();
        stale.push(call("w", "edit_file", ""));
        assert!(command_evidence(&stale, "cargo test").is_none());
        assert!(command_evidence(
            &[
                call("v", "shell", "cargo test"),
                ToolResult::error("failed").message("v")
            ],
            "cargo test"
        )
        .is_none());
        assert!(command_evidence(
            &[Message::assistant("cargo test passed", Vec::new())],
            "cargo test"
        )
        .is_none());
        let mut parallel = call("v", "shell", "cargo test");
        parallel
            .tool_calls
            .extend(call("w", "edit_file", "").tool_calls);
        assert!(
            command_evidence(&[parallel, ToolResult::ok("ok").message("v")], "cargo test")
                .is_none()
        );
    }
    #[test]
    fn budgets_are_cumulative_and_legacy_defaults_keep_time_and_token_protection() {
        let limits: ExecutionLimits = serde_json::from_str("{}").unwrap();
        assert_eq!(limits, ExecutionLimits::default());
        assert_eq!(limits.max_turns, None);
        assert_eq!(limits.max_tool_calls, None);
        assert!(limits.control(1_000_000, 0).is_err());
        assert!(limits.control(0, 45 * 60_000).is_err());
        assert!(ExecutionLimits {
            max_turns: Some(0),
            ..limits.clone()
        }
        .validate()
        .is_err());
        let unlimited = ExecutionLimits {
            max_turns: None,
            max_tool_calls: None,
            max_minutes: None,
            max_reported_tokens: None,
        };
        assert!(unlimited
            .control(u64::MAX, u64::MAX)
            .unwrap()
            .check()
            .is_ok());
    }

    #[test]
    fn long_checks_require_the_matching_process_to_exit_successfully_after_the_last_write() {
        let start = call("start", "process_start", "npm test");
        let started = ToolResult::ok("")
            .with_structured_content(
                serde_json::json!({"id":"p1","state":"running","exit_code":null}),
            )
            .message("start");
        let read = Message::assistant(
            "",
            vec![ToolCall {
                id: "read".into(),
                name: "process_read".into(),
                arguments: r#"{"id":"p1","wait_ms":60000}"#.into(),
            }],
        );
        for (state, code, accepts) in [
            ("running", None, false),
            ("exited", Some(1), false),
            ("exited", Some(0), true),
            ("cancelled", Some(0), false),
        ] {
            let result = ToolResult::ok("")
                .with_structured_content(
                    serde_json::json!({"id":"p1","state":state,"exit_code":code}),
                )
                .message("read");
            let messages = vec![start.clone(), started.clone(), read.clone(), result];
            assert_eq!(command_evidence(&messages, "npm test").is_some(), accepts);
            let mut stale = messages;
            stale.push(call("write", "edit_file", ""));
            assert!(command_evidence(&stale, "npm test").is_none());
        }
        let wrong = ToolResult::ok("")
            .with_structured_content(serde_json::json!({"id":"p2","state":"exited","exit_code":0}))
            .message("read");
        assert!(command_evidence(&[start, started, read, wrong], "npm test").is_none());
    }

    #[test]
    fn a_real_managed_process_supplies_exit_evidence() {
        use agent_kernel::{
            process::{ProcessManager, ProcessScope},
            ToolCtx,
        };
        let root = std::env::temp_dir().join(format!("goal-process-check-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let manager = ProcessManager::new(&root).unwrap();
        let context = ToolCtx::detached("verification");
        let command = if cfg!(windows) {
            "echo verification"
        } else {
            "printf verification"
        };
        let started = manager
            .start(&context, command, ProcessScope::Run, Duration::from_secs(5))
            .unwrap();
        let stopped = manager
            .read(&context, &started.id, 0, 1000, Duration::from_secs(5))
            .unwrap();
        assert_eq!(stopped.exit_code, Some(0));
        let read = Message::assistant(
            "",
            vec![ToolCall {
                id: "read".into(),
                name: "process_read".into(),
                arguments: serde_json::json!({"id":started.id,"wait_ms":5000}).to_string(),
            }],
        );
        let messages = vec![
            call("start", "process_start", command),
            ToolResult::ok("")
                .with_structured_content(serde_json::to_value(&started).unwrap())
                .message("start"),
            read,
            ToolResult::ok("")
                .with_structured_content(serde_json::to_value(&stopped).unwrap())
                .message("read"),
        ];
        assert!(command_evidence(&messages, command).is_some());
        manager.shutdown().unwrap();
        drop(manager);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn acceptance_uses_the_normal_permission_loop_and_durable_run_evidence() {
        use agent_kernel::{
            provider::scripted::{ScriptedProvider, Step},
            Agent, Approval, ApprovalRequest, MemoryStore, Permission, PermissionMode, Risk,
            RunOptions, Tool, ToolCtx, ToolSpec,
        };
        use std::sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            Arc,
        };
        struct CommandTool {
            spec: ToolSpec,
            executions: Arc<AtomicUsize>,
        }
        impl Tool for CommandTool {
            fn spec(&self) -> &ToolSpec {
                &self.spec
            }
            fn execute(&self, _: serde_json::Value, _: &ToolCtx) -> Result<String, String> {
                self.executions.fetch_add(1, Ordering::SeqCst);
                Ok("[exit code: 0]".into())
            }
        }
        for allowed in [true, false] {
            let executions = Arc::new(AtomicUsize::new(0));
            let steps = vec![
                Step::text("done without checking"),
                Step::tool_call("verify", "shell", r#"{"command":"npm test"}"#),
                Step::text("done"),
            ];
            let agent = Agent::builder(
                Arc::new(ScriptedProvider::new(steps)),
                Arc::new(MemoryStore::new()),
            )
            .tool(CommandTool {
                spec: ToolSpec::new(
                    "shell",
                    "test command",
                    serde_json::json!({"type":"object"}),
                )
                .risk(Risk::Exec),
                executions: executions.clone(),
            })
            .middleware(Permission::new(
                PermissionMode::Ask,
                move |_: &ApprovalRequest<'_>| {
                    if allowed {
                        Approval::Once
                    } else {
                        Approval::Deny
                    }
                },
            ))
            .middleware(VerificationGuard {
                command: "npm test".into(),
            })
            .unlimited_turns()
            .build()
            .unwrap();
            let root =
                std::env::temp_dir().join(format!("goal-command-{}-{allowed}", std::process::id()));
            let log = agent_kernel::journal::JsonlRunLog::new(&root);
            let mut run = String::new();
            let result = agent.run_parts_observed(
                "s",
                vec![agent_kernel::Part::Text {
                    text: "work".into(),
                }],
                RunOptions::default(),
                &mut |event| {
                    run = event.run_id.clone();
                    log.append(&event)
                },
                &Arc::new(AtomicBool::new(false)),
            );
            assert_eq!(executions.load(Ordering::SeqCst), usize::from(allowed));
            if allowed {
                assert!(result.is_ok());
                assert!(trace_evidence(&log, "s", &run, "npm test")
                    .unwrap()
                    .is_some());
                assert!(trace_evidence(&log, "different-session", &run, "npm test").is_err());
            } else {
                assert!(result.is_err());
                assert!(trace_evidence(&log, "s", &run, "npm test")
                    .unwrap()
                    .is_none());
            }
            drop(log);
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}

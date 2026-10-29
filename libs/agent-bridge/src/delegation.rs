//! External agent delegation manager: launching, tracking, status polling, approvals and cancellation.
//!
//! Claude Code tasks run through the hook-enforced [`ClaudeCode`] backend: every tool outside the read-only
//! allow-list pauses on a `PermissionRequest` hook until the person answers (on this computer or from the phone),
//! and a hook failure leaves the tool denied. Codex tasks still use `codex exec` pinned to a read-only sandbox until
//! its app-server approvals have been verified end to end.

use crate::claude::ClaudeCode;
use crate::observer::{ObservationHub, ObservedEvent};
use crate::{
    AgentEvent, BridgeError, Decision, ExternalAgent, PermissionProfile, Risk, StartRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, MutexGuard,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn clip(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_string(),
    }
}

/// Execution status of a delegated task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationStatus {
    /// Running in background.
    Running,
    /// Successfully finished.
    Completed,
    /// Exited with error.
    Failed,
    /// Interrupted or cancelled.
    Cancelled,
}

/// Snapshot summary of a delegated task for display and status reporting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DelegatedTaskSummary {
    /// Unique task identifier (e.g. `del-1730000000-1`).
    pub task_id: String,
    /// Target agent ("claude" | "codex").
    pub agent: String,
    /// Task prompt given by the user.
    pub prompt: String,
    /// Working directory.
    pub workspace: String,
    /// Timestamp when launched in milliseconds.
    pub started_at_ms: i64,
    /// Current execution state.
    pub status: DelegationStatus,
    /// Recent tail of text output produced by the agent.
    pub recent_output: String,
    /// Pending approval description if waiting for user decision.
    pub pending_approval: Option<String>,
    /// Identifier to pass to [`DelegationManager::approve`] for the pending approval (`<task id>:<request id>`).
    #[serde(default)]
    pub pending_approval_id: Option<String>,
    /// Underlying session ID reported by the agent (e.g. Claude session UUID or Codex thread ID).
    pub session_id: Option<String>,
}

/// Instructions for the task runner.
enum TaskCommand {
    Answer { approval_id: String, allow: bool },
    Cancel,
}

struct PendingApproval {
    id: String,
    description: String,
}

struct TaskEntry {
    task_id: String,
    agent: String,
    prompt: String,
    workspace: PathBuf,
    started_at_ms: i64,
    status: Mutex<DelegationStatus>,
    output_tail: Mutex<String>,
    session_id: Mutex<Option<String>>,
    approvals: Mutex<Vec<PendingApproval>>,
    commands: mpsc::UnboundedSender<TaskCommand>,
    cancel: Arc<AtomicBool>,
}

impl TaskEntry {
    fn summary(&self) -> DelegatedTaskSummary {
        let status = *lock(&self.status);
        let recent_output = lock(&self.output_tail).clone();
        let session_id = lock(&self.session_id).clone();
        let (pending_approval, pending_approval_id) = match lock(&self.approvals).first() {
            Some(a) => (Some(a.description.clone()), Some(format!("{}:{}", self.task_id, a.id))),
            None => (None, None),
        };
        DelegatedTaskSummary {
            task_id: self.task_id.clone(),
            agent: self.agent.clone(),
            prompt: self.prompt.clone(),
            workspace: self.workspace.display().to_string(),
            started_at_ms: self.started_at_ms,
            status,
            recent_output,
            pending_approval,
            pending_approval_id,
            session_id,
        }
    }

    fn append_output(&self, text: &str) {
        let mut out = lock(&self.output_tail);
        out.push_str(text);
        if out.chars().count() > 4000 {
            let keep: String = out.chars().rev().take(3000).collect();
            *out = keep.chars().rev().collect();
        }
    }

    /// Move out of `Running` exactly once. Returns false when someone else (for example a cancel) already did.
    fn finish(&self, status: DelegationStatus) -> bool {
        let mut current = lock(&self.status);
        if *current != DelegationStatus::Running {
            return false;
        }
        *current = status;
        lock(&self.approvals).clear();
        true
    }
}

/// Central manager coordinating external agent task delegation and lifecycle.
#[derive(Clone)]
pub struct DelegationManager {
    tasks: Arc<Mutex<HashMap<String, Arc<TaskEntry>>>>,
    hub: Arc<ObservationHub>,
    seq: Arc<AtomicU64>,
    claude: ClaudeCode,
}

impl Default for DelegationManager {
    fn default() -> Self {
        Self::with_hub(Arc::new(ObservationHub::new()))
    }
}

impl DelegationManager {
    /// Create a new delegation manager with default observation hub.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with an existing observation hub so events route to speech and UI.
    pub fn with_hub(hub: Arc<ObservationHub>) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            hub,
            seq: Arc::new(AtomicU64::new(1)),
            claude: ClaudeCode::default(),
        }
    }

    /// Use a specific Claude Code backend (a local installation path, or a test double).
    pub fn with_claude(mut self, claude: ClaudeCode) -> Self {
        self.claude = claude;
        self
    }

    /// Access the observation hub.
    pub fn hub(&self) -> &Arc<ObservationHub> {
        &self.hub
    }

    fn next_id(&self) -> String {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        format!("del-{}-{n}", now_ms())
    }

    /// Launch a delegated task using the requested agent ("claude" | "codex").
    pub async fn delegate(
        &self,
        requested_agent: &str,
        prompt: &str,
        workspace: &Path,
    ) -> Result<DelegatedTaskSummary, String> {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return Err("委派任务的描述不能为空".into());
        }
        if !workspace.is_absolute() || !workspace.is_dir() {
            return Err("工作目录必须是一个已存在的绝对路径".into());
        }

        // 识别目标 Agent
        let agent_name = match requested_agent.trim().to_lowercase().as_str() {
            "claude" | "claude-code" | "claudecode" => "claude",
            "codex" | "openai-codex" => "codex",
            "auto" | "" => {
                if self.claude.availability_installed().await {
                    "claude"
                } else if crate::environment::detect_environment().await.codex.installed {
                    "codex"
                } else {
                    return Err("本机未检测到可用的 Claude Code 或 Codex，请先在设置中安装".into());
                }
            }
            other => return Err(format!("不支持的外部 Agent：「{other}」，目前支持 claude 和 codex")),
        };

        let codex_path = crate::process::default_path("codex");
        if agent_name == "claude" {
            if !self.claude.availability_installed().await {
                return Err("未找到 claude 可执行文件，请先安装 Claude Code".into());
            }
        } else if !codex_path.exists() {
            return Err("未在 PATH 中找到 codex 可执行文件，请先安装".into());
        }

        let task_id = self.next_id();
        let cancel = Arc::new(AtomicBool::new(false));
        let (commands, command_rx) = mpsc::unbounded_channel();
        let entry = Arc::new(TaskEntry {
            task_id: task_id.clone(),
            agent: agent_name.to_string(),
            prompt: prompt.to_string(),
            workspace: workspace.to_path_buf(),
            started_at_ms: now_ms(),
            status: Mutex::new(DelegationStatus::Running),
            output_tail: Mutex::new(String::new()),
            session_id: Mutex::new(None),
            approvals: Mutex::new(Vec::new()),
            commands,
            cancel: cancel.clone(),
        });

        lock(&self.tasks).insert(task_id.clone(), entry.clone());

        let hub = self.hub.clone();
        let prompt = prompt.to_string();
        let workspace = workspace.to_path_buf();
        if agent_name == "claude" {
            let claude = self.claude.clone();
            let entry = entry.clone();
            tokio::spawn(async move {
                run_claude_task(entry, hub, claude, prompt, workspace, command_rx).await;
            });
        } else {
            let entry = entry.clone();
            tokio::spawn(async move {
                run_codex_exec(entry, hub, codex_path, prompt, workspace, cancel).await;
            });
        }

        Ok(entry.summary())
    }

    /// Query current status and recent output tail of a delegated task.
    pub fn status(&self, task_id: &str) -> Option<DelegatedTaskSummary> {
        lock(&self.tasks).get(task_id).map(|e| e.summary())
    }

    /// Resolve a pending approval for a running delegated task. `approval_id` is the id from
    /// [`DelegatedTaskSummary::pending_approval_id`] (the bare request id is accepted too); anything that is not
    /// currently pending is refused.
    pub fn approve(&self, task_id: &str, approval_id: &str, decision: &str) -> Result<(), String> {
        let entry = lock(&self.tasks)
            .get(task_id)
            .cloned()
            .ok_or_else(|| format!("未找到委派任务 {task_id}"))?;
        let bare = approval_id
            .strip_prefix(task_id)
            .and_then(|rest| rest.strip_prefix(':'))
            .unwrap_or(approval_id);
        let allow = matches!(decision, "allow" | "once" | "approve");
        {
            let mut approvals = lock(&entry.approvals);
            let position = approvals
                .iter()
                .position(|a| a.id == bare)
                .ok_or("该任务当前没有这条等待处理的审批（可能已被处理或已过期）")?;
            approvals.remove(position);
        }
        entry
            .commands
            .send(TaskCommand::Answer { approval_id: bare.to_string(), allow })
            .map_err(|_| "任务已经结束".to_string())
    }

    /// Cancel and interrupt a running delegated task.
    pub fn cancel(&self, task_id: &str) -> Result<(), String> {
        let entry = lock(&self.tasks)
            .get(task_id)
            .cloned()
            .ok_or_else(|| format!("未找到委派任务 {task_id}"))?;
        entry.cancel.store(true, Ordering::SeqCst);
        if entry.finish(DelegationStatus::Cancelled) {
            entry.append_output("\n[任务已被用户主动打断取消]");
            self.hub.dispatch(ObservedEvent::TaskCompleted {
                session_id: task_id.to_string(),
                summary: "任务已被取消".into(),
                success: false,
            });
        }
        let _ = entry.commands.send(TaskCommand::Cancel);
        Ok(())
    }

    /// List all delegated tasks, newest first.
    pub fn list(&self) -> Vec<DelegatedTaskSummary> {
        let mut list: Vec<_> = lock(&self.tasks).values().map(|e| e.summary()).collect();
        list.sort_by_key(|t| -t.started_at_ms);
        list
    }
}

fn start_error_text(error: &BridgeError) -> &'static str {
    match error {
        BridgeError::NotInstalled => "未找到 claude 可执行文件，请先安装 Claude Code",
        BridgeError::NotLoggedIn => "Claude Code 还没有登录，请先在终端里登录一次",
        BridgeError::Timeout => "Claude Code 启动超时",
        BridgeError::Unsupported => "当前版本不允许启动外部任务",
        BridgeError::InvalidArgument => "启动参数不正确",
        _ => "Claude Code 启动失败或意外退出",
    }
}

fn complete(entry: &TaskEntry, hub: &ObservationHub, status: DelegationStatus, summary: String) {
    if entry.finish(status) {
        hub.dispatch(ObservedEvent::TaskCompleted {
            session_id: entry.task_id.clone(),
            summary,
            success: status == DelegationStatus::Completed,
        });
    }
}

/// Runs one Claude Code task on the hook-enforced backend until its first turn completes.
async fn run_claude_task(
    entry: Arc<TaskEntry>,
    hub: Arc<ObservationHub>,
    claude: ClaudeCode,
    prompt: String,
    workspace: PathBuf,
    mut commands: mpsc::UnboundedReceiver<TaskCommand>,
) {
    let task_id = entry.task_id.clone();
    hub.dispatch(ObservedEvent::SessionStarted {
        session_id: task_id.clone(),
        agent: "claude".into(),
    });

    let request = StartRequest {
        cwd: workspace,
        prompt,
        profile: PermissionProfile::Strict,
        model: None,
        name: Some(format!("小伴委派 {task_id}")),
    };
    let mut session = match claude.start(request).await {
        Ok(session) => session,
        Err(error) => {
            let text = start_error_text(&error);
            entry.append_output(text);
            complete(&entry, &hub, DelegationStatus::Failed, text.to_string());
            return;
        }
    };
    *lock(&entry.session_id) = Some(session.id().to_string());
    let mut events = session.events();
    let mut tool_names: HashMap<String, String> = HashMap::new();
    let mut turn_failed = false;
    let mut commands_open = true;
    let mut outcome: Option<(DelegationStatus, String)> = None;

    while outcome.is_none() {
        tokio::select! {
            event = events.recv() => {
                let Some(event) = event else { break };
                match event {
                    AgentEvent::SessionStarted { session: id } => {
                        *lock(&entry.session_id) = Some(id);
                    }
                    AgentEvent::Message { text } => {
                        entry.append_output(&text);
                        entry.append_output("\n");
                    }
                    AgentEvent::ToolStarted { id, name, summary } => {
                        tool_names.insert(id, name.clone());
                        hub.dispatch(ObservedEvent::ToolStarted {
                            session_id: task_id.clone(),
                            tool_name: name,
                            summary: clip(&summary, 500),
                        });
                    }
                    AgentEvent::ToolFinished { id, ok, preview } => {
                        hub.dispatch(ObservedEvent::ToolFinished {
                            session_id: task_id.clone(),
                            tool_name: tool_names.remove(&id).unwrap_or_default(),
                            ok,
                            preview: clip(&preview, 500),
                        });
                    }
                    AgentEvent::ApprovalRequested(approval) => {
                        let description = clip(&format!("{}：{}", approval.title, approval.detail), 500);
                        lock(&entry.approvals).push(PendingApproval { id: approval.id.clone(), description });
                        hub.dispatch(ObservedEvent::PermissionRequested {
                            session_id: task_id.clone(),
                            request_id: format!("{task_id}:{}", approval.id),
                            tool_name: approval.title,
                            detail: clip(&approval.detail, 2000),
                            risk: if approval.risk == Risk::High { "high" } else { "low" }.into(),
                        });
                    }
                    AgentEvent::ApprovalResolved { id } => {
                        lock(&entry.approvals).retain(|a| a.id != id);
                    }
                    AgentEvent::Error { message, code } => {
                        if code == "agent" {
                            turn_failed = true;
                        }
                        entry.append_output(&format!("[{message}]\n"));
                    }
                    AgentEvent::TurnCompleted { final_text, .. } => {
                        let text = final_text.unwrap_or_default();
                        if lock(&entry.output_tail).trim().is_empty() && !text.is_empty() {
                            entry.append_output(&text);
                        }
                        let status = if turn_failed { DelegationStatus::Failed } else { DelegationStatus::Completed };
                        let summary = if text.is_empty() { "任务已结束".to_string() } else { clip(&text, 300) };
                        outcome = Some((status, summary));
                    }
                    AgentEvent::Closed => {
                        outcome = Some((DelegationStatus::Failed, "Claude Code 进程意外退出".into()));
                    }
                    AgentEvent::Status { .. } | AgentEvent::TextDelta { .. } | AgentEvent::FileChanged { .. } => {}
                }
            }
            command = commands.recv(), if commands_open => {
                match command {
                    Some(TaskCommand::Answer { approval_id, allow }) => {
                        let decision = if allow { Decision::Allow } else { Decision::Deny };
                        if session.answer(&approval_id, decision).await.is_err() {
                            entry.append_output("[审批已失效，未能送达]\n");
                        }
                    }
                    Some(TaskCommand::Cancel) => {
                        let _ = session.interrupt().await;
                        outcome = Some((DelegationStatus::Cancelled, "任务已被取消".into()));
                    }
                    None => commands_open = false,
                }
            }
        }
    }

    let (status, summary) = outcome.unwrap_or((DelegationStatus::Failed, "Claude Code 进程意外退出".into()));
    let _ = session.close().await;
    complete(&entry, &hub, status, summary);
}

/// Codex still runs through `codex exec`, pinned to a read-only sandbox: it cannot ask the person for approval
/// here, so it must not be able to write or run anything with side effects.
async fn run_codex_exec(
    entry: Arc<TaskEntry>,
    hub: Arc<ObservationHub>,
    exe_path: PathBuf,
    prompt: String,
    workspace: PathBuf,
    cancel: Arc<AtomicBool>,
) {
    let task_id = entry.task_id.clone();
    hub.dispatch(ObservedEvent::SessionStarted {
        session_id: task_id.clone(),
        agent: "codex".into(),
    });

    let mut cmd = crate::process::command(&exe_path);
    cmd.current_dir(&workspace);
    cmd.args(["exec", "--sandbox", "read-only", "--skip-git-repo-check", &prompt]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            entry.append_output(&format!("无法启动进程：{e}"));
            complete(&entry, &hub, DelegationStatus::Failed, format!("启动失败：{e}"));
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let entry_for_stdout = entry.clone();
    let stdout_reader = tokio::spawn(async move {
        if let Some(stdout) = stdout {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                entry_for_stdout.append_output(&line);
                entry_for_stdout.append_output("\n");
            }
        }
    });
    let entry_for_stderr = entry.clone();
    let stderr_reader = tokio::spawn(async move {
        if let Some(stderr) = stderr {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                entry_for_stderr.append_output(&line);
                entry_for_stderr.append_output("\n");
            }
        }
    });

    // 等待进程完成或被取消
    let status_res = loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = child.kill().await;
            break Ok(DelegationStatus::Cancelled);
        }
        match tokio::time::timeout(Duration::from_millis(200), child.wait()).await {
            Ok(Ok(exit_status)) => {
                break Ok(if exit_status.success() { DelegationStatus::Completed } else { DelegationStatus::Failed });
            }
            Ok(Err(e)) => break Err(e.to_string()),
            Err(_) => continue,
        }
    };

    let _ = stdout_reader.await;
    let _ = stderr_reader.await;

    let final_status = match status_res {
        Ok(s) => s,
        Err(e) => {
            entry.append_output(&format!("\n等待退出出错：{e}"));
            DelegationStatus::Failed
        }
    };
    let summary_text: String = lock(&entry.output_tail)
        .lines()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ");
    complete(&entry, &hub, final_status, summary_text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_summary_uses_camel_case_keys_and_snake_case_status() {
        let (entry, _commands) = manual_entry("del-json");
        let value = serde_json::to_value(entry.summary()).unwrap();
        for key in ["taskId", "agent", "prompt", "workspace", "startedAtMs", "status", "recentOutput", "pendingApproval", "pendingApprovalId", "sessionId"] {
            assert!(value.get(key).is_some(), "缺少 {key}：{value}");
        }
        assert_eq!(value["status"], "running");
    }

    fn manual_entry(task_id: &str) -> (Arc<TaskEntry>, mpsc::UnboundedReceiver<TaskCommand>) {
        let (commands, rx) = mpsc::unbounded_channel();
        let entry = Arc::new(TaskEntry {
            task_id: task_id.into(),
            agent: "claude".into(),
            prompt: "修好测试".into(),
            workspace: PathBuf::from("."),
            started_at_ms: now_ms(),
            status: Mutex::new(DelegationStatus::Running),
            output_tail: Mutex::new(String::new()),
            session_id: Mutex::new(Some("uuid-123".into())),
            approvals: Mutex::new(Vec::new()),
            commands,
            cancel: Arc::new(AtomicBool::new(false)),
        });
        (entry, rx)
    }

    fn fake_manager() -> DelegationManager {
        let claude = ClaudeCode::new(crate::test_support::fake("claude")).with_timeout(Duration::from_secs(3));
        DelegationManager::new().with_claude(claude)
    }

    async fn wait_for(mgr: &DelegationManager, id: &str, what: &str, done: impl Fn(&DelegatedTaskSummary) -> bool) -> DelegatedTaskSummary {
        for _ in 0..2400 {
            let s = mgr.status(id).unwrap();
            if done(&s) {
                return s;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("等待超时：{what}；当前 {:?}", mgr.status(id));
    }

    #[tokio::test]
    async fn claude_task_waits_for_approval_then_completes_when_allowed() {
        let mgr = fake_manager();
        let mut hub = mgr.hub().subscribe();
        let cwd = std::env::current_dir().unwrap();
        let task = mgr.delegate("claude", "first", &cwd).await.unwrap();
        let id = task.task_id;

        let waiting = wait_for(&mgr, &id, "出现待审批", |s| s.pending_approval_id.is_some()).await;
        assert_eq!(waiting.status, DelegationStatus::Running);
        let approval_id = waiting.pending_approval_id.unwrap();
        assert!(approval_id.starts_with(&format!("{id}:")));

        mgr.approve(&id, &approval_id, "allow").unwrap();
        let done = wait_for(&mgr, &id, "任务完成", |s| s.status != DelegationStatus::Running).await;
        assert_eq!(done.status, DelegationStatus::Completed);
        assert!(done.session_id.is_some());
        assert!(done.pending_approval.is_none());

        let mut saw_request = false;
        let mut saw_done = false;
        while let Ok(event) = hub.try_recv() {
            match event {
                ObservedEvent::PermissionRequested { request_id, .. } => saw_request = request_id == approval_id,
                ObservedEvent::TaskCompleted { success, .. } => saw_done = success,
                _ => {}
            }
        }
        assert!(saw_request && saw_done, "观察层应收到审批请求与完成事件");
    }

    #[tokio::test]
    async fn claude_task_denial_reaches_the_agent() {
        let mgr = fake_manager();
        let cwd = std::env::current_dir().unwrap();
        let id = mgr.delegate("claude", "first", &cwd).await.unwrap().task_id;
        let waiting = wait_for(&mgr, &id, "出现待审批", |s| s.pending_approval_id.is_some()).await;
        mgr.approve(&id, &waiting.pending_approval_id.unwrap(), "deny").unwrap();
        let done = wait_for(&mgr, &id, "任务结束", |s| s.status != DelegationStatus::Running).await;
        assert!(matches!(done.status, DelegationStatus::Completed | DelegationStatus::Failed));
        assert!(mgr.approve(&id, "anything", "allow").is_err());
    }

    #[tokio::test]
    async fn cancelling_a_task_stops_the_agent_process() {
        let mgr = fake_manager();
        let cwd = std::env::current_dir().unwrap();
        let id = mgr.delegate("claude", "first", &cwd).await.unwrap().task_id;
        wait_for(&mgr, &id, "出现待审批", |s| s.pending_approval_id.is_some()).await;
        mgr.cancel(&id).unwrap();
        let s = wait_for(&mgr, &id, "取消生效", |s| s.status == DelegationStatus::Cancelled).await;
        assert!(s.pending_approval.is_none(), "取消后不应再有待审批");
    }

    /// Opt-in only (`XIAOBAN_REAL_CLAUDE=1`): runs the installed Claude Code through the manager and spends model usage.
    #[tokio::test]
    async fn real_claude_delegation_pauses_for_approval_before_running_a_command() {
        if std::env::var("XIAOBAN_REAL_CLAUDE").as_deref() != Ok("1") {
            return;
        }
        let mgr = DelegationManager::new();
        if !mgr.claude.availability_installed().await {
            eprintln!("Real Claude test not run: CLI not installed");
            return;
        }
        let dir = std::env::temp_dir().join(format!("xiaoban-delegate-real-{}-{}", std::process::id(), now_ms()));
        std::fs::create_dir(&dir).unwrap();
        let target = dir.join("delegated.txt");
        let id = mgr
            .delegate("claude", "Use the Bash tool to run exactly `touch delegated.txt` in the current directory, then stop.", &dir)
            .await
            .unwrap()
            .task_id;
        let waiting = wait_for(&mgr, &id, "Bash 审批", |s| s.pending_approval_id.is_some() || s.status != DelegationStatus::Running).await;
        let approval_id = waiting.pending_approval_id.expect("命令执行前必须先出现审批");
        assert!(!target.exists(), "批准之前命令不能已经执行");
        mgr.approve(&id, &approval_id, "allow").unwrap();
        let done = wait_for(&mgr, &id, "任务完成", |s| s.status != DelegationStatus::Running).await;
        let created = target.is_file();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(done.status, DelegationStatus::Completed, "{:?}", done.recent_output);
        assert!(created, "批准之后命令应已执行");
    }

    #[tokio::test]
    async fn a_missing_claude_is_reported_before_anything_starts() {
        let mgr = DelegationManager::new().with_claude(ClaudeCode::new("/nonexistent/claude-for-test"));
        let cwd = std::env::current_dir().unwrap();
        let error = mgr.delegate("claude", "做点事", &cwd).await.unwrap_err();
        assert!(error.contains("claude"));
        assert!(mgr.list().is_empty());
    }

    #[tokio::test]
    async fn delegation_manager_tracks_task_lifecycle_and_cancellation() {
        let mgr = DelegationManager::new();
        let mut rx = mgr.hub().subscribe();

        // 验证空描述与非法目录被拒绝
        assert!(mgr.delegate("claude", "", Path::new(".")).await.is_err());
        assert!(mgr.delegate("claude", "做点事", Path::new("relative")).await.is_err());

        let task_id = "del-test-1".to_string();
        let (entry, _commands) = manual_entry(&task_id);
        lock(&mgr.tasks).insert(task_id.clone(), entry);

        let snap = mgr.status(&task_id).unwrap();
        assert_eq!(snap.status, DelegationStatus::Running);
        assert_eq!(snap.session_id.as_deref(), Some("uuid-123"));

        assert!(mgr.cancel(&task_id).is_ok());
        let after_cancel = mgr.status(&task_id).unwrap();
        assert_eq!(after_cancel.status, DelegationStatus::Cancelled);
        assert!(after_cancel.recent_output.contains("取消"));

        match rx.recv().await.unwrap() {
            ObservedEvent::TaskCompleted { session_id, success, .. } => {
                assert_eq!(session_id, "del-test-1");
                assert!(!success);
            }
            _ => panic!("unexpected event"),
        }
        // 重复取消不会再广播一次完成事件
        assert!(mgr.cancel(&task_id).is_ok());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn approve_only_accepts_what_is_currently_pending() {
        let mgr = DelegationManager::new();
        let (entry, mut commands) = manual_entry("del-a");
        lock(&entry.approvals).push(PendingApproval { id: "req-1".into(), description: "Bash: touch x".into() });
        lock(&mgr.tasks).insert("del-a".into(), entry.clone());

        assert_eq!(mgr.status("del-a").unwrap().pending_approval_id.as_deref(), Some("del-a:req-1"));
        assert!(mgr.approve("del-a", "req-other", "allow").is_err(), "不认识的审批编号必须被拒绝");
        assert!(mgr.approve("del-missing", "req-1", "allow").is_err());

        mgr.approve("del-a", "del-a:req-1", "allow").unwrap();
        assert!(matches!(commands.try_recv(), Ok(TaskCommand::Answer { approval_id, allow: true }) if approval_id == "req-1"));
        assert!(mgr.status("del-a").unwrap().pending_approval.is_none());
        assert!(mgr.approve("del-a", "req-1", "allow").is_err(), "同一条审批不能答两次");
    }

    #[test]
    fn anything_but_an_explicit_allow_is_a_denial() {
        let mgr = DelegationManager::new();
        let (entry, mut commands) = manual_entry("del-b");
        lock(&entry.approvals).push(PendingApproval { id: "req-2".into(), description: "x".into() });
        lock(&mgr.tasks).insert("del-b".into(), entry);
        mgr.approve("del-b", "req-2", "maybe").unwrap();
        assert!(matches!(commands.try_recv(), Ok(TaskCommand::Answer { allow: false, .. })));
    }
}

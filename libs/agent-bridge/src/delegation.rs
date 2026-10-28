//! External agent delegation manager: launching, tracking, status polling, approvals and cancellation.

use crate::environment;
use crate::observer::{ObservationHub, ObservedEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, MutexGuard,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
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
    /// Underlying session ID reported by the agent (e.g. Claude session UUID or Codex thread ID).
    pub session_id: Option<String>,
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
    pending_approval: Mutex<Option<(String, oneshot::Sender<String>)>>,
    cancel: Arc<AtomicBool>,
}

impl TaskEntry {
    fn summary(&self) -> DelegatedTaskSummary {
        let status = *lock(&self.status);
        let recent_output = lock(&self.output_tail).clone();
        let session_id = lock(&self.session_id).clone();
        let pending_approval = lock(&self.pending_approval).as_ref().map(|(desc, _)| desc.clone());
        DelegatedTaskSummary {
            task_id: self.task_id.clone(),
            agent: self.agent.clone(),
            prompt: self.prompt.clone(),
            workspace: self.workspace.display().to_string(),
            started_at_ms: self.started_at_ms,
            status,
            recent_output,
            pending_approval,
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
}

/// Central manager coordinating external agent task delegation and lifecycle.
#[derive(Clone)]
pub struct DelegationManager {
    tasks: Arc<Mutex<HashMap<String, Arc<TaskEntry>>>>,
    hub: Arc<ObservationHub>,
    seq: Arc<AtomicU64>,
}

impl Default for DelegationManager {
    fn default() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            hub: Arc::new(ObservationHub::new()),
            seq: Arc::new(AtomicU64::new(1)),
        }
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
        }
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

        // 识别目标 Agent
        let (agent_name, exe_name) = match requested_agent.trim().to_lowercase().as_str() {
            "claude" | "claude-code" | "claudecode" => ("claude", "claude"),
            "codex" | "openai-codex" => ("codex", "codex"),
            "auto" | "" => {
                let env = environment::detect_environment().await;
                if env.claude.installed {
                    ("claude", "claude")
                } else if env.codex.installed {
                    ("codex", "codex")
                } else {
                    return Err("本机未检测到可用的 Claude Code 或 Codex，请先在设置中安装".into());
                }
            }
            other => return Err(format!("不支持的外部 Agent：「{other}」，目前支持 claude 和 codex")),
        };

        let exe_path = crate::process::default_path(exe_name);
        if !exe_path.exists() {
            return Err(format!("未在 PATH 中找到 {agent_name} 可执行文件，请先安装"));
        }

        let task_id = self.next_id();
        let cancel = Arc::new(AtomicBool::new(false));
        let entry = Arc::new(TaskEntry {
            task_id: task_id.clone(),
            agent: agent_name.to_string(),
            prompt: prompt.to_string(),
            workspace: workspace.to_path_buf(),
            started_at_ms: now_ms(),
            status: Mutex::new(DelegationStatus::Running),
            output_tail: Mutex::new(String::new()),
            session_id: Mutex::new(None),
            pending_approval: Mutex::new(None),
            cancel: cancel.clone(),
        });

        lock(&self.tasks).insert(task_id.clone(), entry.clone());

        // 启动后台进程
        let entry_clone = entry.clone();
        let hub_clone = self.hub.clone();
        let workspace_owned = workspace.to_path_buf();
        let prompt_owned = prompt.to_string();

        tokio::spawn(async move {
            run_delegated_process(
                entry_clone,
                hub_clone,
                exe_path,
                agent_name,
                prompt_owned,
                workspace_owned,
                cancel,
            )
            .await;
        });

        Ok(entry.summary())
    }

    /// Query current status and recent output tail of a delegated task.
    pub fn status(&self, task_id: &str) -> Option<DelegatedTaskSummary> {
        lock(&self.tasks).get(task_id).map(|e| e.summary())
    }

    /// Resolve a pending approval for a running delegated task.
    pub fn approve(&self, task_id: &str, _approval_id: &str, decision: &str) -> Result<(), String> {
        let entry = lock(&self.tasks)
            .get(task_id)
            .cloned()
            .ok_or_else(|| format!("未找到委派任务 {task_id}"))?;
        let sender = lock(&entry.pending_approval)
            .take()
            .map(|(_, tx)| tx)
            .ok_or("该任务当前没有等待处理的审批")?;
        let _ = sender.send(decision.to_string());
        Ok(())
    }

    /// Cancel and interrupt a running delegated task.
    pub fn cancel(&self, task_id: &str) -> Result<(), String> {
        let entry = lock(&self.tasks)
            .get(task_id)
            .cloned()
            .ok_or_else(|| format!("未找到委派任务 {task_id}"))?;
        entry.cancel.store(true, Ordering::SeqCst);
        *lock(&entry.status) = DelegationStatus::Cancelled;
        entry.append_output("\n[任务已被用户主动打断取消]");
        self.hub.dispatch(ObservedEvent::TaskCompleted {
            session_id: task_id.to_string(),
            summary: "任务已被取消".into(),
            success: false,
        });
        Ok(())
    }

    /// List all delegated tasks, newest first.
    pub fn list(&self) -> Vec<DelegatedTaskSummary> {
        let mut list: Vec<_> = lock(&self.tasks).values().map(|e| e.summary()).collect();
        list.sort_by_key(|t| -t.started_at_ms);
        list
    }
}

async fn run_delegated_process(
    entry: Arc<TaskEntry>,
    hub: Arc<ObservationHub>,
    exe_path: PathBuf,
    agent_name: &str,
    prompt: String,
    workspace: PathBuf,
    cancel: Arc<AtomicBool>,
) {
    let task_id = entry.task_id.clone();
    hub.dispatch(ObservedEvent::SessionStarted {
        session_id: task_id.clone(),
        agent: agent_name.to_string(),
    });

    let mut cmd = crate::process::command(&exe_path);
    cmd.current_dir(&workspace);

    if agent_name == "claude" {
        cmd.args(["-p", &prompt, "--output-format", "stream-json", "--verbose"]);
    } else {
        // Codex
        cmd.args(["exec", &prompt]);
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            *lock(&entry.status) = DelegationStatus::Failed;
            entry.append_output(&format!("无法启动进程：{e}"));
            hub.dispatch(ObservedEvent::TaskCompleted {
                session_id: task_id,
                summary: format!("启动失败：{e}"),
                success: false,
            });
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let entry_for_stdout = entry.clone();
    let hub_for_stdout = hub.clone();
    let task_id_for_stdout = task_id.clone();

    let stdout_reader = tokio::spawn(async move {
        if let Some(stdout) = stdout {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(json_obj) = serde_json::from_str::<Value>(&line) {
                    handle_stream_json_line(&entry_for_stdout, &hub_for_stdout, &task_id_for_stdout, &json_obj);
                } else {
                    entry_for_stdout.append_output(&line);
                    entry_for_stdout.append_output("\n");
                }
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
                let s = if exit_status.success() {
                    DelegationStatus::Completed
                } else {
                    DelegationStatus::Failed
                };
                break Ok(s);
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

    *lock(&entry.status) = final_status;
    let summary_text: String = lock(&entry.output_tail)
        .lines()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ");

    hub.dispatch(ObservedEvent::TaskCompleted {
        session_id: task_id,
        summary: summary_text,
        success: final_status == DelegationStatus::Completed,
    });
}

fn handle_stream_json_line(
    entry: &TaskEntry,
    _hub: &ObservationHub,
    _task_id: &str,
    obj: &Value,
) {
    let event_type = obj.get("type").and_then(Value::as_str).unwrap_or("");
    match event_type {
        "system" => {
            if obj.get("subtype").and_then(Value::as_str) == Some("init") {
                if let Some(sid) = obj.get("session_id").and_then(Value::as_str) {
                    *lock(&entry.session_id) = Some(sid.to_string());
                }
            }
        }
        "assistant" => {
            if let Some(content) = obj.get("message").and_then(|m| m.get("content")).and_then(Value::as_array) {
                for item in content {
                    if item.get("type").and_then(Value::as_str) == Some("text") {
                        if let Some(text) = item.get("text").and_then(Value::as_str) {
                            entry.append_output(text);
                            entry.append_output("\n");
                        }
                    }
                }
            }
        }
        "result" => {
            if let Some(res) = obj.get("result").and_then(Value::as_str) {
                entry.append_output(res);
                entry.append_output("\n");
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn delegation_manager_tracks_task_lifecycle_and_cancellation() {
        let mgr = DelegationManager::new();
        let mut rx = mgr.hub().subscribe();

        // 验证空描述被拒绝
        assert!(mgr.delegate("claude", "", Path::new(".")).await.is_err());

        // 模拟手动插入一个委派任务
        let task_id = "del-test-1".to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        let entry = Arc::new(TaskEntry {
            task_id: task_id.clone(),
            agent: "claude".into(),
            prompt: "修好测试".into(),
            workspace: PathBuf::from("."),
            started_at_ms: now_ms(),
            status: Mutex::new(DelegationStatus::Running),
            output_tail: Mutex::new(String::new()),
            session_id: Mutex::new(Some("uuid-123".into())),
            pending_approval: Mutex::new(None),
            cancel: cancel.clone(),
        });
        lock(&mgr.tasks).insert(task_id.clone(), entry);

        let snap = mgr.status(&task_id).unwrap();
        assert_eq!(snap.task_id, "del-test-1");
        assert_eq!(snap.status, DelegationStatus::Running);
        assert_eq!(snap.session_id.as_deref(), Some("uuid-123"));

        // 取消任务
        assert!(mgr.cancel(&task_id).is_ok());
        let after_cancel = mgr.status(&task_id).unwrap();
        assert_eq!(after_cancel.status, DelegationStatus::Cancelled);
        assert!(after_cancel.recent_output.contains("取消"));

        let received = rx.recv().await.unwrap();
        match received {
            ObservedEvent::TaskCompleted { session_id, success, .. } => {
                assert_eq!(session_id, "del-test-1");
                assert!(!success);
            }
            _ => panic!("unexpected event"),
        }
    }
}

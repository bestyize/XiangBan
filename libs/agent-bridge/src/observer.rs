//! Observation layer for external coding agents (Claude Code, Codex).
//!
//! Subscribes to events from loopback hooks and background streams, translating
//! them into unified notification events and speech announcements.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use xiaoban_agent_hooks::claude_settings::{self, InstallSpec, Scope};
use xiaoban_agent_hooks::events::HookEvent;

/// Unified events captured from an external agent during its execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum ObservedEvent {
    /// A session has started.
    SessionStarted {
        /// Session identifier.
        session_id: String,
        /// Name of the agent.
        agent: String,
    },
    /// A tool execution has started.
    ToolStarted {
        /// Session identifier.
        session_id: String,
        /// Name of the tool being executed.
        tool_name: String,
        /// Summary or serialized parameters.
        summary: String,
    },
    /// A tool execution has ended.
    ToolFinished {
        /// Session identifier.
        session_id: String,
        /// Name of the tool.
        tool_name: String,
        /// Whether tool execution succeeded.
        ok: bool,
        /// Preview of the tool output.
        preview: String,
    },
    /// The agent is paused awaiting permission approval.
    PermissionRequested {
        /// Session identifier.
        session_id: String,
        /// Approval request identifier.
        request_id: String,
        /// Tool name requiring permission.
        tool_name: String,
        /// Detailed description or parameters.
        detail: String,
        /// Risk level ("low" | "high").
        risk: String,
    },
    /// A task / turn has completed.
    TaskCompleted {
        /// Session identifier.
        session_id: String,
        /// Completion summary or last assistant response.
        summary: String,
        /// Whether the task was successful.
        success: bool,
    },
    /// An informational notification from the agent.
    Notification {
        /// Session identifier.
        session_id: String,
        /// Notification message.
        message: String,
    },
}

impl ObservedEvent {
    /// Generate a short, natural Chinese spoken announcement for voice broadcasting (P4).
    pub fn announcement(&self) -> Option<String> {
        match self {
            Self::TaskCompleted { summary, success, .. } => {
                if *success {
                    Some(if summary.is_empty() {
                        "小伴提示你：任务已完成。".into()
                    } else {
                        format!("小伴提示你：任务已完成。{summary}")
                    })
                } else {
                    Some(format!("小伴提示你：任务执行遇到问题。{summary}"))
                }
            }
            Self::PermissionRequested { tool_name, risk, .. } => {
                let risk_hint = if risk == "high" { "高风险操作" } else { "操作" };
                Some(format!("等待你确认：是否允许执行{risk_hint} {tool_name}？"))
            }
            Self::Notification { message, .. } if !message.trim().is_empty() => {
                Some(format!("提示：{message}"))
            }
            _ => None,
        }
    }
}

/// Status of Claude Code hooks configuration in a given scope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HooksStatus {
    /// Scope description.
    pub scope: String,
    /// Settings file path.
    pub settings_path: String,
    /// Whether xiaoban-hook is configured.
    pub is_installed: bool,
    /// Registered hook events.
    pub events: Vec<String>,
    /// Whether other third-party hooks are present.
    pub other_hooks: bool,
}

/// Inspect Claude Code hooks status in a project directory or user profile.
pub fn inspect_claude_hooks(project_dir: Option<&Path>) -> Result<HooksStatus, String> {
    let scope = match project_dir {
        Some(dir) => Scope::Project(dir.to_path_buf()),
        None => Scope::User,
    };
    let settings_path = match &scope {
        Scope::Project(p) => p.join(".claude").join("settings.json"),
        Scope::ProjectLocal(p) => p.join(".claude").join("settings.local.json"),
        Scope::User => std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("USERPROFILE")
                    .or_else(|| std::env::var_os("HOME"))
                    .map(|h| PathBuf::from(h).join(".claude"))
            })
            .unwrap_or_else(|| PathBuf::from(".claude"))
            .join("settings.json"),
    };
    let st = claude_settings::status(&scope).map_err(|e| e.to_string())?;
    let events = st.installed.iter().map(|(ev, _, _)| ev.clone()).collect();
    Ok(HooksStatus {
        scope: format!("{scope:?}"),
        settings_path: settings_path.display().to_string(),
        is_installed: !st.installed.is_empty(),
        events,
        other_hooks: st.other_hooks,
    })
}

/// Install Claude Code hooks into a project or user scope, automatically backing up old settings first.
pub fn install_claude_hooks(project_dir: Option<&Path>, hook_executable: &Path) -> Result<Option<String>, String> {
    let scope = match project_dir {
        Some(dir) => Scope::Project(dir.to_path_buf()),
        None => Scope::User,
    };
    let spec = InstallSpec::new(hook_executable.to_path_buf());
    let plan = claude_settings::plan_install(&scope, &spec).map_err(|e| e.to_string())?;
    let applied = claude_settings::apply(&plan).map_err(|e| e.to_string())?;
    Ok(applied.backup.map(|p| p.display().to_string()))
}

/// Uninstall Claude Code hooks, restoring other tools and creating a backup.
pub fn uninstall_claude_hooks(project_dir: Option<&Path>) -> Result<Option<String>, String> {
    let scope = match project_dir {
        Some(dir) => Scope::Project(dir.to_path_buf()),
        None => Scope::User,
    };
    let plan = claude_settings::plan_uninstall(&scope).map_err(|e| e.to_string())?;
    let applied = claude_settings::apply(&plan).map_err(|e| e.to_string())?;
    Ok(applied.backup.map(|p| p.display().to_string()))
}

/// Central hub broadcasting observed external agent events to UI listeners and speech synthesis.
#[derive(Clone)]
pub struct ObservationHub {
    sender: broadcast::Sender<ObservedEvent>,
}

impl Default for ObservationHub {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(512);
        Self { sender }
    }
}

impl ObservationHub {
    /// Create a new observation hub.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to the broadcast stream of observed events.
    pub fn subscribe(&self) -> broadcast::Receiver<ObservedEvent> {
        self.sender.subscribe()
    }

    /// Dispatch an event to all subscribers.
    pub fn dispatch(&self, event: ObservedEvent) {
        let _ = self.sender.send(event);
    }

    /// Convert a Claude Code hook event into an [`ObservedEvent`] and broadcast it.
    pub fn handle_claude_hook_event(&self, event: &HookEvent) {
        let session = if event.session_id.is_empty() {
            "default".to_string()
        } else {
            event.session_id.clone()
        };

        let mapped = match event.hook_event_name.as_str() {
            "SessionStart" => Some(ObservedEvent::SessionStarted {
                session_id: session,
                agent: "claude".into(),
            }),
            "PreToolUse" => {
                let tool = event.tool_name.clone().unwrap_or_else(|| "工具".into());
                let summary = event.tool_input.as_ref()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .unwrap_or_default();
                Some(ObservedEvent::ToolStarted {
                    session_id: session,
                    tool_name: tool,
                    summary,
                })
            }
            "PostToolUse" => {
                let tool = event.tool_name.clone().unwrap_or_else(|| "工具".into());
                let preview = event.tool_response.as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
                    .chars()
                    .take(300)
                    .collect();
                Some(ObservedEvent::ToolFinished {
                    session_id: session,
                    tool_name: tool,
                    ok: true,
                    preview,
                })
            }
            "PermissionRequest" => {
                let tool = event.tool_name.clone().unwrap_or_else(|| "操作".into());
                let detail = event.tool_input.as_ref()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .unwrap_or_default();
                let risk = if crate::model::classify_risk(&detail, Path::new(".")) == crate::model::Risk::High {
                    "high"
                } else {
                    "low"
                };
                Some(ObservedEvent::PermissionRequested {
                    session_id: session,
                    request_id: format!("req-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                    tool_name: tool,
                    detail,
                    risk: risk.into(),
                })
            }
            "Stop" | "SessionEnd" => {
                let summary = event.reason.clone()
                    .or_else(|| event.prompt.clone())
                    .unwrap_or_default();
                let short_summary: String = summary.lines().take(3).collect::<Vec<_>>().join(" ");
                Some(ObservedEvent::TaskCompleted {
                    session_id: session,
                    summary: short_summary,
                    success: true,
                })
            }
            "Notification" => {
                let msg = event.message.clone().unwrap_or_default();
                Some(ObservedEvent::Notification {
                    session_id: session,
                    message: msg,
                })
            }
            _ => None,
        };

        if let Some(ev) = mapped {
            self.dispatch(ev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn announcements_produce_natural_chinese_speech_cues() {
        let task_done = ObservedEvent::TaskCompleted {
            session_id: "s1".into(),
            summary: "代码编译通过，测试全部绿灯。".into(),
            success: true,
        };
        let speech = task_done.announcement().unwrap();
        assert!(speech.contains("小伴提示你：任务已完成"));
        assert!(speech.contains("测试全部绿灯"));

        let perm = ObservedEvent::PermissionRequested {
            session_id: "s1".into(),
            request_id: "r1".into(),
            tool_name: "shell".into(),
            detail: "rm -rf build".into(),
            risk: "high".into(),
        };
        let perm_speech = perm.announcement().unwrap();
        assert!(perm_speech.contains("等待你确认"));
        assert!(perm_speech.contains("高风险操作"));
        assert!(perm_speech.contains("shell"));
    }

    #[tokio::test]
    async fn observation_hub_broadcasts_events() {
        let hub = Arc::new(ObservationHub::new());
        let mut rx = hub.subscribe();

        hub.dispatch(ObservedEvent::SessionStarted {
            session_id: "test-session".into(),
            agent: "claude".into(),
        });

        let received: ObservedEvent = rx.recv().await.unwrap();
        match received {
            ObservedEvent::SessionStarted { session_id, agent } => {
                assert_eq!(session_id, "test-session");
                assert_eq!(agent, "claude");
            }
            _ => panic!("unexpected event"),
        }
    }

    #[test]
    fn hooks_status_inspects_scope_without_crashing() {
        let tmp = std::env::temp_dir().join(format!("hooks-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let status = inspect_claude_hooks(Some(&tmp)).unwrap();
        assert!(!status.is_installed);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

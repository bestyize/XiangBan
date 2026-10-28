//! "安全与权限" settings: optional command isolation and networking, the user's hooks file, and the
//! "always allow" answers saved per conversation so they survive a restart.
//!
//! Stored in `chat/agent-settings.json`; hooks keep their own file (`chat/hooks.json`, see `hooks`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// One "always allow" answer: a tool, and for a command tool the kind of command (`npm test`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Grant {
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentSettings {
    pub execution: super::task_runtime::ExecutionLimits,
    /// Optional macOS command isolation. Full trust uses the user's normal permissions by default.
    pub command_sandbox: bool,
    /// Commands in the full-trust sandbox may reach the network (installing dependencies needs it).
    pub sandbox_network: bool,
    /// Saved grants, by conversation.
    pub grants: BTreeMap<String, Vec<Grant>>,
    /// Windows, experimental: full-trust commands run at low integrity. Off until the user turns it on.
    pub windows_sandbox: bool,
    /// Working folders already labelled low integrity for it.
    pub labelled: Vec<String>,
    /// The conversation each scheduled agent task (a care rule) adds its runs to, by rule id.
    pub scheduled_sessions: BTreeMap<String, String>,
    /// Working folders chosen lately, newest first, for the chat page's folder menu.
    pub recent_workspaces: Vec<String>,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self { execution: Default::default(), command_sandbox: false, sandbox_network: true, grants: BTreeMap::new(), windows_sandbox: false, labelled: Vec::new(), scheduled_sessions: BTreeMap::new(), recent_workspaces: Vec::new() }
    }
}

const FILE: &str = "agent-settings.json";
pub const RECENT_WORKSPACES: usize = 8;

/// A missing or unreadable file gives the defaults: settings are conveniences, never needed to run.
pub fn load(chat: &Path) -> AgentSettings {
    fs::read(chat.join(FILE)).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
}

/// Execution limits are user budget decisions, not disposable UI preferences.
pub fn load_execution(chat: &Path) -> Result<super::task_runtime::ExecutionLimits, String> {
    let bytes = match fs::read(chat.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Default::default()),
        Err(error) => return Err(format!("无法读取执行预算：{error}")),
    };
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| format!("执行设置已损坏：{e}"))?;
    if !value.is_object() { return Err("执行设置必须为对象".into()); }
    let limits: super::task_runtime::ExecutionLimits = match value.get("execution") {
        Some(value) => serde_json::from_value(value.clone()).map_err(|e| format!("执行预算无效：{e}"))?,
        None => Default::default(),
    };
    limits.validate()?;
    Ok(limits)
}

pub fn save(chat: &Path, settings: &AgentSettings) -> Result<(), String> {
    fs::create_dir_all(chat).map_err(|e| e.to_string())?;
    let temporary = chat.join(format!("{FILE}.tmp"));
    fs::write(&temporary, serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?).map_err(|e| format!("无法保存设置：{e}"))?;
    fs::rename(&temporary, chat.join(FILE)).map_err(|e| format!("无法保存设置：{e}"))
}

impl AgentSettings {
    /// Returns whether it was new.
    pub fn add_grant(&mut self, session: &str, grant: Grant) -> bool {
        let list = self.grants.entry(session.to_string()).or_default();
        if list.contains(&grant) {
            return false;
        }
        list.push(grant);
        list.sort();
        true
    }

    pub fn remove_grant(&mut self, session: &str, grant: &Grant) -> bool {
        let Some(list) = self.grants.get_mut(session) else { return false };
        let before = list.len();
        list.retain(|g| g != grant);
        let removed = list.len() != before;
        if list.is_empty() {
            self.grants.remove(session);
        }
        removed
    }

    /// Puts a folder first in the recent list, keeping `RECENT_WORKSPACES` of them.
    pub fn remember_workspace(&mut self, path: &str) {
        self.recent_workspaces.retain(|p| p != path);
        self.recent_workspaces.insert(0, path.to_string());
        self.recent_workspaces.truncate(RECENT_WORKSPACES);
    }

    pub fn forget_session(&mut self, session: &str) -> bool {
        self.grants.remove(session).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_settings_do_not_silently_enable_command_isolation() {
        let mut settings: AgentSettings = serde_json::from_str(r#"{"sandboxNetwork":false}"#).unwrap();
        assert!(!settings.command_sandbox);
        assert!(!settings.sandbox_network);
        settings.command_sandbox = true;
        assert!(serde_json::from_str::<AgentSettings>(&serde_json::to_string(&settings).unwrap()).unwrap().command_sandbox);
    }

    #[test]
    fn missing_legacy_limits_use_defaults_but_corrupt_budgets_fail_closed() {
        let root = std::env::temp_dir().join(format!("xiaoban-budget-settings-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(FILE), b"{}").unwrap();
        assert_eq!(load_execution(&root).unwrap(), Default::default());
        fs::write(root.join(FILE), br#"{"execution":{"maxTurns":0}}"#).unwrap();
        assert!(load_execution(&root).is_err());
        fs::write(root.join(FILE), b"invalid JSON").unwrap();
        assert!(load_execution(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn grants_are_kept_once_per_conversation_and_survive_a_reload() {
        let dir = std::env::temp_dir().join(format!("xiaoban-agent-settings-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(load(&dir), AgentSettings::default(), "no file: defaults, network on");
        let mut settings = AgentSettings::default();
        let npm = Grant { tool: "shell".into(), kind: Some("npm test".into()) };
        assert!(settings.add_grant("s1", npm.clone()));
        assert!(!settings.add_grant("s1", npm.clone()), "once");
        settings.add_grant("s1", Grant { tool: "write_file".into(), kind: None });
        settings.sandbox_network = false;
        save(&dir, &settings).unwrap();
        let loaded = load(&dir);
        assert_eq!(loaded, settings);
        let mut loaded = loaded;
        assert!(loaded.remove_grant("s1", &npm));
        assert!(loaded.forget_session("s1"));
        assert!(loaded.grants.is_empty());
        fs::write(dir.join(FILE), "{ broken").unwrap();
        assert_eq!(load(&dir), AgentSettings::default(), "a broken file falls back to the defaults");
        let _ = fs::remove_dir_all(&dir);
    }
}

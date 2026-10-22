//! "安全与权限" settings: whether full-trust commands may use the network, the user's hooks file, and the
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
    /// Commands in the full-trust sandbox may reach the network (installing dependencies needs it).
    pub sandbox_network: bool,
    /// Saved grants, by conversation.
    pub grants: BTreeMap<String, Vec<Grant>>,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self { sandbox_network: true, grants: BTreeMap::new() }
    }
}

const FILE: &str = "agent-settings.json";

/// A missing or unreadable file gives the defaults: settings are conveniences, never needed to run.
pub fn load(chat: &Path) -> AgentSettings {
    fs::read(chat.join(FILE)).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
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

    pub fn forget_session(&mut self, session: &str) -> bool {
        self.grants.remove(session).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

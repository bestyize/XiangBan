//! Environment detection and installation guides for external coding agents (Claude Code, Codex, Desktop apps).

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Status of a local CLI development tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolStatus {
    /// Tool identifier (e.g. "claude", "codex", "node", "git").
    pub name: String,
    /// Whether the tool executable is found on PATH.
    pub installed: bool,
    /// Full path to the executable if found.
    pub path: Option<String>,
    /// Version string reported by `--version`.
    pub version: Option<String>,
    /// Login status if verifiable (e.g. `claude auth status` or `codex login status`).
    pub logged_in: Option<bool>,
    /// Diagnostic notes or error output.
    pub notes: Vec<String>,
}

/// Running status of a GUI desktop app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesktopAppStatus {
    /// App display name (e.g. "Claude", "ChatGPT", "Codex").
    pub name: String,
    /// Whether a running process was detected.
    pub running: bool,
}

/// Comprehensive report of the local environment for speech coding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentReport {
    /// Operating system and architecture.
    pub os: String,
    /// Node.js status.
    pub node: ToolStatus,
    /// Git status.
    pub git: ToolStatus,
    /// Claude Code CLI status.
    pub claude: ToolStatus,
    /// OpenAI Codex CLI status.
    pub codex: ToolStatus,
    /// Running state of known desktop apps.
    pub desktop_apps: Vec<DesktopAppStatus>,
    /// Whether at least one external agent (Claude Code or Codex) is installed and available.
    pub ready_for_speech_coding: bool,
}

/// Installation and configuration guide for a missing or upgradable tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallGuide {
    /// Tool name.
    pub tool: String,
    /// Official installation command.
    pub official_command: String,
    /// China mainland mirror command (npmmirror / taobao).
    pub mirror_command: Option<String>,
    /// Command to log in.
    pub login_command: Option<String>,
    /// Documentation / download URL.
    pub documentation_url: String,
}

fn command_with_clean_env(path: &Path) -> Command {
    let mut cmd = Command::new(path);
    cmd.env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("CLAUDE_CODE_CHILD_SESSION")
        .env_remove("CLAUDE_CODE_REMOTE_SESSION_ID")
        .kill_on_drop(true);
    cmd
}

async fn run_cmd(path: &Path, args: &[&str], timeout_duration: Duration) -> Result<(bool, String), String> {
    let child = command_with_clean_env(path)
        .args(args)
        .output();
    match timeout(timeout_duration, child).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let text = if stdout.is_empty() { stderr } else { stdout };
            Ok((output.status.success(), text))
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("命令执行超时".into()),
    }
}

async fn probe_cli(name: &str, version_args: &[&str], login_args: Option<&[&str]>) -> ToolStatus {
    let path = crate::process::default_path(name);
    if !path.exists() {
        return ToolStatus {
            name: name.to_string(),
            installed: false,
            path: None,
            version: None,
            logged_in: None,
            notes: vec!["未在 PATH 中找到可执行文件".into()],
        };
    }

    let path_str = path.display().to_string();
    let mut notes = Vec::new();
    let version = match run_cmd(&path, version_args, Duration::from_secs(5)).await {
        Ok((true, text)) => {
            let first_line = text.lines().next().unwrap_or("").trim().to_string();
            if first_line.is_empty() { None } else { Some(first_line) }
        }
        Ok((false, text)) => {
            notes.push(format!("版本检测返回异常：{text}"));
            None
        }
        Err(e) => {
            notes.push(format!("版本检测失败：{e}"));
            None
        }
    };

    let logged_in = if let Some(login_args) = login_args {
        match run_cmd(&path, login_args, Duration::from_secs(5)).await {
            Ok((ok, text)) => {
                if !ok && !text.is_empty() {
                    notes.push(format!("未登录或认证过期（{text}）"));
                }
                Some(ok)
            }
            Err(e) => {
                notes.push(format!("登录状态检测失败：{e}"));
                Some(false)
            }
        }
    } else {
        None
    };

    ToolStatus {
        name: name.to_string(),
        installed: true,
        path: Some(path_str),
        version,
        logged_in,
        notes,
    }
}

async fn detect_desktop_apps() -> Vec<DesktopAppStatus> {
    let mut targets = vec![
        DesktopAppStatus { name: "Claude".into(), running: false },
        DesktopAppStatus { name: "ChatGPT".into(), running: false },
        DesktopAppStatus { name: "Codex".into(), running: false },
    ];

    #[cfg(windows)]
    {
        if let Ok(Ok(output)) = timeout(
            Duration::from_secs(4),
            Command::new("tasklist").args(["/fo", "csv", "/nh"]).output(),
        ).await {
            let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
            for app in &mut targets {
                let needle = format!("\"{}.exe\"", app.name.to_lowercase());
                if text.contains(&needle) {
                    app.running = true;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(Ok(output)) = timeout(
            Duration::from_secs(4),
            Command::new("ps").args(["-axo", "comm"]).output(),
        ).await {
            let text = String::from_utf8_lossy(&output.stdout);
            for app in &mut targets {
                let needle = format!("{}.app", app.name);
                if text.contains(&needle) {
                    app.running = true;
                }
            }
        }
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        if let Ok(Ok(output)) = timeout(
            Duration::from_secs(4),
            Command::new("ps").args(["-axo", "comm"]).output(),
        ).await {
            let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
            for app in &mut targets {
                if text.contains(&app.name.to_lowercase()) {
                    app.running = true;
                }
            }
        }
    }

    targets
}

/// Detect local development tools, coding agents, and desktop applications.
pub async fn detect_environment() -> EnvironmentReport {
    let os = format!("{} {} {}", std::env::consts::OS, std::env::consts::FAMILY, std::env::consts::ARCH);
    let (node, git, claude, codex, desktop_apps) = tokio::join!(
        probe_cli("node", &["--version"], None),
        probe_cli("git", &["--version"], None),
        probe_cli("claude", &["--version"], Some(&["auth", "status"])),
        probe_cli("codex", &["--version"], Some(&["login", "status"])),
        detect_desktop_apps(),
    );

    let ready_for_speech_coding = claude.installed || codex.installed;

    EnvironmentReport {
        os,
        node,
        git,
        claude,
        codex,
        desktop_apps,
        ready_for_speech_coding,
    }
}

/// Official installation commands and mirror options for supported agents and prerequisites.
pub fn install_guides() -> Vec<InstallGuide> {
    vec![
        InstallGuide {
            tool: "claude".into(),
            official_command: "npm install -g @anthropic-ai/claude-code".into(),
            mirror_command: Some("npm install -g @anthropic-ai/claude-code --registry=https://registry.npmmirror.com".into()),
            login_command: Some("claude /login".into()),
            documentation_url: "https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview".into(),
        },
        InstallGuide {
            tool: "codex".into(),
            official_command: "npm install -g @openai/codex".into(),
            mirror_command: Some("npm install -g @openai/codex --registry=https://registry.npmmirror.com".into()),
            login_command: Some("codex login".into()),
            documentation_url: "https://github.com/openai/codex".into(),
        },
        InstallGuide {
            tool: "node".into(),
            official_command: "https://nodejs.org/".into(),
            mirror_command: Some("https://registry.npmmirror.com/-/binary/node/".into()),
            login_command: None,
            documentation_url: "https://nodejs.org/zh-cn/download".into(),
        },
        InstallGuide {
            tool: "git".into(),
            official_command: "https://git-scm.com/downloads".into(),
            mirror_command: Some("https://registry.npmmirror.com/-/binary/git-for-windows/".into()),
            login_command: None,
            documentation_url: "https://git-scm.com/".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detect_environment_reports_os_and_structure() {
        let report = detect_environment().await;
        assert!(!report.os.is_empty());
        assert_eq!(report.node.name, "node");
        assert_eq!(report.git.name, "git");
        assert_eq!(report.claude.name, "claude");
        assert_eq!(report.codex.name, "codex");
        assert!(report.desktop_apps.iter().any(|a| a.name == "Claude"));
        assert!(report.desktop_apps.iter().any(|a| a.name == "ChatGPT"));
        assert!(report.desktop_apps.iter().any(|a| a.name == "Codex"));
    }

    #[test]
    fn install_guides_have_mirror_and_login_commands() {
        let guides = install_guides();
        let claude = guides.iter().find(|g| g.tool == "claude").unwrap();
        assert!(claude.official_command.contains("@anthropic-ai/claude-code"));
        assert!(claude.mirror_command.as_ref().unwrap().contains("npmmirror.com"));
        assert_eq!(claude.login_command.as_deref(), Some("claude /login"));

        let codex = guides.iter().find(|g| g.tool == "codex").unwrap();
        assert!(codex.official_command.contains("@openai/codex"));
        assert_eq!(codex.login_command.as_deref(), Some("codex login"));
    }
}

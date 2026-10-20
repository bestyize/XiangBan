//! Open Computer Use service integration. The host owns its switch, permissions and UI.
//! The MCP transport and tool implementation stay in agent-kernel and the external server.
use agent_kernel::{
    mcp::{McpClient, McpConfig},
    HarnessError, Risk,
};
use std::{
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};

pub const SKILL_NAME: &str = "open-computer-use";
pub const SKILL: &str = include_str!("../skills/open-computer-use/SKILL.md");
/// Kept identical to existing installations so hosts can update their bundled copy
/// without replacing a user-authored skill with the same name.
pub const SKILL_ORIGIN_MARKER: &str = "iFurySt/open-codex-computer-use";

fn config_for(installed: bool) -> McpConfig {
    #[cfg(windows)]
    let npx = "npx.cmd";
    #[cfg(not(windows))]
    let npx = "npx";
    let (command, args): (&str, &[&str]) = if installed {
        ("open-computer-use", &["mcp"])
    } else {
        (npx, &["-y", "open-computer-use", "mcp"])
    };
    McpConfig::new(command, args)
        .default_risk(Risk::Exec)
        .timeout(Duration::from_secs(60))
}

/// Prefer an installed executable; preserve the existing npx fallback.
/// This probes a process; call from a blocking worker rather than a UI thread.
pub fn config() -> McpConfig {
    let installed = Command::new("open-computer-use")
        .arg("-v")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    config_for(installed)
}

/// Start the stdio server and discover its tools through the kernel's MCP client.
/// The caller decides whether it is enabled and registers the returned tools.
pub fn spawn() -> Result<Arc<McpClient>, HarnessError> {
    McpClient::spawn(config())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_service_keeps_its_command_and_execution_permissions() {
        let config = config_for(true);
        assert_eq!(config.command, "open-computer-use");
        assert_eq!(config.args, ["mcp"]);
        assert_eq!(config.default_risk, Risk::Exec);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.env.is_empty());
        assert!(config.inherited_env.is_empty());
    }

    #[test]
    fn fallback_uses_the_platform_npx_launcher() {
        let config = config_for(false);
        assert_eq!(
            config.command,
            if cfg!(windows) { "npx.cmd" } else { "npx" }
        );
        assert_eq!(config.args, ["-y", "open-computer-use", "mcp"]);
        assert_eq!(config.default_risk, Risk::Exec);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn packaged_skill_preserves_discovery_and_existing_installation_identity() {
        assert!(SKILL.contains(&format!("name: {SKILL_NAME}")));
        assert!(SKILL.contains(SKILL_ORIGIN_MARKER));
        for rule in [
            "element_index",
            "text_limit",
            "max_tree_nodes",
            "set_value",
            "query",
            "perform_secondary_action",
            "Drag delivered via app_post",
        ] {
            assert!(SKILL.contains(rule), "missing instruction: {rule}");
        }
    }
}

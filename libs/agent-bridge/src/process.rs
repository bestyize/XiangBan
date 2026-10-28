use crate::{Availability, BridgeError};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    process::{Child, Command},
    time::timeout,
};

pub(crate) fn command(path: &Path) -> Command {
    let mut cmd = Command::new(path);
    cmd.env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("CLAUDE_CODE_CHILD_SESSION")
        .env_remove("CLAUDE_CODE_REMOTE_SESSION_ID")
        .kill_on_drop(true);
    cmd
}
pub(crate) async fn probe(path: &Path) -> Availability {
    let result = timeout(
        Duration::from_secs(5),
        command(path).arg("--version").output(),
    )
    .await;
    match result {
        Ok(Ok(output)) if output.status.success() => Availability {
            installed: true,
            path: Some(path.to_path_buf()),
            version: Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()),
            notes: vec![],
        },
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => Availability {
            installed: false,
            path: None,
            version: None,
            notes: vec![],
        },
        _ => Availability {
            installed: false,
            path: None,
            version: None,
            notes: vec!["Version probe failed or timed out".into()],
        },
    }
}
pub(crate) async fn shutdown(child: &mut Child) -> Result<(), BridgeError> {
    drop(child.stdin.take());
    if timeout(Duration::from_secs(2), child.wait()).await.is_err() {
        child.kill().await.map_err(|_| BridgeError::ProcessExited)?;
        child.wait().await.map_err(|_| BridgeError::ProcessExited)?;
    }
    Ok(())
}
pub(crate) fn default_path(name: &str) -> PathBuf {
    #[cfg(windows)]
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            for suffix in [".exe", ".cmd", ".bat"] {
                let candidate = dir.join(format!("{name}{suffix}"));
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn env_child() {
        if std::env::var_os("XIAOBAN_TEST_ENV_CHILD").is_none() {
            return;
        }
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            for name in ["codex", "claude"] {
                let result = super::command(&crate::test_support::fake(name))
                    .arg("--version")
                    .output()
                    .await
                    .unwrap();
                assert!(
                    result.status.success(),
                    "{name} inherited a session variable"
                );
            }
        });
    }
    #[test]
    fn clears_inherited_session_variables() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "process::tests::env_child"])
            .env("XIAOBAN_TEST_ENV_CHILD", "1")
            .env("CLAUDE_CODE_SESSION_ID", "sentinel")
            .env("CLAUDE_CODE_CHILD_SESSION", "sentinel")
            .env("CLAUDE_CODE_REMOTE_SESSION_ID", "sentinel")
            .output()
            .unwrap();
        assert!(result.status.success());
    }
}

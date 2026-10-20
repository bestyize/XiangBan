//! Open Computer Use service integration. The host owns its switch, permissions and UI.
//! The MCP transport and tool implementation stay in agent-kernel and the external server.
use agent_kernel::{
    mcp::{McpClient, McpConfig},
    HarnessError, Risk,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

pub const SKILL_NAME: &str = "open-computer-use";
pub const SKILL: &str = include_str!("../skills/open-computer-use/SKILL.md");
pub const SKILL_ORIGIN_MARKER: &str = "iFurySt/open-codex-computer-use";
pub const RUNTIME_VERSION: &str = "1.1.2";

fn runtime_target() -> String {
    let platform = match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        other => other,
    };
    format!("{platform}-{arch}")
}

fn executable_relative() -> &'static str {
    if cfg!(target_os = "macos") {
        "Open Computer Use.app/Contents/MacOS/OpenComputerUse"
    } else if cfg!(windows) {
        "open-computer-use.exe"
    } else {
        "open-computer-use"
    }
}

fn config_error(message: impl Into<String>) -> HarnessError {
    HarnessError::Config(message.into())
}

/// Resolve only the host's bundled runtime. No PATH lookup, npm or network fallback.
pub fn executable(root: &Path) -> Result<PathBuf, HarnessError> {
    let manifest = root.join("runtime.json");
    let contents = fs::read_to_string(&manifest).map_err(|e| config_error(format!("内置电脑控制引擎缺失（{}）：{e}。请重新安装完整的小伴安装包；开发环境先运行 npm run computer-use:prepare。", manifest.display())))?;
    let metadata: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| config_error(format!("内置电脑控制引擎清单损坏：{e}")))?;
    if metadata["version"].as_str() != Some(RUNTIME_VERSION)
        || metadata["target"].as_str() != Some(runtime_target().as_str())
    {
        return Err(config_error(format!("内置电脑控制引擎版本或平台不匹配，需要 {RUNTIME_VERSION} / {}，请重新安装对应平台的小伴安装包。", runtime_target())));
    }
    let path = root
        .join(executable_relative())
        .canonicalize()
        .map_err(|e| config_error(format!("内置电脑控制程序缺失（{}）：{e}", root.display())))?;
    if !path.is_file() {
        return Err(config_error("内置电脑控制程序不是普通文件"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(&path)
            .map_err(|e| config_error(e.to_string()))?
            .permissions()
            .mode()
            & 0o111
            == 0
        {
            return Err(config_error(
                "内置电脑控制程序没有执行权限，请重新安装完整的小伴安装包。",
            ));
        }
    }
    Ok(path)
}

pub fn config(root: &Path) -> Result<McpConfig, HarnessError> {
    let executable = executable(root)?;
    let command = executable
        .to_str()
        .ok_or_else(|| config_error("内置电脑控制程序路径不是有效 UTF-8"))?;
    let config = McpConfig::new(command, &["mcp"])
        .default_risk(Risk::Exec)
        .timeout(Duration::from_secs(60));
    #[cfg(target_os = "linux")]
    let config = config
        .inherit_env("DISPLAY")
        .inherit_env("WAYLAND_DISPLAY")
        .inherit_env("XDG_RUNTIME_DIR")
        .inherit_env("DBUS_SESSION_BUS_ADDRESS")
        .inherit_env("XAUTHORITY");
    Ok(config)
}

/// The host supplies its installed resource directory. The native engine speaks MCP directly.
pub fn spawn(root: &Path) -> Result<Arc<McpClient>, HarnessError> {
    #[cfg(target_os = "macos")]
    {
        let version = std::process::Command::new("/usr/bin/sw_vers")
            .arg("-productVersion")
            .output()
            .map_err(|e| config_error(format!("无法检测 macOS 版本：{e}")))?;
        let version = String::from_utf8_lossy(&version.stdout);
        if version
            .trim()
            .split('.')
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .is_none_or(|v| v < 14)
        {
            return Err(config_error("内置电脑控制引擎需要 macOS 14 或更新版本。"));
        }
    }
    let config = config(root)?;
    let command = config.command.clone();
    let client = McpClient::spawn(config).map_err(|e| {
        config_error(format!(
            "内置电脑控制引擎启动失败（{command}）：{e}。macOS 请检查辅助功能和屏幕录制权限。"
        ))
    })?;
    if client.tools().is_empty() {
        return Err(config_error(
            "内置电脑控制引擎未提供任何工具，请重新安装完整的小伴安装包。",
        ));
    }
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Runtime(PathBuf);
    impl Runtime {
        fn new(target: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "ocu-runtime-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let file = root.join(executable_relative());
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, b"fixture").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
            }
            fs::write(
                root.join("runtime.json"),
                serde_json::json!({"version": RUNTIME_VERSION, "target": target}).to_string(),
            )
            .unwrap();
            Self(root)
        }
    }
    impl Drop for Runtime {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn starts_only_the_bundled_native_executable() {
        let runtime = Runtime::new(&runtime_target());
        let config = config(&runtime.0).unwrap();
        assert_eq!(
            Path::new(&config.command),
            runtime
                .0
                .join(executable_relative())
                .canonicalize()
                .unwrap()
        );
        assert_eq!(config.args, ["mcp"]);
        assert_eq!(config.default_risk, Risk::Exec);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn missing_or_wrong_platform_runtime_fails_without_external_fallback() {
        let runtime = Runtime::new("wrong-platform");
        assert!(config(&runtime.0)
            .unwrap_err()
            .to_string()
            .contains("平台不匹配"));
        fs::remove_file(runtime.0.join("runtime.json")).unwrap();
        assert!(config(&runtime.0)
            .unwrap_err()
            .to_string()
            .contains("引擎缺失"));
    }

    #[test]
    fn missing_executable_is_reported() {
        let runtime = Runtime::new(&runtime_target());
        fs::remove_file(runtime.0.join(executable_relative())).unwrap();
        assert!(config(&runtime.0)
            .unwrap_err()
            .to_string()
            .contains("程序缺失"));
    }

    #[test]
    fn packaged_skill_and_version_match_the_pinned_distribution() {
        let lock: serde_json::Value =
            serde_json::from_str(include_str!("../runtime-lock.json")).unwrap();
        assert_eq!(lock["version"], RUNTIME_VERSION);
        assert!(SKILL.contains(&format!("name: {SKILL_NAME}")));
        assert!(SKILL.contains(SKILL_ORIGIN_MARKER));
    }

    #[test]
    #[ignore = "requires the prepared native runtime; only discovers tools, does not control the desktop"]
    fn real_bundled_engine_discovers_tools() {
        let root = std::env::var_os("XIAOBAN_COMPUTER_USE_RUNTIME")
            .expect("set XIAOBAN_COMPUTER_USE_RUNTIME to the prepared runtime directory");
        let client = spawn(Path::new(&root)).unwrap();
        let names: Vec<_> = client
            .tools()
            .iter()
            .map(|tool| tool.spec().name.clone())
            .collect();
        assert!(names.iter().any(|name| name == "list_apps"), "{names:?}");
        assert!(
            names.iter().any(|name| name == "get_app_state"),
            "{names:?}"
        );
        assert!(names.iter().any(|name| name == "click"), "{names:?}");
        println!("Real bundled MCP tools: {names:?}");
    }
}

//! 语音编程（P1 环境层与观察层，P2 界面注入层）：外部 Agent 环境检测、安装指南、hooks 管理与桌面注入。

use agent_bridge::environment::{self, EnvironmentReport, InstallGuide};
use agent_bridge::injector::{self, InjectionRequest, InjectionResult};
use agent_bridge::observer::{self, HooksStatus};
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::Manager;

fn control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control"
        && window.label() != "configuration-settings"
        && window.label() != crate::float_assistant::LABEL
    {
        return Err("当前窗口没有调用外部 Agent 注入的权限".into());
    }
    Ok(())
}

fn resolve_scope_path(workspace: Option<String>) -> Option<PathBuf> {
    workspace
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// Detect Node.js, git, Claude Code CLI, Codex CLI, and desktop apps.
#[tauri::command]
pub async fn speech_agent_env(window: tauri::WebviewWindow) -> Result<EnvironmentReport, String> {
    control(&window)?;
    Ok(environment::detect_environment().await)
}

/// Official installation commands, mirror commands and documentation links.
#[tauri::command]
pub fn speech_agent_install_guides(window: tauri::WebviewWindow) -> Result<Vec<InstallGuide>, String> {
    control(&window)?;
    Ok(environment::install_guides())
}

/// Inspect Claude Code hooks status in a workspace folder or global user settings.
#[tauri::command]
pub fn speech_agent_hooks_status(
    window: tauri::WebviewWindow,
    workspace: Option<String>,
) -> Result<HooksStatus, String> {
    control(&window)?;
    let path = resolve_scope_path(workspace);
    observer::inspect_claude_hooks(path.as_deref())
}

/// Install Claude Code command hooks into a workspace or global settings, creating an automatic backup first.
#[tauri::command]
pub fn speech_agent_hooks_install(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    workspace: Option<String>,
) -> Result<Value, String> {
    control(&window)?;
    let scope_path = resolve_scope_path(workspace);

    // 确定 hook 转发程序的路径：开发环境或打包资源目录
    let exe_suffix = std::env::consts::EXE_SUFFIX;
    let hook_name = format!("xiaoban-hook{exe_suffix}");
    let candidate = app.path().resource_dir()
        .map(|d| d.join(&hook_name))
        .unwrap_or_else(|_| PathBuf::from(&hook_name));
    let hook_bin = if candidate.is_file() {
        candidate
    } else if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let beside = parent.join(&hook_name);
            if beside.is_file() {
                beside
            } else {
                parent.join(format!("../../../libs/agent-hooks/target/debug/{hook_name}"))
            }
        } else {
            PathBuf::from(&hook_name)
        }
    } else {
        PathBuf::from(&hook_name)
    };

    let backup = observer::install_claude_hooks(scope_path.as_deref(), &hook_bin)?;
    let status = observer::inspect_claude_hooks(scope_path.as_deref())?;
    Ok(json!({
        "status": status,
        "backup": backup,
        "hookExecutable": hook_bin.display().to_string(),
    }))
}

/// Uninstall Claude Code command hooks, restoring clean configuration with a backup.
#[tauri::command]
pub fn speech_agent_hooks_uninstall(
    window: tauri::WebviewWindow,
    workspace: Option<String>,
) -> Result<Value, String> {
    control(&window)?;
    let scope_path = resolve_scope_path(workspace);
    let backup = observer::uninstall_claude_hooks(scope_path.as_deref())?;
    let status = observer::inspect_claude_hooks(scope_path.as_deref())?;
    Ok(json!({
        "status": status,
        "backup": backup,
    }))
}

/// P2 界面注入层：先上屏后发送流程（窗口激活、剪贴板暂存与恢复、热键粘贴）。
#[tauri::command]
pub async fn speech_agent_inject(
    window: tauri::WebviewWindow,
    request: InjectionRequest,
) -> Result<InjectionResult, String> {
    control(&window)?;
    // 文字可以为空（只按回车发送刚送进去的内容）；其余校验（应用名称、长度）在注入库里统一做。
    if request.text.trim().is_empty() && !request.text.is_empty() {
        return Err("注入内容不能只有空白".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        injector::inject_text(request)
    })
    .await
    .map_err(|e| e.to_string())
}

/// P3 接续层：发起外部 Agent 委派任务。
#[tauri::command]
pub async fn speech_agent_delegate(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    agent: String,
    prompt: String,
    workspace: Option<String>,
) -> Result<agent_bridge::delegation::DelegatedTaskSummary, String> {
    control(&window)?;
    let target_ws = match resolve_scope_path(workspace) {
        Some(p) => p,
        None => crate::ai_config::default_workspace(&app)?,
    };
    let mgr = app.state::<crate::chat::ChatState>().delegation.clone();
    mgr.delegate(&agent, &prompt, &target_ws).await
}

/// P3 接续层：查询委派任务状态与最近输出。
#[tauri::command]
pub fn speech_agent_task_status(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    task_id: String,
) -> Result<agent_bridge::delegation::DelegatedTaskSummary, String> {
    control(&window)?;
    let mgr = app.state::<crate::chat::ChatState>().delegation.clone();
    mgr.status(&task_id).ok_or_else(|| format!("未找到任务 {task_id}"))
}

/// P3 接续层：取消/打断委派任务。
#[tauri::command]
pub fn speech_agent_task_cancel(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    task_id: String,
) -> Result<(), String> {
    control(&window)?;
    let mgr = app.state::<crate::chat::ChatState>().delegation.clone();
    mgr.cancel(&task_id)
}

/// P3 接续层：列出所有委派任务。
#[tauri::command]
pub fn speech_agent_task_list(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<Vec<agent_bridge::delegation::DelegatedTaskSummary>, String> {
    control(&window)?;
    let mgr = app.state::<crate::chat::ChatState>().delegation.clone();
    Ok(mgr.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_guides_contain_claude_and_codex() {
        let guides = environment::install_guides();
        assert!(guides.iter().any(|g| g.tool == "claude"));
        assert!(guides.iter().any(|g| g.tool == "codex"));
    }

    #[test]
    fn resolve_scope_path_trims_and_handles_none() {
        assert_eq!(resolve_scope_path(None), None);
        assert_eq!(resolve_scope_path(Some("   ".into())), None);
        assert_eq!(resolve_scope_path(Some("C:\\my-project".into())), Some(PathBuf::from("C:\\my-project")));
    }

    #[test]
    fn injection_request_validates_empty_text() {
        let req = InjectionRequest {
            target: "Claude".into(),
            text: "测试文字".into(),
            press_enter: false,
            restore_clipboard_ms: Some(50),
        };
        assert_eq!(req.target, "Claude");
        assert_eq!(req.text, "测试文字");
    }
}

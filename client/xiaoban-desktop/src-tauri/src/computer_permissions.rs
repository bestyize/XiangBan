//! Permission ownership belongs to the actual native engine, not to a shell/AppleScript probe.
use serde_json::{json, Value};
use std::path::PathBuf;
#[cfg(not(debug_assertions))]
use tauri::Manager;

pub fn runtime_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    { let _ = app; Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/computer-use")) }
    #[cfg(not(debug_assertions))]
    { app.path().resource_dir().map(|root| root.join("agent-tools/computer-use")).map_err(|e| format!("无法定位内置电脑控制资源：{e}")) }
}

pub fn diagnose(app: &tauri::AppHandle) -> Result<Value, String> {
    let enabled = crate::ai_config::current(app)?.open_computer_use;
    let root = runtime_root(app)?;
    let engine = agent_computer_use_tools::permissions::diagnose(&root);
    // Missing/damaged runtime is a useful diagnosis too; do not hide it behind a tool exception.
    Ok(json!({
        "enabled": enabled,
        "engine": match engine { Ok(state) => state, Err(error) => json!({"error": error, "available": false}) },
        "host": {"bundleId": crate::assistant_capture::SCREEN_CAPTURE_BUNDLE_ID,
            "screenCapturePreflight": host_capture_access(),
            "guidance": "小伴的屏幕录制用于自身截图功能；电脑控制权限以引擎检测为准。主程序不再通过 System Events 额外申请无障碍。系统可能按责任进程归属显示小伴或引擎，应以实际授权条目为准。刚授权后进程内检查可能仍为 false，必要时退出并重新打开小伴。"},
        "nextStep": if enabled { "参数校验失败先修正原调用。检查 engine.permissions；缺权限先引导，authorizationMismatch.services 确认旧签名冲突时，用 computer_use_repair_permissions 经用户确认定向重置，重新授权后复查。" } else { "电脑控制开关未开启，请用户在输入框的「+ → 工具 → 操作电脑」中启用后重新发起任务。" }
    }))
}

#[cfg(target_os = "macos")]
fn host_capture_access() -> Option<bool> { Some(crate::assistant_capture::macos_permission_status()) }
#[cfg(not(target_os = "macos"))]
fn host_capture_access() -> Option<bool> { None }

pub fn request(app: &tauri::AppHandle) -> Result<Value, String> {
    if !crate::ai_config::current(app)?.open_computer_use {
        return Err("请先在输入框的「+ → 工具 → 操作电脑」中启用电脑控制，再申请引擎权限。".into());
    }
    agent_computer_use_tools::permissions::request_permissions(&runtime_root(app)?)
}

pub fn repair(app: &tauri::AppHandle, permission: &str) -> Result<Value, String> {
    if !crate::ai_config::current(app)?.open_computer_use {
        return Err("请先开启电脑屏幕与应用控制，再修复引擎权限。".into());
    }
    agent_computer_use_tools::permissions::repair_permissions(&runtime_root(app)?, permission)
}

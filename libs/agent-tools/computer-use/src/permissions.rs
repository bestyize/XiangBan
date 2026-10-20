//! Native-engine permission checks. MCP handshakes alone do not establish desktop access.
use crate::{executable, RUNTIME_VERSION};
use serde_json::{json, Value};
use std::path::Path;

pub const ENGINE_BUNDLE_ID: &str = "com.ifuryst.opencomputeruse";

fn permission_service(permission: &str) -> Result<(&'static str, &'static str), String> {
    match permission {
        "accessibility" => Ok(("Accessibility", "Privacy_Accessibility")),
        "screenRecording" => Ok(("ScreenCapture", "Privacy_ScreenCapture")),
        _ => Err("只允许修复内置引擎的 accessibility 或 screenRecording 权限".into()),
    }
}

#[cfg(any(target_os = "macos", test))]
fn mismatched_services(log: &str) -> Vec<&'static str> {
    let mut services = Vec::new();
    let subject = format!("subject {ENGINE_BUNDLE_ID} and service ");
    for line in log.lines().filter(|line| line.contains("Failed to match existing code requirement") && line.contains(&subject)) {
        for (permission, service) in [("accessibility", "kTCCServiceAccessibility"), ("screenRecording", "kTCCServiceScreenCapture")] {
            if line.contains(service) && !services.contains(&permission) { services.push(permission); }
        }
    }
    services
}

#[cfg(target_os = "macos")]
fn authorization_mismatch() -> Value {
    // Read only this engine's explicit code-requirement failures, never the TCC database.
    const PREDICATE: &str = "subsystem == 'com.apple.TCC' AND eventMessage CONTAINS 'Failed to match existing code requirement' AND eventMessage CONTAINS 'com.ifuryst.opencomputeruse'";
    match output(Path::new("/usr/bin/log"), &["show", "--last", "1m", "--style", "compact", "--predicate", PREDICATE]) {
        Ok(result) if result.status.success() => json!({"checked": true, "services": mismatched_services(&String::from_utf8_lossy(&result.stdout)), "source": "recentTccLogs", "note": "有匹配记录可确认旧签名授权冲突；没有记录不能证明授权有效，以实际权限状态为准。"}),
        _ => json!({"checked": false, "services": [], "note": "系统日志不可用，无法确认是否旧签名冲突；缺权限不等于已经确认签名冲突。"}),
    }
}

#[cfg(target_os = "macos")]
fn output(program: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    use std::{io::Read, process::{Command, Stdio}, time::{Duration, Instant}};
    let mut child = Command::new(program).args(args).stdin(Stdio::null())
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
        .map_err(|e| format!("无法运行电脑控制诊断：{e}"))?;
    let stdout = child.stdout.take().ok_or("无法读取诊断输出")?;
    let stderr = child.stderr.take().ok_or("无法读取诊断错误")?;
    let read = |stream: Box<dyn Read + Send>| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = stream.take(16_385).read_to_end(&mut bytes).map(|_| bytes);
            let _ = tx.send(result);
        });
        rx
    };
    let out = read(Box::new(stdout));
    let err = read(Box::new(stderr));
    let deadline = Instant::now() + Duration::from_secs(8);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            result => {
                let _ = child.kill(); let _ = child.wait();
                break Err(match result { Err(e) => format!("诊断进程出错：{e}"), _ => "电脑控制诊断超时".into() });
            }
        }
    };
    let status = status?;
    // Onboarding may spawn another process. Do not wait forever for an inherited pipe to close.
    let stdout = out.recv_timeout(deadline.saturating_duration_since(Instant::now())).map_err(|_| "诊断输出超时")?.map_err(|e| e.to_string())?;
    let stderr = err.recv_timeout(deadline.saturating_duration_since(Instant::now())).map_err(|_| "诊断错误输出超时")?.map_err(|e| e.to_string())?;
    if stdout.len() > 16_384 || stderr.len() > 16_384 { return Err("电脑控制诊断输出超过限制".into()); }
    Ok(std::process::Output { status, stdout, stderr })
}

#[cfg(any(target_os = "macos", test))]
fn permission_snapshot(bytes: &[u8]) -> Result<Value, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| format!("引擎权限诊断不是有效 JSON：{e}"))?;
    let accessibility = value["accessibilityTrusted"].as_bool().ok_or("引擎没有返回无障碍权限状态")?;
    let screen = value["screenCaptureGranted"].as_bool().ok_or("引擎没有返回屏幕录制权限状态")?;
    // Do not trust exit code or an inconsistent allGranted field.
    Ok(json!({
        "accessibilityTrusted": accessibility,
        "screenCaptureGranted": screen,
        "allGranted": accessibility && screen,
        "missingPermissions": ([(!accessibility).then_some("accessibility"), (!screen).then_some("screenRecording")].into_iter().flatten().collect::<Vec<_>>())
    }))
}

/// No dialogs, screenshots, target-app reads, or TCC changes. Always probe the actual bundled engine.
pub fn diagnose(root: &Path) -> Result<Value, String> {
    let exe = executable(root).map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    {
        let result = output(&exe, &["doctor", "--json"])?;
        if !result.status.success() { return Err("内置引擎权限检查失败，请检查系统版本和安装完整性".into()); }
        let permissions = permission_snapshot(&result.stdout)?;
        let signature = output(Path::new("/usr/bin/codesign"), &["--display", "--requirements", "-", exe.to_str().ok_or("引擎路径无效")?])?;
        let requirement = String::from_utf8_lossy(&signature.stdout).trim().to_string();
        let stable = signature.status.success() && requirement.contains("designated =>") && !requirement.contains("cdhash");
        let mismatch = if permissions["allGranted"] == true { json!({"checked": false, "services": []}) } else { authorization_mismatch() };
        Ok(json!({
            "platform": "macos", "runtimeVersion": RUNTIME_VERSION,
            "executable": exe, "bundleId": ENGINE_BUNDLE_ID,
            "permissions": permissions,
            "stableSigningIdentity": stable, "signingRequirement": requirement,
            "authorizationMismatch": mismatch,
            "guidance": "先区分参数校验失败、引擎启动失败和系统权限拒绝。参数校验失败表示没执行工具，应修正原调用，不要改走 Python/AppKit/AppleScript 或重置权限。缺权限时调用 request_accessibility_permission；若 authorizationMismatch.services 确认旧签名冲突，说明影响后调用 computer_use_repair_permissions，经用户确认仅重置对应引擎权限并重新引导。系统最终授权仍需用户点击，再 computer_use_diagnose 复查。禁止直接修改 TCC 数据库、全局 reset All 或用脚本绕过授权。"
        }))
    }
    #[cfg(not(target_os = "macos"))]
    { Ok(json!({"platform": std::env::consts::OS, "runtimeVersion": RUNTIME_VERSION, "executable": exe, "guidance": "当前平台无需 macOS 授权。必须在用户已登录的图形桌面中运行，结合 list_apps/get_app_state 的实际结果确认可用。"})) }
}

/// Preflight the same executable before desktop actions, without opening a permission dialog.
pub fn check_access(root: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let exe = executable(root).map_err(|e| e.to_string())?;
        let result = output(&exe, &["doctor", "--json"])?;
        if !result.status.success() { return Err("内置引擎权限检查失败；请调用 computer_use_diagnose，不能把 MCP 连接成功当成可操作。".into()); }
        let state = permission_snapshot(&result.stdout)?;
        if state["allGranted"] != true {
            return Err(format!("computer_use_permission_required：当前内置引擎未获得系统权限 {}。先调用 computer_use_diagnose 定位，再用 request_accessibility_permission 打开引导，等待用户授权后复查。旧签名授权可能失效；不要静默改用 shell/AppleScript 或宣称目标网页无法操作。", state["missingPermissions"]));
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = root;
    Ok(())
}

/// Explicit user-facing onboarding. It cannot grant permissions or reset old authorizations.
pub fn request_permissions(root: &Path) -> Result<Value, String> {
    #[cfg(target_os = "macos")]
    {
        let exe = executable(root).map_err(|e| e.to_string())?;
        let result = output(&exe, &["doctor"])?;
        if !result.status.success() { return Err("无法启动内置引擎授权引导，请在系统设置中检查辅助功能与屏幕录制权限".into()); }
        let revealed = output(Path::new("/usr/bin/open"), &["-R", root.join("Open Computer Use.app").to_str().ok_or("引擎路径无效")?]).is_ok_and(|result| result.status.success());
        Ok(json!({"requested": true, "engineRevealed": revealed, "permissionCheckRequired": true, "message": "已启动内置引擎权限引导，并尝试在 Finder 选中当前 App。请用户在系统界面授权后再次 computer_use_diagnose。若诊断确认旧签名冲突，可调用 computer_use_repair_permissions，经应用内确认定向清除旧记录，再重新授权；不能把启动引导或重置记录当成已经授权。"}))
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = root; Err("当前平台不使用 macOS 权限引导，请根据电脑控制工具的实际错误处理桌面会话。".into()) }
}

/// The host MUST obtain explicit per-call approval, including in its full-trust mode.
/// Only the fixed engine identity and the selected missing permission can be reset.
pub fn repair_permissions(root: &Path, permission: &str) -> Result<Value, String> {
    let (service, pane) = permission_service(permission)?;
    #[cfg(target_os = "macos")]
    {
        let exe = executable(root).map_err(|e| e.to_string())?;
        let probe = output(&exe, &["doctor", "--json"])?;
        if !probe.status.success() { return Err("无法确认引擎当前权限，未重置任何记录".into()); }
        let before = permission_snapshot(&probe.stdout)?;
        let granted = match permission { "accessibility" => before["accessibilityTrusted"] == true, _ => before["screenCaptureGranted"] == true };
        if granted { return Ok(json!({"reset": false, "permission": permission, "message": "所选权限当前已有效，保留授权记录。请修正工具参数或检查目标应用状态。"})); }
        let result = output(Path::new("/usr/bin/tccutil"), &["reset", service, ENGINE_BUNDLE_ID])?;
        if !result.status.success() {
            return Err(format!("无法定向重置引擎 {service} 权限：{}。请在系统设置中移除该引擎旧条目后重新添加；不要自动使用 sudo 或全局重置。", String::from_utf8_lossy(&result.stderr).chars().take(400).collect::<String>()));
        }
        // Report a successful reset even if opening onboarding later fails; do not invite blind replay.
        let settings_opened = output(Path::new("/usr/bin/open"), &[&format!("x-apple.systempreferences:com.apple.preference.security?{pane}")]).is_ok_and(|result| result.status.success());
        let onboarding = match request_permissions(root) { Ok(value) => value, Err(error) => json!({"error": error}) };
        Ok(json!({"reset": true, "permission": permission, "bundleId": ENGINE_BUNDLE_ID,
            "settingsOpened": settings_opened, "onboarding": onboarding, "permissionCheckRequired": true,
            "message": "已只清除内置引擎所选权限的旧记录。请用户在已打开的系统页面为当前 App 重新授权，然后 computer_use_diagnose 复查。尚未授予权限，不要再次重置；若当前引擎进程仍缓存旧状态，可在下一条消息重建工具连接后重试。"}))
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = (root, service, pane); Err("此定向权限修复仅适用于 macOS".into()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_permissions_are_not_success_even_if_the_engine_exits_zero() {
        let state = permission_snapshot(br#"{"allGranted":true,"accessibilityTrusted":false,"screenCaptureGranted":true}"#).unwrap();
        assert_eq!(state["allGranted"], false);
        assert_eq!(state["missingPermissions"], json!(["accessibility"]));
        assert!(permission_snapshot(br#"{"allGranted":true}"#).is_err());
        assert!(permission_snapshot(br#"{"accessibilityTrusted":"true","screenCaptureGranted":true}"#).is_err());
        assert_eq!(permission_snapshot(br#"{"accessibilityTrusted":true,"screenCaptureGranted":true}"#).unwrap()["allGranted"], true);
    }

    #[test]
    fn mismatch_evidence_is_scoped_to_this_engine_and_service() {
        let log = "Failed to match existing code requirement for subject other.app and service kTCCServiceAccessibility\nFailed to match existing code requirement for subject com.ifuryst.opencomputeruse and service kTCCServiceAccessibility\nFailed to match existing code requirement for subject com.ifuryst.opencomputeruse and service kTCCServiceAccessibility";
        assert_eq!(mismatched_services(log), vec!["accessibility"]);
        assert!(mismatched_services("accessibilityTrusted=false").is_empty());
        assert!(permission_service("All").is_err());
        assert!(permission_service("Accessibility; reset All").is_err());
        assert_eq!(permission_service("screenRecording").unwrap(), ("ScreenCapture", "Privacy_ScreenCapture"));
    }
}

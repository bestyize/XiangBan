//! Native-engine permission checks. MCP handshakes alone do not establish desktop access.
use crate::{executable, RUNTIME_VERSION};
use serde_json::{json, Value};
use std::path::Path;

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
        Ok(json!({
            "platform": "macos", "runtimeVersion": RUNTIME_VERSION,
            "executable": exe, "bundleId": "com.ifuryst.opencomputeruse",
            "permissions": permissions,
            "stableSigningIdentity": stable, "signingRequirement": requirement,
            "guidance": "MCP 连接成功不代表已获系统授权。缺权限时调用 request_accessibility_permission 启动内置引擎授权引导，用户授权后重新 computer_use_diagnose。开关已开仍失败时，可能是旧签名授权记录或进程需重启，不能直接认定是网站不支持。由用户在系统设置移除失效条目并为当前应用重新授权；不要自动重置 TCC、修改权限数据库或改用 AppleScript 绕过。无障碍由电脑控制引擎使用；小伴自己的截图另需屏幕录制权限，系统实际归属以授权页面为准。"
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
        Ok(json!({"requested": true, "permissionCheckRequired": true, "message": "已运行内置 Open Computer Use 引擎的权限引导；这不代表已获授权。请用户按系统显示的应用条目授权辅助功能和屏幕录制，再调用 computer_use_diagnose 检查。若旧条目已开启但仍未授权，移除旧条目、重新添加当前应用并重启小伴后检查；不要反复点击开关或自动重置权限。"}))
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = root; Err("当前平台不使用 macOS 权限引导，请根据电脑控制工具的实际错误处理桌面会话。".into()) }
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
}

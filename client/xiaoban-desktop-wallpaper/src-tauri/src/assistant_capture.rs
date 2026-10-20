//! User-initiated region capture. Fixed OS commands only; no model-facing command or arbitrary paths.
//! The assistant window is hidden by the caller. Temporary macOS files are removed on every exit.
use std::{io::Read, process::{Command, Output, Stdio}, time::{Duration, Instant}};

const MAX_BYTES: usize = 3 * 1024 * 1024;
const MAX_SNAPSHOT_OUTPUT: usize = 48 * 1024 * 1024;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Frame { pub x: f64, pub y: f64, pub width: f64, pub height: f64, pub image: String }

fn run(mut command: Command, seconds: u64) -> Result<Output, String> {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // No console window; the selection form is still shown.
    }
    let mut child = command.spawn().map_err(|e| format!("无法启动截图：{e}"))?;
    // Read concurrently: a full screenshot exceeds the pipe buffer and must not block child exit.
    let output = child.stdout.take().ok_or("无法读取截图结果")?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        output.take((MAX_SNAPSHOT_OUTPUT + 1) as u64).read_to_end(&mut bytes).map(|_| bytes)
    });
    let error_output = child.stderr.take().ok_or("无法读取截图错误信息")?;
    let error_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        error_output.take(8192).read_to_end(&mut bytes).map(|_| bytes)
    });
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            result => {
                let _ = child.kill(); let _ = child.wait();
                break Err(match result { Err(e) => format!("截图失败：{e}"), _ => "截图超时，请重试。".to_string() });
            }
        }
    };
    let bytes = reader.join().map_err(|_| "读取截图失败")?.map_err(|_| "读取截图失败")?;
    let stderr = error_reader.join().map_err(|_| "读取截图错误失败")?.map_err(|_| "读取截图错误失败")?;
    if bytes.len() > MAX_SNAPSHOT_OUTPUT { return Err("截图太大，请缩小区域后重试。".into()); }
    Ok(Output { status: status?, stdout: bytes, stderr })
}

#[cfg(target_os = "macos")]
struct TempCapture(std::path::PathBuf);
#[cfg(target_os = "macos")]
impl TempCapture {
    fn new() -> Result<Self, String> {
        use std::os::unix::fs::DirBuilderExt;
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).map_err(|e| e.to_string())?;
        let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
        let path = std::env::temp_dir().join(format!("xiaoban-capture-{suffix}"));
        std::fs::DirBuilder::new().mode(0o700).create(&path).map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
}
#[cfg(target_os = "macos")]
impl Drop for TempCapture { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

#[cfg(target_os = "macos")]
pub fn request_macos_permission() {
    // macOS can keep this false after a grant until the process is relaunched.
    // It is useful for requesting access, not as a hard gate for interactive capture.
    if !unsafe { CGPreflightScreenCaptureAccess() } { unsafe { CGRequestScreenCaptureAccess(); } }
}

/// Reset only this app's ScreenCapture TCC record, after explicit confirmation in the
/// trusted control window. Never invoke automatically on capture failures.
pub const SCREEN_CAPTURE_BUNDLE_ID: &str = "xyz.xiaban.desktop-wallpaper";

#[tauri::command]
pub fn reset_screen_capture_permission(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" { return Err("只能从小伴设置页修复截图权限。".into()); }
    #[cfg(target_os = "macos")]
    {
        // Absolute command path and fixed arguments: no shell, no caller-supplied service or bundle ID.
        // Removing authorization affects the current app only and requires a fresh grant afterwards.
        let output = Command::new("/usr/bin/tccutil")
            .args(["reset", "ScreenCapture", SCREEN_CAPTURE_BUNDLE_ID])
            .output()
            .map_err(|e| format!("无法运行权限修复工具：{e}"))?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr).trim().chars().take(320).collect::<String>();
            return Err(if detail.is_empty() {
                "无法重置截图权限，请到系统设置的「屏幕与系统音频录制」手动移除小伴，再重新授权。".into()
            } else {
                format!("无法重置截图权限：{detail}。请在系统设置中手动移除小伴授权。")
            });
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    { Err("此功能仅用于 macOS。".into()) }
}

/// Open the exact privacy pane; never change another app's permissions.
#[tauri::command]
pub fn open_screen_capture_settings(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" { return Err("只能从小伴设置页打开截图权限设置。".into()); }
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("/usr/bin/open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .status()
            .map_err(|e| format!("无法打开屏幕录制权限设置：{e}"))?;
        if status.success() { Ok(()) } else { Err("无法打开系统截图权限设置，请手动进入「隐私与安全性」。".into()) }
    }
    #[cfg(not(target_os = "macos"))]
    { Err("此功能仅用于 macOS。".into()) }
}

#[cfg(any(target_os = "macos", test))]
fn macos_capture_outcome(success: bool, code: Option<i32>, stderr: &[u8], has_image: bool, permission: bool) -> Result<bool, String> {
    if success && has_image { return Ok(true); }
    let detail = String::from_utf8_lossy(stderr).trim().chars().take(400).collect::<String>();
    if !detail.is_empty() {
        return Err(format!("macOS 截图失败：{detail}。如刚授予权限，请从托盘完全退出小伴后重新打开再试。"));
    }
    if !permission {
        return Err("未生成截图。如果刚授予屏幕录制权限，请从托盘完全退出小伴后重新打开；仍失败时请检查系统隐私设置中的授权应用是否为当前小伴。".into());
    }
    // Esc produces no image and no diagnostic. Other exit codes are actual failures.
    if !has_image && (success || code == Some(1)) { return Ok(false); }
    Err(format!("macOS 截图未完成（退出状态 {code:?}），请重试。"))
}

#[cfg(target_os = "macos")]
pub fn capture(app: &tauri::AppHandle) -> Result<Vec<Frame>, String> {
    let screens = crate::macos::widget_monitors(app)?;
    if screens.len() > 16 { return Err("显示器数量超过截图支持范围".into()); }
    let temp = TempCapture::new()?;
    let mut total = 0;
    let mut frames = Vec::new();
    for (i, screen) in screens.into_iter().enumerate() {
        let image = temp.0.join(format!("screen-{i}.jpg"));
        // -R takes Quartz global points, including negative coordinates. No system selection UI.
        let region = format!("{},{},{},{}",screen.x as i32,screen.y as i32,screen.width as u32,screen.height as u32);
        let mut command = Command::new("/usr/sbin/screencapture");
        command.args(["-x", "-t", "jpg", "-R", &region]).arg(&image);
        let output = run(command, 30)?;
        if !macos_capture_outcome(output.status.success(), output.status.code(), &output.stderr, image.exists(), unsafe { CGPreflightScreenCaptureAccess() })? {
            return Err("未生成屏幕画面，请检查屏幕录制权限后重试。".into());
        }
        // Keep native Retina pixels unless the snapshot would exceed the IPC budget.
        if std::fs::metadata(&image).map_err(|e|e.to_string())?.len() > 12 * 1024 * 1024 {
            let mut resize = Command::new("/usr/bin/sips");
            resize.args(["-Z", "4096", "-s", "formatOptions", "85"]).arg(&image);
            if !run(resize,20)?.status.success() { return Err("截图处理失败，请重试。".into()); }
        }
        let size = std::fs::metadata(&image).map_err(|e|e.to_string())?.len();
        if size == 0 || size > 16 * 1024 * 1024 { return Err("屏幕画面过大，请降低分辨率后重试。".into()); }
        total += size;
        if total > 32 * 1024 * 1024 { return Err("屏幕画面过大，请降低分辨率后重试。".into()); }
        let bytes = std::fs::read(&image).map_err(|e|e.to_string())?;
        if !bytes.starts_with(&[0xff,0xd8]) { return Err("截图格式错误".into()); }
        frames.push(Frame { x:screen.x, y:screen.y, width:screen.width, height:screen.height, image:format!("data:image/jpeg;base64,{}",encode_base64(&bytes)) });
    }
    Ok(frames)
}

#[cfg(windows)]
pub fn capture(_app: &tauri::AppHandle) -> Result<Vec<Frame>, String> {
    let system = std::env::var_os("SystemRoot").ok_or("无法找到 Windows 系统目录")?;
    let exe = std::path::PathBuf::from(system).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut command = Command::new(exe);
    command.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-STA", "-Command", include_str!("assistant_capture_windows.ps1")]);
    let output = run(command, 30)?;
    if !output.status.success() { return Err("无法获取屏幕画面，请检查系统截图权限或策略。".into()); }
    let frames: Vec<Frame> = serde_json::from_slice(&output.stdout).map_err(|_| "截图结果格式错误")?;
    if frames.is_empty() || frames.len() > 16 || frames.iter().any(|f| !f.image.starts_with("data:image/jpeg;base64,") || !f.width.is_finite() || !f.height.is_finite() || f.width <= 0.0 || f.height <= 0.0) { return Err("屏幕画面无效".into()); }
    Ok(frames)
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn capture(_app: &tauri::AppHandle) -> Result<Vec<Frame>, String> { Err("截图目前支持 Windows 和 macOS。".into()) }

pub fn validate_data_url(value: &str) -> Result<(), String> {
    let encoded = value.strip_prefix("data:image/jpeg;base64,").ok_or("截图结果格式错误")?;
    if encoded.is_empty() || encoded.len() > MAX_BYTES / 3 * 4 || encoded.len() % 4 != 0
        || !encoded.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=')) {
        return Err("截图太大或格式错误，请重新截取。".into());
    }
    Ok(())
}

#[cfg(any(target_os = "macos", test))]
fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0] as usize; let b = chunk.get(1).copied().unwrap_or(0) as usize; let c = chunk.get(2).copied().unwrap_or(0) as usize;
        out.push(TABLE[a >> 2] as char); out.push(TABLE[((a & 3) << 4) | (b >> 4)] as char);
        out.push(if chunk.len() > 1 { TABLE[((b & 15) << 2) | (c >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[c & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn reset_targets_only_the_app_bundle_id() {
        assert_eq!(SCREEN_CAPTURE_BUNDLE_ID, "xyz.xiaban.desktop-wallpaper");
    }
    #[test] fn macos_permission_cache_never_overrides_a_successful_capture() {
        assert_eq!(macos_capture_outcome(true, Some(0), b"", true, false), Ok(true));
    }
    #[test] fn macos_reports_diagnostics_instead_of_silently_cancelling() {
        let error = macos_capture_outcome(false, Some(1), b"could not create image from display", false, true).unwrap_err();
        assert!(error.contains("could not create image from display"));
        assert!(macos_capture_outcome(false, Some(2), b"", false, true).is_err());
        assert!(macos_capture_outcome(false, Some(1), b"", false, false).unwrap_err().contains("完全退出"));
    }
    #[test] fn macos_escape_is_quiet_when_access_is_valid() {
        assert_eq!(macos_capture_outcome(false, Some(1), b"", false, true), Ok(false));
    }
    #[test] fn base64_padding_and_binary_bytes() {
        for (input, expected) in [(b"".as_slice(), ""), (b"f", "Zg=="), (b"fo", "Zm8="), (b"foo", "Zm9v"), (&[255, 216, 255], "/9j/")] {
            assert_eq!(encode_base64(input), expected);
        }
    }
    #[test] fn only_bounded_jpeg_data_urls_are_returned() {
        assert!(validate_data_url("data:image/jpeg;base64,/9j/").is_ok());
        for value in ["file:///private/shot.jpg", "data:image/jpeg;base64,", "data:image/jpeg;base64,<svg>"] { assert!(validate_data_url(value).is_err()); }
        assert!(validate_data_url(&format!("data:image/jpeg;base64,{}", "A".repeat(MAX_BYTES / 3 * 4 + 4))).is_err());
    }
}

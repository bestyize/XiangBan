//! User-initiated region capture. Fixed OS commands only; no model-facing command or arbitrary paths.
//! The assistant window is hidden by the caller. Temporary macOS files are removed on every exit.
use std::{io::Read, process::{Command, ExitStatus, Stdio}, time::{Duration, Instant}};

const MAX_BYTES: usize = 3 * 1024 * 1024;
const MAX_DATA_URL: usize = MAX_BYTES / 3 * 4 + 64;

fn run(mut command: Command, seconds: u64) -> Result<(ExitStatus, Vec<u8>), String> {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // No console window; the selection form is still shown.
    }
    let mut child = command.spawn().map_err(|e| format!("无法启动截图：{e}"))?;
    // Read concurrently: a full screenshot exceeds the pipe buffer and must not block child exit.
    let output = child.stdout.take().ok_or("无法读取截图结果")?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        output.take((MAX_DATA_URL + 1) as u64).read_to_end(&mut bytes).map(|_| bytes)
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
    if bytes.len() > MAX_DATA_URL { return Err("截图太大，请缩小区域后重试。".into()); }
    Ok((status?, bytes))
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
fn capture_platform() -> Result<Option<String>, String> {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }
    if !unsafe { CGPreflightScreenCaptureAccess() || CGRequestScreenCaptureAccess() } {
        return Err("请在系统隐私设置中允许小伴录制屏幕，再重试截图。".into());
    }
    let temp = TempCapture::new()?;
    let image = temp.0.join("region.jpg");
    let mut command = Command::new("/usr/sbin/screencapture");
    command.args(["-i", "-s", "-x", "-t", "jpg"]).arg(&image);
    let (status, _) = run(command, 120)?;
    if !status.success() || !image.exists() { return Ok(None); } // Esc is cancellation, not an error.
    let path = if std::fs::metadata(&image).map_err(|e| e.to_string())?.len() > MAX_BYTES as u64 {
        let smaller = temp.0.join("scaled.jpg");
        let mut resize = Command::new("/usr/bin/sips");
        resize.args(["-Z", "2048", "-s", "format", "jpeg", "-s", "formatOptions", "80"]).arg(&image).arg("--out").arg(&smaller);
        if !run(resize, 20)?.0.success() { return Err("截图处理失败，请重试。".into()); }
        smaller
    } else { image };
    let len = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
    if len == 0 { return Ok(None); }
    if len > MAX_BYTES as u64 { return Err("截图太大，请缩小区域后重试。".into()); }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if !bytes.starts_with(&[0xff, 0xd8]) { return Err("截图格式不正确，请重试。".into()); }
    Ok(Some(format!("data:image/jpeg;base64,{}", encode_base64(&bytes))))
}

#[cfg(windows)]
fn capture_platform() -> Result<Option<String>, String> {
    // Windows PowerShell and .NET Framework are OS components; all script content is compiled in.
    let system = std::env::var_os("SystemRoot").ok_or("无法找到 Windows 系统目录")?;
    let exe = std::path::PathBuf::from(system).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut command = Command::new(exe);
    command.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-STA", "-Command", include_str!("assistant_capture_windows.ps1")]);
    let (status, bytes) = run(command, 120)?;
    if !status.success() { return Err("截图未能启动，请重试；系统策略可能禁止运行截图组件。".into()); }
    let result = String::from_utf8(bytes).map_err(|_| "截图结果格式错误")?;
    let result = result.trim();
    if result.is_empty() { return Ok(None); }
    validate_data_url(result)?;
    Ok(Some(result.to_string()))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn capture_platform() -> Result<Option<String>, String> { Err("截图目前支持 Windows 和 macOS。".into()) }

pub fn capture() -> Result<Option<String>, String> { capture_platform() }

#[cfg(any(windows, test))]
fn validate_data_url(value: &str) -> Result<(), String> {
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

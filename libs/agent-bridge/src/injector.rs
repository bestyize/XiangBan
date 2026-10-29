//! Desktop application text injector: window activation, clipboard preservation, and hotkey paste.
//!
//! Implements the "先上屏后发送" (show text on screen first, then send) workflow from P2.
//! Injects text into Claude Desktop, ChatGPT Desktop, Codex Desktop, or any specified window.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Known desktop application targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopTarget {
    /// Anthropic Claude Desktop (`Claude.exe` / `Claude.app`).
    Claude,
    /// OpenAI ChatGPT Desktop (`ChatGPT.exe` / `ChatGPT.app`).
    ChatGpt,
    /// OpenAI Codex Desktop (`Codex.exe` / `Codex.app`).
    Codex,
    /// Any application with matching window title.
    Custom,
}

impl DesktopTarget {
    /// Return the canonical process or window title query for this target.
    pub fn query(&self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::ChatGpt => "ChatGPT",
            Self::Codex => "Codex",
            Self::Custom => "",
        }
    }
}

/// Request to inject text into a target application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionRequest {
    /// Target application or window title substring.
    pub target: String,
    /// Text to paste into the application's active input.
    pub text: String,
    /// Whether to press Enter after pasting (false = show on screen only).
    #[serde(default)]
    pub press_enter: bool,
    /// Milliseconds to wait before restoring the original clipboard content (default: 300 ms).
    #[serde(default)]
    pub restore_clipboard_ms: Option<u64>,
}

/// Performance and status report of a desktop injection attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InjectionResult {
    /// Whether the injection succeeded end-to-end.
    pub ok: bool,
    /// Whether a target window was found and brought forward.
    pub target_found: bool,
    /// Full title of the window operated upon.
    pub window_title: Option<String>,
    /// Milliseconds spent finding and bringing window to foreground.
    pub activate_ms: u64,
    /// Milliseconds spent pasting text and pressing enter.
    pub paste_ms: u64,
    /// Total duration in milliseconds.
    pub total_ms: u64,
    /// Error message if unsuccessful.
    pub error: Option<String>,
}

/// Longest target query and longest single paste accepted.
pub const MAX_TARGET_CHARS: usize = 64;
/// One paste is one utterance or one edited message, never a document.
pub const MAX_INJECT_CHARS: usize = 20_000;

fn failure(error: impl Into<String>) -> InjectionResult {
    InjectionResult {
        ok: false,
        target_found: false,
        window_title: None,
        activate_ms: 0,
        paste_ms: 0,
        total_ms: 0,
        error: Some(error.into()),
    }
}

/// The target is interpolated into an AppleScript on macOS, so anything that could leave the string literal is refused
/// on every platform. `text` may be empty only when the request just presses Enter (sending what was already pasted).
pub fn validate_request(req: &InjectionRequest) -> Result<(), String> {
    let target = req.target.trim();
    if target.is_empty() {
        return Err("没有指定要送字的应用".into());
    }
    if target.chars().count() > MAX_TARGET_CHARS {
        return Err(format!("应用名称太长（最多 {MAX_TARGET_CHARS} 个字符）"));
    }
    if req.target.chars().any(|c| c.is_control() || c == '"' || c == '\\') {
        return Err("应用名称里含有不允许的字符".into());
    }
    if req.text.is_empty() && !req.press_enter {
        return Err("注入内容不能为空".into());
    }
    if req.text.chars().count() > MAX_INJECT_CHARS {
        return Err(format!("一次最多送 {MAX_INJECT_CHARS} 个字符"));
    }
    Ok(())
}

/// AppleScript that brings `process` to the front, checks it really is in front, then pastes and/or presses Return.
/// Returns "ok", "not_found" or "not_front". Pasting into whatever happens to be in front would put the words in
/// the wrong app, so a failed activation must not type anything.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn mac_script(process: &str, paste: bool, press_enter: bool) -> String {
    let paste_clause = if paste { "keystroke \"v\" using command down\ndelay 0.1\n" } else { "" };
    let enter_clause = if press_enter { "key code 36\n" } else { "" };
    format!(
        "tell application \"System Events\"\n\
            if exists (process \"{process}\") then\n\
                tell process \"{process}\"\n\
                    set frontmost to true\n\
                    delay 0.2\n\
                    if frontmost is not true then return \"not_front\"\n\
                    {paste_clause}{enter_clause}\
                end tell\n\
                return \"ok\"\n\
            else\n\
                return \"not_found\"\n\
            end if\n\
        end tell"
    )
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use std::thread;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, TRUE};
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    };
    use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL, VK_MENU, VK_RETURN,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, IsIconic, IsWindowVisible,
        SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    const CF_UNICODETEXT: u32 = 13;
    const VK_V: u16 = 0x56;

    struct WindowMatch {
        hwnd: HWND,
        title: String,
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> i32 {
        if IsWindowVisible(hwnd) == 0 {
            return TRUE;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return TRUE;
        }
        let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
        let copied = GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1);
        if copied == 0 {
            return TRUE;
        }
        let title = String::from_utf16_lossy(&buf[..copied as usize]);
        let data = &mut *(lparam as *mut (String, Option<WindowMatch>));
        let query = &data.0;
        if title.to_lowercase().contains(&query.to_lowercase()) {
            data.1 = Some(WindowMatch { hwnd, title });
            return 0; // stop enumeration
        }
        TRUE
    }

    fn find_window(target: &str) -> Option<WindowMatch> {
        let mut pair = (target.to_string(), None);
        unsafe {
            EnumWindows(Some(enum_windows_callback), &mut pair as *mut _ as LPARAM);
        }
        pair.1
    }

    pub fn get_clipboard_text() -> Option<String> {
        unsafe {
            if OpenClipboard(ptr::null_mut()) == 0 {
                return None;
            }
            let handle = GetClipboardData(CF_UNICODETEXT);
            if handle.is_null() {
                CloseClipboard();
                return None;
            }
            let ptr = GlobalLock(handle) as *const u16;
            if ptr.is_null() {
                CloseClipboard();
                return None;
            }
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf16_lossy(slice);
            GlobalUnlock(handle);
            CloseClipboard();
            Some(text)
        }
    }

    pub fn set_clipboard_text(text: &str) -> Result<(), String> {
        let utf16: Vec<u16> = OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect();
        let bytes_len = utf16.len() * 2;
        unsafe {
            let mut retries = 0;
            while OpenClipboard(ptr::null_mut()) == 0 {
                retries += 1;
                if retries > 5 {
                    return Err("打开剪贴板超时".into());
                }
                thread::sleep(Duration::from_millis(40));
            }
            EmptyClipboard();
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, bytes_len);
            if h_mem.is_null() {
                CloseClipboard();
                return Err("分配剪贴板内存失败".into());
            }
            let dest = GlobalLock(h_mem) as *mut u16;
            if dest.is_null() {
                CloseClipboard();
                return Err("锁定剪贴板内存失败".into());
            }
            ptr::copy_nonoverlapping(utf16.as_ptr(), dest, utf16.len());
            GlobalUnlock(h_mem);
            SetClipboardData(CF_UNICODETEXT, h_mem);
            CloseClipboard();
        }
        Ok(())
    }

    fn key_input(vk: u16, keyup: bool) -> INPUT {
        let mut input: INPUT = unsafe { std::mem::zeroed() };
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wVk = vk;
        input.Anonymous.ki.dwFlags = if keyup { KEYEVENTF_KEYUP } else { 0 };
        input
    }

    /// Puts the window in front and confirms it is. Windows only lets the process that last received input take the
    /// foreground, so after a failed attempt a synthetic Alt press (which counts as input) is sent before retrying.
    /// Returns false when the window never got there, so nothing is typed into whatever else is in front.
    fn bring_to_front(hwnd: HWND) -> bool {
        unsafe {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            }
            for attempt in 0..6 {
                if GetForegroundWindow() == hwnd {
                    return true;
                }
                SetForegroundWindow(hwnd);
                thread::sleep(Duration::from_millis(60));
                if GetForegroundWindow() == hwnd {
                    return true;
                }
                if attempt >= 1 {
                    let mut alt = [key_input(VK_MENU, false), key_input(VK_MENU, true)];
                    SendInput(alt.len() as u32, alt.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
                }
            }
            GetForegroundWindow() == hwnd
        }
    }

    fn send_keys(paste: bool, press_enter: bool) {
        let mut inputs = Vec::new();
        if paste {
            inputs.extend([
                key_input(VK_CONTROL, false),
                key_input(VK_V, false),
                key_input(VK_V, true),
                key_input(VK_CONTROL, true),
            ]);
        }
        if press_enter {
            inputs.push(key_input(VK_RETURN, false));
            inputs.push(key_input(VK_RETURN, true));
        }
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }

    pub fn inject(req: &InjectionRequest) -> InjectionResult {
        let start = Instant::now();
        let target_window = match find_window(&req.target) {
            Some(w) => w,
            None => {
                return InjectionResult {
                    ok: false,
                    target_found: false,
                    window_title: None,
                    activate_ms: 0,
                    paste_ms: 0,
                    total_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("未找到标题包含「{}」的可视窗口", req.target)),
                };
            }
        };

        // 只按回车（发送刚才送进去的文字）时不碰剪贴板
        let paste = !req.text.is_empty();

        // 1. 保护旧剪贴板内容
        let old_clipboard = if paste { get_clipboard_text() } else { None };

        // 2. 写入要注入的测试/语音文字
        if paste {
            if let Err(e) = set_clipboard_text(&req.text) {
                return InjectionResult {
                    ok: false,
                    target_found: true,
                    window_title: Some(target_window.title),
                    activate_ms: 0,
                    paste_ms: 0,
                    total_ms: start.elapsed().as_millis() as u64,
                    error: Some(e),
                };
            }
        }

        // 3. 激活窗口并置顶，确认真的在最前面才粘贴，否则文字会进错窗口
        let act_start = Instant::now();
        if !bring_to_front(target_window.hwnd) {
            if let Some(old) = old_clipboard {
                let _ = set_clipboard_text(&old);
            }
            return InjectionResult {
                ok: false,
                target_found: true,
                window_title: Some(target_window.title.clone()),
                activate_ms: act_start.elapsed().as_millis() as u64,
                paste_ms: 0,
                total_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("无法把「{}」切到前台，已取消粘贴，避免文字进错窗口", target_window.title)),
            };
        }
        thread::sleep(Duration::from_millis(120));
        let activate_ms = act_start.elapsed().as_millis() as u64;

        // 4. 发送 Ctrl+V（及可选 Enter）
        let paste_start = Instant::now();
        send_keys(paste, req.press_enter);
        let paste_ms = paste_start.elapsed().as_millis() as u64;

        // 5. 延迟恢复剪贴板内容
        let restore_delay = Duration::from_millis(req.restore_clipboard_ms.unwrap_or(300));
        thread::sleep(restore_delay);
        if let Some(old) = old_clipboard {
            let _ = set_clipboard_text(&old);
        }

        let total_ms = start.elapsed().as_millis() as u64;
        InjectionResult {
            ok: true,
            target_found: true,
            window_title: Some(target_window.title),
            activate_ms,
            paste_ms,
            total_ms,
            error: None,
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::process::Command;

    pub fn get_clipboard_text() -> Option<String> {
        let out = Command::new("pbpaste").output().ok()?;
        if out.status.success() {
            Some(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            None
        }
    }

    pub fn set_clipboard_text(text: &str) -> Result<(), String> {
        use std::io::Write;
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
        child.wait().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn inject(req: &InjectionRequest) -> InjectionResult {
        let start = Instant::now();
        let target_name = req.target.trim();
        let paste = !req.text.is_empty();
        let old_clipboard = if paste { get_clipboard_text() } else { None };

        if paste {
            if let Err(e) = set_clipboard_text(&req.text) {
                return InjectionResult {
                    ok: false,
                    target_found: false,
                    window_title: None,
                    activate_ms: 0,
                    paste_ms: 0,
                    total_ms: start.elapsed().as_millis() as u64,
                    error: Some(e),
                };
            }
        }

        let script = mac_script(target_name, paste, req.press_enter);
        let out = Command::new("osascript").args(["-e", &script]).output();

        let (ok, target_found, error) = match out {
            Ok(output) => {
                let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                if res == "ok" {
                    (true, true, None)
                } else if res == "not_found" {
                    (false, false, Some(format!("目标应用 {target_name} 未在运行")))
                } else if res == "not_front" {
                    (false, true, Some(format!("无法把 {target_name} 切到前台，已取消粘贴，避免文字进错应用")))
                } else if stderr.contains("1002") || stderr.contains("not allowed") || stderr.contains("不允许") {
                    (false, true, Some("没有辅助功能权限：请在 系统设置 → 隐私与安全性 → 辅助功能 里允许小伴，然后重试".into()))
                } else {
                    (false, false, Some(format!("向 {target_name} 送字失败：{}", stderr.trim())))
                }
            }
            Err(e) => (false, false, Some(e.to_string())),
        };

        if let Some(old) = old_clipboard {
            std::thread::sleep(std::time::Duration::from_millis(req.restore_clipboard_ms.unwrap_or(300)));
            let _ = set_clipboard_text(&old);
        }

        InjectionResult {
            ok,
            target_found,
            window_title: Some(target_name.to_string()),
            activate_ms: 300,
            paste_ms: 100,
            total_ms: start.elapsed().as_millis() as u64,
            error,
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    use super::*;

    pub fn get_clipboard_text() -> Option<String> { None }
    pub fn set_clipboard_text(_text: &str) -> Result<(), String> { Ok(()) }
    pub fn inject(req: &InjectionRequest) -> InjectionResult {
        InjectionResult {
            ok: false,
            target_found: false,
            window_title: None,
            activate_ms: 0,
            paste_ms: 0,
            total_ms: 0,
            error: Some("当前平台尚未支持窗口热键注入".into()),
        }
    }
}

/// Inject text into the active input of an external desktop application with clipboard preservation.
pub fn inject_text(req: InjectionRequest) -> InjectionResult {
    if let Err(error) = validate_request(&req) {
        return failure(error);
    }
    platform::inject(&req)
}

/// Convenience function: read current clipboard text.
pub fn get_clipboard() -> Option<String> {
    platform::get_clipboard_text()
}

/// Convenience function: write clipboard text.
pub fn set_clipboard(text: &str) -> Result<(), String> {
    platform::set_clipboard_text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_target_queries_match_known_apps() {
        assert_eq!(DesktopTarget::Claude.query(), "Claude");
        assert_eq!(DesktopTarget::ChatGpt.query(), "ChatGPT");
        assert_eq!(DesktopTarget::Codex.query(), "Codex");
    }

    fn request(target: &str, text: &str, press_enter: bool) -> InjectionRequest {
        InjectionRequest { target: target.into(), text: text.into(), press_enter, restore_clipboard_ms: Some(10) }
    }

    #[test]
    fn requests_are_validated_before_any_platform_call() {
        assert!(validate_request(&request("Claude", "你好", false)).is_ok());
        assert!(validate_request(&request("Claude", "", true)).is_ok(), "只按回车是允许的");
        assert!(validate_request(&request("Claude", "", false)).is_err(), "既没有文字也不按回车");
        assert!(validate_request(&request("  ", "x", false)).is_err());
        assert!(validate_request(&request(&"a".repeat(MAX_TARGET_CHARS + 1), "x", false)).is_err());
        assert!(validate_request(&request("Claude", &"字".repeat(MAX_INJECT_CHARS + 1), false)).is_err());
        // 目标会被拼进 AppleScript：任何能跳出字符串字面量的字符都必须拒绝
        for bad in ["Cl\"aude", "Claude\\", "Claude\n", "x\" & (do shell script \"id\") & \"", "a\tb"] {
            assert!(validate_request(&request(bad, "x", false)).is_err(), "{bad:?}");
        }
        let refused = inject_text(request("a\"b", "x", false));
        assert!(!refused.ok && refused.error.is_some());
    }

    #[test]
    fn mac_script_only_types_after_confirming_the_app_is_in_front() {
        let script = mac_script("Claude", true, true);
        let front = script.find("not_front").unwrap();
        assert!(front < script.find("keystroke").unwrap() && script.find("keystroke").unwrap() < script.find("key code 36").unwrap());
        assert!(!mac_script("Claude", false, true).contains("keystroke"));
        assert!(!mac_script("Claude", true, false).contains("key code"));
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn clipboard_roundtrip_preserves_content() {
        let original = get_clipboard();
        let marker = format!("xiaoban-test-{}", std::process::id());
        assert!(set_clipboard(&marker).is_ok());
        assert_eq!(get_clipboard().as_deref(), Some(marker.as_str()));
        if let Some(old) = original {
            let _ = set_clipboard(&old);
        }
    }

    #[test]
    fn non_existent_window_fails_gracefully_with_informative_error() {
        let req = InjectionRequest {
            target: "NonExistentWindowXYZ_123456".into(),
            text: "test".into(),
            press_enter: false,
            restore_clipboard_ms: Some(10),
        };
        let res = inject_text(req);
        assert!(!res.ok);
        assert!(!res.target_found);
        assert!(res.error.is_some());
    }
}

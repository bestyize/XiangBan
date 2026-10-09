//! Read-only session/idle sampling. No hooks, captured input, or location access.
pub struct Activity {
    pub idle_seconds: f64,
    pub locked: bool,
    pub available: bool,
}

pub fn play_sound() {
    #[cfg(windows)]
    unsafe { windows_sys::Win32::System::Diagnostics::Debug::MessageBeep(0); }
    #[cfg(target_os = "macos")]
    { let _ = std::process::Command::new("/usr/bin/afplay").arg("/System/Library/Sounds/Pop.aiff")
        .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn(); }
}

/// Showing a care bubble must never activate it or replace keyboard focus.
#[cfg(windows)]
pub fn show_bubble(window: &tauri::WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::{Foundation::{GetLastError, SetLastError}, UI::WindowsAndMessaging::*};
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetLastError(0);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE as isize | WS_EX_TOOLWINDOW as isize);
        let error = GetLastError();
        if error != 0 { return Err(format!("无法设置关怀气泡不激活属性：{error}")); }
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        if SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED) == 0 {
            return Err(format!("无法显示关怀气泡：{}", GetLastError()));
        }
    }
    Ok(())
}
#[cfg(not(windows))]
pub fn show_bubble(window: &tauri::WebviewWindow) -> Result<(), String> {
    // The builder also sets focused(false) and focusable(false).
    window.show().map_err(|e| e.to_string())
}
#[cfg(windows)]
pub fn hide_bubble(window: &tauri::WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindow, ShowWindow, SW_HIDE};
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;
    unsafe {
        if IsWindow(hwnd) == 0 { return Err("关怀气泡窗口已关闭".into()); }
        // Use the same native lifecycle as show_bubble. A queued Tauri hide
        // must not run after a subsequent native show and hide the new reminder.
        ShowWindow(hwnd, SW_HIDE);
    }
    Ok(())
}
#[cfg(not(windows))]
pub fn hide_bubble(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}
pub fn bubble_diagnostics(window: &tauri::WebviewWindow) -> Result<serde_json::Value, String> {
    let visible = window.is_visible().map_err(|e| e.to_string())?;
    let focused = window.is_focused().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;
        let foreground = unsafe { GetForegroundWindow() };
        let no_activate = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } & WS_EX_NOACTIVATE as isize != 0;
        return Ok(serde_json::json!({"visible":visible,"focused":focused,"nativeVisible":unsafe { IsWindowVisible(hwnd) } != 0,
            "hwnd":hwnd as usize,"foregroundHwnd":foreground as usize,"foregroundIsBubble":foreground == hwnd,"noActivate":no_activate}));
    }
    #[cfg(not(windows))]
    Ok(serde_json::json!({"visible":visible,"focused":focused}))
}

#[cfg(windows)]
pub fn activity() -> Activity {
    use windows_sys::Win32::{
        System::{StationsAndDesktops::{OpenInputDesktop, CloseDesktop, GetUserObjectInformationW, UOI_NAME, DESKTOP_READOBJECTS}, SystemInformation::GetTickCount},
        UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    };
    unsafe {
        let desktop = OpenInputDesktop(0, 0, DESKTOP_READOBJECTS);
        let locked = if desktop.is_null() { true } else {
            let mut name = [0u16; 128];
            let readable = GetUserObjectInformationW(desktop, UOI_NAME, name.as_mut_ptr().cast(), std::mem::size_of_val(&name) as u32, std::ptr::null_mut()) != 0;
            CloseDesktop(desktop);
            let length = name.iter().position(|c| *c == 0).unwrap_or(name.len());
            !readable || !String::from_utf16_lossy(&name[..length]).eq_ignore_ascii_case("default")
        };
        let mut input = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        let available = GetLastInputInfo(&mut input) != 0;
        Activity { idle_seconds: GetTickCount().wrapping_sub(input.dwTime) as f64 / 1000.0, locked, available }
    }
}

#[cfg(target_os = "macos")]
pub fn activity() -> Activity {
    use std::ffi::{c_void, c_char};
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
        fn CGSessionCopyCurrentDictionary() -> *const c_void;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(allocator: *const c_void, value: *const c_char, encoding: u32) -> *const c_void;
        fn CFDictionaryGetValue(dictionary: *const c_void, key: *const c_void) -> *const c_void;
        fn CFBooleanGetValue(value: *const c_void) -> u8;
        fn CFRelease(value: *const c_void);
    }
    unsafe {
        let idle_seconds = CGEventSourceSecondsSinceLastEventType(1, u32::MAX);
        let dictionary = CGSessionCopyCurrentDictionary();
        let locked = if dictionary.is_null() { true } else {
            let key = CFStringCreateWithCString(std::ptr::null(), c"CGSSessionScreenIsLocked".as_ptr(), 0x08000100);
            let value = CFDictionaryGetValue(dictionary, key);
            let locked = !value.is_null() && CFBooleanGetValue(value) != 0;
            CFRelease(key);
            CFRelease(dictionary);
            locked
        };
        Activity { idle_seconds, locked, available: idle_seconds.is_finite() && idle_seconds >= 0.0 }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn activity() -> Activity {
    Activity { idle_seconds: f64::INFINITY, locked: false, available: false }
}

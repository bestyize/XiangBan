//! Windows desktop hosting and read-only cursor sampling. No hooks, keyboard capture,
//! wallpaper registry changes, or Explorer window destruction are used.
use std::{ptr::null_mut, time::Duration};
use tauri::{Emitter, Manager};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{ClientToScreen, ScreenToClient},
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
        WindowsAndMessaging::*,
    },
};

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn class_name(hwnd: HWND) -> String {
    let mut text = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, text.as_mut_ptr(), text.len() as i32) };
    String::from_utf16_lossy(&text[..n.max(0) as usize])
}

unsafe extern "system" fn find_background(hwnd: HWND, data: LPARAM) -> i32 {
    let defview = FindWindowExW(
        hwnd,
        null_mut(),
        wide("SHELLDLL_DefView").as_ptr(),
        std::ptr::null(),
    );
    if !defview.is_null() {
        let next = FindWindowExW(null_mut(), hwnd, wide("WorkerW").as_ptr(), std::ptr::null());
        if !next.is_null() {
            *(data as *mut HWND) = next;
            return 0;
        }
        // Newer Explorer versions can put the background WorkerW inside Progman.
        let child = FindWindowExW(hwnd, null_mut(), wide("WorkerW").as_ptr(), std::ptr::null());
        if !child.is_null() {
            *(data as *mut HWND) = child;
            return 0;
        }
    }
    1
}

pub fn desktop_parent() -> Result<usize, String> {
    unsafe {
        let progman = FindWindowW(wide("Progman").as_ptr(), std::ptr::null());
        if progman.is_null() {
            return Err("找不到 Windows 桌面，请确认 Explorer 正常运行".into());
        }
        let mut result = 0usize;
        // Explorer's de facto wallpaper hosting message; bounded to avoid a hung shell.
        SendMessageTimeoutW(progman, 0x052c, 0, 0, SMTO_ABORTIFHUNG, 1000, &mut result);
        let mut parent: HWND = null_mut();
        EnumWindows(Some(find_background), &mut parent as *mut HWND as LPARAM);
        if parent.is_null() {
            SendMessageTimeoutW(
                progman,
                0x052c,
                0x0d,
                1,
                SMTO_ABORTIFHUNG,
                1000,
                &mut result,
            );
            EnumWindows(Some(find_background), &mut parent as *mut HWND as LPARAM);
        }
        if parent.is_null() {
            return Err("没有找到桌面背景层；已保留悬浮模式。请确认桌面图标已显示".into());
        }
        Ok(parent as usize)
    }
}

pub fn attach_wallpaper(
    window: &tauri::WebviewWindow,
    monitor: &tauri::Monitor,
    parent: usize,
) -> Result<usize, String> {
    let parent = parent as HWND;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            (style & !(WS_POPUP as isize)) | WS_CHILD as isize,
        );
        SetLastError(0);
        SetParent(hwnd, parent);
        let error = GetLastError();
        if GetParent(hwnd) != parent {
            SetWindowLongPtrW(hwnd, GWL_STYLE, style);
            return Err(format!(
                "嵌入桌面失败（Windows 错误 {error}），已保留悬浮模式"
            ));
        }
        let mut origin = POINT {
            x: monitor.position().x,
            y: monitor.position().y,
        };
        ScreenToClient(parent, &mut origin);
        if SetWindowPos(
            hwnd,
            null_mut(),
            origin.x,
            origin.y,
            monitor.size().width as i32,
            monitor.size().height as i32,
            SWP_NOACTIVATE | SWP_NOZORDER | SWP_FRAMECHANGED,
        ) == 0
        {
            return Err(format!(
                "设置壁纸尺寸失败（Windows 错误 {}）",
                GetLastError()
            ));
        }
    }
    Ok(parent as usize)
}

pub fn left_button_down() -> bool {
    unsafe { GetAsyncKeyState(VK_LBUTTON as i32) < 0 }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CursorSample {
    x: f64,
    y: f64,
    moved: bool,
    pressed: bool,
    released: bool,
    desktop: bool,
}

pub fn start_cursor_bridge(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut previous = POINT {
            x: i32::MIN,
            y: i32::MIN,
        };
        let mut previous_down = false;
        let mut ticks = 0u32;
        loop {
            std::thread::sleep(Duration::from_millis(33));
            let mut cursor = POINT::default();
            if unsafe { GetCursorPos(&mut cursor) } == 0 {
                continue;
            }
            let moved = cursor.x != previous.x || cursor.y != previous.y;
            let down = left_button_down();
            let pressed = down && !previous_down;
            let released = !down && previous_down;
            previous = cursor;
            previous_down = down;
            ticks += 1;
            let under = unsafe { WindowFromPoint(cursor) };
            let desktop = ["WorkerW", "Progman", "SysListView32", "SHELLDLL_DefView"]
                .contains(&class_name(under).as_str());
            for label in ["pet", "wallpaper", "media-a", "media-b"] {
                let Some(window) = app.get_webview_window(label) else {
                    continue;
                };
                let Ok(native) = window.hwnd() else {
                    continue;
                };
                let hwnd = native.0 as HWND;
                let mut point = cursor;
                unsafe {
                    ScreenToClient(hwnd, &mut point);
                }
                let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as f64 / 96.0;
                let sample = CursorSample {
                    x: point.x as f64 / dpi,
                    y: point.y as f64 / dpi,
                    moved,
                    pressed,
                    released,
                    desktop: desktop || under == hwnd,
                };
                if crate::content::media_label(label) { crate::content::send_cursor(&window, &sample); }
                else { let _ = window.emit("desktop-cursor", sample); }
                if label == "wallpaper" && ticks % 60 == 0 {
                    let parent = unsafe { GetParent(hwnd) };
                    if parent.is_null() || unsafe { IsWindow(parent) } == 0 {
                        let _ = app.emit("desktop-error", "桌面背景层已变化，请重新切换壁纸模式");
                    }
                }
            }
        }
    });
}

// Check the client area in physical screen pixels, not merely the outer HWND
// rectangle: an undecorated shadow can occupy part of an otherwise full-size window.
pub fn wallpaper_geometry(window: &tauri::WebviewWindow) -> Result<serde_json::Value, String> {
    let monitor = window.app_handle().primary_monitor().map_err(|e| e.to_string())?
        .ok_or("没有主显示器")?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
    let mut client = RECT::default();
    let mut origin = POINT::default();
    unsafe {
        if GetClientRect(hwnd, &mut client) == 0 || ClientToScreen(hwnd, &mut origin) == 0 {
            return Err(format!("读取壁纸绘图区失败（Windows 错误 {}）", GetLastError()));
        }
    }
    let width = client.right - client.left;
    let height = client.bottom - client.top;
    let fills = origin.x == monitor.position().x && origin.y == monitor.position().y
        && width == monitor.size().width as i32 && height == monitor.size().height as i32;
    Ok(serde_json::json!({"x":origin.x,"y":origin.y,"width":width,"height":height,
        "monitor":{"x":monitor.position().x,"y":monitor.position().y,
        "width":monitor.size().width,"height":monitor.size().height},"fillsMonitor":fills}))
}

pub fn diagnostics(app: &tauri::AppHandle) -> serde_json::Value {
    let describe = |label| {
        app.get_webview_window(label)
            .and_then(|w| w.hwnd().ok())
            .map(|native| unsafe {
                let hwnd = native.0 as HWND;
                let parent = GetParent(hwnd);
                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect);
                serde_json::json!({ "hwnd": hwnd as usize, "parent": parent as usize,
                "parentClass": class_name(parent), "visible": IsWindowVisible(hwnd) != 0,
                "width": rect.right, "height": rect.bottom, "dpi": GetDpiForWindow(hwnd),
                "isChild": GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CHILD as isize != 0 })
            })
    };
    let geometry = app.get_webview_window("wallpaper").and_then(|w| wallpaper_geometry(&w).ok());
    serde_json::json!({"platform": "windows", "wallpaperSupported": true,
        "mode": super::settings(app).mode, "floating": describe("pet"), "wallpaper": describe("wallpaper"),
        "wallpaperGeometry":geometry})
}

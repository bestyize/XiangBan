//! Windows desktop hosting, cursor sampling, and a read-only desktop double-click hook.
//! No mouse input is consumed; icon clicks remain with Explorer.
use std::{ptr::null_mut, sync::atomic::{AtomicBool, Ordering}, time::Duration};
use tauri::{Emitter, Manager};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{ClientToScreen, ScreenToClient},
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{GetAsyncKeyState, GetDoubleClickTime, VK_LBUTTON},
        WindowsAndMessaging::*,
    },
    System::Threading::GetCurrentThreadId,
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
    attach_desktop(window, parent, *monitor.position(), monitor.size().width as i32, monitor.size().height as i32)
}

unsafe extern "system" fn find_icon_host(hwnd: HWND, data: LPARAM) -> i32 {
    let host=FindWindowExW(hwnd,null_mut(),wide("SHELLDLL_DefView").as_ptr(),std::ptr::null());
    if !host.is_null(){*(data as *mut HWND)=host;return 0;}1
}
pub fn widget_parent() -> Result<usize,String> {
    unsafe { let mut host: HWND=null_mut();EnumWindows(Some(find_icon_host),&mut host as *mut HWND as LPARAM);
        if host.is_null(){return Err("找不到桌面图标层，请确认 Explorer 正常运行".into());}Ok(host as usize) }
}
/// Host small interactive windows above Explorer's icon list, below application windows.
pub fn attach_widget(window: &tauri::WebviewWindow) -> Result<(),String> {
    let hwnd=window.hwnd().map_err(|e|e.to_string())?.0 as HWND;
    unsafe {
        let host=widget_parent()? as HWND;
        let style=GetWindowLongPtrW(hwnd,GWL_STYLE);
        SetWindowLongPtrW(hwnd,GWL_STYLE,(style&!(WS_POPUP as isize))|WS_CHILD as isize);
        SetLastError(0);SetParent(hwnd,host);
        if GetParent(hwnd)!=host{return Err(format!("组件嵌入桌面失败：{}",GetLastError()));}
        if SetWindowPos(hwnd,HWND_TOP,0,0,0,0,SWP_NOMOVE|SWP_NOSIZE|SWP_NOACTIVATE|SWP_FRAMECHANGED)==0{return Err("组件层级设置失败".into());}
    }Ok(())
}
pub fn place_widget(window:&tauri::WebviewWindow,position:tauri::PhysicalPosition<i32>,width:i32,height:i32)->Result<(),String>{
    let hwnd=window.hwnd().map_err(|e|e.to_string())?.0 as HWND;
    unsafe{let mut parent=GetParent(hwnd);if parent.is_null()||IsWindow(parent)==0{attach_widget(window)?;parent=GetParent(hwnd);}
        let mut origin=POINT{x:position.x,y:position.y};ScreenToClient(parent,&mut origin);
        if SetWindowPos(hwnd,HWND_TOP,origin.x,origin.y,width,height,SWP_NOACTIVATE)==0{return Err(format!("组件位置设置失败：{}",GetLastError()));}}
    Ok(())
}

/// Place either a top-level surface or an Explorer child using screen coordinates.
pub fn place_interactive(window:&tauri::WebviewWindow,x:f64,y:f64,width:f64,height:f64)->Result<(),String>{
    let hwnd=window.hwnd().map_err(|e|e.to_string())?.0 as HWND;
    unsafe {
        let mut point=POINT{x:x.round() as i32,y:y.round() as i32};
        let parent=GetParent(hwnd);
        if !parent.is_null(){ScreenToClient(parent,&mut point);}
        if SetWindowPos(hwnd,null_mut(),point.x,point.y,width.round() as i32,height.round() as i32,SWP_NOACTIVATE|SWP_NOZORDER)==0{return Err(format!("窗口移动失败：{}",GetLastError()));}
    }
    Ok(())
}

fn attach_desktop(
    window: &tauri::WebviewWindow,
    parent: usize,
    position: tauri::PhysicalPosition<i32>,
    width: i32,
    height: i32,
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
            x: position.x,
            y: position.y,
        };
        ScreenToClient(parent, &mut origin);
        if SetWindowPos(
            hwnd,
            null_mut(),
            origin.x,
            origin.y,
            width,
            height,
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

/// Preserve the pet's screen position while moving it between Explorer's background
/// and a floating top-level window. Turning off topmost alone does not put it below icons.
pub fn configure_companion(window: &tauri::WebviewWindow, desktop: bool) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
    if unsafe { GetWindowThreadProcessId(hwnd, null_mut()) } == unsafe { GetCurrentThreadId() } {
        return configure_companion_on_main(window, desktop);
    }
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let target = window.clone();
    window.run_on_main_thread(move || { let _ = sender.send(configure_companion_on_main(&target, desktop)); })
        .map_err(|e| e.to_string())?;
    receiver.recv().map_err(|e| e.to_string())?
}

fn configure_companion_on_main(window: &tauri::WebviewWindow, desktop: bool) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
    let mut client = RECT::default();
    let mut origin = POINT::default();
    unsafe {
        if GetClientRect(hwnd, &mut client) == 0 || ClientToScreen(hwnd, &mut origin) == 0 {
            return Err("无法读取小伴窗口位置".into());
        }
    }
    if desktop {
        let parent = desktop_parent()?;
        window.set_always_on_top(false).map_err(|e| e.to_string())?;
        attach_desktop(window, parent, tauri::PhysicalPosition::new(origin.x, origin.y),
            client.right - client.left, client.bottom - client.top)?;
    } else {
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            // Clear child style before detaching; otherwise SetParent(NULL)
            // leaves it hosted by the desktop and HWND_TOPMOST has no effect.
            SetWindowLongPtrW(hwnd, GWL_STYLE, (style & !(WS_CHILD as isize)) | WS_POPUP as isize);
            SetLastError(0);
            SetParent(hwnd, null_mut());
            let error = GetLastError();
            if !GetParent(hwnd).is_null() {
                return Err(format!("小伴离开桌面背景层失败（Windows 错误 {error}）"));
            }
            if SetWindowPos(hwnd, HWND_TOPMOST, origin.x, origin.y,
                client.right - client.left, client.bottom - client.top,
                SWP_NOACTIVATE | SWP_FRAMECHANGED) == 0 {
                return Err(format!("恢复小伴悬浮位置失败（Windows 错误 {}）", GetLastError()));
            }
        }
        window.set_always_on_top(true).map_err(|e| e.to_string())?;
        // Showing a reparented Tao window can rebuild its styles from the old
        // child state; set the final native z-order after those framework updates.
        unsafe {
            if SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) == 0 {
                return Err(format!("恢复小伴置顶失败（Windows 错误 {}）", GetLastError()));
            }
        }
    }
    order_desktop_layers(window.app_handle())
}

static PET_DRAGGING: AtomicBool = AtomicBool::new(false);

/// A wallpaper pet is an Explorer child window, so the system's top-level
/// window drag operation cannot move it. Keep the cursor's initial grab point.
pub fn drag_companion(window: &tauri::WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
    let parent = unsafe { GetParent(hwnd) };
    if parent.is_null() { return Err("小伴不在桌面背景层".into()); }
    let mut origin = POINT::default();
    let mut cursor = POINT::default();
    if unsafe { ClientToScreen(hwnd, &mut origin) } == 0 || unsafe { GetCursorPos(&mut cursor) } == 0 {
        return Err("无法读取小伴拖动位置".into());
    }
    if PET_DRAGGING.swap(true, Ordering::SeqCst) { return Ok(()); }
    let hwnd = hwnd as usize;
    std::thread::spawn(move || {
        let hwnd = hwnd as HWND;
        let grab_x = cursor.x - origin.x;
        let grab_y = cursor.y - origin.y;
        while left_button_down() && unsafe { IsWindow(hwnd) } != 0 {
            let parent = unsafe { GetParent(hwnd) };
            if parent.is_null() || unsafe { GetCursorPos(&mut cursor) } == 0 { break; }
            let mut position = POINT { x: cursor.x - grab_x, y: cursor.y - grab_y };
            if unsafe { ScreenToClient(parent, &mut position) } == 0 { break; }
            if unsafe { SetWindowPos(hwnd, null_mut(), position.x, position.y, 0, 0,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE) } == 0 { break; }
            std::thread::sleep(Duration::from_millis(16));
        }
        PET_DRAGGING.store(false, Ordering::SeqCst);
    });
    Ok(())
}

/// All desktop layers are siblings below Explorer's icon view. Reapply this order
/// after a scene becomes visible, including when a newly downloaded wallpaper replaces one.
pub fn order_desktop_layers(app: &tauri::AppHandle) -> Result<(), String> {
    for label in ["media-a", "media-b", crate::effects::LABEL, "pet"] {
        let Some(window) = app.get_webview_window(label) else { continue; };
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
        unsafe {
            if GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CHILD as isize == 0 { continue; }
            if !["WorkerW", "Progman"].contains(&class_name(GetParent(hwnd)).as_str()) { continue; }
            if SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) == 0 {
                return Err(format!("排列桌面图层失败：{label}（Windows 错误 {}）", GetLastError()));
            }
        }
    }
    Ok(())
}

/// Rectangles of the visible application windows in physical pixels, front to back. Only geometry and the owning
/// process id are read; titles are never touched. Minimised and cloaked windows (other virtual desktops) are skipped.
pub fn window_rects() -> Vec<crate::desk_windows::Raw> {
    use crate::desk_windows::{Raw, Rect};
    use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
    struct Found {
        windows: Vec<Raw>,
        own: u32,
    }
    unsafe extern "system" fn each(hwnd: HWND, lparam: LPARAM) -> i32 {
        let found = &mut *(lparam as *mut Found);
        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return 1;
        }
        let mut cloaked = 0u32;
        DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED as u32, &mut cloaked as *mut u32 as *mut _, std::mem::size_of::<u32>() as u32);
        if cloaked != 0 {
            return 1;
        }
        // The extended frame is what the eye sees; GetWindowRect also counts the invisible resize border.
        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let framed = DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS as u32, &mut rect as *mut RECT as *mut _, std::mem::size_of::<RECT>() as u32) == 0;
        if !framed && GetWindowRect(hwnd, &mut rect) == 0 {
            return 1;
        }
        let mut process = 0u32;
        GetWindowThreadProcessId(hwnd, &mut process);
        let class = class_name(hwnd);
        let tool = (GetWindowLongW(hwnd, GWL_EXSTYLE) as u32) & WS_EX_TOOLWINDOW != 0;
        let shell = ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"].contains(&class.as_str());
        found.windows.push(Raw {
            rect: Rect::new(rect.left as f64, rect.top as f64, (rect.right - rect.left) as f64, (rect.bottom - rect.top) as f64),
            ours: process == found.own,
            ordinary: !tool && !shell,
        });
        1
    }
    let mut found = Found { windows: Vec::new(), own: std::process::id() };
    unsafe { EnumWindows(Some(each), &mut found as *mut Found as LPARAM) };
    found.windows
}

/// The pointer in physical pixels from the virtual screen's origin.
pub fn cursor_position() -> Option<(f64, f64)> {
    let mut point = POINT::default();
    (unsafe { GetCursorPos(&mut point) } != 0).then(|| (point.x as f64, point.y as f64))
}

/// Keeps a click on the desktop layer from taking keyboard focus away from the program the user is working in,
/// and out of the Alt-Tab list.
pub fn make_overlay(window: &tauri::WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetLastError(0);
        if SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) as isize) == 0 && GetLastError() != 0 {
            return Err(format!("无法设置桌面层窗口样式（{}）", GetLastError()));
        }
    }
    Ok(())
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
            // Keep a periodic sample for animated hit regions, but do not cross the
            // native/WebView boundary thirty times a second while the cursor is idle.
            if !moved && !pressed && !released && ticks % 8 != 0 {
                continue;
            }
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
                let mut outer = RECT::default();
                GetWindowRect(hwnd, &mut outer);
                serde_json::json!({ "hwnd": hwnd as usize, "parent": parent as usize,
                "root": GetAncestor(hwnd, GA_ROOT) as usize, "owner": GetWindow(hwnd, GW_OWNER) as usize,
                "parentClass": class_name(parent), "visible": IsWindowVisible(hwnd) != 0,
                "width": rect.right, "height": rect.bottom, "dpi": GetDpiForWindow(hwnd),
                "x": outer.left, "y": outer.top,
                "topmost": GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_TOPMOST as isize != 0,
                "isChild": GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CHILD as isize != 0 })
            })
    };
    let geometry = app.get_webview_window("wallpaper").and_then(|w| wallpaper_geometry(&w).ok());
    let mut desktop_order = Vec::new();
    let layers: Vec<_> = ["pet", crate::effects::LABEL, "media-a", "media-b"].into_iter()
        .filter_map(|label| app.get_webview_window(label).and_then(|w| w.hwnd().ok())
            .map(|native| (label, native.0 as HWND)))
        .collect();
    if let Some((_, hwnd)) = layers.iter().find(|(_, hwnd)| unsafe {
        GetWindowLongPtrW(*hwnd, GWL_STYLE) & WS_CHILD as isize != 0
    }) {
        unsafe {
            let mut sibling = GetTopWindow(GetParent(*hwnd));
            while !sibling.is_null() {
                if let Some((label, _)) = layers.iter().find(|(_, hwnd)| *hwnd == sibling) {
                    desktop_order.push(*label);
                }
                sibling = GetWindow(sibling, GW_HWNDNEXT);
            }
        }
    }
    serde_json::json!({"platform": "windows", "wallpaperSupported": true,
        "mode": super::settings(app).mode, "floating": describe("pet"), "wallpaper": describe("wallpaper"),
        "effects": describe(crate::effects::LABEL), "mediaA": describe("media-a"), "mediaB": describe("media-b"),
        "desktopOrder": desktop_order, "wallpaperGeometry":geometry})
}

#[derive(Clone, Copy)]
struct DesktopClick { x: i32, y: i32, time: u32, under: usize, released: bool }
static GAME_CLICKS: std::sync::OnceLock<std::sync::mpsc::SyncSender<DesktopClick>> = std::sync::OnceLock::new();
static GAME_CLICK_OVERFLOW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GAME_HIT_TEST_TIMEOUTS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static GAME_POINTER_DOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static GAME_POINTER_X: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static GAME_POINTER_Y: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
unsafe extern "system" fn game_mouse_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    use std::sync::atomic::Ordering;
    if code>=0 && wparam as u32==WM_MOUSEMOVE && GAME_POINTER_DOWN.load(Ordering::Relaxed) {
        let event=&*(lparam as *const MSLLHOOKSTRUCT);
        if (event.pt.x as i64-GAME_POINTER_X.load(Ordering::Relaxed) as i64).abs()>GetSystemMetrics(SM_CXDRAG) as i64/2 || (event.pt.y as i64-GAME_POINTER_Y.load(Ordering::Relaxed) as i64).abs()>GetSystemMetrics(SM_CYDRAG) as i64/2 {
            GAME_CLICK_OVERFLOW.store(true,Ordering::Relaxed);
        }
    }
    if code >= 0 && [WM_LBUTTONDOWN,WM_LBUTTONUP].contains(&(wparam as u32)) {
        let event = &*(lparam as *const MSLLHOOKSTRUCT);
        if event.flags & LLMHF_INJECTED == 0 {
            GAME_POINTER_DOWN.store(wparam as u32==WM_LBUTTONDOWN,Ordering::Relaxed);
            if wparam as u32==WM_LBUTTONDOWN { GAME_POINTER_X.store(event.pt.x,Ordering::Relaxed); GAME_POINTER_Y.store(event.pt.y,Ordering::Relaxed); }
            if let Some(queue) = GAME_CLICKS.get() {
                // Never block the hook: all Explorer IPC and window creation happen on the worker.
                let click=DesktopClick{x:event.pt.x,y:event.pt.y,time:event.time,under:WindowFromPoint(event.pt) as usize,released:wparam as u32==WM_LBUTTONUP};
                if queue.try_send(click).is_err() { GAME_CLICK_OVERFLOW.store(true,std::sync::atomic::Ordering::Relaxed); }
            }
        }
    }
    CallNextHookEx(null_mut(),code,wparam,lparam)
}
/// LVM_HITTEST has a pointer-bearing payload: Explorer is a separate process.
/// Allocate the fixed-layout, pointer-free LVHITTESTINFO in Explorer and fail closed on any error.
fn desktop_blank(click: DesktopClick) -> bool {
    use windows_sys::Win32::{Foundation::CloseHandle, System::{Memory::*, Threading::*, Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory}}};
    #[repr(C)] struct HitTest { point: POINT, flags: u32, item: i32, subitem: i32, group: i32 }
    unsafe {
        if GAME_HIT_TEST_TIMEOUTS.load(std::sync::atomic::Ordering::Relaxed)>=16 { return false; }
        let under=click.under as HWND;
        if under.is_null() || WindowFromPoint(POINT{x:click.x,y:click.y}) != under || class_name(under)!="SysListView32" { return false; }
        let defview=GetParent(under);
        if class_name(defview)!="SHELLDLL_DefView" || !["Progman","WorkerW"].contains(&class_name(GetAncestor(defview,GA_ROOT)).as_str()) { return false; }
        let mut pid=0; GetWindowThreadProcessId(under,&mut pid);
        let mut shell_pid=0; GetWindowThreadProcessId(GetShellWindow(),&mut shell_pid);
        if shell_pid==0 || pid!=shell_pid { return false; }
        let process=OpenProcess(PROCESS_VM_OPERATION|PROCESS_VM_READ|PROCESS_VM_WRITE,false as i32,pid);
        if process.is_null() { return false; }
        let remote=VirtualAllocEx(process,std::ptr::null(),std::mem::size_of::<HitTest>(),MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE);
        if remote.is_null() { CloseHandle(process); return false; }
        let mut info=HitTest{point:POINT{x:click.x,y:click.y},flags:0,item:-2,subitem:0,group:0};
        let mut bytes=0; let mut result=0;
        let written=ScreenToClient(under,&mut info.point)!=0
            && WriteProcessMemory(process,remote,&info as *const _ as _,std::mem::size_of::<HitTest>(),&mut bytes)!=0
            && bytes==std::mem::size_of::<HitTest>();
        let sent=written && SendMessageTimeoutW(under,0x1012,0,remote as isize,SMTO_ABORTIFHUNG|SMTO_BLOCK,100,&mut result)!=0;
        let blank=sent
            && ReadProcessMemory(process,remote,&mut info as *mut _ as _,std::mem::size_of::<HitTest>(),&mut bytes)!=0
            && bytes==std::mem::size_of::<HitTest>() && info.item==-1 && info.flags & 1!=0;
        if sent || !written { VirtualFreeEx(process,remote,0,MEM_RELEASE); }
        else {
            // A timed-out message could still be touching its payload. Keep these tiny buffers
            // until Explorer exits rather than leave it a dangling pointer; bound failures to 16.
            GAME_HIT_TEST_TIMEOUTS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
        }
        CloseHandle(process); blank
    }
}
fn over_companion(app:&tauri::AppHandle,click:DesktopClick)->bool{
    let config=crate::settings(app);
    if !config.companion_enabled{return false;}
    let contains=|rect:RECT|click.x>=rect.left&&click.x<rect.right&&click.y>=rect.top&&click.y<rect.bottom;
    for (label,window) in app.webview_windows(){
        if !crate::pets::is_pet(&label){continue;}
        if let Ok(pet)=window.hwnd(){let mut rect=RECT::default();let hwnd=pet.0 as HWND;
            if unsafe{IsWindowVisible(hwnd)!=0&&GetWindowRect(hwnd,&mut rect)!=0}&&contains(rect){return true;}
        }
    }
    false
}
pub fn start_desktop_game_entry(app: tauri::AppHandle) {
    use std::sync::atomic::Ordering;
    let (sender,receiver)=std::sync::mpsc::sync_channel::<DesktopClick>(64);
    if GAME_CLICKS.set(sender).is_err() { return; }
    let hook_app=app.clone();
    std::thread::spawn(move || unsafe {
        let hook=SetWindowsHookExW(WH_MOUSE_LL,Some(game_mouse_hook),null_mut(),0);
        let state=hook_app.state::<crate::games::GameState>();
        if hook.is_null() { *state.desktop_entry_error.lock().unwrap()=Some(format!("桌面双击入口初始化失败（{}），请使用玩一玩按钮",GetLastError())); return; }
        state.desktop_entry_ready.store(true,Ordering::Relaxed);
        let mut message=MSG::default();
        while GetMessageW(&mut message,null_mut(),0,0)>0 { TranslateMessage(&message); DispatchMessageW(&message); }
        UnhookWindowsHookEx(hook);
        state.desktop_entry_ready.store(false,Ordering::Relaxed);
    });
    std::thread::spawn(move || {
        let mut previous: Option<DesktopClick>=None;
        while let Ok(click)=receiver.recv() {
            if GAME_HIT_TEST_TIMEOUTS.load(Ordering::Relaxed)>=16 {
                let state=app.state::<crate::games::GameState>();
                if state.desktop_entry_ready.swap(false,Ordering::Relaxed) {
                    let error="Explorer 桌面判断连续超时，已停用双击入口，请使用玩一玩按钮";
                    *state.desktop_entry_error.lock().unwrap()=Some(error.into());let _=app.emit("desktop-error",error);
                }
            }
            if GAME_CLICK_OVERFLOW.swap(false,Ordering::Relaxed) { previous=None; }
            if !crate::settings(&app).game_desktop_double_click || crate::games::is_active(&app) || over_companion(&app,click) || !desktop_blank(click) { previous=None; continue; }
            if click.released {
                if let Some(mut p)=previous {
                    if p.under==click.under && (p.x as i64-click.x as i64).abs()<=unsafe{GetSystemMetrics(SM_CXDOUBLECLK)} as i64/2 && (p.y as i64-click.y as i64).abs()<=unsafe{GetSystemMetrics(SM_CYDOUBLECLK)} as i64/2 { p.released=true; previous=Some(p); } else { previous=None; }
                }
                continue;
            }
            let double=previous.is_some_and(|p| p.released && p.under==click.under
                && click.time.wrapping_sub(p.time)<=unsafe{GetDoubleClickTime()}
                && (click.x as i64-p.x as i64).abs()<=unsafe{GetSystemMetrics(SM_CXDOUBLECLK)} as i64 / 2
                && (click.y as i64-p.y as i64).abs()<=unsafe{GetSystemMetrics(SM_CYDOUBLECLK)} as i64 / 2);
            if double {
                previous=None; let handle=app.clone();
                tauri::async_runtime::spawn(async move { if let Err(error)=crate::games::open(&handle).await { let _=handle.emit("desktop-error",error); } });
            } else { previous=Some(click); }
        }
    });
}

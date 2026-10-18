//! AppKit desktop hosting. Every NSWindow/NSView operation runs on the main
//! thread. Cursor sampling reads public mouse state and window layers only;
//! it does not install event taps or request screen recording/accessibility.
use crate::coordinates::{content_point_to_css, is_desktop_level, scene_level, Levels};
use objc2::{rc::autoreleasepool, MainThreadMarker};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSApplication, NSColor, NSEvent, NSScreen, NSWindow, NSWindowCollectionBehavior,
};
use objc2_core_foundation::{CFArray, CFDictionary, CFNumber, CFRetained, CFString, CFType};
use objc2_core_graphics::{
    kCGWindowAlpha, kCGWindowBounds, kCGWindowLayer, kCGWindowOwnerPID, CGWindowLevelForKey, CGWindowLevelKey,
    CGWindowListCopyWindowInfo,
    CGWindowListOption,
};
use std::{sync::mpsc, time::Duration};
use tauri::{Emitter, Manager};

fn on_main<T: Send + 'static>(
    app: &tauri::AppHandle,
    operation: impl FnOnce(MainThreadMarker) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    if let Some(mtm) = MainThreadMarker::new() {
        return autoreleasepool(|_| operation(mtm));
    }
    let (sender, receiver) = mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let result =
            autoreleasepool(|_| operation(MainThreadMarker::new().expect("AppKit main thread")));
        let _ = sender.send(result);
    })
    .map_err(|e| e.to_string())?;
    receiver
        .recv_timeout(Duration::from_secs(5))
        .map_err(|e| format!("AppKit 窗口操作未完成：{e}"))?
}

/// The window level AppKit has for a window, for the smoke test.
pub fn window_level(window: &tauri::WebviewWindow) -> Result<i64, String> {
    let window = window.clone();
    let app = window.app_handle().clone();
    on_main(&app, move |_| Ok(native(&window)?.level() as i64))
}

fn native(window: &tauri::WebviewWindow) -> Result<&NSWindow, String> {
    // Caller holds the live Tauri window and invokes this only on the main
    // thread. The reference never leaves the enclosing AppKit operation.
    if MainThreadMarker::new().is_none() {
        return Err("AppKit 操作必须在主线程执行".into());
    }
    let pointer = window.ns_window().map_err(|e| e.to_string())?;
    if pointer.is_null() {
        return Err("没有找到 macOS 窗口".into());
    }
    Ok(unsafe { &*pointer.cast::<NSWindow>() })
}

/// Use AppKit points consistently, including negative coordinates and Retina screens.
pub fn widget_monitors(app:&tauri::AppHandle)->Result<Vec<crate::widget_geometry::MonitorArea>,String>{
    on_main(app,move|mtm|{
        let screens=NSScreen::screens(mtm);let primary=screens.firstObject().ok_or("没有找到显示器")?.frame();
        Ok(screens.iter().enumerate().map(|(index,screen)|{
            let frame=screen.frame();let name=screen.localizedName().to_string();
            let (x,y)=crate::widget_geometry::from_bottom_left(primary.size.height,frame.origin.x,frame.origin.y,frame.size.height);
            crate::widget_geometry::MonitorArea{id:format!("{}@{},{}",name,frame.origin.x,frame.origin.y),name,x,y,width:frame.size.width,height:frame.size.height,scale:1.0,primary:index==0}
        }).collect())
    })
}
pub fn widget_cursor(app:&tauri::AppHandle)->Result<(f64,f64),String>{
    on_main(app,move|mtm|{let height=NSScreen::screens(mtm).firstObject().ok_or("没有找到显示器")?.frame().size.height;let point=NSEvent::mouseLocation();Ok((point.x,height-point.y))})
}
pub fn interactive_frame(window: &tauri::WebviewWindow) -> Result<(f64,f64,f64,f64),String> {
    let window=window.clone();let app=window.app_handle().clone();
    on_main(&app,move|mtm|{
        let height=NSScreen::screens(mtm).firstObject().ok_or("没有找到显示器")?.frame().size.height;
        let f=native(&window)?.frame();
        Ok((f.origin.x,height-f.origin.y-f.size.height,f.size.width,f.size.height))
    })
}

pub fn set_interactive_layer(window: &tauri::WebviewWindow, layer: &str) -> Result<(),String> {
    let window=window.clone();let layer=layer.to_string();let app=window.app_handle().clone();
    on_main(&app,move|_|{
        let level=match layer.as_str(){
            "desktop"=>CGWindowLevelForKey(CGWindowLevelKey::DesktopIconWindowLevelKey) as isize+1,
            "topmost"=>CGWindowLevelForKey(CGWindowLevelKey::StatusWindowLevelKey) as isize,
            _=>0,
        };
        native(&window)?.setLevel(level);Ok(())
    })
}

pub fn place_widget(window:&tauri::WebviewWindow,x:f64,y:f64,width:f64,height:f64)->Result<(),String>{
    let window=window.clone();let app=window.app_handle().clone();
    on_main(&app,move|mtm|{
        let primary_height=NSScreen::screens(mtm).firstObject().ok_or("没有找到显示器")?.frame().size.height;
        let frame=objc2_foundation::NSRect::new(objc2_foundation::NSPoint::new(x,primary_height-y-height),objc2_foundation::NSSize::new(width,height));
        native(&window)?.setFrame_display(frame,true);Ok(())
    })
}

/// Rectangles of the windows on screen, front to back, in points from the primary display's top-left corner.
/// Only geometry and the owning process id are read; window titles are never touched.
pub fn window_rects() -> Vec<crate::desk_windows::Raw> {
    use crate::desk_windows::{Raw, Rect};
    let options = CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements;
    let Some(list) = CGWindowListCopyWindowInfo(options, 0) else { return Vec::new() };
    // The system hands back an immutable array of dictionaries keyed by CFString.
    let list: CFRetained<CFArray<CFDictionary<CFString, CFType>>> = unsafe { CFRetained::cast_unchecked(list) };
    let own = std::process::id() as i64;
    let number = |dictionary: &CFDictionary<CFString, CFType>, key: &CFString| {
        dictionary.get(key).and_then(|value| value.downcast::<CFNumber>().ok()).and_then(|n| n.as_f64())
    };
    let mut out = Vec::new();
    for index in 0..list.len() {
        let Some(window) = list.get(index) else { continue };
        let Some(bounds) = window.get(unsafe { kCGWindowBounds }).and_then(|value| value.downcast::<CFDictionary>().ok()) else { continue };
        let bounds: CFRetained<CFDictionary<CFString, CFType>> = unsafe { CFRetained::cast_unchecked(bounds) };
        let (Some(x), Some(y), Some(w), Some(h)) = (
            number(&bounds, &CFString::from_static_str("X")),
            number(&bounds, &CFString::from_static_str("Y")),
            number(&bounds, &CFString::from_static_str("Width")),
            number(&bounds, &CFString::from_static_str("Height")),
        ) else { continue };
        let layer = number(&window, unsafe { kCGWindowLayer }).unwrap_or(-1.0);
        let alpha = number(&window, unsafe { kCGWindowAlpha }).unwrap_or(1.0);
        let pid = number(&window, unsafe { kCGWindowOwnerPID }).unwrap_or(-1.0) as i64;
        out.push(Raw { rect: Rect::new(x, y, w, h), ours: pid == own, ordinary: layer == 0.0 && alpha > 0.0 });
    }
    out
}

pub fn configure_scene(
    window: &tauri::WebviewWindow,
    wallpaper: bool,
    place: bool,
) -> Result<(), String> {
    let window = window.clone();
    let app = window.app_handle().clone();
    on_main(&app, move |mtm| {
        let ns = native(&window)?;
        let desktop = CGWindowLevelForKey(CGWindowLevelKey::DesktopWindowLevelKey) as isize;
        // Above the "always on top" tier (floating, level 3) that most utilities and other apps' pinned windows use:
        // inside one level the window used last wins, so staying in that tier meant being covered after a click elsewhere.
        // The status level also sits over the Dock, and these windows are small and click-through where they are empty.
        let floating = CGWindowLevelForKey(CGWindowLevelKey::StatusWindowLevelKey) as isize;
        let icons = CGWindowLevelForKey(CGWindowLevelKey::DesktopIconWindowLevelKey) as isize;
        let levels = Levels { desktop: desktop as i64, icons: icons as i64, floating: floating as i64 };
        ns.setLevel(scene_level(window.label(), crate::widgets::is_widget(window.label()), wallpaper, crate::effects::LABEL, levels) as isize);
        let common = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle;
        ns.setCollectionBehavior(
            common
                | if wallpaper {
                    NSWindowCollectionBehavior::FullScreenNone
                } else {
                    NSWindowCollectionBehavior::FullScreenAuxiliary
                },
        );
        ns.setHidesOnDeactivate(false);
        ns.setCanHide(false);
        ns.setExcludedFromWindowsMenu(true);
        ns.setHasShadow(false);
        let click_through = wallpaper && window.label() != "pet" && !crate::widgets::is_widget(window.label());
        if click_through {
            ns.setIgnoresMouseEvents(true);
        }
        if !click_through || window.label() == crate::effects::LABEL {
            ns.setOpaque(false);
            ns.setBackgroundColor(Some(&NSColor::clearColor()));
        }
        if place {
            // First NSScreen is the menu-bar/primary screen; mainScreen instead
            // follows the focused control window and could pick another display.
            let screen = NSScreen::screens(mtm)
                .firstObject()
                .ok_or("没有找到 macOS 主显示器")?;
            let frame = if wallpaper {
                screen.frame()
            } else {
                let visible = screen.visibleFrame();
                let mut frame = ns.frame();
                frame.origin.x = visible.origin.x + visible.size.width - frame.size.width - 70.0;
                frame.origin.y = visible.origin.y + 100.0;
                frame
            };
            ns.setFrame_display(frame, true);
        }
        Ok(())
    })
}

pub fn left_button_down() -> bool {
    NSEvent::pressedMouseButtons() & 1 != 0
}

fn desktop_at_point(point: objc2_foundation::NSPoint, mtm: MainThreadMarker) -> bool {
    let number = NSWindow::windowNumberAtPoint_belowWindowWithWindowNumber(point, 0, mtm);
    if number <= 0 {
        return true;
    }
    let Some(descriptions) =
        CGWindowListCopyWindowInfo(CGWindowListOption::OptionIncludingWindow, number as u32)
    else {
        return false;
    };
    // Apple's CGWindowListCopyWindowInfo contract is an immutable array of
    // dictionaries keyed by CFString with CFType values. No window titles or
    // screen images are read or retained.
    let descriptions: CFRetained<CFArray<CFDictionary<CFString, CFType>>> =
        unsafe { CFRetained::cast_unchecked(descriptions) };
    let Some(description) = descriptions.get(0) else {
        return false;
    };
    let Some(layer) = description
        .get(unsafe { kCGWindowLayer })
        .and_then(|value| value.downcast::<CFNumber>().ok())
        .and_then(|number| number.as_i64())
    else {
        return false;
    };
    is_desktop_level(
        layer,
        CGWindowLevelForKey(CGWindowLevelKey::DesktopWindowLevelKey) as i64,
        CGWindowLevelForKey(CGWindowLevelKey::DesktopIconWindowLevelKey) as i64,
    )
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
        let mut previous = (f64::NAN, f64::NAN);
        let mut previous_down = false;
        loop {
            std::thread::sleep(Duration::from_millis(33));
            let handle = app.clone();
            let result = on_main(&app, move |mtm| {
                let point = NSEvent::mouseLocation();
                let down = left_button_down();
                let moved = (point.x, point.y) != previous;
                let pressed = down && !previous_down;
                let released = !down && previous_down;
                let desktop = pressed
                    && ["wallpaper", "media-a", "media-b"].iter().any(|label| handle.get_webview_window(label).is_some())
                    && desktop_at_point(point, mtm);
                for label in ["pet", "wallpaper", "media-a", "media-b"] {
                    let Some(window) = handle.get_webview_window(label) else {
                        continue;
                    };
                    // A window that is being created or closed has no AppKit window yet; skip it
                    // for this tick instead of failing the whole sample.
                    let Ok(ns) = native(&window) else {
                        continue;
                    };
                    let Some(view) = ns.contentView() else {
                        continue;
                    };
                    let local = view.convertPoint_fromView(ns.convertPointFromScreen(point), None);
                    let bounds = view.bounds();
                    let (x, y) = content_point_to_css(
                        local.x,
                        local.y,
                        bounds.origin.x,
                        bounds.origin.y,
                        bounds.size.height,
                        view.isFlipped(),
                    );
                    let sample = CursorSample {
                            x,
                            y,
                            moved,
                            pressed,
                            released,
                            desktop,
                        };
                    if crate::content::media_label(label) { crate::content::send_cursor(&window, &sample); }
                    else { let _ = window.emit("desktop-cursor", sample); }
                }
                Ok(((point.x, point.y), down))
            });
            match result {
                Ok((point, down)) => {
                    previous = point;
                    previous_down = down;
                }
                // A busy main thread (timeout) must not end the bridge: the pet would stay
                // click-through for the rest of the session. Once the event loop has exited,
                // run_on_main_thread fails without queuing anything and the process is ending.
                Err(_) => continue,
            }
        }
    });
}

pub fn diagnostics(app: &tauri::AppHandle) -> serde_json::Value {
    let handle = app.clone();
    on_main(app, move |mtm| {
        let desktop_level = CGWindowLevelForKey(CGWindowLevelKey::DesktopWindowLevelKey) as i64;
        let icon_level = CGWindowLevelForKey(CGWindowLevelKey::DesktopIconWindowLevelKey) as i64;
        let describe = |label| -> Result<Option<serde_json::Value>, String> {
            let Some(window) = handle.get_webview_window(label) else { return Ok(None); };
            let ns = native(&window)?;
            let frame = ns.frame();
            let screen = NSScreen::screens(mtm).firstObject().ok_or("没有找到 macOS 主显示器")?.frame();
            let size = ns.contentView().map(|view| view.bounds().size).unwrap_or(ns.frame().size);
            let level = ns.level() as i64;
            Ok(Some(serde_json::json!({"windowNumber": ns.windowNumber(), "visible": ns.isVisible(),
                "x": frame.origin.x, "y": frame.origin.y,
                "screenFrame": {"x": screen.origin.x, "y": screen.origin.y, "width": screen.size.width, "height": screen.size.height},
                "width": size.width, "height": size.height, "scaleFactor": ns.backingScaleFactor(),
                "level": level, "desktopLevel": desktop_level, "desktopIconLevel": icon_level,
                "isDesktopLayer": is_desktop_level(level, desktop_level, icon_level),
                "host": "AppKit 桌面层", "ignoresMouseEvents": ns.ignoresMouseEvents(),
                "opaque": ns.isOpaque(), "joinsAllSpaces": ns.collectionBehavior().contains(NSWindowCollectionBehavior::CanJoinAllSpaces)})))
        };
        Ok(serde_json::json!({"platform": "macos", "wallpaperSupported": true,
            "mode": super::settings(&handle).mode, "floating": describe("pet")?, "wallpaper": describe("wallpaper")?}))
    }).unwrap_or_else(|error| serde_json::json!({"platform":"macos", "error":error}))
}

fn control_state(app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
    let handle = app.clone();
    on_main(app, move |mtm| {
        let control = handle
            .get_webview_window("control")
            .ok_or("没有找到设置面板")?;
        let ns = native(&control)?;
        let wallpaper = handle
            .get_webview_window("pet")
            .ok_or("没有找到小伴窗口")?;
        let wall = native(&wallpaper)?;
        Ok(
            serde_json::json!({"visible":ns.isVisible(), "minimized":ns.isMiniaturized(),
            "focused":ns.isKeyWindow(), "applicationHidden":NSApplication::sharedApplication(mtm).isHidden(),
            "wallpaperWindowNumber":wall.windowNumber(), "wallpaperVisible":wall.isVisible(),
            "mode":super::settings(&handle).mode,
            "reopenCount":handle.state::<super::DesktopState>().reopen_count.load(std::sync::atomic::Ordering::Relaxed)}),
        )
    })
}

fn wait_control(
    app: &tauri::AppHandle,
    matches: impl Fn(&serde_json::Value) -> bool,
) -> Result<serde_json::Value, String> {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        let state = control_state(app)?;
        if matches(&state) {
            return Ok(state);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "Dock 重新打开未恢复设置面板：{}",
        control_state(app)?
    ))
}

/// Exercise the real LaunchServices reopen event, not a direct call to the
/// handler. The wallpaper remains present in all three panel states.
pub fn smoke_dock_reopen(app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let bundle = executable
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .filter(|path| path.extension().is_some_and(|extension| extension == "app"))
        .ok_or("Dock 测试需要从 .app 运行")?;
    let baseline = control_state(app)?;
    let mut cases = Vec::new();
    for kind in ["minimized", "hidden_window", "hidden_application"] {
        let before = control_state(app)?["reopenCount"].as_u64().unwrap_or(0);
        let window = app
            .get_webview_window("control")
            .ok_or("没有找到设置面板")?;
        match kind {
            "minimized" => window.minimize().map_err(|error| error.to_string())?,
            "hidden_window" => window.hide().map_err(|error| error.to_string())?,
            _ => app.hide().map_err(|error| error.to_string())?,
        }
        let hidden = wait_control(app, |state| match kind {
            "minimized" => state["minimized"] == true,
            "hidden_window" => state["visible"] == false,
            _ => state["applicationHidden"] == true,
        })?;
        let status = std::process::Command::new("/usr/bin/open")
            .arg("-a")
            .arg(bundle)
            .status()
            .map_err(|error| error.to_string())?;
        if !status.success() {
            return Err("LaunchServices 无法重新打开小伴".into());
        }
        let restored = wait_control(app, |state| {
            state["reopenCount"].as_u64().unwrap_or(0) > before
                && state["visible"] == true
                && state["minimized"] == false
                && state["focused"] == true
                && state["applicationHidden"] == false
        })?;
        if restored["wallpaperWindowNumber"] != baseline["wallpaperWindowNumber"]
            || restored["mode"] != "wallpaper"
            || restored["wallpaperVisible"] != true
        {
            return Err("Dock 唤起设置面板时改变了壁纸窗口或模式".into());
        }
        cases.push(serde_json::json!({"case":kind, "beforeReopen":hidden, "afterReopen":restored}));
    }
    Ok(serde_json::json!({"cases":cases, "wallpaperPreserved":true}))
}

/// Adapts the native window titlebar, frame controls, and background color to the chosen appearance.
pub fn apply_control_appearance(window: &tauri::WebviewWindow, appearance: &str) -> Result<(), String> {
    let window = window.clone();
    let app = window.app_handle().clone();
    let appearance = appearance.to_string();
    on_main(&app, move |_mtm| {
        let ns = native(&window)?;
        match appearance.as_str() {
            "dark" => {
                let name = unsafe { NSAppearanceNameDarkAqua };
                let dark = NSAppearance::appearanceNamed(name);
                ns.setAppearance(dark.as_deref());
                // Dark theme background matches --ui-sidebar-bg / --ui-app-bg (#161618)
                let bg = NSColor::colorWithSRGBRed_green_blue_alpha(0.086, 0.086, 0.094, 1.0);
                ns.setBackgroundColor(Some(&bg));
            }
            "light" => {
                let name = unsafe { NSAppearanceNameAqua };
                let light = NSAppearance::appearanceNamed(name);
                ns.setAppearance(light.as_deref());
                let bg = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
                ns.setBackgroundColor(Some(&bg));
            }
            _ => {
                ns.setAppearance(None);
                let bg = NSColor::windowBackgroundColor();
                ns.setBackgroundColor(Some(&bg));
            }
        }
        Ok(())
    })
}

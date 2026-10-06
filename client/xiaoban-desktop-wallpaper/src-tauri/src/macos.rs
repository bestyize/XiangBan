//! AppKit desktop hosting. Every NSWindow/NSView operation runs on the main
//! thread. Cursor sampling reads public mouse state and window layers only;
//! it does not install event taps or request screen recording/accessibility.
use crate::coordinates::{content_point_to_css, is_desktop_level};
use objc2::{rc::autoreleasepool, MainThreadMarker};
use objc2_app_kit::{NSColor, NSEvent, NSScreen, NSWindow, NSWindowCollectionBehavior};
use objc2_core_foundation::{CFArray, CFDictionary, CFNumber, CFRetained, CFString, CFType};
use objc2_core_graphics::{
    kCGWindowLayer, CGWindowLevelForKey, CGWindowLevelKey, CGWindowListCopyWindowInfo,
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
        let floating = CGWindowLevelForKey(CGWindowLevelKey::FloatingWindowLevelKey) as isize;
        ns.setLevel(if wallpaper { desktop + 1 } else { floating });
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
        if wallpaper {
            ns.setIgnoresMouseEvents(true);
        } else {
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
                    && handle.get_webview_window("wallpaper").is_some()
                    && desktop_at_point(point, mtm);
                for label in ["pet", "wallpaper"] {
                    let Some(window) = handle.get_webview_window(label) else {
                        continue;
                    };
                    let ns = native(&window)?;
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
                    let _ = window.emit(
                        "desktop-cursor",
                        CursorSample {
                            x,
                            y,
                            moved,
                            pressed,
                            released,
                            desktop,
                        },
                    );
                }
                Ok(((point.x, point.y), down))
            });
            match result {
                Ok((point, down)) => {
                    previous = point;
                    previous_down = down;
                }
                Err(_) => break, // The event loop has exited; do not keep queuing work.
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

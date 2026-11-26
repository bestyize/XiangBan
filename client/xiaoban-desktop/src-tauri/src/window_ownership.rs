//! Non-modal ownership for existing top-level windows.
use tauri::WebviewWindow;

pub(crate) fn set_owner(window: &WebviewWindow, owner: &WebviewWindow) -> Result<(), String> {
    if window.label() == owner.label() {
        return Ok(());
    }
    platform::set_owner(window, owner)
}

/// Keep open editors and their drafts when the window that opened them closes.
pub(crate) fn preserve_children(window: &WebviewWindow) {
    if let Err(error) = platform::preserve_children(window) {
        eprintln!("无法保留 {} 的子窗口：{error}", window.label());
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use crate::macos::{native, on_main};
    use objc2_app_kit::{NSWindow, NSWindowOrderingMode};
    use tauri::Manager;

    fn attach(window: &NSWindow, owner: Option<&NSWindow>) {
        let previous = window.parentWindow();
        if previous.as_deref().map(std::ptr::from_ref) == owner.map(std::ptr::from_ref) {
            return;
        }
        if let Some(previous) = previous {
            previous.removeChildWindow(window);
        }
        if let Some(owner) = owner {
            // The callers only attach windows along the console's opening paths.
            unsafe { owner.addChildWindow_ordered(window, NSWindowOrderingMode::Above) };
        }
    }

    pub(super) fn set_owner(window: &WebviewWindow, owner: &WebviewWindow) -> Result<(), String> {
        let (window, owner) = (window.clone(), owner.clone());
        let app = window.app_handle().clone();
        on_main(&app, move |_| {
            attach(native(&window)?, Some(native(&owner)?));
            Ok(())
        })
    }

    pub(super) fn preserve_children(window: &WebviewWindow) -> Result<(), String> {
        let window = window.clone();
        let app = window.app_handle().clone();
        on_main(&app, move |_| {
            let native = native(&window)?;
            let owner = native.parentWindow();
            if let Some(children) = native.childWindows() {
                for child in children.iter() {
                    attach(&child, owner.as_deref());
                }
            }
            Ok(())
        })
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::sync::mpsc;
    use tauri::Manager;
    use windows_sys::Win32::{
        Foundation::{GetLastError, SetLastError, HWND},
        UI::WindowsAndMessaging::{GetWindow, SetWindowLongPtrW, GWLP_HWNDPARENT, GW_OWNER},
    };

    fn hwnd(window: &WebviewWindow) -> Result<HWND, String> {
        window
            .hwnd()
            .map(|handle| handle.0)
            .map_err(|error| error.to_string())
    }

    fn attach(window: HWND, owner: HWND) -> Result<(), String> {
        // GWLP_HWNDPARENT changes the owner of a top-level window.
        unsafe {
            if GetWindow(window, GW_OWNER) != owner {
                SetLastError(0);
                if SetWindowLongPtrW(window, GWLP_HWNDPARENT, owner as isize) == 0
                    && GetLastError() != 0
                {
                    return Err(std::io::Error::last_os_error().to_string());
                }
            }
        }
        Ok(())
    }

    pub(super) fn set_owner(window: &WebviewWindow, owner: &WebviewWindow) -> Result<(), String> {
        let (target, owner) = (window.clone(), owner.clone());
        let (sender, receiver) = mpsc::sync_channel(1);
        window
            .run_on_main_thread(move || {
                let result = (|| attach(hwnd(&target)?, hwnd(&owner)?))();
                let _ = sender.send(result);
            })
            .map_err(|error| error.to_string())?;
        receiver.recv().map_err(|error| error.to_string())?
    }

    pub(super) fn preserve_children(window: &WebviewWindow) -> Result<(), String> {
        // CloseRequested runs on the event-loop thread, before Windows destroys owned windows.
        let parent = hwnd(window)?;
        let owner = unsafe { GetWindow(parent, GW_OWNER) };
        for child in window.app_handle().webview_windows().values() {
            let Ok(child) = hwnd(child) else { continue };
            if unsafe { GetWindow(child, GW_OWNER) } == parent {
                attach(child, owner)?;
            }
        }
        Ok(())
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    use super::*;

    pub(super) fn set_owner(_: &WebviewWindow, _: &WebviewWindow) -> Result<(), String> {
        Ok(())
    }
    pub(super) fn preserve_children(_: &WebviewWindow) -> Result<(), String> {
        Ok(())
    }
}

use objc2::MainThreadMarker;
use objc2_app_kit::{NSScreen, NSWorkspace};
use objc2_foundation::{NSDictionary, NSString, NSURL};
use std::path::Path;

pub(super) fn apply_desktop_picture(mtm: MainThreadMarker, path: &Path, monitor: &str) -> Result<(), String> {
    let screens = NSScreen::screens(mtm);
    let primary = screens.firstObject().ok_or("没有找到显示器")?;
    let selected = if monitor == "*" {
        screens.iter().collect::<Vec<_>>()
    } else {
        let screen = screens.iter().find(|screen| {
            let frame = screen.frame();
            crate::macos::screen_id(&screen.localizedName().to_string(), frame.origin.x, frame.origin.y) == monitor
        });
        vec![screen.unwrap_or(primary)]
    };

    let path = path.to_str().ok_or("无法读取桌面底图文件路径")?;
    let url = NSURL::fileURLWithPath(&NSString::from_str(path));
    let workspace = NSWorkspace::sharedWorkspace();
    let previous = selected.iter().map(|screen| {
        let url = workspace.desktopImageURLForScreen(screen);
        let options = workspace.desktopImageOptionsForScreen(screen).unwrap_or_else(NSDictionary::new);
        (url, options)
    }).collect::<Vec<_>>();

    for (index, (screen, (_, options))) in selected.iter().zip(&previous).enumerate() {
        // The options came from AppKit for this same screen on the main thread.
        if let Err(error) = unsafe { workspace.setDesktopImageURL_forScreen_options_error(&url, screen, options) } {
            for (screen, (old_url, old_options)) in selected.iter().zip(&previous).take(index) {
                if let Some(old_url) = old_url {
                    let _ = unsafe { workspace.setDesktopImageURL_forScreen_options_error(old_url, screen, old_options) };
                }
            }
            return Err(format!("系统桌面底图设置失败：{error}"));
        }
    }
    Ok(())
}

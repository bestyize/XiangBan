//! Per-user startup registration; test entries have separate names and are cleaned up.
#[cfg(any(windows, target_os = "linux"))]
use tauri::Manager;
#[cfg(target_os = "macos")]
use tauri_plugin_autostart::ManagerExt;

pub fn enabled(app: &tauri::AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    { linux::enabled(&linux::entry(&app.state::<crate::DesktopState>().startup_name)?) }
    #[cfg(windows)]
    {
        win::enabled(&app.state::<crate::DesktopState>().startup_name)
    }
    #[cfg(target_os = "macos")]
    {
        app.autolaunch().is_enabled().map_err(|e| e.to_string())
    }
}

pub fn set(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    { linux::set(&linux::entry(&app.state::<crate::DesktopState>().startup_name)?, enabled) }
    #[cfg(windows)]
    {
        let name = &app.state::<crate::DesktopState>().startup_name;
        if enabled {
            win::enable(name)
        } else {
            win::disable(name)
        }
    }
    #[cfg(target_os = "macos")]
    {
        let manager = app.autolaunch();
        if enabled {
            manager.enable()
        } else {
            manager.disable()
        }
        .map_err(|e| e.to_string())
    }
}

// Refresh moved portable binaries without resetting a system-disabled startup item.
pub fn refresh(app: &tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    { let _ = app; Ok(()) } // The installed desktop entry uses the executable on PATH.
    #[cfg(windows)]
    {
        win::refresh(&app.state::<crate::DesktopState>().startup_name)
    }
    #[cfg(target_os = "macos")]
    {
        app.autolaunch().enable().map_err(|e| e.to_string())
    }
}

pub struct TestEntryGuard(pub Option<String>);
impl Drop for TestEntryGuard {
    fn drop(&mut self) {
        if let Some(name) = &self.0 {
            let _ = cleanup_test_entry(name);
        }
    }
}

pub fn cleanup_test_entry(name: &str) -> Result<(), String> {
    if !name
        .strip_prefix("XiaoBanStartupSmoke-")
        .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err("不是本应用的隔离测试启动项".into());
    }
    #[cfg(target_os = "linux")]
    { linux::set(&linux::entry(name)?, false) }
    #[cfg(windows)]
    {
        win::disable(name)
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .ok_or("没有找到用户目录")?;
        if !home.is_absolute() {
            return Err("用户目录不是绝对路径".into());
        }
        let path = home
            .join("Library/LaunchAgents")
            .join(format!("{name}.plist"));
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::{fs, io::{self, Write}, path::{Path, PathBuf}};

    pub(super) fn entry(name: &str) -> Result<PathBuf, String> {
        dirs::config_dir().map(|dir| dir.join("autostart").join(format!("{name}.desktop")))
            .ok_or_else(|| "没有找到 XDG 用户配置目录".into())
    }

    pub(super) fn enabled(path: &Path) -> Result<bool, String> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.to_string()),
        };
        let mut desktop_entry = false;
        let mut found = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                desktop_entry = line == "[Desktop Entry]";
                found |= desktop_entry;
            } else if desktop_entry && matches!(line, "Hidden=true" | "X-GNOME-Autostart-enabled=false") {
                return Ok(false);
            }
        }
        Ok(found)
    }

    pub(super) fn set(path: &Path, enabled: bool) -> Result<(), String> {
        if !enabled {
            return match fs::remove_file(path) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
                result => result.map_err(|e| e.to_string()),
            };
        }
        let dir = path.parent().ok_or("启动项目录无效")?;
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
        file.write_all(b"[Desktop Entry]\nType=Application\nName=XiaoBan Desktop\nTryExec=xiaoban-desktop\nExec=xiaoban-desktop --autostart\n").map_err(|e| e.to_string())?;
        file.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }

    #[test]
    fn registration_respects_session_disabling_and_can_be_removed_twice() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("autostart/XiaoBanDesktop.desktop");
        assert!(!enabled(&path).unwrap());
        set(&path, true).unwrap();
        assert!(enabled(&path).unwrap());
        let registered = fs::read_to_string(&path).unwrap();
        assert!(registered.contains("Exec=xiaoban-desktop --autostart"));
        fs::write(&path, format!("{registered}Hidden=true\n")).unwrap();
        assert!(!enabled(&path).unwrap());
        set(&path, false).unwrap();
        set(&path, false).unwrap();
        assert!(!path.exists());
    }
}

#[cfg(windows)]
mod win {
    use windows_registry::{Type, CURRENT_USER};
    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

    fn optional<T>(result: windows_registry::Result<T>) -> Result<Option<T>, String> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) if matches!(error.code().0 as u32, 0x80070002 | 0x80070003) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
    pub fn enabled(name: &str) -> Result<bool, String> {
        let registered =
            optional(CURRENT_USER.open(RUN).and_then(|key| key.get_string(name)))?.is_some();
        if !registered {
            return Ok(false);
        }
        let approved = optional(
            CURRENT_USER
                .open(APPROVED)
                .and_then(|key| key.get_value(name)),
        )?;
        Ok(approved
            .as_ref()
            .is_none_or(|value| !matches!(value.first(), Some(3 | 7))))
    }
    pub fn refresh(name: &str) -> Result<(), String> {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        // Quote the executable, including paths with spaces and non-ASCII characters.
        let command = format!("\"{}\" --autostart", executable.display());
        CURRENT_USER
            .create(RUN)
            .and_then(|key| key.set_string(name, &command))
            .map_err(|e| e.to_string())
    }
    pub fn enable(name: &str) -> Result<(), String> {
        refresh(name)?;
        CURRENT_USER
            .create(APPROVED)
            .and_then(|key| key.set_bytes(name, Type::Bytes, &[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]))
            .map_err(|e| e.to_string())
    }
    pub fn disable(name: &str) -> Result<(), String> {
        optional(
            CURRENT_USER
                .options()
                .write()
                .open(RUN)
                .and_then(|key| key.remove_value(name)),
        )?;
        optional(
            CURRENT_USER
                .options()
                .write()
                .open(APPROVED)
                .and_then(|key| key.remove_value(name)),
        )?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        struct Cleanup(String);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = disable(&self.0);
            }
        }
        #[test]
        fn current_user_registration_respects_system_disable_and_cleans_up() {
            let name = format!("XiaoBanStartupTest-{}", std::process::id());
            let _cleanup = Cleanup(name.clone());
            assert!(!enabled(&name).unwrap());
            enable(&name).unwrap();
            assert!(enabled(&name).unwrap());
            let value = CURRENT_USER.open(RUN).unwrap().get_string(&name).unwrap();
            assert!(value.starts_with('"') && value.ends_with("\" --autostart"));
            CURRENT_USER
                .create(APPROVED)
                .unwrap()
                .set_bytes(&name, Type::Bytes, &[3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                .unwrap();
            assert!(!enabled(&name).unwrap());
            refresh(&name).unwrap();
            assert!(
                !enabled(&name).unwrap(),
                "refresh must not override Task Manager"
            );
            enable(&name).unwrap();
            assert!(enabled(&name).unwrap());
            disable(&name).unwrap();
            assert!(!enabled(&name).unwrap());
        }
    }
}

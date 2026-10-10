//! Owned user data. Formal, development and smoke-test profiles never share files.
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tauri::Manager;

pub fn location(isolated: bool) -> Result<PathBuf, String> {
    if isolated {
        return Ok(std::env::var_os("XIAOBAN_TEST_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::temp_dir().join(format!("xiaoban-smoke-{}", std::process::id()))
            }));
    }
    let root = dirs::home_dir()
        .ok_or("无法找到当前用户目录")?
        .join(".xiaobanwallpaper");
    Ok(if cfg!(debug_assertions) {
        root.join("development")
    } else {
        root
    })
}

pub fn root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.state::<crate::DesktopState>()
        .data_root
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "用户数据目录未就绪".into())
}

pub fn webview(root: &Path) -> PathBuf {
    root.join("cache/webview")
}

#[tauri::command]
pub fn open_data_directory(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" {
        return Err("只能从设置面板打开数据目录".into());
    }
    let directory = root(window.app_handle())?;
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    show_in_file_manager(&directory)
}

/// Opens a directory in Explorer or Finder.
pub fn show_in_file_manager(directory: &Path) -> Result<(), String> {
    #[cfg(windows)]
    unsafe {
        let operation: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
        let value: Vec<u16> = directory
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let result = windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            value.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
        if result as isize <= 32 {
            return Err("无法打开目录".into());
        }
    }
    #[cfg(target_os = "macos")]
    if !std::process::Command::new("/usr/bin/open")
        .arg(directory)
        .status()
        .map_err(|e| e.to_string())?
        .success()
    {
        return Err("无法打开目录".into());
    }
    let _ = directory;
    Ok(())
}

// Each copy commits through a temporary file. On an interrupted migration,
// retry only fills missing destinations and never replaces newer user data.
fn copy_missing(source: &Path, target: &Path) -> io::Result<()> {
    if target.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(source)?;
    if !metadata.is_file() {
        return Err(io::Error::other("迁移源不是普通文件"));
    }
    let parent = target
        .parent()
        .ok_or_else(|| io::Error::other("无效迁移路径"))?;
    fs::create_dir_all(parent)?;
    let temporary = target.with_extension(format!("migration-{}.tmp", std::process::id()));
    let result = (|| {
        fs::copy(source, &temporary)?;
        fs::OpenOptions::new()
            .write(true)
            .open(&temporary)?
            .sync_all()?;
        fs::rename(&temporary, target)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub fn initialize(root: &Path, legacy: Option<&Path>) -> io::Result<()> {
    for directory in [
        "config",
        "wallpapers/imports",
        "wallpapers/cache",
        "cache/webview",
    ] {
        fs::create_dir_all(root.join(directory))?;
    }
    let preferences = root.join("config/preferences.json");
    if preferences.exists() {
        return Ok(());
    }
    let Some(legacy) = legacy else {
        return Ok(());
    };
    let old_preferences = legacy.join("preferences.json");
    if !old_preferences.exists() {
        return Ok(());
    }
    // Refuse a broken old configuration instead of silently enabling startup.
    crate::preferences::load(&old_preferences)?;
    let backup = root.join("backups/legacy-v1");
    copy_missing(&old_preferences, &backup.join("preferences.json"))?;
    for name in ["catalog.json", "active.json"] {
        let source = legacy.join("wallpapers").join(name);
        if source.exists() {
            copy_missing(&source, &backup.join(name))?;
            copy_missing(&source, &root.join("wallpapers").join(name))?;
        }
    }
    let cache = legacy.join("wallpapers/cache");
    if cache.exists() {
        for file in fs::read_dir(cache)? {
            let file = file?;
            let name = file.file_name();
            if name.to_str().is_some_and(crate::library::hash_name) {
                copy_missing(&file.path(), &root.join("wallpapers/cache").join(name))?;
            }
        }
    }
    crate::library::save_json(
        &backup.join("migration.json"),
        &serde_json::json!({
            "schemaVersion":1,"source":legacy,"originalsPreserved":true
        }),
    )
    .map_err(io::Error::other)?;
    // Preferences is the completion marker; all active resources must arrive first.
    copy_missing(&old_preferences, &preferences)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sandbox(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("xiaoban-storage-{name}-{}", std::process::id()))
    }
    #[test]
    fn migration_preserves_settings_cache_and_originals() {
        let base = sandbox("migrate");
        let old = base.join("old");
        let new = base.join("new");
        fs::create_dir_all(old.join("wallpapers/cache")).unwrap();
        let mut prefs = crate::preferences::Preferences::default();
        prefs.autostart_initialized = true;
        prefs.settings.autostart = false;
        prefs.settings.mode = "wallpaper".into();
        crate::preferences::save(&old.join("preferences.json"), &prefs).unwrap();
        fs::write(old.join("wallpapers/active.json"), "active fixture").unwrap();
        let hash = "a".repeat(64);
        fs::write(old.join("wallpapers/cache").join(&hash), "media fixture").unwrap();
        fs::write(old.join("wallpapers/cache/incomplete.part"), "partial").unwrap();
        initialize(&new, Some(&old)).unwrap();
        let restored = crate::preferences::load(&new.join("config/preferences.json")).unwrap();
        assert_eq!(restored.settings, prefs.settings);
        assert!(restored.autostart_initialized);
        assert_eq!(
            fs::read(new.join("wallpapers/cache").join(&hash)).unwrap(),
            b"media fixture"
        );
        assert!(old.join("preferences.json").exists());
        assert!(new.join("backups/legacy-v1/preferences.json").exists());
        assert!(!new.join("wallpapers/cache/incomplete.part").exists());
        prefs.settings.theme = "dawn".into();
        crate::preferences::save(&new.join("config/preferences.json"), &prefs).unwrap();
        initialize(&new, Some(&old)).unwrap();
        assert_eq!(
            crate::preferences::load(&new.join("config/preferences.json"))
                .unwrap()
                .settings
                .theme,
            "dawn"
        );
        fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn interrupted_migration_retries_before_committing_preferences() {
        let base = sandbox("retry");
        let old = base.join("old");
        let new = base.join("new");
        let hash = "b".repeat(64);
        let source = old.join("wallpapers/cache").join(&hash);
        fs::create_dir_all(&source).unwrap();
        crate::preferences::save(&old.join("preferences.json"), &Default::default()).unwrap();
        assert!(initialize(&new, Some(&old)).is_err());
        assert!(!new.join("config/preferences.json").exists());
        fs::remove_dir(&source).unwrap();
        fs::write(source, "complete").unwrap();
        initialize(&new, Some(&old)).unwrap();
        assert!(new.join("config/preferences.json").exists());
        assert_eq!(
            fs::read(new.join("wallpapers/cache").join(hash)).unwrap(),
            b"complete"
        );
        fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn broken_legacy_preferences_do_not_commit_migration() {
        let base = sandbox("invalid");
        let old = base.join("old");
        let new = base.join("new");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("preferences.json"), "invalid").unwrap();
        assert!(initialize(&new, Some(&old)).is_err());
        assert!(!new.join("config/preferences.json").exists());
        fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn profiles_without_legacy_have_an_owned_layout() {
        let base = sandbox("fresh");
        initialize(&base, None).unwrap();
        assert!(base.join("wallpapers/imports").is_dir());
        assert!(webview(&base).is_dir());
        assert!(!base.join("backups").exists());
        fs::remove_dir_all(base).unwrap();
    }
}

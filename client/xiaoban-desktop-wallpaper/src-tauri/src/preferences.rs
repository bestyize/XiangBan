use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub const EXPRESSIONS: &[&str] = &[
    "idle",
    "happy",
    "excited",
    "laughing",
    "proud",
    "curious",
    "surprised",
    "shy",
    "love",
    "sad",
    "crying",
    "annoyed",
    "angry",
    "sleepy",
    "sleeping",
    "thinking",
    "speaking",
    "listening",
    "waiting",
    "confused",
    "greeting",
    "worried",
    "charging",
    "offline",
    "embarrassed",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub mode: String,
    pub expression: String,
    pub size: f64,
    pub theme: String,
    pub paused: bool,
    pub fps: u32,
    pub autostart: bool,
    pub wallpaper_id: String,
    pub wallpaper_options: crate::library::Options,
    pub catalog_url: String,
    pub update_url: String,
    pub website_url: String,
    pub automatic_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: "floating".into(),
            expression: "idle".into(),
            size: 260.0,
            theme: "night".into(),
            paused: false,
            fps: 60,
            autostart: true,
            wallpaper_id: "companion".into(),
            wallpaper_options: Default::default(),
            catalog_url: String::new(),
            update_url: format!("{}/api/v1/desktop/updates", crate::website::DEFAULT_SITE),
            website_url: crate::website::DEFAULT_SITE.into(),
            automatic_updates: true,
        }
    }
}

impl Settings {
    pub fn normalize(&mut self) {
        let defaults = Self::default();
        if !["floating", "wallpaper"].contains(&self.mode.as_str()) {
            self.mode = defaults.mode;
        }
        if !EXPRESSIONS.contains(&self.expression.as_str()) {
            self.expression = defaults.expression;
        }
        if !self.size.is_finite() || !(160.0..=420.0).contains(&self.size) {
            self.size = defaults.size;
        }
        if !["night", "dawn"].contains(&self.theme.as_str()) {
            self.theme = defaults.theme;
        }
        if ![30, 60].contains(&self.fps) {
            self.fps = defaults.fps;
        }
        if crate::website::site_url(&self.website_url).is_err() {
            self.website_url = crate::website::DEFAULT_SITE.into();
        }
        if self.update_url.is_empty() {
            self.update_url = format!("{}/api/v1/desktop/updates", self.website_url);
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub settings: Settings,
    // Default-on is applied once. External OS disabling and explicit user disabling win later.
    pub autostart_initialized: bool,
}

pub fn load(path: &Path) -> io::Result<Preferences> {
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Preferences::default()),
        Err(error) => return Err(error),
    };
    let mut value: Preferences = serde_json::from_slice(&data).map_err(io::Error::other)?;
    value.settings.normalize();
    Ok(value)
}

pub fn save(path: &Path, value: &Preferences) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("设置文件没有父目录"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!("preferences-{}.tmp", std::process::id()));
    let result = (|| {
        let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
        let mut file = fs::File::create(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_launch_defaults_to_autostart() {
        let value = Preferences::default();
        assert!(value.settings.autostart);
        assert!(!value.autostart_initialized);
    }
    #[test]
    fn older_preferences_receive_new_defaults_without_resetting_scene() {
        let value: Preferences =
            serde_json::from_str(r#"{"settings":{"mode":"wallpaper","theme":"dawn","fps":30}}"#)
                .unwrap();
        assert_eq!(value.settings.mode, "wallpaper");
        assert!(value.settings.autostart);
    }
    #[test]
    fn invalid_scene_values_are_normalized_but_disabled_autostart_is_preserved() {
        let mut value = Settings {
            mode: "invalid".into(),
            size: f64::NAN,
            fps: 0,
            autostart: false,
            ..Default::default()
        };
        value.normalize();
        assert_eq!(value.mode, "floating");
        assert_eq!(value.size, 260.0);
        assert_eq!(value.fps, 60);
        assert!(!value.autostart);
    }
    #[test]
    fn saves_replace_old_preferences_and_survive_restart() {
        let directory =
            std::env::temp_dir().join(format!("xiaoban-prefs-test-{}", std::process::id()));
        let file = directory.join("preferences.json");
        let mut value = Preferences::default();
        save(&file, &value).unwrap();
        value.autostart_initialized = true;
        value.settings.autostart = false;
        value.settings.mode = "wallpaper".into();
        save(&file, &value).unwrap();
        let reloaded = load(&file).unwrap();
        assert!(reloaded.autostart_initialized);
        assert_eq!(reloaded.settings, value.settings);
        fs::remove_dir_all(directory).unwrap();
    }
}

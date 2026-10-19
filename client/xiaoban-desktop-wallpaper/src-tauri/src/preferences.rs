use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub const SKINS: &[&str] = &["pot", "pot-white-flower"];
/// The console's colour scheme. It lives here, not only in the console's page storage, because the other windows (the
/// floating assistant) cannot read that storage and still have to match.
pub const APPEARANCES: &[&str] = &["system", "light", "dark"];
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
    pub appearance: String,
    /// The host has been given a choice (by the user, or carried over from before the host kept one). Until then the
    /// console's own saved choice is the one to keep.
    pub appearance_kept: bool,
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
    pub game_desktop_double_click: bool,
    pub companion_enabled: bool,
    /// The companion reacts to the desktop effect (and later, to local presence) with expressions only.
    pub companion_links: bool,
    /// Official shell of the companion; an unknown name falls back to the pot.
    pub companion_skin: String,
    /// The floating assistant: a capsule that a global shortcut brings up for a quick spoken or typed question.
    pub assistant_enabled: bool,
    pub assistant_shortcut: String,
    /// A small ball stays on screen; clicking it opens the capsule. Without it the capsule only appears on the shortcut.
    pub assistant_ball: bool,
    /// Where the ball was left, relative to its default spot on the screen it is on, in logical pixels.
    pub assistant_ball_dx: f64,
    pub assistant_ball_dy: f64,
    /// What the capsule does when it opens: `voice` starts listening, `type` waits for typing.
    pub assistant_open: String,
    /// When answers are read aloud: `voice` for spoken questions only, `always`, or `never`.
    pub assistant_read: String,
    /// The one-time "press X anytime" hint has been shown.
    pub assistant_hint_seen: bool,
    /// The particle effect over the desktop, whatever wallpaper (or plain system desktop) is behind it.
    pub effects: crate::effect_choice::EffectsChoice,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: "floating".into(),
            appearance: "system".into(),
            appearance_kept: false,
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
            game_desktop_double_click: false,
            companion_enabled: true,
            companion_links: true,
            companion_skin: "pot".into(),
            assistant_enabled: true,
            assistant_shortcut: crate::float_assistant::DEFAULT_SHORTCUT.into(),
            assistant_ball: true,
            assistant_ball_dx: 0.0,
            assistant_ball_dy: 0.0,
            assistant_open: "voice".into(),
            assistant_read: "voice".into(),
            assistant_hint_seen: false,
            effects: Default::default(),
        }
    }
}

impl Settings {
    pub fn normalize(&mut self) {
        let defaults = Self::default();
        if !["floating", "wallpaper"].contains(&self.mode.as_str()) {
            self.mode = defaults.mode;
        }
        if !APPEARANCES.contains(&self.appearance.as_str()) {
            self.appearance = defaults.appearance;
        }
        if !SKINS.contains(&self.companion_skin.as_str()) {
            self.companion_skin = defaults.companion_skin;
        }
        if !crate::float_assistant::SHORTCUTS.contains(&self.assistant_shortcut.as_str()) {
            self.assistant_shortcut = defaults.assistant_shortcut;
        }
        if !crate::float_assistant::OPEN_MODES.contains(&self.assistant_open.as_str()) {
            self.assistant_open = defaults.assistant_open;
        }
        if !crate::float_assistant::READ_MODES.contains(&self.assistant_read.as_str()) {
            self.assistant_read = defaults.assistant_read;
        }
        // A saved spot is only a hint: anything absurd (or not a number) means the default spot.
        if ![self.assistant_ball_dx, self.assistant_ball_dy].iter().all(|v| v.is_finite() && v.abs() <= 20000.0) {
            self.assistant_ball_dx = 0.0;
            self.assistant_ball_dy = 0.0;
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
        // The website and its update service are built into the app; addresses saved by older
        // versions, which let users change them, are reset on load.
        self.website_url = defaults.website_url;
        self.update_url = defaults.update_url;
        // Effects used to be saved with each wallpaper. The choice that was in use becomes the desktop-wide
        // one, once; the old copy is emptied here and never written again.
        let legacy = std::mem::take(&mut self.wallpaper_options.effects);
        if self.effects == Default::default() && !legacy.is_null() {
            self.effects = crate::effect_choice::EffectsChoice::from_value(&legacy);
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
        assert!(!value.settings.game_desktop_double_click);
        assert!(value.settings.companion_enabled);
        assert!(value.settings.companion_links);
        assert_eq!(value.settings.effects, Default::default());
    }
    #[test]
    fn companion_links_default_on_and_an_explicit_off_survives_reload() {
        assert!(Settings::default().companion_links);
        let mut value: Preferences = serde_json::from_str(r#"{"settings":{"companionLinks":false}}"#).unwrap();
        value.settings.normalize();
        assert!(!value.settings.companion_links);
        let saved = serde_json::to_string(&value).unwrap();
        assert!(saved.contains(r#""companionLinks":false"#), "{saved}");
        let again: Preferences = serde_json::from_str(&saved).unwrap();
        assert!(!again.settings.companion_links);
    }
    #[test]
    fn the_shell_is_the_pot_and_every_retired_or_unknown_choice_falls_back_to_it() {
        assert_eq!(Settings::default().companion_skin, "pot");
        let mut older: Preferences = serde_json::from_str(r#"{"settings":{"mode":"wallpaper"}}"#).unwrap();
        older.settings.normalize();
        assert_eq!(older.settings.companion_skin, "pot");
        for saved in ["pot", "cat", "bunny", "nezha", "bear", "<script>"] {
            let mut prefs: Preferences = serde_json::from_str(&format!(r#"{{"settings":{{"companionSkin":"{saved}"}}}}"#)).unwrap();
            prefs.settings.normalize();
            assert_eq!(prefs.settings.companion_skin, "pot", "{saved}");
        }
    }
    #[test]
    fn the_white_flower_choice_survives_normalization_and_reload() {
        let mut value: Preferences = serde_json::from_str(r#"{"settings":{"companionSkin":"pot-white-flower"}}"#).unwrap();
        value.settings.normalize();
        let saved=serde_json::to_string(&value).unwrap();
        let mut again:Preferences=serde_json::from_str(&saved).unwrap();
        again.settings.normalize();
        assert_eq!(again.settings.companion_skin,"pot-white-flower");
        assert_eq!(again.settings.size,Settings::default().size);
    }

    #[test]
    fn the_assistant_is_on_by_default_and_an_unknown_shortcut_falls_back() {
        let defaults = Settings::default();
        assert!(defaults.assistant_enabled);
        assert_eq!(defaults.assistant_shortcut, "Alt+Space");
        let mut older: Preferences = serde_json::from_str(r#"{"settings":{"mode":"wallpaper"}}"#).unwrap();
        older.settings.normalize();
        assert!(older.settings.assistant_enabled);
        assert_eq!(older.settings.assistant_shortcut, "Alt+Space");
        let mut chosen: Preferences = serde_json::from_str(r#"{"settings":{"assistantEnabled":false,"assistantShortcut":"Ctrl+Alt+Space"}}"#).unwrap();
        chosen.settings.normalize();
        assert!(!chosen.settings.assistant_enabled);
        assert_eq!(chosen.settings.assistant_shortcut, "Ctrl+Alt+Space");
        let mut odd: Preferences = serde_json::from_str(r#"{"settings":{"assistantShortcut":"Ctrl+Q"}}"#).unwrap();
        odd.settings.normalize();
        assert_eq!(odd.settings.assistant_shortcut, "Alt+Space");
    }

    #[test]
    fn the_appearance_defaults_to_the_system_and_repairs_an_unknown_value() {
        assert_eq!(Settings::default().appearance, "system");
        assert!(!Settings::default().appearance_kept);
        for saved in ["system", "light", "dark"] {
            let mut prefs: Preferences = serde_json::from_str(&format!(r#"{{"settings":{{"appearance":"{saved}"}}}}"#)).unwrap();
            prefs.settings.normalize();
            assert_eq!(prefs.settings.appearance, saved);
        }
        let mut odd: Preferences = serde_json::from_str(r#"{"settings":{"appearance":"sepia"}}"#).unwrap();
        odd.settings.normalize();
        assert_eq!(odd.settings.appearance, "system");
        let mut older: Preferences = serde_json::from_str(r#"{"settings":{"mode":"wallpaper"}}"#).unwrap();
        older.settings.normalize();
        assert_eq!(older.settings.appearance, "system");
    }

    #[test]
    fn an_appearance_saved_by_an_older_version_counts_as_not_yet_kept_by_the_host() {
        // Older files have no `appearanceKept`: the console's own saved choice must still be carried up once.
        let mut older: Preferences = serde_json::from_str(r#"{"settings":{"mode":"floating"}}"#).unwrap();
        older.settings.normalize();
        assert!(!older.settings.appearance_kept);
        let mut kept: Preferences = serde_json::from_str(r#"{"settings":{"appearance":"dark","appearanceKept":true}}"#).unwrap();
        kept.settings.normalize();
        assert!(kept.settings.appearance_kept && kept.settings.appearance == "dark");
    }

    #[test]
    fn the_assistant_ball_and_its_voice_preferences_default_on_and_repair_bad_values() {
        let defaults = Settings::default();
        assert!(defaults.assistant_ball && !defaults.assistant_hint_seen);
        assert_eq!((defaults.assistant_open.as_str(), defaults.assistant_read.as_str()), ("voice", "voice"));
        let mut chosen: Preferences = serde_json::from_str(
            r#"{"settings":{"assistantBall":false,"assistantBallDx":-120.5,"assistantBallDy":30,"assistantOpen":"type","assistantRead":"never"}}"#,
        ).unwrap();
        chosen.settings.normalize();
        assert!(!chosen.settings.assistant_ball);
        assert_eq!((chosen.settings.assistant_ball_dx, chosen.settings.assistant_ball_dy), (-120.5, 30.0));
        assert_eq!((chosen.settings.assistant_open.as_str(), chosen.settings.assistant_read.as_str()), ("type", "never"));
        let mut bad: Preferences = serde_json::from_str(
            r#"{"settings":{"assistantBallDx":1e30,"assistantBallDy":5,"assistantOpen":"shout","assistantRead":"loud"}}"#,
        ).unwrap();
        bad.settings.normalize();
        assert_eq!((bad.settings.assistant_ball_dx, bad.settings.assistant_ball_dy), (0.0, 0.0));
        assert_eq!((bad.settings.assistant_open.as_str(), bad.settings.assistant_read.as_str()), ("voice", "voice"));
    }
    #[test]
    fn effects_chosen_per_wallpaper_become_the_desktop_wide_choice_once() {
        let mut value: Preferences = serde_json::from_str(
            r#"{"settings":{"wallpaperOptions":{"effects":{"leaves":{"enabled":true,"intensity":0.8,"speed":1.5}}}}}"#).unwrap();
        value.settings.normalize();
        assert_eq!(value.settings.effects.selected, "maple");
        assert_eq!(value.settings.effects.params["maple"].density, 1.6);
        assert!(value.settings.wallpaper_options.effects.is_null());
        // Saving writes only the new place, so choosing "no effects" later is not undone by a stale copy.
        let saved = serde_json::to_string(&value).unwrap();
        let tree: serde_json::Value = serde_json::from_str(&saved).unwrap();
        assert!(tree["settings"]["wallpaperOptions"].get("effects").is_none(), "{saved}");
        assert_eq!(tree["settings"]["effects"]["selected"], "maple", "{saved}");
        let mut again: Preferences = serde_json::from_str(&saved).unwrap();
        again.settings.effects.selected.clear();
        again.settings.normalize();
        assert_eq!(again.settings.effects.selected, "");
    }
    #[test]
    fn the_previous_global_shape_and_a_desktop_wide_choice_are_both_kept() {
        let mut value: Preferences = serde_json::from_str(r#"{"settings":{
            "effects":{"rain":{"enabled":true,"intensity":0.4,"speed":2},"snow":{"enabled":false}},
            "wallpaperOptions":{"effects":{"snow":{"enabled":true}}}}}"#).unwrap();
        value.settings.normalize();
        assert_eq!(value.settings.effects.selected, "rain", "an existing desktop-wide choice is not replaced by an old per-wallpaper one");
        assert_eq!(value.settings.effects.params["rain"].density, 0.8);
        let broken: Preferences = serde_json::from_str(r#"{"settings":{"effects":"nonsense"}}"#).unwrap();
        assert_eq!(broken.settings.effects, Default::default());
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
    fn a_saved_custom_website_is_replaced_by_the_built_in_one() {
        let mut value: Preferences = serde_json::from_str(r#"{"settings":{
            "websiteUrl":"https://other.example","updateUrl":"https://other.example/updates.json","automaticUpdates":false}}"#).unwrap();
        value.settings.normalize();
        assert_eq!(value.settings.website_url, crate::website::DEFAULT_SITE);
        assert_eq!(value.settings.update_url, format!("{}/api/v1/desktop/updates", crate::website::DEFAULT_SITE));
        assert!(!value.settings.automatic_updates, "turning automatic checks off is still the user's choice");
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

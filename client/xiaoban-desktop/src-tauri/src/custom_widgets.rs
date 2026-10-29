//! Custom standalone HTML desktop widgets.
//! Each widget is a single JSON pack containing metadata and a self-contained HTML document.
//! They are stored in `~/.xiaobanwallpaper/custom_widgets/<id>.json`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const MAX_WIDGET_FILE: u64 = 128 * 1024; // 128 KB
pub const MAX_USER_WIDGETS: usize = 40;
/// The eight reference cards the library ships with. A custom template may not reuse these ids.
pub const BUILTIN_KINDS: [&str; 8] = ["clock", "weather", "notes", "reminders", "stocks", "woodfish", "calendar", "countdown"];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomWidgetPack {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_width")]
    pub width: f64,
    #[serde(default = "default_height")]
    pub height: f64,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub html: String,
}

fn default_width() -> f64 {
    280.0
}
fn default_height() -> f64 {
    240.0
}
fn default_theme() -> String {
    "transparent".to_string()
}

pub fn valid_widget_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=48).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'-' | b'_'))
}

pub fn is_builtin_kind(id: &str) -> bool {
    BUILTIN_KINDS.contains(&id) || id == "custom"
}

/// Validates a custom widget pack structure.
pub fn validate(pack: &Value) -> Result<(), String> {
    if pack.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err("schemaVersion 需要是 1".into());
    }
    let id = pack
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| valid_widget_id(id))
        .ok_or("缺少有效的 id（小写字母、数字、- 和 _，不超过 48 个字符）")?;
    if is_builtin_kind(id) {
        return Err(format!("id「{id}」与内置组件重名，请换一个"));
    }
    let name = pack
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or("缺少组件名称 name")?;
    if name.chars().count() > 24 {
        return Err("组件名称 name 最多 24 个字".into());
    }
    if let Some(desc) = pack.get("description").filter(|v| !v.is_null()) {
        let text = desc.as_str().ok_or("description 需要是文字")?;
        if text.chars().count() > 80 {
            return Err("description 最多 80 个字".into());
        }
    }
    if let Some(w) = pack.get("width").filter(|v| !v.is_null()) {
        let width = w.as_f64().ok_or("width 必须是数字")?;
        if !width.is_finite() || !(100.0..=800.0).contains(&width) {
            return Err("width 宽度范围为 100 到 800 像素".into());
        }
    }
    if let Some(h) = pack.get("height").filter(|v| !v.is_null()) {
        let height = h.as_f64().ok_or("height 必须是数字")?;
        if !height.is_finite() || !(80.0..=800.0).contains(&height) {
            return Err("height 高度范围为 80 到 800 像素".into());
        }
    }
    if let Some(theme) = pack.get("theme").filter(|v| !v.is_null()) {
        let t = theme.as_str().ok_or("theme 必须是字符串")?;
        if !["transparent", "paper", "glass"].contains(&t) {
            return Err("theme 必须是 transparent、paper 或 glass".into());
        }
    }
    let html = pack
        .get("html")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .ok_or("缺少 html 内容")?;
    if html.len() as u64 > MAX_WIDGET_FILE {
        return Err("html 内容过大，不能超过 128 KB".into());
    }
    Ok(())
}

/// Loads all custom widget packs from `directory`, in file-name order.
pub fn load_all(directory: &Path) -> Vec<Value> {
    let mut list = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return list;
    };
    let mut names: Vec<_> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    for name in names {
        if name.starts_with('_') || name.starts_with('.') || !name.to_lowercase().ends_with(".json") {
            continue;
        }
        if list.len() >= MAX_USER_WIDGETS {
            break;
        }
        let path = directory.join(&name);
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if metadata.is_file() && metadata.len() <= MAX_WIDGET_FILE {
                if let Ok(text) = fs::read_to_string(&path) {
                    if let Ok(val) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
                        if validate(&val).is_ok() {
                            list.push(val);
                        }
                    }
                }
            }
        }
    }
    list
}

/// Finds a custom widget pack by id in `directory`.
pub fn find(directory: &Path, id: &str) -> Option<Value> {
    if !valid_widget_id(id) {
        return None;
    }
    let path = directory.join(format!("{id}.json"));
    if let Ok(text) = fs::read_to_string(&path) {
        if let Ok(val) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
            if val.get("id").and_then(Value::as_str) == Some(id) && validate(&val).is_ok() {
                return Some(val);
            }
        }
    }
    load_all(directory).into_iter().find(|p| p.get("id").and_then(Value::as_str) == Some(id))
}

/// Atomically saves a custom widget pack into `directory`.
pub fn save(directory: &Path, pack: &Value, overwrite: bool) -> Result<PathBuf, String> {
    validate(pack)?;
    let id = pack["id"].as_str().unwrap_or_default();
    let path = directory.join(format!("{id}.json"));
    if path.exists() && !overwrite {
        return Err(format!("小组件「{id}」已经存在（{id}.json），请换个 id 或确认覆盖它"));
    }
    let text = serde_json::to_string_pretty(pack).map_err(|e| e.to_string())?;
    if text.len() as u64 > MAX_WIDGET_FILE {
        return Err("小组件文件超过 128 KB".into());
    }
    fs::create_dir_all(directory).map_err(|e| format!("无法创建小组件文件夹：{e}"))?;
    let temporary = directory.join(format!(".{id}-{}.tmp", std::process::id()));
    fs::write(&temporary, text.as_bytes()).map_err(|e| format!("无法写入小组件文件：{e}"))?;
    fs::rename(&temporary, &path).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        format!("无法保存小组件：{e}")
    })?;
    Ok(path)
}

/// Removes a custom widget pack file by id.
pub fn remove(directory: &Path, id: &str) -> Result<(), String> {
    if !valid_widget_id(id) {
        return Err("组件 id 不合法".into());
    }
    let path = directory.join(format!("{id}.json"));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("无法删除小组件：{e}"))?;
        Ok(())
    } else {
        Err(format!("未找到小组件 {id}"))
    }
}

/// The size a custom widget asks for, falling back to the library's own default.
pub fn pack_size(pack: &Value) -> (f64, f64) {
    let width = pack.get("width").and_then(Value::as_f64).unwrap_or(280.0);
    let height = pack.get("height").and_then(Value::as_f64).unwrap_or(240.0);
    (width, height)
}

/// The empty pack the console shows before anything has been created.
pub fn placeholder() -> Value {
    json!({"schemaVersion":1,"id":"","name":"","html":"","width":280.0,"height":240.0})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_custom_widget_pack() {
        let valid = json!({
            "schemaVersion": 1,
            "id": "pomodoro-timer",
            "name": "番茄工作钟",
            "width": 260.0,
            "height": 200.0,
            "theme": "glass",
            "html": "<div id='app'>25:00</div>"
        });
        assert!(validate(&valid).is_ok());

        for builtin in BUILTIN_KINDS {
            let clashing = json!({"schemaVersion":1,"id":builtin,"name":"测试","html":"<div>test</div>"});
            assert!(validate(&clashing).is_err(), "{builtin} is reserved");
        }
        let invalid_id = json!({"schemaVersion":1,"id":"My_Clock!","name":"时钟","html":"<div>clock</div>"});
        assert!(validate(&invalid_id).is_err());
        let too_big = json!({"schemaVersion":1,"id":"big","name":"大","html":"x".repeat(MAX_WIDGET_FILE as usize + 1)});
        assert!(validate(&too_big).is_err());
    }

    #[test]
    fn saves_and_finds_a_pack_round_trip() {
        let dir = std::env::temp_dir().join(format!("xiaoban-custom-widgets-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let pack = json!({"schemaVersion":1,"id":"pomodoro","name":"番茄钟","width":300.0,"height":180.0,"html":"<p>25:00</p>"});
        save(&dir, &pack, false).unwrap();
        assert!(save(&dir, &pack, false).is_err(), "the same id needs overwrite");
        let found = find(&dir, "pomodoro").expect("saved pack is readable");
        assert_eq!(pack_size(&found), (300.0, 180.0));
        assert_eq!(load_all(&dir).len(), 1);
        remove(&dir, "pomodoro").unwrap();
        assert!(find(&dir, "pomodoro").is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_files_are_skipped_when_listing() {
        let dir = std::env::temp_dir().join(format!("xiaoban-custom-widgets-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("broken.json"), "not json").unwrap();
        fs::write(dir.join("reserved.json"), r#"{"schemaVersion":1,"id":"clock","name":"冒充时钟","html":"<p>x</p>"}"#).unwrap();
        assert!(load_all(&dir).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}

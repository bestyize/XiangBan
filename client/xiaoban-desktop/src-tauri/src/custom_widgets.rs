//! Custom standalone HTML desktop widgets.
//! Each widget is a single JSON pack containing metadata and a self-contained HTML document.
//! They are stored in `~/.xiaobanwallpaper/custom_widgets/<id>.json`.

use crate::standalone_html;
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

/// Official built-in HTML widget templates shipped with the desktop app.
pub const BUILTIN_WIDGETS: [(&str, &str); 1] = [
    ("pelican-bike", include_str!("../../public/widgets/pelican-bike.json")),
];

pub fn builtin() -> Vec<Value> {
    BUILTIN_WIDGETS
        .iter()
        .filter_map(|(_, text)| serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')).ok())
        .filter(|val| validate(val).is_ok())
        .map(|mut pack| {
            pack["source"] = json!("builtin");
            pack
        })
        .collect()
}

pub fn is_builtin_template(id: &str) -> bool {
    BUILTIN_WIDGETS.iter().any(|(bid, _)| *bid == id)
}

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

fn validate_metadata(pack: &Value) -> Result<(), String> {
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
    Ok(())
}

/// Validates the inline JSON form used by authoring and ZIP import.
pub fn validate(pack: &Value) -> Result<(), String> {
    validate_metadata(pack)?;
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

/// Whether a CSS background value draws nothing.
fn clear_background(value: &str) -> bool {
    let value = value.trim().trim_end_matches("!important").trim().to_ascii_lowercase();
    if ["transparent", "none", "inherit", "initial", "unset", ""].contains(&value.as_str()) || value.starts_with("var(") {
        return true;
    }
    let alpha = |inner: &str| inner.rsplit([',', '/']).next().map(str::trim).is_some_and(|a| a.trim_end_matches('%').parse::<f64>().is_ok_and(|a| a == 0.0));
    ["rgba(", "hsla(", "rgb(", "hsl("].iter().any(|f| value.strip_prefix(f).and_then(|v| v.strip_suffix(')')).is_some_and(|inner| (inner.contains(',') && inner.split(',').count() == 4 || inner.contains('/')) && alpha(inner)))
}

/// Report common HTML incompatibilities for the assistant's preview and save tools.
pub(crate) fn widget_html_problems(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut problems = Vec::new();
    if lower.contains("-webkit-app-region") {
        problems.push("不要使用 -webkit-app-region：宿主已经提供按住空白处拖动".to_string());
    }
    if lower.contains("__tauri") || lower.contains("@tauri-apps") {
        problems.push("不要直接调用 Tauri，组件在沙盒里运行".to_string());
    }
    for storage in ["localStorage", "sessionStorage"] {
        let accesses_storage = html.match_indices(storage).any(|(start, _)| {
            let preceding = html[..start].chars().next_back();
            let suffix = html[start + storage.len()..].trim_start();
            !preceding.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '$'))
                && (suffix.starts_with('.') || suffix.starts_with('[') || suffix.starts_with("?."))
        });
        if accesses_storage {
            problems.push(format!(
                "组件沙箱禁止访问 {storage}，读取或写入都会抛错。请用内存保存本次输入，让首次界面直接可见，提交有效数据后立即更新界面。"
            ));
        }
    }
    let mut opaque = |what: &str, declarations: &str| {
        for declaration in declarations.split(';') {
            let Some((property, value)) = declaration.split_once(':') else { continue };
            let property = property.trim();
            if (property == "background" || property == "background-color") && !clear_background(value) {
                problems.push(format!("{what} 的背景是 `{}`。请让 html、body 保持透明，把组件背景画在内部容器上。", value.trim()));
                return;
            }
        }
    };
    let mut rest = lower.as_str();
    while let Some(start) = rest.find("<style") {
        let after = &rest[start..];
        let Some(open) = after.find('>') else { break };
        let end = after.find("</style").unwrap_or(after.len());
        let css = &after[(open + 1).min(end)..end];
        let mut selector_start = 0;
        while let Some(brace) = css[selector_start..].find('{').map(|i| i + selector_start) {
            let Some(close) = css[brace..].find('}').map(|i| i + brace) else { break };
            let selector = css[selector_start..brace].rsplit("*/").next().unwrap_or("");
            if let Some(page) = selector.split(',').map(str::trim).find(|s| *s == "html" || *s == "body" || *s == ":root") {
                opaque(page, &css[brace + 1..close]);
            }
            selector_start = close + 1;
        }
        rest = &after[end..];
    }
    if let Some(tag) = lower.find("<body").map(|i| &lower[i..]).and_then(|t| t.split_once('>').map(|(tag, _)| tag)) {
        if let Some(style) = tag.split_once("style=").map(|(_, v)| v.trim_start_matches(['"', '\'']).split(['"', '\'']).next().unwrap_or("")) {
            opaque("body", style);
        }
    }
    problems
}

/// Loads all user custom widget packs from `directory`, in file-name order.
pub fn load_user(directory: &Path) -> Vec<Value> {
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
                    if let Ok(mut val) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
                        if validate(&val).is_ok() {
                            if let Some(fields) = val.as_object_mut() { fields.remove("standaloneHtml"); }
                            val["source"] = json!("user");
                            list.push(val);
                        }
                    }
                }
            }
        }
    }
    let base = directory.join("html");
    if fs::symlink_metadata(&base).is_ok_and(|meta| meta.is_dir()) {
        if let Ok(entries) = fs::read_dir(base) {
            let mut ids: Vec<_> = entries.flatten().filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|id| valid_widget_id(id)).collect();
            ids.sort();
            for id in ids {
                if list.len() >= MAX_USER_WIDGETS { break; }
                if list.iter().any(|pack| pack["id"] == id) { continue; }
                if let Ok(pack) = standalone_entry(directory, &id) { list.push(pack); }
            }
        }
    }
    list
}

/// Loads all custom widget packs: built-ins first (with user overrides applied), then user-created widgets.
pub fn load_all(directory: &Path) -> Vec<Value> {
    let mut list = Vec::new();
    let mut user_packs: std::collections::HashMap<String, Value> = std::collections::HashMap::new();

    for pack in load_user(directory) {
        if let Some(id) = pack.get("id").and_then(Value::as_str) {
            user_packs.insert(id.to_string(), pack);
        }
    }

    for builtin_pack in builtin() {
        if let Some(id) = builtin_pack.get("id").and_then(Value::as_str) {
            if let Some(user_pack) = user_packs.remove(id) {
                list.push(user_pack);
            } else {
                list.push(builtin_pack);
            }
        }
    }

    let mut remaining: Vec<_> = user_packs.into_values().collect();
    remaining.sort_by(|a, b| {
        a.get("id").and_then(Value::as_str).unwrap_or_default()
            .cmp(b.get("id").and_then(Value::as_str).unwrap_or_default())
    });
    list.extend(remaining);
    list
}

/// Finds a custom widget pack by id in `directory`, falling back to built-ins.
pub fn find(directory: &Path, id: &str) -> Option<Value> {
    if !valid_widget_id(id) {
        return None;
    }
    let path = directory.join(format!("{id}.json"));
    if let Ok(text) = fs::read_to_string(&path) {
        if let Ok(mut val) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
            if val.get("id").and_then(Value::as_str) == Some(id) && validate(&val).is_ok() {
                val["source"] = json!("user");
                if let Some(fields) = val.as_object_mut() { fields.remove("standaloneHtml"); }
                return Some(val);
            }
        }
    }
    standalone_entry(directory, id).ok()
        .or_else(|| builtin().into_iter().find(|p| p.get("id").and_then(Value::as_str) == Some(id)))
}

fn standalone_dir(directory: &Path, id: &str) -> Result<PathBuf, String> {
    standalone_html::installed_dir(&directory.join("html"), id)
}

fn standalone_entry(directory: &Path, id: &str) -> Result<Value, String> {
    let path = standalone_dir(directory, id)?;
    let bytes = crate::resource_packs::read_bytes(&path.join("manifest.json"), MAX_WIDGET_FILE)?;
    let mut pack: Value = serde_json::from_slice(&bytes).map_err(|_| "组件清单无效")?;
    if pack["id"] != id { return Err("组件清单 id 与文件夹不一致".into()); }
    validate_metadata(&pack)?;
    standalone_html::document(&path)?;
    pack["standaloneHtml"] = json!(true);
    pack["source"] = json!("user");
    Ok(pack)
}

pub fn standalone_document(directory: &Path, id: &str) -> Result<PathBuf, String> {
    standalone_html::document(&standalone_dir(directory, id)?)
}

pub fn import_html(directory: &Path, source: &Path, id: &str, name: &str) -> Result<(), String> {
    let pack = json!({"schemaVersion": 1, "id": id, "name": name});
    validate_metadata(&pack)?;
    if find(directory, id).is_some() { return Err(format!("组件「{id}」已存在")); }
    if load_user(directory).len() >= MAX_USER_WIDGETS {
        return Err(format!("最多导入 {MAX_USER_WIDGETS} 个自定义组件"));
    }
    standalone_html::install(&directory.join("html"), id, &pack, |target| standalone_html::copy(source, target))
}

/// The bounded JSON form for ZIP export and the component authoring tool.
pub fn read_inline(directory: &Path, id: &str) -> Result<Value, String> {
    let mut pack = find(directory, id).ok_or_else(|| format!("未找到组件模板「{id}」"))?;
    if pack["standaloneHtml"] == true {
        let path = standalone_document(directory, id)?;
        let html = String::from_utf8(crate::resource_packs::read_bytes(&path, MAX_WIDGET_FILE)
            .map_err(|_| "独立 HTML 超过 128 KiB 的组件 ZIP 和创作工具上限，请直接使用原 HTML 文件".to_string())?)
            .map_err(|_| "组件源码需要使用 UTF-8")?;
        if let Some(fields) = pack.as_object_mut() { fields.remove("standaloneHtml"); }
        pack["html"] = json!(html);
        validate(&pack)?;
        let mut payload = pack.clone();
        payload.as_object_mut().expect("validated pack").remove("source");
        inline_json(&payload)?;
    }
    Ok(pack)
}

fn inline_json(pack: &Value) -> Result<String, String> {
    let text = serde_json::to_string_pretty(pack).map_err(|e| e.to_string())?;
    if text.len() as u64 > MAX_WIDGET_FILE { return Err("小组件文件超过 128 KB".into()); }
    Ok(text)
}

/// Atomically saves a custom widget pack into `directory`.
pub fn save(directory: &Path, pack: &Value, overwrite: bool) -> Result<PathBuf, String> {
    validate(pack)?;
    let id = pack["id"].as_str().unwrap_or_default();
    let path = directory.join(format!("{id}.json"));
    let standalone = standalone_dir(directory, id).ok();
    if (path.exists() || standalone.is_some() || is_builtin_template(id)) && !overwrite {
        return Err(format!("小组件「{id}」已经存在，请换个 id 或确认覆盖它"));
    }
    let text = inline_json(pack)?;
    fs::create_dir_all(directory).map_err(|e| format!("无法创建小组件文件夹：{e}"))?;
    let temporary = directory.join(format!(".{id}-{}.tmp", std::process::id()));
    fs::write(&temporary, text.as_bytes()).map_err(|e| format!("无法写入小组件文件：{e}"))?;
    fs::rename(&temporary, &path).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        format!("无法保存小组件：{e}")
    })?;
    if let Some(dir) = standalone { let _ = fs::remove_dir_all(dir); }
    Ok(path)
}

/// Removes a custom widget pack file by id.
pub fn remove(directory: &Path, id: &str) -> Result<(), String> {
    if !valid_widget_id(id) {
        return Err("组件 id 不合法".into());
    }
    let path = directory.join(format!("{id}.json"));
    let standalone = standalone_dir(directory, id).ok();
    if let Some(dir) = &standalone {
        fs::remove_dir_all(dir).map_err(|e| format!("无法删除 HTML 组件：{e}"))?;
    }
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("无法删除小组件：{e}"))?;
        Ok(())
    } else if standalone.is_some() {
        Ok(())
    } else if is_builtin_template(id) {
        Err(format!("「{id}」为内置组件，不能删除"))
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
        assert_eq!(load_user(&dir).len(), 1);
        assert_eq!(load_all(&dir).len(), builtin().len() + 1);
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
        assert!(load_user(&dir).is_empty());
        assert_eq!(load_all(&dir).len(), builtin().len());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn builtin_pelican_bike_is_shipped_and_found() {
        let dir = std::env::temp_dir().join(format!("xiaoban-custom-widgets-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let pelican = find(&dir, "pelican-bike").expect("pelican-bike builtin widget is found");
        assert_eq!(pelican["name"], "鹈鹕骑车");
        assert_eq!(pelican["source"], "builtin");
        assert_eq!(pack_size(&pelican), (300.0, 260.0));
        assert!(remove(&dir, "pelican-bike").is_err(), "builtin widget cannot be removed");
    }
}

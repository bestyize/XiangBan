//! Effect packs: the data files that describe each particle effect. The built-in ones are compiled into the app;
//! more can be dropped into the `effects` folder of the user data directory. The host only checks that a file is
//! a sound JSON object with a usable id; the page that draws it checks and clamps every number again.
use crate::effect_choice::valid_id;
use serde_json::{json, Value};
use std::{collections::HashSet, fs, path::{Path, PathBuf}};

const BUILTIN: [(&str, &str); 12] = [
    ("lanterns", include_str!("../../public/effects/lanterns.json")),
    ("maple", include_str!("../../public/effects/maple.json")),
    ("galaxy", include_str!("../../public/effects/galaxy.json")),
    ("butterfly-dream", include_str!("../../public/effects/butterfly-dream.json")),
    ("fireworks", include_str!("../../public/effects/fireworks.json")),
    ("snow", include_str!("../../public/effects/snow.json")),
    ("rain", include_str!("../../public/effects/rain.json")),
    ("fireflies", include_str!("../../public/effects/fireflies.json")),
    ("petals", include_str!("../../public/effects/petals.json")),
    ("meteors", include_str!("../../public/effects/meteors.json")),
    ("stars", include_str!("../../public/effects/stars.json")),
    ("aurora", include_str!("../../public/effects/aurora.json")),
];
#[cfg(test)]
pub const CATALOG: &str = include_str!("../../public/effects/catalog.json");
/// Written into a new `effects` folder so the format has an example next to it. A name that starts with `_` is not loaded.
pub const EXAMPLE: &str = include_str!("../../docs/effect-pack-example.json");
pub const EXAMPLE_NAME: &str = "_示例-星尘.json";
const MAX_FILE: u64 = 64 * 1024;
const MAX_USER_PACKS: usize = 40;

pub struct Packs { pub packs: Vec<Value>, pub errors: Vec<String> }

fn builtin() -> Vec<Value> {
    BUILTIN.iter().filter_map(|(_, text)| serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')).ok())
        .map(|mut pack| { pack["source"] = json!("builtin"); pack }).collect()
}

/// Reads `*.json` from `directory`. A bad file is reported by name and skipped; it never stops the others.
pub fn load_user(directory: &Path, taken: &mut HashSet<String>) -> Packs {
    let mut result = Packs { packs: vec![], errors: vec![] };
    let Ok(entries) = fs::read_dir(directory) else { return result };
    let mut names: Vec<_> = entries.flatten().filter_map(|entry| entry.file_name().into_string().ok()).collect();
    names.sort();
    for name in names {
        if name.starts_with('_') || name.starts_with('.') || !name.to_lowercase().ends_with(".json") { continue; }
        if result.packs.len() >= MAX_USER_PACKS { result.errors.push(format!("{name}：最多加载 {MAX_USER_PACKS} 个自定义特效，其余忽略")); break; }
        let path = directory.join(&name);
        let outcome = (|| -> Result<Value, String> {
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if !metadata.is_file() { return Err("不是普通文件".into()); }
            if metadata.len() > MAX_FILE { return Err("文件超过 64 KB".into()); }
            let text = fs::read_to_string(&path).map_err(|_| "无法读取（需要 UTF-8 文本）".to_string())?;
            let mut pack: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("JSON 格式错误：{e}"))?;
            let id = pack["id"].as_str().filter(|id| valid_id(id)).ok_or("缺少有效的 id（小写字母、数字、- 和 _）")?.to_string();
            if pack["schemaVersion"] != 1 { return Err("schemaVersion 需要是 1".into()); }
            if !taken.insert(id.clone()) { return Err(format!("id「{id}」已被其他特效使用")); }
            pack["source"] = json!("user");
            Ok(pack)
        })();
        match outcome { Ok(pack) => result.packs.push(pack), Err(error) => result.errors.push(format!("{name}：{error}")) }
    }
    result
}

/// Built-in packs first, then the user's own from `directory`.
pub fn all(directory: &Path) -> Packs {
    let mut packs = builtin();
    let mut taken: HashSet<String> = packs.iter().filter_map(|pack| pack["id"].as_str().map(str::to_string)).collect();
    let user = load_user(directory, &mut taken);
    packs.extend(user.packs);
    Packs { packs, errors: user.errors }
}
pub fn find(directory: &Path, id: &str) -> Option<Value> {
    all(directory).packs.into_iter().find(|pack| pack["id"] == id)
}

/// Whether `id` is one of the compiled-in packs, which a user file must never shadow.
pub fn is_builtin_id(id: &str) -> bool { BUILTIN.iter().any(|(name, _)| *name == id) }

fn hex_color(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        let digits = text.strip_prefix('#').unwrap_or("");
        (digits.len() == 3 || digits.len() == 6) && digits.chars().all(|c| c.is_ascii_hexdigit())
    })
}

fn check_colors(value: &Value, where_: &str) -> Result<(), String> {
    let list = value.as_array().filter(|list| (1..=24).contains(&list.len()))
        .ok_or_else(|| format!("{where_}的 colors 需要 1 到 24 个颜色"))?;
    for item in list {
        let (base, tip) = match item {
            Value::Array(pair) if !pair.is_empty() => (pair.first().unwrap(), pair.get(1).unwrap_or(&pair[0])),
            other => (other, other),
        };
        if !hex_color(base) || !hex_color(tip) { return Err(format!("{where_}的颜色只能写成 #rgb 或 #rrggbb")); }
    }
    Ok(())
}

/// Checks the structure of a pack the app is about to write, so a broken file never reaches the folder.
/// The page that draws it checks and clamps every number again; this only refuses what is clearly wrong.
pub fn validate(pack: &Value) -> Result<(), String> {
    if pack["schemaVersion"] != 1 { return Err("schemaVersion 需要是 1".into()); }
    let id = pack["id"].as_str().filter(|id| valid_id(id)).ok_or("缺少有效的 id（小写字母、数字、- 和 _，不超过 48 个字符）")?;
    if is_builtin_id(id) { return Err(format!("id「{id}」是内置特效的名字，请换一个")); }
    let name = pack["name"].as_str().map(str::trim).filter(|name| !name.is_empty()).ok_or("缺少 name")?;
    if name.chars().count() > 24 { return Err("name 最多 24 个字".into()); }
    if let Some(description) = pack.get("description").filter(|value| !value.is_null()) {
        let text = description.as_str().ok_or("description 需要是文字")?;
        if text.chars().count() > 80 { return Err("description 最多 80 个字".into()); }
    }
    if let Some(sky) = pack.get("sky").filter(|value| !value.is_null()) {
        let list = sky.as_array().filter(|list| (2..=4).contains(&list.len())).ok_or("sky 需要 2 到 4 个颜色")?;
        if !list.iter().all(hex_color) { return Err("sky 的颜色只能写成 #rgb 或 #rrggbb".into()); }
    }
    let layers = pack["layers"].as_array().filter(|layers| (1..=6).contains(&layers.len()))
        .ok_or("layers 需要 1 到 6 层")?;
    let mut total = 0f64;
    for (index, layer) in layers.iter().enumerate() {
        let where_ = format!("第 {} 层", index + 1);
        if !layer.is_object() { return Err(format!("{where_}不是一个对象")); }
        let kind = layer["shape"]["kind"].as_str().unwrap_or("");
        if !["circle", "glow", "path", "streak", "ring", "beam"].contains(&kind) {
            return Err(format!("{where_}的 shape.kind 只能是 circle、glow、path、streak、ring 或 beam"));
        }
        if kind == "path" {
            let d = layer["shape"]["d"].as_str().unwrap_or("");
            if d.is_empty() || d.chars().count() > 6000 { return Err(format!("{where_}的 shape.d 必须是不超过 6000 个字符的 SVG 路径数据")); }
        }
        check_colors(&layer["colors"], &where_)?;
        // `count` may be left out (the format says 0), but a layer that draws nothing is not worth saving.
        let count = match layer.get("count").filter(|value| !value.is_null()) {
            None => 0.0,
            Some(value) => value.as_f64().filter(|n| n.is_finite() && (0.0..=1500.0).contains(n))
                .ok_or_else(|| format!("{where_}的 count 需要是 0 到 1500 的数字"))?,
        };
        total += count;
    }
    if total <= 0.0 { return Err("所有层的 count 都是 0，这个特效不会画出任何东西".into()); }
    if total > 3000.0 { return Err("所有层的 count 加起来不能超过 3000".into()); }
    Ok(())
}

/// Writes a pack as `<id>.json` into `directory`, replacing the file only when `overwrite` allows it.
/// The check runs first, and the file is written through a temporary name so a half-written pack is never read.
pub fn save(directory: &Path, pack: &Value, overwrite: bool) -> Result<PathBuf, String> {
    validate(pack)?;
    let id = pack["id"].as_str().unwrap_or_default();
    let path = directory.join(format!("{id}.json"));
    // The same id in a differently named file would shadow this one, since only the first file wins.
    if let Some(taken) = file_with_id(directory, id).filter(|name| *name != format!("{id}.json")) {
        return Err(format!("已有一个特效包（{taken}）使用了同一个 id「{id}」，请换一个 id，或先删掉那个文件"));
    }
    if path.exists() && !overwrite { return Err(format!("特效「{id}」已经存在（{id}.json），请换个 id，或确认覆盖它")); }
    let text = serde_json::to_string_pretty(pack).map_err(|e| e.to_string())?;
    if text.len() as u64 > MAX_FILE { return Err("特效包超过 64 KB".into()); }
    fs::create_dir_all(directory).map_err(|e| format!("无法创建特效文件夹：{e}"))?;
    let temporary = directory.join(format!(".{id}-{}.tmp", std::process::id()));
    fs::write(&temporary, text.as_bytes()).map_err(|e| format!("无法写入特效包：{e}"))?;
    fs::rename(&temporary, &path).map_err(|e| { let _ = fs::remove_file(&temporary); format!("无法保存特效包：{e}") })?;
    Ok(path)
}

/// The name of the file in `directory` that already carries this `id`, if one does. Hidden files
/// (a leading `_` or `.`) are skipped: they are not loaded, so they cannot clash with anything.
pub fn file_with_id(directory: &Path, id: &str) -> Option<String> {
    let mut names: Vec<String> = fs::read_dir(directory).ok()?.flatten()
        .filter_map(|entry| entry.file_name().into_string().ok()).collect();
    names.sort();
    for name in names {
        if name.starts_with('_') || name.starts_with('.') || !name.to_lowercase().ends_with(".json") { continue; }
        let Ok(text) = fs::read_to_string(directory.join(&name)) else { continue };
        let Ok(pack) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) else { continue };
        if pack["id"].as_str() == Some(id) { return Some(name); }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn folder(tag: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("xiaoban-effects-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn the_catalog_lists_exactly_the_built_in_packs_in_order() {
        let catalog: Value = serde_json::from_str(CATALOG).unwrap();
        let listed: Vec<&str> = catalog["effects"].as_array().unwrap().iter().map(|id| id.as_str().unwrap()).collect();
        assert_eq!(listed, BUILTIN.iter().map(|(id, _)| *id).collect::<Vec<_>>());
        let packs = builtin();
        assert_eq!(packs.len(), BUILTIN.len(), "every built-in file is valid JSON");
        for (pack, (id, _)) in packs.iter().zip(BUILTIN) {
            assert_eq!(pack["id"], id); assert_eq!(pack["schemaVersion"], 1); assert_eq!(pack["source"], "builtin");
            assert!(pack["layers"].as_array().is_some_and(|layers| !layers.is_empty()), "{id}");
        }
    }
    #[test]
    fn the_shipped_example_is_a_usable_pack_that_is_not_loaded_by_name() {
        let example: Value = serde_json::from_str(EXAMPLE).unwrap();
        assert!(valid_id(example["id"].as_str().unwrap()) && example["schemaVersion"] == 1);
        let dir = folder("example");
        fs::write(dir.join(EXAMPLE_NAME), EXAMPLE).unwrap();
        assert!(all(&dir).packs.iter().all(|pack| pack["source"] == "builtin"), "a file starting with _ is left alone");
        fs::write(dir.join("glitter.json"), EXAMPLE).unwrap();
        assert_eq!(find(&dir, "glitter").unwrap()["source"], "user");
    }
    #[test]
    fn user_packs_load_and_bad_files_are_named_without_stopping_the_rest() {
        let dir = folder("user");
        fs::write(dir.join("a-good.json"), r#"{"schemaVersion":1,"id":"a-good","name":"好","layers":[{"count":1}]}"#).unwrap();
        fs::write(dir.join("broken.json"), "{ not json").unwrap();
        fs::write(dir.join("old.json"), r#"{"schemaVersion":2,"id":"old"}"#).unwrap();
        fs::write(dir.join("noid.json"), r#"{"schemaVersion":1,"id":"Bad Id"}"#).unwrap();
        fs::write(dir.join("rain.json"), r#"{"schemaVersion":1,"id":"rain","name":"冒名"}"#).unwrap();
        fs::write(dir.join("huge.json"), format!(r#"{{"schemaVersion":1,"id":"huge","pad":"{}"}}"#, "x".repeat(70_000))).unwrap();
        fs::write(dir.join("notes.txt"), "ignored").unwrap();
        fs::write(dir.join("_off.json"), r#"{"schemaVersion":1,"id":"off"}"#).unwrap();
        let loaded = all(&dir);
        let ids: Vec<&str> = loaded.packs.iter().map(|pack| pack["id"].as_str().unwrap()).collect();
        assert_eq!(ids.last(), Some(&"a-good"));
        assert!(!ids.contains(&"off") && !ids.contains(&"huge") && !ids.contains(&"old"));
        assert_eq!(loaded.packs.iter().filter(|pack| pack["id"] == "rain").count(), 1);
        assert_eq!(find(&dir, "rain").unwrap()["source"], "builtin", "a user file cannot replace a built-in effect");
        assert_eq!(loaded.errors.len(), 5, "{:?}", loaded.errors);
        for name in ["broken.json", "old.json", "noid.json", "rain.json", "huge.json"] { assert!(loaded.errors.iter().any(|e| e.starts_with(name)), "{name}"); }
    }
    #[test]
    fn a_missing_folder_just_means_no_user_packs() {
        let loaded = all(&std::env::temp_dir().join("xiaoban-no-such-effects-folder"));
        assert_eq!(loaded.packs.len(), BUILTIN.len());
        assert!(loaded.errors.is_empty());
    }

    fn good_pack() -> Value {
        json!({"schemaVersion": 1, "id": "glitter", "name": "星尘", "description": "亮粉", "sky": ["#101828", "#2a3358"],
            "layers": [{"count": 120, "shape": {"kind": "glow", "core": 0.2}, "colors": [["#fff4c2", "#ffc94d"]], "size": [8, 18]}]})
    }

    #[test]
    fn a_pack_is_refused_with_a_reason_when_it_is_not_usable() {
        assert!(validate(&good_pack()).is_ok(), "the example is fine");
        for (change, fragment) in [
            (json!({"schemaVersion": 2}), "schemaVersion"),
            (json!({"id": "Bad Id"}), "id"),
            (json!({"id": "rain"}), "内置"),
            (json!({"id": ""}), "id"),
            (json!({"name": "  "}), "name"),
            (json!({"name": "一".repeat(25)}), "24"),
            (json!({"description": "一".repeat(81)}), "80"),
            (json!({"sky": ["#fff"]}), "sky"),
            (json!({"sky": ["#fff", "red"]}), "sky"),
            (json!({"layers": []}), "1 到 6 层"),
            (json!({"layers": [{"count": 5, "shape": {"kind": "spiral"}, "colors": ["#fff"]}]}), "shape.kind"),
            (json!({"layers": [{"count": 5, "shape": {"kind": "circle"}}]}), "colors"),
            (json!({"layers": [{"count": 5, "shape": {"kind": "circle"}, "colors": ["#fff", "#f00", "#0f0", "#00f", "#abc", "#cba", "#123", "#321", "#456", "#654", "#789", "#987", "#aaa", "#bbb", "#ccc", "#ddd", "#eee", "#fff", "#111", "#222", "#333", "#444", "#555", "#666", "#777"]}]}), "colors"),
            (json!({"layers": [{"count": 5, "shape": {"kind": "circle"}, "colors": ["#fff", "rgba(0,0,0,1)"]}]}), "颜色"),
            (json!({"layers": [{"count": 2000, "shape": {"kind": "circle"}, "colors": ["#fff"]}]}), "count"),
            (json!({"layers": [{"shape": {"kind": "circle"}, "colors": ["#fff"]}]}), "count 都是 0"),
            (json!({"layers": [{"count": 1500, "shape": {"kind": "circle"}, "colors": ["#fff"]}, {"count": 1500, "shape": {"kind": "circle"}, "colors": ["#fff"]}, {"count": 1, "shape": {"kind": "circle"}, "colors": ["#fff"]}]}), "3000"),
            (json!({"layers": [{"count": 5, "shape": {"kind": "path"}, "colors": ["#fff"]}]}), "shape.d"),
        ] {
            let mut pack = good_pack();
            for (key, value) in change.as_object().unwrap() { pack[key] = value.clone(); }
            let error = validate(&pack).unwrap_err();
            assert!(error.contains(fragment), "{change}: {error}");
        }
    }

    #[test]
    fn a_saved_pack_is_named_after_its_id_and_loads_back() {
        let dir = folder("save");
        let path = save(&dir, &good_pack(), false).unwrap();
        assert_eq!(path.file_name().unwrap().to_string_lossy(), "glitter.json");
        assert_eq!(find(&dir, "glitter").unwrap()["source"], "user");
        assert!(fs::read_to_string(&path).unwrap().contains("\"id\": \"glitter\""), "saved as readable JSON");
        assert!(fs::read_dir(&dir).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")), "no temporary file is left behind");
    }

    #[test]
    fn saving_never_silently_replaces_another_pack() {
        let dir = folder("replace");
        save(&dir, &good_pack(), false).unwrap();
        let error = save(&dir, &good_pack(), false).unwrap_err();
        assert!(error.contains("已经存在"), "{error}");
        assert!(save(&dir, &good_pack(), true).is_ok(), "overwrite is allowed when asked for");

        // The same id under a different file name would shadow this one, so it is refused either way.
        fs::rename(dir.join("glitter.json"), dir.join("mine.json")).unwrap();
        let error = save(&dir, &good_pack(), true).unwrap_err();
        assert!(error.contains("mine.json"), "{error}");
        assert!(error.contains("同一个 id"), "{error}");
    }

    #[test]
    fn a_file_with_the_same_id_is_found_even_under_another_name() {
        let dir = folder("with-id");
        fs::write(dir.join("mine.json"), good_pack().to_string()).unwrap();
        fs::write(dir.join("_off.json"), good_pack().to_string()).unwrap();
        fs::write(dir.join("other.json"), json!({"id": "other"}).to_string()).unwrap();
        assert_eq!(file_with_id(&dir, "glitter").as_deref(), Some("mine.json"));
        assert_eq!(file_with_id(&dir, "other").as_deref(), Some("other.json"));
        assert_eq!(file_with_id(&dir, "missing"), None, "a file whose name starts with _ is not loaded");
    }
}

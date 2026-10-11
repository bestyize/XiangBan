//! Effect packs: the data files that describe each particle effect. The built-in ones are compiled into the app;
//! more can be dropped into the `effects` folder of the user data directory. The host only checks that a file is
//! a sound JSON object with a usable id; the page that draws it checks and clamps every number again.
use crate::effect_choice::valid_id;
use serde_json::{json, Value};
use std::{collections::HashSet, fs, path::Path};

const BUILTIN: [(&str, &str); 13] = [
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
    ("ripples", include_str!("../../public/effects/ripples.json")),
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
}

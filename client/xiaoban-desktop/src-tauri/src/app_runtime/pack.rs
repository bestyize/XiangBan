//! Package rules for the V2 window runtime, shared by authoring and ZIP exchange.
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::Path,
};

pub const MAX_FILES: usize = 3_000;
pub const MAX_ENTRIES: usize = MAX_FILES * 8;
pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_PACK_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
pub const GAME_CATEGORIES: &[&str] = &["action", "puzzle", "board", "casual", "story"];
pub const APP_CATEGORIES: &[&str] = &["tool", "quiz"];
pub const INPUTS: &[&str] = &["touch", "mouse", "keyboard"];
pub const CAPABILITIES: &[&str] = &[
    "haptics",
    "share",
    "clipboard",
    "screen",
    "notify",
    "profile",
    "camera",
    "microphone",
    "contacts",
    "files",
];

pub type Files = Vec<(String, Vec<u8>)>;

fn valid_segment(segment: &str) -> bool {
    if segment.is_empty()
        || segment.starts_with('.')
        || segment.ends_with('.')
        || !segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-@".contains(&byte))
    {
        return false;
    }
    // A package must be writable on Windows as well as on case-sensitive file systems.
    let stem = segment.split('.').next().unwrap_or("").to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

pub fn valid_id(id: &str) -> bool {
    id.len() >= 2 && super::storage::valid_app_id(id) && valid_segment(id)
}

fn valid_path(path: &str) -> bool {
    path.split('/').count() <= 8
        && path.split('/').all(valid_segment)
        && !path
            .split('/')
            .next()
            .is_some_and(|part| part.eq_ignore_ascii_case("__MACOSX"))
}

fn extension(path: &str) -> String {
    path.rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default()
}

fn check_file_path(path: &str) -> Result<String, String> {
    let ext = extension(path);
    if !valid_path(path)
        || path
            .split('/')
            .next()
            .is_some_and(|part| part.eq_ignore_ascii_case("manifest.json"))
        || super::protocol::mime_for(&ext).is_none()
    {
        return Err(format!("作品文件路径或类型不受支持：{path}"));
    }
    Ok(ext)
}

fn string<'a>(manifest: &'a Value, key: &str) -> Result<&'a str, String> {
    manifest
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("manifest 的 {key} 需要是非空文本"))
}

fn check_list(manifest: &Value, key: &str, allowed: &[&str], required: bool) -> Result<(), String> {
    let Some(value) = manifest.get(key) else {
        return if required {
            Err(format!("manifest 缺少 {key}"))
        } else {
            Ok(())
        };
    };
    let list = value
        .as_array()
        .ok_or_else(|| format!("{key} 需要是文本列表"))?;
    let mut seen = HashSet::new();
    if (required && list.is_empty())
        || !list.iter().all(|value| {
            value
                .as_str()
                .is_some_and(|value| allowed.contains(&value) && seen.insert(value))
        })
    {
        return Err(format!("{key} 的可选值为 {}，不能重复", allowed.join("、")));
    }
    Ok(())
}

/// Returns the exact entry, icon and cover paths that the package must contain.
pub fn check_manifest(manifest: &Value) -> Result<[&str; 3], String> {
    if manifest["schemaVersion"] != 3 || manifest["runtimeProtocol"] != 2 {
        return Err("V2 作品需要 schemaVersion=3、runtimeProtocol=2".into());
    }
    if !valid_id(string(manifest, "id")?) {
        return Err("作品 id 需要 2 到 64 位小写字母、数字、- 或 _，以字母或数字开头".into());
    }
    let categories = match string(manifest, "kind")? {
        "game" => GAME_CATEGORIES,
        "app" => APP_CATEGORIES,
        _ => return Err("作品 kind 只能是 game 或 app".into()),
    };
    if !categories.contains(&string(manifest, "category")?) {
        return Err(format!("category 的可选值为 {}", categories.join("、")));
    }
    for (key, max) in [("name", 24), ("description", 80)] {
        if string(manifest, key)?.trim().chars().count() > max {
            return Err(format!("{key} 最多 {max} 字"));
        }
    }
    let version = string(manifest, "version")?;
    if version.len() > 32
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte))
    {
        return Err("version 最多 32 位，只能用字母、数字、点和连字符".into());
    }
    if manifest
        .get("orientation")
        .is_some_and(|value| !matches!(value.as_str(), Some("portrait" | "landscape" | "any")))
    {
        return Err("orientation 只能是 portrait、landscape 或 any".into());
    }
    if manifest.get("layer").is_some() {
        return Err("V2 独立窗口作品不支持 layer，请使用普通窗口作品包".into());
    }
    check_list(manifest, "input", INPUTS, true)?;
    check_list(manifest, "capabilities", CAPABILITIES, false)?;
    if manifest.get("intents").is_some_and(|value| {
        !value.as_array().is_some_and(|list| {
            list.len() <= 8
                && list.iter().all(|item| {
                    item.as_str().is_some_and(|intent| {
                        !intent.trim().is_empty() && intent.chars().count() <= 12
                    })
                })
        })
    }) {
        return Err("intents 最多 8 条，每条最多 12 字".into());
    }
    if manifest
        .get("swipeBack")
        .is_some_and(|value| !value.is_boolean())
    {
        return Err("swipeBack 需要是布尔值".into());
    }

    let entry = if manifest.get("entry").is_some() {
        string(manifest, "entry")?
    } else {
        "index.html"
    };
    let paths = [entry, string(manifest, "icon")?, string(manifest, "cover")?];
    if !matches!(check_file_path(entry)?.as_str(), "html" | "htm") {
        return Err("entry 需要指向 HTML 入口".into());
    }
    for path in &paths[1..] {
        if !matches!(
            check_file_path(path)?.as_str(),
            "svg" | "png" | "jpg" | "jpeg" | "webp" | "avif" | "gif" | "ico"
        ) {
            return Err(format!("图标和封面需要使用图片文件：{path}"));
        }
    }
    Ok(paths)
}

pub fn validate(manifest: &Value, files: &Files) -> Result<(), String> {
    let required = check_manifest(manifest)?;
    let manifest_len = serde_json::to_vec(manifest)
        .map_err(|e| e.to_string())?
        .len() as u64;
    if manifest_len > MAX_MANIFEST_BYTES {
        return Err("manifest.json 超过 64 KB".into());
    }
    if files.len() >= MAX_FILES {
        return Err(format!("作品最多 {MAX_FILES} 个文件（含清单）"));
    }
    let mut names = HashSet::new();
    let mut total = manifest_len;
    for (path, bytes) in files {
        let ext = check_file_path(path)?;
        let len = bytes.len() as u64;
        total += len;
        if len > MAX_FILE_BYTES || total > MAX_PACK_BYTES {
            return Err(format!("作品单个文件最多 50 MB，合计最多 100 MB：{path}"));
        }
        if matches!(
            ext.as_str(),
            "html"
                | "htm"
                | "js"
                | "mjs"
                | "css"
                | "json"
                | "svg"
                | "md"
                | "txt"
                | "xml"
                | "csv"
                | "map"
                | "atlas"
                | "glsl"
        ) && std::str::from_utf8(bytes).is_err()
        {
            return Err(format!("作品文本需要使用 UTF-8：{path}"));
        }
        if !names.insert(path.to_ascii_lowercase()) {
            return Err(format!("作品文件路径重复：{path}"));
        }
    }
    let mut folders = HashMap::new();
    for (path, _) in files {
        for (index, _) in path.match_indices('/') {
            let folder = &path[..index];
            let key = folder.to_ascii_lowercase();
            if names.contains(&key)
                || folders
                    .insert(key, folder)
                    .is_some_and(|other| other != folder)
            {
                return Err(format!("作品目录路径冲突：{folder}"));
            }
        }
    }
    for path in required {
        if !files
            .iter()
            .any(|(name, bytes)| name == path && !bytes.is_empty())
        {
            return Err(format!("作品缺少文件或文件为空：{path}"));
        }
    }
    Ok(())
}

/// Reads a bounded snapshot of a draft or installed pack. A draft must leave the manifest to the creator.
pub fn read_files(root: &Path, installed: bool) -> Result<Files, String> {
    let mut files = Vec::new();
    visit(root, "", installed, &mut files, &mut 0, &mut 0)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn visit(
    dir: &Path,
    prefix: &str,
    installed: bool,
    files: &mut Files,
    entries: &mut usize,
    total: &mut u64,
) -> Result<(), String> {
    if !fs::symlink_metadata(dir)
        .map_err(|e| e.to_string())?
        .is_dir()
    {
        return Err("作品目录需要是普通文件夹".into());
    }
    for child in fs::read_dir(dir).map_err(|e| e.to_string())? {
        *entries += 1;
        if *entries > MAX_ENTRIES {
            return Err("作品目录条目过多".into());
        }
        let child = child.map_err(|e| e.to_string())?;
        let name = child
            .file_name()
            .into_string()
            .map_err(|_| "作品文件名需要是 UTF-8")?;
        if installed && prefix.is_empty() && name == "manifest.json" {
            continue;
        }
        let path = format!("{prefix}{name}");
        let kind = child.file_type().map_err(|e| e.to_string())?;
        if !valid_path(&path) || (prefix.is_empty() && name.eq_ignore_ascii_case("manifest.json")) {
            return Err(format!("作品路径不合法：{path}"));
        }
        if kind.is_dir() {
            visit(
                &child.path(),
                &format!("{path}/"),
                installed,
                files,
                entries,
                total,
            )?;
        } else if kind.is_file() {
            check_file_path(&path)?;
            if files.len() + 1 >= MAX_FILES {
                return Err(format!("作品最多 {MAX_FILES} 个文件（含清单）"));
            }
            let mut bytes = Vec::new();
            let limit = MAX_FILE_BYTES.min(MAX_PACK_BYTES - *total);
            fs::File::open(child.path())
                .map_err(|e| e.to_string())?
                .take(limit + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() as u64 > limit {
                return Err(format!("作品单个文件最多 50 MB，合计最多 100 MB：{path}"));
            }
            *total += bytes.len() as u64;
            files.push((path, bytes));
        } else {
            return Err(format!("作品只能包含普通文件和目录：{path}"));
        }
    }
    Ok(())
}

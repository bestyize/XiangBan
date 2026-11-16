//! Locally installed HTML effects keep their V2 manifest and document separate.
use crate::{
    app_runtime::{html_import, pack::SCHEMA_VERSION},
    effect_choice::valid_id,
    standalone_html,
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

pub use standalone_html::ENTRY;
pub const MAX_ZIP_HTML_BYTES: u64 = 16 * 1024 * 1024;

pub fn folder(directory: &Path) -> PathBuf {
    directory.join("html")
}

pub fn normalize_manifest(data: &Value) -> Result<Value, String> {
    let id = data["id"]
        .as_str()
        .filter(|id| valid_id(id))
        .ok_or("HTML 特效缺少有效的 id")?;
    if super::is_builtin_id(id) {
        return Err(format!("id「{id}」与内置特效重名"));
    }
    let name = data["name"]
        .as_str()
        .map(str::trim)
        .filter(|name| !name.is_empty() && name.chars().count() <= 24)
        .ok_or("特效名称需要 1 到 24 个字")?;
    let description = data["description"].as_str().unwrap_or_default();
    if description.chars().count() > 80 {
        return Err("特效描述最多 80 个字".into());
    }
    Ok(
        json!({"schemaVersion": SCHEMA_VERSION, "kind": "effect", "entry": ENTRY,
        "id": id, "name": name, "description": description}),
    )
}

pub fn validate_document(bytes: &[u8]) -> Result<(), String> {
    let html = std::str::from_utf8(bytes).map_err(|_| "特效必须使用 UTF-8 HTML")?;
    html_import::check_document(html)
}

fn pack_path(directory: &Path, id: &str) -> Result<PathBuf, String> {
    if super::is_builtin_id(id) {
        return Err("无效的 HTML 特效 id".into());
    }
    standalone_html::installed_dir(&folder(directory), id)
}

pub fn occupied(directory: &Path, id: &str) -> bool {
    fs::symlink_metadata(folder(directory).join(id)).is_ok()
}

pub fn catalog_entry(directory: &Path, id: &str) -> Result<Value, String> {
    let path = pack_path(directory, id)?;
    let bytes = crate::resource_packs::read_bytes(&path.join("manifest.json"), 64 * 1024)?;
    let manifest: Value = serde_json::from_slice(&bytes).map_err(|_| "HTML 特效的清单无效")?;
    if manifest["id"] != id
        || manifest["kind"] != "effect"
        || manifest["entry"] != ENTRY
        || manifest["schemaVersion"] != SCHEMA_VERSION
        || manifest.get("runtimeProtocol").is_some()
    {
        return Err("HTML 特效的清单无效".into());
    }
    let mut data = normalize_manifest(&manifest)?;
    let entry = fs::symlink_metadata(path.join(ENTRY)).map_err(|_| "找不到 HTML 特效文件")?;
    if !entry.is_file() || entry.len() == 0 {
        return Err("HTML 特效文件无效".into());
    }
    let modified = entry
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok());
    data["revision"] = json!(format!(
        "{}-{}",
        entry.len(),
        modified.map_or(0, |t| t.as_nanos())
    ));
    data["renderer"] = json!("html");
    data["source"] = json!("user");
    Ok(data)
}

pub fn document(directory: &Path, id: &str) -> Result<PathBuf, String> {
    standalone_html::document(&pack_path(directory, id)?)
}

pub fn save(directory: &Path, manifest: &Value, bytes: &[u8]) -> Result<(), String> {
    validate_document(bytes)?;
    save_document(directory, manifest, |target| {
        fs::write(target, bytes).map_err(|e| e.to_string())
    })
}

pub fn save_document(
    directory: &Path,
    manifest: &Value,
    write_html: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let manifest = normalize_manifest(manifest)?;
    let id = manifest["id"].as_str().unwrap_or_default();
    if occupied(directory, id) || super::file_with_id(directory, id).is_some() {
        return Err(format!("特效「{id}」已存在"));
    }
    if super::all(directory)
        .packs
        .iter()
        .filter(|pack| pack["source"] == "user")
        .count()
        >= super::MAX_USER_PACKS
    {
        return Err(format!("最多导入 {} 个自定义特效", super::MAX_USER_PACKS));
    }
    standalone_html::install(&folder(directory), id, &manifest, write_html)
}

pub fn delete(directory: &Path, id: &str) -> Result<String, String> {
    let path = pack_path(directory, id)?;
    fs::remove_dir_all(path).map_err(|e| format!("无法删除 HTML 特效：{e}"))?;
    Ok(format!("html/{id}/{ENTRY}"))
}

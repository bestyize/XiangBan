//! Portable authoring resources. Packs contain templates, never runtime user data or saves.
use crate::{custom_widgets, effect_packs, game_packs, local_media};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, io::Read, path::Path};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

const MAX_PACK: u64 = 32 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pack {
    schema_version: u32,
    kind: String,
    data: Value,
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| format!("资源包缺少 {key}"))
}

fn read_text(path: &Path, limit: u64) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err("资源文件过大或不是普通文件".into());
    }
    let mut content = String::new();
    fs::File::open(path).map_err(|e| e.to_string())?.take(limit + 1)
        .read_to_string(&mut content).map_err(|_| "资源必须使用 UTF-8 文本".to_string())?;
    if content.len() as u64 > limit { return Err("资源文件过大".into()); }
    Ok(content)
}

fn collect_game_files(directory: &Path, prefix: &str, files: &mut Vec<(String, String)>, total: &mut usize) -> Result<(), String> {
    if !fs::symlink_metadata(directory).map_err(|e| e.to_string())?.is_dir() {
        return Err("游戏文件夹不能是链接".into());
    }
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().into_string().map_err(|_| "游戏文件名不是 UTF-8")?;
        if prefix.is_empty() && name == "manifest.json" { continue; }
        let relative = format!("{prefix}{name}");
        let metadata = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if metadata.is_dir() {
            if relative.split('/').count() >= 4 { return Err("游戏目录层数过多".into()); }
            collect_game_files(&entry.path(), &format!("{relative}/"), files, total)?;
        } else {
            if files.len() >= game_packs::MAX_FILES { return Err("游戏文件数量过多".into()); }
            let content = read_text(&entry.path(), game_packs::MAX_FILE_BYTES as u64)?;
            *total += content.len();
            if *total > game_packs::MAX_TOTAL_BYTES { return Err("游戏文件合计超过 4 MB".into()); }
            files.push((relative, content));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(())
}

pub fn export(root: &Path, kind: &str, id: &str) -> Result<Pack, String> {
    let data = match kind {
        "wallpaper" => {
            let item = local_media::read(&root.join("wallpapers"))?.items.into_iter()
                .find(|item| item.id == id && item.kind == "web").ok_or("只能导出自己创作的动画壁纸")?;
            let path = local_media::resource_path(&root.join("wallpapers"), &item.resource);
            if !crate::library::verify_file(&path, &item.resource)? { return Err("壁纸文件校验失败".into()); }
            json!({"title":item.title,"description":item.description,"html":read_text(&path, 4 * 1024 * 1024)?})
        }
        "effect" => {
            if effect_packs::is_builtin_id(id) { return Err("只能导出自创特效".into()); }
            let mut pack = effect_packs::find(&root.join("effects"), id).ok_or("没有找到自创特效")?;
            effect_packs::validate(&pack)?;
            if let Some(object) = pack.as_object_mut() { object.remove("source"); }
            pack
        }
        "widget" => {
            if !custom_widgets::valid_widget_id(id) { return Err("组件标识不合法".into()); }
            let pack = custom_widgets::find(&root.join("custom_widgets"), id).ok_or("没有找到自创组件")?;
            custom_widgets::validate(&pack)?;
            pack
        }
        "game" => {
            let manifest = game_packs::find_user_game(root, id)?;
            let mut files = Vec::new();
            let mut total = 0;
            collect_game_files(&game_packs::packs_dir(root).join(id), "", &mut files, &mut total)?;
            let name = text(&manifest, "name")?;
            let description = manifest["description"].as_str().unwrap_or("");
            game_packs::validate_request(id, name, description, &files)?;
            json!({"id":id,"name":name,"description":description,"files":files,"save":manifest["save"].as_bool().unwrap_or(false)})
        }
        _ => return Err("不支持的资源类型".into()),
    };
    Ok(Pack { schema_version: 1, kind: kind.into(), data })
}

pub fn import(root: &Path, content: &str, expected_kind: &str) -> Result<Value, String> {
    if content.len() as u64 > MAX_PACK { return Err("资源包不能超过 32 MB".into()); }
    let pack: Pack = serde_json::from_str(content).map_err(|_| "不是有效的小伴资源包".to_string())?;
    if pack.schema_version != 1 { return Err("资源包版本暂不支持，请更新小伴桌面".into()); }
    if pack.kind != expected_kind { return Err("资源类型不匹配，请到对应的壁纸、特效、组件或游戏页导入".into()); }
    let data = &pack.data;
    let (id, name) = match pack.kind.as_str() {
        "wallpaper" => {
            let item = local_media::save_local_html(&root.join("wallpapers"), text(data, "title")?, Some(text(data, "description")?), text(data, "html")?)?;
            (item.id, item.title)
        }
        "effect" => {
            if effect_packs::find(&root.join("effects"), text(data, "id")?).is_some() { return Err("同名特效已存在，请先修改资源包的 id 或移除原作品".into()); }
            effect_packs::save(&root.join("effects"), data, false)?;
            (text(data, "id")?.to_string(), text(data, "name")?.to_string())
        }
        "widget" => {
            if !custom_widgets::valid_widget_id(text(data, "id")?) { return Err("组件标识不合法".into()); }
            if custom_widgets::find(&root.join("custom_widgets"), text(data, "id")?).is_some() { return Err("同名组件已存在，请先修改资源包的 id 或移除原作品".into()); }
            custom_widgets::save(&root.join("custom_widgets"), data, false)?;
            (text(data, "id")?.to_string(), text(data, "name")?.to_string())
        }
        "game" => {
            let files: Vec<(String, String)> = serde_json::from_value(data["files"].clone()).map_err(|_| "游戏文件清单不合法")?;
            let id = text(data, "id")?;
            let name = text(data, "name")?;
            game_packs::create_pack(root, id, name, text(data, "description")?, &files, data["save"].as_bool().unwrap_or(false), false)?;
            (id.to_string(), name.to_string())
        }
        _ => return Err("不支持的资源类型".into()),
    };
    Ok(json!({"kind":pack.kind,"id":id,"name":name}))
}

/// Asks where to save, then writes the pack. `None` means the user cancelled the dialog. Blocks, so call it off the main thread.
pub fn export_with_dialog(app: &tauri::AppHandle, kind: &str, id: &str) -> Result<Option<(std::path::PathBuf, u64)>, String> {
    let pack = export(&crate::storage::root(app)?, kind, id)?;
    let content = serde_json::to_vec_pretty(&pack).map_err(|e| e.to_string())?;
    if content.len() as u64 > MAX_PACK { return Err("资源包不能超过 32 MB".into()); }
    let filename = format!("{kind}-{id}.xiaoban");
    let Some(file) = app.dialog().file().set_title("导出小伴资源包").add_filter("小伴资源包", &["xiaoban"]).set_file_name(&filename).blocking_save_file() else { return Ok(None); };
    let path = file.into_path().map_err(|_| "请选择本机文件")?;
    fs::write(&path, &content).map_err(|e| format!("导出失败：{e}"))?;
    Ok(Some((path, content.len() as u64)))
}

#[tauri::command]
pub async fn export_resource_pack(window: tauri::WebviewWindow, kind: String, id: String) -> Result<bool, String> {
    if window.label() != "control" { return Err("只能从控制台导出资源".into()); }
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || export_with_dialog(&app, &kind, &id).map(|saved| saved.is_some()))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_resource_pack(window: tauri::WebviewWindow, kind: String) -> Result<Option<Value>, String> {
    if window.label() != "control" { return Err("只能从控制台导入资源".into()); }
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app.dialog().file().set_title("导入小伴资源包").add_filter("小伴资源包", &["xiaoban"]).blocking_pick_file() else { return Ok(None); };
        let path = file.into_path().map_err(|_| "请选择本机文件")?;
        let content = read_text(&path, MAX_PACK)?;
        let root = crate::storage::root(&app)?;
        // Wallpaper creation and downloads share the same catalog lock.
        let state = app.state::<crate::content::ContentState>();
        let _guard = state.io.lock().unwrap();
        import(&root, &content, &kind).map(Some)
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wallpaper_round_trip_preserves_animation_and_never_applies_it() {
        let root = std::env::temp_dir().join(format!("xiaoban-resource-roundtrip-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let original = local_media::save_local_html(&root.join("a/wallpapers"), "星空", Some("动画"), "<!doctype html><html><body><canvas></canvas></body></html>").unwrap();
        let pack = export(&root.join("a"), "wallpaper", &original.id).unwrap();
        let content = serde_json::to_string(&pack).unwrap();
        let imported = import(&root.join("b"), &content, "wallpaper").unwrap();
        assert_eq!(imported["id"], original.id);
        let items = local_media::read(&root.join("b/wallpapers")).unwrap().items;
        assert_eq!(items[0].kind, "web");
        assert_eq!(items[0].resource.sha256, original.resource.sha256);
        assert!(!root.join("b/wallpapers/active.json").exists());
        assert!(import(&root.join("c"), &content, "effect").is_err());
        assert!(!root.join("c").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn import_rejects_unknown_versions_types_and_game_traversal() {
        let root = std::env::temp_dir().join(format!("xiaoban-resource-rejected-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for content in [
            json!({"schemaVersion":2,"kind":"game","data":{}}),
            json!({"schemaVersion":1,"kind":"unknown","data":{}}),
            json!({"schemaVersion":1,"kind":"game","data":{"id":"my-game","name":"游戏","description":"test","files":[["../escape.html","bad"],["index.html","ok"]]}}),
        ] {
            let kind = content["kind"].as_str().unwrap();
            assert!(import(&root, &content.to_string(), kind).is_err());
        }
        assert!(!root.exists());
    }
}

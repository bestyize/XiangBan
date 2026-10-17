//! Portable authoring resources, shared as plain `.zip` files so any tool can make or open them.
//! Packs contain templates, never runtime user data or saves. The layout is in docs/resource-pack-format.md.
use crate::{custom_widgets, effect_packs, game_packs, local_media};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MAX_PACK: u64 = 32 * 1024 * 1024;
/// Games hold at most 40 files plus the manifest.
const MAX_ENTRIES: usize = 100;
/// The largest single file a pack may hold is a wallpaper page.
const MAX_ENTRY: u64 = 4 * 1024 * 1024;

pub struct Pack {
    kind: String,
    data: Value,
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| format!("资源包缺少 {key}"))
}

fn read_bytes(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err("资源文件过大或不是普通文件".into());
    }
    let mut content = Vec::new();
    fs::File::open(path).map_err(|e| e.to_string())?.take(limit + 1)
        .read_to_end(&mut content).map_err(|e| e.to_string())?;
    if content.len() as u64 > limit { return Err("资源文件过大".into()); }
    Ok(content)
}

fn read_text(path: &Path, limit: u64) -> Result<String, String> {
    String::from_utf8(read_bytes(path, limit)?).map_err(|_| "资源必须使用 UTF-8 文本".to_string())
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
    Ok(Pack { kind: kind.into(), data })
}

/// The manifest and the files a pack is written as. Wallpapers and widgets keep their page in `index.html`,
/// an effect is its own `effect.json`, and a game keeps its folder as it is.
fn zip_entries(pack: &Pack) -> Result<(Value, Vec<(String, Vec<u8>)>), String> {
    let data = &pack.data;
    let mut manifest = json!({"schemaVersion": 1, "kind": pack.kind});
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    match pack.kind.as_str() {
        "wallpaper" => {
            manifest["title"] = json!(text(data, "title")?);
            manifest["description"] = json!(data["description"].as_str().unwrap_or(""));
            files.push(("index.html".into(), text(data, "html")?.as_bytes().to_vec()));
        }
        "effect" => files.push(("effect.json".into(), serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?)),
        "widget" => {
            for (key, value) in data.as_object().ok_or("组件内容不合法")? {
                if key != "html" && key != "schemaVersion" { manifest[key] = value.clone(); }
            }
            files.push(("index.html".into(), text(data, "html")?.as_bytes().to_vec()));
        }
        "game" => {
            for key in ["id", "name", "description"] { manifest[key] = json!(text(data, key)?); }
            manifest["save"] = json!(data["save"].as_bool().unwrap_or(false));
            let list: Vec<(String, String)> = serde_json::from_value(data["files"].clone()).map_err(|_| "游戏文件清单不合法")?;
            files.extend(list.into_iter().map(|(path, content)| (path, content.into_bytes())));
        }
        _ => return Err("不支持的资源类型".into()),
    }
    Ok((manifest, files))
}

fn to_zip(pack: &Pack) -> Result<Vec<u8>, String> {
    let (manifest, files) = zip_entries(pack)?;
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut add = |name: &str, bytes: &[u8]| -> Result<(), String> {
        writer.start_file(name, options).map_err(|e| e.to_string())?;
        writer.write_all(bytes).map_err(|e| e.to_string())
    };
    add("manifest.json", &serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?)?;
    for (name, bytes) in &files { add(name, bytes)?; }
    Ok(writer.finish().map_err(|e| e.to_string())?.into_inner())
}

fn clean_name(raw: &str) -> Result<String, String> {
    let ok = !raw.starts_with('/') && !raw.contains(['\\', ':', '\0']) && raw.split('/').all(|part| !part.is_empty() && part != "." && part != "..");
    if ok { Ok(raw.to_string()) } else { Err(format!("压缩包里的文件路径不合法：{raw}")) }
}

/// Reads a zip into a pack. Sizes are counted while unpacking, so a small archive cannot expand without limit.
fn from_zip(bytes: &[u8]) -> Result<Pack, String> {
    if bytes.len() as u64 > MAX_PACK { return Err("资源包不能超过 32 MB".into()); }
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "不是有效的 zip 资源包".to_string())?;
    if archive.len() > MAX_ENTRIES { return Err(format!("压缩包里的文件太多，最多 {MAX_ENTRIES} 个")); }
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|e| format!("压缩包无法读取：{e}"))?;
        let name = file.name().map_err(|e| format!("压缩包里的文件名无法读取：{e}"))?.to_string();
        // Folder entries and the files macOS's Finder and `ditto` add (`__MACOSX/`, `.DS_Store`, `._name`) carry no content.
        let base = name.rsplit('/').next().unwrap_or_default();
        if file.is_dir() || name.starts_with("__MACOSX/") || base == ".DS_Store" || base.starts_with("._") { continue; }
        if file.is_symlink() { return Err("压缩包不能包含符号链接".into()); }
        let name = clean_name(&name)?;
        let mut content = Vec::new();
        (&mut file).take(MAX_ENTRY + 1).read_to_end(&mut content).map_err(|e| format!("无法解压「{name}」：{e}"))?;
        if content.len() as u64 > MAX_ENTRY { return Err(format!("文件「{name}」超过 4 MB")); }
        total += content.len() as u64;
        if total > MAX_PACK { return Err("资源包解压后超过 32 MB".into()); }
        entries.push((name, content));
    }
    // Zipping a folder wraps everything in that folder; step into it when manifest.json is not at the top.
    if !entries.iter().any(|(name, _)| name == "manifest.json") {
        if let Some(prefix) = entries.first().and_then(|(name, _)| name.split_once('/')).map(|(top, _)| format!("{top}/")) {
            if entries.iter().all(|(name, _)| name.starts_with(&prefix)) {
                for (name, _) in &mut entries { *name = name[prefix.len()..].to_string(); }
            }
        }
    }
    let mut seen = HashSet::new();
    for (name, _) in &entries {
        if !seen.insert(name.to_ascii_lowercase()) { return Err(format!("压缩包里有重复的文件：{name}")); }
    }
    let file = |name: &str| entries.iter().find(|(entry, _)| entry == name).map(|(_, content)| content.as_slice());
    let utf8 = |name: &str, content: &[u8]| String::from_utf8(content.to_vec()).map_err(|_| format!("「{name}」必须使用 UTF-8 文本"));
    let page = || -> Result<String, String> { utf8("index.html", file("index.html").ok_or("压缩包里缺少 index.html")?) };
    let manifest: Value = serde_json::from_slice(file("manifest.json").ok_or("压缩包里缺少 manifest.json，它要放在最外层")?)
        .map_err(|_| "manifest.json 不是有效的 JSON".to_string())?;
    if manifest.get("schemaVersion").and_then(Value::as_u64) != Some(1) { return Err("资源包版本暂不支持，请更新小伴桌面".into()); }
    let kind = text(&manifest, "kind")?.to_string();
    let data = match kind.as_str() {
        "wallpaper" => json!({"title": text(&manifest, "title")?, "description": manifest["description"].as_str().unwrap_or(""), "html": page()?}),
        "effect" => serde_json::from_slice(file("effect.json").ok_or("压缩包里缺少 effect.json")?).map_err(|_| "effect.json 不是有效的 JSON".to_string())?,
        "widget" => {
            let mut data = manifest.as_object().cloned().ok_or("manifest.json 不合法")?;
            data.remove("kind");
            data.insert("html".into(), json!(page()?));
            Value::Object(data)
        }
        "game" => {
            let mut files = Vec::new();
            for (name, content) in entries.iter().filter(|(name, _)| name != "manifest.json") { files.push((name.clone(), utf8(name, content)?)); }
            files.sort();
            json!({"id": text(&manifest, "id")?, "name": text(&manifest, "name")?, "description": text(&manifest, "description")?, "files": files, "save": manifest["save"].as_bool().unwrap_or(false)})
        }
        _ => return Err("不支持的资源类型".into()),
    };
    Ok(Pack { kind, data })
}

pub fn import(root: &Path, content: &[u8], expected_kind: &str) -> Result<Value, String> {
    let pack = from_zip(content)?;
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
    let content = to_zip(&pack)?;
    if content.len() as u64 > MAX_PACK { return Err("资源包不能超过 32 MB".into()); }
    let filename = format!("{kind}-{id}.zip");
    let Some(file) = app.dialog().file().set_title("导出小伴资源包").add_filter("小伴资源包（zip）", &["zip"]).set_file_name(&filename).blocking_save_file() else { return Ok(None); };
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
        let Some(file) = app.dialog().file().set_title("导入小伴资源包").add_filter("小伴资源包（zip）", &["zip"]).blocking_pick_file() else { return Ok(None); };
        let path = file.into_path().map_err(|_| "请选择本机文件")?;
        let content = read_bytes(&path, MAX_PACK)?;
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

    fn make_zip(files: &[(&str, &str)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, content) in files {
            writer.start_file(*name, SimpleFileOptions::default()).unwrap();
            writer.write_all(content.as_bytes()).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xiaoban-resource-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn wallpaper_round_trip_preserves_animation_and_never_applies_it() {
        let root = temp_root("roundtrip");
        let original = local_media::save_local_html(&root.join("a/wallpapers"), "星空", Some("动画"), "<!doctype html><html><body><canvas></canvas></body></html>").unwrap();
        let pack = export(&root.join("a"), "wallpaper", &original.id).unwrap();
        let content = to_zip(&pack).unwrap();
        assert!(content.starts_with(b"PK"), "the pack is an ordinary zip");
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
    fn every_kind_survives_a_zip_round_trip() {
        let widget = json!({"schemaVersion": 1, "id": "tiny-clock", "name": "小时钟", "description": "一个时钟", "width": 200, "height": 120, "theme": "glass", "html": "<div id=t></div><script>t.textContent=1</script>"});
        let effect: Value = serde_json::from_str(effect_packs::EXAMPLE).unwrap();
        let game = json!({"id": "star-catcher", "name": "接星星", "description": "接住星星", "save": true,
            "files": [["game.js", "var a = 1;"], ["index.html", "<script src=\"game.js\"></script>"], ["sub/style.css", "body{}"]]});
        for (kind, data) in [("widget", widget), ("effect", effect), ("game", game)] {
            let pack = Pack { kind: kind.into(), data: data.clone() };
            let back = from_zip(&to_zip(&pack).unwrap()).unwrap();
            assert_eq!(back.kind, kind);
            assert_eq!(back.data, data, "{kind}");
        }
    }

    #[test]
    fn hand_made_zips_import_like_exported_ones() {
        let root = temp_root("handmade");
        // A folder zipped by the Finder: everything sits inside one folder and macOS adds junk next to it.
        let widget = make_zip(&[
            ("tiny-clock/manifest.json", r#"{"schemaVersion":1,"kind":"widget","id":"tiny-clock","name":"小时钟","width":200,"height":120}"#),
            ("tiny-clock/index.html", "<div>12:00</div>"),
            ("__MACOSX/tiny-clock/._index.html", "junk"),
            ("tiny-clock/.DS_Store", "junk"),
            ("tiny-clock/._manifest.json", "junk"),
        ]);
        let done = import(&root, &widget, "widget").unwrap();
        assert_eq!(done["id"], "tiny-clock");
        assert!(custom_widgets::find(&root.join("custom_widgets"), "tiny-clock").is_some());

        let effect = make_zip(&[("manifest.json", r#"{"schemaVersion":1,"kind":"effect"}"#), ("effect.json", effect_packs::EXAMPLE)]);
        assert!(import(&root, &effect, "effect").is_ok());

        let game = make_zip(&[
            ("manifest.json", r#"{"schemaVersion":1,"kind":"game","id":"star-catcher","name":"接星星","description":"接住星星"}"#),
            ("index.html", "<script src=\"game.js\"></script>"),
            ("game.js", "var a = 1;"),
        ]);
        assert_eq!(import(&root, &game, "game").unwrap()["id"], "star-catcher");

        let wallpaper = make_zip(&[("manifest.json", r#"{"schemaVersion":1,"kind":"wallpaper","title":"手作"}"#), ("index.html", "<html><body>hi</body></html>")]);
        assert_eq!(import(&root, &wallpaper, "wallpaper").unwrap()["name"], "手作");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn import_rejects_unknown_versions_types_and_unsafe_archives() {
        let root = temp_root("rejected");
        let manifest = |kind: &str, version: u32| format!(r#"{{"schemaVersion":{version},"kind":"{kind}","id":"my-game","name":"游戏","description":"test"}}"#);
        let mut symlink = ZipWriter::new(Cursor::new(Vec::new()));
        symlink.start_file("manifest.json", SimpleFileOptions::default()).unwrap();
        symlink.write_all(manifest("game", 1).as_bytes()).unwrap();
        symlink.add_symlink("index.html", "/etc/passwd", SimpleFileOptions::default()).unwrap();
        let symlink = symlink.finish().unwrap().into_inner();
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("game", make_zip(&[("manifest.json", &manifest("game", 2)), ("index.html", "x")])),
            ("unknown", make_zip(&[("manifest.json", &manifest("unknown", 1))])),
            ("game", make_zip(&[("manifest.json", &manifest("game", 1)), ("../escape.html", "bad"), ("index.html", "ok")])),
            ("game", make_zip(&[("manifest.json", &manifest("game", 1)), ("/abs.html", "bad"), ("index.html", "ok")])),
            ("game", make_zip(&[("manifest.json", &manifest("game", 1)), ("a\\b.html", "bad"), ("index.html", "ok")])),
            ("game", make_zip(&[("manifest.json", &manifest("game", 1)), ("Index.html", "dup"), ("index.html", "ok")])),
            ("game", make_zip(&[("index.html", "no manifest")])),
            ("widget", make_zip(&[("manifest.json", &manifest("widget", 1))])),
            ("game", symlink),
            ("game", b"{\"schemaVersion\":1}".to_vec()),
            ("game", make_zip(&[("manifest.json", &manifest("game", 1)), ("index.html", "x"), ("big.js", &"a".repeat(MAX_ENTRY as usize + 1))])),
        ];
        for (kind, content) in cases {
            assert!(import(&root, &content, kind).is_err());
        }
        assert!(!root.exists(), "nothing is written for a refused pack");
    }
}

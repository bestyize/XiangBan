//! Edits managed authoring sources, with optimistic revisions and five previous versions.
//! Virtual filenames never become filesystem paths. Runtime widget data and game saves are excluded.
use crate::{custom_widgets, effect_packs, game_packs, library, local_media, resource_packs};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Read, path::{Path, PathBuf}, sync::Mutex};

static EDIT_LOCK: Mutex<()> = Mutex::new(());
const HISTORY_LIMIT: usize = 5;
const MAX_SOURCE: u64 = 32 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadRequest {
    pub kind: String,
    pub id: String,
    pub file: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change { pub file: String, pub old_string: String, pub new_string: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditRequest {
    pub kind: String,
    pub id: String,
    pub revision: String,
    pub changes: Vec<Change>,
    pub apply_now: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreRequest {
    pub kind: String,
    pub id: String,
    pub revision: String,
    pub version: String,
    pub apply_now: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct Version { revision: String, saved_at: u64 }

fn text<'a>(data: &'a Value, key: &str) -> Result<&'a str, String> {
    data[key].as_str().ok_or_else(|| format!("缺少文字字段 {key}"))
}

fn digest(data: &Value) -> Result<String, String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(data).map_err(|e| e.to_string())?)))
}

fn snapshot(root: &Path, kind: &str, id: &str) -> Result<Value, String> {
    // Export already checks resource ownership, integrity and supported source formats.
    let pack = serde_json::to_value(resource_packs::export(root, kind, id)?).map_err(|e| e.to_string())?;
    Ok(pack["data"].clone())
}

fn history_dir(root: &Path, kind: &str, id: &str) -> PathBuf {
    let key = format!("{:x}", Sha256::digest(format!("{kind}\0{id}").as_bytes()));
    root.join("resource_history").join(key)
}

fn read_json(path: &Path, max: u64) -> Result<Value, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > max { return Err("版本文件过大或不是普通文件".into()); }
    let mut bytes = Vec::new();
    fs::File::open(path).map_err(|e| e.to_string())?.take(max + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max { return Err("版本文件过大".into()); }
    serde_json::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes)).map_err(|e| e.to_string())
}

fn history(directory: &Path) -> Result<Vec<Version>, String> {
    let path = directory.join("index.json");
    if !path.exists() { return Ok(Vec::new()); }
    let versions: Vec<Version> = serde_json::from_value(read_json(&path, 8192)?).map_err(|e| e.to_string())?;
    if versions.len() > HISTORY_LIMIT || versions.iter().any(|v| !library::hash_name(&v.revision)) {
        return Err("版本记录不合法".into());
    }
    Ok(versions)
}

fn backup(root: &Path, kind: &str, id: &str, data: &Value) -> Result<(), String> {
    let directory = history_dir(root, kind, id);
    let revision = digest(data)?;
    let mut versions = history(&directory)?;
    versions.retain(|v| v.revision != revision);
    let saved_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs();
    versions.insert(0, Version { revision: revision.clone(), saved_at });
    let removed = if versions.len() > HISTORY_LIMIT { versions.pop() } else { None };
    library::save_json(&directory.join(format!("{revision}.json")), &json!({"kind":kind,"id":id,"data":data}))?;
    library::save_json(&directory.join("index.json"), &versions)?;
    if let Some(old) = removed { let _ = fs::remove_file(directory.join(format!("{}.json", old.revision))); }
    Ok(())
}

fn virtual_files(kind: &str, data: &Value) -> Result<BTreeMap<String, String>, String> {
    let mut files = BTreeMap::new();
    if kind == "effect" {
        files.insert("effect.json".into(), serde_json::to_string_pretty(data).map_err(|e| e.to_string())?);
    } else {
        let mut metadata = data.clone();
        let object = metadata.as_object_mut().ok_or("资源元数据不合法")?;
        if kind == "game" {
            let sources: Vec<(String, String)> = serde_json::from_value(object.remove("files").ok_or("缺少游戏文件")?).map_err(|e| e.to_string())?;
            files.extend(sources);
        } else {
            files.insert("index.html".into(), text(data, "html")?.to_string());
            object.remove("html");
        }
        files.insert("$metadata.json".into(), serde_json::to_string_pretty(&metadata).map_err(|e| e.to_string())?);
    }
    Ok(files)
}

fn from_files(kind: &str, mut files: BTreeMap<String, String>) -> Result<Value, String> {
    if kind == "effect" {
        return serde_json::from_str(files.get("effect.json").ok_or("缺少 effect.json")?).map_err(|e| e.to_string());
    }
    let mut data: Value = serde_json::from_str(&files.remove("$metadata.json").ok_or("缺少元数据")?).map_err(|e| e.to_string())?;
    let object = data.as_object_mut().ok_or("资源元数据必须是对象")?;
    if kind == "game" { object.insert("files".into(), json!(files.into_iter().collect::<Vec<_>>())); }
    else { object.insert("html".into(), json!(files.remove("index.html").ok_or("缺少 index.html")?)); }
    Ok(data)
}

fn validate(kind: &str, id: &str, data: &Value) -> Result<(), String> {
    if kind != "wallpaper" && data["id"].as_str() != Some(id) { return Err("编辑不能更改作品 id".into()); }
    match kind {
        "wallpaper" => {
            if data.as_object().is_none_or(|o| o.len() != 3) { return Err("壁纸仅支持 title、description、html 字段".into()); }
            let title = text(data, "title")?.trim();
            if title.is_empty() || title.chars().count() > 48 { return Err("壁纸名称需要 1 到 48 个字".into()); }
            if text(data, "description")?.chars().count() > 80 { return Err("壁纸介绍最多 80 个字".into()); }
            let html = text(data, "html")?;
            if html.trim().is_empty() || html.len() > 4 * 1024 * 1024 { return Err("HTML 必须非空且不超过 4 MB".into()); }
        }
        "effect" => {
            effect_packs::validate(data)?;
            if serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?.len() > 64 * 1024 { return Err("特效包超过 64 KB".into()); }
        }
        "widget" => {
            custom_widgets::validate(data)?;
            if serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?.len() > 128 * 1024 { return Err("组件包超过 128 KB".into()); }
        }
        "game" => {
            let files = serde_json::from_value::<Vec<(String, String)>>(data["files"].clone()).map_err(|e| e.to_string())?;
            game_packs::validate_request(id, text(data, "name")?, text(data, "description")?, &files)?;
            if !data["save"].is_boolean() { return Err("save 必须是布尔值".into()); }
        }
        _ => return Err("不支持的资源类型".into()),
    }
    Ok(())
}

// Preserve the original filename too: manually named JSON files must not shadow a new copy.
fn existing_pack(directory: &Path, id: &str, limit: u64) -> Result<PathBuf, String> {
    let mut found = None;
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name.starts_with('_') || !name.to_lowercase().ends_with(".json") { continue; }
        if read_json(&entry.path(), limit).ok().is_some_and(|v| v["id"].as_str() == Some(id)) {
            if found.is_some() { return Err("存在多个同 id 资源，请先处理重复文件".into()); }
            found = Some(entry.path());
        }
    }
    found.ok_or_else(|| "没有找到资源文件".into())
}

fn write(root: &Path, kind: &str, id: &str, data: &Value) -> Result<(), String> {
    match kind {
        "wallpaper" => { local_media::update_local_html(&root.join("wallpapers"), id, text(data, "title")?, text(data, "description")?, text(data, "html")?)?; }
        "effect" | "widget" => {
            let (directory, limit) = if kind == "effect" { ("effects", 64 * 1024) } else { ("custom_widgets", 128 * 1024) };
            let path = existing_pack(&root.join(directory), id, limit)?;
            library::save_json(&path, data)?;
        }
        "game" => {
            let files = serde_json::from_value::<Vec<(String, String)>>(data["files"].clone()).map_err(|e| e.to_string())?;
            game_packs::edit_pack(root, id, text(data, "name")?, text(data, "description")?, &files, data["save"].as_bool().unwrap_or(false))?;
        }
        _ => return Err("不支持的资源类型".into()),
    }
    Ok(())
}

pub fn read(root: &Path, request: &ReadRequest) -> Result<Value, String> {
    let _guard = EDIT_LOCK.lock().map_err(|e| e.to_string())?;
    let builtin = request.kind == "effect" && effect_packs::is_builtin_id(&request.id);
    let data = if builtin { effect_packs::find(&root.join("effects"), &request.id).ok_or("没有找到特效")? }
        else { snapshot(root, &request.kind, &request.id)? };
    let files = virtual_files(&request.kind, &data)?;
    let default_file = if request.kind == "effect" { "effect.json" } else { "index.html" };
    let file = request.file.as_deref().unwrap_or(default_file);
    let source = files.get(file).ok_or("没有这个资源文件，请使用 files 中的名称")?;
    let total = source.chars().count();
    let offset = request.offset.unwrap_or(0);
    if offset > total { return Err("offset 超出了文件长度".into()); }
    let limit = request.limit.unwrap_or(12000).clamp(1, 16000);
    let content: String = source.chars().skip(offset).take(limit).collect();
    let next = offset + content.chars().count();
    Ok(json!({"kind":request.kind,"id":request.id,"revision":digest(&data)?,"editable":!builtin,
        "files":files.iter().map(|(file,text)| json!({"file":file,"characters":text.chars().count()})).collect::<Vec<_>>(),
        "file":file,"content":content,"offset":offset,"total":total,"next_offset":if next < total {Some(next)} else {None},
        "history":history(&history_dir(root, &request.kind, &request.id))?}))
}

fn commit(root: &Path, kind: &str, id: &str, revision: &str, before: &Value, after: &Value) -> Result<Value, String> {
    if digest(before)? != revision { return Err("作品已被修改，请重新 resource_read 后再编辑".into()); }
    validate(kind, id, after)?;
    let changed = before != after;
    if changed { backup(root, kind, id, before)?; write(root, kind, id, after)?; }
    // Writers may normalize titles or metadata. Return the revision of what was actually saved.
    let saved = if changed { snapshot(root, kind, id)? } else { after.clone() };
    Ok(json!({"kind":kind,"id":id,"saved":true,"changed":changed,"revision":digest(&saved)?,"previous_revision":revision}))
}

pub fn edit(root: &Path, request: &EditRequest) -> Result<Value, String> {
    let _guard = EDIT_LOCK.lock().map_err(|e| e.to_string())?;
    if request.changes.is_empty() || request.changes.len() > 16 { return Err("changes 需要 1 到 16 处修改".into()); }
    let before = snapshot(root, &request.kind, &request.id)?;
    if digest(&before)? != request.revision { return Err("作品已被修改，请重新 resource_read 后再编辑".into()); }
    let mut files = virtual_files(&request.kind, &before)?;
    for change in &request.changes {
        let source = files.get_mut(&change.file).ok_or("只能修改 resource_read 列出的文件")?;
        if change.old_string.is_empty() || change.old_string == change.new_string || source.matches(change.old_string.as_str()).count() != 1 {
            return Err("old_string 必须在原文中恰好出现一次，且新旧内容不能相同；请读取完整上下文后重试".into());
        }
        if change.new_string.len() as u64 > MAX_SOURCE { return Err("单次修改内容过大".into()); }
        *source = source.replacen(change.old_string.as_str(), &change.new_string, 1);
    }
    let after = from_files(&request.kind, files)?;
    commit(root, &request.kind, &request.id, &request.revision, &before, &after)
}

pub fn restore(root: &Path, request: &RestoreRequest) -> Result<Value, String> {
    let _guard = EDIT_LOCK.lock().map_err(|e| e.to_string())?;
    let before = snapshot(root, &request.kind, &request.id)?;
    let directory = history_dir(root, &request.kind, &request.id);
    if !library::hash_name(&request.version) || !history(&directory)?.iter().any(|v| v.revision == request.version) {
        return Err("版本不存在或已超出最近 5 个版本".into());
    }
    let saved = read_json(&directory.join(format!("{}.json", request.version)), MAX_SOURCE)?;
    if saved["kind"] != request.kind || saved["id"] != request.id || digest(&saved["data"])? != request.version {
        return Err("版本内容校验失败".into());
    }
    commit(root, &request.kind, &request.id, &request.revision, &before, &saved["data"])
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Root(PathBuf);
    impl Root {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("xiaoban-editor-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            Self(path)
        }
    }
    impl Drop for Root { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
    fn read_source(root: &Path, kind: &str, id: &str) -> Value {
        read(root, &ReadRequest { kind:kind.into(), id:id.into(), file:None, offset:None, limit:None }).unwrap()
    }
    fn replace(root: &Path, kind: &str, id: &str, old: &str, new: &str) -> Value {
        let source = read_source(root, kind, id);
        let request: EditRequest = serde_json::from_value(json!({"kind":kind,"id":id,"revision":source["revision"],
            "changes":[{"file":source["file"],"old_string":old,"new_string":new}]})).unwrap();
        edit(root, &request).unwrap()
    }

    #[test]
    fn wallpaper_edit_keeps_identity_and_restore_can_itself_be_undone() {
        let root = Root::new("restore");
        let item = local_media::save_local_html(&root.0.join("wallpapers"), "雪", Some("冬天"), "<html>const count=80;</html>").unwrap();
        let before = read_source(&root.0, "wallpaper", &item.id);
        let saved = replace(&root.0, "wallpaper", &item.id, "count=80", "count=160");
        let catalog = local_media::read(&root.0.join("wallpapers")).unwrap();
        assert_eq!(catalog.items.len(), 1);
        assert_eq!(catalog.items[0].id, item.id);
        assert_eq!(catalog.items[0].version, "1.0.1");
        assert_ne!(catalog.items[0].resource.sha256, item.resource.sha256);
        assert!(local_media::resource_path(&root.0.join("wallpapers"), &item.resource).exists());
        let current = read_source(&root.0, "wallpaper", &item.id);
        assert_eq!(current["revision"], saved["revision"]);
        assert_eq!(current["history"][0]["revision"], before["revision"]);
        let restored = restore(&root.0, &serde_json::from_value(json!({"kind":"wallpaper","id":item.id,
            "revision":current["revision"],"version":before["revision"]})).unwrap()).unwrap();
        assert_eq!(restored["revision"], before["revision"]);
        let current = read_source(&root.0, "wallpaper", &item.id);
        assert_eq!(current["content"], before["content"]);
        assert_eq!(current["history"][0]["revision"], saved["revision"]);
        restore(&root.0, &serde_json::from_value(json!({"kind":"wallpaper","id":item.id,
            "revision":current["revision"],"version":saved["revision"]})).unwrap()).unwrap();
        assert!(read_source(&root.0, "wallpaper", &item.id)["content"].as_str().unwrap().contains("count=160"));
        let recreated = local_media::save_local_html(&root.0.join("wallpapers"), "另一张雪", Some("冬天"), "<html>const count=80;</html>").unwrap();
        assert_ne!(recreated.id,item.id);
        assert_eq!(local_media::read(&root.0.join("wallpapers")).unwrap().items.len(),2);
    }

    #[test]
    fn stale_ambiguous_invalid_and_outside_file_edits_leave_source_untouched() {
        let root = Root::new("rejected");
        let item = local_media::save_local_html(&root.0.join("wallpapers"), "雪", Some("冬天"), "<html>snow snow</html>").unwrap();
        let source = read_source(&root.0, "wallpaper", &item.id);
        for (revision, changes) in [
            ("stale".to_string(), json!([{"file":"index.html","old_string":"<html>","new_string":"<body>"}])),
            (source["revision"].as_str().unwrap().into(), json!([{"file":"index.html","old_string":"snow","new_string":"rain"}])),
            (source["revision"].as_str().unwrap().into(), json!([{"file":"../escape","old_string":"snow","new_string":"rain"}])),
            (source["revision"].as_str().unwrap().into(), json!([
                {"file":"index.html","old_string":"<html>","new_string":"<body>"},
                {"file":"$metadata.json","old_string":"\"雪\"","new_string":"\"\""}
            ])),
        ] {
            assert!(edit(&root.0, &serde_json::from_value(json!({"kind":"wallpaper","id":item.id,"revision":revision,"changes":changes})).unwrap()).is_err());
            assert_eq!(read_source(&root.0, "wallpaper", &item.id)["content"], source["content"]);
            assert_eq!(read_source(&root.0, "wallpaper", &item.id)["history"], json!([]));
        }
    }

    #[test]
    fn unicode_paging_and_history_are_bounded() {
        let root = Root::new("bounded");
        let item = local_media::save_local_html(&root.0.join("wallpapers"), "雪", Some("冬天"), "<html>雪花☃️-0</html>").unwrap();
        let paged = read(&root.0, &ReadRequest {kind:"wallpaper".into(),id:item.id.clone(),file:None,offset:Some(6),limit:Some(3)}).unwrap();
        assert_eq!(paged["content"], "雪花☃");
        assert_eq!(paged["next_offset"], 9);
        let original = read_source(&root.0, "wallpaper", &item.id)["revision"].clone();
        for i in 0..7 { replace(&root.0, "wallpaper", &item.id, &format!("-{i}</html>"), &format!("-{}</html>", i+1)); }
        let current = read_source(&root.0, "wallpaper", &item.id);
        assert_eq!(current["history"].as_array().unwrap().len(), 5);
        assert!(!current["history"].as_array().unwrap().iter().any(|v| v["revision"] == original));
        assert_eq!(fs::read_dir(history_dir(&root.0,"wallpaper",&item.id)).unwrap().count(), 6);
        assert!(restore(&root.0, &serde_json::from_value(json!({"kind":"wallpaper","id":item.id,"revision":current["revision"],"version":"../../escape"})).unwrap()).is_err());
    }

    #[test]
    fn widget_source_edit_does_not_touch_layout_or_runtime_state() {
        let root = Root::new("widget");
        let directory = root.0.join("custom_widgets");
        let pack = json!({"schemaVersion":1,"id":"my-clock","name":"时钟","html":"<html>color:red</html>"});
        custom_widgets::save(&directory, &pack, false).unwrap();
        fs::rename(directory.join("my-clock.json"), directory.join("clock-template.json")).unwrap();
        fs::write(root.0.join("widget-layout.json"), "original layout").unwrap();
        replace(&root.0, "widget", "my-clock", "color:red", "color:blue");
        assert!(!directory.join("my-clock.json").exists());
        assert!(custom_widgets::find(&directory,"my-clock").unwrap()["html"].as_str().unwrap().contains("blue"));
        assert_eq!(fs::read_to_string(root.0.join("widget-layout.json")).unwrap(), "original layout");
        let current = read_source(&root.0,"widget","my-clock");
        assert!(edit(&root.0, &serde_json::from_value(json!({"kind":"widget","id":"my-clock","revision":current["revision"],
            "changes":[{"file":"$metadata.json","old_string":"my-clock","new_string":"other-clock"}]})).unwrap()).is_err());
    }

    #[test]
    fn game_edit_preserves_other_files_and_saves() {
        let root = Root::new("game");
        game_packs::create_pack(&root.0, "my-game", "游戏", "测试游戏", &[("index.html".into(),"<html>level=1</html>".into()),("style.css".into(),"body{color:red}".into())], true, false).unwrap();
        let manifest_path = game_packs::packs_dir(&root.0).join("my-game/manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["orientation"] = json!("portrait");
        library::save_json(&manifest_path,&manifest).unwrap();
        fs::create_dir_all(game_packs::save_dir(&root.0)).unwrap();
        let save = game_packs::save_dir(&root.0).join(game_packs::user_save_name("my-game"));
        fs::write(&save,"private save").unwrap();
        replace(&root.0,"game","my-game","level=1","level=2");
        assert_eq!(fs::read_to_string(&save).unwrap(),"private save");
        assert_eq!(game_packs::find_user_game(&root.0,"my-game").unwrap()["orientation"],"portrait");
        assert_eq!(fs::read_to_string(game_packs::packs_dir(&root.0).join("my-game/style.css")).unwrap(),"body{color:red}");
        let current = read_source(&root.0,"game","my-game");
        assert!(!serde_json::to_string(&current).unwrap().contains("private save"));
    }

    #[test]
    fn builtin_effects_can_be_read_but_cannot_be_overwritten() {
        let root = Root::new("builtin");
        let source = read_source(&root.0,"effect","snow");
        assert_eq!(source["editable"],false);
        assert!(source["content"].as_str().unwrap().contains("snow"));
        assert!(edit(&root.0, &serde_json::from_value(json!({"kind":"effect","id":"snow","revision":source["revision"],
            "changes":[{"file":"effect.json","old_string":"snow","new_string":"other-snow"}]})).unwrap()).is_err());
        assert!(!root.0.exists());
    }
}

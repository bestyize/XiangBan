//! Portable authoring resources, shared as plain `.zip` files so any tool can make or open them.
//! Packs contain templates, never runtime user data or saves. The layout is in docs/resource-pack-format.md.
use crate::{app_runtime::pack as app_pack, custom_widgets, effect_packs, game_packs, local_media, pet_packs};
use crate::app_runtime::html_import;
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MAX_PACK: u64 = 32 * 1024 * 1024;
/// A game holds at most 40 text files and 60 assets, plus the manifest.
const MAX_ENTRIES: usize = 110;
/// The largest single file a pack may hold is a wallpaper page.
const MAX_ENTRY: u64 = 4 * 1024 * 1024;

pub struct Pack {
    kind: String,
    data: Value,
    /// Resource assets, or all files beside the manifest in a V2 app or game.
    assets: Vec<(String, Vec<u8>)>,
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| format!("资源包缺少 {key}"))
}

fn game_orientation(value: &Value) -> Result<Option<&str>, String> {
    match value.get("orientation").filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(value) => match value.as_str() {
            Some(direction @ ("portrait" | "landscape")) => Ok(Some(direction)),
            _ => Err("orientation 需要是 landscape 或 portrait".into()),
        },
    }
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

struct GameFiles {
    text: Vec<(String, String)>,
    text_total: usize,
    assets: Vec<(String, Vec<u8>)>,
    assets_total: usize,
}

fn collect_game_files(directory: &Path, prefix: &str, found: &mut GameFiles) -> Result<(), String> {
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
            collect_game_files(&entry.path(), &format!("{relative}/"), found)?;
        } else if game_packs::is_asset_path(&relative) {
            if found.assets.len() >= game_packs::MAX_ASSETS { return Err("游戏资源文件数量过多".into()); }
            let bytes = read_bytes(&entry.path(), game_packs::MAX_ASSET_BYTES as u64)?;
            found.assets_total += bytes.len();
            if found.assets_total > game_packs::MAX_ASSETS_TOTAL { return Err("游戏资源文件合计超过 16 MB".into()); }
            found.assets.push((relative, bytes));
        } else {
            if found.text.len() >= game_packs::MAX_FILES { return Err("游戏文件数量过多".into()); }
            let content = read_text(&entry.path(), game_packs::MAX_FILE_BYTES as u64)
                .map_err(|e| format!("「{relative}」无法导出：{e}。图片、音频和字体要放在 assets/ 下"))?;
            found.text_total += content.len();
            if found.text_total > game_packs::MAX_TOTAL_BYTES { return Err("游戏文本文件合计超过 4 MB".into()); }
            found.text.push((relative, content));
        }
    }
    found.text.sort_by(|a, b| a.0.cmp(&b.0));
    found.assets.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(())
}

pub fn export(root: &Path, kind: &str, id: &str) -> Result<Pack, String> {
    let mut assets = Vec::new();
    let data = match kind {
        "pet" => {
            let themes = pet_packs::load(&root.join("pets/themes"))?;
            let theme = themes.into_iter().find(|t| t.id == id).ok_or("只能导出自定义桌宠主题；官方主题请先另存创作")?;
            let (theme, files) = pet_packs::export_assets(&theme)?;
            assets = files;
            serde_json::to_value(theme).map_err(|e| e.to_string())?
        }
        "wallpaper" => {
            let item = local_media::read(&root.join("wallpapers"))?.items.into_iter()
                .find(|item| item.id == id && item.kind == "web").ok_or("只能导出自己创作的动画壁纸")?;
            if item.resource.bytes > MAX_ENTRY {
                return Err("壁纸超过 ZIP 资源包的 4 MB 单文件限制，可以直接分享原始 HTML。".into());
            }
            let path = local_media::resource_path(&root.join("wallpapers"), &item.resource);
            if !crate::library::verify_file(&path, &item.resource)? { return Err("壁纸文件校验失败".into()); }
            json!({"title":item.title,"description":item.description,"html":read_text(&path, MAX_ENTRY)?})
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
        "game" | "app" => {
            let manifest = game_packs::find_user_game(root, id)?;
            if manifest["runtimeProtocol"] == 2 && manifest["schemaVersion"] == app_pack::SCHEMA_VERSION {
                let (manifest, files) = game_packs::read_v2_pack(root, id)?;
                return Ok(Pack { kind: text(&manifest, "kind")?.into(), data: manifest, assets: files });
            }
            if kind == "app" { return Err("请用对应类型导出旧版游戏".into()); }
            let mut found = GameFiles { text: Vec::new(), text_total: 0, assets: Vec::new(), assets_total: 0 };
            collect_game_files(&game_packs::packs_dir(root).join(id), "", &mut found)?;
            let name = text(&manifest, "name")?;
            let description = manifest["description"].as_str().unwrap_or("");
            game_packs::validate_request(id, name, description, &found.text)?;
            assets = found.assets;
            let mut data = json!({"id":id,"name":name,"description":description,"files":found.text,"save":manifest["save"].as_bool().unwrap_or(false)});
            if let Some(orientation) = game_orientation(&manifest)? { data["orientation"] = json!(orientation); }
            let layer = game_packs::Layer::from_value(&manifest)?;
            if layer.desktop { data["layer"] = json!("desktop"); data["capabilities"] = json!(layer.capabilities); }
            data
        }
        _ => return Err("不支持的资源类型".into()),
    };
    Ok(Pack { kind: kind.into(), data, assets })
}

fn resource_entry(kind: &str) -> Option<&'static str> {
    match kind {
        "wallpaper" | "widget" => Some("index.html"),
        "effect" => Some("effect.json"),
        "pet" => Some("pet.json"),
        _ => None,
    }
}

fn widget_metadata(data: &Value) -> serde_json::Map<String, Value> {
    ["id", "name", "description", "width", "height", "theme"]
        .into_iter()
        .filter_map(|key| data.get(key).map(|value| (key.to_string(), value.clone())))
        .collect()
}

fn resource_manifest(kind: &str, entry: &str, data: &Value) -> Result<Value, String> {
    let mut manifest = match kind {
        "wallpaper" => json!({
            "name": text(data, "title")?,
            "description": data["description"].as_str().unwrap_or("")
        }),
        "widget" => Value::Object(widget_metadata(data)),
        _ => json!({}),
    };
    manifest["schemaVersion"] = json!(app_pack::SCHEMA_VERSION);
    manifest["kind"] = json!(kind);
    manifest["entry"] = json!(entry);
    Ok(manifest)
}

fn zip_entries(pack: &Pack) -> Result<(Value, Vec<(String, Vec<u8>)>), String> {
    let data = &pack.data;
    if let Some(entry) = resource_entry(&pack.kind) {
        let manifest = resource_manifest(&pack.kind, entry, data)?;
        let payload = if matches!(pack.kind.as_str(), "effect" | "pet") {
            serde_json::to_vec(data).map_err(|e| e.to_string())?
        } else {
            text(data, "html")?.as_bytes().to_vec()
        };
        let mut files = vec![(entry.into(), payload)];
        if pack.kind == "pet" {
            files.extend(pack.assets.iter().cloned());
        } else if !pack.assets.is_empty() {
            return Err("只有游戏和桌宠包可以带资源文件".into());
        }
        return Ok((manifest, files));
    }
    if pack.kind != "game" { return Err("不支持的资源类型".into()); }

    // Old games still use the manifest understood by their game host.
    let layer = game_packs::Layer::from_value(data)?;
    let version = if pack.assets.is_empty() && !layer.desktop { 1 } else { 2 };
    let mut manifest = json!({
        "schemaVersion": version,
        "kind": "game",
        "id": text(data, "id")?,
        "name": text(data, "name")?,
        "description": text(data, "description")?,
        "save": data["save"].as_bool().unwrap_or(false)
    });
    if let Some(orientation) = game_orientation(data)? { manifest["orientation"] = json!(orientation); }
    if layer.desktop { manifest["layer"] = json!("desktop"); manifest["capabilities"] = data["capabilities"].clone(); }
    let list: Vec<(String, String)> = serde_json::from_value(data["files"].clone()).map_err(|_| "游戏文件清单不合法")?;
    let mut files: Vec<_> = list.into_iter().map(|(path, content)| (path, content.into_bytes())).collect();
    files.extend(pack.assets.iter().cloned());
    Ok((manifest, files))
}

fn kind_label(kind: &str) -> &'static str {
    match kind { "wallpaper" => "壁纸", "effect" => "特效", "widget" => "组件", "pet" => "桌宠", _ => "游戏" }
}

fn to_zip(pack: &Pack) -> Result<Vec<u8>, String> {
    let app_v2 = pack.data["runtimeProtocol"] == 2 && matches!(pack.kind.as_str(), "game" | "app");
    let entries;
    let (manifest, files) = if app_v2 {
        (&pack.data, &pack.assets)
    } else {
        entries = zip_entries(pack)?;
        (&entries.0, &entries.1)
    };
    let manifest_bytes = if app_v2 { serde_json::to_vec(manifest) } else { serde_json::to_vec_pretty(manifest) }
        .map_err(|e| e.to_string())?;
    if !app_v2 {
        if let Some((name, _)) = files.iter().find(|(_, content)| content.len() as u64 > MAX_ENTRY) {
            return Err(format!("文件「{name}」超过 4 MB"));
        }
        let total = files.iter().map(|(_, content)| content.len() as u64).sum::<u64>();
        if total + manifest_bytes.len() as u64 > MAX_PACK {
            return Err("资源包解压后不能超过 32 MB".into());
        }
    }
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut add = |name: &str, bytes: &[u8]| -> Result<(), String> {
        writer.start_file(name, options).map_err(|e| e.to_string())?;
        writer.write_all(bytes).map_err(|e| e.to_string())
    };
    add("manifest.json", &manifest_bytes)?;
    for (name, bytes) in files { add(name, bytes)?; }
    let content = writer.finish().map_err(|e| e.to_string())?.into_inner();
    let limit = if app_v2 { app_pack::MAX_PACK_BYTES } else { MAX_PACK };
    if content.len() as u64 > limit {
        return Err(if app_v2 { "V2 作品 ZIP 不能超过 100 MB" } else { "资源包不能超过 32 MB" }.into());
    }
    Ok(content)
}

fn clean_name(raw: &str) -> Result<String, String> {
    let ok = !raw.starts_with('/') && !raw.contains(['\\', ':', '\0']) && raw.split('/').all(|part| !part.is_empty() && part != "." && part != "..");
    if ok { Ok(raw.to_string()) } else { Err(format!("压缩包里的文件路径不合法：{raw}")) }
}

/// Reads a zip into a pack. Sizes are counted while unpacking, so a small archive cannot expand without limit.
fn from_zip(bytes: &[u8]) -> Result<Pack, String> {
    if bytes.len() as u64 > app_pack::MAX_PACK_BYTES { return Err("资源包不能超过 100 MB".into()); }
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "不是有效的 zip 资源包".to_string())?;
    if archive.len() > app_pack::MAX_ENTRIES { return Err("压缩包里的条目太多".into()); }
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
        let limit = app_pack::MAX_FILE_BYTES.min(app_pack::MAX_PACK_BYTES - total);
        (&mut file).take(limit + 1).read_to_end(&mut content).map_err(|e| format!("无法解压「{name}」：{e}"))?;
        if content.len() as u64 > limit { return Err(format!("文件「{name}」超过单文件 50 MB 或解压合计 100 MB 的上限")); }
        total += content.len() as u64;
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
    let raw_manifest = file("manifest.json").ok_or("压缩包里缺少 manifest.json，它要放在最外层")?;
    if raw_manifest.len() as u64 > MAX_ENTRY { return Err("manifest.json 超过 4 MB".into()); }
    let manifest: Value = serde_json::from_slice(raw_manifest)
        .map_err(|_| "manifest.json 不是有效的 JSON".to_string())?;
    if manifest["runtimeProtocol"] == 2 && matches!(manifest["kind"].as_str(), Some("game" | "app")) {
        if raw_manifest.len() as u64 > app_pack::MAX_MANIFEST_BYTES {
            return Err("manifest.json 超过 64 KB".into());
        }
        app_pack::check_manifest(&manifest)?;
        let files = entries.into_iter().filter(|(name, _)| name != "manifest.json").collect();
        return Ok(Pack { kind: text(&manifest, "kind")?.into(), data: manifest, assets: files });
    }
    if bytes.len() as u64 > MAX_PACK || total > MAX_PACK { return Err("桌面资源包不能超过 32 MB".into()); }
    if archive.len() > MAX_ENTRIES { return Err(format!("压缩包里的文件太多，最多 {MAX_ENTRIES} 个")); }
    if let Some((name, _)) = entries.iter().find(|(_, content)| content.len() as u64 > MAX_ENTRY) {
        return Err(format!("文件「{name}」超过 4 MB"));
    }
    let kind = text(&manifest, "kind")?.to_string();
    let version = manifest.get("schemaVersion").and_then(Value::as_u64);
    let entry = if resource_entry(&kind).is_some() {
        if version != Some(app_pack::SCHEMA_VERSION) {
            return Err("桌面资源包需要 V2 格式（schemaVersion=3）".into());
        }
        if raw_manifest.len() as u64 > app_pack::MAX_MANIFEST_BYTES {
            return Err("manifest.json 超过 64 KB".into());
        }
        if manifest.get("runtimeProtocol").is_some() {
            return Err("runtimeProtocol 只用于游戏和小程序".into());
        }
        let entry = text(&manifest, "entry")?;
        let ext = app_pack::check_file_path(entry)?;
        if matches!(kind.as_str(), "wallpaper" | "widget") {
            if !matches!(ext.as_str(), "html" | "htm") { return Err("entry 需要指向 HTML 文件".into()); }
        } else if ext != "json" {
            return Err("entry 需要指向 JSON 文件".into());
        }
        entry
    } else {
        if !matches!(version, Some(1 | 2)) { return Err("资源包版本暂不支持，请更新小伴桌面".into()); }
        "index.html"
    };
    let payload = || file(entry).ok_or_else(|| format!("压缩包里缺少 {entry}"));
    // Only the listed files are read. Anything else would be dropped without a word, so it is refused instead.
    let allowed = ["manifest.json", entry];
    if matches!(kind.as_str(), "wallpaper" | "widget" | "effect") {
        let extra: Vec<&str> = entries.iter().map(|(name, _)| name.as_str()).filter(|name| !allowed.contains(name)).collect();
        if !extra.is_empty() {
            let hint = if extra.iter().any(|name| name.starts_with(game_packs::ASSET_DIR)) {
                "这个类型还不支持 assets/ 资源文件，图片请用 data: 地址写进页面里"
            } else {
                "这个类型只读取上面这些文件，多余的文件请删掉"
            };
            return Err(format!("{}包里只能有 {}，多出了：{}。{hint}", kind_label(&kind), allowed.join("、"), extra.join("、")));
        }
    }
    let mut assets: Vec<(String, Vec<u8>)> = Vec::new();
    let data = match kind.as_str() {
        "pet" => {
            let mut theme: pet_packs::Theme = serde_json::from_slice(payload()?).map_err(|e| format!("{entry} 格式无效：{e}"))?;
            for (name, content) in entries.iter().filter(|(name, _)| !allowed.contains(&name.as_str())) {
                assets.push((name.clone(), content.clone()));
            }
            pet_packs::import_assets(&mut theme, &assets)?;
            serde_json::to_value(theme).map_err(|e|e.to_string())?
        }
        "wallpaper" => json!({"title": text(&manifest, "name")?, "description": manifest["description"].as_str().unwrap_or(""), "html": utf8(entry, payload()?)?}),
        "effect" => serde_json::from_slice(payload()?).map_err(|_| format!("{entry} 不是有效的 JSON"))?,
        "widget" => {
            let mut data = widget_metadata(&manifest);
            data.insert("schemaVersion".into(), json!(1));
            data.insert("html".into(), json!(utf8(entry, payload()?)?));
            Value::Object(data)
        }
        "game" => {
            let mut files = Vec::new();
            for (name, content) in entries.iter().filter(|(name, _)| name != "manifest.json") {
                if game_packs::is_asset_path(name) { assets.push((name.clone(), content.clone())); } else { files.push((name.clone(), utf8(name, content)?)); }
            }
            files.sort();
            assets.sort();
            if !assets.is_empty() && version == Some(1) { return Err("带 assets/ 资源文件的包要把 manifest.json 的 schemaVersion 写成 2".into()); }
            let layer = game_packs::Layer::from_value(&manifest)?;
            if layer.desktop && version == Some(1) { return Err("桌面层游戏的包要把 manifest.json 的 schemaVersion 写成 2".into()); }
            let mut data = json!({"id": text(&manifest, "id")?, "name": text(&manifest, "name")?, "description": text(&manifest, "description")?, "files": files, "save": manifest["save"].as_bool().unwrap_or(false)});
            if let Some(orientation) = game_orientation(&manifest)? { data["orientation"] = json!(orientation); }
            if layer.desktop { data["layer"] = json!("desktop"); data["capabilities"] = json!(layer.capabilities); }
            data
        }
        _ => return Err("不支持的资源类型".into()),
    };
    Ok(Pack { kind, data, assets })
}

/// Imports a pack into the place its `kind` belongs to, whichever page the file was picked on.
pub fn import(root: &Path, content: &[u8]) -> Result<Value, String> {
    import_pack(root, from_zip(content)?)
}

pub(crate) fn import_effect_zip(root: &Path, path: &Path) -> Result<Value, String> {
    let pack = from_zip(&read_bytes(path, MAX_PACK)?)?;
    if pack.kind != "effect" {
        return Err("请选择特效 ZIP，其他资源包可在「应用 → 导入作品」导入".into());
    }
    import_pack(root, pack)
}

fn import_pack(root: &Path, pack: Pack) -> Result<Value, String> {
    let data = &pack.data;
    if data["runtimeProtocol"] == 2 && matches!(pack.kind.as_str(), "game" | "app") {
        let created = game_packs::create_v2_pack(root, data, &pack.assets, false)?;
        return Ok(json!({"kind":pack.kind,"id":created["id"],"name":created["name"]}));
    }
    let (id, name) = match pack.kind.as_str() {
        "pet" => {
            let theme: pet_packs::Theme = serde_json::from_value(data.clone()).map_err(|e|e.to_string())?;
            pet_packs::save(&root.join("pets/themes"), &theme, false)?;
            (theme.id, theme.name)
        }
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
            let layer = game_packs::Layer::from_value(data)?;
            game_packs::create_oriented_pack_with_assets(root, id, name, text(data, "description")?, &files, &pack.assets, &layer, data["save"].as_bool().unwrap_or(false), false, game_orientation(data)?)?;
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
    let filename = format!("{}-{id}.zip", pack.kind);
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
pub async fn import_resource_pack(
    window: tauri::WebviewWindow,
    allow_html: Option<bool>,
) -> Result<Option<Value>, String> {
    if window.label() != "control" { return Err("只能从控制台导入资源".into()); }
    let app = window.app_handle().clone();
    let allow_html = allow_html.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog().file();
        let dialog = if allow_html {
            dialog.set_title("导入作品").add_filter("作品文件（ZIP、HTML）", &["zip", "html", "htm"])
        } else {
            dialog.set_title("导入小伴资源包").add_filter("小伴资源包（zip）", &["zip"])
        };
        let Some(file) = dialog.blocking_pick_file() else { return Ok(None); };
        let path = file.into_path().map_err(|_| "请选择本机文件")?;
        let pack = pack_from_file(&path, allow_html)?;
        if pack.kind == "pet" {
            let theme: pet_packs::Theme = serde_json::from_value(pack.data).map_err(|e|e.to_string())?;
            let result = json!({"kind":"pet", "id":theme.id, "name":theme.name});
            crate::pets::save_theme(&app, theme, false)?;
            return Ok(Some(result));
        }
        let root = crate::storage::root(&app)?;
        // Wallpaper creation and downloads share the same catalog lock.
        let result = {
            let state = app.state::<crate::content::ContentState>();
            let _guard = state.io.lock().unwrap_or_else(|e| e.into_inner());
            import_pack(&root, pack)?
        };
        if result["kind"] == "wallpaper" {
            let _ = app.emit_to("control", "wallpaper-library-changed", ());
        }
        Ok(Some(result))
    }).await.map_err(|e| e.to_string())?
}

fn pack_from_file(path: &Path, allow_html: bool) -> Result<Pack, String> {
    let html = path.extension().and_then(|ext| ext.to_str())
        .is_some_and(|ext| ["html", "htm"].iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)));
    if html && allow_html {
        let stem = path.file_stem().and_then(|name| name.to_str()).unwrap_or_default();
        let (data, assets) = html_import::make_pack(stem, read_text(path, html_import::MAX_HTML_BYTES)?)?;
        return Ok(Pack { kind: "app".into(), data, assets });
    }
    from_zip(&read_bytes(path, app_pack::MAX_PACK_BYTES)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pet_zip_preserves_images_and_never_imports_instance_layout() {
        let root = temp_root("pet-roundtrip");
        let mut theme = pet_packs::builtin().remove(0);
        theme.id = "shared-garden".into();
        pet_packs::save(&root.join("a/pets/themes"), &theme, false).unwrap();
        let pack=export(&root.join("a"),"pet",&theme.id).unwrap();
        assert_eq!(pack.assets.len(),2);
        let zip=to_zip(&pack).unwrap();
        assert_eq!(zip_manifest(&zip), json!({"schemaVersion":3, "kind":"pet", "entry":"pet.json"}));
        assert_eq!(import(&root.join("b"),&zip).unwrap()["kind"],"pet");
        let restored=pet_packs::load(&root.join("b/pets/themes")).unwrap();
        assert_eq!(serde_json::to_value(&restored[0]).unwrap(),serde_json::to_value(&theme).unwrap());
        assert!(!root.join("b/config/pets.json").exists());
        assert!(import(&root.join("b"),&zip).is_err(),"same-id imports never overwrite");
        assert!(export(&root.join("a"),"pet","flower-whispers").is_err());
        fs::remove_dir_all(root).unwrap();
    }

    fn make_zip(files: &[(&str, &str)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, content) in files {
            writer.start_file(*name, SimpleFileOptions::default()).unwrap();
            writer.write_all(content.as_bytes()).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn zip_manifest(zip: &[u8]) -> Value {
        let mut archive = ZipArchive::new(Cursor::new(zip)).unwrap();
        serde_json::from_reader(archive.by_name("manifest.json").unwrap()).unwrap()
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xiaoban-resource-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn single_html_installs_as_a_portable_v2_app() {
        let root = temp_root("single-html");
        fs::create_dir_all(&root).unwrap();
        let input = root.join("计数 & 练习.HTM");
        let prolog = "\u{feff}<!-- local source --><!DOCTYPE html><html lang='zh'><head data-example='a>b'>";
        let body = "<title>计数</title></head><body><button onclick='this.textContent++'>0</button></body></html>";
        let html = format!("{prolog}{body}");
        fs::write(&input, &html).unwrap();
        assert!(pack_from_file(&input, false).is_err());

        let install = || import_pack(&root, pack_from_file(&input, true).unwrap()).unwrap();
        let imported = install();
        let id = imported["id"].as_str().unwrap();
        assert_ne!(imported["id"], install()["id"]);
        assert_eq!(imported["name"], "计数 & 练习");
        let catalog = game_packs::find_user_game(&root, id).unwrap();
        assert_eq!(catalog["runtimeProtocol"], 2);
        assert_eq!(catalog["kind"], "app");
        assert_eq!(catalog["entry"], "index.html");

        let served = crate::app_runtime::protocol::serve_for(
            &root, &format!("app-runtime-{id}"), &format!("/{id}/index.html"), None,
        );
        assert_eq!(served.status, 200);
        let page = String::from_utf8(served.body).unwrap();
        assert!(page.starts_with(&format!("{prolog}<script>")));
        assert!(page.contains("window.xb?.ready?.()"));
        assert!(page.ends_with(body));
        assert_eq!(fs::read_to_string(&input).unwrap(), html);

        let zip = to_zip(&export(&root, "app", id).unwrap()).unwrap();
        import(&root.join("restored"), &zip).unwrap();
        let (manifest, files) = game_packs::read_v2_pack(&root.join("restored"), id).unwrap();
        assert_eq!(manifest["icon"], "icon.svg");
        assert_eq!(manifest["cover"], "cover.svg");
        assert!(files.iter().any(|(path, bytes)| path == "cover.svg"
            && String::from_utf8_lossy(bytes).contains("计数 &amp; 练习")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn v2_mobile_game_and_desktop_app_keep_their_manifest_and_runtime_files() {
        let root = temp_root("v2-exchange");
        for (id, kind, category, entry) in [
            ("touch-game", "game", "puzzle", "index.html"),
            ("my_tool", "app", "tool", "pages/tool.htm"),
        ] {
            let manifest = json!({
                "schemaVersion":3, "runtimeProtocol":2, "kind":kind, "id":id, "name":"测试作品",
                "description":"可在独立窗口运行", "version":"1.2.0", "category":category,
                "entry":entry, "icon":"icon.svg", "cover":"cover.svg",
                "orientation":"any", "input":["touch","mouse"], "capabilities":["clipboard"],
            });
            let page = "<!doctype html><script>window.xb.ready()</script>";
            let icon = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 192 192"/>"#;
            let cover = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 360"/>"#;
            let zip = make_zip(&[
                ("manifest.json", &manifest.to_string()),
                (entry, page), ("icon.svg", icon), ("cover.svg", cover), ("README.md", "使用说明"),
            ]);
            let imported = import(&root, &zip).unwrap();
            assert_eq!(imported["kind"], kind);
            let catalog = game_packs::find_user_game(&root, id).unwrap();
            assert_eq!(catalog["runtimeProtocol"], 2);
            assert_eq!(catalog["kind"], kind);
            assert_eq!(catalog["entry"], entry);
            assert_eq!(catalog["orientation"], "any");
            let served = crate::app_runtime::protocol::serve_for(
                &root, &format!("app-runtime-{id}"), &format!("/{id}/{entry}"), None,
            );
            assert_eq!(served.status, 200);
            assert_eq!(served.body, page.as_bytes());
            let cover_response = game_packs::serve_from(&root, &format!("/{id}/cover.svg"), None);
            assert_eq!(cover_response.body, cover.as_bytes());

            let exported = to_zip(&export(&root, kind, id).unwrap()).unwrap();
            import(&root.join("roundtrip"), &exported).unwrap();
            let (restored, files) = game_packs::read_v2_pack(&root.join("roundtrip"), id).unwrap();
            assert_eq!(restored, manifest);
            assert!(files.iter().any(|(name, bytes)| name == entry && bytes == page.as_bytes()));

            let mut malformed = manifest.clone();
            malformed["cover"] = json!("../cover.svg");
            assert!(game_packs::create_v2_pack(&root, &malformed, &files, true).is_err());
            assert!(import(&root, &zip).is_err(), "manual imports never overwrite");
            assert_eq!(game_packs::read_v2_pack(&root, id).unwrap().0, manifest);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wallpaper_round_trip_preserves_animation_and_never_applies_it() {
        let root = temp_root("roundtrip");
        let original = local_media::save_local_html(&root.join("a/wallpapers"), "星空", Some("动画"), "<!doctype html><html><body><canvas></canvas></body></html>").unwrap();
        let pack = export(&root.join("a"), "wallpaper", &original.id).unwrap();
        let content = to_zip(&pack).unwrap();
        assert!(content.starts_with(b"PK"), "the pack is an ordinary zip");
        let imported = import(&root.join("b"), &content).unwrap();
        assert_eq!(imported["id"], original.id);
        assert_eq!(imported["kind"], "wallpaper");
        let items = local_media::read(&root.join("b/wallpapers")).unwrap().items;
        assert_eq!(items[0].kind, "web");
        assert_eq!(items[0].resource.sha256, original.resource.sha256);
        assert!(!root.join("b/wallpapers/active.json").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn every_kind_survives_a_zip_round_trip() {
        let widget = json!({"schemaVersion": 1, "id": "tiny-clock", "name": "小时钟", "description": "一个时钟", "width": 200, "height": 120, "theme": "glass", "html": "<div id=t></div><script>t.textContent=1</script>"});
        let effect: Value = serde_json::from_str(effect_packs::EXAMPLE).unwrap();
        let game = json!({"id": "star-catcher", "name": "接星星", "description": "接住星星", "save": true,
            "files": [["game.js", "var a = 1;"], ["index.html", "<script src=\"game.js\"></script>"], ["sub/style.css", "body{}"]]});
        for (kind, data) in [("widget", widget), ("effect", effect), ("game", game)] {
            let pack = Pack { kind: kind.into(), data: data.clone(), assets: Vec::new() };
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
            ("tiny-clock/manifest.json", r#"{"schemaVersion":3,"kind":"widget","entry":"index.html","id":"tiny-clock","name":"小时钟","width":200,"height":120}"#),
            ("tiny-clock/index.html", "<div>12:00</div>"),
            ("__MACOSX/tiny-clock/._index.html", "junk"),
            ("tiny-clock/.DS_Store", "junk"),
            ("tiny-clock/._manifest.json", "junk"),
        ]);
        let done = import(&root, &widget).unwrap();
        assert_eq!(done["id"], "tiny-clock");
        assert!(custom_widgets::find(&root.join("custom_widgets"), "tiny-clock").is_some());

        let effect = make_zip(&[("manifest.json", r#"{"schemaVersion":3,"kind":"effect","entry":"effect.json"}"#), ("effect.json", effect_packs::EXAMPLE)]);
        assert!(import(&root, &effect).is_ok());

        let game = make_zip(&[
            ("manifest.json", r#"{"schemaVersion":1,"kind":"game","id":"star-catcher","name":"接星星","description":"接住星星"}"#),
            ("index.html", "<script src=\"game.js\"></script>"),
            ("game.js", "var a = 1;"),
        ]);
        assert_eq!(import(&root, &game).unwrap()["id"], "star-catcher");

        let wallpaper = make_zip(&[("manifest.json", r#"{"schemaVersion":3,"kind":"wallpaper","entry":"index.html","name":"手作"}"#), ("index.html", "<html><body>hi</body></html>")]);
        let wallpaper = import(&root, &wallpaper).unwrap();
        assert_eq!(wallpaper["name"], "手作");

        for (kind, id, entry) in [
            ("widget", "tiny-clock", "index.html"),
            ("effect", "glitter", "effect.json"),
            ("wallpaper", wallpaper["id"].as_str().unwrap(), "index.html"),
        ] {
            let zip = to_zip(&export(&root, kind, id).unwrap()).unwrap();
            let manifest = zip_manifest(&zip);
            assert_eq!(manifest["schemaVersion"], 3);
            assert_eq!(manifest["kind"], kind);
            assert_eq!(manifest["entry"], entry);
            assert!(manifest.get("runtimeProtocol").is_none());
            let imported = import(&root.join("reimported"), &zip).unwrap();
            assert_eq!(imported["kind"], kind);
            assert_eq!(imported["id"], id);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn effect_file_picker_imports_v2_zip_at_its_declared_entry() {
        let root = temp_root("effect-zip-picker");
        fs::create_dir_all(&root).unwrap();
        let zip = root.join("effect.zip");
        fs::write(&zip, make_zip(&[
            ("manifest.json", r#"{"schemaVersion":3,"kind":"effect","entry":"theme/particles.json"}"#),
            ("theme/particles.json", effect_packs::EXAMPLE),
        ])).unwrap();
        assert_eq!(import_effect_zip(&root, &zip).unwrap()["id"], "glitter");
        assert!(import_effect_zip(&root, &zip).is_err());
        assert_eq!(effect_packs::find(&root.join("effects"), "glitter").unwrap()["schemaVersion"], 1);

        fs::write(&zip, make_zip(&[
            ("manifest.json", r#"{"schemaVersion":3,"kind":"wallpaper","entry":"index.html","name":"星空"}"#),
            ("index.html", "<!doctype html><canvas></canvas>"),
        ])).unwrap();
        assert!(import_effect_zip(&root, &zip).unwrap_err().contains("请选择特效 ZIP"));
        assert!(!root.join("wallpapers").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_v2_checks_entry_type_and_does_not_discard_files() {
        let root = temp_root("desktop-v2-invalid");
        let manifest = json!({"schemaVersion":3, "kind":"widget", "entry":"card.html", "id":"tiny-clock", "name":"小时钟"});
        for (key, value, expected) in [
            ("entry", json!("style.css"), "HTML 文件"),
            ("entry", json!("../card.html"), "路径或类型"),
            ("entry", json!("missing.html"), "缺少 missing.html"),
            ("runtimeProtocol", json!(2), "runtimeProtocol"),
        ] {
            let mut invalid = manifest.clone();
            invalid[key] = value;
            let error = import(&root, &make_zip(&[("manifest.json", &invalid.to_string())])).unwrap_err();
            assert!(error.contains(expected), "{error}");
        }
        let zip = make_zip(&[
            ("manifest.json", &manifest.to_string()),
            ("card.html", "<p>12:00</p>"),
            ("style.css", "p{}"),
        ]);
        assert!(import(&root, &zip).unwrap_err().contains("style.css"));
        assert!(!root.exists());
    }

    // 1x1 pixel images and short headers: enough bytes to pass the type check.
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
    const JPG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0x10];
    const OGG: &[u8] = b"OggS\0\x02";

    fn make_zip_bytes(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, content) in files {
            writer.start_file(*name, SimpleFileOptions::default()).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn game_manifest(version: u32) -> String {
        format!(r#"{{"schemaVersion":{version},"kind":"game","id":"pic-game","name":"图片游戏","description":"带图片和音效"}}"#)
    }

    #[test]
    fn games_carry_pictures_and_sound_and_the_files_come_back_byte_for_byte() {
        let root = temp_root("assets");
        let zip = make_zip_bytes(&[
            ("manifest.json", game_manifest(2).as_bytes()),
            ("index.html", b"<img src=\"assets/hero.png\">"),
            ("assets/hero.png", PNG),
            ("assets/sfx/jump.ogg", OGG),
        ]);
        let done = import(&root, &zip).unwrap();
        assert_eq!((done["kind"].as_str(), done["id"].as_str()), (Some("game"), Some("pic-game")));
        let folder = game_packs::packs_dir(&root).join("pic-game");
        assert_eq!(fs::read(folder.join("assets/hero.png")).unwrap(), PNG);
        assert_eq!(fs::read(folder.join("assets/sfx/jump.ogg")).unwrap(), OGG);
        // The pack protocol hands the pictures out with the right type.
        let served = game_packs::serve_from(&root, "/pic-game/assets/hero.png", None);
        assert_eq!((served.status, served.mime, served.body.as_slice()), (200, "image/png", PNG));

        // Exporting that game again keeps the files, and the pack asks for version 2.
        let exported = export(&root, "game", "pic-game").unwrap();
        assert_eq!(exported.assets.len(), 2);
        let again = to_zip(&exported).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(again.clone())).unwrap();
        let mut manifest = String::new();
        archive.by_name("manifest.json").unwrap().read_to_string(&mut manifest).unwrap();
        assert!(manifest.contains("\"schemaVersion\": 2"), "{manifest}");
        let other = temp_root("assets-again");
        import(&other, &again).unwrap();
        assert_eq!(fs::read(game_packs::packs_dir(&other).join("pic-game/assets/hero.png")).unwrap(), PNG);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(other).unwrap();
    }

    #[test]
    fn a_desktop_game_asks_for_version_two_and_keeps_its_capabilities() {
        let root = temp_root("desktop-game");
        let manifest = |version: u32| format!(r#"{{"schemaVersion":{version},"kind":"game","id":"desk-pal","name":"桌面伙伴","description":"在窗口上走","layer":"desktop","capabilities":["windows","pointer"]}}"#);
        // Older apps would run it as a full-screen game, so the version has to stop them.
        let old = make_zip_bytes(&[("manifest.json", manifest(1).as_bytes()), ("index.html", b"x")]);
        assert!(import(&root, &old).unwrap_err().contains("schemaVersion"));
        assert!(!root.exists());

        let zip = make_zip_bytes(&[("manifest.json", manifest(2).as_bytes()), ("index.html", b"x")]);
        let done = import(&root, &zip).unwrap();
        assert_eq!(done["id"], "desk-pal");
        let entry = game_packs::find_user_game(&root, "desk-pal").unwrap();
        assert_eq!((entry["layer"].as_str(), entry["capabilities"].clone()), (Some("desktop"), json!(["pointer", "windows"])));

        let exported = export(&root, "game", "desk-pal").unwrap();
        let mut archive = ZipArchive::new(Cursor::new(to_zip(&exported).unwrap())).unwrap();
        let mut text = String::new();
        archive.by_name("manifest.json").unwrap().read_to_string(&mut text).unwrap();
        let back: Value = serde_json::from_str(&text).unwrap();
        assert_eq!((back["schemaVersion"].clone(), back["layer"].clone(), back["capabilities"].clone()), (json!(2), json!("desktop"), json!(["pointer", "windows"])));

        let bad = make_zip_bytes(&[("manifest.json", br#"{"schemaVersion":2,"kind":"game","id":"bad-pal","name":"x","description":"y","layer":"desktop","capabilities":["icons"]}"#), ("index.html", b"x")]);
        assert!(import(&temp_root("desktop-game-bad"), &bad).unwrap_err().contains("icons"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_example_desktop_game_in_the_docs_imports_as_one() {
        let root = temp_root("example-desk-pal");
        let zip = make_zip(&[
            ("manifest.json", include_str!("../../docs/examples/desk-pal/manifest.json")),
            ("index.html", include_str!("../../docs/examples/desk-pal/index.html")),
            ("game.js", include_str!("../../docs/examples/desk-pal/game.js")),
        ]);
        let done = import(&root, &zip).unwrap();
        assert_eq!((done["kind"].as_str(), done["id"].as_str()), (Some("game"), Some("desk-pal")));
        let entry = game_packs::find_user_game(&root, "desk-pal").unwrap();
        assert_eq!((entry["layer"].as_str(), entry["capabilities"].clone()), (Some("desktop"), json!(["pointer", "windows"])));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_text_only_pack_stays_at_version_one_so_older_apps_still_open_it() {
        let pack = Pack { kind: "game".into(), data: json!({"id": "plain-game", "name": "纯文本", "description": "没有资源", "files": [["index.html", "x"]], "save": false}), assets: Vec::new() };
        let mut archive = ZipArchive::new(Cursor::new(to_zip(&pack).unwrap())).unwrap();
        let mut manifest = String::new();
        archive.by_name("manifest.json").unwrap().read_to_string(&mut manifest).unwrap();
        assert!(manifest.contains("\"schemaVersion\": 1"), "{manifest}");
    }

    #[test]
    fn mobile_text_game_orientation_survives_import_export_and_reimport() {
        let root = temp_root("mobile-orientation");
        for orientation in ["landscape", "portrait"] {
            // Exact resource-manifest shape generated by MobileGameStore on the phone.
            let manifest = json!({"schemaVersion":1,"kind":"game","id":"touch-runner","name":"触控跑酷","description":"点按跳跃","save":true,"orientation":orientation,"layer":"space"}).to_string();
            let zip = make_zip(&[
                ("manifest.json", &manifest),
                ("index.html", "<!doctype html><script src='game.js'></script>"),
                ("game.js", "let score = 0;"),
                ("cover.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>")
            ]);
            let first = root.join(orientation).join("first");
            import(&first, &zip).unwrap();
            let entry = game_packs::find_user_game(&first, "touch-runner").unwrap();
            assert_eq!(entry["orientation"], orientation);
            assert_eq!(entry["save"], true);
            assert_eq!(entry["cover"], "cover.svg");
            let packed = export(&first, "game", "touch-runner").unwrap();
            let again = to_zip(&packed).unwrap();
            let mut archive = ZipArchive::new(Cursor::new(&again)).unwrap();
            let mut text = String::new();
            archive.by_name("manifest.json").unwrap().read_to_string(&mut text).unwrap();
            let roundtrip: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(roundtrip["schemaVersion"], 1);
            assert_eq!(roundtrip["orientation"], orientation);
            let second = root.join(orientation).join("second");
            import(&second, &again).unwrap();
            assert_eq!(game_packs::find_user_game(&second, "touch-runner").unwrap()["orientation"], orientation);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_mobile_game_orientations_are_rejected_before_writing() {
        let root = temp_root("mobile-bad-orientation");
        for orientation in [json!("diagonal"), json!(false), json!(42)] {
            let manifest = json!({"schemaVersion":1,"kind":"game","id":"touch-runner","name":"触控跑酷","description":"点按跳跃","orientation":orientation}).to_string();
            let zip = make_zip(&[("manifest.json", &manifest), ("index.html", "x")]);
            assert!(import(&root, &zip).unwrap_err().contains("orientation"));
            assert!(!root.exists(), "invalid orientation wrote files");
        }
    }

    #[test]
    fn assets_are_checked_before_anything_is_written() {
        let root = temp_root("bad-assets");
        let big = vec![0x89u8; game_packs::MAX_ASSET_BYTES + 1];
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("pictures need version 2", make_zip_bytes(&[("manifest.json", game_manifest(1).as_bytes()), ("index.html", b"x"), ("assets/a.png", PNG)])),
            ("a jpg that is really a png", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("assets/a.jpg", PNG)])),
            ("a binary outside assets/", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("a.png", PNG)])),
            ("an unsupported type", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("assets/a.exe", b"MZ")])),
            ("an empty file", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("assets/a.png", b"")])),
            ("a file over the size limit", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("assets/a.png", &[PNG, &big].concat())])),
            ("a hidden file", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("assets/.a.png", PNG)])),
            ("the same name twice", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("assets/a.png", PNG), ("assets/A.png", PNG)])),
            ("text that is not utf-8", make_zip_bytes(&[("manifest.json", game_manifest(2).as_bytes()), ("index.html", b"x"), ("game.js", &[0xFF, 0xFE, 0x00])])),
        ];
        for (what, zip) in cases {
            assert!(import(&root, &zip).is_err(), "{what} should be refused");
        }
        assert!(!root.exists(), "nothing is written for a refused pack");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn kinds_that_cannot_carry_files_say_so_instead_of_dropping_them() {
        let root = temp_root("extra-files");
        let widget = r#"{"schemaVersion":3,"kind":"widget","entry":"index.html","id":"tiny-clock","name":"小时钟"}"#;
        let with_picture = make_zip_bytes(&[("manifest.json", widget.as_bytes()), ("index.html", b"<p>x</p>"), ("assets/a.png", PNG)]);
        let error = import(&root, &with_picture).unwrap_err();
        assert!(error.contains("assets/") && error.contains("data:"), "{error}");
        let with_style = make_zip_bytes(&[("manifest.json", widget.as_bytes()), ("index.html", b"<p>x</p>"), ("style.css", b"p{}")]);
        assert!(import(&root, &with_style).unwrap_err().contains("style.css"));
        let effect = make_zip_bytes(&[("manifest.json", br#"{"schemaVersion":3,"kind":"effect","entry":"effect.json"}"#), ("effect.json", effect_packs::EXAMPLE.as_bytes()), ("note.md", b"hi")]);
        assert!(import(&root, &effect).unwrap_err().contains("note.md"));
        assert!(!root.exists());
        fs::remove_dir_all(&root).ok();
        // The page itself still imports, and a picture can ride inside it as a data: address.
        let inline = make_zip_bytes(&[("manifest.json", widget.as_bytes()), ("index.html", b"<img src=\"data:image/png;base64,AAAA\">")]);
        let other = temp_root("extra-files-ok");
        assert!(import(&other, &inline).is_ok());
        fs::remove_dir_all(other).unwrap();
    }

    #[test]
    fn desktop_resource_zips_require_v2_manifest() {
        let root = temp_root("legacy-resource-rejected");
        for kind in ["wallpaper", "effect", "widget", "pet"] {
            for old_version in [1, 2] {
                let manifest = json!({"schemaVersion": old_version, "kind": kind});
                let zip = make_zip(&[("manifest.json", &manifest.to_string())]);
                assert!(import(&root, &zip).unwrap_err().contains("需要 V2 格式"), "{manifest}");
            }
        }
        assert!(!root.exists());
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
        let cases: Vec<Vec<u8>> = vec![
            make_zip(&[("manifest.json", &manifest("game", 3)), ("index.html", "x")]),
            make_zip(&[("manifest.json", &manifest("unknown", 1))]),
            make_zip(&[("manifest.json", &manifest("game", 1)), ("../escape.html", "bad"), ("index.html", "ok")]),
            make_zip(&[("manifest.json", &manifest("game", 1)), ("/abs.html", "bad"), ("index.html", "ok")]),
            make_zip(&[("manifest.json", &manifest("game", 1)), ("a\\b.html", "bad"), ("index.html", "ok")]),
            make_zip(&[("manifest.json", &manifest("game", 1)), ("Index.html", "dup"), ("index.html", "ok")]),
            make_zip(&[("index.html", "no manifest")]),
            make_zip(&[("manifest.json", &manifest("widget", 1))]),
            symlink,
            b"{\"schemaVersion\":1}".to_vec(),
            make_zip(&[("manifest.json", &manifest("game", 1)), ("index.html", "x"), ("big.js", &"a".repeat(MAX_ENTRY as usize + 1))]),
        ];
        for content in cases {
            assert!(import(&root, &content).is_err());
        }
        assert!(!root.exists(), "nothing is written for a refused pack");
    }
}



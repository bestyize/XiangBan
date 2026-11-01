//! Declarative application skins. Packages are data only, never executable UI.
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageFormat, ImageReader, Limits};
use serde_json::{json, Value};
use std::{collections::{HashMap, HashSet}, fs, io::{Cursor, Read}, path::{Path, PathBuf}, sync::Mutex};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use zip::ZipArchive;

const MAX_PACK: u64 = 32 * 1024 * 1024;
const MAX_FILE: u64 = 4 * 1024 * 1024;
const MAX_SKINS: usize = 20;
const BUILTIN_SKINS: &[&str] = &["builtin:spring-river", "builtin:jiangnan", "builtin:seaside-train"];
static LOCK: Mutex<()> = Mutex::new(());
fn valid_id(id: &str) -> bool { !id.is_empty() && id.len() <= 64 && id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') }
fn field<'a>(v: &'a Value, key: &str, max: usize) -> Result<&'a str, String> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)).ok_or_else(|| format!("皮肤字段 {key} 无效"))
}
fn safe_path(name: &str) -> bool {
    !name.is_empty() && name.len() <= 160 && !name.starts_with('/') && !name.contains(['\\', ':', '\0']) && name.split('/').all(|p| !p.is_empty() && p != "." && p != "..")
}
fn image_data(name: &str, bytes: &[u8]) -> Result<String, String> {
    let fmt = image::guess_format(bytes).map_err(|_| format!("图片 {name} 无法识别"))?;
    let mime = match (name.rsplit('.').next().unwrap_or_default().to_ascii_lowercase().as_str(), fmt) {
        ("png", ImageFormat::Png) => "image/png", ("jpg" | "jpeg", ImageFormat::Jpeg) => "image/jpeg", ("webp", ImageFormat::WebP) => "image/webp", _ => return Err(format!("图片 {name} 格式不支持或扩展名不符")),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), fmt);
    let mut limits = Limits::default(); limits.max_image_width = Some(4096); limits.max_image_height = Some(4096); limits.max_alloc = Some(64 * 1024 * 1024); reader.limits(limits);
    let decoded = reader.decode().map_err(|_| format!("图片 {name} 损坏或过大"))?;
    let decoded = if name == "preview.webp" { decoded.thumbnail(512, 512) } else { decoded };
    // Decode and re-encode: animated WebP/APNG cannot execute outside the bounded renderer.
    let mut safe = Cursor::new(Vec::new());
    decoded.write_to(&mut safe, ImageFormat::Png).map_err(|_| "图片转换失败")?;
    let _ = mime;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(safe.into_inner())))
}
fn parse(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() as u64 > MAX_PACK { return Err("皮肤包超过 32 MB".into()); }
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "不是有效的 ZIP")?;
    if zip.len() > 60 { return Err("皮肤包文件过多".into()); }
    let mut files = HashMap::new(); let mut folded = HashSet::new(); let mut total=0u64;
    for i in 0..zip.len() {
        let mut f=zip.by_index(i).map_err(|_| "无法读取 ZIP")?;
        let name=f.name().map_err(|_| "文件名无效")?.to_string();
        if f.is_dir() { continue; }
        if f.is_symlink() || !safe_path(&name) || !folded.insert(name.to_ascii_lowercase()) { return Err("皮肤包包含不安全或重复的路径".into()); }
        let mut data=Vec::new(); (&mut f).take(MAX_FILE+1).read_to_end(&mut data).map_err(|_| "无法解压皮肤")?;
        total+=data.len() as u64; if data.len() as u64 > MAX_FILE || total > MAX_PACK { return Err("皮肤素材过大".into()); }
        files.insert(name,data);
    }
    let parse_json = |name: &str| -> Result<Value,String> { let data=files.get(name).ok_or_else(|| format!("缺少 {name}"))?; if data.len()>128*1024 { return Err(format!("{name} 过大")); } serde_json::from_slice(data).map_err(|_|format!("{name} 格式无效")) };
    let m=parse_json("manifest.json")?;
    if m["kind"] != "application-skin" || m["schemaVersion"] != 1 { return Err("不支持的皮肤类型或版本".into()); }
    if let Some(min)=m.get("minAppVersion") {
        let version=|s:&str| -> Option<(u64,u64,u64)> { let mut p=s.split('.');let value=(p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?);if p.next().is_some(){None}else{Some(value)} };
        let required=version(min.as_str().ok_or("最低版本无效")?).ok_or("最低版本无效")?;
        let current=version(env!("CARGO_PKG_VERSION")).ok_or("应用版本无效")?;
        if required>current {return Err("皮肤需要更新的小伴桌面版本".into());}
    }
    let id=field(&m,"id",64)?; if !valid_id(id) { return Err("皮肤 ID 无效".into()); }
    let name=field(&m,"name",120)?; let version=field(&m,"version",40)?; let author=m["author"].as_str().unwrap_or("");
    if author.len()>120 || author.chars().any(char::is_control) { return Err("作者名无效".into()); }
    if m["scene"] != "scene.json" || m["theme"] != "theme.json" || m["preview"] != "preview.webp" { return Err("皮肤文件入口无效".into()); }
    let theme=parse_json("theme.json")?; if !theme.is_object() { return Err("theme.json 无效".into()); }
    let scene=parse_json("scene.json")?;
    let layers=scene["layers"].as_array().ok_or("缺少图层")?; if layers.is_empty() || layers.len()>20 { return Err("图层数量无效".into()); }
    let mut used=HashSet::from(["manifest.json".to_string(),"theme.json".to_string(),"scene.json".to_string(),"preview.webp".to_string()]);
    let mut ids=HashSet::new(); let mut output=Vec::new(); let mut image_budget=0usize;
    for l in layers {
        let lid=field(l,"id",64)?; if !valid_id(lid) || !ids.insert(lid.to_owned()) { return Err("图层 ID 无效或重复".into()); }
        let path=field(l,"image",160)?; if !safe_path(path) || !path.starts_with("assets/") { return Err("素材路径无效".into()); }
        let depth=l["depth"].as_i64().filter(|n| (0..=10).contains(n)).ok_or("图层深度无效")?;
        let anchor=field(l,"anchor",32)?; if !["center","bottom-left","bottom-right","top-left","top-right"].contains(&anchor) { return Err("图层锚点无效".into()); }
        let url=image_data(path,files.get(path).ok_or("缺少图层素材")?)?; image_budget += url.len(); if image_budget > 16*1024*1024 { return Err("皮肤图层解码后过大".into()); } used.insert(path.to_string());
        output.push(json!({"id":lid,"image":url,"depth":depth,"anchor":anchor,"interactive":l["interactive"].as_bool().unwrap_or(false)}));
    }
    let interactions=scene.get("interactions").and_then(Value::as_array).cloned().unwrap_or_default();
    if interactions.len()>20 { return Err("互动数量过多".into()); }
    for it in &interactions {
        let target=field(it,"target",64)?; if !ids.contains(target) { return Err("互动目标不存在".into()); }
        if !["click","hover","idle"].contains(&field(it,"trigger",16)?) || !["sway","glow"].contains(&field(it,"action",24)?) { return Err("不支持的互动效果".into()); }
        field(it,"label",100)?;
        if it.get("repeat").is_some() && !["restart","ignore"].contains(&field(it,"repeat",16)?) { return Err("互动重复策略无效".into()); }
    }
    if files.keys().any(|name| !used.contains(name)) { return Err("皮肤包含未声明文件".into()); }
    let preview=image_data("preview.webp",files.get("preview.webp").ok_or("缺少预览图")?)?;
    Ok(json!({"id":id,"name":name,"version":version,"author":author,"preview":preview,"scene":{"layers":output,"interactions":interactions}}))
}
fn dir(root: &Path) -> PathBuf { root.join("application-skins") }
fn file(root: &Path,id:&str)->Result<PathBuf,String>{if !valid_id(id) { return Err("皮肤 ID 无效".into()); } Ok(dir(root).join(format!("{id}.zip")))}
fn read(path:&Path)->Result<Vec<u8>,String>{let mut out=Vec::new(); fs::File::open(path).map_err(|e|e.to_string())?.take(MAX_PACK+1).read_to_end(&mut out).map_err(|e|e.to_string())?; if out.len() as u64>MAX_PACK{return Err("皮肤包过大".into())} Ok(out)}
fn listing(root: &Path)->Result<Value,String>{
    fs::create_dir_all(dir(root)).map_err(|e|e.to_string())?;
    let mut active=String::new();
    if let Ok(f)=fs::File::open(dir(root).join("active")) { let _=f.take(128).read_to_string(&mut active); }
    let active=active.trim();
    let mut skins=Vec::new();
    for entry in fs::read_dir(dir(root)).map_err(|e|e.to_string())?.take(MAX_SKINS * 4) {
        let e=entry.map_err(|e|e.to_string())?; let p=e.path();
        if p.extension().is_some_and(|e|e=="zip") && e.file_type().map(|t|t.is_file()).unwrap_or(false) {
            if let Ok(mut s)=read(&p).and_then(|b|parse(&b)) {
                if p.file_stem().and_then(|s|s.to_str())==s["id"].as_str() {
                    if s["id"].as_str()!=Some(active) { s["scene"]=json!({"layers":[],"interactions":[]}); }
                    skins.push(s);
                    if skins.len() == MAX_SKINS { break; }
                }
            }
        }
    }
    skins.sort_by(|a,b|a["name"].as_str().cmp(&b["name"].as_str()));
    let selected=if BUILTIN_SKINS.contains(&active) || skins.iter().any(|s|s["id"]==active) {Some(active)} else {None};
    Ok(json!({"skins":skins,"activeId":selected}))
}
fn save_active(root:&Path, id:Option<&str>)->Result<(),String> {
    let temp=dir(root).join("active.tmp");
    let result=fs::write(&temp,id.unwrap_or("")).and_then(|_|fs::rename(&temp,dir(root).join("active")));
    if result.is_err() { let _=fs::remove_file(&temp); }
    result.map_err(|e|e.to_string())
}
fn ensure_control(w:&tauri::WebviewWindow)->Result<(),String>{if w.label()=="control"{Ok(())}else{Err("只能从控制台管理皮肤".into())}}
#[tauri::command]
pub fn skin_list(window:tauri::WebviewWindow)->Result<Value,String>{ensure_control(&window)?; let _guard=LOCK.lock().unwrap_or_else(|e|e.into_inner()); listing(&crate::storage::root(window.app_handle())?)}
#[tauri::command]
pub fn skin_apply(window:tauri::WebviewWindow,id:Option<String>)->Result<Value,String>{ensure_control(&window)?; let _guard=LOCK.lock().unwrap_or_else(|e|e.into_inner()); let root=crate::storage::root(window.app_handle())?; let state=listing(&root)?;
    if let Some(ref id)=id {if !BUILTIN_SKINS.contains(&id.as_str()) && !state["skins"].as_array().unwrap().iter().any(|s|s["id"]==*id){return Err("皮肤不存在或已损坏".into());}}
    save_active(&root,id.as_deref())?; listing(&root)
}
#[tauri::command]
pub fn skin_delete(window:tauri::WebviewWindow,id:String)->Result<Value,String>{ensure_control(&window)?; let _guard=LOCK.lock().unwrap_or_else(|e|e.into_inner()); let root=crate::storage::root(window.app_handle())?; let path=file(&root,&id)?; if listing(&root)?["activeId"].as_str()==Some(id.as_str()) { save_active(&root,None)?; } fs::remove_file(path).map_err(|e|e.to_string())?; listing(&root)}
#[tauri::command]
pub async fn skin_import(window:tauri::WebviewWindow)->Result<Option<Value>,String>{ensure_control(&window)?;let app=window.app_handle().clone();tauri::async_runtime::spawn_blocking(move||{
    let Some(p)=app.dialog().file().set_title("导入皮肤").add_filter("小伴皮肤",&["zip"]).blocking_pick_file() else{return Ok(None)};let path=p.into_path().map_err(|_|"请选择本机文件")?;let data=read(&path)?;let skin=parse(&data)?;let root=crate::storage::root(&app)?;let _guard=LOCK.lock().unwrap_or_else(|e|e.into_inner());let current=listing(&root)?;if current["skins"].as_array().unwrap().len()>=MAX_SKINS{return Err("已安装皮肤数量达到上限".into())} let path=file(&root,skin["id"].as_str().unwrap())?;if path.exists(){return Err("同名皮肤已存在，请先移除或更换 ID".into())} let temp=path.with_extension("tmp");fs::write(&temp,&data).and_then(|_|fs::rename(&temp,&path)).map_err(|e|e.to_string())?;Ok(Some(skin))
}).await.map_err(|e|e.to_string())?}
#[tauri::command]
pub async fn skin_export(window:tauri::WebviewWindow,id:String)->Result<bool,String>{ensure_control(&window)?;let app=window.app_handle().clone();tauri::async_runtime::spawn_blocking(move||{let root=crate::storage::root(&app)?;let data={let _guard=LOCK.lock().unwrap_or_else(|e|e.into_inner());let bytes=read(&file(&root,&id)?)?;parse(&bytes)?;bytes};let Some(p)=app.dialog().file().set_title("导出皮肤").add_filter("小伴皮肤",&["zip"]).set_file_name(format!("skin-{id}.zip")).blocking_save_file() else{return Ok(false)};let path=p.into_path().map_err(|_|"请选择本机文件")?;fs::write(path,data).map_err(|e|e.to_string())?;Ok(true)}).await.map_err(|e|e.to_string())?}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{ZipWriter, write::SimpleFileOptions};
    fn pack(extra: Option<(&str, &[u8])>, scene: &str) -> Vec<u8> {
        let mut image = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(2,2).write_to(&mut image, ImageFormat::WebP).unwrap();
        let image=image.into_inner();
        let mut z=ZipWriter::new(Cursor::new(Vec::new()));
        for (name,data) in [
            ("manifest.json",br#"{"schemaVersion":1,"kind":"application-skin","id":"demo","name":"Demo","version":"1","preview":"preview.webp","scene":"scene.json","theme":"theme.json"}"#.as_slice()),
            ("scene.json",scene.as_bytes()), ("theme.json",b"{}".as_slice()), ("preview.webp",image.as_slice()), ("assets/a.webp",image.as_slice())
        ] { z.start_file(name,SimpleFileOptions::default()).unwrap(); z.write_all(data).unwrap(); }
        if let Some((name,data))=extra { z.start_file(name,SimpleFileOptions::default()).unwrap();z.write_all(data).unwrap(); }
        z.finish().unwrap().into_inner()
    }
    const SCENE:&str=r#"{"layers":[{"id":"base","image":"assets/a.webp","depth":0,"anchor":"center"}],"interactions":[]}"#;
    #[test] fn validated_package_exposes_only_safe_decoded_data(){let s=parse(&pack(None,SCENE)).unwrap();assert_eq!(s["id"],"demo");assert!(s["scene"]["layers"][0]["image"].as_str().unwrap().starts_with("data:image/png;base64,"));}
    #[test] fn rejects_path_escape_and_undeclared_code(){assert!(parse(&pack(Some(("../evil.js",b"x")),SCENE)).is_err());assert!(parse(&pack(Some(("evil.js",b"x")),SCENE)).is_err());}
    #[test] fn listing_only_exposes_active_scene_and_replaces_selection() {
        let root=std::env::temp_dir().join(format!("xiaoban-skin-test-{}",std::process::id()));
        let _=fs::remove_dir_all(&root);fs::create_dir_all(dir(&root)).unwrap();
        fs::write(file(&root,"demo").unwrap(),pack(None,SCENE)).unwrap();
        assert!(listing(&root).unwrap()["skins"][0]["scene"]["layers"].as_array().unwrap().is_empty());
        save_active(&root,Some("demo")).unwrap();
        assert_eq!(listing(&root).unwrap()["skins"][0]["scene"]["layers"].as_array().unwrap().len(),1);
        for id in BUILTIN_SKINS {
            save_active(&root,Some(id)).unwrap();
            assert_eq!(listing(&root).unwrap()["activeId"],*id);
        }
        save_active(&root,None).unwrap();assert!(listing(&root).unwrap()["activeId"].is_null());
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn rejects_unknown_action_and_external_asset(){let bad=SCENE.replace("assets/a.webp","https://evil/a.webp");assert!(parse(&pack(None,&bad)).is_err()); let bad=r#"{"layers":[{"id":"base","image":"assets/a.webp","depth":0,"anchor":"center"}],"interactions":[{"target":"base","trigger":"click","action":"shell","label":"bad"}]}"#;assert!(parse(&pack(None,bad)).is_err());}
}

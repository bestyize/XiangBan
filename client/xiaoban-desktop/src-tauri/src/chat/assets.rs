//! Conversation-scoped local deliverables and user-selected file/folder references.
//! Importing records a reference, never moves a user's files or sends them to the model.
use super::*;
use agent_kernel::Role;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::path::Component;
use tauri_plugin_dialog::DialogExt;

const MAX_IMPORTS: usize = 500;
const MAX_ENTRIES: usize = 500;
const MAX_IMAGE: u64 = 16 * 1024 * 1024;
static IMPORT_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Imported { path: PathBuf, canonical: PathBuf, added_ms: u64, #[serde(default)] directory: bool }

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    id: String, name: String, path: String, kind: String, exists: bool,
    imported: bool, bytes: u64, updated_ms: u64, tool_call_id: Option<String>,
}

fn control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "control" { Ok(()) } else { Err("请在聊天页管理资产".into()) }
}
fn registry(chat: &Path, session: &str) -> PathBuf { chat.join("assets").join(format!("{session}.json")) }
fn read_imports(path: &Path) -> Result<Vec<Imported>, String> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| "资产关联记录损坏，请保留原文件后重试".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
        Err(e) => Err(format!("无法读取资产关联：{e}")),
    }
}
fn describe(path: &Path, imported: bool, updated: u64, call: Option<String>) -> Asset {
    let meta = fs::metadata(path).ok();
    let folder = meta.as_ref().is_some_and(|m| m.is_dir());
    let image = path.extension().and_then(|e| e.to_str()).is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "png"|"jpg"|"jpeg"|"webp"|"gif"|"bmp"));
    let modified = meta.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|t| t.as_millis() as u64).unwrap_or(0);
    Asset {
        id: format!("{:x}", Sha256::digest(path.to_string_lossy().as_bytes())),
        name: path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned(),
        path: path.to_string_lossy().into_owned(),
        kind: if folder { "folder" } else if image { "image" } else { "file" }.into(),
        exists: meta.as_ref().is_some_and(|m| m.is_file() || m.is_dir()), imported,
        bytes: meta.as_ref().filter(|m| m.is_file()).map(|m| m.len()).unwrap_or(0),
        updated_ms: updated.max(modified), tool_call_id: call,
    }
}

/// Only paths within the saved workspace are inferred from model output. Other locations require a picker.
fn output_path(workspace: &Path, raw: &str) -> Option<PathBuf> {
    let text = raw.trim().trim_matches(['<', '>']);
    if text.is_empty() || text.chars().any(char::is_control) { return None; }
    let raw = if text.starts_with("file://") { tauri::Url::parse(text).ok()?.to_file_path().ok()? } else {
        if text.contains("://") || text.starts_with("data:") || text.starts_with("mailto:") || text.starts_with("sandbox:") { return None; }
        PathBuf::from(text)
    };
    if raw.components().any(|c| matches!(c, Component::ParentDir)) { return None; }
    let path = if raw.is_absolute() { raw } else { workspace.join(raw) };
    if !path.starts_with(workspace) || path == workspace { return None; }
    // Check the nearest existing ancestor too, so missing files through an escaping symlink are refused.
    if let Ok(root) = workspace.canonicalize() {
        let ancestor = path.ancestors().find(|p| p.exists())?.canonicalize().ok()?;
        if !ancestor.starts_with(root) { return None; }
    }
    Some(path)
}

fn switched(message: &Message) -> Option<(PathBuf, PathBuf)> {
    if !message.synthetic || message.role != Role::User { return None; }
    let text = message.text();
    if !text.starts_with(workspace::SWITCH_MARK) { return None; }
    let quoted: Vec<&str> = text.split('“').skip(1).filter_map(|s| s.split('”').next()).take(2).collect();
    if quoted.len() != 2 { return None; }
    let (after, before) = (PathBuf::from(quoted[0]), PathBuf::from(quoted[1]));
    (after.is_absolute() && before.is_absolute()).then_some((after,before))
}
fn outputs(messages: &[Message], workspace: &Path) -> Vec<(PathBuf, Option<String>)> {
    let mut calls = HashMap::new();
    let mut skin_calls = HashMap::new();
    let mut paths = vec![];
    let mut active = messages.iter().find_map(switched).map(|(_,before)| before).unwrap_or_else(||workspace.to_path_buf());
    for message in messages {
        if let Some((after,_)) = switched(message) { active = after; continue; }
        if message.role == Role::Assistant {
            for call in &message.tool_calls {
                if matches!(call.name.as_str(), "skin_create"|"skin_export") {
                    skin_calls.insert(call.id.clone(), active.clone());
                }
                if matches!(call.name.as_str(), "write_file"|"edit_file") {
                    if let Ok(args) = serde_json::from_str::<Value>(&call.arguments) {
                        if let Some(path) = args["path"].as_str().and_then(|p| output_path(&active, p)) { calls.insert(call.id.clone(), path); }
                    }
                }
            }
            // Local Markdown delivery links also cover files/folders produced through shell or other tools.
            let text = message.text();
            for rest in text.split("](").skip(1) {
                let end = if rest.starts_with('<') { rest.find('>').map(|n| n + 1) } else { rest.find(')') };
                if let Some(path) = end.and_then(|end| output_path(&active, &rest[..end])) { paths.push((path, None)); }
            }
        }
        if message.role == Role::Tool && !message.is_error {
            if let Some((id, root)) = message.tool_call_id.as_ref().and_then(|id| skin_calls.remove(id).map(|root| (id.clone(), root))) {
                if let Ok(result) = serde_json::from_str::<Value>(&message.text()) {
                    if let Some(path) = result["packagePath"].as_str().and_then(|p| output_path(&root, p)) {
                        paths.push((path, Some(id)));
                    }
                }
            }
            if let Some((id, path)) = message.tool_call_id.as_ref().and_then(|id| calls.remove(id).map(|path| (id.clone(), path))) { paths.push((path, Some(id))); }
        }
    }
    paths
}

fn collect(chat: &Path, session: &str, workspace: &Path, messages: &[Message]) -> Result<Vec<Asset>, String> {
    let mut rows = HashMap::new();
    for (path, call) in outputs(messages, workspace) {
        let call = call.or_else(|| rows.get(&path).and_then(|a: &Asset| a.tool_call_id.clone()));
        rows.insert(path.clone(), describe(&path, false, 0, call));
    }
    for item in read_imports(&registry(chat, session))? {
        let mut asset = describe(&item.path, true, item.added_ms, None);
        if item.directory { asset.kind = "folder".into(); }
        if item.path.canonicalize().ok().as_ref() != Some(&item.canonical) { asset.exists = false; }
        rows.insert(item.path, asset);
    }
    let folders: Vec<PathBuf> = rows.iter().filter(|(_,a)| a.kind == "folder" && a.exists).map(|(p,_)| p.clone()).collect();
    let mut assets: Vec<Asset> = rows.into_iter().filter(|(p,_)| !folders.iter().any(|f| p != f && p.starts_with(f))).map(|(_,a)| a).collect();
    assets.sort_by(|a,b| b.updated_ms.cmp(&a.updated_ms).then(a.name.cmp(&b.name)));
    Ok(assets)
}

fn context(app: &tauri::AppHandle, session: &str) -> Result<(PathBuf, PathBuf, Vec<Message>), String> {
    checked_session_id(session)?;
    let summary = mutate(app, |index| index.get(session).cloned())?.ok_or("这个对话已经不存在")?;
    let dirs = Dirs::of(app)?;
    let workspace = workspace::of(&summary, &ai_config::current(app)?, &ai_config::default_workspace(app)?);
    let messages = JsonlStore::new(dirs.sessions()).load(session).map_err(|e| e.to_string())?;
    Ok((dirs.chat, workspace, messages))
}
fn list(app: &tauri::AppHandle, session: &str) -> Result<Vec<Asset>, String> {
    let (chat, workspace, messages) = context(app, session)?;
    collect(&chat, session, &workspace, &messages)
}

/// Resolve an opaque asset ID, never a caller-supplied absolute path; directory traversal and symlink escapes fail.
fn within(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let sub = Path::new(relative);
    if sub.components().any(|c| !matches!(c, Component::Normal(_))) { return Err("资产路径无效".into()); }
    let canonical = root.canonicalize().map_err(|_| "原文件已移动或删除")?;
    let child = if relative.is_empty() { canonical.clone() } else { canonical.join(sub).canonicalize().map_err(|_| "原文件已移动或删除")? };
    if !child.starts_with(&canonical) { return Err("无法访问资产文件夹之外的文件".into()); }
    if !child.is_file() && !child.is_dir() { return Err("不是普通文件或文件夹".into()); }
    Ok(child)
}
pub(crate) fn resolve(app: &tauri::AppHandle, session: &str, id: &str, relative: &str) -> Result<PathBuf, String> {
    let asset = list(app, session)?.into_iter().find(|a| a.id == id).ok_or("这份资产已不在当前对话中")?;
    if !asset.exists { return Err("原文件已移动或删除，请定位交付消息".into()); }
    within(Path::new(&asset.path), relative)
}

#[tauri::command]
pub async fn chat_assets_list(window: tauri::WebviewWindow, session_id: String) -> Result<Vec<Asset>, String> {
    control(&window)?; let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || list(&app, &session_id)).await.map_err(|e| e.to_string())?
}

fn associate(chat: &Path, session: &str, paths: Vec<PathBuf>) -> Result<usize, String> {
    let file = registry(chat, session);
    let mut items = read_imports(&file)?;
    let mut count = 0;
    for path in paths {
        let canonical = path.canonicalize().map_err(|_| "选择的文件已不存在")?;
        if !canonical.is_file() && !canonical.is_dir() { return Err("请选择普通文件或文件夹".into()); }
        if items.iter().any(|a| a.canonical == canonical) { continue; }
        if items.len() >= MAX_IMPORTS { return Err("每个对话最多导入 500 份资产".into()); }
        let directory = canonical.is_dir();
        items.push(Imported { path, canonical, directory, added_ms: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }); count += 1;
    }
    if count > 0 { crate::library::save_json(&file, &items)?; }
    Ok(count)
}

pub(super) fn forget(chat: &Path, session: &str) -> Result<(), String> {
    let _guard = lock(&IMPORT_LOCK);
    match fs::remove_file(registry(chat,session)) {
        Ok(()) => Ok(()), Err(e) if e.kind()==std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("无法删除资产关联：{e}")),
    }
}

#[tauri::command]
pub async fn chat_assets_import(window: tauri::WebviewWindow, session_id: String, directory: bool) -> Result<usize, String> {
    control(&window)?; let app = window.app_handle().clone();
    context(&app, &session_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog().file().set_parent(&window);
        let paths = if directory { dialog.set_title("导入资产文件夹").blocking_pick_folder().and_then(|p| p.into_path().ok()).into_iter().collect() }
            else { dialog.set_title("导入对话资产").blocking_pick_files().unwrap_or_default().into_iter().filter_map(|p| p.into_path().ok()).collect() };
        // The picker can outlive a session deletion; do not write an orphaned association.
        let _guard = lock(&IMPORT_LOCK);
        let (chat, _, _) = context(&app, &session_id)?;
        let count = associate(&chat, &session_id, paths)?;
        emit(&app, "chat-assets-changed", json!({"session":session_id}));
        Ok(count)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn chat_asset_children(window: tauri::WebviewWindow, session_id: String, id: String, relative: String) -> Result<Value, String> {
    control(&window)?; let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = resolve(&app, &session_id, &id, &relative)?;
        let mut entries = vec![]; let mut truncated = false;
        for entry in fs::read_dir(&path).map_err(|_| "文件夹无法读取")? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entries.len() >= MAX_ENTRIES { truncated = true; break; }
            // Do not expose symlinks, sockets, devices, or traverse aliases outside this folder.
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if !kind.is_file() && !kind.is_dir() { continue; }
            entries.push(describe(&entry.path(), false, 0, None));
        }
        entries.sort_by(|a,b| (a.kind != "folder").cmp(&(b.kind != "folder")).then(a.name.cmp(&b.name)));
        Ok(json!({"entries":entries,"truncated":truncated}))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn chat_asset_open(window: tauri::WebviewWindow, session_id: String, id: String, relative: String, action: String) -> Result<bool, String> {
    control(&window)?; let app = window.app_handle().clone();
    open_for_preview(app,window,session_id,id,relative,action).await
}
pub(crate) async fn open_for_preview(app:tauri::AppHandle,window:tauri::WebviewWindow,session_id:String,id:String,relative:String,action:String)->Result<bool,String>{
    tauri::async_runtime::spawn_blocking(move || {
        let path = resolve(&app, &session_id, &id, &relative)?;
        match action.as_str() {
            "open" => crate::storage::show_in_file_manager(&path)?,
            "reveal" => crate::storage::show_in_file_manager(if path.is_dir() { &path } else { path.parent().ok_or("无效文件路径")? })?,
            "save" => {
                if !path.is_file() { return Err("请在系统文件管理器中复制文件夹".into()); }
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                let Some(file) = app.dialog().file().set_parent(&window).set_title("资产另存为").set_file_name(name).blocking_save_file() else { return Ok(false); };
                let destination = file.into_path().map_err(|_| "请选择本机文件")?;
                if destination != path && destination.canonicalize().ok().as_ref() != Some(&path) { fs::copy(&path, destination).map_err(|e| format!("保存失败：{e}"))?; }
            }
            _ => return Err("不支持的资产操作".into()),
        }
        Ok(true)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn chat_asset_thumbnail(window: tauri::WebviewWindow, session_id: String, id: String, relative: String) -> Result<String, String> {
    control(&window)?; let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = resolve(&app, &session_id, &id, &relative)?;
        let mut bytes = vec![];
        fs::File::open(path).map_err(|_| "图片读取失败")?.take(MAX_IMAGE + 1).read_to_end(&mut bytes).map_err(|_| "图片读取失败")?;
        if bytes.len() as u64 > MAX_IMAGE { return Err("图片超过 16 MB，请用系统应用打开".into()); }
        let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|_| "图片格式无效")?;
        let mut limits = image::Limits::default(); limits.max_image_width = Some(8192); limits.max_image_height = Some(8192); limits.max_alloc = Some(128 * 1024 * 1024); reader.limits(limits);
        let image = reader.decode().map_err(|_| "图片损坏或尺寸过大，请用系统应用打开")?;
        let mut preview = Cursor::new(Vec::new()); image.thumbnail(384,384).write_to(&mut preview,image::ImageFormat::Png).map_err(|_| "无法生成缩略图")?;
        Ok(format!("data:image/png;base64,{}", STANDARD.encode(preview.into_inner())))
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(PathBuf);
    impl Temp { fn new() -> Self { static NEXT: std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);let p=std::env::temp_dir().join(format!("xb-assets-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));fs::create_dir_all(&p).unwrap(); Self(p) } }
    impl Drop for Temp { fn drop(&mut self) { let _=fs::remove_dir_all(&self.0); } }
    #[test]
    fn imports_persist_deduplicate_and_retain_missing_files() {
        let t=Temp::new();let file=t.0.join("报告.pdf");fs::write(&file,"pdf").unwrap();
        assert_eq!(associate(&t.0,"s",vec![file.clone()]).unwrap(),1);
        assert_eq!(associate(&t.0,"s",vec![file.clone()]).unwrap(),0);
        fs::remove_file(&file).unwrap();let a=collect(&t.0,"s",&t.0,&[]).unwrap();
        assert_eq!(a.len(),1);assert!(a[0].imported);assert!(!a[0].exists);
        fs::write(registry(&t.0,"s"),"broken").unwrap();assert!(associate(&t.0,"s",vec![]).is_err());
    }
    #[test]
    fn only_successful_writes_and_assistant_local_links_become_outputs() {
        let t=Temp::new();let call=|id:&str,name:&str|agent_kernel::ToolCall{id:id.into(),name:name.into(),arguments:r#"{"path":"report.md"}"#.into()};
        let messages=vec![Message::user_text("[私人文件](secret.txt)"),Message::assistant("",vec![call("w","write_file"),call("r","read_file"),call("e","edit_file")]),Message::tool("w","ok"),Message::tool("r","read"),{let mut m=Message::tool("e","failed");m.is_error=true;m},Message::assistant("[文件夹](delivery) [外链](https://example.com) [越界](../secret)",vec![])];
        let out=outputs(&messages,&t.0);assert_eq!(out.len(),2);assert_eq!(out[0].1.as_deref(),Some("w"));assert!(out[1].0.ends_with("delivery"));
    }
    #[test]
    fn skin_zip_deliveries_are_discovered_without_a_final_markdown_link() {
        let t=Temp::new();
        let call=|id:&str,name:&str|agent_kernel::ToolCall{id:id.into(),name:name.into(),arguments:"{}".into()};
        let result=json!({"packagePath":t.0.join("skin-earth.zip").display().to_string()}).to_string();
        let mut failed=Message::tool("failed", &result);failed.is_error=true;
        let messages=vec![Message::assistant("",vec![call("create","skin_create"),call("export","skin_export"),call("read","skin_list"),call("failed","skin_create"),call("outside","skin_export")]),Message::tool("create",&result),Message::tool("export",&result),Message::tool("read",&result),failed,Message::tool("outside",r#"{"packagePath":"../secret.zip"}"#)];
        let out=outputs(&messages,&t.0);
        assert_eq!(out.len(),2);
        assert!(out.iter().all(|(path,_)|path.ends_with("skin-earth.zip")));
        assert_eq!(out[0].1.as_deref(),Some("create"));
        assert_eq!(out[1].1.as_deref(),Some("export"));
    }
    #[test]
    fn folders_own_their_descendants_without_double_counting() {
        let t=Temp::new();let folder=t.0.join("交付");fs::create_dir(&folder).unwrap();fs::write(folder.join("a.txt"),"x").unwrap();
        associate(&t.0,"s",vec![folder.clone(),folder.join("a.txt")]).unwrap();
        assert_eq!(collect(&t.0,"s",&t.0,&[]).unwrap().len(),1);
        assert!(within(&folder,"../a.txt").is_err());assert!(within(&folder,"/a.txt").is_err());assert!(within(&folder,"a.txt").unwrap().is_file());
    }
    #[test]
    fn workspace_changes_keep_earlier_deliveries_and_deletion_only_removes_references() {
        let t=Temp::new();let first=t.0.join("first");let second=t.0.join("second");fs::create_dir(&first).unwrap();fs::create_dir(&second).unwrap();
        let messages=vec![Message::assistant("[早期文件](before.md)",vec![]),Message::reminder(workspace::switch_note(&first,&second)),Message::assistant("[新文件](after.md)",vec![])];
        let paths=outputs(&messages,&second);assert_eq!(paths[0].0,first.join("before.md"));assert_eq!(paths[1].0,second.join("after.md"));
        associate(&t.0,"s",vec![first.clone()]).unwrap();forget(&t.0,"s").unwrap();assert!(first.exists());assert!(read_imports(&registry(&t.0,"s")).unwrap().is_empty());
        fs::remove_dir(&first).unwrap();assert_eq!(outputs(&messages,&second).len(),2);
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_cannot_escape_or_retarget_imported_roots() {
        let t=Temp::new();let root=t.0.join("root");fs::create_dir(&root).unwrap();let outside=t.0.join("private.txt");fs::write(&outside,"secret").unwrap();
        std::os::unix::fs::symlink(&outside,root.join("escape")).unwrap();
        assert!(within(&root,"escape").is_err());assert!(output_path(&root,"escape").is_none());
        let alias=t.0.join("alias");std::os::unix::fs::symlink(&root,&alias).unwrap();associate(&t.0,"s",vec![alias.clone()]).unwrap();
        fs::remove_file(&alias).unwrap();std::os::unix::fs::symlink(&t.0,&alias).unwrap();assert!(!collect(&t.0,"s",&root,&[]).unwrap()[0].exists);
    }
}

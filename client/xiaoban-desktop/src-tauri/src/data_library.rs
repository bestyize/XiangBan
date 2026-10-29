//! 资料库: the user's own documents (copied in and sorted into folders), links to cloud documents, and things saved
//! from answers. Everything lives under `<data root>/data-library`: `library.json` holds the records and
//! `files/<id>/<name>` the copies, so a file the user later moves or deletes stays readable here.
//!
//! Each imported document is read once in the background. What can be read decides whether 小伴 can use it when
//! the user asks with it ("小伴可参考"), and the first lines become its preview.

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager};

/// What the chat composer accepts: a document may be this large, an image smaller.
const MAX_DOCUMENT: u64 = 10 * 1024 * 1024;
const MAX_IMAGE: u64 = 3 * 1024 * 1024;
/// Anything bigger is not copied in at all.
const MAX_IMPORT: u64 = 200 * 1024 * 1024;
const PREVIEW_CHARS: usize = 280;
const MAX_FOLDER_DEPTH: usize = 6;
/// The same document types the chat composer takes.
const OFFICE: &[&str] = &["pdf", "docx", "xlsx", "pptx"];
const TEXT: &[&str] = &["txt", "md", "csv", "tsv", "json", "jsonl", "log", "xml", "yaml", "yml", "toml", "ini", "html", "css", "js", "ts", "tsx", "jsx", "py", "rs", "java", "kt", "swift", "c", "cpp", "h", "sh", "sql"];
const IMAGES: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
    pub created_ms: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Doc {
    pub id: String,
    pub name: String,
    pub folder: Option<String>,
    pub ext: String,
    pub size: u64,
    pub imported_ms: u64,
    /// Where it was copied from, for the user's reference; empty for a file dropped in.
    pub origin: String,
    /// `reading`, `ready` (小伴 can use it), `failed` (no readable text, e.g. a scan), `unsupported` or `tooLarge`.
    pub status: String,
    pub note: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub design_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Link {
    pub id: String,
    pub url: String,
    pub title: String,
    /// `feishu`, `tencent`, `baidu`, `shimo`, `yuque`, `wps` or `other`.
    pub platform: String,
    pub kind: String,
    /// A share link's extraction code (百度网盘 `pwd=`), kept with it.
    pub code: Option<String>,
    pub folder: Option<String>,
    pub added_ms: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Saved {
    pub id: String,
    /// `answer`, manually authored `text`, `file`, `image` or `shot`.
    pub kind: String,
    pub title: String,
    pub text: String,
    pub session_id: Option<String>,
    pub session_title: String,
    /// `main` (the chat page) or `assistant` (超级小伴).
    pub source: String,
    pub saved_ms: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Library {
    pub folders: Vec<Folder>,
    pub docs: Vec<Doc>,
    pub links: Vec<Link>,
    pub saved: Vec<Saved>,
}

/// Serializes every change to `library.json`.
#[derive(Default)]
pub struct LibraryState {
    lock: Mutex<()>,
    counter: AtomicU64,
    /// A background reader is running; another is not started next to it.
    reading: AtomicBool,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or_default()
}

fn new_id(app: &tauri::AppHandle) -> String {
    let n = app.state::<LibraryState>().counter.fetch_add(1, Ordering::SeqCst);
    format!("{:x}{:04x}", now_ms(), n & 0xffff)
}

fn root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("data-library"))
}

fn index_path(root: &Path) -> PathBuf { root.join("library.json") }

fn load(root: &Path) -> Library {
    fs::read(index_path(root)).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
}

fn save(root: &Path, library: &Library) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("资料库目录无法创建：{e}"))?;
    let bytes = serde_json::to_vec_pretty(library).map_err(|e| e.to_string())?;
    let temp = root.join("library.json.tmp");
    fs::write(&temp, bytes).map_err(|e| format!("资料库无法保存：{e}"))?;
    fs::rename(&temp, index_path(root)).map_err(|e| format!("资料库无法保存：{e}"))
}

/// Reads, changes and writes the library under the lock, and tells the console it changed.
fn change<T>(app: &tauri::AppHandle, edit: impl FnOnce(&mut Library, &Path) -> Result<T, String>) -> Result<(T, Library), String> {
    let state = app.state::<LibraryState>();
    let _guard = state.lock.lock().unwrap_or_else(|e| e.into_inner());
    let root = root(app)?;
    let mut library = load(&root);
    let value = edit(&mut library, &root)?;
    save(&root, &library)?;
    let _ = app.emit_to("control", "library-changed", ());
    Ok((value, library))
}

fn control_only(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" { return Err("请在主窗口的资料库里操作".into()); }
    Ok(())
}

fn clean_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 { return Err("名称需要 1 到 120 个字".into()); }
    if name.contains(['/', '\\', '\0']) || name == "." || name == ".." { return Err("名称里不能有 / 或 \\".into()); }
    Ok(name.to_string())
}

fn ext_of(name: &str) -> String {
    Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}

/// Whether 小伴 can take the document along when asked with it, before reading it.
fn readable_kind(ext: &str, size: u64) -> Option<(&'static str, &'static str)> {
    let image = IMAGES.contains(&ext);
    if !image && !OFFICE.contains(&ext) && !TEXT.contains(&ext) { return Some(("unsupported", "这种文件暂时不能给小伴参考")); }
    if size > if image { MAX_IMAGE } else { MAX_DOCUMENT } {
        return Some(("tooLarge", if image { "图片超过 3MB，提问时带不上" } else { "超过 10MB，提问时带不上" }));
    }
    None
}

fn folder_exists(library: &Library, id: &Option<String>) -> Result<(), String> {
    match id {
        Some(id) if !library.folders.iter().any(|f| &f.id == id) => Err("这个文件夹已经不在了".into()),
        _ => Ok(()),
    }
}

fn depth<'a>(library: &'a Library, mut id: Option<&'a str>) -> usize {
    let mut n = 0;
    while let Some(current) = id {
        n += 1;
        if n > 64 { break; }
        id = library.folders.iter().find(|f| f.id == current).and_then(|f| f.parent.as_deref());
    }
    n
}

/// `folder` and everything inside it.
fn subtree(library: &Library, folder: &str) -> Vec<String> {
    let mut out = vec![folder.to_string()];
    let mut i = 0;
    while i < out.len() {
        let parent = out[i].clone();
        out.extend(library.folders.iter().filter(|f| f.parent.as_deref() == Some(&parent)).map(|f| f.id.clone()));
        i += 1;
    }
    out
}

fn add_folder(app: &tauri::AppHandle, library: &mut Library, name: &str, parent: Option<String>) -> Result<String, String> {
    let name = clean_name(name)?;
    folder_exists(library, &parent)?;
    if depth(library, parent.as_deref()) >= MAX_FOLDER_DEPTH { return Err("文件夹最多套 6 层".into()); }
    // The same name twice in one place would only confuse: reuse it (an imported folder merges in).
    if let Some(existing) = library.folders.iter().find(|f| f.parent == parent && f.name == name) { return Ok(existing.id.clone()); }
    let id = new_id(app);
    library.folders.push(Folder { id: id.clone(), name, parent, created_ms: now_ms() });
    Ok(id)
}

/// Copies one file in. The copy keeps the original name so 小伴 and the file manager show it as the user knows it.
fn copy_in(app: &tauri::AppHandle, library: &mut Library, root: &Path, name: &str, bytes: Option<&[u8]>, source: Option<&Path>, folder: Option<String>) -> Result<String, String> {
    let name = clean_name(name)?;
    let size = match (bytes, source) {
        (Some(bytes), _) => bytes.len() as u64,
        (None, Some(path)) => fs::metadata(path).map_err(|e| format!("{name}：{e}"))?.len(),
        _ => return Err("没有文件内容".into()),
    };
    if size > MAX_IMPORT { return Err(format!("{name} 超过 200MB，没有导入")); }
    let id = new_id(app);
    let dir = root.join("files").join(&id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join(&name);
    match (bytes, source) {
        (Some(bytes), _) => fs::write(&target, bytes),
        (None, Some(path)) => fs::copy(path, &target).map(|_| ()),
        _ => unreachable!(),
    }.map_err(|e| format!("{name} 复制失败：{e}"))?;
    let ext = ext_of(&name);
    let (status, note) = readable_kind(&ext, size).unwrap_or(("reading", ""));
    library.docs.push(Doc {
        id: id.clone(), name, folder, ext, size, imported_ms: now_ms(),
        origin: source.map(|p| p.display().to_string()).unwrap_or_default(),
        status: status.into(), note: note.into(), preview: String::new(), tags: Vec::new(), design_id: None,
    });
    Ok(id)
}

/// Imports files, and folders with their layout, into `folder`. Hidden files and links are skipped.
fn import_paths(app: &tauri::AppHandle, library: &mut Library, root: &Path, paths: &[PathBuf], folder: Option<String>, errors: &mut Vec<String>, level: usize) {
    for path in paths {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if name.starts_with('.') { continue; }
        let Ok(meta) = fs::symlink_metadata(path) else { errors.push(format!("{name} 读取失败")); continue };
        if meta.file_type().is_symlink() { continue; }
        if meta.is_dir() {
            if level >= MAX_FOLDER_DEPTH { errors.push(format!("{name} 层级太深，没有继续导入")); continue; }
            match add_folder(app, library, name, folder.clone()) {
                Ok(id) => {
                    let mut children: Vec<PathBuf> = fs::read_dir(path).map(|r| r.flatten().map(|e| e.path()).collect()).unwrap_or_default();
                    children.sort();
                    import_paths(app, library, root, &children, Some(id), errors, level + 1);
                }
                Err(error) => errors.push(format!("{name}：{error}")),
            }
        } else if let Err(error) = copy_in(app, library, root, name, None, Some(path), folder.clone()) {
            errors.push(error);
        }
    }
}

/// Reads the text of each document still marked `reading`, one at a time, off the main thread.
fn read_pending(app: &tauri::AppHandle) {
    if app.state::<LibraryState>().reading.swap(true, Ordering::SeqCst) { return; }
    let app = app.clone();
    std::thread::spawn(move || {
        read_all(&app);
        app.state::<LibraryState>().reading.store(false, Ordering::SeqCst);
    });
}

fn read_all(app: &tauri::AppHandle) {
    loop {
        let Ok(root) = root(app) else { return };
        let Some(doc) = load(&root).docs.into_iter().find(|d| d.status == "reading") else { return };
        let path = root.join("files").join(&doc.id).join(&doc.name);
        let (status, note, preview) = read_text(&path, &doc.ext);
        let result = change(app, |library, _| {
            if let Some(entry) = library.docs.iter_mut().find(|d| d.id == doc.id) {
                entry.status = status.into(); entry.note = note; entry.preview = preview;
            }
            Ok(())
        });
        if result.is_err() { return; }
    }
}

fn read_text(path: &Path, ext: &str) -> (&'static str, String, String) {
    if IMAGES.contains(&ext) { return ("ready", String::new(), String::new()); }
    let text = if OFFICE.contains(&ext) {
        agent_file_tools::extract_document_text(path, ext, &agent_kernel::ToolCtx::detached("library"))
    } else {
        fs::read(path).map(|bytes| String::from_utf8_lossy(&bytes).into_owned()).map_err(|e| e.to_string())
    };
    match text {
        Ok(text) => {
            let preview = preview_of(&text);
            if preview.is_empty() { ("failed", "里面没有可读的文字，可能是扫描件".into(), String::new()) } else { ("ready", String::new(), preview) }
        }
        Err(error) if error.contains("no extractable text") => ("failed", "里面没有可读的文字，可能是扫描件".into(), String::new()),
        Err(error) if error.contains("encrypted") || error.contains("malformed") => ("failed", "文件加密或已损坏，读不出内容".into(), String::new()),
        Err(_) => ("failed", "读不出内容".into(), String::new()),
    }
}

/// The opening of a document, on one line, without the extractor's own notes.
fn preview_of(text: &str) -> String {
    let body = text.lines().filter(|line| !line.trim_start().starts_with("[PDF text only")).collect::<Vec<_>>().join(" ");
    body.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(PREVIEW_CHARS).collect()
}

/// Which service a link belongs to and what kind of thing it opens, from its address alone.
pub fn detect(url: &str) -> Result<(String, String, Option<String>), String> {
    let url = url.trim();
    if url.len() > 2048 { return Err("链接太长了".into()); }
    let rest = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).ok_or("请粘贴以 http:// 或 https:// 开头的链接")?;
    let (host, path) = rest.split_once('/').map(|(h, p)| (h, format!("/{p}"))).unwrap_or((rest, "/".into()));
    let host = host.split('@').last().unwrap_or(host).split(':').next().unwrap_or(host).to_ascii_lowercase();
    if host.is_empty() || !host.contains('.') { return Err("这不像一个网址".into()); }
    let ends = |domain: &str| host == domain || host.ends_with(&format!(".{domain}"));
    let has = |part: &str| path.contains(part);
    let code = path.split(['?', '&']).find_map(|p| p.strip_prefix("pwd=")).map(|c| c.chars().take_while(|c| c.is_ascii_alphanumeric()).collect::<String>()).filter(|c| !c.is_empty());
    let (platform, kind) = if ends("feishu.cn") || ends("larksuite.com") || ends("larkoffice.com") {
        ("feishu", if has("/sheets/") { "表格" } else if has("/base/") { "多维表格" } else if has("/wiki/") { "知识库" } else if has("/drive/folder") { "文件夹" } else if has("/minutes/") { "妙记" } else { "文档" })
    } else if ends("docs.qq.com") {
        ("tencent", if has("/sheet/") { "在线表格" } else if has("/slide/") { "幻灯片" } else if has("/form/") { "收集表" } else if has("/folder/") { "文件夹" } else { "文档" })
    } else if ends("pan.baidu.com") {
        ("baidu", "网盘分享")
    } else if ends("shimo.im") {
        ("shimo", "文档")
    } else if ends("yuque.com") {
        ("yuque", "知识库")
    } else if ends("kdocs.cn") || ends("wps.cn") {
        ("wps", "文档")
    } else {
        ("other", "网页")
    };
    Ok((platform.into(), kind.into(), code))
}

#[tauri::command]
pub fn library_state(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Library, String> {
    control_only(&window)?;
    let root = root(&app)?;
    // Recover a source commit whose indexing was interrupted, without duplicating documents.
    for p in crate::design::store(&app)?.list("*")? { upsert_design(&app,&p)?; }
    let library = load(&root);
    // A reading interrupted by quitting picks up where it stopped.
    if library.docs.iter().any(|d| d.status == "reading") { read_pending(&app); }
    Ok(library)
}

/// Opens the system picker for files (or a folder) and imports what was chosen into `folder`.
#[tauri::command]
pub async fn library_pick_import(app: tauri::AppHandle, window: tauri::WebviewWindow, folder: Option<String>, directory: bool) -> Result<Value, String> {
    control_only(&window)?;
    use tauri_plugin_dialog::DialogExt;
    let picker = app.clone();
    let parent = window.clone();
    let paths: Vec<PathBuf> = tauri::async_runtime::spawn_blocking(move || {
        let dialog = picker.dialog().file().set_parent(&parent);
        if directory {
            dialog.set_title("选择要放进资料库的文件夹").blocking_pick_folder().and_then(|p| p.into_path().ok()).into_iter().collect()
        } else {
            dialog.set_title("选择要放进资料库的文件").blocking_pick_files().unwrap_or_default().into_iter().filter_map(|p| p.into_path().ok()).collect()
        }
    }).await.map_err(|e| e.to_string())?;
    if paths.is_empty() { return Ok(json!({ "library": load(&root(&app)?), "imported": 0, "errors": [] })); }
    let mut errors = Vec::new();
    let (before, library) = change(&app, |library, root| {
        folder_exists(library, &folder)?;
        let before = library.docs.len();
        import_paths(&app, library, root, &paths, folder.clone(), &mut errors, depth(library, folder.as_deref()));
        Ok(before)
    })?;
    read_pending(&app);
    Ok(json!({ "imported": library.docs.len() - before, "errors": errors, "library": library }))
}

/// A file dropped onto the page: the page has its bytes but not its path.
#[tauri::command]
pub fn library_import_data(app: tauri::AppHandle, window: tauri::WebviewWindow, name: String, data_url: String, folder: Option<String>) -> Result<Library, String> {
    control_only(&window)?;
    let encoded = data_url.split_once(',').map(|(_, data)| data).ok_or("文件内容不对")?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(encoded.trim()).map_err(|_| "文件内容不对")?;
    let (_, library) = change(&app, |library, root| {
        folder_exists(library, &folder)?;
        copy_in(&app, library, root, &name, Some(&bytes), None, folder.clone())
    })?;
    read_pending(&app);
    Ok(library)
}

#[tauri::command]
pub fn library_create_folder(app: tauri::AppHandle, window: tauri::WebviewWindow, name: String, parent: Option<String>) -> Result<Value, String> {
    control_only(&window)?;
    let (id, library) = change(&app, |library, _| add_folder(&app, library, &name, parent))?;
    Ok(json!({ "id": id, "library": library }))
}

#[tauri::command]
pub fn library_rename_folder(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String, name: String) -> Result<Library, String> {
    control_only(&window)?;
    change(&app, |library, _| {
        let name = clean_name(&name)?;
        let parent = library.folders.iter().find(|f| f.id == id).ok_or("这个文件夹已经不在了")?.parent.clone();
        if library.folders.iter().any(|f| f.id != id && f.parent == parent && f.name == name) { return Err("这里已经有同名的文件夹".into()); }
        if let Some(folder) = library.folders.iter_mut().find(|f| f.id == id) { folder.name = name; }
        Ok(())
    }).map(|(_, library)| library)
}

/// Deletes a folder but nothing in it: its files, links and folders move up one level.
#[tauri::command]
pub fn library_delete_folder(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<Library, String> {
    control_only(&window)?;
    change(&app, |library, _| {
        let parent = library.folders.iter().find(|f| f.id == id).ok_or("这个文件夹已经不在了")?.parent.clone();
        for folder in library.folders.iter_mut().filter(|f| f.parent.as_deref() == Some(&id)) { folder.parent = parent.clone(); }
        for doc in library.docs.iter_mut().filter(|d| d.folder.as_deref() == Some(&id)) { doc.folder = parent.clone(); }
        for link in library.links.iter_mut().filter(|l| l.folder.as_deref() == Some(&id)) { link.folder = parent.clone(); }
        library.folders.retain(|f| f.id != id);
        Ok(())
    }).map(|(_, library)| library)
}

/// Moves documents, links or folders into `folder` (`None`: 未分组 / the top level).
#[tauri::command]
pub fn library_move(app: tauri::AppHandle, window: tauri::WebviewWindow, ids: Vec<String>, folder: Option<String>) -> Result<Library, String> {
    control_only(&window)?;
    change(&app, |library, _| {
        folder_exists(library, &folder)?;
        for id in &ids {
            if library.folders.iter().any(|f| &f.id == id) {
                // A folder cannot go inside itself or one of its own folders.
                if folder.as_ref().is_some_and(|target| subtree(library, id).contains(target)) { return Err("不能把文件夹移到它自己里面".into()); }
                if let Some(f) = library.folders.iter_mut().find(|f| &f.id == id) { f.parent = folder.clone(); }
            } else if let Some(doc) = library.docs.iter_mut().find(|d| &d.id == id) {
                doc.folder = folder.clone();
            } else if let Some(link) = library.links.iter_mut().find(|l| &l.id == id) {
                link.folder = folder.clone();
            }
        }
        Ok(())
    }).map(|(_, library)| library)
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct DocPatch { name: Option<String>, tags: Option<Vec<String>> }

#[tauri::command]
pub fn library_update_doc(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String, patch: DocPatch) -> Result<Library, String> {
    control_only(&window)?;
    change(&app, |library, root| {
        let doc = library.docs.iter_mut().find(|d| d.id == id).ok_or("这份文档已经不在了")?;
        if let Some(name) = patch.name {
            let name = clean_name(&name)?;
            if let Some(id)=&doc.design_id {
                let store=crate::design::store(&app)?;let mut p=store.load(id)?;let rev=p.revision;p.title=name.clone();let p=store.save(p,rev)?;doc.name=name;doc.imported_ms=p.updated_ms;
                doc.preview=design_preview(&p);
                let _=app.emit("design-changed",open_claude_design::summary(&p));
            } else {
            // The copy keeps its extension, so it still opens with the same app.
            let name = if ext_of(&name) == doc.ext || doc.ext.is_empty() { name } else { format!("{name}.{}", doc.ext) };
            let dir = root.join("files").join(&doc.id);
            fs::rename(dir.join(&doc.name), dir.join(&name)).map_err(|e| format!("改名失败：{e}"))?;
            doc.name = name;
            }
        }
        if let Some(tags) = patch.tags {
            let mut clean: Vec<String> = tags.into_iter().map(|t| t.trim().chars().take(20).collect::<String>()).filter(|t| !t.is_empty()).collect();
            clean.dedup();
            clean.truncate(8);
            doc.tags = clean;
        }
        Ok(())
    }).map(|(_, library)| library)
}

/// Removes documents (and their copies), links or saved items.
#[tauri::command]
pub fn library_remove(app: tauri::AppHandle, window: tauri::WebviewWindow, ids: Vec<String>) -> Result<Library, String> {
    control_only(&window)?;
    change(&app, |library, root| {
        for doc in library.docs.iter().filter(|d| ids.contains(&d.id)) {
            if let Some(id)=&doc.design_id { crate::design::store(&app)?.remove(id)?;let _=app.emit("design-deleted",serde_json::json!({"id":id})); }
            else { let _ = fs::remove_dir_all(root.join("files").join(&doc.id)); }
        }
        library.docs.retain(|d| !ids.contains(&d.id));
        library.links.retain(|l| !ids.contains(&l.id));
        library.saved.retain(|s| !ids.contains(&s.id));
        Ok(())
    }).map(|(_, library)| library)
}

/// Shows the copy in Finder or Explorer.
#[tauri::command]
pub fn library_reveal(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    control_only(&window)?;
    let root = root(&app)?;
    let doc = load(&root).docs.into_iter().find(|d| d.id == id).ok_or("这份文档已经不在了")?;
    crate::storage::show_in_file_manager(&root.join("files").join(&doc.id))
}

/// The document as a chat attachment, for "用这份资料提问".
#[tauri::command]
pub fn library_attachment(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<Value, String> {
    control_only(&window)?;
    let root = root(&app)?;
    let doc = load(&root).docs.into_iter().find(|d| d.id == id).ok_or("这份文档已经不在了")?;
    if doc.design_id.is_some() { return Err("请打开设计画布继续修改这份作品".into()); }
    if let Some((_, note)) = readable_kind(&doc.ext, doc.size) { return Err(note.into()); }
    let bytes = fs::read(root.join("files").join(&doc.id).join(&doc.name)).map_err(|e| format!("读取失败：{e}"))?;
    let image = IMAGES.contains(&doc.ext.as_str());
    let mime = match doc.ext.as_str() {
        "png" => "image/png", "jpg" | "jpeg" => "image/jpeg", "gif" => "image/gif", "webp" => "image/webp",
        "pdf" => "application/pdf", "md" => "text/markdown", "csv" => "text/csv", "json" => "application/json",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "text/plain",
    };
    let data = format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes));
    Ok(if image { json!({ "image": data }) } else { json!({ "name": doc.name, "size": doc.size, "mime": mime, "dataUrl": data }) })
}

#[tauri::command]
pub fn library_add_link(app: tauri::AppHandle, window: tauri::WebviewWindow, url: String, title: String, folder: Option<String>) -> Result<Library, String> {
    control_only(&window)?;
    let (platform, kind, code) = detect(&url)?;
    let title = title.trim();
    if title.chars().count() > 120 { return Err("标题最多 120 个字".into()); }
    change(&app, |library, _| {
        folder_exists(library, &folder)?;
        let url = url.trim().to_string();
        if library.links.iter().any(|l| l.url == url) { return Err("这个链接已经在资料库里了".into()); }
        let title = if title.is_empty() { kind.clone() } else { title.to_string() };
        library.links.insert(0, Link { id: new_id(&app), url, title, platform, kind, code, folder, added_ms: now_ms() });
        Ok(())
    }).map(|(_, library)| library)
}

#[tauri::command]
pub fn library_rename_link(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String, title: String) -> Result<Library, String> {
    control_only(&window)?;
    let title = clean_name(&title)?;
    change(&app, |library, _| {
        library.links.iter_mut().find(|l| l.id == id).ok_or("这个链接已经不在了")?.title = title;
        Ok(())
    }).map(|(_, library)| library)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveInput {
    kind: String,
    title: String,
    text: String,
    session_id: Option<String>,
    #[serde(default)]
    session_title: String,
}

/// "存到资料库" under an answer, from the chat page or from 超级小伴.
#[tauri::command]
pub fn library_save(app: tauri::AppHandle, window: tauri::WebviewWindow, item: SaveInput) -> Result<String, String> {
    let source = match window.label() { "control" => "main", crate::float_assistant::LABEL => "assistant", _ => return Err("这个窗口不能存到资料库".into()) };
    save_item(&app, source, item)
}

/// 手机上点“存到资料库”：只存回答文字，来源记为 phone。
pub(crate) fn remote_save(app: &tauri::AppHandle, title: &str, text: &str, session_id: Option<String>, session_title: &str) -> Result<String, String> {
    save_item(app, "phone", SaveInput { kind: "answer".into(), title: title.into(), text: text.into(), session_id, session_title: session_title.into() })
}

fn save_item(app: &tauri::AppHandle, source: &str, item: SaveInput) -> Result<String, String> {
    if !["answer", "text", "file", "image", "shot"].contains(&item.kind.as_str()) { return Err("不支持这种收藏".into()); }
    let text = item.text.trim().to_string();
    if text.is_empty() || text.len() > 200_000 { return Err("要收藏的内容是空的或太长".into()); }
    let title: String = if item.title.trim().is_empty() { text.lines().next().unwrap_or("").chars().take(40).collect() } else { item.title.trim().chars().take(80).collect() };
    let (id, _) = change(app, |library, _| {
        // Saving the same answer twice keeps one.
        if let Some(existing) = library.saved.iter().find(|s| s.kind == item.kind && s.text == text && s.session_id == item.session_id) { return Ok(existing.id.clone()); }
        let id = new_id(app);
        library.saved.insert(0, Saved { id: id.clone(), kind: item.kind, title, text, session_id: item.session_id, session_title: item.session_title.chars().take(80).collect(), source: source.into(), saved_ms: now_ms() });
        Ok(id)
    })?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_recognised_by_service_and_kind_and_keep_the_extraction_code() {
        assert_eq!(detect("https://example.feishu.cn/docx/Q4okr2026").unwrap(), ("feishu".into(), "文档".into(), None));
        assert_eq!(detect("https://abc.feishu.cn/base/bascn123?table=1").unwrap().1, "多维表格");
        assert_eq!(detect("https://my.larksuite.com/wiki/xyz").unwrap(), ("feishu".into(), "知识库".into(), None));
        assert_eq!(detect("https://docs.qq.com/sheet/DSkFt").unwrap(), ("tencent".into(), "在线表格".into(), None));
        assert_eq!(detect("https://pan.baidu.com/s/1AbC?pwd=x7k2").unwrap(), ("baidu".into(), "网盘分享".into(), Some("x7k2".into())));
        assert_eq!(detect("https://www.yuque.com/team/handbook").unwrap().0, "yuque");
        assert_eq!(detect("http://example.com/page").unwrap(), ("other".into(), "网页".into(), None));
        // A lookalike host is not the service.
        assert_eq!(detect("https://feishu.cn.evil.example/docx/1").unwrap().0, "other");
        assert!(detect("ftp://example.com").is_err() && detect("https://nohost/").is_err() && detect("飞书文档").is_err());
    }

    #[test]
    fn what_small_can_read_is_decided_by_type_and_size() {
        assert_eq!(readable_kind("pdf", 2 * 1024 * 1024), None);
        assert_eq!(readable_kind("md", 1), None);
        assert_eq!(readable_kind("png", 4 * 1024 * 1024).unwrap().0, "tooLarge");
        assert_eq!(readable_kind("pdf", 11 * 1024 * 1024).unwrap().0, "tooLarge");
        assert_eq!(readable_kind("dmg", 1).unwrap().0, "unsupported");
        assert_eq!(preview_of("[PDF text only; scanned pages…]\n  全年预算\n\n1,280 万  "), "全年预算 1,280 万");
    }

    #[test]
    fn folders_nest_merge_by_name_and_never_go_inside_themselves() {
        let mut library = Library::default();
        library.folders = vec![
            Folder { id: "work".into(), name: "工作".into(), parent: None, created_ms: 0 },
            Folder { id: "fin".into(), name: "财务".into(), parent: Some("work".into()), created_ms: 0 },
            Folder { id: "q4".into(), name: "四季度".into(), parent: Some("fin".into()), created_ms: 0 },
        ];
        assert_eq!(subtree(&library, "work"), vec!["work", "fin", "q4"]);
        assert_eq!(depth(&library, Some("q4")), 3);
        assert!(clean_name("a/b").is_err() && clean_name("  ").is_err() && clean_name("..").is_err());
        assert_eq!(clean_name(" 报销 ").unwrap(), "报销");
    }

    #[test]
    fn text_files_are_read_and_a_scan_like_empty_file_is_said_to_have_no_text() {
        let dir = std::env::temp_dir().join(format!("xiaoban-library-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let note = dir.join("品牌规范.md");
        fs::write(&note, "# 品牌规范\n主色小伴红").unwrap();
        assert_eq!(read_text(&note, "md"), ("ready", String::new(), "# 品牌规范 主色小伴红".into()));
        let blank = dir.join("blank.txt");
        fs::write(&blank, "   \n").unwrap();
        assert_eq!(read_text(&blank, "txt").0, "failed");
        let broken = dir.join("broken.pdf");
        fs::write(&broken, "not a pdf").unwrap();
        assert_eq!(read_text(&broken, "pdf").0, "failed");
        let _ = fs::remove_dir_all(&dir);
    }
}

/// Project source is durable before this index. Startup reconciliation repairs an interrupted update.
pub fn upsert_design(app:&tauri::AppHandle, project:&open_claude_design::Project)->Result<(),String>{
    let p=crate::design::store(app)?.load(&project.id)?;
    let preview=design_preview(&p);
    if load(&root(app)?).docs.iter().any(|d|d.design_id.as_deref()==Some(&p.id)&&d.name==p.title&&d.imported_ms==p.updated_ms&&d.preview==preview){return Ok(());}
    change(app,|library,_|{
        let p=crate::design::store(app)?.load(&project.id)?;
        let bytes=serde_json::to_vec(&p).map_err(|e|e.to_string())?.len() as u64;
        if let Some(doc)=library.docs.iter_mut().find(|d|d.design_id.as_deref()==Some(&p.id)){
            doc.name=p.title.clone();doc.size=bytes;doc.imported_ms=p.updated_ms;doc.preview=design_preview(&p);
        }else{library.docs.push(Doc{id:p.id.clone(),name:p.title.clone(),folder:None,ext:"design".into(),size:bytes,imported_ms:p.updated_ms,origin:String::new(),status:"ready".into(),note:String::new(),preview:design_preview(&p),tags:vec!["设计稿".into()],design_id:Some(p.id.clone())});}
        Ok(())
    }).map(|_|())
}

pub(crate) fn forget_design(app:&tauri::AppHandle,id:&str)->Result<(),String>{change(app,|library,_|{library.docs.retain(|doc|doc.design_id.as_deref()!=Some(id));Ok(())}).map(|_|())}

fn design_preview(p:&open_claude_design::Project)->String{let status=match p.status{open_claude_design::ProjectStatus::Draft=>"草稿",open_claude_design::ProjectStatus::Generating=>"生成中",open_claude_design::ProjectStatus::Ready=>"已完成",open_claude_design::ProjectStatus::Failed=>"待恢复"};format!("{status} · {} 个画板 · 版本 {}",open_claude_design::boards(&p.canvas).map(|b|b.len()).unwrap_or(0),p.revision)}

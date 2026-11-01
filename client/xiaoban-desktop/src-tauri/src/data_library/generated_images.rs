//! Copies generated originals into the library. Never calls the image service.
use super::{Doc, Folder, Library};
use image::{ImageFormat, ImageReader, Limits};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, io::{Cursor, Read}, path::Path, time::UNIX_EPOCH};
use tauri::{Emitter, Manager};

const MAX_GENERATED_IMAGE: u64 = 16 * 1024 * 1024;
const FOLDER_ID: &str = "ai-generated-images";
const FOLDER_NAME: &str = "AI 生成图片";

fn valid_id(id: &str) -> bool {
    let Some((hash, ext)) = id.split_once('.') else { return false };
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && matches!(ext, "png" | "jpg" | "webp")
}

fn load_checked(root: &Path) -> Result<Library, String> {
    match fs::read(super::index_path(root)) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| "资料库索引无法读取，生成图片仍保存在聊天中".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Library::default()),
        Err(e) => Err(format!("资料库索引读取失败：{e}")),
    }
}

fn image_folder(library: &mut Library) -> String {
    if let Some(folder) = library.folders.iter().find(|f| f.id == FOLDER_ID || (f.parent.is_none() && f.name == FOLDER_NAME)) {
        return folder.id.clone();
    }
    library.folders.push(Folder { id: FOLDER_ID.into(), name: FOLDER_NAME.into(), parent: None, created_ms: super::now_ms() });
    FOLDER_ID.into()
}

fn import_one(library: &mut Library, root: &Path, directory: &Path, id: &str) -> Result<(), String> {
    if !valid_id(id) { return Err("生成图片编号无效".into()); }
    let source = directory.join(id);
    let origin = source.display().to_string();
    let hash = &id[..64];
    let doc_id = format!("generated-{hash}");
    // A previous manual import, rename or move must not create another copy.
    if library.docs.iter().any(|d| d.id == doc_id || d.origin == origin) { return Ok(()); }
    let metadata = source.symlink_metadata().map_err(|_| "生成图片原文件已不存在")?;
    if !metadata.is_file() || metadata.len() > MAX_GENERATED_IMAGE { return Err("生成图片文件无效或超过 16MB".into()); }
    let mut bytes = Vec::new();
    fs::File::open(&source).map_err(|_| "生成图片无法读取")?.take(MAX_GENERATED_IMAGE + 1)
        .read_to_end(&mut bytes).map_err(|_| "生成图片无法读取")?;
    if bytes.len() as u64 > MAX_GENERATED_IMAGE || format!("{:x}", Sha256::digest(&bytes)) != hash {
        return Err("生成图片校验失败，原文件可能尚未保存完整".into());
    }
    let ext = &id[65..];
    let format = image::guess_format(&bytes).map_err(|_| "生成图片格式无法识别")?;
    if !matches!((ext, format), ("png", ImageFormat::Png) | ("jpg", ImageFormat::Jpeg) | ("webp", ImageFormat::WebP)) {
        return Err("生成图片扩展名与内容不符".into());
    }
    let mut reader = ImageReader::with_format(Cursor::new(&bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(8192); limits.max_image_height = Some(8192); limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map_err(|_| "生成图片损坏或尺寸过大")?;

    let name = format!("小伴生图-{}.{}", &hash[..12], ext);
    let target = root.join("files").join(&doc_id);
    fs::create_dir_all(&target).map_err(|e| format!("资料库图片目录无法创建：{e}"))?;
    let temp = target.join(".importing");
    fs::write(&temp, &bytes).and_then(|_| fs::rename(&temp, target.join(&name)))
        .map_err(|e| format!("图片未能复制到资料库：{e}"))?;
    let folder = image_folder(library);
    let (status, note) = super::readable_kind(ext, bytes.len() as u64).unwrap_or(("ready", ""));
    let imported_ms = metadata.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|t| t.as_millis() as u64).unwrap_or_else(super::now_ms);
    library.docs.push(Doc {
        id: doc_id, name, folder: Some(folder), ext: ext.into(), size: bytes.len() as u64, imported_ms,
        origin, status: status.into(), note: note.into(), preview: String::new(), tags: vec!["AI 生成".into()], design_id: None,
    });
    Ok(())
}

/// The import history doubles as a deletion marker; only successful imports are recorded.
fn reconcile(library: &mut Library, root: &Path, directory: &Path, ids: &[String]) -> (bool, Vec<String>) {
    let mut seen: HashSet<String> = library.generated_image_imports.iter().cloned().collect();
    let mut changed = false;
    let mut warnings = Vec::new();
    for id in ids {
        if seen.contains(id) { continue; }
        match import_one(library, root, directory, id) {
            Ok(()) => { seen.insert(id.clone()); library.generated_image_imports.push(id.clone()); changed = true; }
            Err(error) => warnings.push(format!("图片已保存在聊天中，资料库入库未完成：{error}")),
        }
    }
    (changed, warnings)
}

fn synchronize(app: &tauri::AppHandle, ids: Option<&[String]>) -> Result<Vec<String>, String> {
    let directory = crate::storage::root(app)?.join("chat/generated-images");
    let scanned;
    let ids = match ids {
        Some(ids) => ids,
        None => {
            scanned = match fs::read_dir(&directory) {
                Ok(entries) => entries.filter_map(Result::ok).filter_map(|e| e.file_name().into_string().ok())
                    .filter(|id| valid_id(id)).collect::<Vec<_>>(),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
                Err(e) => return Err(format!("历史生成图片读取失败：{e}")),
            };
            &scanned
        }
    };
    if ids.is_empty() { return Ok(Vec::new()); }
    let state = app.state::<super::LibraryState>();
    let guard = state.lock.lock().unwrap_or_else(|e| e.into_inner());
    let root = super::root(app)?;
    let mut library = load_checked(&root)?;
    let (changed, warnings) = reconcile(&mut library, &root, &directory, ids);
    if changed { super::save(&root, &library)?; }
    drop(guard);
    // No event for a no-op: opening the library also triggers reconciliation.
    if changed { let _ = app.emit_to("control", "library-changed", ()); }
    Ok(warnings)
}

pub(crate) fn register(app: &tauri::AppHandle, ids: &[String]) -> Result<Vec<String>, String> {
    synchronize(app, Some(ids))
}

pub(super) fn recover(app: &tauri::AppHandle) -> Result<(), String> {
    for warning in synchronize(app, None)? { eprintln!("{warning}"); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!("xiaoban-image-library-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
            fs::create_dir_all(p.join("generated")).unwrap(); Self(p)
        }
        fn image(&self, width: u32) -> (String, Vec<u8>) {
            let mut png = Cursor::new(Vec::new());
            image::DynamicImage::new_rgba8(width, 2).write_to(&mut png, ImageFormat::Png).unwrap();
            let bytes = png.into_inner(); let id = format!("{:x}.png", Sha256::digest(&bytes));
            fs::write(self.0.join("generated").join(&id), &bytes).unwrap(); (id, bytes)
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

    #[test]
    fn imports_original_once_and_preserves_renames_moves_and_deletions() {
        let temp = Temp::new(); let (id, bytes) = temp.image(3); let mut library = Library::default();
        let root = temp.0.join("library"); let generated = temp.0.join("generated");
        assert_eq!(reconcile(&mut library, &root, &generated, &[id.clone(), id.clone()]), (true, vec![]));
        assert_eq!(library.docs.len(), 1); let doc = &library.docs[0];
        assert_eq!(doc.folder.as_deref(), Some(FOLDER_ID)); assert_eq!(doc.status, "ready");
        assert_eq!(doc.tags, ["AI 生成"]); assert_eq!(fs::read(root.join("files").join(&doc.id).join(&doc.name)).unwrap(), bytes);
        assert!(generated.join(&id).exists());
        library.docs[0].name = "我的壁纸.png".into(); library.docs[0].folder = None; library.docs[0].tags.clear();
        let before = library.clone();
        assert_eq!(reconcile(&mut library, &root, &generated, &[id.clone()]), (false, vec![])); assert_eq!(library, before);
        library.docs.clear(); super::super::save(&root, &library).unwrap(); library = load_checked(&root).unwrap();
        assert_eq!(reconcile(&mut library, &root, &generated, &[id]), (false, vec![])); assert!(library.docs.is_empty());
    }

    #[test]
    fn failed_copy_can_be_recovered_without_losing_other_images_or_regenerating() {
        let temp = Temp::new(); let (first, _) = temp.image(3); let (second, _) = temp.image(4);
        let root = temp.0.join("library"); let generated = temp.0.join("generated");
        fs::create_dir_all(root.join("files")).unwrap();
        let blocked = root.join("files").join(format!("generated-{}", &second[..64])); fs::write(&blocked, b"blocked").unwrap();
        let mut library = Library::default(); let (changed, warnings) = reconcile(&mut library, &root, &generated, &[first.clone(), second.clone()]);
        assert!(changed); assert_eq!(warnings.len(), 1); assert_eq!(library.generated_image_imports, [first.clone()]);
        super::super::save(&root, &library).unwrap(); fs::remove_file(blocked).unwrap(); library = load_checked(&root).unwrap();
        assert_eq!(reconcile(&mut library, &root, &generated, &[first, second]), (true, vec![])); assert_eq!(library.docs.len(), 2);
    }

    #[test]
    fn rejects_previews_traversal_corrupt_or_mismatched_files_and_links() {
        let temp = Temp::new(); let (id, _) = temp.image(3); let root = temp.0.join("library"); let generated = temp.0.join("generated");
        let mut library = Library::default();
        for bad in ["../outside.png".into(), format!("{id}.preview.png"), "a".repeat(64) + ".png", "f".repeat(64) + ".png"] {
            if valid_id(&bad) { fs::write(generated.join(&bad), b"broken").unwrap(); }
            assert_eq!(reconcile(&mut library, &root, &generated, &[bad]).1.len(), 1);
        }
        let corrupt = b"not an image"; let bad = format!("{:x}.png", Sha256::digest(corrupt)); fs::write(generated.join(&bad), corrupt).unwrap();
        assert_eq!(reconcile(&mut library, &root, &generated, &[bad]).1.len(), 1);
        #[cfg(unix)] {
            let linked = "b".repeat(64) + ".png"; std::os::unix::fs::symlink(generated.join(&id), generated.join(&linked)).unwrap();
            assert_eq!(reconcile(&mut library, &root, &generated, &[linked]).1.len(), 1);
        }
        assert!(library.docs.is_empty() && library.generated_image_imports.is_empty());
    }

    #[test]
    fn legacy_library_and_manual_imports_are_preserved_and_corrupt_index_is_not_overwritten() {
        let temp = Temp::new(); let (id, _) = temp.image(3); let root = temp.0.join("library"); let generated = temp.0.join("generated");
        fs::create_dir_all(&root).unwrap(); fs::write(super::super::index_path(&root), br#"{"docs":[],"folders":[],"links":[],"saved":[]}"#).unwrap();
        let mut library = load_checked(&root).unwrap();
        library.docs.push(Doc { id: "manual".into(), name: "我的图片.png".into(), origin: generated.join(&id).display().to_string(), ..Doc::default() });
        assert_eq!(reconcile(&mut library, &root, &generated, &[id]), (true, vec![])); assert_eq!(library.docs.len(), 1);
        assert_eq!(library.docs[0].id, "manual"); assert!(library.folders.is_empty());
        fs::write(super::super::index_path(&root), b"broken").unwrap(); assert!(load_checked(&root).is_err());
        assert_eq!(fs::read(super::super::index_path(&root)).unwrap(), b"broken");
    }
}

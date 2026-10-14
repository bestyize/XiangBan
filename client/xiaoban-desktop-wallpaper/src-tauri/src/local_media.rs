//! Native selection and streamed copying of user-owned images/videos.
//! The renderer sees managed hashes, never the original filesystem path.
use crate::library::{self, Catalog, Resource, Wallpaper};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

pub const MAX_LOCAL_FILE: u64 = 1024 * 1024 * 1024;
const MAX_LOCAL_IMAGE: u64 = 64 * 1024 * 1024;
const PREFIX: &str = "local://";

pub fn is_local(item: &Wallpaper) -> bool {
    item.id.starts_with("local-")
}
pub fn resource_local(resource: &Resource) -> bool {
    resource.url.starts_with(PREFIX)
}
pub fn resource_path(root: &Path, resource: &Resource) -> std::path::PathBuf {
    root.join(if resource_local(resource) {
        "imports"
    } else {
        "cache"
    })
    .join(&resource.sha256)
}

pub fn validate(item: &Wallpaper) -> Result<(), String> {
    let r = &item.resource;
    if !library::hash_name(&r.sha256)
        || r.bytes == 0
        || r.bytes > MAX_LOCAL_FILE
        || item.id != format!("local-{}", &r.sha256[..32])
        || r.url != format!("{PREFIX}{}", r.sha256)
        || item.cover.is_some()
        || !item.assets.is_empty()
        || !item.controls.is_empty()
        || !match item.kind.as_str() {
            "image" => {
                r.bytes <= MAX_LOCAL_IMAGE
                    && [
                        "image/png",
                        "image/jpeg",
                        "image/webp",
                        "image/gif",
                        "image/bmp",
                    ]
                    .contains(&r.mime.as_str())
            }
            "video" => ["video/mp4", "video/webm"].contains(&r.mime.as_str()),
            "web" => r.bytes <= 4 * 1024 * 1024 && r.mime == "text/html",
            _ => false,
        }
    {
        return Err("本地壁纸元数据不合法".into());
    }
    // Reuse general title/version validation without allowing local:// into the
    // remote delivery protocol or raising its 256 MB resource limit.
    let mut remote = item.clone();
    remote.id = "validation".into();
    remote.resource.url = "https://example.invalid/local-media".into();
    remote.resource.bytes = r.bytes.min(library::MAX_FILE);
    remote.resource.mime = if item.kind == "video" {
        "video/mp4"
    } else if item.kind == "web" {
        "text/html"
    } else {
        "image/png"
    }
    .into();
    library::validate_item(&remote, false)
}

pub(crate) fn media_type(data: &[u8]) -> Result<(&'static str, &'static str), String> {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok(("image", "image/png"));
    }
    if data.starts_with(b"\xff\xd8\xff") {
        return Ok(("image", "image/jpeg"));
    }
    if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Ok(("image", "image/webp"));
    }
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return Ok(("image", "image/gif"));
    }
    if data.starts_with(b"BM") {
        return Ok(("image", "image/bmp"));
    }
    if data.len() >= 16 && &data[4..8] == b"ftyp" {
        let size = u32::from_be_bytes(data[..4].try_into().unwrap()) as usize;
        if size >= 16
            && size <= data.len()
            && data[8..size].chunks_exact(4).any(|brand| {
                [
                    b"isom", b"iso2", b"iso3", b"iso4", b"iso5", b"iso6", b"mp41", b"mp42",
                    b"avc1", b"M4V ",
                ]
                .iter()
                .any(|known| brand == *known)
            })
            && !data[8..size]
                .chunks_exact(4)
                .any(|b| b == b"avif" || b == b"avis" || b == b"heic")
        {
            return Ok(("video", "video/mp4"));
        }
    }
    if data.starts_with(b"\x1a\x45\xdf\xa3") && data.windows(4).any(|b| b == b"webm") {
        return Ok(("video", "video/webm"));
    }
    Err("不支持这个文件。请选择 PNG、JPG、WebP、GIF、BMP 图片，或 MP4、M4V、WebM 视频。视频建议使用 H.264 MP4。".into())
}

pub fn read(root: &Path) -> Result<Catalog, String> {
    let path = root.join("local.json");
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Catalog {
                schema_version: 1,
                ..Default::default()
            })
        }
        Err(e) => return Err(e.to_string()),
    };
    let catalog: Catalog =
        serde_json::from_slice(&bytes).map_err(|e| format!("本地壁纸库读取失败：{e}"))?;
    if catalog.schema_version != 1 || catalog.items.len() > 1000 {
        return Err("本地壁纸库版本或数量不支持".into());
    }
    for item in &catalog.items {
        validate(item)?;
    }
    Ok(catalog)
}

pub fn import_file(root: &Path, source: &Path) -> Result<Wallpaper, String> {
    let mut input = fs::File::open(source).map_err(|e| format!("无法读取所选文件：{e}"))?;
    let metadata = input.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_LOCAL_FILE {
        return Err("请选择非空的图片或视频，单个文件不能超过 1 GB。".into());
    }
    let mut buffer = [0u8; 65536];
    let first = input.read(&mut buffer).map_err(|e| e.to_string())?;
    let (kind, mime) = media_type(&buffer[..first])?;
    if kind == "image" && metadata.len() > MAX_LOCAL_IMAGE {
        return Err("单张图片不能超过 64 MB。".into());
    }
    let directory = root.join("imports");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let temporary = directory.join(format!("import-{}.part", std::process::id()));
    let result = (|| {
        let mut output = fs::File::create(&temporary).map_err(|e| e.to_string())?;
        let mut digest = Sha256::new();
        let mut bytes = 0u64;
        let mut count = first;
        loop {
            if count == 0 {
                break;
            }
            bytes += count as u64;
            if bytes > MAX_LOCAL_FILE {
                return Err("文件超过 1 GB，已停止导入。".into());
            }
            digest.update(&buffer[..count]);
            output
                .write_all(&buffer[..count])
                .map_err(|e| e.to_string())?;
            count = input.read(&mut buffer).map_err(|e| e.to_string())?;
        }
        if bytes != metadata.len() {
            return Err("导入时源文件发生变化，请重试。".into());
        }
        output.sync_all().map_err(|e| e.to_string())?;
        drop(output);
        let hash = format!("{:x}", digest.finalize());
        let title: String = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("我的壁纸")
            .chars()
            .take(32)
            .collect();
        let item = Wallpaper {
            id: format!("local-{}", &hash[..32]),
            version: "1.0.0".into(),
            title: if title.trim().is_empty() {
                "我的壁纸".into()
            } else {
                title
            },
            description: "从电脑导入，已保存副本，移动原文件也能继续使用。".into(),
            author: "我的本地壁纸".into(),
            kind: kind.into(),
            resource: Resource {
                url: format!("{PREFIX}{hash}"),
                sha256: hash.clone(),
                bytes,
                mime: mime.into(),
            },
            cover: None,
            assets: Default::default(),
            controls: Vec::new(),
        };
        validate(&item)?;
        let mut catalog = read(root)?;
        if let Some(existing) = catalog.items.iter().find(|old| old.id == item.id) {
            if existing.resource.sha256 != hash {
                return Err("本地壁纸标识冲突".into());
            }
        } else if catalog.items.len() >= 1000 {
            return Err("本地壁纸库最多保存 1000 张壁纸。".into());
        }
        let target = directory.join(&hash);
        if library::verify_file(&target, &item.resource)? {
            fs::remove_file(&temporary).map_err(|e| e.to_string())?;
        } else {
            fs::rename(&temporary, &target).map_err(|e| e.to_string())?;
        }
        if let Some(existing) = catalog.items.iter().find(|old| old.id == item.id) {
            return Ok(existing.clone());
        }
        catalog.items.push(item.clone());
        library::save_json(&root.join("local.json"), &catalog)?;
        Ok(item)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub fn save_local_html(
    root: &Path,
    title: &str,
    description: Option<&str>,
    html: &str,
) -> Result<Wallpaper, String> {
    if html.trim().is_empty() {
        return Err("HTML 内容不能为空".into());
    }
    if html.len() as u64 > 4 * 1024 * 1024 {
        return Err("动态壁纸 HTML 内容不能超过 4 MB".into());
    }
    let title_clean = title.trim();
    if title_clean.is_empty() || title_clean.chars().count() > 48 {
        return Err("壁纸名称不能为空且不能超过 48 个字".into());
    }

    let mut hasher = Sha256::new();
    hasher.update(html.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let directory = root.join("imports");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let temporary = directory.join(format!(".html-{}.tmp", &hash[..16]));
    let target = directory.join(&hash);

    let desc = description.unwrap_or("由小伴现场创作的自包含动态桌面壁纸。").trim();
    let item = Wallpaper {
        id: format!("local-{}", &hash[..32]),
        version: "1.0.0".into(),
        title: title_clean.to_string(),
        description: if desc.is_empty() { "自包含动态桌面壁纸。".into() } else { desc.to_string() },
        author: "小伴创作".into(),
        kind: "web".into(),
        resource: Resource {
            url: format!("{PREFIX}{hash}"),
            sha256: hash.clone(),
            bytes: html.len() as u64,
            mime: "text/html".into(),
        },
        cover: None,
        assets: Default::default(),
        controls: Vec::new(),
    };
    validate(&item)?;

    let result = (|| -> Result<Wallpaper, String> {
        let mut catalog = read(root)?;
        if let Some(existing) = catalog.items.iter().find(|old| old.id == item.id) {
            if existing.resource.sha256 != hash {
                return Err("本地壁纸标识冲突".into());
            }
        } else if catalog.items.len() >= 1000 {
            return Err("本地壁纸库最多保存 1000 张壁纸。".into());
        }

        if !target.exists() || !library::verify_file(&target, &item.resource)? {
            fs::write(&temporary, html.as_bytes()).map_err(|e| e.to_string())?;
            fs::rename(&temporary, &target).map_err(|e| e.to_string())?;
        }

        if let Some(existing) = catalog.items.iter().find(|old| old.id == item.id) {
            return Ok(existing.clone());
        }
        catalog.items.push(item.clone());
        library::save_json(&root.join("local.json"), &catalog)?;
        Ok(item)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// Deletes a local wallpaper by id. Only local wallpapers (`local-*`) can be deleted.
/// If no other local wallpapers reference the imported file, the physical file is also removed.
pub fn delete_local_wallpaper(root: &Path, id: &str) -> Result<String, String> {
    if !id.starts_with("local-") {
        return Err("只能删除本地导入或创作的壁纸".into());
    }
    let mut catalog = read(root)?;
    let pos = catalog.items.iter().position(|w| w.id == id).ok_or_else(|| format!("未找到壁纸 {id}"))?;
    let removed = catalog.items.remove(pos);
    library::save_json(&root.join("local.json"), &catalog)?;

    // Check if any remaining wallpaper shares the same file hash
    let hash = removed.resource.sha256;
    let in_use = catalog.items.iter().any(|w| w.resource.sha256 == hash);
    if !in_use {
        let file_path = root.join("imports").join(&hash);
        let _ = fs::remove_file(file_path);
    }
    Ok(hash)
}

#[tauri::command]
pub async fn import_local_wallpaper(
    window: tauri::WebviewWindow,
) -> Result<Option<serde_json::Value>, String> {
    if window.label() != "control" {
        return Err("只能从设置面板导入文件".into());
    }
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let chosen = app.dialog().file().set_parent(&window).set_title("选择图片或视频壁纸")
            .add_filter("图片和视频", &["png","jpg","jpeg","webp","gif","bmp","mp4","m4v","webm"])
            .blocking_pick_file();
        let Some(chosen) = chosen else { return Ok(None); };
        let source = chosen.into_path().map_err(|e| e.to_string())?;
        let state = app.state::<crate::content::ContentState>();
        let _io = state.io.lock().unwrap();
        let item = import_file(&crate::content::root(&app)?, &source)?;
        Ok(Some(serde_json::json!({"id":item.id,"library":crate::content::get_wallpaper_library(app.clone())?})))
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_actual_media_and_rejects_scripts_and_disguised_files() {
        assert_eq!(media_type(b"\x89PNG\r\n\x1a\nrest").unwrap().1, "image/png");
        assert_eq!(
            media_type(include_bytes!(
                "../../public/wallpapers/builtin/sea-breeze.mp4"
            ))
            .unwrap()
            .1,
            "video/mp4"
        );
        assert!(media_type(b"<svg onload='alert(1)'/>").is_err());
        assert!(media_type(b"not really an mp4").is_err());
    }
    #[test]
    fn import_is_owned_deduplicated_and_never_serializes_source_paths() {
        let base = std::env::temp_dir().join(format!("xiaoban-import-test-{}", std::process::id()));
        let root = base.join("wallpapers");
        fs::create_dir_all(&base).unwrap();
        let source = base.join("原图 image.png");
        fs::write(&source, b"\x89PNG\r\n\x1a\nfixture").unwrap();
        let one = import_file(&root, &source).unwrap();
        let two = import_file(&root, &source).unwrap();
        assert_eq!(one.id, two.id);
        assert_eq!(read(&root).unwrap().items.len(), 1);
        fs::remove_file(&source).unwrap();
        assert!(library::verify_file(&resource_path(&root, &one.resource), &one.resource).unwrap());
        assert!(!fs::read_to_string(root.join("local.json"))
            .unwrap()
            .contains(&base.to_string_lossy().to_string()));
        assert!(
            library::parse_catalog(&serde_json::to_string(&read(&root).unwrap()).unwrap()).is_err()
        );
        let mut invalid = one.clone();
        invalid.resource.url = "local://../../secret".into();
        assert!(validate(&invalid).is_err());
        invalid = one.clone();
        invalid.kind = "canvas".into();
        assert!(validate(&invalid).is_err());
        invalid = one;
        invalid.resource.bytes = MAX_LOCAL_FILE + 1;
        assert!(validate(&invalid).is_err());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn saves_and_validates_custom_html_wallpaper() {
        let base = std::env::temp_dir().join(format!("xiaoban-html-test-{}", std::process::id()));
        let root = base.join("wallpapers");
        fs::create_dir_all(&base).unwrap();
        let item = save_local_html(
            &root,
            "水墨互动桌面",
            Some("自包含互动壁纸"),
            "<!DOCTYPE html><html><body><canvas></canvas></body></html>",
        )
        .unwrap();
        assert_eq!(item.kind, "web");
        assert_eq!(item.title, "水墨互动桌面");
        assert!(item.id.starts_with("local-"));
        assert_eq!(item.resource.mime, "text/html");

        let loaded = read(&root).unwrap();
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].id, item.id);
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn delete_local_wallpaper_removes_entry_and_file() {
        let base = std::env::temp_dir().join(format!("xiaoban-del-wall-{}", std::process::id()));
        let root = base.join("wallpapers");
        fs::create_dir_all(&base).unwrap();
        let item = save_local_html(&root, "测试壁纸", None, "<html>hello</html>").unwrap();
        let file_path = root.join("imports").join(&item.resource.sha256);
        assert!(file_path.exists());

        assert!(delete_local_wallpaper(&root, "builtin-soft-hills").unwrap_err().contains("只能删除"));

        let hash = delete_local_wallpaper(&root, &item.id).unwrap();
        assert_eq!(hash, item.resource.sha256);
        assert!(!file_path.exists(), "physical file is cleaned up when no other wallpaper references it");
        assert_eq!(read(&root).unwrap().items.len(), 0);
        let _ = fs::remove_dir_all(base);
    }
}

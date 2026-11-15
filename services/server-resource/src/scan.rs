//! 上传文件的检查：封面看文件头，内容包检查压缩包结构并读出清单。

use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 压缩包里最多这么多个条目，解压后合计不超过这个大小，防止压缩炸弹。
const MAX_ENTRIES: usize = 2000;
const MAX_UNCOMPRESSED: u64 = 128 * 1024 * 1024;
const MAX_MANIFEST: u64 = 64 * 1024;
const MB: u64 = 1024 * 1024;

/// 按文件头判断封面格式；只收 PNG、JPEG、WebP，返回内容类型。
pub fn sniff_image(head: &[u8]) -> Option<&'static str> {
    if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if head.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if head.len() >= 12 && &head[0..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.split('/').any(|part| part == "..")
}

fn desktop_path(name: &str) -> bool {
    let parts = name.split('/');
    parts.clone().count() <= 8 && parts.into_iter().all(|part| {
        if part.is_empty() || part.starts_with('.') || part.ends_with('.') || !part.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-@".contains(&b)) {
            return false;
        }
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") &&
            !((stem.starts_with("COM") || stem.starts_with("LPT")) && stem.len() == 4 && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    })
}

fn pet_asset(name: &str) -> bool {
    name.starts_with("assets/") && name.split('/').count() <= 4 && !name.contains('@') &&
        matches!(name.rsplit_once('.').map(|(_, ext)| ext), Some("png" | "gif" | "webp"))
}

/// Keep the archive unchanged. The signed callback includes metadata from a desktop ZIP's JSON entry.
fn desktop_manifest(archive: &mut zip::ZipArchive<File>, files: &[(String, u64)], mut manifest: Value) -> Result<Value, String> {
    if manifest.get("runtimeProtocol").is_some() {
        return Err("桌面作品的清单不需要 runtimeProtocol".into());
    }
    let kind = manifest["kind"].as_str().unwrap_or_default();
    let json_entry = matches!(kind, "effect" | "pet");
    let path = manifest["entry"].as_str().filter(|path| desktop_path(path) &&
        !path.split('/').next().is_some_and(|part| part.eq_ignore_ascii_case("manifest.json")))
        .ok_or("桌面作品的清单需要合法的 entry 路径")?;
    let extension = path.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase());
    if (json_entry && extension.as_deref() != Some("json")) || (!json_entry && !matches!(extension.as_deref(), Some("html" | "htm"))) {
        return Err("entry 文件格式与桌面作品类型不一致".into());
    }
    let mut seen = HashSet::new();
    let (mut assets, mut asset_bytes, mut total) = (0, 0, 0);
    for (name, size) in files {
        let base = name.rsplit('/').next().unwrap_or("");
        if name.starts_with("__MACOSX/") || base == ".DS_Store" || base.starts_with("._") { continue; }
        if !desktop_path(name) || !seen.insert(name.to_ascii_lowercase()) {
            return Err(format!("ZIP 包含无效或重复的路径：{name}"));
        }
        total += size;
        if *size > 4 * MB || total > 32 * MB || seen.len() > 110 {
            return Err("桌面作品超过文件大小或数量上限".into());
        }
        if name == "manifest.json" || name == path { continue; }
        if kind != "pet" || !pet_asset(name) {
            return Err(format!("桌面作品包不支持额外文件：{name}"));
        }
        assets += 1;
        asset_bytes += size;
        if assets > 60 || asset_bytes > 16 * MB {
            return Err("伙伴图片最多 60 个，合计不能超过 16 MB".into());
        }
        let mut asset = archive.by_name(name).map_err(|_| format!("无法读取 {name}"))?;
        let mut head = Vec::with_capacity(12);
        asset.by_ref().take(12).read_to_end(&mut head).map_err(|_| format!("无法解压 {name}"))?;
        let valid_image = if name.ends_with(".gif") {
            head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a")
        } else {
            sniff_image(&head) == Some(if name.ends_with(".png") { "image/png" } else { "image/webp" })
        };
        if !valid_image {
            return Err(format!("伙伴图片格式与文件名不一致：{name}"));
        }
    }
    if !files.iter().any(|(name, _)| name == path) || !seen.contains(&path.to_ascii_lowercase()) {
        return Err(format!("ZIP 缺少入口文件 {path}"));
    }
    let entry_limit = if kind == "widget" { 128 * 1024 } else { 4 * MB };
    let mut entry = archive.by_name(path).map_err(|_| format!("无法读取 {path}"))?;
    let mut bytes = Vec::new();
    entry.by_ref().take(entry_limit + 1).read_to_end(&mut bytes).map_err(|_| format!("无法解压 {path}"))?;
    if bytes.len() as u64 > entry_limit || bytes.is_empty() {
        return Err(format!("{path} 为空或超过大小上限"));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| format!("{path} 必须是 UTF-8 文本"))?;
    if json_entry {
        let data: Value = serde_json::from_str(text).map_err(|_| format!("{path} 不是有效的 JSON"))?;
        let version = data["schemaVersion"].as_u64();
        if version != Some(1) && !(kind == "pet" && version == Some(2)) {
            return Err(format!("{path} 的数据版本不受支持"));
        }
        let id = data["id"].as_str().ok_or_else(|| format!("{path} 缺少作品 ID"))?;
        let name = data["name"].as_str().ok_or_else(|| format!("{path} 缺少作品名称"))?;
        if manifest.get("id").is_some_and(|declared| declared.as_str() != Some(id)) {
            return Err("清单和入口 JSON 的作品 ID 不一致".into());
        }
        manifest["id"] = Value::String(id.into());
        manifest["name"] = Value::String(name.into());
    }
    Ok(manifest)
}

/// 检查内容包 zip：条目数与解压总量有上限、路径不能穿越目录、根目录必须有 `manifest.json`。
/// 返回清单与桌面作品的入口元数据；清单字段的业务检查（类型、id、版本）由论坛回调时做。
pub fn scan_pack(path: &Path) -> Result<Value, String> {
    let file = File::open(path).map_err(|e| format!("读取文件失败：{e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| "不是有效的 zip 压缩包".to_string())?;
    if archive.len() > MAX_ENTRIES {
        return Err(format!("压缩包里的文件太多（上限 {MAX_ENTRIES}）"));
    }
    let mut total: u64 = 0;
    let mut manifest: Option<Value> = None;
    let mut files = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|_| "压缩包已损坏".to_string())?;
        let name = entry.name().to_string();
        if !safe_name(&name) {
            return Err(format!("压缩包里有不安全的路径：{name}"));
        }
        if entry.is_symlink() {
            return Err(format!("压缩包里不允许符号链接：{name}"));
        }
        total = total.saturating_add(entry.size());
        if total > MAX_UNCOMPRESSED {
            return Err("压缩包解压后过大".into());
        }
        if !entry.is_dir() { files.push((name.clone(), entry.size())); }
        if name == "manifest.json" {
            if entry.size() > MAX_MANIFEST {
                return Err("manifest.json 过大".into());
            }
            let mut buf = Vec::new();
            // 以实际读到的为准，不信条目声明的大小。
            entry.by_ref().take(MAX_MANIFEST + 1).read_to_end(&mut buf).map_err(|_| "压缩包已损坏".to_string())?;
            if buf.len() as u64 > MAX_MANIFEST {
                return Err("manifest.json 过大".into());
            }
            manifest = Some(serde_json::from_slice(&buf).map_err(|_| "manifest.json 不是有效的 JSON".to_string())?);
        }
    }
    let manifest = manifest.ok_or("压缩包根目录缺少 manifest.json")?;
    if manifest["schemaVersion"] == 3 && matches!(manifest["kind"].as_str(), Some("wallpaper" | "effect" | "widget" | "pet")) {
        return desktop_manifest(&mut archive, &files, manifest);
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_zip(entries: &[(&str, &[u8])]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("scan-{}.zip", uuid::Uuid::new_v4()));
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        for (name, data) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap();
        path
    }

    #[test]
    fn reads_manifest_and_rejects_bad_archives() {
        let ok = make_zip(&[("manifest.json", br#"{"id":"a"}"#), ("assets/x.png", b"png")]);
        assert_eq!(scan_pack(&ok).unwrap()["id"], "a");

        let none = make_zip(&[("readme.txt", b"x")]);
        assert!(scan_pack(&none).unwrap_err().contains("manifest.json"));

        let nested = make_zip(&[("dir/manifest.json", b"{}")]);
        assert!(scan_pack(&nested).is_err());

        let traversal = make_zip(&[("manifest.json", b"{}"), ("../evil", b"x")]);
        assert!(scan_pack(&traversal).unwrap_err().contains("不安全"));

        let abs = make_zip(&[("manifest.json", b"{}"), ("/etc/passwd", b"x")]);
        assert!(scan_pack(&abs).is_err());

        let bad_json = make_zip(&[("manifest.json", b"not json")]);
        assert!(scan_pack(&bad_json).unwrap_err().contains("JSON"));

        let not_zip = std::env::temp_dir().join(format!("scan-{}.bin", uuid::Uuid::new_v4()));
        std::fs::write(&not_zip, b"hello").unwrap();
        assert!(scan_pack(&not_zip).is_err());
    }

    #[test]
    fn image_sniffing() {
        assert_eq!(sniff_image(b"\x89PNG\r\n\x1a\nxxxx"), Some("image/png"));
        assert_eq!(sniff_image(&[0xff, 0xd8, 0xff, 0xe0]), Some("image/jpeg"));
        assert_eq!(sniff_image(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff_image(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"), None);
        assert_eq!(sniff_image(b"GIF89a"), None);
    }

    #[test]
    fn desktop_exports_keep_wallpaper_metadata_and_read_effect_and_pet_metadata() {
        let wallpaper = make_zip(&[
            ("manifest.json", br#"{"schemaVersion":3,"kind":"wallpaper","entry":"page/main.htm","name":"Night"}"#),
            ("page/main.htm", b"<!doctype html><title>Night</title>"),
        ]);
        let manifest = scan_pack(&wallpaper).unwrap();
        assert_eq!(manifest["name"], "Night");
        assert!(manifest.get("id").is_none());

        let effect = make_zip(&[
            ("manifest.json", br#"{"schemaVersion":3,"kind":"effect","entry":"effect.json"}"#),
            ("effect.json", br#"{"schemaVersion":1,"id":"aurora","name":"Aurora","layers":[]}"#),
        ]);
        let manifest = scan_pack(&effect).unwrap();
        assert_eq!(manifest["id"], "aurora");
        assert_eq!(manifest["name"], "Aurora");

        let pet = make_zip(&[
            ("manifest.json", br#"{"schemaVersion":3,"kind":"pet","entry":"pet.json"}"#),
            ("pet.json", br#"{"schemaVersion":2,"id":"fox","name":"Fox","pets":[]}"#),
            ("assets/Face.webp", b"RIFF\0\0\0\0WEBPVP8 "),
        ]);
        let manifest = scan_pack(&pet).unwrap();
        assert_eq!(manifest["id"], "fox");
        assert_eq!(manifest["name"], "Fox");
        for path in [wallpaper, effect, pet] { std::fs::remove_file(path).unwrap(); }
    }

    #[test]
    fn desktop_exports_reject_missing_entries_conflicting_ids_and_unexpected_files() {
        let no_entry = make_zip(&[("manifest.json", br#"{"schemaVersion":3,"kind":"widget","entry":"index.html"}"#)]);
        assert!(scan_pack(&no_entry).unwrap_err().contains("缺少入口"));
        let wrong_id = make_zip(&[
            ("manifest.json", br#"{"schemaVersion":3,"kind":"pet","entry":"pet.json","id":"other"}"#),
            ("pet.json", br#"{"schemaVersion":1,"id":"fox","name":"Fox"}"#),
        ]);
        assert!(scan_pack(&wrong_id).unwrap_err().contains("ID 不一致"));
        let extra = make_zip(&[
            ("manifest.json", br#"{"schemaVersion":3,"kind":"effect","entry":"effect.json"}"#),
            ("effect.json", br#"{}"#),
            ("extra.html", b"<title>extra</title>"),
        ]);
        assert!(scan_pack(&extra).unwrap_err().contains("额外文件"));
        for path in [no_entry, wrong_id, extra] { std::fs::remove_file(path).unwrap(); }
    }
}

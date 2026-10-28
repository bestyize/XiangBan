//! 上传文件的检查：封面看文件头，内容包检查压缩包结构并读出清单。

use serde_json::Value;
use std::io::Read;
use std::path::Path;

/// 压缩包里最多这么多个条目，解压后合计不超过这个大小，防止压缩炸弹。
const MAX_ENTRIES: usize = 2000;
const MAX_UNCOMPRESSED: u64 = 128 * 1024 * 1024;
const MAX_MANIFEST: u64 = 64 * 1024;

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

/// 检查内容包 zip：条目数与解压总量有上限、路径不能穿越目录、根目录必须有 `manifest.json`。
/// 返回读出的清单；清单字段的业务检查（类型、id、版本）由论坛回调时做。
pub fn scan_pack(path: &Path) -> Result<Value, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("读取文件失败：{e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| "不是有效的 zip 压缩包".to_string())?;
    if archive.len() > MAX_ENTRIES {
        return Err(format!("压缩包里的文件太多（上限 {MAX_ENTRIES}）"));
    }
    let mut total: u64 = 0;
    let mut manifest: Option<Value> = None;
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
    manifest.ok_or_else(|| "压缩包根目录缺少 manifest.json".to_string())
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
}

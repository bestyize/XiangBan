//! 技能安装包校验与清单生成。
//!
//! 限制与桌面客户端 `libs/agent-tools/skill-discovery` 保持一致：上架时就拒绝客户端会拒绝的包。
//! 清单摘要（manifest_sha256）是本服务自有算法：按路径排序后对每行 `"<sha256>  <path>\n"` 求 SHA-256。
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    io::{Cursor, Read},
};

pub const MAX_ZIP_BYTES: usize = 12 * 1024 * 1024;
pub const MAX_FILES: usize = 512;
pub const MAX_TOTAL_BYTES: u64 = 12 * 1024 * 1024;
pub const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_SKILL_MD_BYTES: u64 = 100 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug)]
pub struct Inspected {
    pub files: Vec<FileEntry>,
    pub manifest_sha256: String,
    pub skill_md: String,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
    "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

fn safe_path(name: &str) -> Result<(), String> {
    let bad = || format!("不安全的文件路径：{name}");
    if name.is_empty() || name.len() > 240 || name.contains('\\') || name.starts_with('/') || name.contains(':') {
        return Err(bad());
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(bad());
    }
    // 与客户端 skill-discovery 的 valid_path 一致：深度 ≤ 8，单段 ≤ 100，且不允许以 . 开头的隐藏文件。
    if name.split('/').count() > 8 {
        return Err(bad());
    }
    for part in name.split('/') {
        if part.is_empty() || part.len() > 100 || part.starts_with('.') || part.ends_with('.') || part.ends_with(' ') {
            return Err(bad());
        }
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if RESERVED.contains(&stem.as_str()) {
            return Err(bad());
        }
    }
    Ok(())
}

pub fn inspect_zip(bytes: &[u8]) -> Result<Inspected, String> {
    if bytes.len() > MAX_ZIP_BYTES {
        return Err("安装包超过 12 MiB".into());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("不是有效的 ZIP：{e}"))?;
    if archive.len() > MAX_FILES * 2 {
        return Err("安装包条目过多".into());
    }
    let mut names = Vec::new();
    for i in 0..archive.len() {
        let f = archive.by_index(i).map_err(|e| format!("无法读取 ZIP 条目：{e}"))?;
        if f.is_dir() {
            continue;
        }
        if f.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err(format!("不允许符号链接：{}", f.name()));
        }
        names.push((i, f.name().to_string()));
    }
    if names.is_empty() {
        return Err("安装包为空".into());
    }
    if names.len() > MAX_FILES {
        return Err("安装包文件数超过 512".into());
    }
    // 允许单个顶层包装目录。
    let first_dir = |n: &str| n.split_once('/').map(|(d, _)| d.to_string());
    let wrapper = first_dir(&names[0].1).filter(|d| names.iter().all(|(_, n)| first_dir(n).as_deref() == Some(d.as_str())));
    let mut seen = HashSet::new();
    let mut total = 0u64;
    let mut files = Vec::new();
    let mut skill_md = None;
    for (i, name) in &names {
        let path = match &wrapper {
            Some(d) => name[d.len() + 1..].to_string(),
            None => name.clone(),
        };
        safe_path(&path)?;
        if !seen.insert(path.to_ascii_lowercase()) {
            return Err(format!("路径大小写重复：{path}"));
        }
        let limit = if path == "SKILL.md" { MAX_SKILL_MD_BYTES } else { MAX_FILE_BYTES };
        let mut buf = Vec::new();
        archive
            .by_index(*i)
            .map_err(|e| format!("无法读取 {path}：{e}"))?
            .take(limit + 1)
            .read_to_end(&mut buf)
            .map_err(|e| format!("无法解压 {path}：{e}"))?;
        if buf.len() as u64 > limit {
            return Err(format!("文件过大：{path}"));
        }
        total += buf.len() as u64;
        if total > MAX_TOTAL_BYTES {
            return Err("解压后总大小超过 12 MiB".into());
        }
        if path == "SKILL.md" {
            skill_md = Some(String::from_utf8(buf.clone()).map_err(|_| "SKILL.md 不是 UTF-8".to_string())?);
        }
        files.push(FileEntry { path, size: buf.len() as u64, sha256: sha256_hex(&buf) });
    }
    let skill_md = skill_md.ok_or("安装包根目录缺少 SKILL.md")?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut manifest = Sha256::new();
    for f in &files {
        manifest.update(format!("{}  {}\n", f.sha256, f.path));
    }
    Ok(Inspected { files, manifest_sha256: format!("{:x}", manifest.finalize()), skill_md })
}

/// 读取 SKILL.md 头部的 `name` / `description`；没有头部时返回 `None`。
pub fn frontmatter(md: &str) -> (Option<String>, Option<String>) {
    let Some(rest) = md.strip_prefix("---\n").or_else(|| md.strip_prefix("---\r\n")) else {
        return (None, None);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, None);
    };
    let (mut name, mut description) = (None, None);
    for line in rest[..end].lines() {
        let Some((k, v)) = line.split_once(':') else { continue };
        let v = v.trim().trim_matches(|c| c == '"' || c == '\'').trim();
        if v.is_empty() {
            continue;
        }
        match k.trim() {
            "name" => name = Some(v.to_string()),
            "description" => description = Some(v.to_string()),
            _ => {}
        }
    }
    (name, description)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    pub fn build(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, data) in entries {
            w.start_file(*name, opts).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    #[test]
    fn accepts_plain_and_wrapped_packages_with_the_same_manifest() {
        let plain = inspect_zip(&build(&[("SKILL.md", b"# hi"), ("scripts/run.py", b"print(1)")])).unwrap();
        let wrapped = inspect_zip(&build(&[("pkg/SKILL.md", b"# hi"), ("pkg/scripts/run.py", b"print(1)")])).unwrap();
        assert_eq!(plain.files.len(), 2);
        assert_eq!(plain.manifest_sha256, wrapped.manifest_sha256);
        assert_eq!(plain.skill_md, "# hi");
    }

    #[test]
    fn rejects_what_the_desktop_client_would_reject() {
        for bad in ["../evil.txt", "a/../b.txt", "/abs.txt", "C:/x.txt", "dir\\x.txt", "CON.txt", "trail.", "a//b", ".hidden", "dir/.env", "a/b/c/d/e/f/g/h/i.txt"] {
            let err = inspect_zip(&build(&[("SKILL.md", b"x"), (bad, b"x")]));
            assert!(err.is_err(), "{bad} should be rejected");
        }
        let long = format!("{}.txt", "n".repeat(101));
        assert!(inspect_zip(&build(&[("SKILL.md", b"x"), (long.as_str(), b"x")])).is_err());
        assert!(inspect_zip(&build(&[("README.md", b"x")])).unwrap_err().contains("SKILL.md"));
        assert!(inspect_zip(&build(&[("SKILL.md", b"x"), ("A.txt", b"1"), ("a.TXT", b"2")])).unwrap_err().contains("大小写"));
        let big = vec![0u8; MAX_SKILL_MD_BYTES as usize + 1];
        assert!(inspect_zip(&build(&[("SKILL.md", &big)])).is_err());
        assert!(inspect_zip(b"not a zip").is_err());
    }

    #[test]
    fn frontmatter_is_optional() {
        let (n, d) = frontmatter("---\nname: demo\ndescription: \"做点事\"\n---\nbody");
        assert_eq!((n.as_deref(), d.as_deref()), (Some("demo"), Some("做点事")));
        assert_eq!(frontmatter("# no header"), (None, None));
    }
}

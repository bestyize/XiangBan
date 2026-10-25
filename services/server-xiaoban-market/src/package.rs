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

/// 上游注册表在包里附带的元数据目录（如 ClawHub 的 `.clawhub/origin.json`），不属于技能本身。
const REGISTRY_METADATA_DIRS: [&str; 1] = [".clawhub"];

fn is_registry_metadata(name: &str) -> bool {
    name.split('/').any(|part| REGISTRY_METADATA_DIRS.contains(&part))
}

/// 把 SKILL.md 头部的 `name` 改成 `slug`。已经一致、没有头部或没有 `name` 行时返回 `None`。
/// 桌面端要求头部 name 与市场 slug 一致（安装目录和技能名都按它走），而上游改过 slug 的技能
/// （例如重名时加了作者后缀）文件里还是原名。只改这一行，其余字节不动。
pub fn set_frontmatter_name(md: &str, slug: &str) -> Option<String> {
    let (bom, body) = match md.strip_prefix('\u{feff}') {
        Some(rest) => ("\u{feff}", rest),
        None => ("", md),
    };
    let mut lines = body.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end() != "---" {
        return None;
    }
    let mut out = String::from(bom);
    out.push_str(first);
    let mut changed = false;
    let mut closed = false;
    for line in lines {
        if closed {
            out.push_str(line);
        } else if line.trim_end() == "---" {
            closed = true;
            out.push_str(line);
        } else if !changed && !line.starts_with(char::is_whitespace) && line.starts_with("name:") {
            let value = line["name:".len()..].trim().trim_matches(|c| c == '"' || c == '\'');
            if value == slug {
                return None;
            }
            let eol = if line.ends_with("\r\n") { "\r\n" } else if line.ends_with('\n') { "\n" } else { "" };
            out.push_str(&format!("name: {slug}{eol}"));
            changed = true;
        } else {
            out.push_str(line);
        }
    }
    (closed && changed).then_some(out)
}

/// 入库前整理安装包：剔除上游注册表的元数据目录，并让 SKILL.md 头部的 name 与 slug 一致。
/// 都不需要改时原样返回，保持字节和摘要不变；其余条目按原样（含压缩数据）复制，客户端的安全校验不放宽。
pub fn prepare_package(bytes: &[u8], slug: &str) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("不是有效的 ZIP：{e}"))?;
    let mut keep = Vec::new();
    let mut dropped = false;
    let mut skill_entry: Option<(usize, String)> = None;
    for i in 0..archive.len() {
        let f = archive.by_index_raw(i).map_err(|e| format!("无法读取 ZIP 条目：{e}"))?;
        let name = f.name().to_string();
        if is_registry_metadata(&name) {
            dropped = true;
            continue;
        }
        let at_root = name == "SKILL.md";
        let in_wrapper = name.ends_with("/SKILL.md") && name.matches('/').count() == 1;
        if (at_root || (in_wrapper && skill_entry.is_none())) && !f.is_dir() {
            skill_entry = Some((i, name));
        }
        keep.push(i);
    }
    let mut renamed: Option<(usize, String, String)> = None;
    if let Some((i, name)) = skill_entry {
        let mut buf = Vec::new();
        archive
            .by_index(i)
            .map_err(|e| format!("无法读取 {name}：{e}"))?
            .take(MAX_SKILL_MD_BYTES + 1)
            .read_to_end(&mut buf)
            .map_err(|e| format!("无法解压 {name}：{e}"))?;
        // 过大或不是 UTF-8 的交给后面的校验报错。
        if buf.len() as u64 <= MAX_SKILL_MD_BYTES {
            if let Some(fixed) = String::from_utf8(buf).ok().and_then(|md| set_frontmatter_name(&md, slug)) {
                renamed = Some((i, name, fixed));
            }
        }
    }
    if !dropped && renamed.is_none() {
        return Ok(bytes.to_vec());
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in keep {
        match &renamed {
            Some((ri, name, content)) if *ri == i => {
                let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
                writer.start_file(name.as_str(), opts).map_err(|e| format!("重新打包失败：{e}"))?;
                std::io::Write::write_all(&mut writer, content.as_bytes()).map_err(|e| format!("重新打包失败：{e}"))?;
            }
            _ => {
                let f = archive.by_index_raw(i).map_err(|e| format!("无法读取 ZIP 条目：{e}"))?;
                writer.raw_copy_file(f).map_err(|e| format!("重新打包失败：{e}"))?;
            }
        }
    }
    writer.finish().map(Cursor::into_inner).map_err(|e| format!("重新打包失败：{e}"))
}

/// 把专家的工作流正文打成只含 SKILL.md 的安装包：SkillHub 的专家本身就是一份 meta-skill，
/// 桌面端按技能的方式安装它（版本、清单、下载一套流程）。头部 name 会对齐 slug。
pub fn expert_package(content: &str, slug: &str) -> Result<Vec<u8>, String> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    writer.start_file("SKILL.md", opts).map_err(|e| format!("打包失败：{e}"))?;
    std::io::Write::write_all(&mut writer, content.as_bytes()).map_err(|e| format!("打包失败：{e}"))?;
    let raw = writer.finish().map(Cursor::into_inner).map_err(|e| format!("打包失败：{e}"))?;
    prepare_package(&raw, slug)
}

/// 专家包的版本号：取 SKILL.md 头部的 `version`（没有或不合法就用 1.0.0）。
/// 上游改了正文却没改版本号时，加上内容摘要前 8 位作后缀，让客户端能发现更新。
pub fn expert_version(md: &str, package_sha256: &str, stored_sha256: Option<&str>) -> String {
    let declared = md
        .strip_prefix("---\n")
        .or_else(|| md.strip_prefix("---\r\n"))
        .and_then(|rest| rest.split("\n---").next())
        .and_then(|header| header.lines().find_map(|l| l.strip_prefix("version:")))
        .map(|v| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
        .filter(|v| !v.is_empty() && v.len() <= 30 && v.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-')));
    let base = declared.unwrap_or_else(|| "1.0.0".into());
    match stored_sha256 {
        Some(stored) if stored != package_sha256 => format!("{base}+{}", &package_sha256[..8]),
        _ => base,
    }
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
    fn manifest_digest_matches_the_desktop_client_vector() {
        // The desktop client asserts the same value in skill-discovery's market tests.
        let info = inspect_zip(&build(&[("SKILL.md", b"# hi"), ("scripts/run.py", b"print(1)")])).unwrap();
        assert_eq!(info.manifest_sha256, "ec97286e918f9e7ff5b49adfef3d8f61a1a502409ed93201e5046588cf0a427f");
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
    fn registry_metadata_is_stripped_before_inspection() {
        let raw = build(&[("SKILL.md", b"# hi"), (".clawhub/origin.json", b"{}"), ("scripts/run.py", b"print(1)")]);
        assert!(inspect_zip(&raw).is_err());
        let clean = prepare_package(&raw, "demo").unwrap();
        let info = inspect_zip(&clean).unwrap();
        assert_eq!(info.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), ["SKILL.md", "scripts/run.py"]);
        let plain = inspect_zip(&build(&[("SKILL.md", b"# hi"), ("scripts/run.py", b"print(1)")])).unwrap();
        assert_eq!(info.manifest_sha256, plain.manifest_sha256);
        let wrapped = build(&[("pkg/SKILL.md", b"# hi"), ("pkg/.clawhub/origin.json", b"{}")]);
        assert_eq!(inspect_zip(&prepare_package(&wrapped, "demo").unwrap()).unwrap().files.len(), 1);
    }

    #[test]
    fn packages_without_registry_metadata_pass_through_unchanged() {
        let raw = build(&[("SKILL.md", b"# hi"), ("scripts/run.py", b"print(1)")]);
        assert_eq!(prepare_package(&raw, "demo").unwrap(), raw);
        // 其他点开头的文件仍然拒绝。
        let hidden = prepare_package(&build(&[("SKILL.md", b"x"), (".env", b"x")]), "demo").unwrap();
        assert!(inspect_zip(&hidden).is_err());
    }

    #[test]
    fn skill_name_is_aligned_with_the_slug() {
        let md = "---\nname: long-article-illustration\ndescription: 配图\n---\n# body\nname: not-header\n";
        let raw = build(&[("SKILL.md", md.as_bytes()), ("scripts/run.py", b"print(1)")]);
        let fixed = prepare_package(&raw, "long-article-illustration-by-hongzhua").unwrap();
        let info = inspect_zip(&fixed).unwrap();
        assert_eq!(frontmatter(&info.skill_md).0.as_deref(), Some("long-article-illustration-by-hongzhua"));
        assert_eq!(frontmatter(&info.skill_md).1.as_deref(), Some("配图"));
        assert!(info.skill_md.ends_with("# body\nname: not-header\n"), "只改头部的 name 行");
        assert_eq!(info.files.len(), 2);
        // 包着一层目录、CRLF 和带引号的 name 也能对齐；一致时不动。
        let crlf = "---\r\nname: \"old\"\r\ndescription: d\r\n---\r\nbody";
        let wrapped = prepare_package(&build(&[("pkg/SKILL.md", crlf.as_bytes())]), "new-slug").unwrap();
        assert_eq!(inspect_zip(&wrapped).unwrap().skill_md, "---\r\nname: new-slug\r\ndescription: d\r\n---\r\nbody");
        let same = build(&[("SKILL.md", b"---\nname: demo\ndescription: d\n---\n")]);
        assert_eq!(prepare_package(&same, "demo").unwrap(), same);
        // 没有头部的交给注册表元数据，不改。
        let bare = build(&[("SKILL.md", b"# no header")]);
        assert_eq!(prepare_package(&bare, "demo").unwrap(), bare);
        assert_eq!(set_frontmatter_name("---\ndescription: d\n---\n", "demo"), None);
    }

    #[test]
    fn expert_content_becomes_an_installable_package() {
        let md = "---\nname: tech-test-automation\ndescription: 自动化测试\nversion: 1.0.0\n---\n# 流程\n";
        let zip = expert_package(md, "tech-test-automation").unwrap();
        let info = inspect_zip(&zip).unwrap();
        assert_eq!(info.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), ["SKILL.md"]);
        assert_eq!(info.skill_md, md);
        // 头部 name 与 slug 不一致时对齐；没有头部的正文也能打包。
        let renamed = inspect_zip(&expert_package(md, "other-slug").unwrap()).unwrap();
        assert_eq!(frontmatter(&renamed.skill_md).0.as_deref(), Some("other-slug"));
        assert!(inspect_zip(&expert_package("# 没有头部", "demo").unwrap()).is_ok());
    }

    #[test]
    fn expert_versions_follow_the_header_and_change_with_the_content() {
        let md = "---\nname: x\nversion: 2.1.0\n---\nbody";
        let sha = "a".repeat(64);
        assert_eq!(expert_version(md, &sha, None), "2.1.0");
        assert_eq!(expert_version(md, &sha, Some(&sha)), "2.1.0", "同一内容重复入库不换版本");
        assert_eq!(expert_version(md, &sha, Some(&"b".repeat(64))), "2.1.0+aaaaaaaa");
        assert_eq!(expert_version("# 没有头部", &sha, None), "1.0.0");
        assert_eq!(expert_version("---\nversion: bad version!\n---\n", &sha, None), "1.0.0");
    }

    #[test]
    fn frontmatter_is_optional() {
        let (n, d) = frontmatter("---\nname: demo\ndescription: \"做点事\"\n---\nbody");
        assert_eq!((n.as_deref(), d.as_deref()), (Some("demo"), Some("做点事")));
        assert_eq!(frontmatter("# no header"), (None, None));
    }
}

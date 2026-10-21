//! Validate a pinned SkillHub package before writing *any* of its files.
//! The API manifest is authoritative; ZIP entry names are never trusted as paths.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

const MAX_FILES: usize = 100;
const MAX_FILE: u64 = 4 * 1024 * 1024;
const MAX_TOTAL: u64 = 12 * 1024 * 1024;
const MAX_ARCHIVE: usize = 12 * 1024 * 1024;

#[derive(Clone, Debug)]
pub(crate) struct Item {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone)]
pub(crate) struct Plan {
    pub version: String,
    pub files: Vec<Item>,
    pub sha256: String,
}

pub(crate) fn valid_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 240 || path.starts_with('/') || path.contains('\\')
        || path.contains(':') || path.chars().any(char::is_control) {
        return false;
    }
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() > 8 {
        return false;
    }
    parts.iter().all(|p| {
        if p.is_empty() || *p == "." || *p == ".." || p.len() > 100
            || (p.ends_with(' ') || p.ends_with('.')) || p.starts_with('.') {
            return false;
        }
        let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
        !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL"
            | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9"
            | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9")
    })
}

pub(crate) fn is_script(path: &str) -> bool {
    path.starts_with("scripts/")
        && matches!(path.rsplit('.').next(), Some("py" | "js" | "mjs" | "cjs" | "sh" | "ps1" | "rb"))
}

pub(crate) fn plan(version: &str, values: &[Value]) -> Result<Plan, String> {
    if values.is_empty() || values.len() > MAX_FILES {
        return Err("invalid SkillHub file count".into());
    }
    let mut files = Vec::new();
    let mut folded = HashSet::new();
    let mut total = 0u64;
    for value in values {
        let path = value["path"].as_str().ok_or("file path missing")?;
        let size = value["size"].as_u64().ok_or("file size missing")?;
        let sha = value["sha256"].as_str().ok_or("file checksum missing")?;
        if !valid_path(path) || !folded.insert(path.to_lowercase()) {
            return Err(format!("unsafe or duplicate SkillHub path: {path}"));
        }
        if size > MAX_FILE || (path == "SKILL.md" && size > 100 * 1024) {
            return Err(format!("SkillHub file exceeds per-file limit: {path}"));
        }
        if sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("invalid SHA256: {path}"));
        }
        total = total.checked_add(size).ok_or("SkillHub package too large")?;
        if total > MAX_TOTAL {
            return Err("SkillHub package exceeds 12 MiB".into());
        }
        files.push(Item { path: path.into(), size, sha256: sha.to_ascii_lowercase() });
    }
    if !files.iter().any(|i| i.path == "SKILL.md") {
        return Err("SkillHub package has no SKILL.md".into());
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    // Pin a *whole version manifest*, not merely SKILL.md. Changes to any script invalidate consent.
    let mut hash = Sha256::new();
    hash.update(b"xiaoban-skillhub-package-v1\0");
    hash.update(version.as_bytes());
    hash.update(b"\0");
    for f in &files {
        hash.update(f.path.as_bytes());
        hash.update(b"\0");
        hash.update(f.size.to_string().as_bytes());
        hash.update(b"\0");
        hash.update(f.sha256.as_bytes());
        hash.update(b"\n");
    }
    Ok(Plan { version: version.to_string(), files, sha256: format!("{:x}", hash.finalize()) })
}

impl Plan {
    pub(crate) fn summary(&self) -> Value {
        json!({
            "version": self.version,
            "sha256": self.sha256,
            "fileCount": self.files.len(),
            "totalBytes": self.files.iter().map(|i| i.size).sum::<u64>(),
            "files": self.files.iter().map(|i| json!({
                "path": i.path, "size": i.size, "sha256": i.sha256,
                "executableScript": is_script(&i.path)
            })).collect::<Vec<_>>()
        })
    }
}

/// Validate every ZIP entry against the API's pinned list. No ZIP extraction API is used.
pub(crate) fn verified_files(slug: &str, plan: &Plan, archive: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if archive.len() > MAX_ARCHIVE {
        return Err("SkillHub download exceeds 12 MiB".into());
    }
    let mut zip = ZipArchive::new(Cursor::new(archive)).map_err(|e| format!("invalid SkillHub zip: {e}"))?;
    if zip.len() > MAX_FILES * 3 {
        return Err("SkillHub archive has too many entries".into());
    }
    // ZIPs may have no wrapper or one enclosing folder (slug or slug-version).
    // Never strip more than one component; every file still needs a manifest match.
    let prefix = if (0..zip.len()).any(|n| zip.by_index(n).is_ok_and(|f| f.name() == "SKILL.md")) {
        String::new()
    } else {
        let mut roots = Vec::new();
        for n in 0..zip.len() {
            let entry = zip.by_index(n).map_err(|e| e.to_string())?;
            if let Some(root) = entry.name().strip_suffix("/SKILL.md") {
                if !root.contains('/') && valid_path(root) {
                    roots.push(root.to_string());
                }
            }
        }
        if roots.len() != 1 {
            return Err("SkillHub zip must contain SKILL.md at root or under one folder".into());
        }
        format!("{}/", roots[0])
    };
    let listed: BTreeMap<&str, &Item> = plan.files.iter().map(|f| (f.path.as_str(), f)).collect();
    let mut found = BTreeMap::new();
    let mut total = 0u64;
    for n in 0..zip.len() {
        let mut entry = zip.by_index(n).map_err(|e| e.to_string())?;
        let raw = entry.name();
        // Directory entries are not materialized, but still must have safe names.
        let bare = raw.trim_end_matches('/');
        if !valid_path(bare) {
            return Err(format!("unsafe ZIP path: {raw}"));
        }
        if let Some(mode) = entry.unix_mode() {
            let kind = mode & 0o170000;
            if kind != 0 && kind != 0o100000 && !(entry.is_dir() && kind == 0o040000) {
                return Err(format!("links and special ZIP entries are forbidden: {raw}"));
            }
        }
        if entry.is_dir() {
            continue;
        }
        let path = raw.strip_prefix(&prefix).ok_or("unexpected SkillHub archive root")?;
        let file = listed.get(path).ok_or_else(|| format!("unlisted ZIP file: {path}"))?;
        if found.contains_key(path) || entry.size() != file.size {
            return Err(format!("duplicate or incorrectly sized ZIP entry: {path}"));
        }
        total = total.checked_add(file.size).ok_or("SkillHub package too large")?;
        if total > MAX_TOTAL {
            return Err("SkillHub package exceeds 12 MiB".into());
        }
        let mut data = Vec::new();
        entry.take(file.size + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
        if data.len() as u64 != file.size
            || format!("{:x}", Sha256::digest(&data)) != file.sha256 {
            return Err(format!("SkillHub SHA256 mismatch: {path}"));
        }
        found.insert(path.to_string(), data);
    }
    if found.len() != listed.len() {
        return Err("SkillHub archive is missing one or more listed files".into());
    }
    Ok(found)
}

struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn install(root: &Path, slug: &str, plan: &Plan, archive: &[u8]) -> Result<(), String> {
    let files = verified_files(slug, plan, archive)?;
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let destination = root.join(slug);
    if destination.exists() {
        return Err("skill already exists; no overwrite permitted".into());
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let temporary = root.join(format!(".skillhub-install-{}-{nonce}", std::process::id()));
    fs::create_dir(&temporary).map_err(|e| e.to_string())?;
    let stage = Stage(temporary);
    for (name, data) in files {
        let target = stage.0.join(&name);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(target, data).map_err(|e| e.to_string())?;
    }
    let lock = json!({"provider":"skillhub","slug":slug,"version":plan.version,
        "manifestSha256":plan.sha256,
        "files":plan.files.iter().map(|i|json!({"path":i.path,"sha256":i.sha256})).collect::<Vec<_>>()});
    fs::write(stage.0.join(".skillhub-lock.json"), lock.to_string()).map_err(|e| e.to_string())?;
    if destination.exists() {
        return Err("skill already exists; no overwrite permitted".into());
    }
    fs::rename(&stage.0, &destination).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn fixture() -> (Plan, Vec<u8>) {
        let skill = b"---\nname: demo-skill\ndescription: demo\n---\n\nUse scripts/hello.py\n";
        let script = b"print('hello')\n";
        let files = [("SKILL.md", skill.as_slice()), ("scripts/hello.py", script.as_slice())];
        let manifest = files.iter().map(|(path, bytes)| {
            json!({"path":path,"size":bytes.len(),"sha256":format!("{:x}", Sha256::digest(bytes))})
        }).collect::<Vec<_>>();
        let plan = plan("1.0.0", &manifest).unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (path, bytes) in files {
            writer.start_file(path, SimpleFileOptions::default()).unwrap();
            writer.write_all(bytes).unwrap();
        }
        (plan, writer.finish().unwrap().into_inner())
    }

    #[test]
    fn archive_validates_every_file() {
        let (plan, zip) = fixture();
        let extracted = verified_files("demo-skill", &plan, &zip).unwrap();
        assert!(extracted.contains_key("scripts/hello.py"));
        let mut wrong = plan.clone();
        wrong.files[1].sha256 = "0".repeat(64);
        assert!(verified_files("demo-skill", &wrong, &zip).is_err());
    }

    #[test]
    fn unsafe_paths_and_case_collisions_are_refused() {
        for p in ["../escape", "/absolute", "scripts\\evil.py", "CON.txt", ".hidden", "scripts/../evil", "scripts//evil", "foo:bar"] {
            assert!(!valid_path(p), "{p}");
        }
        let v = vec![json!({"path":"SKILL.md","size":1,"sha256":"a".repeat(64)}),
            json!({"path":"scripts/A.py","size":1,"sha256":"a".repeat(64)}),
            json!({"path":"scripts/a.py","size":1,"sha256":"a".repeat(64)})];
        assert!(plan("1.0.0", &v).is_err());
    }

    #[test]
    fn pinned_manifest_changes_when_script_changes() {
        let (p, _) = fixture();
        let mut m = p.files.iter().map(|i| json!({"path":i.path,"size":i.size,"sha256":i.sha256})).collect::<Vec<_>>();
        m[1]["sha256"] = json!("f".repeat(64));
        assert_ne!(p.sha256, plan("1.0.0", &m).unwrap().sha256);
    }
}

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

// Guard against oversized manifests without rejecting ordinary multi-file skill packs.
// Byte, path and hash limits below remain independent of this count limit.
pub(crate) const MAX_FILES: usize = 512;
pub(crate) const MAX_FILE: u64 = 4 * 1024 * 1024;
pub(crate) const MAX_TOTAL: u64 = 12 * 1024 * 1024;
pub(crate) const MAX_ARCHIVE: usize = 12 * 1024 * 1024;

/// WorkBuddy plugin manifest directories. With the root `.mcp.json` of connector plugins they are the only
/// hidden paths a package may contain, and only as the first path component (`.codebuddy-plugin/plugin.json`).
pub(crate) const MANIFEST_DIRS: [&str; 2] = [".codebuddy-plugin", ".workbuddy-plugin"];
pub(crate) const ROOT_HIDDEN_FILES: [&str; 1] = [".mcp.json"];
/// WorkBuddy plugins can be large (measured: up to 635 files / 5.5 MB); total bytes stay capped at 12 MiB.
pub(crate) const MAX_PLUGIN_FILES: usize = 2048;

#[derive(Clone, Debug)]
pub(crate) struct Item {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Plan {
    pub version: String,
    pub namespace: Option<String>,
    pub metadata: Option<Value>,
    // Display metadata only; changing an icon does not invalidate reviewed package bytes.
    pub icon_url: Option<String>,
    /// Which service the package came from; recorded in the lock file for display and update checks.
    pub origin: Option<Value>,
    pub files: Vec<Item>,
    pub sha256: String,
}

pub(crate) fn valid_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 240 || path.starts_with('/') || path.contains('\\')
        || path.contains(':') || path.chars().any(char::is_control) {
        return false;
    }
    let parts: Vec<_> = path.split('/').collect();
    // WorkBuddy plugins nest references deeper than 8 levels (measured: 9); 12 is still a sane bound.
    if parts.len() > 12 {
        return false;
    }
    parts.iter().enumerate().all(|(index, p)| {
        let hidden_ok = index == 0 && (MANIFEST_DIRS.contains(p) || (parts.len() == 1 && ROOT_HIDDEN_FILES.contains(p)));
        if p.is_empty() || *p == "." || *p == ".." || p.len() > 100
            || (p.ends_with(' ') || p.ends_with('.')) || (p.starts_with('.') && !hidden_ok) {
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
    if values.is_empty() {
        return Err("SkillHub returned an empty file manifest; no files can be installed".into());
    }
    if values.len() > MAX_FILES {
        return Err(format!(
            "SkillHub manifest contains {} files, exceeding the supported limit of {MAX_FILES}",
            values.len()
        ));
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
    Ok(Plan { version: version.to_string(), namespace: None, metadata: None, icon_url: None, origin: None, files, sha256: format!("{:x}", hash.finalize()) })
}

/// The 小伴市场 pins a version with its own manifest digest: SHA-256 over the sorted lines
/// `"<file sha256>  <path>\n"`. The server computes the same value when a package is listed,
/// so a web link, the review dialog and the installed lock all show one comparable string.
pub(crate) fn market_plan(version: &str, files: &[Value]) -> Result<Plan, String> {
    let mut plan = plan(version, files)?;
    let mut hash = Sha256::new();
    for f in &plan.files {
        hash.update(format!("{}  {}\n", f.sha256, f.path).as_bytes());
    }
    plan.sha256 = format!("{:x}", hash.finalize());
    plan.origin = Some(json!({"service":"xiaoban-market"}));
    Ok(plan)
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
pub(crate) fn verified_files(_slug: &str, plan: &Plan, archive: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if archive.len() > MAX_ARCHIVE {
        return Err("SkillHub download exceeds 12 MiB".into());
    }
    let mut zip = ZipArchive::new(Cursor::new(archive)).map_err(|e| format!("invalid SkillHub zip: {e}"))?;
    if zip.len() > MAX_FILES * 3 {
        return Err("SkillHub archive has too many entries".into());
    }
    // ZIPs may have no wrapper or one enclosing folder (slug or slug-version).
    // Never strip more than one component; every file still needs a manifest match.
    let mut has_root_skill = false;
    let mut roots = Vec::new();
    // ZIP 9 returns Result<Cow<str>, ZipError> from name(); reject invalid names.
    for n in 0..zip.len() {
        let entry = zip.by_index(n).map_err(|e| e.to_string())?;
        let name = entry.name().map_err(|e| format!("invalid ZIP entry name: {e}"))?;
        if name.as_ref() == "SKILL.md" {
            has_root_skill = true;
        }
        if let Some(root) = name.strip_suffix("/SKILL.md") {
            if !root.contains('/') && valid_path(root) {
                roots.push(root.to_string());
            }
        }
    }
    let prefix = if has_root_skill {
        String::new()
    } else {
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
        // Own the decoded name before consuming the entry; do not borrow from it
        // across entry.take() below.
        let raw = entry.name().map_err(|e| format!("invalid ZIP entry name: {e}"))?.into_owned();
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
        if found.contains_key(path) { return Err(format!("duplicate ZIP entry: {path}")); }
        if entry.size() != file.size {
            return Err(format!("incorrectly sized ZIP entry: {path} (expected {}, received {})", file.size, entry.size()));
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

/// Canonical file endpoints avoid registry ZIPs which resolve a different publisher.
pub(crate) fn canonical_files(plan:&Plan,mut read:impl FnMut(&Item)->Result<Vec<u8>,String>)->Result<BTreeMap<String,Vec<u8>>,String>{
    let mut files=BTreeMap::new();
    for item in &plan.files{
        let data=read(item)?;
        if data.len() as u64!=item.size || format!("{:x}",Sha256::digest(&data))!=item.sha256{
            return Err(format!("SkillHub canonical file SHA256 or size mismatch: {}",item.path));
        }
        files.insert(item.path.clone(),data);
    }
    Ok(files)
}
pub(crate) fn install(root: &Path, slug: &str, plan: &Plan, files: &BTreeMap<String,Vec<u8>>) -> Result<(), String> {
    install_with(root, slug, plan, files, None)
}
/// Swap an installed package for a newer verified one. The old directory is moved (not deleted)
/// into `backups`, and moved back if the swap fails, so an update can never leave the skill missing.
pub(crate) fn replace(root: &Path, slug: &str, plan: &Plan, files: &BTreeMap<String,Vec<u8>>, backups: &Path) -> Result<(), String> {
    install_with(root, slug, plan, files, Some(backups))
}
fn install_with(root: &Path, slug: &str, plan: &Plan, files: &BTreeMap<String,Vec<u8>>, backups: Option<&Path>) -> Result<(), String> {
    if files.len()!=plan.files.len(){return Err("incomplete SkillHub package".into());}
    // Validate the entire set again before any filesystem writes, independent of transport.
    canonical_files(plan,|item|files.get(&item.path).cloned().ok_or("missing SkillHub file".into()))?;
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let destination = root.join(slug);
    if destination.exists() && backups.is_none() {
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
        "namespace":plan.namespace,"metadata":plan.metadata,"iconUrl":plan.icon_url,"origin":plan.origin,"manifestSha256":plan.sha256,
        "files":plan.files.iter().map(|i|json!({"path":i.path,"sha256":i.sha256})).collect::<Vec<_>>()});
    fs::write(stage.0.join(".skillhub-lock.json"), lock.to_string()).map_err(|e| e.to_string())?;
    let Some(backups) = backups else {
        if destination.exists() {
            return Err("skill already exists; no overwrite permitted".into());
        }
        fs::rename(&stage.0, &destination).map_err(|e| e.to_string())?;
        return Ok(());
    };
    // Only a package this app installed from the market may be replaced.
    let old_lock = fs::read(destination.join(".skillhub-lock.json")).ok()
        .and_then(|v| serde_json::from_slice::<Value>(&v).ok())
        .ok_or("the installed skill is not managed by the store; it will not be replaced")?;
    if old_lock["slug"] != slug || old_lock["origin"]["service"] != "xiaoban-market" {
        return Err("the installed skill did not come from the market; it will not be replaced".into());
    }
    fs::create_dir_all(backups).map_err(|e| e.to_string())?;
    let backup = backups.join(format!("{slug}-{}-{nonce}", old_lock["version"].as_str().unwrap_or("old")));
    fs::rename(&destination, &backup).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&stage.0, &destination) {
        let _ = fs::rename(&backup, &destination);
        return Err(e.to_string());
    }
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
    fn large_but_bounded_manifests_are_supported() {
        let sha = format!("{:x}", Sha256::digest(b"x"));
        let mut values = vec![json!({"path":"SKILL.md","size":1,"sha256":sha})];
        // Regression: the previous 100-file cap rejected otherwise valid skills.
        for i in 0..120 {
            values.push(json!({"path":format!("references/doc-{i:03}.md"),"size":1,"sha256":sha}));
        }
        let package = plan("1.2.3", &values).unwrap();
        assert_eq!(package.files.len(), 121);
        assert_eq!(package.summary()["fileCount"], 121);
        // Increasing file count must not bypass the 12 MiB overall limit.
        values[1]["size"] = json!(4 * 1024 * 1024);
        values[2]["size"] = json!(4 * 1024 * 1024);
        values[3]["size"] = json!(4 * 1024 * 1024);
        assert!(plan("1.2.3", &values).unwrap_err().contains("exceeds 12 MiB"));
    }

    #[test]
    fn file_count_errors_distinguish_empty_and_excessive_manifests() {
        assert!(plan("1.0.0", &[]).unwrap_err().contains("empty file manifest"));
        let sha = "a".repeat(64);
        let mut values = vec![json!({"path":"SKILL.md","size":1,"sha256":sha})];
        for i in 1..MAX_FILES {
            values.push(json!({"path":format!("references/ref-{i:04}.md"),"size":1,"sha256":sha}));
        }
        assert_eq!(plan("1.0.0", &values).unwrap().files.len(), MAX_FILES);
        values.push(json!({"path":"references/extra.md","size":1,"sha256":sha}));
        let err = plan("1.0.0", &values).unwrap_err();
        assert!(err.contains("513 files") && err.contains("512"));
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
    fn wrapped_archive_with_directory_entry_validates() {
        let (plan, bare_zip) = fixture();
        let mut original = ZipArchive::new(Cursor::new(bare_zip)).unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.add_directory("demo-skill/", SimpleFileOptions::default()).unwrap();
        for n in 0..original.len() {
            let mut entry = original.by_index(n).unwrap();
            let name = entry.name().unwrap().into_owned();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            writer.start_file(format!("demo-skill/{name}"), SimpleFileOptions::default()).unwrap();
            writer.write_all(&data).unwrap();
        }
        let archive = writer.finish().unwrap().into_inner();
        let extracted = verified_files("demo-skill", &plan, &archive).unwrap();
        assert_eq!(extracted.len(), plan.files.len());
    }

    #[test]
    fn workbuddy_manifest_paths_are_allowed_only_where_workbuddy_puts_them() {
        for ok in [".codebuddy-plugin/plugin.json", ".workbuddy-plugin/plugin.json", ".mcp.json", "ai.workbuddy/token-schema.json", "a/b/c/d/e/f/g/h/i.md"] {
            assert!(valid_path(ok), "{ok}");
        }
        for bad in ["a/.mcp.json", "a/.codebuddy-plugin/plugin.json", ".codebuddy-plugin/.env", ".env", ".mcp.json/x", "a/b/c/d/e/f/g/h/i/j/k/l/m.md"] {
            assert!(!valid_path(bad), "{bad}");
        }
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
    #[test]
    fn canonical_downloads_require_all_reviewed_bytes_before_publishing(){
        let (plan,archive)=fixture();let files=verified_files("demo-skill",&plan,&archive).unwrap();
        assert_eq!(canonical_files(&plan,|item|Ok(files[&item.path].clone())).unwrap(),files);
        assert!(canonical_files(&plan,|_|Ok(b"changed".to_vec())).is_err());
        let root=std::env::temp_dir().join(format!("canonical-skill-test-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let mut tampered=files.clone();tampered.insert("scripts/hello.py".into(),b"bad".to_vec());
        assert!(install(&root,"demo-skill",&plan,&tampered).is_err());assert!(!root.exists());
        let mut missing=files.clone();missing.remove("SKILL.md");assert!(install(&root,"demo-skill",&plan,&missing).is_err());
        install(&root,"demo-skill",&plan,&files).unwrap();assert_eq!(fs::read(root.join("demo-skill/SKILL.md")).unwrap(),files["SKILL.md"]);
        fs::remove_dir_all(root).unwrap();
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io::{Cursor, Read, Write}, path::{Path, PathBuf}, sync::Mutex};

pub type Result<T> = std::result::Result<T, String>;
pub const MAX_ARCHIVE: usize = 12 * 1024 * 1024;
const MAX_FILE: u64 = 4 * 1024 * 1024;
const MAX_TOTAL: u64 = 32 * 1024 * 1024;
const MAX_FILES: usize = 512;
// Desktop is single-instance. All Store instances in this process share publication ownership.
static STORE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub package_name: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub verified_app_versions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatchMeta {
    pub app_version: String,
    pub based_on: String,
    pub observed_at: DateTime<Utc>,
    pub evidence: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl PatchMeta {
    pub fn applies(&self, meta: &Metadata, app_version: Option<&str>, now: DateTime<Utc>) -> bool {
        app_version == Some(self.app_version.as_str()) && self.based_on == meta.version
            && self.observed_at <= now && self.expires_at.is_none_or(|until| until > now)
    }
}

#[derive(Clone)]
pub struct Store { root: PathBuf }

#[derive(Debug, Serialize)]
pub struct TextPage {
    pub source: String,
    pub content: String,
    pub next_offset: usize,
    pub has_more: bool,
    pub verified_app_version: bool,
}

pub fn valid_package(value: &str) -> bool {
    value.len() <= 200 && value.contains('.') && value.split('.').all(|p| {
        !p.is_empty() && p.as_bytes()[0].is_ascii_alphabetic()
            && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    })
}

pub(crate) fn valid_path(value: &str) -> bool {
    !value.is_empty() && value.len() <= 220 && !value.contains('\\') && value.split('/').all(|p| {
        let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
        !p.is_empty() && p != "." && p != ".." && !p.ends_with(['.', ' '])
            && !p.chars().any(|c| c.is_control() || ":*?\"<>|".contains(c))
            && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            && !(stem.len() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'0'..=b'9'))
    })
}

fn allowed_file(path: &str) -> bool {
    matches!(path.rsplit('.').next(), Some("md" | "txt" | "json" | "yaml" | "yml" | "png" | "jpg" | "jpeg" | "webp"))
}

pub(crate) fn safe_file(root: &Path, relative: &str) -> Result<PathBuf> {
    if !valid_path(relative) { return Err("invalid relative pack path".into()); }
    let mut at = root.to_path_buf();
    // Reject links/junctions even when they currently point inside the package.
    if fs::symlink_metadata(&at).map_err(|e| e.to_string())?.file_type().is_symlink() {
        return Err("pack root is a link".into());
    }
    let real_root = root.canonicalize().map_err(|e| e.to_string())?;
    for part in relative.split('/') {
        at.push(part);
        if let Ok(meta) = fs::symlink_metadata(&at) {
            if meta.file_type().is_symlink() { return Err("links in packs are not allowed".into()); }
            let real = at.canonicalize().map_err(|e| e.to_string())?;
            if !real.starts_with(&real_root) { return Err("pack path escapes root".into()); }
        }
    }
    Ok(at)
}

fn read_small(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path).map_err(|e| e.to_string())?.take(MAX_FILE + 1)
        .read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_FILE { return Err("pack file exceeds 4 MiB".into()); }
    Ok(bytes)
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("missing parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    tmp.write_all(bytes).and_then(|_| tmp.as_file().sync_all()).map_err(|e| e.to_string())?;
    tmp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn split_patch(bytes: &[u8]) -> Result<(PatchMeta, String)> {
    let text = std::str::from_utf8(bytes).map_err(|_| "patch must be UTF-8")?;
    let text = text.strip_prefix("---\n").ok_or("patch metadata is missing")?;
    let (meta, content) = text.split_once("\n---\n").ok_or("invalid patch metadata")?;
    Ok((serde_yaml_ng::from_str(meta).map_err(|e| e.to_string())?, content.to_string()))
}

pub fn page(text: &str, source: &str, offset: usize, limit: usize, verified: bool) -> Result<TextPage> {
    if offset > text.len() || !text.is_char_boundary(offset) { return Err("offset is not a UTF-8 byte boundary".into()); }
    let mut end = offset.saturating_add(limit.clamp(4, 8192)).min(text.len());
    while !text.is_char_boundary(end) { end -= 1; }
    Ok(TextPage { source: source.into(), content: text[offset..end].into(), next_offset: end,
        has_more: end < text.len(), verified_app_version: verified })
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }

    pub fn contains(&self, package: &str) -> Result<bool> {
        if !valid_package(package) { return Err("invalid package name".into()); }
        if !self.root.exists() { return Ok(false); }
        Ok(self.dir(package)?.exists())
    }

    pub fn asset(&self, package: &str, path: &str) -> Result<Vec<u8>> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        if !path.starts_with("assets/") || !valid_path(path)
            || !matches!(path.rsplit('.').next(), Some("png" | "jpg" | "jpeg" | "webp")) {
            return Err("expected an image under assets/".into());
        }
        read_small(&safe_file(&self.dir(package)?, &format!("basic/{path}"))?)
    }

    fn dir(&self, package: &str) -> Result<PathBuf> {
        if !valid_package(package) { return Err("invalid Android package name".into()); }
        safe_file(&self.root, package)
    }

    fn metadata_unlocked(&self, package: &str) -> Result<Metadata> {
        let dir = self.dir(package)?;
        let path = safe_file(&dir, "basic/package.json")?;
        let meta: Metadata = serde_json::from_slice(&read_small(&path)?).map_err(|e| e.to_string())?;
        validate_metadata(&meta, package)?;
        Ok(meta)
    }

    pub fn metadata(&self, package: &str) -> Result<Metadata> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        self.metadata_unlocked(package)
    }

    /// A bounded local index, not the contents of every skill/page.
    pub fn discover(&self, query: &str) -> Result<serde_json::Value> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        if !self.root.exists() { return Ok(serde_json::json!({"packs":[],"total":0})); }
        let query = query.to_lowercase();
        let mut packs = Vec::new();
        let mut warnings = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|e| e.to_string())?.take(256) {
            let entry = entry.map_err(|e| e.to_string())?;
            let package = entry.file_name().to_string_lossy().into_owned();
            if !valid_package(&package) { continue; }
            match self.metadata_unlocked(&package) {
                Ok(meta) if format!("{} {} {}", meta.package_name, meta.name, meta.description).to_lowercase().contains(&query) => packs.push(meta),
                Ok(_) => {},
                Err(error) => warnings.push(format!("{package}: {error}")),
            }
        }
        packs.sort_by(|a,b| a.package_name.cmp(&b.package_name));
        let total = packs.len();
        packs.truncate(20);
        warnings.truncate(5);
        Ok(serde_json::json!({"packs":packs,"total":total,"warnings":warnings}))
    }

    pub fn read(&self, package: &str, path: &str, app_version: Option<&str>, offset: usize, limit: usize, now: DateTime<Utc>) -> Result<TextPage> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        let meta = self.metadata_unlocked(package)?;
        let dir = self.dir(package)?;
        let verified = app_version.is_some_and(|v| meta.verified_app_versions.iter().any(|a| a == v));
        if path == "skill.md" {
            return page(&String::from_utf8(read_small(&safe_file(&dir, path)?)?).map_err(|_| "not UTF-8")?, "skill.md", offset, limit, verified);
        }
        if !valid_path(path) || !matches!(path.split('/').next(), Some("tasks" | "pages" | "knowledge" | "routes" | "assets")) {
            return Err("read paths are relative to basic/; use skill.md for the entry point".into());
        }
        // Structured route maps must go through routes(), which merges IDs and filters expiry.
        if path.starts_with("routes/") && (path.ends_with(".yaml") || path.ends_with(".yml")) {
            return Err("use phone_routes for YAML route maps".into());
        }
        let patch = safe_file(&dir, &format!("patch/{path}"))?;
        if patch.exists() {
            let (info, text) = split_patch(&read_small(&patch)?)?;
            if info.applies(&meta, app_version, now) {
                // A learned correction is not independent certification of the whole APP version.
                return page(&text, &format!("patch/{path}"), offset, limit, verified);
            }
        }
        let text = String::from_utf8(read_small(&safe_file(&dir, &format!("basic/{path}"))?)?).map_err(|_| "not a UTF-8 text asset")?;
        page(&text, &format!("basic/{path}"), offset, limit, verified)
    }

    pub fn patches(&self, package: &str, app_version: Option<&str>, now: DateTime<Utc>) -> Result<serde_json::Value> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        let meta = self.metadata_unlocked(package)?;
        let dir = self.dir(package)?;
        let mut files = Vec::new();
        collect(&safe_file(&dir, "patch")?, "", &mut files, &mut 0)?;
        let mut result = Vec::new();
        for (relative, _) in files {
            let file = safe_file(&dir, &format!("patch/{relative}"))?;
            let (info, _) = split_patch(&read_small(&file)?)?;
            result.push(serde_json::json!({"path":relative,"applicable":info.applies(&meta,app_version,now),"metadata":info}));
        }
        Ok(serde_json::json!({"patches":result}))
    }

    /// Resolve a bounded workflow atomically; never concatenate pages from changing pack revisions.
    pub fn workflow_document(&self,package:&str,path:&str,app_version:Option<&str>,now:DateTime<Utc>)->Result<(String,String)> {
        let _guard=STORE_LOCK.lock().map_err(|_|"pack store lock poisoned")?;
        if !valid_path(path)||!path.starts_with("tasks/")||!path.ends_with(".yaml") {return Err("workflow must be a tasks/*.yaml logical path".into());}
        let meta=self.metadata_unlocked(package)?;let dir=self.dir(package)?;
        let patch=safe_file(&dir,&format!("patch/{path}"))?;
        let (source,text)=if patch.exists() {
            let (info,text)=split_patch(&read_small(&patch)?)?;
            if info.applies(&meta,app_version,now) {(format!("patch/{path}"),text)}
            else {(format!("basic/{path}"),String::from_utf8(read_small(&safe_file(&dir,&format!("basic/{path}"))?)?).map_err(|_|"workflow is not UTF-8")?)}
        } else {(format!("basic/{path}"),String::from_utf8(read_small(&safe_file(&dir,&format!("basic/{path}"))?)?).map_err(|_|"workflow is not UTF-8")?)};
        if text.len()>64*1024 {return Err("workflow exceeds 64 KiB".into());}
        Ok((source,text))
    }

    pub fn save_patch(&self, package: &str, path: &str, content: &str, info: PatchMeta) -> Result<()> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        let meta = self.metadata_unlocked(package)?;
        if !valid_path(path) || !matches!(path.split('/').next(), Some("tasks" | "pages" | "knowledge" | "routes"))
            || !matches!(path.rsplit('.').next(), Some("md" | "yaml" | "yml")) || path == "tasks/index.md" {
            return Err("patch must be a task, page, knowledge document or route map; discovery indexes are generated".into());
        }
        if info.based_on != meta.version || info.app_version.trim().is_empty() || info.evidence.trim().is_empty()
            || info.app_version.len() > 100 || info.evidence.len() > 4000 || info.observed_at > Utc::now()
            || info.expires_at.is_some_and(|until| until <= info.observed_at) {
            return Err("patch requires the current base version, exact app version, evidence and valid observation/expiry times".into());
        }
        if path.starts_with("routes/") {
            let graph: crate::routes::Graph = serde_yaml_ng::from_str(content).map_err(|e| e.to_string())?;
            graph.validate(false)?;
            if (path.starts_with("routes/temporary/") || graph.has_temporary()) && info.expires_at.is_none() {
                return Err("temporary routes must expire".into());
            }
        }
        let header = serde_yaml_ng::to_string(&info).map_err(|e| e.to_string())?;
        let document = format!("---\n{header}---\n{content}");
        if document.len() as u64 > MAX_FILE { return Err("patch too large".into()); }
        let dir = self.dir(package)?;
        let target = safe_file(&dir, &format!("patch/{path}"))?;
        let mut files = Vec::new();
        let mut total = 0;
        collect(&safe_file(&dir, "patch")?, "", &mut files, &mut total)?;
        let previous = files.iter().find(|(p,_)| p == path).map(|(_,n)| *n).unwrap_or(0);
        if (previous == 0 && files.len() >= MAX_FILES) || total - previous + document.len() as u64 > MAX_TOTAL {
            return Err("patch budget exceeded".into());
        }
        atomic_write(&target, document.as_bytes())
    }

    /// No downloaded archive may modify learned patches. SHA-256 pins the selected bytes.
    pub fn install(&self, package: &str, bytes: &[u8], sha256: &str) -> Result<Metadata> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        if !valid_package(package) || bytes.len() > MAX_ARCHIVE || sha256.len() != 64
            || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) || format!("{:x}", Sha256::digest(bytes)) != sha256.to_lowercase() {
            return Err("invalid package, archive size or SHA-256".into());
        }
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let stage = tempfile::tempdir_in(&self.root).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        if archive.len() > MAX_FILES { return Err("too many ZIP entries".into()); }
        let mut names = BTreeSet::new();
        let mut total = 0u64;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
            let raw = entry.name().map_err(|e| e.to_string())?.into_owned();
            let name = raw.strip_prefix(&format!("{package}/")).unwrap_or(&raw).trim_end_matches('/');
            if name.is_empty() && entry.is_dir() { continue; }
            if !valid_path(name) || entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
                || !names.insert(name.to_lowercase()) {
                return Err("ZIP contains unsafe, linked or duplicate paths".into());
            }
            if name != "skill.md" && name != "basic" && !name.starts_with("basic/") {
                return Err("ZIP may contain only skill.md and basic/; patch/ belongs to this device".into());
            }
            if entry.is_dir() { continue; }
            if !allowed_file(name) || entry.size() > MAX_FILE { return Err("unsupported or oversized pack file".into()); }
            total = total.checked_add(entry.size()).ok_or("ZIP size overflow")?;
            if total > MAX_TOTAL { return Err("uncompressed ZIP budget exceeded".into()); }
            let mut data = Vec::new();
            (&mut entry).take(MAX_FILE + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
            if data.len() as u64 != entry.size() { return Err("ZIP entry size mismatch".into()); }
            let target = safe_file(stage.path(), name)?;
            fs::create_dir_all(target.parent().ok_or("invalid ZIP parent")?).map_err(|e| e.to_string())?;
            fs::write(target, data).map_err(|e| e.to_string())?;
        }
        let meta: Metadata = serde_json::from_slice(&read_small(&stage.path().join("basic/package.json"))?).map_err(|e| e.to_string())?;
        validate_metadata(&meta, package)?;
        let skill = String::from_utf8(read_small(&stage.path().join("skill.md"))?).map_err(|_| "skill.md is not UTF-8")?;
        if skill.trim().is_empty() || skill.len() > 64 * 1024 { return Err("invalid skill.md".into()); }
        let mut files = Vec::new();
        collect(&stage.path().join("basic"), "", &mut files, &mut 0)?;
        for (name, _) in files {
            if name.starts_with("routes/") && (name.ends_with(".yaml") || name.ends_with(".yml")) {
                let graph: crate::routes::Graph = serde_yaml_ng::from_slice(&read_small(&stage.path().join("basic").join(name))?).map_err(|e| e.to_string())?;
                graph.validate(true)?;
            }
        }
        let destination = self.dir(package)?;
        fs::create_dir_all(stage.path().join("patch")).map_err(|e| e.to_string())?;
        if destination.exists() {
            // A case-insensitive host must not let a differently cased Android identity replace this pack.
            self.metadata_unlocked(package)?;
            let patch = safe_file(&destination, "patch")?;
            let mut files = Vec::new();
            collect(&patch, "", &mut files, &mut 0)?;
            for (name, _) in files {
                let target = safe_file(stage.path(), &format!("patch/{name}"))?;
                fs::create_dir_all(target.parent().ok_or("invalid patch path")?).map_err(|e| e.to_string())?;
                fs::copy(safe_file(&patch, &name)?, target).map_err(|e| e.to_string())?;
            }
        }
        let backup_holder = tempfile::tempdir_in(&self.root).map_err(|e| e.to_string())?;
        let backup = backup_holder.path().join("previous");
        let existed = destination.exists();
        if existed { fs::rename(&destination, &backup).map_err(|e| e.to_string())?; }
        if let Err(error) = fs::rename(stage.path(), &destination) {
            if existed {
                if let Err(rollback) = fs::rename(&backup, &destination) {
                    // Retain the only recoverable copy if the filesystem blocks rollback.
                    let recovery = backup_holder.keep();
                    return Err(format!("publish failed: {error}; rollback failed: {rollback}; recovery: {}", recovery.display()));
                }
            }
            return Err(error.to_string());
        }
        Ok(meta)
    }

    pub fn routes(&self, package: &str, scope: &str, app_version: Option<&str>, include_temporary: bool, now: DateTime<Utc>) -> Result<crate::routes::Graph> {
        let _guard = STORE_LOCK.lock().map_err(|_| "pack store lock poisoned")?;
        let meta = self.metadata_unlocked(package)?;
        let dir = self.dir(package)?;
        if !valid_path(scope) || scope.contains('.') { return Err("invalid route scope".into()); }
        let mut graph = crate::routes::Graph::default();
        let mut scopes = vec!["core"];
        if scope != "core" { scopes.push(scope); }
        for scope in scopes {
            let relative = format!("routes/{scope}.yaml");
            let base = safe_file(&dir, &format!("basic/{relative}"))?;
            let patch = safe_file(&dir, &format!("patch/{relative}"))?;
            let mut found = false;
            if base.exists() {
                let data: crate::routes::Graph = serde_yaml_ng::from_slice(&read_small(&base)?).map_err(|e| e.to_string())?;
                data.validate(true)?;
                graph.overlay(data);
                found = true;
            }
            if patch.exists() {
                let (info, text) = split_patch(&read_small(&patch)?)?;
                if info.applies(&meta, app_version, now) {
                    let data: crate::routes::Graph = serde_yaml_ng::from_str(&text).map_err(|e| e.to_string())?;
                    data.validate(false)?;
                    if data.has_temporary() && info.expires_at.is_none() { return Err("temporary route patch has no expiry".into()); }
                    graph.overlay(data);
                    found = true;
                }
            }
            if !found { return Err(format!("route scope not found or not applicable: {scope}")); }
        }
        graph.filter(include_temporary, now);
        graph.validate_resolved()?;
        Ok(graph)
    }
}

fn validate_metadata(meta: &Metadata, package: &str) -> Result<()> {
    if meta.package_name != package || meta.version.is_empty() || meta.version.len() > 100
        || meta.name.is_empty() || meta.name.len() > 200 || meta.description.len() > 2000
        || meta.verified_app_versions.len() > 100 || meta.verified_app_versions.iter().any(|v| v.is_empty() || v.len() > 100) {
        return Err("invalid pack identity or metadata".into());
    }
    Ok(())
}

fn collect(root: &Path, prefix: &str, files: &mut Vec<(String,u64)>, total: &mut u64) -> Result<()> {
    if !root.exists() { return Ok(()); }
    if fs::symlink_metadata(root).map_err(|e| e.to_string())?.file_type().is_symlink() { return Err("linked pack directory".into()); }
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = format!("{prefix}{}", entry.file_name().to_str().ok_or("non UTF-8 pack path")?);
        if !valid_path(&name) || entry.file_type().map_err(|e| e.to_string())?.is_symlink() { return Err("unsafe pack path".into()); }
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        if metadata.is_dir() {
            if name.split('/').count() > 12 { return Err("pack nesting too deep".into()); }
            collect(&entry.path(), &format!("{name}/"), files, total)?;
        } else if metadata.is_file() {
            *total = total.checked_add(metadata.len()).ok_or("pack size overflow")?;
            if metadata.len() > MAX_FILE || *total > MAX_TOTAL || files.len() >= MAX_FILES { return Err("pack budget exceeded".into()); }
            files.push((name, metadata.len()));
        } else { return Err("unsupported pack entry".into()); }
    }
    Ok(())
}

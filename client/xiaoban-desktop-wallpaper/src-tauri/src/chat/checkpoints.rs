//! "撤销这轮的文件改动": before an answer first changes a file in the working folder with `write_file` or
//! `edit_file`, the file as it was is copied aside, so the whole answer's file changes can be put back later.
//!
//! Only the file tools are covered. What a shell command or a background process changes is not recorded, the same
//! limit Claude Code's checkpoints have; the run history still shows those commands.
//!
//! Layout: `chat/checkpoints/<session>/<run>/manifest.json` plus one numbered copy per file that existed before.

use agent_kernel::{Middleware, ToolCall, ToolCtx, ToolGate, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Files larger than this are not copied; the manifest says so and undo leaves them alone.
const MAX_BACKUP_BYTES: u64 = 16 * 1024 * 1024;
/// Answers kept per conversation; older checkpoints are removed when a new one is made.
const KEEP_RUNS: usize = 20;
const MANIFEST: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Where the file is, relative to the working folder.
    pub path: String,
    /// The file did not exist before the answer: undo removes it.
    pub created: bool,
    /// The copy of the original, inside the checkpoint folder. `None` for a created or a too-large file.
    pub backup: Option<String>,
    /// Too large to copy: undo cannot restore it and says so.
    #[serde(default)]
    pub skipped: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub run_id: String,
    pub workspace: String,
    pub created_at: i64,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub undone: bool,
}

/// What undo did, for the page to report.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UndoReport {
    pub restored: Vec<String>,
    pub removed: Vec<String>,
    /// Could not be put back (too large to have been copied, or a write failed), with the reason.
    pub failed: Vec<(String, String)>,
}

pub struct Checkpoints {
    root: PathBuf,
    workspace: PathBuf,
    /// The manifest of the answer being recorded, saved after each new entry.
    current: Mutex<Option<(PathBuf, Manifest)>>,
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or_default()
}

fn safe(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

impl Checkpoints {
    /// `root` is `chat/checkpoints`; `workspace` the folder the file tools write in.
    pub fn new(root: impl Into<PathBuf>, workspace: &Path) -> Self {
        let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());
        Self { root: root.into(), workspace, current: Mutex::new(None) }
    }

    /// The file a write or edit call is about to change, if it is inside the working folder.
    fn target(&self, args: &Value) -> Option<(PathBuf, String)> {
        let raw = args.get("path")?.as_str()?.trim();
        if raw.is_empty() {
            return None;
        }
        let path = Path::new(raw);
        let joined = if path.is_absolute() { path.to_path_buf() } else { self.workspace.join(path) };
        let parent = joined.parent()?;
        // The parent may not exist yet (write_file creates folders): find the nearest existing ancestor.
        let mut ancestor = parent.to_path_buf();
        let mut rest = Vec::new();
        while !ancestor.exists() {
            rest.push(ancestor.file_name()?.to_os_string());
            ancestor = ancestor.parent()?.to_path_buf();
        }
        let mut real = ancestor.canonicalize().ok()?;
        for part in rest.into_iter().rev() {
            real.push(part);
        }
        real.push(joined.file_name()?);
        let relative = real.strip_prefix(&self.workspace).ok()?.to_string_lossy().replace('\\', "/");
        (!relative.is_empty() && !relative.split('/').any(|p| p == "..")).then_some((real, relative))
    }

    fn record(&self, session: &str, run_id: &str, args: &Value) -> Result<(), String> {
        if !safe(session) || !safe(run_id) {
            return Ok(());
        }
        let Some((file, relative)) = self.target(args) else { return Ok(()) };
        let mut current = self.current.lock().unwrap_or_else(|e| e.into_inner());
        let dir = self.root.join(session).join(run_id);
        if current.as_ref().is_none_or(|(d, _)| *d != dir) {
            let manifest = load(&dir).unwrap_or_else(|| Manifest {
                run_id: run_id.to_string(),
                workspace: self.workspace.display().to_string(),
                created_at: now(),
                ..Default::default()
            });
            *current = Some((dir.clone(), manifest));
            prune(&self.root.join(session), run_id);
        }
        let (dir, manifest) = current.as_mut().expect("set above");
        if manifest.entries.iter().any(|e| e.path == relative) {
            return Ok(());
        }
        fs::create_dir_all(&*dir).map_err(|e| e.to_string())?;
        let entry = match fs::symlink_metadata(&file) {
            Ok(meta) if meta.is_file() && meta.len() <= MAX_BACKUP_BYTES => {
                let name = format!("{}.bak", manifest.entries.len());
                fs::copy(&file, dir.join(&name)).map_err(|e| e.to_string())?;
                Entry { path: relative, created: false, backup: Some(name), skipped: false }
            }
            Ok(meta) if meta.is_file() => Entry { path: relative, created: false, backup: None, skipped: true },
            // A folder or a link: the file tools refuse those, nothing to keep.
            Ok(_) => return Ok(()),
            Err(_) => Entry { path: relative, created: true, backup: None, skipped: false },
        };
        manifest.entries.push(entry);
        save(dir, manifest)
    }
}

fn load(dir: &Path) -> Option<Manifest> {
    serde_json::from_slice(&fs::read(dir.join(MANIFEST)).ok()?).ok()
}

fn save(dir: &Path, manifest: &Manifest) -> Result<(), String> {
    let temporary = dir.join(format!("{MANIFEST}.tmp"));
    fs::write(&temporary, serde_json::to_vec_pretty(manifest).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(&temporary, dir.join(MANIFEST)).map_err(|e| e.to_string())
}

/// Keeps the newest `KEEP_RUNS` checkpoints of a conversation, besides the one being made.
fn prune(session_dir: &Path, keep: &str) {
    let Ok(read) = fs::read_dir(session_dir) else { return };
    let mut runs: Vec<(i64, PathBuf)> = read
        .flatten()
        .filter(|e| e.file_name().to_string_lossy() != keep)
        .filter_map(|e| load(&e.path()).map(|m| (m.created_at, e.path())))
        .collect();
    runs.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, dir) in runs.into_iter().skip(KEEP_RUNS.saturating_sub(1)) {
        let _ = fs::remove_dir_all(dir);
    }
}

impl Middleware for Checkpoints {
    fn before_tool(&self, ctx: &ToolCtx, _call: &ToolCall, args: &Value, spec: &ToolSpec) -> ToolGate {
        if matches!(spec.name.as_str(), "write_file" | "edit_file") {
            // A copy that cannot be made must not stop the work; undo then simply has less to put back.
            let _ = self.record(ctx.session(), ctx.run_id(), args);
        }
        ToolGate::Proceed
    }
}

/// The answers of a conversation that changed files, newest first.
pub fn list(root: &Path, session: &str) -> Vec<Manifest> {
    if !safe(session) {
        return Vec::new();
    }
    let Ok(read) = fs::read_dir(root.join(session)) else { return Vec::new() };
    let mut runs: Vec<Manifest> = read.flatten().filter_map(|e| load(&e.path())).filter(|m| !m.entries.is_empty()).collect();
    runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    runs
}

/// Puts back every file one answer changed. Files are restored even if they were changed again since; the page
/// asks the user first and says so.
pub fn undo(root: &Path, session: &str, run_id: &str) -> Result<UndoReport, String> {
    if !safe(session) || !safe(run_id) {
        return Err("记录编号不对".into());
    }
    let dir = root.join(session).join(run_id);
    let mut manifest = load(&dir).ok_or("没有这轮回答的文件记录")?;
    if manifest.undone {
        return Err("这轮回答的文件改动已经撤销过了".into());
    }
    let workspace = PathBuf::from(&manifest.workspace);
    let mut report = UndoReport::default();
    for entry in manifest.entries.iter().rev() {
        if entry.path.split(['/', '\\']).any(|p| p == "..") || Path::new(&entry.path).is_absolute() {
            report.failed.push((entry.path.clone(), "路径不在工作目录内".into()));
            continue;
        }
        let file = workspace.join(&entry.path);
        if entry.created {
            match fs::remove_file(&file) {
                Ok(()) => report.removed.push(entry.path.clone()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => report.removed.push(entry.path.clone()),
                Err(e) => report.failed.push((entry.path.clone(), e.to_string())),
            }
        } else if let Some(backup) = &entry.backup {
            let restored = file.parent().map_or(Ok(()), fs::create_dir_all).and_then(|()| fs::copy(dir.join(backup), &file).map(|_| ()));
            match restored {
                Ok(()) => report.restored.push(entry.path.clone()),
                Err(e) => report.failed.push((entry.path.clone(), e.to_string())),
            }
        } else {
            report.failed.push((entry.path.clone(), "文件太大，当时没有备份".into()));
        }
    }
    manifest.undone = true;
    save(&dir, &manifest)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiaoban-checkpoints-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("work")).unwrap();
        dir
    }

    #[test]
    fn undo_puts_changed_files_back_and_removes_created_ones() {
        let dir = temp("undo");
        let work = dir.join("work");
        fs::write(work.join("a.txt"), "原来的").unwrap();
        let checkpoints = Checkpoints::new(dir.join("cp"), &work);
        checkpoints.record("s1", "r1", &json!({"path": "a.txt"})).unwrap();
        fs::write(work.join("a.txt"), "改过的").unwrap();
        checkpoints.record("s1", "r1", &json!({"path": "a.txt"})).unwrap();
        fs::write(work.join("a.txt"), "又改了").unwrap();
        checkpoints.record("s1", "r1", &json!({"path": work.join("new/b.txt").display().to_string()})).unwrap();
        fs::create_dir_all(work.join("new")).unwrap();
        fs::write(work.join("new/b.txt"), "新文件").unwrap();
        checkpoints.record("s1", "r1", &json!({"path": "../outside.txt"})).unwrap();

        let runs = list(&dir.join("cp"), "s1");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].entries.iter().map(|e| (e.path.as_str(), e.created)).collect::<Vec<_>>(), vec![("a.txt", false), ("new/b.txt", true)],
            "each file once, its state before the first change; nothing outside the working folder");

        let report = undo(&dir.join("cp"), "s1", "r1").unwrap();
        assert_eq!(report.restored, vec!["a.txt"]);
        assert_eq!(report.removed, vec!["new/b.txt"]);
        assert_eq!(fs::read_to_string(work.join("a.txt")).unwrap(), "原来的");
        assert!(!work.join("new/b.txt").exists());
        assert!(undo(&dir.join("cp"), "s1", "r1").is_err(), "an answer is undone once");
        assert!(undo(&dir.join("cp"), "s1", "../x").is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_newest_answers_are_kept() {
        let dir = temp("prune");
        let work = dir.join("work");
        let root = dir.join("cp");
        for run in 0..(KEEP_RUNS + 3) {
            let manifest = Manifest { run_id: format!("r{run}"), workspace: work.display().to_string(), created_at: run as i64, entries: vec![], undone: false };
            fs::create_dir_all(root.join("s1").join(format!("r{run}"))).unwrap();
            save(&root.join("s1").join(format!("r{run}")), &manifest).unwrap();
        }
        let checkpoints = Checkpoints::new(&root, &work);
        checkpoints.record("s1", "fresh", &json!({"path": "x.txt"})).unwrap();
        let left = fs::read_dir(root.join("s1")).unwrap().count();
        assert_eq!(left, KEEP_RUNS);
        assert!(root.join("s1/fresh").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}

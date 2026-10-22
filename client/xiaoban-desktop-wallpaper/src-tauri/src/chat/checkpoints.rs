//! "撤销这轮的文件改动": before an answer first changes a file in the working folder with `write_file` or
//! `edit_file`, the file as it was is copied aside, so the whole answer's file changes can be put back later.
//!
//! When the working folder is in a git repository, the first shell command or background process of an answer also
//! takes a snapshot of the working files (see `git_snapshot`), so what commands changed is put back too. Ignored
//! files (`node_modules`, build output) are not in it; outside a repository commands are not covered.
//!
//! Layout: `chat/checkpoints/<session>/<run>/manifest.json` plus one numbered copy per file that existed before.

use super::git_snapshot;
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
    /// The working files before the answer's first command, when the folder is in a git repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitSnapshot {
    pub root: String,
    /// The working folder inside the repository, `.` at its root.
    pub pathspec: String,
    pub tree: String,
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

    /// Runs `change` on this answer's manifest and saves it when `change` says it changed something.
    fn with_manifest(&self, session: &str, run_id: &str, change: impl FnOnce(&Path, &mut Manifest) -> Result<bool, String>) -> Result<(), String> {
        if !safe(session) || !safe(run_id) {
            return Ok(());
        }
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
        fs::create_dir_all(&*dir).map_err(|e| e.to_string())?;
        if change(dir, manifest)? {
            save(dir, manifest)?;
        }
        Ok(())
    }

    /// The first command of an answer in a git working folder: remember the working files as they are.
    fn record_git(&self, session: &str, run_id: &str) -> Result<(), String> {
        let workspace = self.workspace.clone();
        self.with_manifest(session, run_id, |_, manifest| {
            if manifest.git.is_some() {
                return Ok(false);
            }
            let Some((root, pathspec)) = git_snapshot::locate(&workspace) else { return Ok(false) };
            let tree = git_snapshot::snapshot(&root, &pathspec)?;
            manifest.git = Some(GitSnapshot { root: root.display().to_string(), pathspec, tree });
            Ok(true)
        })
    }

    fn record(&self, session: &str, run_id: &str, args: &Value) -> Result<(), String> {
        let Some((file, relative)) = self.target(args) else { return Ok(()) };
        self.with_manifest(session, run_id, |dir, manifest| Self::add_entry(dir, manifest, &file, relative))
    }

    fn add_entry(dir: &Path, manifest: &mut Manifest, file: &Path, relative: String) -> Result<bool, String> {
        if manifest.entries.iter().any(|e| e.path == relative) {
            return Ok(false);
        }
        let entry = match fs::symlink_metadata(file) {
            Ok(meta) if meta.is_file() && meta.len() <= MAX_BACKUP_BYTES => {
                let name = format!("{}.bak", manifest.entries.len());
                fs::copy(file, dir.join(&name)).map_err(|e| e.to_string())?;
                Entry { path: relative, created: false, backup: Some(name), skipped: false }
            }
            Ok(meta) if meta.is_file() => Entry { path: relative, created: false, backup: None, skipped: true },
            // A folder or a link: the file tools refuse those, nothing to keep.
            Ok(_) => return Ok(false),
            Err(_) => Entry { path: relative, created: true, backup: None, skipped: false },
        };
        manifest.entries.push(entry);
        Ok(true)
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
        // A copy or snapshot that cannot be made must not stop the work; undo then simply has less to put back.
        match spec.name.as_str() {
            "write_file" | "edit_file" => {
                let _ = self.record(ctx.session(), ctx.run_id(), args);
            }
            "shell" | "process_start" => {
                let _ = self.record_git(ctx.session(), ctx.run_id());
            }
            _ => {}
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
    let mut runs: Vec<Manifest> = read.flatten().filter_map(|e| load(&e.path())).filter(|m| !m.entries.is_empty() || m.git.is_some()).collect();
    runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    runs
}

/// What commands changed since the snapshot: added, and changed or deleted, relative to the working folder.
fn git_changes(snapshot: &GitSnapshot) -> Result<(Vec<String>, Vec<String>), String> {
    let root = PathBuf::from(&snapshot.root);
    let now = git_snapshot::snapshot(&root, &snapshot.pathspec)?;
    let (added, changed) = git_snapshot::changes(&root, &snapshot.tree, &now)?;
    Ok((added, changed))
}

fn relative_to_workspace(snapshot: &GitSnapshot, path: &str) -> String {
    match snapshot.pathspec.as_str() {
        "." => path.to_string(),
        prefix => path.strip_prefix(&format!("{prefix}/")).unwrap_or(path).to_string(),
    }
}

/// Every file an answer changed, for the notice after it: the file tools' and, in a git folder, the commands'.
pub fn changed_files(manifest: &Manifest) -> Vec<String> {
    let mut files: Vec<String> = manifest.entries.iter().map(|e| e.path.clone()).collect();
    if let Some(snapshot) = &manifest.git {
        if let Ok((added, changed)) = git_changes(snapshot) {
            for path in added.iter().chain(&changed).map(|p| relative_to_workspace(snapshot, p)) {
                if !files.contains(&path) {
                    files.push(path);
                }
            }
        }
    }
    files
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
    // Commands first: the file tools' copies are older (taken before each first edit) and are written last.
    if let Some(snapshot) = &manifest.git {
        let root = PathBuf::from(&snapshot.root);
        match git_changes(snapshot) {
            Ok((added, changed)) => {
                for path in &added {
                    match fs::remove_file(root.join(path)) {
                        Ok(()) => report.removed.push(relative_to_workspace(snapshot, path)),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => report.failed.push((relative_to_workspace(snapshot, path), e.to_string())),
                    }
                }
                match git_snapshot::restore(&root, &snapshot.tree, &changed) {
                    Ok(()) => report.restored.extend(changed.iter().map(|p| relative_to_workspace(snapshot, p))),
                    Err(e) => report.failed.extend(changed.iter().map(|p| (relative_to_workspace(snapshot, p), e.clone()))),
                }
            }
            Err(e) => report.failed.push(("（命令改动的文件）".into(), format!("无法读取 git 快照：{e}"))),
        }
    }
    for entry in manifest.entries.iter().rev() {
        if entry.path.split(['/', '\\']).any(|p| p == "..") || Path::new(&entry.path).is_absolute() {
            report.failed.push((entry.path.clone(), "路径不在工作目录内".into()));
            continue;
        }
        let file = workspace.join(&entry.path);
        if entry.created {
            match fs::remove_file(&file) {
                Ok(()) => push_once(&mut report.removed, &entry.path),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => push_once(&mut report.removed, &entry.path),
                Err(e) => report.failed.push((entry.path.clone(), e.to_string())),
            }
        } else if let Some(backup) = &entry.backup {
            let restored = file.parent().map_or(Ok(()), fs::create_dir_all).and_then(|()| fs::copy(dir.join(backup), &file).map(|_| ()));
            match restored {
                Ok(()) => push_once(&mut report.restored, &entry.path),
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

fn push_once(list: &mut Vec<String>, path: &str) {
    if !list.iter().any(|p| p == path) {
        list.push(path.to_string());
    }
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
    fn in_a_git_folder_undo_also_puts_back_what_commands_changed() {
        let dir = temp("git");
        let work = dir.join("work");
        let git = |args: &[&str]| assert!(std::process::Command::new("git").arg("-C").arg(&work).args(args).output().unwrap().status.success(), "{args:?}");
        git(&["init", "-q"]);
        fs::write(work.join("a.txt"), "原始").unwrap();
        fs::write(work.join("b.txt"), "原始 b").unwrap();
        let checkpoints = Checkpoints::new(dir.join("cp"), &work);
        // The answer edits a.txt with a file tool, then runs commands that change b.txt and create c.txt.
        checkpoints.record("s1", "r1", &json!({"path": "a.txt"})).unwrap();
        fs::write(work.join("a.txt"), "小伴改过").unwrap();
        checkpoints.record_git("s1", "r1").unwrap();
        fs::write(work.join("a.txt"), "命令又改了").unwrap();
        fs::write(work.join("b.txt"), "命令改了 b").unwrap();
        fs::write(work.join("c.txt"), "命令生成").unwrap();

        let manifest = list(&dir.join("cp"), "s1").remove(0);
        let mut files = changed_files(&manifest);
        files.sort();
        assert_eq!(files, vec!["a.txt", "b.txt", "c.txt"]);

        let report = undo(&dir.join("cp"), "s1", "r1").unwrap();
        assert!(report.failed.is_empty(), "{:?}", report.failed);
        assert_eq!(fs::read_to_string(work.join("a.txt")).unwrap(), "原始", "the file tool's older copy wins");
        assert_eq!(fs::read_to_string(work.join("b.txt")).unwrap(), "原始 b");
        assert!(!work.join("c.txt").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_newest_answers_are_kept() {
        let dir = temp("prune");
        let work = dir.join("work");
        let root = dir.join("cp");
        for run in 0..(KEEP_RUNS + 3) {
            let manifest = Manifest { run_id: format!("r{run}"), workspace: work.display().to_string(), created_at: run as i64, entries: vec![], undone: false, git: None };
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

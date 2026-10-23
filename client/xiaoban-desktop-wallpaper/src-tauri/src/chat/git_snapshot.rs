//! Snapshots of a git working folder, so undo can also put back what shell commands changed.
//!
//! A snapshot is a tree object made from the working files through a throwaway index: the user's index is copied
//! (so unchanged files are not hashed again), `git add -A` and `git write-tree` run on the copy, and the copy is
//! deleted. The user's index, branches and stash are never touched; the tree is an unreferenced object that git
//! collects on its own schedule. Untracked files are included; ignored ones (`node_modules`, build output) are not.
//!
//! A working folder outside any repository gets a private one instead (`private`): its git folder lives in the
//! app's data (`chat/snapshots/<id>.git`) with the working folder as its work tree, so nothing is added to the
//! user's folder. Folders that are too big to snapshot quickly, the home folder and a disk's root are left out.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_INDEX: AtomicU64 = AtomicU64::new(0);

/// A large repository's first `add -A` may take a while; a snapshot that takes longer is skipped, not waited for.
const TIMEOUT: Duration = Duration::from_secs(20);
/// A folder outside a repository is snapshotted only up to this size (files and bytes, ignored folders not counted).
const PRIVATE_MAX_FILES: usize = 20_000;
const PRIVATE_MAX_BYTES: u64 = 512 * 1024 * 1024;
/// Not kept in a private snapshot: dependencies, build output and environments, which commands recreate.
const PRIVATE_EXCLUDES: &[&str] = &["node_modules", "target", "dist", "build", ".venv", "venv", "__pycache__", ".gradle", ".next", ".cache", "Pods", "DerivedData"];

/// A repository to snapshot: the user's own, or a private one whose git folder is kept elsewhere.
#[derive(Debug, Clone, PartialEq)]
pub struct Repo {
    pub root: PathBuf,
    pub git_dir: Option<PathBuf>,
}

impl Repo {
    pub fn user(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), git_dir: None }
    }
}

fn git(repo: &Repo, args: &[&str], index: Option<&Path>) -> Result<Output, String> {
    let mut cmd = Command::new("git");
    match &repo.git_dir {
        Some(git_dir) => cmd.arg("--git-dir").arg(git_dir).arg("--work-tree").arg(&repo.root),
        None => cmd.arg("-C").arg(&repo.root),
    };
    cmd.args(["-c", "core.quotepath=off"]).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(index) = index {
        cmd.env("GIT_INDEX_FILE", index);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd.spawn().map_err(|e| format!("git is not available: {e}"))?;
    let end = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < end => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("git took too long".into());
            }
        }
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn text(output: Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// The repository root and the working folder's path inside it (`.` at the root), or `None` outside a repository.
pub fn locate(workspace: &Path) -> Option<(PathBuf, String)> {
    let here = Repo::user(workspace);
    let root = PathBuf::from(text(git(&here, &["rev-parse", "--show-toplevel"], None).ok()?));
    let prefix = text(git(&here, &["rev-parse", "--show-prefix"], None).ok()?);
    let pathspec = if prefix.is_empty() { ".".to_string() } else { prefix.trim_end_matches('/').to_string() };
    Some((root, pathspec))
}

/// Stable across builds (unlike the standard hasher), so a folder keeps its private repository.
fn fnv1a(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3))
}

/// Whether a folder outside a repository is small and ordinary enough to snapshot.
fn worth_snapshotting(workspace: &Path) -> bool {
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    let same = |other: &Path| other.canonicalize().is_ok_and(|other| workspace.canonicalize().is_ok_and(|w| w == other));
    if workspace.parent().is_none() || home.as_deref().is_some_and(same) {
        return false;
    }
    let (mut files, mut bytes, mut pending) = (0usize, 0u64, vec![workspace.to_path_buf()]);
    while let Some(dir) = pending.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else { continue };
        for entry in read.flatten() {
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_dir() {
                let name = entry.file_name();
                if name != ".git" && !PRIVATE_EXCLUDES.iter().any(|x| name == *x) {
                    pending.push(entry.path());
                }
            } else if kind.is_file() {
                files += 1;
                bytes += entry.metadata().map_or(0, |m| m.len());
                if files > PRIVATE_MAX_FILES || bytes > PRIVATE_MAX_BYTES {
                    return false;
                }
            }
        }
    }
    true
}

/// The private repository for a working folder outside any repository (see the module comment), made the first
/// time; `None` when the folder is not worth snapshotting. `store` is where the git folders are kept.
pub fn private(workspace: &Path, store: &Path) -> Result<Option<Repo>, String> {
    if !worth_snapshotting(workspace) {
        return Ok(None);
    }
    let git_dir = store.join(format!("{:016x}.git", fnv1a(&workspace.display().to_string())));
    let repo = Repo { root: workspace.to_path_buf(), git_dir: Some(git_dir.clone()) };
    if !git_dir.join("HEAD").exists() {
        std::fs::create_dir_all(store).map_err(|e| e.to_string())?;
        let bare = Repo::user(store);
        git(&bare, &["init", "-q", "--bare", &git_dir.display().to_string()], None)?;
        git(&repo, &["config", "core.bare", "false"], None)?;
        let excludes: String = PRIVATE_EXCLUDES.iter().map(|x| format!("{x}/\n")).collect();
        std::fs::create_dir_all(git_dir.join("info")).map_err(|e| e.to_string())?;
        std::fs::write(git_dir.join("info/exclude"), excludes).map_err(|e| e.to_string())?;
        let _ = std::fs::write(git_dir.join("xiaoban-workspace"), workspace.display().to_string());
    }
    Ok(Some(repo))
}

/// A tree of the working folder's current files (see the module comment).
pub fn snapshot(repo: &Repo, pathspec: &str) -> Result<String, String> {
    let index = std::env::temp_dir().join(format!("xiaoban-snapshot-{}-{}-{}.index", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos(),
        NEXT_INDEX.fetch_add(1, Ordering::Relaxed)));
    let user_index = PathBuf::from(text(git(repo, &["rev-parse", "--path-format=absolute", "--git-path", "index"], None)?));
    // Starting from the user's index only saves time; without one the throwaway index starts empty.
    let _ = std::fs::copy(&user_index, &index);
    let result = git(repo, &["add", "-A", "--", pathspec], Some(&index)).and_then(|_| git(repo, &["write-tree"], Some(&index))).map(text);
    let _ = std::fs::remove_file(&index);
    if repo.git_dir.is_some() {
        // Old snapshots nothing refers to are removed after git's usual two weeks.
        let _ = git(repo, &["gc", "--auto", "--quiet"], None);
    }
    result
}

/// Paths (relative to the repository root) that differ between two snapshots: added since, and changed or deleted.
pub fn changes(repo: &Repo, from: &str, to: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let output = git(repo, &["diff", "-z", "--no-renames", "--name-status", from, to], None)?;
    let fields: Vec<String> = output.stdout.split(|b| *b == 0).filter(|f| !f.is_empty()).map(|f| String::from_utf8_lossy(f).into_owned()).collect();
    let (mut added, mut other) = (Vec::new(), Vec::new());
    for pair in fields.chunks(2) {
        if let [status, path] = pair {
            if status.starts_with('A') { added.push(path.clone()) } else { other.push(path.clone()) }
        }
    }
    Ok((added, other))
}

/// Writes these paths back to how they were in `tree`, in the working files only (the index stays as it is).
pub fn restore(repo: &Repo, tree: &str, paths: &[String]) -> Result<(), String> {
    for chunk in paths.chunks(100) {
        let mut args = vec!["restore", "--worktree"];
        let source = format!("--source={tree}");
        args.push(&source);
        args.push("--");
        args.extend(chunk.iter().map(String::as_str));
        git(repo, &args, None)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiaoban-git-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("app")).unwrap();
        let run = |args: &[&str]| assert!(Command::new("git").arg("-C").arg(&dir).args(args).output().unwrap().status.success(), "{args:?}");
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@example.com"]);
        run(&["config", "user.name", "t"]);
        fs::write(dir.join("app/a.txt"), "原始").unwrap();
        fs::write(dir.join("app/b.txt"), "保留").unwrap();
        fs::write(dir.join("app/notes.md"), "未提交的笔记").unwrap();
        fs::write(dir.join("outside.txt"), "仓库根下的文件").unwrap();
        run(&["add", "app/a.txt", "app/b.txt", "outside.txt"]);
        run(&["commit", "-qm", "init"]);
        dir
    }

    #[test]
    fn a_snapshot_covers_untracked_files_and_puts_back_only_the_working_folder() {
        let dir = repo("snap");
        let (root, pathspec) = locate(&dir.join("app")).unwrap();
        assert_eq!(pathspec, "app");
        let repo = Repo::user(&root);
        let staged_before = fs::read(root.join(".git/index")).unwrap();
        let before = snapshot(&repo, &pathspec).unwrap();
        assert_eq!(fs::read(root.join(".git/index")).unwrap(), staged_before, "the user's index is untouched");

        fs::write(dir.join("app/a.txt"), "被命令改了").unwrap();
        fs::remove_file(dir.join("app/notes.md")).unwrap();
        fs::write(dir.join("app/new.txt"), "命令生成的").unwrap();
        fs::write(dir.join("outside.txt"), "工作目录外的改动").unwrap();
        let after = snapshot(&repo, &pathspec).unwrap();
        let (added, changed) = changes(&repo, &before, &after).unwrap();
        assert_eq!(added, vec!["app/new.txt"]);
        assert_eq!(changed, vec!["app/a.txt", "app/notes.md"], "an untracked file that was deleted is seen too; outside.txt is not");

        restore(&repo, &before, &changed).unwrap();
        assert_eq!(fs::read_to_string(dir.join("app/a.txt")).unwrap(), "原始");
        assert_eq!(fs::read_to_string(dir.join("app/notes.md")).unwrap(), "未提交的笔记");
        assert_eq!(fs::read_to_string(dir.join("outside.txt")).unwrap(), "工作目录外的改动");
        assert_eq!(fs::read(root.join(".git/index")).unwrap(), staged_before, "restoring leaves the index alone too");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_folder_outside_a_repository_gets_a_private_one_kept_elsewhere() {
        let dir = std::env::temp_dir().join(format!("xiaoban-private-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let (work, store) = (dir.join("我的 文件夹"), dir.join("snapshots"));
        fs::create_dir_all(work.join("node_modules/x")).unwrap();
        fs::write(work.join("报告.md"), "第一版").unwrap();
        fs::write(work.join("node_modules/x/index.js"), "dep").unwrap();
        assert!(locate(&work).is_none());
        let repo = private(&work, &store).unwrap().unwrap();
        assert_eq!(private(&work, &store).unwrap().unwrap(), repo, "the same folder keeps its repository");
        let before = snapshot(&repo, ".").unwrap();
        assert!(!work.join(".git").exists(), "nothing is added to the user's folder");

        fs::write(work.join("报告.md"), "被命令改了").unwrap();
        fs::write(work.join("新文件.txt"), "x").unwrap();
        fs::write(work.join("node_modules/x/index.js"), "changed dep").unwrap();
        let after = snapshot(&repo, ".").unwrap();
        let (added, changed) = changes(&repo, &before, &after).unwrap();
        assert_eq!((added, changed.clone()), (vec!["新文件.txt".to_string()], vec!["报告.md".to_string()]), "dependencies are left out");
        restore(&repo, &before, &changed).unwrap();
        assert_eq!(fs::read_to_string(work.join("报告.md")).unwrap(), "第一版");

        let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from).unwrap();
        assert!(private(&home, &store).unwrap().is_none(), "never the home folder");
        let _ = fs::remove_dir_all(&dir);
    }
}

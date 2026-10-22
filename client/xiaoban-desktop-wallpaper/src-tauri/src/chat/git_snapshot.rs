//! Snapshots of a git working folder, so undo can also put back what shell commands changed.
//!
//! A snapshot is a tree object made from the working files through a throwaway index: the user's index is copied
//! (so unchanged files are not hashed again), `git add -A` and `git write-tree` run on the copy, and the copy is
//! deleted. The user's index, branches and stash are never touched; the tree is an unreferenced object that git
//! collects on its own schedule. Untracked files are included; ignored ones (`node_modules`, build output) are not.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// A large repository's first `add -A` may take a while; a snapshot that takes longer is skipped, not waited for.
const TIMEOUT: Duration = Duration::from_secs(20);

fn git(dir: &Path, args: &[&str], index: Option<&Path>) -> Result<Output, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir).args(["-c", "core.quotepath=off"]).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
    let root = PathBuf::from(text(git(workspace, &["rev-parse", "--show-toplevel"], None).ok()?));
    let prefix = text(git(workspace, &["rev-parse", "--show-prefix"], None).ok()?);
    let pathspec = if prefix.is_empty() { ".".to_string() } else { prefix.trim_end_matches('/').to_string() };
    Some((root, pathspec))
}

/// A tree of the working folder's current files (see the module comment).
pub fn snapshot(root: &Path, pathspec: &str) -> Result<String, String> {
    let index = std::env::temp_dir().join(format!("xiaoban-snapshot-{}-{}.index", std::process::id(), super::now().1));
    let user_index = PathBuf::from(text(git(root, &["rev-parse", "--path-format=absolute", "--git-path", "index"], None)?));
    // Starting from the user's index only saves time; without one the throwaway index starts empty.
    let _ = std::fs::copy(&user_index, &index);
    let result = git(root, &["add", "-A", "--", pathspec], Some(&index)).and_then(|_| git(root, &["write-tree"], Some(&index))).map(text);
    let _ = std::fs::remove_file(&index);
    result
}

/// Paths (relative to the repository root) that differ between two snapshots: added since, and changed or deleted.
pub fn changes(root: &Path, from: &str, to: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let output = git(root, &["diff", "-z", "--no-renames", "--name-status", from, to], None)?;
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
pub fn restore(root: &Path, tree: &str, paths: &[String]) -> Result<(), String> {
    for chunk in paths.chunks(100) {
        let mut args = vec!["restore", "--worktree"];
        let source = format!("--source={tree}");
        args.push(&source);
        args.push("--");
        args.extend(chunk.iter().map(String::as_str));
        git(root, &args, None)?;
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
        let staged_before = fs::read(root.join(".git/index")).unwrap();
        let before = snapshot(&root, &pathspec).unwrap();
        assert_eq!(fs::read(root.join(".git/index")).unwrap(), staged_before, "the user's index is untouched");

        fs::write(dir.join("app/a.txt"), "被命令改了").unwrap();
        fs::remove_file(dir.join("app/notes.md")).unwrap();
        fs::write(dir.join("app/new.txt"), "命令生成的").unwrap();
        fs::write(dir.join("outside.txt"), "工作目录外的改动").unwrap();
        let after = snapshot(&root, &pathspec).unwrap();
        let (added, changed) = changes(&root, &before, &after).unwrap();
        assert_eq!(added, vec!["app/new.txt"]);
        assert_eq!(changed, vec!["app/a.txt", "app/notes.md"], "an untracked file that was deleted is seen too; outside.txt is not");

        restore(&root, &before, &changed).unwrap();
        assert_eq!(fs::read_to_string(dir.join("app/a.txt")).unwrap(), "原始");
        assert_eq!(fs::read_to_string(dir.join("app/notes.md")).unwrap(), "未提交的笔记");
        assert_eq!(fs::read_to_string(dir.join("outside.txt")).unwrap(), "工作目录外的改动");
        assert_eq!(fs::read(root.join(".git/index")).unwrap(), staged_before, "restoring leaves the index alone too");
        let _ = fs::remove_dir_all(&dir);
    }
}

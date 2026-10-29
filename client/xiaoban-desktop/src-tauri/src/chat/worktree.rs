//! A conversation can work in an isolated copy of its git folder (a git worktree), so what it changes does not
//! touch the user's own files until the user says so. Several conversations can then edit the same project at
//! once without overwriting each other, which is what Claude Code and Hermes recommend for parallel sessions.
//!
//! The copy lives in the app's data folder (`worktrees/<folder>-<id>`), never inside the user's folder, on its own
//! branch `xiaoban/<id>`. While it is active the conversation's folder *is* the copy, so every tool, permission and
//! undo snapshot works unchanged. Afterwards the user chooses: merge the branch back into the folder they started
//! in, keep the copy as it is, or throw it away. Nothing in the original folder is touched before a merge, and a
//! merge is refused (not forced) when the original has uncommitted changes or the branches conflict.

use super::sessions::WorktreeLink;
use super::*;
use serde::Serialize;
use std::process::{Command, Stdio};

const SHORT_TIMEOUT: Duration = Duration::from_secs(20);
const LONG_TIMEOUT: Duration = Duration::from_secs(180);
/// How many changed files the page lists; the count is always exact.
const LISTED_CHANGES: usize = 30;
const NO_HOOKS: &str = if cfg!(windows) { "core.hooksPath=NUL" } else { "core.hooksPath=/dev/null" };

/// Runs git in `dir` and returns its trimmed stdout; the error is git's own message.
fn git(dir: &Path, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir).args(["-c", "core.quotepath=off"]).args(args)
        .env("GIT_TERMINAL_PROMPT", "0").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let output = super::git_command::output(&mut cmd, timeout)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if message.is_empty() { String::from_utf8_lossy(&output.stdout).trim().to_string() } else { message })
    }
}

/// The root of the repository `dir` is in.
pub fn repo_root(dir: &Path) -> Result<PathBuf, String> {
    git(dir, &["rev-parse", "--show-toplevel"], SHORT_TIMEOUT)
        .map(PathBuf::from)
        .map_err(|_| "这个文件夹不在 git 仓库里，没法创建独立副本（需要先 git init 并提交一次）".to_string())
}

/// Stable short name for a conversation, used in folder and branch names.
fn short(session_id: &str) -> String {
    let hash = session_id.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    format!("{:08x}", hash as u32)
}

fn folder_name(root: &Path) -> String {
    let name: String = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
        .chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).take(24).collect();
    if name.is_empty() { "folder".into() } else { name }
}

/// Where the copies are kept; also the only place this module is willing to delete from.
pub fn store(data_root: &Path) -> PathBuf {
    data_root.join("worktrees")
}

/// A link is only trusted when its copy is under the app's own folder, so an edited index cannot point a merge
/// or a delete somewhere else.
fn owned(store: &Path, link: &WorktreeLink) -> Result<PathBuf, String> {
    let path = PathBuf::from(&link.path);
    if path.starts_with(store) && path != store { Ok(path) } else { Err("这个独立副本不在小伴的数据目录里，已拒绝操作".into()) }
}

#[derive(Debug)]
pub struct Created {
    pub link: WorktreeLink,
    /// The folder to work in: the same sub-folder of the copy as the user was in.
    pub workspace: PathBuf,
}

/// Makes the copy for the folder `origin` (any folder inside a repository that has at least one commit).
pub fn create(origin: &Path, store: &Path, session_id: &str) -> Result<Created, String> {
    let root = repo_root(origin)?;
    let base = git(&root, &["rev-parse", "--verify", "HEAD"], SHORT_TIMEOUT)
        .map_err(|_| "这个仓库还没有任何提交，没法创建独立副本。先提交一次再试".to_string())?;
    let relative = match (origin.canonicalize(), root.canonicalize()) {
        (Ok(o), Ok(r)) => o.strip_prefix(&r).map(Path::to_path_buf).unwrap_or_default(),
        _ => PathBuf::new(),
    };
    let id = short(session_id);
    std::fs::create_dir_all(store).map_err(|e| format!("无法创建副本目录：{e}"))?;
    let name = format!("{}-{id}", folder_name(&root));
    let mut path = store.join(&name);
    let mut n = 1;
    while path.exists() {
        n += 1;
        if n > 9 { return Err("这个对话的独立副本已经存在，请先处理掉旧的".into()); }
        path = store.join(format!("{name}-{n}"));
    }
    let mut branch = format!("xiaoban/{id}");
    let mut n = 1;
    while git(&root, &["rev-parse", "--verify", "--quiet", &format!("refs/heads/{branch}")], SHORT_TIMEOUT).is_ok() {
        n += 1;
        branch = format!("xiaoban/{id}-{n}");
    }
    // Hooks stay off: a post-checkout hook would otherwise run just because the app made a copy.
    git(&root, &["-c", NO_HOOKS, "worktree", "add", "-b", &branch, &path.display().to_string(), &base], LONG_TIMEOUT)?;
    Ok(Created {
        workspace: path.join(&relative),
        link: WorktreeLink { origin: origin.display().to_string(), path: path.display().to_string(), branch, base },
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub exists: bool,
    pub branch: String,
    /// Commits the copy has made since it started.
    pub ahead: u32,
    /// Files with changes that are not committed yet (at most `LISTED_CHANGES` are listed).
    pub changed: Vec<String>,
    pub changed_count: usize,
    /// The branch checked out in the original folder, and whether it can take a merge right now.
    pub origin_branch: String,
    pub origin_clean: bool,
}

pub fn status(store: &Path, link: &WorktreeLink) -> Result<Status, String> {
    let path = owned(store, link)?;
    let mut status = Status { exists: path.is_dir(), branch: link.branch.clone(), ahead: 0, changed: Vec::new(), changed_count: 0, origin_branch: String::new(), origin_clean: false };
    if status.exists {
        status.ahead = git(&path, &["rev-list", "--count", &format!("{}..HEAD", link.base)], SHORT_TIMEOUT).ok().and_then(|n| n.parse().ok()).unwrap_or(0);
        let lines = git(&path, &["status", "--porcelain"], SHORT_TIMEOUT)?;
        let files: Vec<&str> = lines.lines().filter(|l| !l.trim().is_empty()).collect();
        status.changed_count = files.len();
        status.changed = files.iter().take(LISTED_CHANGES).map(|l| l.to_string()).collect();
    }
    if let Ok(root) = repo_root(Path::new(&link.origin)) {
        status.origin_branch = git(&root, &["symbolic-ref", "--short", "-q", "HEAD"], SHORT_TIMEOUT).unwrap_or_default();
        status.origin_clean = git(&root, &["status", "--porcelain", "--untracked-files=no"], SHORT_TIMEOUT).is_ok_and(|s| s.trim().is_empty());
    }
    Ok(status)
}

/// Commits what is left uncommitted in the copy, so the merge brings all of it.
fn commit_pending(path: &Path, title: &str) -> Result<(), String> {
    if git(path, &["status", "--porcelain"], SHORT_TIMEOUT)?.trim().is_empty() {
        return Ok(());
    }
    git(path, &["add", "-A"], LONG_TIMEOUT)?;
    let has_identity = git(path, &["config", "user.email"], SHORT_TIMEOUT).is_ok_and(|e| !e.trim().is_empty());
    let message = format!("小伴：{title}");
    let mut args = vec!["-c", NO_HOOKS];
    if !has_identity {
        args.extend(["-c", "user.name=小伴", "-c", "user.email=xiaoban@localhost"]);
    }
    args.extend(["commit", "-m", &message]);
    git(path, &args, LONG_TIMEOUT).map(|_| ())
}

/// Merges the copy's branch into the original folder and removes the copy. Refuses (and changes nothing in the
/// original) when the original has uncommitted changes, is not on a branch, or the merge conflicts.
pub fn merge(store: &Path, link: &WorktreeLink, title: &str) -> Result<String, String> {
    let path = owned(store, link)?;
    if !path.is_dir() {
        return Err("独立副本的文件夹已经不存在了".into());
    }
    let root = repo_root(Path::new(&link.origin))?;
    let origin_branch = git(&root, &["symbolic-ref", "--short", "-q", "HEAD"], SHORT_TIMEOUT)
        .map_err(|_| "原文件夹现在不在任何分支上（游离状态），没法合并。先切回一个分支".to_string())?;
    if !git(&root, &["status", "--porcelain", "--untracked-files=no"], SHORT_TIMEOUT)?.trim().is_empty() {
        return Err("原文件夹里还有没提交的改动，先提交或暂存它们再合并，免得和副本的改动混在一起".into());
    }
    commit_pending(&path, title)?;
    let ahead: u32 = git(&path, &["rev-list", "--count", &format!("{}..HEAD", link.base)], SHORT_TIMEOUT)?.parse().unwrap_or(0);
    if ahead == 0 {
        discard(store, link)?;
        return Ok("副本里没有改动，已清理掉副本".into());
    }
    let message = format!("合并小伴的改动：{title}");
    let before = git(&root, &["rev-parse", "HEAD"], SHORT_TIMEOUT)?;
    if let Err(error) = git(&root, &["merge", "--no-ff", "-m", &message, &link.branch], LONG_TIMEOUT) {
        // A timeout/output error may happen after git committed. Check reality before aborting or reporting failure.
        let after = git(&root, &["rev-parse", "HEAD"], SHORT_TIMEOUT);
        let committed = after.as_ref().is_ok_and(|head| head != &before)
            && git(&root, &["merge-base", "--is-ancestor", &link.branch, "HEAD"], SHORT_TIMEOUT).is_ok();
        if !committed {
            let merging = git(&root, &["rev-parse", "--verify", "MERGE_HEAD"], SHORT_TIMEOUT).is_ok();
            let aborted = merging && git(&root, &["merge", "--abort"], SHORT_TIMEOUT).is_ok();
            let detail = if aborted { "合并未完成（可能有冲突），已中止本次合并" }
                else { "合并未能确认完成，请检查原文件夹的文件和 Git 状态" };
            return Err(format!("{detail}。副本和分支“{}”都还在：{error}", link.branch));
        }
    }
    discard(store, link)?;
    Ok(format!("已把 {ahead} 个提交合并进“{origin_branch}”，并清理了副本"))
}

/// Removes the copy and its branch.
pub fn discard(store: &Path, link: &WorktreeLink) -> Result<(), String> {
    let path = owned(store, link)?;
    let root = repo_root(Path::new(&link.origin)).ok();
    let anchor = root.as_deref().unwrap_or(&path);
    if path.exists() {
        let removed = git(anchor, &["worktree", "remove", "--force", &path.display().to_string()], LONG_TIMEOUT);
        if removed.is_err() && path.exists() {
            std::fs::remove_dir_all(&path).map_err(|e| format!("无法删除副本文件夹：{e}"))?;
        }
    }
    if let Some(root) = root {
        let _ = git(&root, &["worktree", "prune"], SHORT_TIMEOUT);
        let _ = git(&root, &["branch", "-D", &link.branch], SHORT_TIMEOUT);
    }
    Ok(())
}

fn started_note(link: &WorktreeLink, workspace: &Path) -> String {
    format!(
        "{}从现在起这个对话在原文件夹“{}”的独立副本“{}”里工作（git 分支 {}）。你的改动不会影响原文件夹，直到用户选择合并。前面提到的文件路径指的是原文件夹，需要时先重新查看副本里的内容。",
        workspace::SWITCH_MARK, link.origin, workspace.display(), link.branch
    )
}

fn finished_note(link: &WorktreeLink, action: &str) -> String {
    let what = match action {
        "merge" => "副本的改动已合并回原文件夹，副本已删除",
        "discard" => "副本和它的改动已丢弃",
        _ => "副本保留在原处不再使用，改动没有合并",
    };
    format!("{}从现在起这个对话回到原文件夹“{}”工作。{what}。之前提到的副本里的路径已失效，需要时先重新查看。", workspace::SWITCH_MARK, link.origin)
}

fn guard(app: &tauri::AppHandle, window: &tauri::WebviewWindow, session_id: &str) -> Result<SessionSummary, String> {
    if window.label() != "control" {
        return Err("请在聊天页使用独立副本".into());
    }
    checked_session_id(session_id)?;
    let summary = mutate(app, |index| index.get(session_id).cloned())?.ok_or("这个对话已经不存在")?;
    if lock(&app.state::<ChatState>().runs).contains_key(session_id) {
        return Err("这个对话还在回答，请等它结束或先停止".into());
    }
    Ok(summary)
}

fn add_note(app: &tauri::AppHandle, session_id: &str, note: String) -> Result<(), String> {
    let store = JsonlStore::new(Dirs::of(app)?.sessions());
    if !store.load(session_id).map_err(|e| e.to_string())?.is_empty() {
        store.append(session_id, &[Message::reminder(note)]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn changed(app: &tauri::AppHandle) {
    if let Ok(sessions) = mutate(app, |index| index.list()) {
        emit(app, "chat-sessions-changed", json!(sessions));
    }
}

/// What the page needs: whether the conversation is in a copy and how it stands, or whether it could make one.
#[tauri::command(async)]
pub fn chat_worktree_status(app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String) -> Result<Value, String> {
    if window.label() != "control" {
        return Err("请在聊天页使用独立副本".into());
    }
    checked_session_id(&session_id)?;
    let summary = mutate(&app, |index| index.get(&session_id).cloned())?.ok_or("这个对话已经不存在")?;
    let store = store(&crate::storage::root(&app)?);
    if let Some(link) = &summary.worktree {
        return Ok(json!({ "active": true, "link": link, "status": status(&store, link)? }));
    }
    let config = ai_config::current(&app)?;
    let default = ai_config::default_workspace(&app)?;
    let folder = workspace::of(&summary, &config, &default);
    Ok(match repo_root(&folder) {
        Err(reason) => json!({ "active": false, "available": false, "reason": reason }),
        Ok(root) => match git(&root, &["rev-parse", "--verify", "HEAD"], SHORT_TIMEOUT) {
            Ok(_) => json!({ "active": false, "available": true }),
            Err(_) => json!({ "active": false, "available": false, "reason": "这个仓库还没有任何提交，先提交一次再使用独立副本" }),
        },
    })
}

/// Moves the conversation into a new copy of its folder.
#[tauri::command(async)]
pub fn chat_worktree_start(app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String) -> Result<Value, String> {
    let summary = guard(&app, &window, &session_id)?;
    if summary.worktree.is_some() {
        return Err("这个对话已经在独立副本里了".into());
    }
    let config = ai_config::current(&app)?;
    let default = ai_config::default_workspace(&app)?;
    let origin = workspace::of(&summary, &config, &default);
    let created = create(&origin, &store(&crate::storage::root(&app)?), &session_id)?;
    add_note(&app, &session_id, started_note(&created.link, &created.workspace))?;
    let folder = created.workspace.display().to_string();
    let summary = mutate(&app, |index| {
        index.set_worktree(&session_id, &folder, Some(created.link.clone()));
        index.get(&session_id).cloned()
    })?
    .ok_or("这个对话已经不存在")?;
    changed(&app);
    Ok(json!({ "session": summary }))
}

/// Ends the copy: `merge` brings its branch back into the original folder, `keep` leaves it on disk, `discard`
/// deletes it. The conversation goes back to the original folder in every case; a refused merge changes nothing.
#[tauri::command(async)]
pub fn chat_worktree_finish(app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String, action: String) -> Result<Value, String> {
    let summary = guard(&app, &window, &session_id)?;
    let link = summary.worktree.clone().ok_or("这个对话没有在独立副本里")?;
    let store = store(&crate::storage::root(&app)?);
    let message = match action.as_str() {
        "merge" => merge(&store, &link, &summary.title)?,
        "discard" => {
            discard(&store, &link)?;
            "已丢弃副本和它的改动".to_string()
        }
        "keep" => format!("副本保留在“{}”，分支 {}", link.path, link.branch),
        _ => return Err("操作只能是 merge、keep 或 discard".into()),
    };
    add_note(&app, &session_id, finished_note(&link, &action))?;
    let origin = link.origin.clone();
    let summary = mutate(&app, |index| {
        index.set_worktree(&session_id, &origin, None);
        index.get(&session_id).cloned()
    })?
    .ok_or("这个对话已经不存在")?;
    changed(&app);
    Ok(json!({ "session": summary, "message": message }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git").arg("-C").arg(dir).args(["-c", "user.name=t", "-c", "user.email=t@t"]).args(args).output().unwrap();
        assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Git on Windows may check files out with CRLF; the tests care about content, not line endings.
    fn text(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap().replace("\r\n", "\n")
    }

    struct Rig {
        base: PathBuf,
        repo: PathBuf,
        store: PathBuf,
    }

    impl Rig {
        fn new(tag: &str) -> Self {
            let base = std::env::temp_dir().join(format!("xiaoban-worktree-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&base);
            let repo = base.join("project");
            std::fs::create_dir_all(repo.join("src")).unwrap();
            run(&repo, &["init", "-q", "-b", "main"]);
            std::fs::write(repo.join("a.txt"), "one\n").unwrap();
            std::fs::write(repo.join("src/lib.txt"), "lib\n").unwrap();
            run(&repo, &["add", "-A"]);
            run(&repo, &["commit", "-q", "-m", "first"]);
            Self { repo, store: base.join("data/worktrees"), base }
        }
        fn make(&self, session: &str) -> Created {
            create(&self.repo, &self.store, session).unwrap()
        }
    }

    impl Drop for Rig {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    #[test]
    fn a_copy_is_made_outside_the_folder_on_its_own_branch() {
        let rig = Rig::new("create");
        let made = rig.make("session-one");
        assert!(made.link.path.starts_with(&rig.store.display().to_string()), "{}", made.link.path);
        assert!(made.workspace.join("a.txt").is_file(), "files are checked out in the copy");
        assert!(made.link.branch.starts_with("xiaoban/"));
        assert_eq!(run(&rig.repo, &["rev-parse", "HEAD"]), made.link.base);
        assert!(!rig.repo.join(".xiaoban").exists() && rig.repo.read_dir().unwrap().count() == 3, "nothing is added to the user's folder besides .git bookkeeping");
    }

    #[test]
    fn working_in_a_sub_folder_keeps_the_same_sub_folder_in_the_copy() {
        let rig = Rig::new("subdir");
        let made = create(&rig.repo.join("src"), &rig.store, "s").unwrap();
        assert!(made.workspace.ends_with("src") && made.workspace.join("lib.txt").is_file(), "{}", made.workspace.display());
    }

    #[test]
    fn edits_in_the_copy_do_not_reach_the_original_until_merged() {
        let rig = Rig::new("merge");
        let made = rig.make("s");
        std::fs::write(made.workspace.join("a.txt"), "changed in the copy\n").unwrap();
        std::fs::write(made.workspace.join("new.txt"), "brand new\n").unwrap();
        assert_eq!(text(&rig.repo.join("a.txt")), "one\n");
        let state = status(&rig.store, &made.link).unwrap();
        assert!(state.exists && state.changed_count == 2 && state.ahead == 0 && state.origin_clean && state.origin_branch == "main", "{state:?}");
        let message = merge(&rig.store, &made.link, "改动测试").unwrap();
        assert!(message.contains("合并进“main”"), "{message}");
        assert_eq!(text(&rig.repo.join("a.txt")), "changed in the copy\n");
        assert!(rig.repo.join("new.txt").is_file());
        assert!(!Path::new(&made.link.path).exists(), "the copy is removed after a merge");
        assert!(run(&rig.repo, &["branch", "--list", &made.link.branch]).is_empty(), "and so is its branch");
        assert!(run(&rig.repo, &["log", "-1", "--format=%s"]).contains("合并小伴的改动"));
    }

    #[test]
    fn merges_large_diffstat_output_and_reports_the_committed_result() {
        let rig = Rig::new("large-output");
        let made = rig.make("large");
        for n in 0..2000 {
            std::fs::write(made.workspace.join(format!("file-{n:04}-normal-source-name.txt")), "new data\n").unwrap();
        }
        // Commit outside the helper so this specifically exercises merge's output and result handling.
        run(&made.workspace, &["add", "-A"]);
        run(&made.workspace, &["commit", "-q", "-m", "many files"]);
        assert!(merge(&rig.store, &made.link, "大批文件").unwrap().contains("合并进"));
        assert!(rig.repo.join("file-0000-normal-source-name.txt").is_file());
        assert!(run(&rig.repo, &["log", "-1", "--format=%s"]).contains("大批文件"));
        assert!(!made.workspace.exists());
    }

    #[test]
    fn a_merge_is_refused_while_the_original_has_uncommitted_changes() {
        let rig = Rig::new("dirty");
        let made = rig.make("s");
        std::fs::write(made.workspace.join("a.txt"), "from the copy\n").unwrap();
        std::fs::write(rig.repo.join("a.txt"), "typed by the user\n").unwrap();
        let error = merge(&rig.store, &made.link, "t").unwrap_err();
        assert!(error.contains("没提交的改动"), "{error}");
        assert_eq!(text(&rig.repo.join("a.txt")), "typed by the user\n", "the user's work is untouched");
        assert!(Path::new(&made.link.path).is_dir(), "and the copy is kept");
    }

    #[test]
    fn a_conflicting_merge_is_aborted_and_keeps_everything() {
        let rig = Rig::new("conflict");
        let made = rig.make("s");
        std::fs::write(made.workspace.join("a.txt"), "copy side\n").unwrap();
        std::fs::write(rig.repo.join("a.txt"), "original side\n").unwrap();
        run(&rig.repo, &["commit", "-q", "-am", "original moves on"]);
        let error = merge(&rig.store, &made.link, "t").unwrap_err();
        assert!(error.contains("冲突") && error.contains(&made.link.branch), "{error}");
        assert_eq!(text(&rig.repo.join("a.txt")), "original side\n");
        assert!(run(&rig.repo, &["status", "--porcelain"]).is_empty(), "no half-finished merge is left behind");
        assert!(Path::new(&made.link.path).is_dir() && !run(&rig.repo, &["branch", "--list", &made.link.branch]).is_empty());
    }

    #[test]
    fn discarding_removes_the_copy_and_its_branch_and_nothing_else() {
        let rig = Rig::new("discard");
        let made = rig.make("s");
        std::fs::write(made.workspace.join("a.txt"), "throw away\n").unwrap();
        discard(&rig.store, &made.link).unwrap();
        assert!(!Path::new(&made.link.path).exists());
        assert!(run(&rig.repo, &["branch", "--list", &made.link.branch]).is_empty());
        assert_eq!(text(&rig.repo.join("a.txt")), "one\n");
        discard(&rig.store, &made.link).unwrap();
    }

    #[test]
    fn a_copy_without_changes_is_just_cleaned_up_by_a_merge() {
        let rig = Rig::new("empty");
        let made = rig.make("s");
        assert!(merge(&rig.store, &made.link, "t").unwrap().contains("没有改动"));
        assert!(!Path::new(&made.link.path).exists());
    }

    #[test]
    fn two_conversations_get_separate_copies_and_branches() {
        let rig = Rig::new("two");
        let (a, b) = (rig.make("alpha"), rig.make("beta"));
        assert_ne!(a.link.path, b.link.path);
        assert_ne!(a.link.branch, b.link.branch);
        std::fs::write(a.workspace.join("a.txt"), "A\n").unwrap();
        assert_eq!(text(&b.workspace.join("a.txt")), "one\n", "they cannot see each other's edits");
    }

    #[test]
    fn a_link_outside_the_app_folder_is_never_touched() {
        let rig = Rig::new("guard");
        let made = rig.make("s");
        let forged = WorktreeLink { path: rig.repo.display().to_string(), ..made.link.clone() };
        assert!(discard(&rig.store, &forged).is_err() && merge(&rig.store, &forged, "t").is_err() && status(&rig.store, &forged).is_err());
        assert!(rig.repo.join("a.txt").is_file());
    }

    #[test]
    fn folders_that_cannot_have_a_copy_say_why() {
        let base = std::env::temp_dir().join(format!("xiaoban-worktree-none-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        assert!(create(&base, &base.join("w"), "s").unwrap_err().contains("不在 git 仓库里"));
        run(&base, &["init", "-q", "-b", "main"]);
        assert!(create(&base, &base.join("w"), "s").unwrap_err().contains("还没有任何提交"));
        let _ = std::fs::remove_dir_all(&base);
    }
}

//! The list of conversations: titles and times, kept beside the conversation files.
//!
//! The index is only a convenience. If it is missing or damaged it is rebuilt from the
//! conversation files themselves, so a bad index never loses a conversation.

use agent_kernel::{JsonlStore, Role, SessionStore};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const DEFAULT_TITLE: &str = "新对话";
const MAX_TITLE_CHARS: usize = 40;
/// A name the user typed may be longer than the title cut from a first message.
const MAX_NAME_CHARS: usize = 60;
const MAX_GROUP_NAME_CHARS: usize = 20;
const MAX_SESSIONS: usize = 500;
const MAX_GROUPS: usize = 30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub permission_mode: String,
    /// The group this conversation was moved into; empty when it is not in one.
    pub group: String,
    /// The folder this conversation works in. Empty for one made before conversations had their own: it works in
    /// the default working folder (the setting), as it always did.
    pub workspace: String,
    /// A companion is an identity, not a filesystem workspace. Empty for ordinary tasks.
    pub companion_id: String,
    pub companion_name: String,
    pub companion_cursor: u64,
    /// Set while this conversation works in an isolated git worktree. `workspace` is then the copy's folder
    /// and `origin` below is where the user's own files are.
    pub worktree: Option<WorktreeLink>,
    /// The expert (a WorkBuddy-style agent persona) this conversation was started with; `None` for an ordinary task.
    pub expert: Option<ExpertBinding>,
}

/// The installed expert a conversation is bound to: the plugin it lives in and the agent inside it, plus the
/// name and profession the page shows. Binding only decides whose persona is put in front of the model;
/// tools and permissions stay what the conversation already has.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertBinding {
    pub plugin: String,
    pub agent: String,
    pub name: String,
    pub profession: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
}

/// The link between a conversation and the git worktree it works in (see `worktree`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeLink {
    /// The folder the user was in when the copy was made; the conversation goes back to it afterwards.
    pub origin: String,
    /// The root of the copy.
    pub path: String,
    pub branch: String,
    /// The commit the copy started from.
    pub base: String,
}

impl Default for SessionSummary {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: DEFAULT_TITLE.into(),
            created_at: 0,
            updated_at: 0,
            permission_mode: "ask".into(),
            group: String::new(),
            workspace: String::new(),
            companion_id: String::new(), companion_name: String::new(), companion_cursor: 0,
            worktree: None,
            expert: None,
        }
    }
}

/// A folder in the conversation list. Conversations only point at it, so deleting a group never deletes them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
}

/// Everything the list shows, sent together so the page never sees conversations and groups out of step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Library {
    pub sessions: Vec<SessionSummary>,
    pub groups: Vec<Group>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct IndexFile {
    version: u32,
    sessions: Vec<SessionSummary>,
    groups: Vec<Group>,
}

/// Ids are written by us, so they use a small alphabet that is safe in file names.
pub fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub fn new_id(now_millis: u128) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let random = getrandom::u32().unwrap_or(0);
    format!("c{now_millis:x}-{random:08x}-{:x}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// The first non-empty line, with runs of spaces squeezed, cut to a short title.
pub fn title_from(text: &str) -> String {
    let line = text.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).find(|l| !l.is_empty()).unwrap_or_default();
    if line.is_empty() {
        return DEFAULT_TITLE.to_string();
    }
    match line.char_indices().nth(MAX_TITLE_CHARS) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line,
    }
}

pub struct Index {
    path: PathBuf,
    sessions: Vec<SessionSummary>,
    groups: Vec<Group>,
}

/// Keeps only groups that can be told apart, and sends conversations whose group is gone back to the top level.
fn tidy(sessions: &mut [SessionSummary], groups: Vec<Group>) -> Vec<Group> {
    let mut kept: Vec<Group> = Vec::new();
    for group in groups {
        let name = group.name.trim().to_string();
        if valid_id(&group.id) && !name.is_empty() && !kept.iter().any(|g| g.id == group.id) {
            kept.push(Group { id: group.id, name });
        }
    }
    for session in sessions.iter_mut() {
        if !session.group.is_empty() && !kept.iter().any(|g| g.id == session.group) {
            session.group.clear();
        }
    }
    kept
}

/// Collapses runs of whitespace, then checks the result is a usable name.
fn clean_name(raw: &str, max_chars: usize, what: &str) -> Result<String, String> {
    let name = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err(format!("{what}不能为空"));
    }
    if name.chars().any(char::is_control) {
        return Err(format!("{what}里不能有特殊控制字符"));
    }
    if name.chars().count() > max_chars {
        return Err(format!("{what}最多 {max_chars} 个字"));
    }
    Ok(name)
}

fn new_group_id(now_millis: u128) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let random = getrandom::u32().unwrap_or(0);
    format!("g{now_millis:x}-{random:08x}-{:x}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

impl Index {
    /// `dir` holds `index.json`; `store` is where the conversation files are.
    pub fn load(dir: &Path, sessions_dir: &Path, store: &JsonlStore) -> Self {
        let path = dir.join("index.json");
        let (mut sessions, groups) = match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<IndexFile>(&bytes) {
                Ok(file) => (file.sessions.into_iter().filter(|s| valid_id(&s.id)).collect(), file.groups),
                Err(_) => {
                    // Keep the damaged file for inspection instead of writing over it.
                    let _ = fs::rename(&path, dir.join(format!("index.json.damaged-{}", std::process::id())));
                    (rebuild(sessions_dir, store), Vec::new())
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => (rebuild(sessions_dir, store), Vec::new()),
            Err(_) => (Vec::new(), Vec::new()),
        };
        let groups = tidy(&mut sessions, groups);
        Self { path, sessions, groups }
    }

    pub fn groups(&self) -> Vec<Group> {
        self.groups.clone()
    }

    pub fn library(&self) -> Library {
        Library { sessions: self.list(), groups: self.groups() }
    }

    /// The name is the user's own, so later messages never replace it.
    pub fn rename(&mut self, id: &str, raw: &str) -> Result<(), String> {
        let title = clean_name(raw, MAX_NAME_CHARS, "对话名称")?;
        let session = self.sessions.iter_mut().find(|s| s.id == id).ok_or("这个对话已经不存在")?;
        session.title = title;
        Ok(())
    }

    /// `None` or an empty id takes the conversation out of its group.
    pub fn move_session(&mut self, id: &str, group: Option<&str>) -> Result<(), String> {
        let target = group.unwrap_or_default();
        if !target.is_empty() && !self.groups.iter().any(|g| g.id == target) {
            return Err("这个分组已经不存在".into());
        }
        let session = self.sessions.iter_mut().find(|s| s.id == id).ok_or("这个对话已经不存在")?;
        session.group = target.to_string();
        Ok(())
    }

    fn check_group_name(&self, raw: &str, except: Option<&str>) -> Result<String, String> {
        let name = clean_name(raw, MAX_GROUP_NAME_CHARS, "分组名称")?;
        let taken = self.groups.iter().any(|g| Some(g.id.as_str()) != except && g.name.to_lowercase() == name.to_lowercase());
        if taken {
            return Err("已经有同名的分组".into());
        }
        Ok(name)
    }

    pub fn create_group(&mut self, now_millis: u128, raw: &str) -> Result<Group, String> {
        if self.groups.len() >= MAX_GROUPS {
            return Err(format!("分组最多 {MAX_GROUPS} 个，请先删除一些不用的分组"));
        }
        let group = Group { id: new_group_id(now_millis), name: self.check_group_name(raw, None)? };
        self.groups.push(group.clone());
        Ok(group)
    }

    pub fn rename_group(&mut self, id: &str, raw: &str) -> Result<(), String> {
        let name = self.check_group_name(raw, Some(id))?;
        let group = self.groups.iter_mut().find(|g| g.id == id).ok_or("这个分组已经不存在")?;
        group.name = name;
        Ok(())
    }

    /// The conversations inside stay, and go back to the top level.
    pub fn delete_group(&mut self, id: &str) -> Result<(), String> {
        if !self.groups.iter().any(|g| g.id == id) {
            return Err("这个分组已经不存在".into());
        }
        self.groups.retain(|g| g.id != id);
        for session in self.sessions.iter_mut().filter(|s| s.group == id) {
            session.group.clear();
        }
        Ok(())
    }

    /// Newest first.
    pub fn list(&self) -> Vec<SessionSummary> {
        let mut list = self.sessions.clone();
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| b.created_at.cmp(&a.created_at)).then_with(|| a.id.cmp(&b.id)));
        list
    }

    pub fn get(&self, id: &str) -> Option<&SessionSummary> {
        self.sessions.iter().find(|s| s.id == id)
    }

    /// A new conversation starts in the mode the top bar was showing, so what the user sees is what takes effect.
    pub fn create(&mut self, now: i64, now_millis: u128, permission_mode: &str) -> Result<SessionSummary, String> {
        self.create_in(now, now_millis, permission_mode, "")
    }

    /// `create` in a working folder of its own.
    pub fn create_in(&mut self, now: i64, now_millis: u128, permission_mode: &str, workspace: &str) -> Result<SessionSummary, String> {
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(format!("对话最多保存 {MAX_SESSIONS} 个，请先删除一些旧对话"));
        }
        let summary = SessionSummary {
            id: new_id(now_millis), created_at: now, updated_at: now, permission_mode: permission_mode.into(), workspace: workspace.into(),
            ..SessionSummary::default()
        };
        self.sessions.push(summary.clone());
        Ok(summary)
    }

    pub fn set_expert(&mut self, id: &str, expert: Option<ExpertBinding>) -> bool {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) {
            s.expert = expert;
            true
        } else {
            false
        }
    }

    pub fn set_workspace(&mut self, id: &str, workspace: &str) -> bool {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) {
            s.workspace = workspace.to_string();
            true
        } else {
            false
        }
    }

    /// Moves a conversation into (`Some`) or out of a worktree together with its folder.
    pub fn set_worktree(&mut self, id: &str, workspace: &str, link: Option<WorktreeLink>) -> bool {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) {
            s.workspace = workspace.to_string();
            s.worktree = link;
            true
        } else {
            false
        }
    }

    pub fn set_companion(&mut self, id: &str, key: &str, name: &str, cursor: u64) {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.id == id) {
            if session.companion_id.is_empty() || session.title == session.companion_name { session.title = name.chars().take(MAX_NAME_CHARS).collect(); }
            session.companion_id = key.into(); session.companion_name = name.into();
            session.companion_cursor = cursor;
        }
    }

    pub fn set_permission_mode(&mut self, id: &str, mode: &str) -> bool {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) {
            s.permission_mode = mode.to_string();
            true
        } else {
            false
        }
    }

    pub fn touch(&mut self, id: &str, now: i64) {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) {
            s.updated_at = now.max(s.updated_at);
        }
    }

    /// Names a conversation after its first message; later messages never rename it.
    pub fn name_from_first_message(&mut self, id: &str, text: &str) {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id && s.title == DEFAULT_TITLE) {
            s.title = title_from(text);
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.sessions.len();
        self.sessions.retain(|s| s.id != id);
        self.sessions.len() != before
    }

    pub fn save(&self) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| io::Error::other("索引文件没有父目录"))?;
        fs::create_dir_all(parent)?;
        let temporary = parent.join(format!("index-{}.tmp", std::process::id()));
        let result = (|| {
            let bytes = serde_json::to_vec_pretty(&IndexFile { version: 1, sessions: self.sessions.clone(), groups: self.groups.clone() }).map_err(io::Error::other)?;
            let mut file = fs::File::create(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &self.path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn seconds(time: io::Result<std::time::SystemTime>) -> i64 {
    time.ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs() as i64)
}

/// Recovers the list from the conversation files: titles from the first user message, times from the files.
fn rebuild(sessions_dir: &Path, store: &JsonlStore) -> Vec<SessionSummary> {
    let Ok(read) = fs::read_dir(sessions_dir) else { return Vec::new() };
    let mut found = Vec::new();
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(id) = name.strip_prefix("s-").and_then(|n| n.strip_suffix(".jsonl")) else { continue };
        if !valid_id(id) || !entry.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }
        let meta = entry.metadata().ok();
        let first_user = store.load(id).ok().and_then(|messages| messages.into_iter().find(|m| m.role == Role::User)).map(|m| m.text());
        let updated_at = meta.as_ref().map_or(0, |m| seconds(m.modified()));
        let created_at = meta.as_ref().map_or(updated_at, |m| seconds(m.created()).min(updated_at).max(0));
        found.push(SessionSummary { id: id.to_string(), title: first_user.map_or_else(|| DEFAULT_TITLE.to_string(), |t| title_from(&t)), created_at, updated_at, ..SessionSummary::default() });
        if found.len() >= MAX_SESSIONS {
            break;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::Message;

    fn temp(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiaoban-sessions-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn open(dir: &Path) -> (Index, JsonlStore) {
        let sessions = dir.join("sessions");
        let store = JsonlStore::new(&sessions);
        (Index::load(dir, &sessions, &store), store)
    }

    #[test]
    fn titles_come_from_the_first_non_empty_line_and_stay_short() {
        assert_eq!(title_from("  \n\n  帮我整理   下载文件夹  \n第二行"), "帮我整理 下载文件夹");
        assert_eq!(title_from("   "), DEFAULT_TITLE);
        assert_eq!(title_from(""), DEFAULT_TITLE);
        let long = title_from(&"字".repeat(100));
        assert_eq!(long.chars().count(), 41);
        assert!(long.ends_with('…'));
        assert_eq!(title_from(&"a".repeat(40)), "a".repeat(40), "exactly the limit is not cut");
        assert!(title_from(&format!("{}😀tail", "x".repeat(39))).starts_with(&"x".repeat(39)), "cuts on character boundaries");
    }

    #[test]
    fn ids_are_unique_and_file_safe() {
        let ids: std::collections::HashSet<String> = (0..500).map(|_| new_id(1_700_000_000_000)).collect();
        assert_eq!(ids.len(), 500);
        assert!(ids.iter().all(|id| valid_id(id)));
        for bad in ["", "A", "a b", "../x", "a/b", "a.b", &"a".repeat(65)] {
            assert!(!valid_id(bad), "{bad:?}");
        }
    }

    #[test]
    fn conversations_are_created_named_once_touched_listed_newest_first_and_removed() {
        let dir = temp("lifecycle");
        let (mut index, _) = open(&dir);
        assert!(index.list().is_empty());
        let first = index.create(100, 1, "ask").unwrap();
        let second = index.create(200, 2, "ask").unwrap();
        assert_eq!((first.title.as_str(), second.updated_at), (DEFAULT_TITLE, 200));
        assert_eq!(index.list().iter().map(|s| s.id.clone()).collect::<Vec<_>>(), vec![second.id.clone(), first.id.clone()]);

        index.name_from_first_message(&first.id, "第一件事\n细节");
        index.name_from_first_message(&first.id, "后来的话不改名");
        assert_eq!(index.get(&first.id).unwrap().title, "第一件事");
        index.touch(&first.id, 300);
        index.touch(&first.id, 250);
        assert_eq!(index.get(&first.id).unwrap().updated_at, 300, "time never goes backwards");
        assert_eq!(index.list()[0].id, first.id);

        assert!(index.remove(&second.id));
        assert!(!index.remove(&second.id));
        assert!(index.get(&second.id).is_none());
        index.name_from_first_message("missing", "x");
        index.touch("missing", 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_new_conversation_starts_in_the_mode_it_is_given() {
        let dir = temp("new-mode");
        let (mut index, _) = open(&dir);
        let trusted = index.create(1, 1, "auto").unwrap();
        let careful = index.create(2, 2, "ask").unwrap();
        assert_eq!((trusted.permission_mode.as_str(), careful.permission_mode.as_str()), ("auto", "ask"));
        assert_eq!(index.get(&trusted.id).unwrap().permission_mode, "auto", "the stored copy agrees with what was returned");
        index.save().unwrap();
        let (again, _) = open(&dir);
        assert_eq!(again.get(&trusted.id).unwrap().permission_mode, "auto", "it survives a restart");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_list_survives_restart_and_leaves_no_temporary_files() {
        let dir = temp("persist");
        let (mut index, _) = open(&dir);
        let one = index.create(10, 1, "ask").unwrap();
        index.name_from_first_message(&one.id, "记住我");
        index.save().unwrap();
        let (again, _) = open(&dir);
        assert_eq!(again.list(), vec![SessionSummary { title: "记住我".into(), ..one }]);
        assert!(fs::read_dir(&dir).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().ends_with(".tmp")));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_missing_index_is_rebuilt_from_the_conversation_files() {
        let dir = temp("rebuild");
        let (_, store) = open(&dir);
        store.append("c1-aa-0", &[Message::user_text("  分析这份   报告\n谢谢"), Message::assistant("好的", vec![])]).unwrap();
        store.append("c2-bb-1", &[Message::assistant("没有用户消息", vec![])]).unwrap();
        fs::write(dir.join("sessions/not-a-session.txt"), "x").unwrap();
        fs::write(dir.join("sessions/s-UPPER.jsonl"), "x").unwrap();
        let (index, _) = open(&dir);
        let mut titles: Vec<(String, String)> = index.list().into_iter().map(|s| (s.id, s.title)).collect();
        titles.sort();
        assert_eq!(titles, vec![("c1-aa-0".to_string(), "分析这份 报告".to_string()), ("c2-bb-1".to_string(), DEFAULT_TITLE.to_string())]);
        assert!(index.list().iter().all(|s| s.updated_at > 0));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_damaged_index_is_kept_aside_and_rebuilt_without_losing_conversations() {
        let dir = temp("damaged");
        let (_, store) = open(&dir);
        store.append("c3-cc-2", &[Message::user_text("还在吗")]).unwrap();
        fs::write(dir.join("index.json"), "{ not json at all").unwrap();
        let (index, _) = open(&dir);
        assert_eq!(index.list().len(), 1);
        assert_eq!(index.list()[0].title, "还在吗");
        assert!(!dir.join("index.json").exists());
        let kept: Vec<_> = fs::read_dir(&dir).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with("index.json.damaged")).collect();
        assert_eq!(kept.len(), 1, "the broken file is preserved");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn entries_with_unsafe_ids_are_dropped_and_the_count_is_limited() {
        let dir = temp("limits");
        fs::write(dir.join("index.json"), r#"{"version":1,"sessions":[
            {"id":"ok-1","title":"a","createdAt":1,"updatedAt":1},{"id":"../evil","title":"b","createdAt":1,"updatedAt":1}]}"#).unwrap();
        let (mut index, _) = open(&dir);
        assert_eq!(index.list().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), vec!["ok-1"]);
        for i in 0..499 {
            index.create(i, i as u128 + 10, "ask").unwrap();
        }
        assert!(index.create(1000, 9999, "ask").unwrap_err().contains("最多保存"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_conversation_can_be_renamed_and_the_name_is_cleaned_checked_and_kept() {
        let dir = temp("rename");
        let (mut index, _) = open(&dir);
        let one = index.create(100, 1, "ask").unwrap();
        index.rename(&one.id, "  周报   第 3 周\n").unwrap();
        assert_eq!(index.get(&one.id).unwrap().title, "周报 第 3 周");
        index.name_from_first_message(&one.id, "用户第一句话");
        assert_eq!(index.get(&one.id).unwrap().title, "周报 第 3 周", "the first message does not rename a conversation the user named");
        assert_eq!(index.get(&one.id).unwrap().updated_at, 100, "renaming is not activity");

        assert!(index.rename(&one.id, "  \n ").unwrap_err().contains("不能为空"));
        assert!(index.rename(&one.id, "名\u{7}字").unwrap_err().contains("控制字符"));
        assert!(index.rename(&one.id, &"长".repeat(61)).unwrap_err().contains("最多 60"));
        index.rename(&one.id, &"长".repeat(60)).unwrap();
        assert!(index.rename("gone", "x").unwrap_err().contains("已经不存在"));
        assert_eq!(index.get(&one.id).unwrap().title.chars().count(), 60, "a refused name leaves the old one");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn groups_are_created_with_clean_unique_names_and_a_limit() {
        let dir = temp("groups-create");
        let (mut index, _) = open(&dir);
        let work = index.create_group(1, "  工作   项目 ").unwrap();
        assert_eq!(work.name, "工作 项目");
        assert!(valid_id(&work.id) && work.id.starts_with('g'), "group ids are file-safe and cannot be mistaken for conversation ids");
        assert!(index.create_group(2, "工作 项目").unwrap_err().contains("同名"));
        assert!(index.create_group(3, "Abc").is_ok());
        assert!(index.create_group(4, "aBC").unwrap_err().contains("同名"), "names differing only in case count as the same");
        assert!(index.create_group(5, "").unwrap_err().contains("不能为空"));
        assert!(index.create_group(6, &"组".repeat(21)).unwrap_err().contains("最多 20"));
        for i in 0..28 {
            index.create_group(10 + i, &format!("组{i}")).unwrap();
        }
        assert_eq!(index.groups().len(), 30);
        assert!(index.create_group(99, "多一个").unwrap_err().contains("最多 30"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_conversation_moves_into_a_group_and_back_out() {
        let dir = temp("groups-move");
        let (mut index, _) = open(&dir);
        let one = index.create(100, 1, "ask").unwrap();
        let group = index.create_group(1, "工作").unwrap();
        assert_eq!(index.get(&one.id).unwrap().group, "");
        index.move_session(&one.id, Some(&group.id)).unwrap();
        assert_eq!(index.get(&one.id).unwrap().group, group.id);
        assert_eq!(index.get(&one.id).unwrap().updated_at, 100, "moving is not activity");
        index.move_session(&one.id, None).unwrap();
        assert_eq!(index.get(&one.id).unwrap().group, "");
        index.move_session(&one.id, Some(&group.id)).unwrap();
        index.move_session(&one.id, Some("")).unwrap();
        assert_eq!(index.get(&one.id).unwrap().group, "", "an empty id also means no group");

        assert!(index.move_session(&one.id, Some("g-missing")).unwrap_err().contains("分组已经不存在"));
        assert!(index.move_session("gone", Some(&group.id)).unwrap_err().contains("对话已经不存在"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_group_can_be_renamed_but_not_to_another_groups_name() {
        let dir = temp("groups-rename");
        let (mut index, _) = open(&dir);
        let a = index.create_group(1, "工作").unwrap();
        let b = index.create_group(2, "生活").unwrap();
        index.rename_group(&a.id, "工作 2").unwrap();
        index.rename_group(&a.id, "工作 2").unwrap();
        index.rename_group(&b.id, "生活".to_uppercase().as_str()).unwrap();
        assert!(index.rename_group(&b.id, "工作 2").unwrap_err().contains("同名"));
        assert!(index.rename_group(&b.id, " ").unwrap_err().contains("不能为空"));
        assert!(index.rename_group("g-missing", "新").unwrap_err().contains("已经不存在"));
        assert_eq!(index.groups().iter().map(|g| g.name.as_str()).collect::<Vec<_>>(), vec!["工作 2", "生活".to_uppercase().as_str()]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn deleting_a_group_keeps_its_conversations() {
        let dir = temp("groups-delete");
        let (mut index, _) = open(&dir);
        let inside = index.create(100, 1, "ask").unwrap();
        let elsewhere = index.create(200, 2, "ask").unwrap();
        let doomed = index.create_group(1, "要删的").unwrap();
        let other = index.create_group(2, "留着的").unwrap();
        index.move_session(&inside.id, Some(&doomed.id)).unwrap();
        index.move_session(&elsewhere.id, Some(&other.id)).unwrap();

        index.delete_group(&doomed.id).unwrap();
        assert_eq!(index.groups(), vec![other.clone()]);
        assert_eq!(index.list().len(), 2, "no conversation is lost");
        assert_eq!(index.get(&inside.id).unwrap().group, "");
        assert_eq!(index.get(&elsewhere.id).unwrap().group, other.id, "other groups are untouched");
        assert!(index.delete_group(&doomed.id).unwrap_err().contains("已经不存在"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn groups_and_names_survive_a_restart_and_the_library_carries_both() {
        let dir = temp("groups-persist");
        let (mut index, _) = open(&dir);
        let one = index.create(100, 1, "ask").unwrap();
        let group = index.create_group(1, "工作").unwrap();
        index.move_session(&one.id, Some(&group.id)).unwrap();
        index.rename(&one.id, "我取的名字").unwrap();
        index.save().unwrap();

        let (again, _) = open(&dir);
        let library = again.library();
        assert_eq!(library.groups, vec![group.clone()]);
        assert_eq!((library.sessions[0].title.as_str(), library.sessions[0].group.as_str()), ("我取的名字", group.id.as_str()));
        let json = serde_json::to_value(&library).unwrap();
        assert_eq!(json["sessions"][0]["group"], group.id.as_str());
        assert_eq!(json["groups"][0]["name"], "工作");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_index_from_before_groups_existed_loads_and_a_dangling_group_is_cleaned() {
        let dir = temp("groups-legacy");
        fs::write(dir.join("index.json"), r#"{"version":1,"sessions":[{"id":"old-1","title":"旧对话","createdAt":1,"updatedAt":1}]}"#).unwrap();
        let (index, _) = open(&dir);
        assert_eq!((index.list()[0].group.as_str(), index.groups().len()), ("", 0));
        drop(index);

        fs::write(
            dir.join("index.json"),
            r#"{"version":1,"groups":[{"id":"g1","name":" 工作 "},{"id":"g1","name":"重复编号"},{"id":"../bad","name":"坏编号"},{"id":"g2","name":"  "}],
                "sessions":[{"id":"a","title":"在组里","createdAt":1,"updatedAt":1,"group":"g1"},{"id":"b","title":"组没了","createdAt":1,"updatedAt":2,"group":"g9"}]}"#,
        )
        .unwrap();
        let (index, _) = open(&dir);
        assert_eq!(index.groups(), vec![Group { id: "g1".into(), name: "工作".into() }], "duplicates, unsafe ids and blank names are dropped");
        assert_eq!(index.get("a").unwrap().group, "g1");
        assert_eq!(index.get("b").unwrap().group, "", "a conversation pointing at a group that is gone goes back to the top level");
        fs::remove_dir_all(dir).unwrap();
    }
}

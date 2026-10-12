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
const MAX_SESSIONS: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub permission_mode: String,
}

impl Default for SessionSummary {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: DEFAULT_TITLE.into(),
            created_at: 0,
            updated_at: 0,
            permission_mode: "ask".into(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct IndexFile {
    version: u32,
    sessions: Vec<SessionSummary>,
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
}

impl Index {
    /// `dir` holds `index.json`; `store` is where the conversation files are.
    pub fn load(dir: &Path, sessions_dir: &Path, store: &JsonlStore) -> Self {
        let path = dir.join("index.json");
        let sessions = match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<IndexFile>(&bytes) {
                Ok(file) => file.sessions.into_iter().filter(|s| valid_id(&s.id)).collect(),
                Err(_) => {
                    // Keep the damaged file for inspection instead of writing over it.
                    let _ = fs::rename(&path, dir.join(format!("index.json.damaged-{}", std::process::id())));
                    rebuild(sessions_dir, store)
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => rebuild(sessions_dir, store),
            Err(_) => Vec::new(),
        };
        Self { path, sessions }
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
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(format!("对话最多保存 {MAX_SESSIONS} 个，请先删除一些旧对话"));
        }
        let summary = SessionSummary { id: new_id(now_millis), title: DEFAULT_TITLE.into(), created_at: now, updated_at: now, permission_mode: permission_mode.into() };
        self.sessions.push(summary.clone());
        Ok(summary)
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
            let bytes = serde_json::to_vec_pretty(&IndexFile { version: 1, sessions: self.sessions.clone() }).map_err(io::Error::other)?;
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
        found.push(SessionSummary { id: id.to_string(), title: first_user.map_or_else(|| DEFAULT_TITLE.to_string(), |t| title_from(&t)), created_at, updated_at, permission_mode: "ask".into() });
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
}

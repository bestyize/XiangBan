//! Read-only search over saved user/assistant text. Model state, tools and images are excluded.
use super::{sessions::SessionSummary, Dirs};
use agent_kernel::{JsonlStore, Role, SessionStore};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

static GENERATION: AtomicU64 = AtomicU64::new(0);
#[derive(Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Field { #[default] All, Title, Content }
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Request {
    query: String,
    field: Field,
    session_ids: Option<Vec<String>>,
    labels: HashMap<String, String>,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    session_id: String, title: String, updated_at: i64,
    excerpt: String, matched_field: &'static str, score: usize,
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response { hits: Vec<Hit>, skipped: usize, cancelled: bool }

fn excerpt(text: &str, terms: &[String]) -> String {
    let lower = text.to_lowercase();
    let at = terms.iter().filter_map(|term| lower.find(term)).min().unwrap_or(0);
    let start = lower[..at].chars().count().saturating_sub(16);
    let snippet: String = text.chars().skip(start).take(110).collect();
    format!("{}{}{}", if start > 0 { "…" } else { "" }, snippet.replace(['\r', '\n'], " "), if text.chars().count() > start + 110 { "…" } else { "" })
}

fn search(store: &dyn SessionStore, sessions: Vec<SessionSummary>, request: Request, cancelled: impl Fn() -> bool) -> Result<Response, String> {
    if request.query.chars().count() > 200 { return Err("关键词最多 200 个字符".into()); }
    let terms: Vec<String> = request.query.split_whitespace().map(str::to_lowercase).collect();
    let mut result = Response::default();
    for session in sessions {
        if cancelled() { result.cancelled = true; return Ok(result); }
        if request.session_ids.as_ref().is_some_and(|ids| !ids.contains(&session.id)) { continue; }
        let title = session.title.to_lowercase();
        let label = request.labels.get(&session.id).map(String::as_str).unwrap_or(&session.workspace).to_lowercase();
        let title_hits: Vec<bool> = terms.iter().map(|term| title.contains(term)).collect();
        let label_hits: Vec<bool> = terms.iter().map(|term| label.contains(term)).collect();
        let mut found: Vec<bool> = terms.iter().enumerate().map(|(i, _)| match request.field {
            Field::Title => title_hits[i], Field::All => title_hits[i] || label_hits[i], Field::Content => false,
        }).collect();
        let mut body_hits = vec![false; terms.len()];
        let mut snippet = String::new();
        if !terms.is_empty() && request.field != Field::Title && (request.field == Field::Content || !found.iter().all(|v| *v)) {
            match store.load(&session.id) {
                Ok(messages) => for message in messages {
                    if message.synthetic || !matches!(message.role, Role::User | Role::Assistant) { continue; }
                    let text = message.text();
                    let lower = text.to_lowercase();
                    let mut matched = false;
                    for (i, term) in terms.iter().enumerate() {
                        if lower.contains(term) { body_hits[i] = true; found[i] = true; matched = true; }
                    }
                    if matched && snippet.is_empty() { snippet = excerpt(&text, &terms); }
                    if found.iter().all(|v| *v) { break; }
                },
                Err(_) => { result.skipped += 1; continue; }
            }
        }
        if !found.iter().all(|v| *v) { continue; }
        let score = title_hits.iter().filter(|v| **v).count() * 12 + label_hits.iter().filter(|v| **v).count() * 6 + body_hits.iter().filter(|v| **v).count() * 4;
        let matched_field = if request.field == Field::Content { "content" } else if title_hits.iter().any(|v| *v) { "title" } else if body_hits.iter().any(|v| *v) { "content" } else { "workspace" };
        result.hits.push(Hit { session_id: session.id, title: session.title, updated_at: session.updated_at, excerpt: snippet, matched_field, score });
    }
    result.hits.sort_by(|a, b| b.score.cmp(&a.score).then(b.updated_at.cmp(&a.updated_at)).then(a.session_id.cmp(&b.session_id)));
    if terms.is_empty() { result.hits.truncate(8); }
    Ok(result)
}

#[tauri::command]
pub async fn chat_search_sessions(app: tauri::AppHandle, window: tauri::WebviewWindow, request: Request) -> Result<Response, String> {
    super::chat_window(&window)?;
    let sessions = super::list(&app)?;
    let root = Dirs::of(&app)?.sessions();
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    tauri::async_runtime::spawn_blocking(move || search(&JsonlStore::new(root), sessions, request, || GENERATION.load(Ordering::Relaxed) != generation))
        .await.map_err(|e| format!("会话搜索失败：{e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::{MemoryStore, Message};
    fn session(id: &str, title: &str, updated_at: i64) -> SessionSummary { SessionSummary { id: id.into(), title: title.into(), updated_at, ..Default::default() } }
    #[test]
    fn searches_saved_content_across_messages_but_excludes_tools_and_synthetic_text() {
        let store = MemoryStore::new();
        store.append("one", &[Message::user_text("读取 Word 文档"), Message::user_text("包含 OCR 方案")]).unwrap();
        let mut hidden = Message::user_text("隐藏关键词"); hidden.synthetic = true;
        let mut tool = Message::user_text("工具秘密"); tool.role = Role::Tool;
        store.append("two", &[hidden, tool]).unwrap();
        let sessions = vec![session("one", "阅读", 1), session("two", "其他", 2)];
        let result = search(&store, sessions.clone(), Request { query: "文档 ocr".into(), field: Field::Content, ..Default::default() }, || false).unwrap();
        assert_eq!(result.hits.len(), 1); assert_eq!(result.hits[0].session_id, "one"); assert_eq!(result.hits[0].matched_field, "content");
        for query in ["隐藏关键词", "工具秘密"] { assert!(search(&store, sessions.clone(), Request { query: query.into(), ..Default::default() }, || false).unwrap().hits.is_empty()); }
    }
    #[test]
    fn filters_titles_scope_and_aliases_without_loading_other_conversations() {
        let store = MemoryStore::new();
        let sessions = vec![session("one", "文档", 1), session("two", "代码", 2)];
        let request = Request { query: "自定义项目".into(), labels: HashMap::from([("two".into(), "自定义项目".into())]), ..Default::default() };
        assert_eq!(search(&store, sessions.clone(), request, || false).unwrap().hits[0].session_id, "two");
        assert!(search(&store, sessions.clone(), Request { query: "文档".into(), session_ids: Some(vec!["two".into()]), ..Default::default() }, || false).unwrap().hits.is_empty());
        assert_eq!(search(&store, sessions, Request { query: "文档".into(), field: Field::Title, ..Default::default() }, || false).unwrap().hits.len(), 1);
    }
    #[test]
    fn bounds_recent_results_and_handles_cancellation_and_unicode_snippets() {
        let sessions = (0..12).map(|i| session(&format!("s{i}"), "标题", i)).collect();
        let store = MemoryStore::new();
        let recent = search(&store, sessions, Request::default(), || false).unwrap();
        assert_eq!(recent.hits.len(), 8); assert_eq!(recent.hits[0].session_id, "s11");
        assert!(search(&store, vec![session("s", "标题", 0)], Request::default(), || true).unwrap().cancelled);
        assert!(excerpt(&format!("{}目标🙂", "甲".repeat(150)), &["目标".into()]).contains("目标🙂"));
    }
}

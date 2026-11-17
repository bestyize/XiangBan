//! Mobile host policy. Settings are session-scoped; raw history remains append-only.
use agent_kernel::{model::TokenBudget, *};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{cell::RefCell, collections::HashMap, path::{Path, PathBuf}, sync::{Arc, Mutex}};

thread_local! { pub static MODEL_SESSION: RefCell<String> = const { RefCell::new(String::new()) }; }

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct SessionSettings {
    pub model: String,
    pub protocol: String,
    pub model_source: String,
    pub context_tokens: usize,
    pub model_window: Option<usize>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub max_output_tokens: Option<u32>,
    pub max_turns: Option<usize>,
    pub permission: String,
    pub grants: Vec<String>,
}
impl Default for SessionSettings {
    fn default() -> Self {
        Self { model: String::new(), protocol: "chat".into(), model_source: String::new(), context_tokens: 256_000, model_window: None,
            supports_tools: true, supports_vision: true, max_output_tokens: None, max_turns: None,
            permission: "ask".into(), grants: Vec::new() }
    }
}
pub type Settings = Arc<Mutex<HashMap<String, SessionSettings>>>;

pub fn valid_session(session: &str) -> bool {
    !session.is_empty() && session.len() <= 128 && session.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
}

pub fn configure(settings: &Settings, permission: &Permission, trust: &Path, session: &str, next: SessionSettings) -> Result<(), String> {
    if !valid_session(session) { return Err("invalid session identity".into()); }
    if !(16_000..=1_000_000).contains(&next.context_tokens)
        || next.max_turns.is_some_and(|n| n == 0 || n > 1_000_000)
        || next.model_window == Some(0)
        || next.max_output_tokens == Some(0)
        || !matches!(next.permission.as_str(), "ask" | "auto")
        || !matches!(next.protocol.as_str(), "chat" | "responses" | "anthropic") || next.model_source.len() > 4096 {
        return Err("invalid session settings".into());
    }
    permission.restore_trust(trust, session, false)?;
    let mut saved = settings.lock().unwrap_or_else(|e| e.into_inner());
    permission.revoke_session_grants(session);
    for tool in &next.grants { permission.grant(session, tool, None); }
    permission.set_session_mode(session, if next.permission == "auto" { PermissionMode::Auto } else { PermissionMode::Ask });
    saved.insert(session.into(), next);
    Ok(())
}

pub fn danger(tool: &str, args: &Value) -> Option<String> {
    match tool {
        "delete_file" => Some("删除手机工作区内容无法自动撤销，请确认目标路径".into()),
        "app_create" if args["overwrite"] == true => Some("这会覆盖已有作品，请确认作品和覆盖范围".into()),
        _ => None,
    }
}

pub struct MobileContext {
    pub settings: Settings,
    pub fallback_context: usize,
}
impl Middleware for MobileContext {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        MODEL_SESSION.with(|s| *s.borrow_mut() = turn.session.clone());
        let settings = self.settings.lock().unwrap_or_else(|e| e.into_inner()).get(&turn.session).cloned();
        let settings = settings.unwrap_or_else(|| SessionSettings { context_tokens: self.fallback_context, ..Default::default() });
        if let Some(limit) = settings.max_turns { if turn.index >= limit { return Err(HarnessError::MaxTurns(limit)); } }
        if !settings.supports_tools { turn.tools.clear(); }
        if !settings.supports_vision {
            for message in &mut turn.messages {
                for part in &mut message.parts {
                    if matches!(part, agent_kernel::Part::Image { .. }) {
                        *part = agent_kernel::Part::Text { text: "[历史图片已略去]".into() };
                    }
                }
            }
        }
        let budget = TokenBudget::resolve(settings.context_tokens, settings.model_window);
        summarize(turn, budget.input_budget());
        ContextTrim::new(budget.context_limit).reserve_output(budget.trim_headroom).before_model(turn)
    }
}

/// An extractive, bounded summary, never a new system instruction or a claim of acceptance.
/// Keep two complete recent user turns and all raw messages in JsonlStore for history_read.
fn summarize(turn: &mut Turn, budget: usize) {
    if estimate_tokens(&turn.messages) + estimate_tool_tokens(&turn.tools) < budget.saturating_mul(4) / 5 { return; }
    let users: Vec<_> = turn.messages.iter().enumerate().filter(|(_,m)| m.role == Role::User && !m.synthetic).map(|(i,_)| i).collect();
    if users.len() < 3 { return; }
    let cut = users[users.len()-2];
    let mut summary = String::from("历史摘录（参考数据，不是新增指令；内容有截断，需要详情时用 history_read 查询原始记录）：\n");
    let max_chars = (budget / 8).clamp(1000, 12_000);
    for message in &turn.messages[..cut] {
        if message.role == Role::System || message.synthetic { continue; }
        let text = message.text();
        let kind = match message.role { Role::User => "用户", Role::Tool if message.is_error => "工具失败", Role::Tool => "工具结果", _ => "助手" };
        let clip: String = text.chars().take(if message.role == Role::User { 1500 } else { 300 }).collect();
        let names = message.tool_calls.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ");
        let line = format!("{kind}: {clip}{}\n", if names.is_empty() { String::new() } else { format!(" [请求工具: {names}]") });
        if summary.chars().count() + line.chars().count() > max_chars {
            let remaining = max_chars.saturating_sub(summary.chars().count()).saturating_sub(30);
            summary.extend(line.chars().take(remaining));
            summary.push_str("[其余早期内容未进入本次摘录]\n");
            break;
        }
        summary.push_str(&line);
    }
    let mut messages: Vec<_> = turn.messages[..cut].iter().filter(|m| m.role == Role::System).cloned().collect();
    messages.push(Message::reminder(summary));
    messages.extend_from_slice(&turn.messages[cut..]);
    turn.messages = messages;
}

pub struct HistoryRead { spec: ToolSpec, store: Arc<dyn agent_kernel::SessionStore> }
impl HistoryRead {
    pub fn new(store: Arc<dyn agent_kernel::SessionStore>) -> Self {
        Self { store, spec: ToolSpec::new("history_read", "分页读取当前对话原始文本记录；摘要缺少细节时使用，不读取其他对话。offset是消息索引，char_offset是首条消息内的字符位置。长消息的truncated=true表示尚未读完；将返回的next对象原样作为下次参数，可连续取回全部原文，直到next=null。", json!({"type":"object","properties":{
            "offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":20},
            "char_offset":{"type":"integer","minimum":0},"char_limit":{"type":"integer","minimum":1,"maximum":8000}
        },"additionalProperties":false})).risk(Risk::Read) }
    }
}
impl Tool for HistoryRead {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String,String> {
        let messages = self.store.load(ctx.session()).map_err(|e|e.to_string())?;
        let offset = usize::try_from(args["offset"].as_u64().unwrap_or(0)).map_err(|_| "offset is too large")?;
        let char_offset = usize::try_from(args["char_offset"].as_u64().unwrap_or(0)).map_err(|_| "char_offset is too large")?;
        let limit = args["limit"].as_u64().unwrap_or(10).clamp(1,20) as usize;
        let char_limit = args["char_limit"].as_u64().unwrap_or(2000).clamp(1,8000) as usize;
        let mut records = Vec::new();
        let mut next_offset = offset.min(messages.len());
        let mut next_char_offset = char_offset;
        if next_offset == messages.len() && char_offset > 0 { return Err("char_offset is past the end of history".into()); }
        let page = |records: &[Value], next_offset: usize, next_char_offset: usize| json!({
            "records":records,"total":messages.len(),"next_offset":next_offset,"next_char_offset":next_char_offset,
            "next": if next_offset < messages.len() { json!({"offset":next_offset,"char_offset":next_char_offset,"limit":limit,"char_limit":char_limit}) } else { Value::Null }
        });
        while next_offset < messages.len() && records.len() < limit {
            let message = &messages[next_offset];
            let text = message.text();
            let total_chars = text.chars().count();
            if next_char_offset > total_chars { return Err("char_offset is past the end of this message".into()); }
            let start = next_char_offset;
            let mut take = char_limit.min(total_chars - start);
            loop {
                let end = start + take;
                let truncated = end < total_chars;
                let candidate_next = if truncated { (next_offset, end) } else { (next_offset + 1, 0) };
                records.push(json!({"index":next_offset,"role":message.role,"text":text.chars().skip(start).take(take).collect::<String>(),
                    "is_error":message.is_error,"char_offset":start,"char_end":end,"total_chars":total_chars,"truncated":truncated}));
                if page(&records, candidate_next.0, candidate_next.1).to_string().len() <= 14_000 {
                    (next_offset, next_char_offset) = candidate_next;
                    break;
                }
                records.pop();
                if !records.is_empty() { return Ok(page(&records, next_offset, start).to_string()); }
                // JSON escaping (including control characters) counts against the
                // byte cap. Shorten this first record, never return a stuck cursor.
                if take <= 1 { return Err("history record metadata exceeds page size".into()); }
                take = take.div_ceil(2);
            }
            if next_char_offset > 0 { break; }
        }
        Ok(page(&records, next_offset, next_char_offset).to_string())
    }
}

pub(crate) fn session_component(session: &str) -> String {
    session.bytes().map(|b| if b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b,b'-'|b'_') { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

pub struct MobileSpill(pub PathBuf);
impl Middleware for MobileSpill {
    fn after_tool(&self, session: &str, call: &ToolCall, result: &mut ToolResult) {
        if valid_session(session) { SpillOutput::new(self.0.join(session_component(session))).after_tool(session, call, result); }
    }
}

pub struct OutputRead { spec: ToolSpec, root: PathBuf }
impl OutputRead {
    pub fn new(root: PathBuf) -> Self {
        Self { root, spec: ToolSpec::new("tool_output_read", "分页读取过长工具输出的落盘文件；path 只能来自本次对话工具返回的输出路径。offset/limit 按字符计。", json!({"type":"object","properties":{"path":{"type":"string"},"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":16000}},"required":["path"],"additionalProperties":false})).risk(Risk::Read) }
    }
}
impl Tool for OutputRead {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String,String> {
        let requested = Path::new(args["path"].as_str().ok_or("path is required")?);
        let name = requested.file_name().and_then(|n|n.to_str()).ok_or("invalid output path")?;
        // All mobile conversation ids are restricted to ASCII. SpillOutput prefixes filenames with it.
        if !valid_session(ctx.session()) { return Err("invalid conversation identity".into()); }
        let root = self.root.join(session_component(ctx.session())).canonicalize().map_err(|e|e.to_string())?;
        let path = root.join(name);
        if path.canonicalize().map_err(|e|e.to_string())? != path { return Err("invalid output path".into()); }
        let text = std::fs::read_to_string(path).map_err(|e|e.to_string())?;
        let offset = args["offset"].as_u64().unwrap_or(0) as usize;
        let limit = args["limit"].as_u64().unwrap_or(8000).clamp(1,16000) as usize;
        let mut part: String = text.chars().skip(offset).take(limit).collect();
        loop {
            let output = json!({"text":part,"next_offset":offset+part.chars().count(),"total_chars":text.chars().count()}).to_string();
            if output.len() <= 14_000 { return Ok(output); }
            let count = part.chars().count() / 2;
            part = part.chars().take(count).collect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn history_pages_recover_every_character_without_crossing_sessions() {
        let store = Arc::new(MemoryStore::new());
        let original = format!("{}{}末尾约束", "中🙂\n\"\u{0001}".repeat(3000), "x".repeat(9000));
        store.append("s1", &[Message::user_text(&original), Message::assistant("next message", vec![])]).unwrap();
        store.append("s2", &[Message::user_text("private elsewhere")]).unwrap();
        let tool = HistoryRead::new(store);
        let mut args = json!({"offset":0,"limit":20,"char_limit":8000});
        let mut restored = [String::new(), String::new()];
        let mut previous = (0, 0);
        let mut saw_truncated = false;
        for _ in 0..50 {
            let output = tool.execute(args, &ToolCtx::detached("s1")).unwrap();
            assert!(output.len() <= 14_000 && !output.contains("private elsewhere"));
            let page: Value = serde_json::from_str(&output).unwrap();
            assert_eq!(page["total"], 2);
            for record in page["records"].as_array().unwrap() {
                let index = record["index"].as_u64().unwrap() as usize;
                assert_eq!(record["char_offset"].as_u64().unwrap() as usize, restored[index].chars().count());
                restored[index].push_str(record["text"].as_str().unwrap());
                saw_truncated |= record["truncated"] == true;
            }
            let cursor = (page["next_offset"].as_u64().unwrap(), page["next_char_offset"].as_u64().unwrap());
            assert!(cursor > previous, "every nonempty page must advance");
            previous = cursor;
            if page["next"].is_null() { break; }
            args = page["next"].clone();
        }
        assert!(saw_truncated);
        assert_eq!(restored, [original, "next message".into()]);
    }

    #[test]
    fn history_cursor_handles_empty_messages_and_rejects_out_of_range_characters() {
        let store = Arc::new(MemoryStore::new());
        store.append("s", &[Message::user_text(""), Message::assistant("abc", vec![])]).unwrap();
        let tool = HistoryRead::new(store);
        let page: Value = serde_json::from_str(&tool.execute(json!({"offset":0,"limit":1}), &ToolCtx::detached("s")).unwrap()).unwrap();
        assert_eq!(page["next_offset"], 1);
        assert_eq!(page["records"][0]["truncated"], false);
        assert!(tool.execute(json!({"offset":1,"char_offset":4}), &ToolCtx::detached("s")).is_err());
        let end: Value = serde_json::from_str(&tool.execute(json!({"offset":2}), &ToolCtx::detached("s")).unwrap()).unwrap();
        assert!(end["next"].is_null());
        assert!(end["records"].as_array().unwrap().is_empty());
    }

    #[test]
    fn replacing_grants_revokes_an_always_answer_without_losing_review() {
        let root = std::env::temp_dir().join(format!("mobile-policy-grants-{}",std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let asks = Arc::new(AtomicUsize::new(0));
        let count = asks.clone();
        let permission = Permission::new(PermissionMode::Ask,move |_: &ApprovalRequest<'_>| { count.fetch_add(1,Ordering::SeqCst); Approval::Always });
        let settings = Arc::new(Mutex::new(HashMap::new()));
        configure(&settings,&permission,&root,"s1",SessionSettings::default()).unwrap();
        let spec = ToolSpec::new("write_file","write",json!({"type":"object"})).risk(Risk::Write);
        let call = ToolCall { id:"c1".into(),name:"write_file".into(),arguments:"{}".into() };
        for _ in 0..2 { permission.before_tool(&ToolCtx::detached("s1"),&call,&json!({}),&spec); }
        assert_eq!(asks.load(Ordering::SeqCst),1);
        // The old configure snapshot did not contain the newly answered Always grant.
        configure(&settings,&permission,&root,"s1",SessionSettings::default()).unwrap();
        permission.before_tool(&ToolCtx::detached("s1"),&call,&json!({}),&spec);
        assert_eq!(asks.load(Ordering::SeqCst),2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn summary_keeps_recent_turns_and_does_not_mutate_the_raw_transcript() {
        let mut raw = vec![Message::system("system")];
        for i in 0..6 { raw.push(Message::user_text(format!("约束{i}：{}","文".repeat(5000)))); raw.push(Message::assistant("记录",vec![])); }
        raw.push(Message::user_text("最近一轮")); raw.push(Message::assistant("收到",vec![]));
        raw.push(Message::user_text("当前请求"));
        let mut turn = Turn { session:"s1".into(), index:0, messages:raw.clone(), tools:vec![] };
        MobileContext { settings:Arc::new(Mutex::new(HashMap::new())),fallback_context:16_000 }.before_model(&mut turn).unwrap();
        let text = turn.messages.iter().map(Message::text).collect::<Vec<_>>().join("\n");
        assert!(text.contains("约束0") && text.contains("最近一轮") && text.contains("当前请求"));
        assert!(text.contains("history_read"));
        assert!(turn.messages.len() < raw.len());
        assert!(raw[1].text().chars().count()>5000);
    }

    #[test]
    fn spilled_outputs_are_isolated_even_for_overlapping_session_prefixes() {
        let root = std::env::temp_dir().join(format!("mobile-policy-output-{}",std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for session in ["s1","s1-child"] {
            let mut result = ToolResult::ok(format!("{session}:{}","x".repeat(20_000)));
            MobileSpill(root.clone()).after_tool(session,&ToolCall { id:"c".into(),name:"read_file".into(),arguments:"{}".into() },&mut result);
        }
        let other = std::fs::read_dir(root.join("s1-child")).unwrap().next().unwrap().unwrap().path();
        let reader = OutputRead::new(root.clone());
        assert!(reader.execute(json!({"path":other}),&ToolCtx::detached("s1")).is_err());
        let own = std::fs::read_dir(root.join("s1")).unwrap().next().unwrap().unwrap().path();
        assert!(reader.execute(json!({"path":own}),&ToolCtx::detached("s1")).unwrap().contains("s1:"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unicode_output_pages_fit_without_being_spilled_again() {
        let root = std::env::temp_dir().join(format!("mobile-policy-unicode-{}",std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let folder = root.join("s1"); std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("s1-output.txt");
        std::fs::write(&path,"中文\n\"".repeat(10_000)).unwrap();
        let reader = OutputRead::new(root.clone());
        let output = reader.execute(json!({"path":path}),&ToolCtx::detached("s1")).unwrap();
        assert!(output.len() <= 14_000);
        let data: Value = serde_json::from_str(&output).unwrap();
        assert!(data["next_offset"].as_u64().unwrap()>0);
        assert!(data["text"].as_str().unwrap().starts_with("中文"));
        let _ = std::fs::remove_dir_all(root);
    }
}

//! 手机互联的线上消息整形（纯函数，不依赖 Tauri，便于单测）：
//! - 互联协议单个信封最多 256 KiB 密文，超了 `Link::send` 会返回 `TooLarge`，所以发出前必须把内容压进预算；
//! - 给手机的会话快照是“精简投影”：只含角色与文本，不含 `provider_state`、思考过程、图片地址等桌面内部数据；
//! - 快照按字节预算分页，`has_more` 为真时手机用 `session.open` + `after` 取下一页。

use serde_json::{json, Value};

/// 单条明文消息的字节上限（协议上限 256 KiB，留出填充与 tag 的余量）。
pub const MAX_WIRE_BYTES: usize = 240 * 1024;
/// 一页快照里消息内容的字节预算。
pub const SNAPSHOT_PAGE_BYTES: usize = 120 * 1024;
/// 快照里单条消息文本的字符上限。
pub const MAX_MESSAGE_CHARS: usize = 20_000;
/// 事件里工具参数 / 输出的字符上限。
pub const MAX_EVENT_FIELD_CHARS: usize = 8_000;
/// 快照里一次工具调用的参数、一条工具结果的字符上限（历史里的卡片只要能看清做了什么，不用全文）。
pub const MAX_SNAPSHOT_TOOL_CHARS: usize = 3_000;
/// 快照里一条消息最多带几段思考，每段的字符上限。
pub const MAX_SNAPSHOT_REASONING: usize = 6;
pub const MAX_SNAPSHOT_REASONING_CHARS: usize = 2_000;

/// 手机发来的消息写进发送队列时用的幂等编号：同一台手机的同一个 `client_id` 永远得到同一个编号。
/// 队列只认字母、数字、`-`、`_`，且最长 128 个字符，所以其余字符（比如手机编号里没有、但 `client_id` 可能带的冒号）换成 `_`。
pub fn remote_request_id(peer_id: &str, client_id: &str) -> String {
    fn clean(raw: &str, max: usize) -> String {
        raw.chars().take(max).map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
    }
    format!("remote-{}-{}", clean(peer_id, 40), clean(client_id, 80))
}

/// 文件夹路径的最后一段（项目名）。同时认 `/` 和 `\\`，因为路径来自 Windows 或 macOS 的电脑，而测试可能跑在另一种系统上。
pub fn folder_name(path: &str) -> String {
    let trimmed = path.trim_end_matches(['/', '\\']);
    let last = trimmed.rsplit(['/', '\\']).next().unwrap_or("");
    if last.is_empty() { path.to_string() } else { last.to_string() }
}

/// 手机指定的项目文件夹是否在电脑已知的文件夹里（最近用过的、已有对话用的）。只比较去掉末尾分隔符后的原样文本，
/// 所以手机不能借此把对话指到电脑上任意一个路径。
pub fn workspace_allowed(candidate: &str, known: &[String]) -> bool {
    let norm = |p: &str| p.trim().trim_end_matches(['/', '\\']).to_string();
    let wanted = norm(candidate);
    !wanted.is_empty() && known.iter().any(|k| norm(k) == wanted)
}

/// 按字符数截断；返回（文本，是否被截断）。
pub fn truncate_chars(text: &str, max: usize) -> (String, bool) {
    match text.char_indices().nth(max) {
        Some((end, _)) => (text[..end].to_string(), true),
        None => (text.to_string(), false),
    }
}

/// 把任意 JSON 值压成不超过 `max` 字符的形式：字符串直接截断，其他类型过长时改成截断后的 JSON 文本。
pub fn clip_value(value: &Value, max: usize) -> Value {
    match value {
        Value::String(s) => {
            let (text, cut) = truncate_chars(s, max);
            if cut { Value::String(format!("{text}…（已截断）")) } else { value.clone() }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
        other => {
            let raw = other.to_string();
            if raw.chars().count() <= max {
                other.clone()
            } else {
                let (text, _) = truncate_chars(&raw, max);
                Value::String(format!("{text}…（已截断）"))
            }
        }
    }
}

/// 审批卡片的 `detail` 必须是对象：过长或不是对象时包成 `{"text": ...}`。
pub fn clip_detail(value: &Value, max: usize) -> Value {
    match value {
        Value::Null => json!({}),
        Value::Object(_) => match clip_value(value, max) {
            Value::String(text) => json!({ "text": text }),
            small => small,
        },
        other => json!({ "text": clip_value(other, max) }),
    }
}

/// 桌面消息（`agent_kernel::Message` 的 JSON）→ 手机用的精简消息。合成消息（桌面自己加的提示）返回 `None`。
pub fn slim_message(message: &Value) -> Option<Value> {
    if message.get("synthetic").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let role = message.get("role").and_then(Value::as_str).unwrap_or("assistant");
    // 系统提示词是桌面内部指令，不给手机。
    if role == "system" {
        return None;
    }
    let mut text = String::new();
    let mut images = 0usize;
    if let Some(parts) = message.get("parts").and_then(Value::as_array) {
        for part in parts {
            match part.get("type").and_then(Value::as_str) {
                Some("text") => text.push_str(part.get("text").and_then(Value::as_str).unwrap_or("")),
                Some("image") => images += 1,
                _ => {}
            }
        }
    }
    // 工具结果只是给卡片看的，截得更短；其余消息按正文上限。
    let limit = if role == "tool" { MAX_SNAPSHOT_TOOL_CHARS } else { MAX_MESSAGE_CHARS };
    let (text, truncated) = truncate_chars(&text, limit);
    let mut out = json!({
        "role": role,
        "parts": [{ "type": "text", "text": text }],
    });
    if truncated {
        out["truncated"] = json!(true);
    }
    if images > 0 {
        out["images"] = json!(images);
    }
    if message.get("is_error").and_then(Value::as_bool) == Some(true) {
        out["is_error"] = json!(true);
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        let items: Vec<Value> = calls
            .iter()
            .map(|c| json!({ "id": c["id"], "name": c["name"], "arguments": truncate_chars(c["arguments"].as_str().unwrap_or(""), MAX_SNAPSHOT_TOOL_CHARS).0 }))
            .collect();
        if !items.is_empty() {
            out["tool_calls"] = Value::Array(items);
        }
    }
    // 工具结果要对得上是哪次调用，手机才能把结果放回卡片里。
    if let Some(id) = message.get("tool_call_id").and_then(Value::as_str) {
        out["tool_call_id"] = json!(id);
    }
    // 保存下来的思考过程（模型明确给出可显示文字的那部分），按段截短。
    if let Some(segments) = message.get("reasoning").and_then(Value::as_array).filter(|_| role == "assistant") {
        let texts: Vec<Value> = segments
            .iter()
            .filter_map(|r| r.get("text").and_then(Value::as_str))
            .filter(|t| !t.trim().is_empty())
            .take(MAX_SNAPSHOT_REASONING)
            .map(|t| json!(truncate_chars(t, MAX_SNAPSHOT_REASONING_CHARS).0))
            .collect();
        if !texts.is_empty() {
            out["reasoning"] = Value::Array(texts);
        }
    }
    Some(out)
}

pub struct SnapshotPage {
    pub messages: Vec<Value>,
    /// 下一页从第几条（原始下标）开始取。
    pub cursor: usize,
    pub has_more: bool,
}

/// 从原始下标 `after` 起取一页，序列化后的消息总量不超过 `budget` 字节（至少取一条，保证能前进）。
pub fn snapshot_page(all: &[Value], after: usize, budget: usize) -> SnapshotPage {
    let start = after.min(all.len());
    let mut messages = Vec::new();
    let mut used = 0usize;
    let mut cursor = start;
    for message in &all[start..] {
        let slim = slim_message(message);
        let size = slim.as_ref().map_or(0, |m| m.to_string().len());
        if !messages.is_empty() && used + size > budget {
            break;
        }
        if let Some(m) = slim {
            used += size;
            messages.push(m);
        }
        cursor += 1;
    }
    SnapshotPage { messages, cursor, has_more: cursor < all.len() }
}

/// 发送前的最后一道关：序列化后超过上限的事件，换成一条简短的错误事件；非事件消息返回 `None`（调用方自行处理）。
pub fn fit_for_wire(msg: &Value) -> Result<Vec<u8>, Option<Value>> {
    let bytes = serde_json::to_vec(msg).map_err(|_| None)?;
    if bytes.len() <= MAX_WIRE_BYTES {
        return Ok(bytes);
    }
    if msg.get("t").and_then(Value::as_str) == Some("event") {
        return Err(Some(json!({
            "t": "event",
            "id": msg["id"],
            "session": msg["session"],
            "kind": "error",
            "data": { "message": "这条内容太大，手机上不显示，请在电脑上查看" },
        })));
    }
    Err(None)
}

/// 给没有订阅该会话的手机的事件脱敏：只保留“有事发生”的最小信息。
pub fn redact_for_unsubscribed(kind: &str, data: &Value) -> Value {
    match kind {
        "error" => json!({ "message": "对话出错了，请打开该会话查看" }),
        _ => {
            let _ = data;
            json!({})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, text: &str) -> Value {
        json!({ "role": role, "parts": [{ "type": "text", "text": text }], "provider_state": { "secret": "x" } })
    }

    #[test]
    fn remote_request_ids_are_accepted_by_the_outbox_rules() {
        let ok = |id: &str| !id.is_empty() && id.len() <= 128 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
        for (peer, client) in [("phn_a1b2c3", "c-1730000000000-1"), ("phn_x", "带:冒号 和空格/斜杠"), ("p", &"x".repeat(500))] {
            let id = remote_request_id(peer, client);
            assert!(ok(&id), "{id}");
        }
        assert_eq!(remote_request_id("phn_a", "c-1"), remote_request_id("phn_a", "c-1"));
        assert_ne!(remote_request_id("phn_a", "c-1"), remote_request_id("phn_b", "c-1"));
        assert_ne!(remote_request_id("phn_a", "c-1"), remote_request_id("phn_a", "c-2"));
    }

    #[test]
    fn slim_message_drops_internal_fields_and_images() {
        let m = json!({
            "role": "user",
            "parts": [{ "type": "text", "text": "看图" }, { "type": "image", "url": "data:image/png;base64,AAAA" }],
            "provider_state": { "opaque": 1 },
            "reasoning": [{ "kind": "text", "text": "思考" }],
        });
        let s = slim_message(&m).unwrap();
        assert_eq!(s["parts"][0]["text"], "看图");
        assert_eq!(s["images"], 1);
        let raw = s.to_string();
        assert!(!raw.contains("provider_state") && !raw.contains("data:image") && !raw.contains("思考"));
        assert!(slim_message(&json!({ "role": "user", "synthetic": true, "parts": [] })).is_none());
        assert!(slim_message(&msg("system", "你是小伴")).is_none());
    }

    #[test]
    fn tool_calls_keep_their_arguments_and_results_point_back_at_the_call() {
        let call = json!({
            "role": "assistant", "parts": [],
            "tool_calls": [{ "id": "c1", "name": "shell", "arguments": "{\"command\":\"npm test\"}" }],
            "reasoning": [{ "kind": "summary", "text": "先跑测试" }, { "kind": "summary", "text": "  " }]
        });
        let slim = slim_message(&call).unwrap();
        assert_eq!(slim["tool_calls"][0]["arguments"], "{\"command\":\"npm test\"}");
        assert_eq!(slim["reasoning"], json!(["先跑测试"]));
        let long = "x".repeat(MAX_SNAPSHOT_TOOL_CHARS + 10);
        let result = json!({ "role": "tool", "tool_call_id": "c1", "parts": [{ "type": "text", "text": long }], "is_error": true });
        let slim = slim_message(&result).unwrap();
        assert_eq!(slim["tool_call_id"], "c1");
        assert_eq!(slim["is_error"], true);
        assert_eq!(slim["truncated"], true);
        assert_eq!(slim["parts"][0]["text"].as_str().unwrap().chars().count(), MAX_SNAPSHOT_TOOL_CHARS);
    }

    #[test]
    fn long_text_is_truncated_on_char_boundary() {
        let long = "字".repeat(MAX_MESSAGE_CHARS + 5);
        let s = slim_message(&msg("assistant", &long)).unwrap();
        assert_eq!(s["truncated"], true);
        assert_eq!(s["parts"][0]["text"].as_str().unwrap().chars().count(), MAX_MESSAGE_CHARS);
    }

    #[test]
    fn snapshot_pages_cover_every_message_exactly_once_and_stay_in_budget() {
        let all: Vec<Value> = (0..200).map(|i| msg("assistant", &format!("{i}-{}", "x".repeat(4000)))).collect();
        let mut after = 0;
        let mut seen = Vec::new();
        let mut pages = 0;
        loop {
            let page = snapshot_page(&all, after, SNAPSHOT_PAGE_BYTES);
            assert!(!page.messages.is_empty() || !page.has_more);
            let size: usize = page.messages.iter().map(|m| m.to_string().len()).sum();
            assert!(size <= SNAPSHOT_PAGE_BYTES);
            for m in &page.messages {
                seen.push(m["parts"][0]["text"].as_str().unwrap().split('-').next().unwrap().parse::<usize>().unwrap());
            }
            pages += 1;
            assert!(page.cursor > after, "必须前进");
            after = page.cursor;
            if !page.has_more {
                break;
            }
        }
        assert!(pages > 1);
        assert_eq!(seen, (0..200).collect::<Vec<_>>());
    }

    #[test]
    fn single_oversized_message_still_advances() {
        let all = vec![msg("assistant", &"字".repeat(MAX_MESSAGE_CHARS * 2)), msg("user", "后一条")];
        let page = snapshot_page(&all, 0, 10);
        assert_eq!(page.messages.len(), 1);
        assert_eq!(page.cursor, 1);
        assert!(page.has_more);
    }

    #[test]
    fn synthetic_messages_are_skipped_but_counted_in_cursor() {
        let all = vec![json!({ "role": "user", "synthetic": true, "parts": [] }), msg("user", "你好")];
        let page = snapshot_page(&all, 0, SNAPSHOT_PAGE_BYTES);
        assert_eq!(page.messages.len(), 1);
        assert_eq!(page.cursor, 2);
        assert!(!page.has_more);
        assert_eq!(snapshot_page(&all, 99, 10).cursor, 2);
    }

    #[test]
    fn oversized_events_become_a_short_error_event() {
        let big = json!({ "t": "event", "id": "d-1", "session": "s_1", "kind": "tool.end", "data": { "output": "x".repeat(MAX_WIRE_BYTES) } });
        match fit_for_wire(&big) {
            Err(Some(e)) => {
                assert_eq!(e["kind"], "error");
                assert_eq!(e["session"], "s_1");
                assert!(serde_json::to_vec(&e).unwrap().len() < 1024);
            }
            _ => panic!("应替换为错误事件"),
        }
        assert!(matches!(fit_for_wire(&json!({ "t": "sessions.page", "pad": "x".repeat(MAX_WIRE_BYTES) })), Err(None)));
        assert!(fit_for_wire(&json!({ "t": "hello" })).is_ok());
    }

    #[test]
    fn clip_value_limits_strings_and_structures() {
        let s = clip_value(&json!("字".repeat(100)), 10);
        assert!(s.as_str().unwrap().starts_with(&"字".repeat(10)));
        assert_eq!(clip_value(&json!({ "a": 1 }), 100), json!({ "a": 1 }));
        let big = clip_value(&json!({ "content": "x".repeat(1000) }), 50);
        assert!(big.as_str().unwrap().chars().count() < 80);
        assert_eq!(clip_value(&json!(7), 1), json!(7));
    }

    #[test]
    fn clip_detail_always_returns_an_object() {
        assert_eq!(clip_detail(&Value::Null, 10), json!({}));
        assert_eq!(clip_detail(&json!({ "cmd": "ls" }), 100), json!({ "cmd": "ls" }));
        assert!(clip_detail(&json!({ "cmd": "x".repeat(500) }), 50)["text"].is_string());
        assert_eq!(clip_detail(&json!("rm -rf"), 50), json!({ "text": "rm -rf" }));
    }

    #[test]
    fn folder_name_takes_the_last_segment_of_either_style_of_path() {
        assert_eq!(folder_name("D:\\work\\xiaoban-web"), "xiaoban-web");
        assert_eq!(folder_name("/Users/me/docs/"), "docs");
        assert_eq!(folder_name("C:\\"), "C:");
        assert_eq!(folder_name("/"), "/");
        assert_eq!(folder_name("项目"), "项目");
    }

    #[test]
    fn a_phone_can_only_pick_folders_the_computer_already_knows() {
        let known = vec!["D:\\work\\xiaoban-web".to_string(), "/Users/me/docs".to_string()];
        assert!(workspace_allowed("D:\\work\\xiaoban-web", &known));
        assert!(workspace_allowed("/Users/me/docs/", &known));
        assert!(!workspace_allowed("D:\\work", &known));
        assert!(!workspace_allowed("/Users/me/docs/../.ssh", &known));
        assert!(!workspace_allowed("", &known));
        assert!(!workspace_allowed("/", &known));
    }

    #[test]
    fn unsubscribed_peers_get_no_payload() {
        assert_eq!(redact_for_unsubscribed("turn.done", &json!({ "x": 1 })), json!({}));
        let e = redact_for_unsubscribed("error", &json!({ "message": "key sk-abc 无效" }));
        assert!(!e.to_string().contains("sk-abc"));
    }
}

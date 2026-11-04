//! 输入框里 `@` 点名的东西（WorkBuddy 的做法）：不往消息文本里塞前置提示词，点名随这条消息作为结构化字段传来，
//! 请求前按它们在这条用户消息后面加说明，只加进这一次请求，不写进对话记录。
//! 仅注入本次选中的连接器状态，完整列表由 connector_status 按需提供。
use agent_kernel::{HarnessError, Message, Middleware, Role, Turn};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

/// 一条消息最多带这么多个点名。
pub const MAX_MENTIONS: usize = 8;

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct Mention {
    /// `skill` / `connector` / `expert`。专家在界面上召唤成新对话，这里不处理。
    pub kind: String,
    pub id: String,
}

/// 点名的技能名，去掉不合法的。
pub fn skill_names(mentions: &[Mention]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for m in mentions.iter().filter(|m| m.kind == "skill") {
        let ok = !m.id.is_empty() && m.id.len() <= 64 && m.id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if ok && !names.contains(&m.id) {
            names.push(m.id.clone());
        }
    }
    names
}

pub struct ConnectorContext {
    statuses: Arc<dyn Fn() -> Value + Send + Sync>,
    focus: Vec<String>,
}

impl ConnectorContext {
    /// `statuses` is `ConnectorManager::statuses`; `mentions` are the ones this message carries.
    pub fn new(statuses: Arc<dyn Fn() -> Value + Send + Sync>, mentions: &[Mention]) -> Self {
        let focus = mentions.iter().filter(|m| m.kind == "connector").map(|m| m.id.clone()).take(MAX_MENTIONS).collect();
        Self { statuses, focus }
    }
}

fn state_of(status: &Value) -> String {
    if status["enabled"] != true {
        return "已停用".into();
    }
    if status["connected"] == true {
        return format!("已连接，{} 个工具", status["toolCount"].as_u64().unwrap_or(0));
    }
    match status["error"].as_str().filter(|e| !e.is_empty()) {
        Some(error) => format!("未连接（{}）", error.chars().take(80).collect::<String>()),
        None => "未连接".into(),
    }
}

/// Report only the connectors selected for this request.
pub fn context_text(statuses: &Value, focus: &[String]) -> Option<String> {
    if focus.is_empty() { return None; }
    let named: Vec<String> = focus.iter().map(|id| {
        match statuses.as_array().and_then(|list| list.iter().find(|s| s["id"].as_str() == Some(id))) {
            Some(s) => format!("「{}」（{}）", s["name"].as_str().unwrap_or(id), state_of(s)),
            None => format!("「{id}」（没有配置这个连接器）"),
        }
    }).collect();
    Some(format!(
        "<connector-status>\n用户在这条消息里点名要使用连接器：{}。优先使用这些连接器，必要时按名称使用 tool_search。\n\
         可用状态与本轮实际调用分别判断。明确认证失效时，引导用户在「AI 插件 → 连接器」重新授权。点名的连接器不可用时先说明障碍，不绕过用户指定的连接方式。停用、网络错误、超时或业务权限不足按实际原因说明。\n</connector-status>",
        named.join("、")
    ))
}

impl Middleware for ConnectorContext {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let Some(at) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()) };
        let selected = turn.messages[at].structured_content.as_ref().and_then(|v| v.get("mentions"))
            .and_then(|v| serde_json::from_value::<Vec<Mention>>(v.clone()).ok());
        let focus: Vec<String> = selected.as_ref().map(|mentions| mentions.iter()
            .filter(|m| m.kind == "connector").take(MAX_MENTIONS).map(|m| m.id.clone()).collect())
            .unwrap_or_else(|| self.focus.clone());
        if focus.is_empty() { return Ok(()); }
        if let Some(text) = context_text(&(self.statuses)(), &focus) {
            turn.messages.insert(at + 1, Message::reminder(text));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mention(kind: &str, id: &str) -> Mention {
        Mention { kind: kind.into(), id: id.into() }
    }

    #[test]
    fn steered_connector_selections_reach_the_model_and_override_the_initial_focus() {
        use super::super::outbox::{Delivery, Outbox};
        let root = std::env::temp_dir().join(format!("xiaoban-steer-mentions-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut queue = Outbox::new(root.clone());
        queue.open("s");
        let id = queue.enqueue("s", Message::user_text("把结果放到这里"), Delivery::Steer,
            json!({}), json!([mention("connector", "new-service")]), None).unwrap();
        let message = queue.claim("s", true, false).unwrap().unwrap().into_message();
        let context = ConnectorContext::new(Arc::new(|| json!([])), &[mention("connector", "old-service")]);
        let mut turn = Turn { session: "s".into(), index: 1, messages: vec![message], tools: vec![] };
        context.before_model(&mut turn).unwrap();
        assert!(turn.messages[1].text().contains("new-service"));
        assert!(!turn.messages[1].text().contains("old-service"));
        assert_eq!(turn.messages[0].structured_content.as_ref().unwrap()["outboxId"], id);
        queue.accepted("s", &[id]).unwrap();
        assert!(queue.snapshot("s").unwrap().items.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_explicit_empty_selection_does_not_reuse_an_older_messages_connector() {
        let context = ConnectorContext::new(Arc::new(|| json!([])), &[mention("connector", "old-service")]);
        let mut message = Message::user_text("新的补充要求");
        message.structured_content = Some(json!({"mentions":[]}));
        let mut turn = Turn { session: "s".into(), index: 1, messages: vec![message], tools: vec![] };
        context.before_model(&mut turn).unwrap();
        assert_eq!(turn.messages.len(), 1);
    }

    #[test]
    fn only_well_formed_skill_mentions_count_and_repeats_collapse() {
        let all = [mention("skill", "pptx"), mention("connector", "feishu"), mention("skill", "Bad Name"), mention("skill", "pptx"), mention("skill", ""), mention("skill", "x-1")];
        assert_eq!(skill_names(&all), ["pptx", "x-1"]);
    }

    #[test]
    fn status_is_only_injected_for_selected_connectors() {
        let statuses = json!([
            {"id": "feishu", "name": "飞书", "enabled": true, "connected": true, "toolCount": 7},
            {"id": "notion", "name": "Notion", "enabled": true, "connected": false, "error": "令牌已过期"},
            {"id": "old", "name": "旧的", "enabled": false, "connected": false}
        ]);
        assert_eq!(context_text(&statuses, &[]), None);
        let focused = context_text(&statuses, &["notion".into(), "ghost".into()]).unwrap();
        assert!(focused.contains("点名要使用连接器：「Notion」（未连接（令牌已过期））、「ghost」（没有配置这个连接器）"), "{focused}");
        assert!(!focused.contains("飞书") && !focused.contains("旧的"), "{focused}");
        assert!(context_text(&json!([]), &["feishu".into()]).unwrap().contains("没有配置这个连接器"));
    }

    #[test]
    fn the_note_lands_right_after_the_users_own_message_and_never_in_the_history() {
        let statuses: Arc<dyn Fn() -> Value + Send + Sync> = Arc::new(|| json!([{"id": "feishu", "name": "飞书", "enabled": true, "connected": true, "toolCount": 2}]));
        let middleware = ConnectorContext::new(statuses, &[mention("connector", "feishu")]);
        let mut turn = Turn { session: "s".into(), index: 0, messages: vec![Message::system("sys"), Message::user_text("旧"), Message::assistant("好", vec![]), Message::user_text("把周报发飞书")], tools: vec![] };
        middleware.before_model(&mut turn).unwrap();
        assert_eq!(turn.messages.len(), 5);
        assert!(turn.messages[4].synthetic && turn.messages[4].text().contains("<connector-status>"));
        assert_eq!(turn.messages[3].text(), "把周报发飞书");
    }
}

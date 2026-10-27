//! 输入框里 `@` 点名的东西（WorkBuddy 的做法）：不往消息文本里塞前置提示词，点名随这条消息作为结构化字段传来，
//! 请求前按它们在这条用户消息后面加说明，只加进这一次请求，不写进对话记录。
//! 同时，每次请求都带一份连接器状态（WorkBuddy 的 `<connector-status>`），让模型知道哪些连接器已经连接。
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

/// 状态块和点名说明。没有配置任何连接器、也没有点名时返回 `None`。
pub fn context_text(statuses: &Value, focus: &[String]) -> Option<String> {
    let list = statuses.as_array().filter(|a| !a.is_empty());
    let mut text = String::new();
    if let Some(list) = list {
        text.push_str("<connector-status>\n");
        for s in list {
            text.push_str(&format!("- {}（{}）：{}\n", s["name"].as_str().unwrap_or("连接器"), s["id"].as_str().unwrap_or(""), state_of(s)));
        }
        text.push_str("连接器认证失效或需要重新连接时，告诉用户到「AI 插件 → 连接器」里重新授权或重新连接；已停用、网络错误、超时、业务权限不足都不是认证失效，不要因此建议重新授权。\n</connector-status>");
    }
    let named: Vec<String> = focus
        .iter()
        .map(|id| {
            let found = list.and_then(|l| l.iter().find(|s| s["id"].as_str() == Some(id.as_str())));
            match found {
                Some(s) => format!("「{}」（{}）", s["name"].as_str().unwrap_or(id), state_of(s)),
                None => format!("「{id}」（没有配置这个连接器）"),
            }
        })
        .collect();
    if !named.is_empty() {
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(&format!(
            "用户在这条消息里点名要使用连接器：{}。请优先用它们提供的工具完成请求（工具清单里没有时用 tool_search，以连接器名称为关键词查找）；\
             状态不是已连接的，先告诉用户去连接，不要用别的办法绕开。",
            named.join("、")
        ));
    }
    (!text.is_empty()).then_some(text)
}

impl Middleware for ConnectorContext {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let Some(at) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()) };
        if let Some(text) = context_text(&(self.statuses)(), &self.focus) {
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
    fn only_well_formed_skill_mentions_count_and_repeats_collapse() {
        let all = [mention("skill", "pptx"), mention("connector", "feishu"), mention("skill", "Bad Name"), mention("skill", "pptx"), mention("skill", ""), mention("skill", "x-1")];
        assert_eq!(skill_names(&all), ["pptx", "x-1"]);
    }

    #[test]
    fn the_status_block_says_who_is_connected_and_a_mention_names_the_ones_to_use() {
        let statuses = json!([
            {"id": "feishu", "name": "飞书", "enabled": true, "connected": true, "toolCount": 7},
            {"id": "notion", "name": "Notion", "enabled": true, "connected": false, "error": "令牌已过期"},
            {"id": "old", "name": "旧的", "enabled": false, "connected": false}
        ]);
        let text = context_text(&statuses, &[]).unwrap();
        assert!(text.contains("- 飞书（feishu）：已连接，7 个工具") && text.contains("- Notion（notion）：未连接（令牌已过期）") && text.contains("- 旧的（old）：已停用"), "{text}");
        assert!(!text.contains("点名"));
        let focused = context_text(&statuses, &["notion".into(), "ghost".into()]).unwrap();
        assert!(focused.contains("点名要使用连接器：「Notion」（未连接（令牌已过期））、「ghost」（没有配置这个连接器）"), "{focused}");
        assert_eq!(context_text(&json!([]), &[]), None);
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

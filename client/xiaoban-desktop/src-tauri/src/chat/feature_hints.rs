//! Before each model call, puts the features the user's latest request is about in front of the model (see
//! `app_guide::hint_for`). Whether the model remembers to look up its own abilities is not left to chance: the
//! matching is plain code with a test table, and the hint is added to this request only, never to the stored chat.
use agent_kernel::{HarnessError, Message, Middleware, Role, Turn};

use crate::community_media::{BILIBILI_HTTP_567, DOWNLOAD_TOOL};

pub struct FeatureHints;

impl Middleware for FeatureHints {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        // Right after the user's own message, so a tool call keeps its result next to it.
        let Some(at) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()) };
        if let Some(hint) = super::app_guide::hint_for(&turn.messages[at].text()) {
            turn.messages.insert(at + 1, Message::reminder(hint));
        }
        Ok(())
    }
}

/// Keep the reminder next to the failing tool batch without splitting a call/result pair.
pub struct RecoveryHint;

impl Middleware for RecoveryHint {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        if !turn.tools.iter().any(|tool| tool.name == "web_search") { return Ok(()); }
        let Some(user) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()); };
        let current = &turn.messages[user + 1..];
        let trailing_results = current.iter().rev().take_while(|m| m.role == Role::Tool);
        let calls = current.iter().flat_map(|m| &m.tool_calls);
        if trailing_results.filter(|m| m.is_error).any(|result| {
            result.structured_content.as_ref().is_some_and(|meta| meta["recoverable_error"] == BILIBILI_HTTP_567)
                && calls.clone().any(|call| call.name == DOWNLOAD_TOOL && Some(call.id.as_str()) == result.tool_call_id.as_deref())
        }) {
            turn.messages.push(Message::reminder(
                "【桌面故障排查】B 站下载工具在媒体流阶段返回 HTTP 567。若用户允许继续联网排查，请用 yt-dlp、Bilibili、HTTP 567 检索上游问题记录，核对条件后只改一个参数验证一次。用公开关键词，不把用户链接、凭据或完整日志发给搜索服务。按工具结果确认成品，不凭错误码判断登录要求。",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(messages: Vec<Message>) -> Turn { Turn { session: "s".into(), index: 0, messages, tools: vec![] } }

    #[test]
    fn a_matching_request_gets_the_hint_right_after_it_and_others_are_untouched() {
        let mut matching = turn(vec![Message::system("sys"), Message::user_text("旧问题"), Message::assistant("旧回答", vec![]), Message::user_text("9点提醒我买花")]);
        FeatureHints.before_model(&mut matching).unwrap();
        assert_eq!(matching.messages.len(), 5);
        assert!(matching.messages[4].synthetic && matching.messages[4].text().contains("care_set"));
        assert_eq!(matching.messages[3].text(), "9点提醒我买花");

        let mut plain = turn(vec![Message::system("sys"), Message::user_text("帮我写一份周报")]);
        FeatureHints.before_model(&mut plain).unwrap();
        assert_eq!(plain.messages.len(), 2);

        // Later steps of the same answer keep the hint beside the request, ahead of the tool calls.
        let mut stepped = turn(vec![Message::user_text("切换成深色模式"), Message::assistant("", vec![])]);
        FeatureHints.before_model(&mut stepped).unwrap();
        assert!(stepped.messages[1].synthetic && !stepped.messages[2].synthetic);
    }
}

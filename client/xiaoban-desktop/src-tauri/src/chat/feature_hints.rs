//! Before each model call, puts the features the user's latest request is about in front of the model (see
//! `app_guide::hint_for`). Whether the model remembers to look up its own abilities is not left to chance: the
//! matching is plain code with a test table, and the hint is added to this request only, never to the stored chat.
use agent_kernel::{HarnessError, Message, Middleware, Role, Turn};
use std::sync::Arc;

use crate::community_media::{BILIBILI_HTTP_567, DOWNLOAD_TOOL};

/// Re-read native state for every model request, including later steps of one answer.
/// A transient reminder keeps credentials and stale login snapshots out of chat history.
pub struct AccountContext {
    signed_in: Arc<dyn Fn() -> Option<bool> + Send + Sync>,
}

impl AccountContext {
    pub fn new(signed_in: impl Fn() -> Option<bool> + Send + Sync + 'static) -> Self {
        Self { signed_in: Arc::new(signed_in) }
    }
}

impl Middleware for AccountContext {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let Some(at) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()) };
        let Some(signed_in) = (self.signed_in)() else { return Ok(()) };
        let mut hint = if signed_in {
            "【小伴账号当前状态】已登录。这是本次请求前读取的本机登录状态，历史的未登录报错不代表当前状态；额度、服务能力和令牌有效性仍以实际工具结果为准。用户要求继续或告知已登录时，继续原任务；若之前生图明确因未登录而尚未开始，现在应重新调用 image_generate，不要只复述旧报错。若之前已生成、已提交但结果未知、超时或取消，不得因登录变化自动重新生成，以免重复计费。"
        } else {
            "【小伴账号当前状态】未登录。这是本次请求前读取的本机登录状态。使用小伴 AI 生图需要先登录；历史已登录状态不代表当前状态。"
        }.to_string();
        if turn.messages[at].text().trim() == "请接着上面的内容继续写完。" {
            hint.push_str("用户点击了“继续”，意思是继续完成前面的原始任务，包括未完成的工具操作，并非仅续写文字；先检查当前状态，不能重放结果未知的操作。");
        }
        turn.messages.insert(at + 1, Message::reminder(hint));
        Ok(())
    }
}

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
    fn account_context_tracks_login_and_logout_without_changing_history_or_tool_results() {
        use std::sync::atomic::{AtomicU8, Ordering};
        let state = Arc::new(AtomicU8::new(0));
        let observed = state.clone();
        let context = AccountContext::new(move || match observed.load(Ordering::SeqCst) {
            1 => Some(false), 2 => Some(true), _ => None,
        });
        let history = vec![
            Message::user_text("帮我画一张秋风明月图"),
            Message::assistant("需要登录", vec![]),
            Message::user_text("请接着上面的内容继续写完。"),
            Message::tool("image", "生图结果未知，上游可能已生成并计费，请勿自动重试"),
        ];
        let mut unknown_host = turn(history.clone());
        context.before_model(&mut unknown_host).unwrap();
        assert_eq!(unknown_host.messages.len(), history.len());
        for (value, status) in [(1, "未登录"), (2, "已登录"), (1, "未登录")] {
            state.store(value, Ordering::SeqCst);
            let mut request = turn(history.clone());
            context.before_model(&mut request).unwrap();
            let hint = &request.messages[3];
            assert!(hint.synthetic && hint.text().contains(&format!("当前状态】{status}")));
            assert!(hint.text().contains("继续完成前面的原始任务"));
            if value == 2 {
                assert!(hint.text().contains("重新调用 image_generate"));
                assert!(hint.text().contains("不得因登录变化自动重新生成"));
            }
            assert_eq!(request.messages[4].text(), history[3].text(), "keep unknown outcomes intact");
            assert_eq!(history.len(), 4, "the account snapshot belongs only to this model request");
        }
    }

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

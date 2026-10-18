use super::{Middleware, Nudge, Stop, StopDecision};
use crate::types::{FinishReason, Message, Role};
use serde_json::Value;

const LENGTH: &str = "Your previous reply was cut off by the output length limit. Continue exactly where it stopped. \
                      Do not repeat or summarize what you already wrote.";
const EMPTY: &str = "Your previous reply was empty. If the task is not finished, continue it now with tool calls; \
                     if it is finished, reply to the user with the result.";
const ANNOUNCED: &str = "You said what you would do next but did not call any tool, so nothing was done. \
                         Do it now: put the tool call in this reply. If you meant to stop and hand back to the user, \
                         say so in one sentence.";

/// Phrases that, at the end of a reply, announce work the model is about to do itself.
const ANNOUNCING: &[&str] = &[
    "让我", "我先", "我现在就", "我这就", "接下来我", "我马上", "现在开始", "下面开始", "马上开始",
    "let me", "i'll now", "now i'll", "i'm going to", "i am going to", "next, i'll", "next i'll",
];
/// Common sign-offs that contain an announcing phrase but hand the turn back to the user.
const HANDING_BACK: &[&str] = &["让我知道", "告诉我", "let me know"];

/// Successful new tool calls restore the idle budget; `max_turns` still bounds the entire run.
pub struct ContinueGuard {
    max_per_run: usize,
    max_without_progress: usize,
    todo_tool: String,
}

impl ContinueGuard {
    pub fn new() -> Self {
        Self { max_per_run: usize::MAX, max_without_progress: 3, todo_tool: "todo".to_string() }
    }

    /// Most nudges in one run. Zero turns the guard off.
    pub fn max_per_run(mut self, nudges: usize) -> Self {
        self.max_per_run = nudges;
        self
    }

    pub fn max_without_progress(mut self, nudges: usize) -> Self {
        self.max_without_progress = nudges;
        self
    }

    /// The name of the task-list tool whose open items count as unfinished work.
    pub fn todo_tool(mut self, name: impl Into<String>) -> Self {
        self.todo_tool = name.into();
        self
    }

    /// Open items of the latest successful todo update made in this run, when any of them was never
    /// started. A lone item left `in_progress` usually means the model finished and forgot to tick it.
    fn open_todos(&self, run: &[Message]) -> Vec<String> {
        for (at, message) in run.iter().enumerate().rev() {
            let Some(call) = message.tool_calls.iter().rev().find(|c| c.name == self.todo_tool) else {
                continue;
            };
            let applied = run[at + 1..]
                .iter()
                .any(|m| m.role == Role::Tool && m.tool_call_id.as_deref() == Some(call.id.as_str()) && m.text().starts_with("Todo list updated"));
            if !applied {
                continue;
            }
            let Ok(args) = serde_json::from_str::<Value>(&call.arguments) else { return Vec::new() };
            let items: Vec<(&str, &str)> = args
                .get("todos")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|item| Some((item.get("content")?.as_str()?, item.get("status")?.as_str()?)))
                .collect();
            if !items.iter().any(|&(_, status)| status == "pending") {
                return Vec::new();
            }
            return items.into_iter().filter(|&(_, status)| status != "completed").map(|(content, _)| content.to_string()).collect();
        }
        Vec::new()
    }
}

impl Default for ContinueGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// The text after the last sentence break, ignoring trailing whitespace and markdown emphasis.
fn last_sentence(text: &str) -> &str {
    let text = text.trim_end_matches(|c: char| c.is_whitespace() || c == '*' || c == '`');
    // The final character may itself be a full stop; look for a break before it.
    let body_end = text.char_indices().last().map_or(0, |(at, _)| at);
    let start = text[..body_end]
        .char_indices()
        .rev()
        .find(|&(_, c)| matches!(c, '\n' | '。' | '！' | '!' | '？' | '?' | '.'))
        .map_or(0, |(at, c)| at + c.len_utf8());
    text[start..].trim()
}

fn asks_user(sentence: &str) -> bool {
    sentence.ends_with('?') || sentence.ends_with('？') || sentence.ends_with("吗") || sentence.ends_with("呢")
}

/// The reply ends by introducing something that should follow ("…as follows:") or by saying what the
/// model will do now, with nothing after it.
fn announces_action(text: &str) -> bool {
    let sentence = last_sentence(text);
    if sentence.is_empty() || asks_user(sentence) {
        return false;
    }
    if sentence.ends_with(':') || sentence.ends_with('：') {
        return true;
    }
    let lower = sentence.to_lowercase();
    !HANDING_BACK.iter().any(|phrase| lower.contains(phrase)) && ANNOUNCING.iter().any(|phrase| lower.contains(phrase))
}

impl Middleware for ContinueGuard {
    fn before_stop(&self, stop: &Stop<'_>) -> StopDecision {
        if self.max_per_run == 0 {
            return StopDecision::Finish;
        }
        let Some(reply) = stop.history.last() else { return StopDecision::Finish };
        let text = reply.text();
        let nudge = if stop.finish == FinishReason::Length {
            Nudge { reason: "length".into(), message: LENGTH.into() }
        } else if stop.finish != FinishReason::Stop || asks_user(last_sentence(&text)) {
            return StopDecision::Finish;
        } else if text.trim().is_empty() {
            Nudge { reason: "empty".into(), message: EMPTY.into() }
        } else {
            let run = stop.history.get(stop.run_start..).unwrap_or_default();
            let open = self.open_todos(run);
            let after_nudge = stop.history.len() >= 2 && stop.history[stop.history.len() - 2].synthetic;
            if open.is_empty() || after_nudge {
                if announces_action(&text) {
                    return self.decide(stop, Nudge { reason: "announced_action".into(), message: ANNOUNCED.into() });
                }
                return StopDecision::Finish;
            }
            let list: String = open.iter().map(|item| format!("\n- {item}")).collect();
            Nudge {
                reason: "unfinished_todos".into(),
                message: format!(
                    "You stopped, but your todo list still has unfinished items:{list}\n\
                     If you can keep going, continue now with tool calls. If you are waiting for the user or are blocked, \
                     say so in one or two sentences and end your turn. If the plan changed, update the todo list first."
                ),
            }
        };
        self.decide(stop, nudge)
    }
}

impl ContinueGuard {
    fn decide(&self, stop: &Stop<'_>, nudge: Nudge) -> StopDecision {
        if stop.continuations >= self.max_per_run {
            StopDecision::Pause("continuation_limit".into())
        } else if stop.continuations_without_progress >= self.max_without_progress {
            StopDecision::Pause("no_progress".into())
        } else {
            StopDecision::Continue(nudge)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_sentence_is_what_follows_the_last_full_stop() {
        assert_eq!(last_sentence("设计定稿。\n\n先写世界数据和数学基础："), "先写世界数据和数学基础：");
        assert_eq!(last_sentence("Done. Let me check the file."), "Let me check the file.");
        assert_eq!(last_sentence("**全部完成**"), "**全部完成");
        assert_eq!(last_sentence("单句"), "单句");
    }

    #[test]
    fn announcements_are_told_apart_from_answers_and_questions() {
        assert!(announces_action("现在开始写核心引擎。先写世界数据和数学基础："));
        assert!(announces_action("Got it. Let me write the renderer next."));
        assert!(announces_action("好的，我先看一下目录结构"));
        assert!(!announces_action("游戏已经写好了，可以在游戏空间里打开。"));
        assert!(!announces_action("要我先读一下代码吗？"));
        assert!(!announces_action("Should I go ahead?"));
        assert!(!announces_action("以下是攻略：\n1. 第一天去香山。"));
        assert!(!announces_action("Let me know if you want changes."));
        assert!(!announces_action("有问题随时让我知道。"));
    }
}

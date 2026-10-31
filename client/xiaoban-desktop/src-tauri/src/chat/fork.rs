//! "从这句话分叉": a new conversation that starts with a copy of the history before a chosen message.
//!
//! A turn is counted by the messages the person wrote (not the ones the harness wrote), so the page and the saved
//! history agree on which message is meant without sharing any other numbering.

use agent_file_tools::Attachment;
use agent_kernel::{Message, Role};

/// Where the history is cut so that it ends just before the `turn`th message the person wrote (counting from 0).
/// A `turn` equal to the number of such messages keeps everything.
pub fn cut_before_turn(history: &[Message], turn: usize) -> Result<usize, String> {
    let mut seen = 0;
    for (at, message) in history.iter().enumerate() {
        if message.role == Role::User && !message.synthetic {
            if seen == turn {
                return Ok(at);
            }
            seen += 1;
        }
    }
    if seen == turn {
        Ok(history.len())
    } else {
        Err("找不到这句话，对话可能已经变了，请重新打开再试".into())
    }
}

/// Points every attachment card in `history` at the copy `copy` made, so the new conversation owns its files.
pub fn rewrite_attachments(
    history: &mut [Message],
    mut copy: impl FnMut(&[Attachment]) -> Result<Vec<Attachment>, String>,
) -> Result<(), String> {
    for message in history.iter_mut().filter(|m| m.role == Role::User) {
        let Some(saved) = message.structured_content.as_mut().and_then(|v| v.get_mut("attachments")) else { continue };
        let Ok(items) = serde_json::from_value::<Vec<Attachment>>(saved.clone()) else { continue };
        if items.is_empty() {
            continue;
        }
        *saved = serde_json::to_value(copy(&items)?).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn user(text: &str) -> Message {
        Message::user_text(text)
    }

    fn answer(text: &str) -> Message {
        Message::assistant(text, Vec::new())
    }

    fn history() -> Vec<Message> {
        vec![user("一"), answer("答一"), Message::reminder("请继续"), user("二"), answer("答二")]
    }

    #[test]
    fn the_cut_falls_just_before_the_chosen_message_the_person_wrote() {
        let h = history();
        assert_eq!(cut_before_turn(&h, 0), Ok(0));
        assert_eq!(cut_before_turn(&h, 1), Ok(3));
    }

    #[test]
    fn a_turn_after_the_last_message_keeps_everything() {
        let h = history();
        assert_eq!(cut_before_turn(&h, 2), Ok(h.len()));
        assert_eq!(cut_before_turn(&[], 0), Ok(0));
    }

    #[test]
    fn a_turn_that_does_not_exist_is_refused() {
        assert!(cut_before_turn(&history(), 3).is_err());
        assert!(cut_before_turn(&[], 1).is_err());
    }

    #[test]
    fn messages_the_harness_wrote_are_not_counted_as_turns() {
        assert_eq!(cut_before_turn(&[Message::reminder("请继续")], 0), Ok(1));
    }

    #[test]
    fn attachment_cards_are_rewritten_and_other_messages_are_left_alone() {
        let card = |path: &str| json!({"id":"a","name":"预算.txt","size":2,"mime":"text/plain","path":path});
        let mut with = user("看附件");
        with.structured_content = Some(json!({"attachments":[card("/old/a.txt")],"outboxId":"x"}));
        let mut h = vec![with, user("没有附件")];
        let before = h[1].clone();
        let mut asked = 0;
        rewrite_attachments(&mut h, |items| {
            asked += 1;
            Ok(items.iter().cloned().map(|a| Attachment { path: "/new/a.txt".into(), ..a }).collect())
        })
        .unwrap();
        assert_eq!(asked, 1);
        let saved = h[0].structured_content.as_ref().unwrap();
        assert_eq!(saved["attachments"][0]["path"], "/new/a.txt");
        assert_eq!(saved["outboxId"], "x");
        assert_eq!(h[1], before);
    }

    #[test]
    fn a_failed_copy_stops_the_rewrite() {
        let mut with = user("看附件");
        with.structured_content = Some(json!({"attachments":[{"id":"a","name":"n","size":1,"mime":"text/plain","path":"/p"}]}));
        let mut h = vec![with];
        assert_eq!(rewrite_attachments(&mut h, |_| Err("磁盘满了".into())), Err("磁盘满了".to_string()));
    }
}

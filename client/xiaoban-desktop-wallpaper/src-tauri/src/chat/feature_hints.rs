//! Before each model call, puts the features the user's latest request is about in front of the model (see
//! `app_guide::hint_for`). Whether the model remembers to look up its own abilities is not left to chance: the
//! matching is plain code with a test table, and the hint is added to this request only, never to the stored chat.
use agent_kernel::{HarnessError, Message, Middleware, Role, Turn};

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

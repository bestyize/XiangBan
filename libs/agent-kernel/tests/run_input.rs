mod common;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

struct Inbox {
    pending: Mutex<Vec<Message>>,
    accepted: Mutex<usize>,
    store: Arc<MemoryStore>,
}
impl std::fmt::Debug for Inbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("test inbox")
    }
}
impl RunInput for Inbox {
    fn poll(&self, session: &str, _closing: bool) -> Result<Vec<Message>, HarnessError> {
        if session != "s1" {
            return Ok(vec![]);
        }
        Ok(std::mem::take(&mut *self.pending.lock().unwrap()))
    }
    fn accepted(&self, session: &str, messages: &[Message]) -> Result<(), HarnessError> {
        let saved = self.store.load(session)?;
        for message in messages {
            assert!(saved
                .iter()
                .any(|m| m.structured_content == message.structured_content));
        }
        *self.accepted.lock().unwrap() += messages.len();
        Ok(())
    }
}
fn input(store: Arc<MemoryStore>) -> Arc<Inbox> {
    Arc::new(Inbox {
        pending: Mutex::new(vec![]),
        accepted: Mutex::new(0),
        store,
    })
}
fn follow() -> Message {
    let mut m = Message::user(vec![
        Part::Text {
            text: "look at the screenshot".into(),
        },
        Part::Image {
            url: "data:image/png;base64,YQ==".into(),
        },
    ]);
    m.structured_content = Some(json!({"outboxId":"q1"}));
    m
}
#[test]
fn accepts_follow_up_after_tool_results_and_persists_before_ack() {
    let store = Arc::new(MemoryStore::new());
    let inbox = input(store.clone());
    let queued = inbox.clone();
    let provider = Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(
        vec![
            Step::tool_call("t", "work", "{}"),
            Step::text("checked screenshot"),
        ],
    ));
    let agent = Agent::builder(provider.clone(), store.clone())
        .tool(FnTool::new(spec("work"), move |_, _| {
            queued.pending.lock().unwrap().push(follow());
            Ok("done".into())
        }))
        .build()
        .unwrap();
    let mut events = vec![];
    agent
        .run_parts_observed(
            "s1",
            vec![Part::Text {
                text: "work".into(),
            }],
            RunOptions {
                input: Some(inbox.clone()),
                ..Default::default()
            },
            &mut |r| {
                if let RunEventBody::Event { event } = r.body {
                    events.push(event);
                }
                Ok(())
            },
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let roles = store
        .load("s1")
        .unwrap()
        .iter()
        .map(|m| m.role)
        .collect::<Vec<_>>();
    assert_eq!(
        roles,
        vec![
            Role::User,
            Role::Assistant,
            Role::Tool,
            Role::User,
            Role::Assistant
        ]
    );
    let requests = provider.requests();
    assert!(matches!(
        requests[1].messages.last().unwrap().parts.last(),
        Some(Part::Image { .. })
    ));
    assert_eq!(*inbox.accepted.lock().unwrap(), 1);
    let tool = events
        .iter()
        .position(|e| matches!(e, Event::ToolEnd { .. }))
        .unwrap();
    let user = events
        .iter()
        .position(|e| matches!(e, Event::UserMessage { .. }))
        .unwrap();
    assert!(tool < user);
}
#[test]
fn catches_input_arriving_with_final_text_instead_of_finishing_early() {
    let h = harness(
        vec![Step::text("first answer"), Step::text("updated answer")],
        |b| b,
    );
    let inbox = input(h.store.clone());
    let mut sent = false;
    let result = h
        .agent
        .run_parts_observed(
            "s1",
            vec![Part::Text {
                text: "question".into(),
            }],
            RunOptions {
                input: Some(inbox.clone()),
                ..Default::default()
            },
            &mut |r| {
                if !sent
                    && matches!(
                        r.body,
                        RunEventBody::Event {
                            event: Event::TextDelta { .. }
                        }
                    )
                {
                    inbox.pending.lock().unwrap().push(follow());
                    sent = true;
                }
                Ok(())
            },
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(result.turns, 2);
    assert_eq!(result.text, "updated answer");
    assert_eq!(*inbox.accepted.lock().unwrap(), 1);
}
#[test]
fn cancellation_does_not_consume_waiting_input() {
    let store = Arc::new(MemoryStore::new());
    let inbox = input(store.clone());
    let queued = inbox.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let stop = cancel.clone();
    let provider = Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(
        vec![Step::tool_call("t", "work", "{}")],
    ));
    let agent = Agent::builder(provider, store.clone())
        .tool(FnTool::new(spec("work"), move |_, _| {
            queued.pending.lock().unwrap().push(follow());
            stop.store(true, Ordering::SeqCst);
            Ok("done".into())
        }))
        .build()
        .unwrap();
    let result = agent.run_parts_observed(
        "s1",
        vec![Part::Text {
            text: "question".into(),
        }],
        RunOptions {
            input: Some(inbox.clone()),
            ..Default::default()
        },
        &mut |_| Ok(()),
        &cancel,
    );
    assert!(matches!(result, Err(HarnessError::Cancelled)));
    assert_eq!(*inbox.accepted.lock().unwrap(), 0);
    assert_eq!(inbox.pending.lock().unwrap().len(), 1);
}

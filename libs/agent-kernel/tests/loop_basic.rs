mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn plain_reply_streams_text_and_persists_history() {
    let h = harness(vec![Step::chunks(&["Hel", "lo"]).with_usage(10, 2)], |b| b.system_prompt("be brief"));
    let (result, events) = run(&h.agent, "hi");
    let summary = result.unwrap();
    assert_eq!(summary.text, "Hello");
    assert_eq!(summary.turns, 1);
    assert_eq!(summary.usage, Usage { prompt_tokens: 10, completion_tokens: 2, ..Default::default() });
    assert_eq!(
        events,
        vec![
            Event::TextDelta { text: "Hel".into() },
            Event::TextDelta { text: "lo".into() },
            Event::Usage { prompt_tokens: 10, completion_tokens: 2 },
            Event::Done,
        ]
    );
    let history = h.store.load("s1").unwrap();
    assert_eq!(history.iter().map(|m| m.role).collect::<Vec<_>>(), vec![Role::User, Role::Assistant]);
    let first = &h.provider.requests()[0].messages;
    assert_eq!((first[0].role, first[0].text().as_str()), (Role::System, "be brief"));
}

#[test]
fn kernel_streams_and_persists_model_exposed_reasoning_without_polluting_answer() {
    struct SummaryProvider;
    impl ModelProvider for SummaryProvider {
        fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool)
            -> Result<Reply, HarnessError>
        {
            self.chat_with_reasoning(request, on_text, &mut |_, _| {}, cancel)
        }

        fn chat_with_reasoning(&self, _request: &ChatRequest, on_text: &mut dyn FnMut(&str),
            on_reasoning: &mut dyn FnMut(ReasoningKind, &str), _cancel: &AtomicBool)
            -> Result<Reply, HarnessError>
        {
            on_reasoning(ReasoningKind::Summary, "检查问题");
            on_reasoning(ReasoningKind::Summary, "，给出答案");
            on_text("最终答案");
            Ok(Reply { message: Message::assistant("最终答案", vec![]), usage: None, finish: FinishReason::Stop })
        }
    }
    let store = Arc::new(MemoryStore::new());
    let agent = Agent::builder(Arc::new(SummaryProvider), store.clone()).build().unwrap();
    let mut events = Vec::new();
    let summary = agent.run("reasoning-1", "请回答", &mut |e| events.push(e),
        &Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(summary.text, "最终答案");
    assert_eq!(&events[..3], &[
        Event::ReasoningDelta { kind: ReasoningKind::Summary, text: "检查问题".into() },
        Event::ReasoningDelta { kind: ReasoningKind::Summary, text: "，给出答案".into() },
        Event::TextDelta { text: "最终答案".into() },
    ]);
    let history = store.load("reasoning-1").unwrap();
    let answer = history.iter().find(|m| m.role == Role::Assistant).unwrap();
    assert_eq!(answer.text(), "最终答案");
    assert_eq!(answer.reasoning, vec![ReasoningSegment {
        kind: ReasoningKind::Summary, text: "检查问题，给出答案".into(),
    }]);
}

#[test]
fn tool_round_trip_feeds_result_back() {
    let echo = FnTool::new(spec("echo"), |args, _| Ok(format!("got {}", args["x"])));
    let steps = vec![Step::tool_call("c1", "echo", r#"{"x":1}"#), Step::text("done")];
    let h = harness(steps, |b| b.tool(echo));
    let (result, events) = run(&h.agent, "go");
    assert_eq!(result.unwrap().text, "done");
    assert!(events.contains(&Event::ToolStart { id: "c1".into(), name: "echo".into(), arguments: r#"{"x":1}"#.into() }));
    assert!(events.contains(&Event::ToolEnd { id: "c1".into(), output: "got 1".into(), is_error: false, parts: vec![], structured_content: None }));
    assert_eq!(tool_messages(&h), vec![("c1".to_string(), "got 1".to_string())]);
    assert_eq!(h.store.load("s1").unwrap().len(), 4);
    assert_eq!(h.provider.requests()[0].tools[0].name, "echo");
}

#[test]
fn unknown_tool_and_bad_arguments_become_error_results() {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = ran.clone();
    let tool = FnTool::new(spec("real"), move |_, _| {
        flag.store(true, Ordering::SeqCst);
        Ok("ran".into())
    });
    let steps = vec![
        Step::tool_call("a", "missing", "{}").and_tool_call("b", "real", "{not json"),
        Step::text("ok"),
    ];
    let h = harness(steps, |b| b.tool(tool));
    let (result, events) = run(&h.agent, "go");
    assert!(result.is_ok());
    assert!(!ran.load(Ordering::SeqCst), "tool must not run with unparsable arguments");
    let messages = tool_messages(&h);
    assert!(messages[0].1.contains("unknown tool `missing`"));
    assert!(messages[1].1.contains("not valid JSON"));
    let errors = events.iter().filter(|e| matches!(e, Event::ToolEnd { is_error: true, .. })).count();
    assert_eq!(errors, 2);
}

#[test]
fn empty_arguments_are_an_empty_object() {
    let seen = Arc::new(Mutex::new(None));
    let slot = seen.clone();
    let tool = FnTool::new(spec("noargs"), move |args, _| {
        *slot.lock().unwrap() = Some(args);
        Ok("fine".into())
    });
    let h = harness(vec![Step::tool_call("c", "noargs", ""), Step::text("ok")], |b| b.tool(tool));
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(seen.lock().unwrap().clone().unwrap(), serde_json::json!({}));
}

#[test]
fn turn_limit_stops_a_looping_model() {
    let tool = FnTool::new(spec("t"), |_, _| Ok("x".into()));
    let steps = vec![Step::tool_call("1", "t", "{}"), Step::tool_call("2", "t", "{}"), Step::text("never")];
    let h = harness(steps, |b| b.tool(tool).max_turns(2));
    let (result, events) = run(&h.agent, "go");
    assert_eq!(result.unwrap_err(), HarnessError::MaxTurns(2));
    assert!(matches!(events.last(), Some(Event::Error { .. })));
    assert_eq!(h.provider.requests().len(), 2);
}

#[test]
fn second_run_on_a_busy_session_is_rejected() {
    let slow = FnTool::new(spec("slow"), |_, _| {
        std::thread::sleep(Duration::from_millis(300));
        Ok("done".into())
    });
    let h = harness(vec![Step::tool_call("c", "slow", "{}"), Step::text("ok"), Step::text("again")], |b| b.tool(slow));
    std::thread::scope(|scope| {
        let first = scope.spawn(|| run(&h.agent, "first").0);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(run(&h.agent, "second").0.unwrap_err(), HarnessError::SessionBusy("s1".into()));
        assert!(first.join().unwrap().is_ok());
    });
    assert!(run(&h.agent, "third").0.is_ok(), "session is released after the run ends");
}

#[test]
fn builder_rejects_bad_tool_names() {
    let build = |names: &[&str]| {
        let provider = Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(vec![]));
        let mut b = Agent::builder(provider, Arc::new(MemoryStore::new()));
        for n in names {
            b = b.tool(FnTool::new(spec(n), |_, _| Ok(String::new())));
        }
        b.build().err()
    };
    assert!(build(&["ok_name", "other-1"]).is_none());
    assert!(matches!(build(&["dup", "dup"]), Some(HarnessError::Config(_))));
    assert!(matches!(build(&["has space"]), Some(HarnessError::Config(_))));
    assert!(matches!(build(&[""]), Some(HarnessError::Config(_))));
}

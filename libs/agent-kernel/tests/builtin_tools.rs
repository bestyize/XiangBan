mod common;

use agent_kernel::builtin::{AskUser, Question, TimeNow, Todo, TodoStatus};
use agent_kernel::provider::scripted::Step;
use common::*;
use std::sync::{Arc, Mutex};

fn one_call(tool_name: &str, args: &str, configure: impl FnOnce(agent_kernel::AgentBuilder) -> agent_kernel::AgentBuilder) -> String {
    let h = harness(vec![Step::tool_call("c", tool_name, args), Step::text("ok")], configure);
    assert!(run(&h.agent, "go").0.is_ok());
    tool_messages(&h).remove(0).1
}

#[test]
fn time_now_reports_a_plausible_current_time_in_the_host_offset() {
    let utc = one_call("time_now", "{}", |b| b.tool(TimeNow::new()));
    assert!(utc.contains("(UTC+00:00)") && utc.contains("Unix time: "), "{utc}");
    let year: u32 = utc[..4].parse().unwrap();
    assert!(year >= 2025, "{utc}");
    let tokyo = one_call("time_now", "{}", |b| b.tool(TimeNow::new().utc_offset_minutes(540)));
    assert!(tokyo.contains("(UTC+09:00)"), "{tokyo}");
}

#[test]
fn todo_replaces_the_list_per_session_and_validates_input() {
    let todo = Arc::new(Todo::new());
    let args = r#"{"todos":[{"content":"read code","status":"completed"},{"content":" write fix ","status":"in_progress"},{"content":"test","status":"pending"}]}"#;
    let out = one_call("todo", args, |b| b.tool_arc(todo.clone()));
    assert_eq!(out, "Todo list updated:\n[x] read code\n[~] write fix\n[ ] test");
    let saved = todo.snapshot("s1");
    assert_eq!((saved.len(), saved[1].content.as_str(), saved[1].status), (3, "write fix", TodoStatus::InProgress));
    assert!(todo.snapshot("other").is_empty());

    let replaced = one_call("todo", r#"{"todos":[]}"#, |b| b.tool_arc(todo.clone()));
    assert!(replaced.ends_with("(empty)"));
    assert!(todo.snapshot("s1").is_empty());

    let bad = |args: &str| one_call("todo", args, |b| b.tool(Todo::new()));
    assert!(bad(r#"{"todos":[{"content":"a","status":"in_progress"},{"content":"b","status":"in_progress"}]}"#).contains("only one item"));
    assert!(bad(r#"{"todos":[{"content":"a","status":"started"}]}"#).contains("item 1"));
    assert!(bad(r#"{"todos":[{"content":"  ","status":"pending"}]}"#).contains("1 to 300"));
    assert!(bad(r#"{"nothing":1}"#).contains("`todos` is required"));
    let many = format!(r#"{{"todos":[{}]}}"#, vec![r#"{"content":"x","status":"pending"}"#; 51].join(","));
    assert!(bad(&many).contains("too many items"));
    assert!(todo.snapshot("s1").is_empty(), "invalid updates leave the list unchanged");
}

#[test]
fn ask_user_passes_the_question_to_the_host_and_returns_the_answer() {
    let seen: Arc<Mutex<Vec<(String, Question)>>> = Arc::default();
    let log = seen.clone();
    let answers = Mutex::new(vec![None, Some("blue".to_string())].into_iter());
    let ask = AskUser::new(move |session, q, _| {
        log.lock().unwrap().push((session.to_string(), q.clone()));
        answers.lock().unwrap().next().flatten()
    });
    let args = r#"{"question":" Which colour? ","options":["red"," blue ","green"]}"#;
    let steps = vec![Step::tool_call("a", "ask_user", args).and_tool_call("b", "ask_user", args), Step::text("ok")];
    let h = harness(steps, |b| b.tool(ask));
    assert!(run(&h.agent, "go").0.is_ok());
    let messages = tool_messages(&h);
    assert!(messages[0].1.contains("dismissed"));
    assert_eq!(messages[1].1, "The user answered: blue");
    let seen = seen.lock().unwrap();
    assert_eq!(seen[0].0, "s1");
    assert_eq!(seen[0].1, Question { text: "Which colour?".into(), options: vec!["red".into(), "blue".into(), "green".into()] });
}

#[test]
fn ask_user_rejects_bad_arguments_without_bothering_the_user() {
    let asked = Arc::new(Mutex::new(0));
    let counter = asked.clone();
    let ask = AskUser::new(move |_, _, _| {
        *counter.lock().unwrap() += 1;
        Some("x".into())
    });
    let steps = vec![
        Step::tool_call("a", "ask_user", r#"{"question":""}"#)
            .and_tool_call("b", "ask_user", r#"{"question":"q","options":[1]}"#)
            .and_tool_call("c", "ask_user", r#"{"question":"q","options":"red"}"#),
        Step::text("ok"),
    ];
    let h = harness(steps, |b| b.tool(ask));
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(*asked.lock().unwrap(), 0);
    assert_eq!(tool_messages(&h).len(), 3);
}

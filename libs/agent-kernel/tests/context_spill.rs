mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::Arc;

fn call(id: &str) -> ToolCall {
    ToolCall { id: id.into(), name: "t".into(), arguments: "{}".into() }
}

fn turn(messages: Vec<Message>) -> Turn {
    Turn { session: "s".into(), index: 0, messages, tools: vec![] }
}

fn exchange(n: usize, output_len: usize) -> Vec<Message> {
    vec![
        Message::user_text(format!("question {n}")),
        Message::assistant("", vec![call(&format!("c{n}"))]),
        Message::tool(format!("c{n}"), "x".repeat(output_len)),
        Message::assistant(format!("answer {n}"), vec![]),
    ]
}

#[test]
fn large_images_cost_more_and_output_space_is_reserved() {
    let image = |size| Message::user(vec![Part::Image { url: format!("data:image/png;base64,{}", "A".repeat(size)) }]);
    let small = estimate_tokens(&[image(100)]);
    let large = estimate_tokens(&[image(128_000)]);
    assert!(small >= 4_096 && large > small);
    let mut t = turn(vec![Message::user_text("x".repeat(4_000))]);
    assert!(ContextTrim::new(1_200).reserve_output(300).before_model(&mut t).is_err());
}

#[test]
fn synthetic_continuation_does_not_turn_the_current_exchange_into_discardable_history() {
    let mut t = turn(vec![Message::user_text("x".repeat(4_000)), Message::reminder("keep working"), Message::assistant("reply", vec![])]);
    assert!(ContextTrim::new(100).before_model(&mut t).is_err());
    assert_eq!(t.messages.len(), 3);
}

#[test]
fn under_budget_nothing_changes() {
    let mut t = turn([vec![Message::system("sys")], exchange(1, 40)].concat());
    let before = t.messages.clone();
    ContextTrim::new(10_000).before_model(&mut t).unwrap();
    assert_eq!(t.messages, before);
}

#[test]
fn old_tool_output_is_cleared_before_anything_is_dropped() {
    let mut messages = vec![Message::system("sys")];
    for n in 1..=3 {
        messages.extend(exchange(n, 4000));
    }
    let total = estimate_tokens(&messages);
    let mut t = turn(messages.clone());
    ContextTrim::new(total - 1500).keep_recent(4).before_model(&mut t).unwrap();
    assert_eq!(t.messages.len(), messages.len(), "no message dropped");
    let cleared: Vec<_> = t.messages.iter().filter(|m| m.text().starts_with("[older tool output cleared")).collect();
    assert!(!cleared.is_empty());
    let last_tool = t.messages.iter().rev().find(|m| m.role == Role::Tool).unwrap();
    assert_eq!(last_tool.text().len(), 4000, "recent tool output is kept");
    assert!(estimate_tokens(&t.messages) <= total - 1500);
}

#[test]
fn oldest_exchanges_are_dropped_at_user_boundaries_keeping_system_and_pairs() {
    let mut messages = vec![Message::system("sys")];
    for n in 1..=4 {
        messages.extend(exchange(n, 400));
    }
    let one = estimate_tokens(&exchange(1, 400));
    let mut t = turn(messages);
    // keep_recent covers every message, so clearing is skipped and only dropping can help.
    ContextTrim::new(one * 2 + 40).keep_recent(1000).before_model(&mut t).unwrap();
    assert_eq!(t.messages[0].text(), "sys");
    assert!(t.messages[1].text().contains("omitted"));
    assert_eq!(t.messages[2].text(), "question 3", "starts again at a user message");
    let ids: Vec<_> = t.messages.iter().filter_map(|m| m.tool_call_id.clone()).collect();
    let called: Vec<_> = t.messages.iter().flat_map(|m| m.tool_calls.iter().map(|c| c.id.clone())).collect();
    assert_eq!(ids, called, "every kept tool result still has its call");
}

#[test]
fn the_current_exchange_is_never_dropped() {
    let mut t = turn(vec![Message::user_text("x".repeat(40_000)), Message::assistant("", vec![call("c")]), Message::tool("c", "y".repeat(40_000))]);
    assert!(ContextTrim::new(100).before_model(&mut t).is_err());
    assert_eq!(t.messages.len(), 3, "an oversized current exchange is retained but rejected before calling the provider");
}

#[test]
fn estimate_counts_cjk_heavier_than_ascii() {
    let ascii = estimate_tokens(&[Message::user_text("a".repeat(400))]);
    let cjk = estimate_tokens(&[Message::user_text("你".repeat(400))]);
    assert!(cjk > ascii * 3);
}

#[test]
fn trimming_applies_to_requests_not_to_the_stored_session() {
    let history: Vec<Message> = (1..=6).flat_map(|n| exchange(n, 2000)).collect();
    let h = harness(vec![Step::text("ok")], |b| b.middleware(ContextTrim::new(1500).keep_recent(2)));
    h.store.append("s1", &history).unwrap();
    assert!(run(&h.agent, "next").0.is_ok());
    let sent = &h.provider.requests()[0].messages;
    assert!(estimate_tokens(sent) < estimate_tokens(&history));
    assert_eq!(h.store.load("s1").unwrap().len(), history.len() + 2, "stored history is untouched");
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("agent-kernel-{tag}-{}-{:?}", std::process::id(), std::thread::current().id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn spill_saves_big_output_and_leaves_a_preview_with_the_path() {
    let dir = temp_dir("spill");
    let body = format!("START{}END", "m".repeat(50_000));
    let (big, small) = (body.clone(), "tiny".to_string());
    let tools = [("big", big), ("small", small)].map(|(name, text)| FnTool::new(spec(name).max_output(1_000_000), move |_, _| Ok(text.clone())));
    let steps = vec![Step::tool_call("a", "big", "{}").and_tool_call("b", "small", "{}"), Step::text("ok")];
    let h = harness(steps, |b| {
        let [t1, t2] = tools;
        b.tool(t1).tool(t2).middleware(SpillOutput::new(&dir).threshold(10_000))
    });
    assert!(run(&h.agent, "go").0.is_ok());
    let messages = tool_messages(&h);
    assert_eq!(messages[1].1, "tiny");
    let shown = &messages[0].1;
    assert!(shown.starts_with("START") && shown.contains("bytes omitted") && shown.len() < 5000);
    let path = shown.split("is saved at ").nth(1).unwrap().split(". Read it").next().unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), body);
    assert!(std::path::Path::new(path).starts_with(&dir));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn spill_falls_back_to_plain_truncation_when_the_directory_is_unusable() {
    let dir = temp_dir("spill-bad");
    std::fs::create_dir_all(dir.parent().unwrap()).unwrap();
    std::fs::write(&dir, "i am a file, not a directory").unwrap();
    let tool = FnTool::new(spec("big").max_output(2_000), |_, _| Ok("z".repeat(30_000)));
    let h = harness(vec![Step::tool_call("a", "big", "{}"), Step::text("ok")], |b| {
        b.tool(tool).middleware(Arc::new(SpillOutput::new(&dir).threshold(10_000)))
    });
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(tool_messages(&h)[0].1.contains("bytes truncated"));
    let _ = std::fs::remove_file(&dir);
}

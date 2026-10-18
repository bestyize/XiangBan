mod common;

use agent_kernel::builtin::Todo;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const PLAN: &str = r#"{"todos": [{"content": "设计方案", "status": "completed"}, {"content": "编写 game.js", "status": "pending"}]}"#;
const DONE: &str = r#"{"todos": [{"content": "设计方案", "status": "completed"}, {"content": "编写 game.js", "status": "completed"}]}"#;

fn echo(runs: &Arc<AtomicUsize>) -> FnTool {
    let runs = runs.clone();
    FnTool::new(spec("echo"), move |_, _| {
        runs.fetch_add(1, Ordering::SeqCst);
        Ok("echoed".into())
    })
}

fn reasons(events: &[Event]) -> Vec<String> {
    events.iter().filter_map(|e| if let Event::Continuing { reason } = e { Some(reason.clone()) } else { None }).collect()
}

fn guarded(steps: Vec<Step>, runs: &Arc<AtomicUsize>) -> Harness {
    harness(steps, |b| b.tool(Todo::new()).tool(echo(runs)).middleware(ContinueGuard::new()))
}

#[test]
fn an_announced_step_with_open_todos_is_nudged_and_the_work_gets_done() {
    let runs = Arc::new(AtomicUsize::new(0));
    let steps = vec![
        Step::tool_call("t1", "todo", PLAN),
        Step::text("设计定稿。现在开始写核心引擎。\n\n先写世界数据和数学基础："),
        Step::tool_call("w1", "echo", r#"{"path": "game.js"}"#),
        Step::tool_call("t2", "todo", DONE),
        Step::text("花果山写好了，可以在游戏空间里打开。"),
    ];
    let h = guarded(steps, &runs);
    let (result, events) = run(&h.agent, "做一个花果山游戏");
    let summary = result.unwrap();
    assert_eq!(summary.text, "花果山写好了，可以在游戏空间里打开。");
    assert_eq!(runs.load(Ordering::SeqCst), 1, "the step it announced really ran");
    assert_eq!(reasons(&events), vec!["unfinished_todos"]);

    let history = h.store.load("s1").unwrap();
    let synthetic: Vec<&Message> = history.iter().filter(|m| m.synthetic).collect();
    assert_eq!(synthetic.len(), 1);
    assert_eq!(synthetic[0].role, Role::User);
    assert!(synthetic[0].text().contains("编写 game.js"), "the nudge names what is still open");
    assert!(!synthetic[0].text().contains("设计方案"), "completed items are not listed");

    let requests = h.provider.requests();
    assert!(requests[2].messages.last().unwrap().synthetic, "the model sees the nudge as the latest message");
}

#[test]
fn an_announced_step_without_a_plan_is_nudged_too() {
    let runs = Arc::new(AtomicUsize::new(0));
    let steps = vec![Step::text("Sure. Let me check the file."), Step::tool_call("a", "echo", "{}"), Step::text("It is fine.")];
    let h = guarded(steps, &runs);
    let (result, events) = run(&h.agent, "check it");
    assert_eq!(result.unwrap().text, "It is fine.");
    assert_eq!(reasons(&events), vec!["announced_action"]);
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}

#[test]
fn plain_answers_and_questions_end_the_run() {
    for text in ["北京今天晴，最高 18 度。", "要我先读一下代码吗？", "Done. Let me know if you want changes."] {
        let h = guarded(vec![Step::text(text)], &Arc::new(AtomicUsize::new(0)));
        let (result, events) = run(&h.agent, "hi");
        assert_eq!(result.unwrap().turns, 1, "{text}");
        assert!(reasons(&events).is_empty(), "{text}");
        assert!(h.store.load("s1").unwrap().iter().all(|m| !m.synthetic));
    }
}

#[test]
fn a_last_item_left_in_progress_is_taken_as_forgotten_not_unfinished() {
    let plan = r#"{"todos": [{"content": "设计", "status": "completed"}, {"content": "写文件", "status": "in_progress"}]}"#;
    let h = guarded(vec![Step::tool_call("t1", "todo", plan), Step::text("写好了")], &Arc::new(AtomicUsize::new(0)));
    let (result, events) = run(&h.agent, "写");
    assert_eq!(result.unwrap().turns, 2);
    assert!(reasons(&events).is_empty());
}

#[test]
fn a_question_to_the_user_is_not_overridden_by_open_todos() {
    let steps = vec![Step::tool_call("t1", "todo", PLAN), Step::text("你更想要战斗还是休闲？")];
    let h = guarded(steps, &Arc::new(AtomicUsize::new(0)));
    let (result, events) = run(&h.agent, "做个游戏");
    assert_eq!(result.unwrap().turns, 2);
    assert!(reasons(&events).is_empty());
}

#[test]
fn todos_left_open_by_an_earlier_run_do_not_hold_up_a_new_one() {
    let h = guarded(vec![Step::tool_call("t1", "todo", PLAN), Step::text("先停在这里，等你确认方案。"), Step::text("你好！")], &Arc::new(AtomicUsize::new(0)));
    let (_, first) = run(&h.agent, "做个游戏");
    assert_eq!(reasons(&first), vec!["unfinished_todos"], "the first run is nudged once");
    let h = guarded(vec![Step::tool_call("t1", "todo", PLAN), Step::text("你确认一下方案吗？"), Step::text("你好！")], &Arc::new(AtomicUsize::new(0)));
    run(&h.agent, "做个游戏").0.unwrap();
    let (result, events) = run(&h.agent, "hi");
    assert_eq!(result.unwrap().text, "你好！");
    assert!(reasons(&events).is_empty());
}

#[test]
fn a_nudge_the_model_declines_is_accepted() {
    let steps = vec![Step::text("我先看一下目录"), Step::text("我需要你先告诉我目录在哪里。")];
    let h = guarded(steps, &Arc::new(AtomicUsize::new(0)));
    let (result, events) = run(&h.agent, "整理文件");
    let summary = result.unwrap();
    assert_eq!((summary.text.as_str(), summary.turns), ("我需要你先告诉我目录在哪里。", 2));
    assert_eq!(reasons(&events).len(), 1);
}

#[test]
fn a_cut_off_answer_is_continued_without_flagging_truncation() {
    let steps = vec![Step::text("第一部分").finish(FinishReason::Length), Step::text("第二部分。")];
    let h = guarded(steps, &Arc::new(AtomicUsize::new(0)));
    let (result, events) = run(&h.agent, "写长文");
    let summary = result.unwrap();
    assert_eq!((summary.text.as_str(), summary.finish), ("第二部分。", FinishReason::Stop));
    assert_eq!(reasons(&events), vec!["length"]);
    assert!(!events.contains(&Event::Truncated), "it was continued, so the host has nothing to offer");
}

#[test]
fn the_last_cut_off_answer_is_still_flagged_when_nudges_run_out() {
    let steps = vec![Step::text("a").finish(FinishReason::Length), Step::text("b").finish(FinishReason::Length)];
    let h = harness(steps, |b| b.middleware(ContinueGuard::new().max_per_run(1)));
    let (result, events) = run(&h.agent, "写长文");
    assert_eq!(result.unwrap().finish, FinishReason::Length);
    let tail: Vec<&Event> = events.iter().filter(|e| matches!(e, Event::Truncated | Event::Done | Event::Continuing { .. })).collect();
    assert_eq!(tail, vec![&Event::Continuing { reason: "length".into() }, &Event::Truncated, &Event::Done]);
}

#[test]
fn an_empty_reply_is_nudged_once() {
    let h = guarded(vec![Step::text(""), Step::text("这是答案。")], &Arc::new(AtomicUsize::new(0)));
    let (result, events) = run(&h.agent, "hi");
    assert_eq!(result.unwrap().text, "这是答案。");
    assert_eq!(reasons(&events), vec!["empty"]);
}

#[test]
fn nudges_are_capped_per_run() {
    let mut steps = Vec::new();
    for i in 0..5 {
        steps.push(Step::text("让我继续："));
        steps.push(Step::tool_call(&format!("c{i}"), "echo", &format!("{{\"n\": {i}}}")));
    }
    steps.push(Step::text("让我继续："));
    let h = harness(steps, |b| b.tool(echo(&Arc::new(AtomicUsize::new(0)))).middleware(ContinueGuard::new().max_per_run(2)));
    let (result, events) = run(&h.agent, "go");
    assert!(result.is_ok());
    assert_eq!(reasons(&events).len(), 2);
}

#[test]
fn no_nudge_on_the_last_turn_so_the_run_ends_cleanly() {
    let h = harness(vec![Step::text("让我继续：")], |b| b.middleware(ContinueGuard::new()).max_turns(1));
    let (result, events) = run(&h.agent, "go");
    assert_eq!(result.unwrap().turns, 1);
    assert!(reasons(&events).is_empty());
}

#[test]
fn without_the_guard_a_text_reply_still_ends_the_run() {
    let h = harness(vec![Step::text("先写世界数据和数学基础：")], |b| b);
    let (result, events) = run(&h.agent, "go");
    assert_eq!(result.unwrap().turns, 1);
    assert!(reasons(&events).is_empty());
}

#[test]
fn synthetic_messages_round_trip_and_ordinary_ones_stay_unchanged_on_disk() {
    let dir = TempDir::new("guards-store");
    let store = JsonlStore::new(dir.0.clone());
    store.append("s", &[Message::user_text("hi"), Message::reminder("keep going")]).unwrap();
    let loaded = store.load("s").unwrap();
    assert_eq!((loaded[0].synthetic, loaded[1].synthetic), (false, true));
    let file = std::fs::read_dir(&dir.0).unwrap().map(|e| e.unwrap().path()).find(|p| p.extension().is_some_and(|x| x == "jsonl")).unwrap();
    let raw = std::fs::read_to_string(file).unwrap();
    let lines: Vec<&str> = raw.lines().collect();
    assert!(!lines[0].contains("synthetic"), "{}", lines[0]);
    assert!(lines[1].contains("\"synthetic\":true"), "{}", lines[1]);
}

#[test]
fn identical_calls_get_a_warning_then_are_refused() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut steps: Vec<Step> = (0..7).map(|i| Step::tool_call(&format!("c{i}"), "echo", r#"{"path": "a"}"#)).collect();
    steps.push(Step::text("放弃了。"));
    let h = harness(steps, |b| b.tool(echo(&runs)).middleware(RepeatGuard::new()));
    let (result, events) = run(&h.agent, "go");
    assert!(result.is_ok());
    assert_eq!(runs.load(Ordering::SeqCst), 5, "the sixth and seventh identical calls are refused");
    let outputs: Vec<(String, bool)> = events
        .iter()
        .filter_map(|e| if let Event::ToolEnd { output, is_error, .. } = e { Some((output.clone(), *is_error)) } else { None })
        .collect();
    assert_eq!(outputs[1], ("echoed".to_string(), false));
    assert!(outputs[2].0.starts_with("echoed") && outputs[2].0.contains("3 times in a row"), "{}", outputs[2].0);
    assert!(outputs[5].1 && outputs[5].0.starts_with("Not run:"), "{:?}", outputs[5]);
    assert!(outputs[6].1 && outputs[6].0.starts_with("Not run:"), "a refusal does not reset the streak");
}

#[test]
fn a_changed_argument_or_a_new_message_resets_the_streak() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut steps = Vec::new();
    for (i, args) in [r#"{"n": 1}"#, r#"{"n": 1}"#, r#"{"n": 2}"#, r#"{"n": 1}"#, r#"{"n": 1}"#].iter().enumerate() {
        steps.push(Step::tool_call(&format!("a{i}"), "echo", args));
    }
    steps.push(Step::text("ok"));
    let h = harness(steps, |b| b.tool(echo(&runs)).middleware(RepeatGuard::new()));
    let (_, events) = run(&h.agent, "go");
    assert!(events.iter().all(|e| !matches!(e, Event::ToolEnd { output, .. } if output.contains("in a row"))));

    let runs = Arc::new(AtomicUsize::new(0));
    let mut steps: Vec<Step> = Vec::new();
    for run_no in 0..2 {
        for i in 0..2 {
            steps.push(Step::tool_call(&format!("r{run_no}-{i}"), "echo", "{}"));
        }
        steps.push(Step::text("ok"));
    }
    let h = harness(steps, |b| b.tool(echo(&runs)).middleware(RepeatGuard::new()));
    run(&h.agent, "first").0.unwrap();
    let (_, events) = run(&h.agent, "second");
    assert!(events.iter().all(|e| !matches!(e, Event::ToolEnd { output, .. } if output.contains("in a row"))), "streaks do not carry across messages");
}

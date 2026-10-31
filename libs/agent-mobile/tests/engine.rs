//! Plays the part of the app: answers the engine's model and tool requests and checks a whole run.

use agent_mobile::{Engine, Polled};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn temp_dir(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("agent-mobile-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().into_owned()
}

fn config(dir: &str) -> String {
    json!({
        "model": "standard",
        "sessions_dir": dir,
        "system_prompt": "你是小伴",
        "tools": [{
            "name": "set_reminder",
            "description": "设一个提醒",
            "parameters": { "type": "object", "properties": { "title": { "type": "string" }, "minutes": { "type": "integer" } }, "required": ["title", "minutes"] },
            "risk": "write"
        }]
    })
    .to_string()
}

fn next(engine: &Engine, within: Duration) -> Value {
    let deadline = Instant::now() + within;
    loop {
        match engine.poll(Duration::from_millis(200)) {
            Polled::Line(line) => return serde_json::from_str(&line).unwrap(),
            Polled::Closed => panic!("the engine closed"),
            Polled::Timeout => assert!(Instant::now() < deadline, "no output in time"),
        }
    }
}

fn next_of(engine: &Engine, kind: &str) -> Value {
    for _ in 0..200 {
        let line = next(engine, Duration::from_secs(10));
        if line["type"] == kind {
            return line;
        }
    }
    panic!("never saw {kind}");
}

fn reply_with(engine: &Engine, id: &str, sse: &[String]) {
    for line in sse {
        engine.send(&json!({ "type": "model_line", "id": id, "line": line }).to_string()).unwrap();
    }
    engine.send(&json!({ "type": "model_end", "id": id, "status": 200 }).to_string()).unwrap();
}

fn data(chunk: Value) -> Vec<String> {
    vec![format!("data: {chunk}"), String::new()]
}

fn answer_text(engine: &Engine, request: &Value, text: &str) {
    let mut sse = data(json!({ "choices": [{ "index": 0, "delta": { "content": text }, "finish_reason": "stop" }] }));
    sse.extend(["data: [DONE]".to_string(), String::new()]);
    reply_with(engine, request["id"].as_str().unwrap(), &sse);
}

fn ask_tool(engine: &Engine, request: &Value, name: &str, arguments: Value, call_id: &str) {
    let mut sse = data(json!({ "choices": [{ "index": 0, "delta": { "tool_calls": [{ "index": 0, "id": call_id, "type": "function",
        "function": { "name": name, "arguments": arguments.to_string() } }] }, "finish_reason": "tool_calls" }] }));
    sse.extend(["data: [DONE]".to_string(), String::new()]);
    reply_with(engine, request["id"].as_str().unwrap(), &sse);
}

#[test]
fn a_run_calls_a_host_tool_after_the_user_agrees_and_answers_in_text() {
    let dir = temp_dir("tool");
    let engine = Engine::start(&config(&dir)).unwrap();
    assert_eq!(next(&engine, Duration::from_secs(5))["type"], "hello");

    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "十分钟后提醒我喝水" }).to_string()).unwrap();

    // 1st model call: the model asks for the tool.
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    assert_eq!(body["model"], "standard");
    assert_eq!(body["stream"], true);
    assert!(body["tools"].to_string().contains("set_reminder"), "the host tool is offered to the model: {body}");
    let call = json!({ "choices": [{ "index": 0, "delta": { "tool_calls": [{ "index": 0, "id": "c1", "type": "function",
        "function": { "name": "set_reminder", "arguments": "{\"title\":\"喝水\",\"minutes\":10}" } }] }, "finish_reason": "tool_calls" }] });
    let mut sse = data(call);
    sse.extend(["data: [DONE]".to_string(), String::new()]);
    reply_with(&engine, request["id"].as_str().unwrap(), &sse);

    // The tool changes something, so the user is asked first.
    let approval = next_of(&engine, "approval_request");
    assert_eq!(approval["tool"], "set_reminder");
    engine.send(&json!({ "type": "approval", "id": approval["id"], "decision": "once" }).to_string()).unwrap();

    let tool = next_of(&engine, "tool_request");
    assert_eq!(tool["name"], "set_reminder");
    assert_eq!(tool["arguments"]["title"], "喝水");
    engine.send(&json!({ "type": "tool_result", "id": tool["id"], "ok": true, "output": "已设置：10 分钟后提醒“喝水”" }).to_string()).unwrap();

    // 2nd model call sees the tool result and answers.
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    assert!(body["messages"].to_string().contains("已设置"), "the tool result goes back to the model");
    let mut sse = data(json!({ "choices": [{ "index": 0, "delta": { "content": "好的，" } }] }));
    sse.extend(data(json!({ "choices": [{ "index": 0, "delta": { "content": "10 分钟后叫你。" }, "finish_reason": "stop" }] })));
    sse.extend(["data: [DONE]".to_string(), String::new()]);
    reply_with(&engine, request["id"].as_str().unwrap(), &sse);

    let end = next_of(&engine, "run_end");
    assert_eq!(end["ok"], true, "{end}");
    assert_eq!(end["text"], "好的，10 分钟后叫你。");

    engine.stop();
    // The conversation was written to disk for the next time.
    assert!(std::fs::read_dir(&dir).unwrap().count() > 0);
}

#[test]
fn a_denied_tool_is_not_run_and_the_model_is_told() {
    let dir = temp_dir("deny");
    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "提醒我" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    let call = json!({ "choices": [{ "index": 0, "delta": { "tool_calls": [{ "index": 0, "id": "c1", "type": "function",
        "function": { "name": "set_reminder", "arguments": "{\"title\":\"x\",\"minutes\":1}" } }] }, "finish_reason": "tool_calls" }] });
    let mut sse = data(call);
    sse.extend(["data: [DONE]".to_string(), String::new()]);
    reply_with(&engine, request["id"].as_str().unwrap(), &sse);
    let approval = next_of(&engine, "approval_request");
    engine.send(&json!({ "type": "approval", "id": approval["id"], "decision": "deny" }).to_string()).unwrap();

    // No tool_request follows: the next thing is another model call that carries the refusal.
    let line = loop {
        let line = next(&engine, Duration::from_secs(10));
        assert_ne!(line["type"], "tool_request", "a denied tool must not run");
        if line["type"] == "model_request" {
            break line;
        }
    };
    let mut sse = data(json!({ "choices": [{ "index": 0, "delta": { "content": "好的，不设了。" }, "finish_reason": "stop" }] }));
    sse.extend(["data: [DONE]".to_string(), String::new()]);
    reply_with(&engine, line["id"].as_str().unwrap(), &sse);
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();
}

#[test]
fn a_failing_model_ends_the_run_with_the_servers_message() {
    let dir = temp_dir("fail");
    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "你好" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    engine
        .send(&json!({ "type": "model_end", "id": request["id"], "status": 402, "body": "{\"error\":{\"message\":\"额度不足\"}}" }).to_string())
        .unwrap();
    let end = next_of(&engine, "run_end");
    assert_eq!(end["ok"], false);
    assert!(end["error"].as_str().unwrap().contains("额度不足"), "{end}");
    engine.stop();
}

#[test]
fn cancelling_a_run_stops_the_model_request() {
    let dir = temp_dir("cancel");
    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "写一篇长文" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    engine.send(&json!({ "type": "cancel", "id": "r1" }).to_string()).unwrap();
    let cancel = next_of(&engine, "model_cancel");
    assert_eq!(cancel["id"], request["id"]);
    let end = next_of(&engine, "run_end");
    assert_eq!(end["ok"], false);
    engine.stop();
}

#[test]
fn bad_config_and_bad_lines_are_refused_without_crashing() {
    assert!(Engine::start("not json").is_err());
    assert!(Engine::start(&json!({ "model": "m", "sessions_dir": "x", "tools": [{ "name": "t", "description": "d", "parameters": {}, "risk": "wild" }] }).to_string()).is_err());

    let dir = temp_dir("bad");
    let engine: Arc<Engine> = Engine::start(&config(&dir)).unwrap();
    assert!(engine.send("not json").is_err());
    engine.send(&json!({ "type": "set_model", "model": "fast" }).to_string()).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "hi" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    assert_eq!(body["model"], "fast", "set_model applies from the next request");
    engine.stop();
    // After stopping: one closed line, then nothing.
    let mut saw_closed = false;
    for _ in 0..50 {
        match engine.poll(Duration::from_millis(100)) {
            Polled::Line(line) if line.contains("closed") => saw_closed = true,
            Polled::Closed if saw_closed => return,
            _ => {}
        }
    }
    assert!(saw_closed, "a stopped engine says so once");
}

#[test]
fn without_a_model_the_request_leaves_the_field_out() {
    let dir = temp_dir("nomodel");
    let engine = Engine::start(&json!({ "sessions_dir": dir }).to_string()).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "hi" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    assert!(body.get("model").is_none(), "{body}");
    engine.stop();
}

#[test]
fn todo_progress_and_handoff_are_restored_after_restarting_the_engine() {
    let dir = temp_dir("progress");
    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "game", "input": "做一个小游戏" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    ask_tool(&engine, &request, "todo", json!({
        "todos": [{ "content": "实现触控", "status": "in_progress", "evidence": "index.html 已创建" }],
        "handoff": "下一步检查双指触控"
    }), "plan");
    let request = next_of(&engine, "model_request");
    answer_text(&engine, &request, "需要你确认触控方式吗？");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();

    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r2", "session": "game", "input": "继续" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    let checkpoint = body["messages"].as_array().unwrap().iter().find(|m| m["content"].as_str().is_some_and(|s| s.starts_with("任务进度记录"))).expect("the independent progress checkpoint is injected");
    assert!(checkpoint["content"].as_str().unwrap().contains("下一步检查双指触控"));
    assert!(checkpoint["content"].as_str().unwrap().contains("index.html 已创建"));
    answer_text(&engine, &request, "已恢复进度。");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);

    engine.send(&json!({ "type": "run", "id": "r3", "session": "other", "input": "你好" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    assert!(!body["messages"].to_string().contains("双指触控"), "progress is isolated by session");
    answer_text(&engine, &request, "你好。");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();
}

#[test]
fn repeated_identical_host_calls_are_warned_about_and_then_blocked() {
    let dir = temp_dir("repeat");
    let mut settings: Value = serde_json::from_str(&config(&dir)).unwrap();
    settings["tools"][0]["risk"] = json!("read");
    let engine = Engine::start(&settings.to_string()).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "检查提醒状态" }).to_string()).unwrap();
    let mut request = next_of(&engine, "model_request");
    for count in 1..=6 {
        ask_tool(&engine, &request, "set_reminder", json!({ "title": "相同", "minutes": 1 }), &format!("c{count}"));
        if count <= 5 {
            let tool = next_of(&engine, "tool_request");
            engine.send(&json!({ "type": "tool_result", "id": tool["id"], "ok": true, "output": "same state" }).to_string()).unwrap();
        }
        loop {
            let line = next(&engine, Duration::from_secs(10));
            assert_ne!(line["type"], "tool_request", "the sixth identical call must be blocked");
            if line["type"] == "model_request" {
                request = line;
                break;
            }
        }
        let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
        if count == 3 {
            assert!(body["messages"].to_string().contains("same result 3 times"));
        }
        if count == 6 {
            assert!(body["messages"].to_string().contains("Not run:"));
        }
    }
    answer_text(&engine, &request, "状态没有变化，已停止重复检查。");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();
}

#[test]
fn empty_answers_pause_after_three_continuations_instead_of_finishing_successfully() {
    let dir = temp_dir("no-progress");
    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "继续制作" }).to_string()).unwrap();
    for _ in 0..4 {
        let request = next_of(&engine, "model_request");
        answer_text(&engine, &request, "");
    }
    let end = next_of(&engine, "run_end");
    assert_eq!(end["ok"], false);
    assert!(end["error"].as_str().unwrap().contains("no_progress"), "{end}");
    // A real new user message may resume the same session after the pause.
    engine.send(&json!({ "type": "run", "id": "r2", "session": "s1", "input": "先给我当前结果" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    answer_text(&engine, &request, "当前没有生成作品。");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();
}

#[test]
fn ask_user_accepts_answers_and_dismissal_through_the_existing_stdio_protocol() {
    let dir = temp_dir("question");
    let engine = Engine::start(&config(&dir)).unwrap();
    for (id, answer, expected) in [("r1", Some("竖屏"), "The user answered: 竖屏"), ("r2", None, "dismissed the question")] {
        engine.send(&json!({ "type": "run", "id": id, "session": "s1", "input": "设计游戏" }).to_string()).unwrap();
        let request = next_of(&engine, "model_request");
        ask_tool(&engine, &request, "ask_user", json!({ "question": "使用什么方向？", "options": ["竖屏", "横屏"] }), id);
        let question = next_of(&engine, "question");
        assert_eq!(question["session"], "s1");
        assert_eq!(question["question"], "使用什么方向？");
        assert_eq!(question["options"], json!(["竖屏", "横屏"]));
        engine.send(&json!({ "type": "answer", "id": question["id"], "answer": answer }).to_string()).unwrap();
        let request = next_of(&engine, "model_request");
        let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
        assert!(body["messages"].to_string().contains(expected), "{body}");
        answer_text(&engine, &request, "已收到。");
        assert_eq!(next_of(&engine, "run_end")["ok"], true);
    }
    engine.stop();
}

#[test]
fn cancelling_while_waiting_for_an_answer_releases_the_question_and_session() {
    let dir = temp_dir("question-cancel");
    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "设计游戏" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    ask_tool(&engine, &request, "ask_user", json!({ "question": "选哪种玩法？" }), "q1");
    let question = next_of(&engine, "question");
    engine.send(&json!({ "type": "cancel", "id": "r1" }).to_string()).unwrap();
    let end = next_of(&engine, "run_end");
    assert_eq!(end["ok"], false);
    assert!(end["error"].as_str().unwrap().contains("cancel"), "{end}");
    engine.send(&json!({ "type": "answer", "id": question["id"], "answer": "迟到的回答" }).to_string()).unwrap();
    assert!(next_of(&engine, "error")["message"].as_str().unwrap().contains("no open question"));
    engine.send(&json!({ "type": "run", "id": "r2", "session": "s1", "input": "先不用做了" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    answer_text(&engine, &request, "好的。");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();
}

#[test]
fn transient_model_failures_stop_after_two_retries() {
    let dir = temp_dir("retry");
    let engine = Engine::start(&config(&dir)).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "你好" }).to_string()).unwrap();
    for _ in 0..3 {
        let request = next_of(&engine, "model_request");
        engine.send(&json!({ "type": "model_end", "id": request["id"], "status": 503, "body": "temporarily unavailable" }).to_string()).unwrap();
    }
    let end = next_of(&engine, "run_end");
    assert_eq!(end["ok"], false);
    assert!(end["error"].as_str().unwrap().contains("HTTP 503"), "{end}");
    engine.stop();
}

#[test]
fn text_and_image_parts_reach_the_host_model_request_without_changing_jni() {
    let dir = temp_dir("image");
    let engine = Engine::start(&config(&dir)).unwrap();
    let image_url = "data:image/png;base64,iVBORw0KGgo=";
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "parts": [
        { "type": "text", "text": "参考这个画面" }, { "type": "image", "url": image_url }
    ] }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    let user = body["messages"].as_array().unwrap().iter().find(|m| m["role"] == "user").unwrap();
    assert_eq!(user["content"][0], json!({ "type": "text", "text": "参考这个画面" }));
    assert_eq!(user["content"][1], json!({ "type": "image_url", "image_url": { "url": image_url } }));
    answer_text(&engine, &request, "参考图已收到。");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();
}

#[test]
fn time_now_uses_the_phones_optional_utc_offset() {
    let dir = temp_dir("timezone");
    let engine = Engine::start(&json!({ "sessions_dir": dir, "utc_offset_minutes": 480 }).to_string()).unwrap();
    engine.send(&json!({ "type": "run", "id": "r1", "session": "s1", "input": "现在几点" }).to_string()).unwrap();
    let request = next_of(&engine, "model_request");
    ask_tool(&engine, &request, "time_now", json!({}), "time");
    let request = next_of(&engine, "model_request");
    let body: Value = serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
    assert!(body["messages"].to_string().contains("UTC+08:00"), "{body}");
    answer_text(&engine, &request, "这是本地时间。");
    assert_eq!(next_of(&engine, "run_end")["ok"], true);
    engine.stop();
}

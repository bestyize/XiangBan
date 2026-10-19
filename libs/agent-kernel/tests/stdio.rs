#![cfg(feature = "stdio")]
mod common;

use agent_kernel::builtin::{AskUser, Todo};
use agent_kernel::provider::scripted::{ScriptedProvider, Step};
use agent_kernel::stdio::StdioBridge;
use agent_kernel::*;
use common::*;
use serde_json::{json, Value};
use std::io::{self, BufReader, Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct ChanReader {
    rx: Receiver<Vec<u8>>,
    pending: Vec<u8>,
}

impl Read for ChanReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pending.is_empty() {
            match self.rx.recv() {
                Ok(chunk) => self.pending = chunk,
                Err(_) => return Ok(0),
            }
        }
        let n = buf.len().min(self.pending.len());
        buf[..n].copy_from_slice(&self.pending[..n]);
        self.pending.drain(..n);
        Ok(n)
    }
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct FailAfter {
    left: usize,
}

impl Write for FailAfter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.left == 0 {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"));
        }
        self.left -= 1;
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct Wire {
    tx: Sender<Vec<u8>>,
    out: Capture,
    server: JoinHandle<io::Result<()>>,
    provider: Arc<ScriptedProvider>,
}

fn slow_tool(name: &str, ms: u64) -> FnTool {
    FnTool::new(spec(name), move |_, _| {
        thread::sleep(Duration::from_millis(ms));
        Ok("slept".into())
    })
}

fn waiting_tool(name: &str) -> FnTool {
    FnTool::new(spec(name), |_, ctx| {
        while !ctx.is_cancelled() {
            thread::sleep(Duration::from_millis(10));
        }
        Ok("stopped".into())
    })
}

fn start(steps: Vec<Step>, mode: PermissionMode, extra: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Wire {
    let out = Capture::default();
    start_with(Box::new(out.clone()), out, steps, mode, extra)
}

fn start_with(sink: Box<dyn Write + Send>, out: Capture, steps: Vec<Step>, mode: PermissionMode, extra: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Wire {
    struct Boxed(Box<dyn Write + Send>);
    impl Write for Boxed {
        fn write(&mut self, d: &[u8]) -> io::Result<usize> {
            self.0.write(d)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }
    let bridge = StdioBridge::new(Boxed(sink));
    let provider = Arc::new(ScriptedProvider::new(steps));
    let permission = Arc::new(Permission::new(mode, bridge.approver()));
    let builder = Agent::builder(provider.clone(), Arc::new(MemoryStore::new()))
        .tool(AskUser::new(bridge.ask_handler()))
        .tool(Todo::new())
        .tool(FnTool::new(spec("save").risk(Risk::Write), |_, _| Ok("saved".into())))
        .middleware(permission);
    let agent = Arc::new(extra(builder).build().unwrap());
    let (tx, rx) = mpsc::channel();
    let server = thread::spawn(move || bridge.serve(agent, BufReader::new(ChanReader { rx, pending: Vec::new() })));
    Wire { tx, out, server, provider }
}

impl Wire {
    fn send(&self, value: Value) {
        self.send_raw(&format!("{value}\n"));
    }

    fn send_raw(&self, text: &str) {
        self.tx.send(text.as_bytes().to_vec()).unwrap();
    }

    fn lines(&self) -> Vec<Value> {
        let bytes = self.out.0.lock().unwrap().clone();
        String::from_utf8(bytes).unwrap().lines().map(|l| serde_json::from_str(l).expect(l)).collect()
    }

    fn wait_for(&self, what: &str, pred: impl Fn(&Value) -> bool) -> Value {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(found) = self.lines().into_iter().find(|v| pred(v)) {
                return found;
            }
            assert!(Instant::now() < end, "timed out waiting for {what}; got {:#?}", self.lines());
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_type(&self, kind: &'static str) -> Value {
        self.wait_for(kind, move |v| v["type"] == kind)
    }

    fn run_end(&self, run: &str) -> Value {
        let run = run.to_string();
        self.wait_for("run_end", move |v| v["type"] == "run_end" && v["run"] == run.as_str())
    }

    fn finish(self) -> (io::Result<()>, Vec<Value>) {
        drop(self.tx);
        let result = self.server.join().unwrap();
        let bytes = self.out.0.lock().unwrap().clone();
        let lines = String::from_utf8(bytes).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        (result, lines)
    }
}

fn run_msg(id: &str, session: &str, input: &str) -> Value {
    json!({"type": "run", "id": id, "session": session, "input": input})
}

#[test]
fn a_run_streams_events_and_ends_with_a_summary() {
    let w = start(vec![Step::chunks(&["Hel", "lo"]).with_usage(7, 2)], PermissionMode::Auto, |b| b);
    w.send(run_msg("r1", "s1", "hi"));
    let end = w.run_end("r1");
    let (result, lines) = w.finish();
    assert!(result.is_ok());
    assert_eq!(lines[0], json!({"type": "hello", "protocol": 1, "capabilities":["run_events","tool_call_budget"]}));
    assert_eq!(lines[1], json!({"type": "event", "run": "r1", "event": {"type": "text_delta", "text": "Hel"}}));
    assert!(lines.contains(&json!({"type": "event", "run": "r1", "event": {"type": "done"}})));
    assert_eq!(end, json!({"type": "run_end", "run": "r1", "ok": true, "text": "Hello", "turns": 1, "finish": "stop", "prompt_tokens": 7, "completion_tokens": 2}));
}

#[test]
fn observed_mode_exposes_run_identity_states_and_a_tool_budget_without_changing_legacy_frames() {
    let w = start(vec![Step::tool_call("c", "save", "{}")], PermissionMode::Auto, |b| b);
    w.send(json!({"type":"run","id":"outer","session":"s","input":"go","observed":true,"max_tool_calls":0}));
    let end=w.run_end("outer");
    assert_eq!(end["ok"],false);
    assert!(end["error"].as_str().unwrap().contains("tool calls"));
    let (_,lines)=w.finish();
    assert!(!lines.iter().any(|line|line["type"]=="event"));
    let records:Vec<_>=lines.iter().filter(|line|line["type"]=="run_event").map(|line|&line["record"]).collect();
    assert_eq!(records.first().unwrap()["state"],"running");
    assert_eq!(records.last().unwrap()["state"],"paused");
    for (i,record) in records.iter().enumerate() {
        assert_eq!(record["sequence"],i as u64+1);
        assert_eq!(record["run_id"],end["run_id"]);
    }
    assert!(!records.iter().any(|record|record["event"]["type"]=="tool_dispatched"));
}

#[test]
fn multi_part_input_reaches_the_model_and_failures_are_reported() {
    let w = start(vec![Step::text("seen"), Step::error(HarnessError::provider("model is down", false))], PermissionMode::Auto, |b| b);
    w.send(json!({"type": "run", "id": "r1", "session": "s1", "parts": [{"type": "text", "text": "look"}, {"type": "image", "url": "data:image/png;base64,AA"}]}));
    assert_eq!(w.run_end("r1")["ok"], true);
    w.send(run_msg("r2", "s1", "again"));
    let failed = w.run_end("r2");
    assert_eq!((failed["ok"].clone(), failed["error"].clone()), (json!(false), json!("provider error: model is down")));
    let sent = w.provider.requests()[0].messages[0].parts.clone();
    assert_eq!(sent, vec![Part::Text { text: "look".into() }, Part::Image { url: "data:image/png;base64,AA".into() }]);
    w.finish().0.unwrap();
}

#[test]
fn bad_input_gets_an_error_line_and_the_server_keeps_going() {
    let w = start(vec![Step::text("fine")], PermissionMode::Auto, |b| b);
    let bad = [
        "this is not json\n".to_string(),
        "{\"type\":\"teleport\"}\n".to_string(),
        "{\"type\":\"run\",\"id\":\"x\",\"session\":\"s\"}\n".to_string(),
        "{\"type\":\"run\",\"id\":\"x\",\"session\":\"s\",\"input\":\"a\",\"parts\":[{\"type\":\"text\",\"text\":\"b\"}]}\n".to_string(),
        "{\"type\":\"run\",\"id\":\"x\",\"session\":\"s\",\"parts\":[]}\n".to_string(),
        "{\"type\":\"run\",\"id\":\"\",\"session\":\"s\",\"input\":\"a\"}\n".to_string(),
        format!("{{\"type\":\"run\",\"id\":\"x\",\"session\":\"{}\",\"input\":\"a\"}}\n", "s".repeat(257)),
        "{\"type\":\"approval\",\"id\":\"nope\",\"decision\":\"once\"}\n".to_string(),
        "{\"type\":\"answer\",\"id\":\"nope\",\"answer\":null}\n".to_string(),
        "\n   \n".to_string(),
    ];
    for line in &bad {
        w.send_raw(line);
    }
    w.send_raw(&format!("{}\n", "x".repeat(9 * 1024 * 1024)));
    w.send(run_msg("ok", "s1", "hello"));
    assert_eq!(w.run_end("ok")["text"], "fine");
    let (result, lines) = w.finish();
    assert!(result.is_ok());
    let errors: Vec<String> = lines.iter().filter(|v| v["type"] == "error").map(|v| v["message"].as_str().unwrap().to_string()).collect();
    assert_eq!(errors.len(), 10, "nine malformed lines plus the oversized one: {errors:#?}");
    assert!(errors.iter().any(|e| e.contains("line too long")) && errors.iter().any(|e| e.contains("no open approval request")));
}

#[test]
fn approvals_round_trip_and_dismissal_is_a_denial() {
    let steps = vec![Step::tool_call("a", "save", "{}"), Step::tool_call("b", "save", "{}"), Step::text("ok")];
    let w = start(steps, PermissionMode::Ask, |b| b);
    w.send(run_msg("r1", "s1", "go"));
    let first = w.wait_type("approval_request");
    assert_eq!(first["tool"], "save");
    assert_eq!((first["risk"].clone(), first["session"].clone(), first["tainted"].clone()), (json!("write"), json!("s1"), json!(false)));
    w.send(json!({"type": "approval", "id": first["id"], "decision": "maybe"}));
    w.wait_for("unknown decision error", |v| v["type"] == "error" && v["message"].as_str().is_some_and(|m| m.contains("unknown decision")));
    w.send(json!({"type": "approval", "id": first["id"], "decision": "once"}));
    let second = w.wait_for("second request", |v| v["type"] == "approval_request" && v["id"] != first["id"]);
    w.send(json!({"type": "approval", "id": second["id"], "decision": "deny"}));
    assert_eq!(w.run_end("r1")["ok"], true);
    let tool_ends: Vec<Value> = w.lines().into_iter().filter(|v| v["event"]["type"] == "tool_end").map(|v| v["event"]["output"].clone()).collect();
    assert_eq!(tool_ends, vec![json!("saved"), json!("The user denied permission to run `save`.")]);
    w.finish().0.unwrap();
}

#[test]
fn questions_round_trip_with_answers_and_dismissals() {
    let args = r#"{"question":"Which colour?","options":["red","blue"]}"#;
    let steps = vec![Step::tool_call("a", "ask_user", args).and_tool_call("b", "ask_user", args), Step::text("ok")];
    let w = start(steps, PermissionMode::Auto, |b| b);
    w.send(run_msg("r1", "s1", "go"));
    let q = w.wait_type("question");
    assert_eq!((q["question"].clone(), q["options"].clone(), q["session"].clone()), (json!("Which colour?"), json!(["red", "blue"]), json!("s1")));
    w.send(json!({"type": "answer", "id": q["id"], "answer": "blue"}));
    let q2 = w.wait_for("second question", |v| v["type"] == "question" && v["id"] != q["id"]);
    w.send(json!({"type": "answer", "id": q2["id"], "answer": null}));
    assert_eq!(w.run_end("r1")["ok"], true);
    let outputs: Vec<Value> = w.lines().into_iter().filter(|v| v["event"]["type"] == "tool_end").map(|v| v["event"]["output"].clone()).collect();
    assert_eq!(outputs[0], json!("The user answered: blue"));
    assert!(outputs[1].as_str().unwrap().contains("dismissed"));
    w.finish().0.unwrap();
}

#[test]
fn cancel_stops_a_run_and_resolves_its_open_approval() {
    let w = start(vec![Step::tool_call("a", "save", "{}"), Step::text("never")], PermissionMode::Ask, |b| b);
    w.send(run_msg("r1", "s1", "go"));
    w.wait_type("approval_request");
    w.send(json!({"type": "cancel", "id": "r1"}));
    let end = w.run_end("r1");
    assert_eq!((end["ok"].clone(), end["error"].clone()), (json!(false), json!("cancelled")));
    w.send(json!({"type": "cancel", "id": "r1"}));
    w.send(json!({"type": "cancel", "id": "never-existed"}));
    w.send(run_msg("r2", "s1", "again"));
    w.run_end("r2");
    let (result, lines) = w.finish();
    assert!(result.is_ok());
    assert!(lines.iter().all(|v| v["type"] != "error"), "cancelling an ended or unknown run is silent");
}

#[test]
fn cancel_reaches_a_running_tool_and_the_run_id_can_be_reused() {
    let w = start(vec![Step::tool_call("a", "wait", "{}"), Step::text("second run")], PermissionMode::Auto, |b| b.tool(waiting_tool("wait")));
    w.send(run_msg("r1", "s1", "go"));
    w.wait_for("tool start", |v| v["event"]["type"] == "tool_start");
    w.send(json!({"type": "cancel", "id": "r1"}));
    assert_eq!(w.run_end("r1")["error"], "cancelled");
    w.send(run_msg("r1", "s2", "reuse the id"));
    let again = w.wait_for("second run_end", |v| v["type"] == "run_end" && v["ok"] == true);
    assert_eq!(again["text"], "second run");
    w.finish().0.unwrap();
}

#[test]
fn runs_proceed_in_parallel_but_ids_and_sessions_are_exclusive() {
    let steps = vec![Step::tool_call("a", "nap", "{}"), Step::tool_call("b", "nap", "{}"), Step::text("one"), Step::text("two")];
    let w = start(steps, PermissionMode::Auto, |b| b.tool(slow_tool("nap", 300)));
    let started = Instant::now();
    w.send(run_msg("r1", "s1", "go"));
    w.send(run_msg("r2", "s2", "go"));
    w.send(run_msg("r1", "s3", "duplicate id"));
    w.send(run_msg("r3", "s1", "same session"));
    assert_eq!(w.run_end("r1")["ok"], true);
    assert_eq!(w.run_end("r2")["ok"], true);
    assert!(started.elapsed() < Duration::from_millis(900), "{:?}", started.elapsed());
    let busy = w.run_end("r3");
    assert_eq!(busy["ok"], false);
    assert!(busy["error"].as_str().unwrap().contains("already has a run in progress"));
    let (_, lines) = w.finish();
    assert!(lines.iter().any(|v| v["type"] == "error" && v["message"].as_str().unwrap().contains("`r1` is already in use")));
}

#[test]
fn closing_the_input_denies_open_requests_and_lets_the_run_finish() {
    let w = start(vec![Step::tool_call("a", "save", "{}"), Step::text("done anyway")], PermissionMode::Ask, |b| b);
    w.send(run_msg("r1", "s1", "go"));
    w.wait_type("approval_request");
    let (result, lines) = w.finish();
    assert!(result.is_ok());
    let end = lines.iter().find(|v| v["type"] == "run_end").unwrap();
    assert_eq!(end["text"], "done anyway");
    let denied = lines.iter().find(|v| v["event"]["type"] == "tool_end").unwrap();
    assert!(denied["event"]["output"].as_str().unwrap().contains("denied"));
}

#[test]
fn a_closed_output_cancels_runs_and_serve_reports_it() {
    let out = Capture::default();
    let steps = vec![Step::tool_call("a", "wait", "{}"), Step::text("this write fails")];
    let w = start_with(Box::new(FailAfter { left: 2 }), out, steps, PermissionMode::Auto, |b| b.tool(waiting_tool("wait")));
    w.send(run_msg("r1", "s1", "go"));
    thread::sleep(Duration::from_millis(300));
    w.send(run_msg("r2", "s2", "this run's first event cannot be delivered"));
    thread::sleep(Duration::from_millis(300));
    w.send_raw("{\"type\":\"cancel\",\"id\":\"wake-up\"}\n");
    let started = Instant::now();
    let (result, _) = w.finish();
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::BrokenPipe);
    assert!(started.elapsed() < Duration::from_secs(3), "the waiting tool was cancelled instead of hanging the server: {:?}", started.elapsed());
}

#[test]
fn a_cut_off_answer_is_reported_in_events_and_run_end() {
    let w = start(vec![Step::text("cut").finish(FinishReason::Length)], PermissionMode::Auto, |b| b);
    w.send(run_msg("r1", "s1", "hi"));
    let end = w.run_end("r1");
    assert_eq!((end["ok"].clone(), end["finish"].clone(), end["text"].clone()), (json!(true), json!("length"), json!("cut")));
    assert!(w.lines().contains(&json!({"type": "event", "run": "r1", "event": {"type": "truncated"}})));
    w.finish().0.unwrap();
}

#[test]
fn cancel_closes_a_pending_question_promptly() {
    let args = r#"{"question":"Still there?"}"#;
    let w = start(vec![Step::tool_call("a", "ask_user", args), Step::text("never")], PermissionMode::Auto, |b| b);
    w.send(run_msg("r1", "s1", "go"));
    w.wait_type("question");
    let started = Instant::now();
    w.send(json!({"type": "cancel", "id": "r1"}));
    let end = w.run_end("r1");
    assert_eq!((end["ok"].clone(), end["error"].clone()), (json!(false), json!("cancelled")));
    assert!(started.elapsed() < Duration::from_secs(2), "{:?}", started.elapsed());
    w.finish().0.unwrap();
}

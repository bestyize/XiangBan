mod common;

use agent_kernel::builtin::Subagent;
use agent_kernel::provider::scripted::{ScriptedProvider, Step};
use agent_kernel::*;
use common::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Child {
    agent: Arc<Agent>,
    provider: Arc<ScriptedProvider>,
    store: Arc<MemoryStore>,
}

fn child(steps: Vec<Step>, configure: impl FnOnce(AgentBuilder) -> AgentBuilder) -> Child {
    let provider = Arc::new(ScriptedProvider::new(steps));
    let store = Arc::new(MemoryStore::new());
    let agent = Arc::new(configure(Agent::builder(provider.clone(), store.clone()).system_prompt("You are a researcher.")).build().unwrap());
    Child { agent, provider, store }
}

fn delegate(c: &Child, configure: impl FnOnce(Subagent) -> Subagent) -> Subagent {
    configure(Subagent::new("task", "Delegate a job to a researcher.", c.agent.clone(), Risk::Read))
}

fn call_task(sub: Subagent, args: &str) -> (Harness, String) {
    let h = harness(vec![Step::tool_call("c", "task", args), Step::text("ok")], |b| b.tool(sub));
    assert!(run(&h.agent, "go").0.is_ok());
    let out = tool_messages(&h).remove(0).1;
    (h, out)
}

#[test]
fn the_parent_gets_only_the_final_answer() {
    let c = child(vec![Step::tool_call("x", "probe", "{}"), Step::text("The answer is 42.")], |b| b.tool(FnTool::new(spec("probe"), |_, _| Ok("raw data".into()))));
    let (h, out) = call_task(delegate(&c, |s| s), r#"{"task":"  find the answer  "}"#);
    assert_eq!(out, "The answer is 42.");
    let sent = &c.provider.requests()[0].messages;
    assert_eq!((sent[0].text().as_str(), sent[1].text().as_str()), ("You are a researcher.", "find the answer"), "the child sees only its own prompt and the task");
    let ids = c.store.session_ids();
    assert_eq!(ids.len(), 1);
    assert!(ids[0].starts_with("s1#task-"), "{ids:?}");
    let child_history = c.store.load(&ids[0]).unwrap();
    assert_eq!(child_history.len(), 4, "the child's tool round trip lives in the child's session");
    let parent_history = h.store.load("s1").unwrap();
    assert!(parent_history.iter().all(|m| !m.text().contains("raw data")), "the parent's context stays small");
}

#[test]
fn several_tasks_in_one_turn_get_their_own_sessions_and_run_together() {
    let sleeper = |name: &str| FnTool::new(spec(name), |_, _| {
        std::thread::sleep(Duration::from_millis(250));
        Ok("done".into())
    });
    let c = child(
        vec![Step::tool_call("a", "nap", "{}"), Step::tool_call("b", "nap", "{}"), Step::text("first"), Step::text("second")],
        |b| b.tool(sleeper("nap")),
    );
    let steps = vec![Step::tool_call("1", "task", r#"{"task":"one"}"#).and_tool_call("2", "task", r#"{"task":"two"}"#), Step::text("ok")];
    let h = harness(steps, |b| b.tool(delegate(&c, |s| s)));
    let started = Instant::now();
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(started.elapsed() < Duration::from_millis(480), "both children slept at the same time: {:?}", started.elapsed());
    assert_eq!(tool_messages(&h).len(), 2);
    let ids = c.store.session_ids();
    assert_eq!(ids.len(), 2, "{ids:?}");
    assert_ne!(ids[0], ids[1]);
    assert!(ids.iter().all(|id| id.starts_with("s1#task-") && !c.store.load(id).unwrap().is_empty()));
}

#[test]
fn the_observer_sees_child_events_under_the_parent_session() {
    let c = child(vec![Step::chunks(&["par", "tial"])], |b| b);
    let seen: Arc<Mutex<Vec<(String, Event)>>> = Arc::default();
    let log = seen.clone();
    let sub = delegate(&c, |s| s.observe(move |session, event| log.lock().unwrap().push((session.to_string(), event.clone()))));
    call_task(sub, r#"{"task":"t"}"#);
    let seen = seen.lock().unwrap();
    assert!(seen.iter().all(|(s, _)| s == "s1"));
    assert_eq!(seen.iter().filter(|(_, e)| matches!(e, Event::TextDelta { .. })).count(), 2);
    assert_eq!(seen.last().unwrap().1, Event::Done);
}

#[test]
fn child_failures_and_empty_answers_become_tool_results() {
    let c = child(vec![Step::error(HarnessError::provider("model is down", false))], |b| b);
    assert_eq!(call_task(delegate(&c, |s| s), r#"{"task":"t"}"#).1, "sub-agent failed: provider error: model is down");
    let c = child(vec![Step::tool_call("x", "silent", "{}"), Step::tool_call("y", "silent", "{}")], |b| b.tool(FnTool::new(spec("silent"), |_, _| Ok("".into()))).max_turns(1));
    assert!(call_task(delegate(&c, |s| s), r#"{"task":"t"}"#).1.contains("turn limit"));
    let c = child(vec![Step::text("")], |b| b);
    assert!(call_task(delegate(&c, |s| s), r#"{"task":"t"}"#).1.contains("without a text answer"));
}

#[test]
fn invalid_tasks_never_start_the_child() {
    for args in [r#"{"task":"   "}"#.to_string(), serde_json::json!({"task": "x".repeat(20_001)}).to_string(), "{}".to_string()] {
        let c = child(vec![Step::text("should not run")], |b| b);
        assert!(call_task(delegate(&c, |s| s), &args).1.contains("`task`"));
        assert!(c.provider.requests().is_empty());
    }
}

#[test]
fn cancelling_the_parent_cancels_the_child() {
    let stopped = Arc::new(AtomicBool::new(false));
    let flag = stopped.clone();
    let waiter = FnTool::new(spec("wait"), move |_, ctx| {
        while !ctx.is_cancelled() {
            std::thread::sleep(Duration::from_millis(10));
        }
        flag.store(true, Ordering::SeqCst);
        Ok("stopped".into())
    });
    let c = child(vec![Step::tool_call("w", "wait", "{}"), Step::text("never")], |b| b.tool(waiter));
    let h = harness(vec![Step::tool_call("c", "task", r#"{"task":"t"}"#), Step::text("never")], |b| b.tool(delegate(&c, |s| s)));
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = cancel.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        trigger.store(true, Ordering::SeqCst);
    });
    let started = Instant::now();
    let (result, _) = run_with_cancel(&h.agent, "go", &cancel);
    canceller.join().unwrap();
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(3));
    let end = Instant::now() + Duration::from_secs(2);
    while !stopped.load(Ordering::SeqCst) && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(stopped.load(Ordering::SeqCst), "the child's tool saw the cancellation");
}

type AskLog = Arc<Mutex<Vec<(String, String, bool)>>>;

fn permission(asked: &AskLog) -> Arc<Permission> {
    let log = asked.clone();
    Arc::new(Permission::new(PermissionMode::Auto, move |r: &ApprovalRequest<'_>| {
        log.lock().unwrap().push((r.session.to_string(), r.tool.to_string(), r.tainted));
        Approval::Once
    }))
}

#[test]
fn a_shared_permission_gates_the_childs_tools_under_the_child_session() {
    let asked = Arc::default();
    let p = permission(&asked);
    let writer = FnTool::new(spec("save").risk(Risk::Write).always_ask(), |_, _| Ok("saved".into()));
    let c = child(vec![Step::tool_call("x", "save", "{}"), Step::text("done")], |b| b.tool(writer).middleware(p.clone()));
    let (_, out) = call_task(delegate(&c, |s| s), r#"{"task":"t"}"#);
    assert_eq!(out, "done");
    let asked = asked.lock().unwrap();
    assert_eq!(asked.len(), 1);
    assert!(asked[0].0.starts_with("s1#task-") && asked[0].0 != "s1", "the child's own session id: {asked:?}");
    assert_eq!((asked[0].1.as_str(), asked[0].2), ("save", false));
}

#[test]
fn the_answer_taints_the_parent_unless_declared_trusted() {
    for (trusted, expect_tainted) in [(false, true), (true, false)] {
        let asked = Arc::default();
        let p = permission(&asked);
        let c = child(vec![Step::text("summary")], |b| b);
        let sub = delegate(&c, |s| if trusted { s.trusted_output() } else { s });
        let writer = FnTool::new(spec("save").risk(Risk::Write), |_, _| Ok("saved".into()));
        let steps = vec![Step::tool_call("1", "task", r#"{"task":"t"}"#).and_tool_call("2", "save", "{}"), Step::text("ok")];
        let h = harness(steps, |b| b.tool(sub).tool(writer).middleware(p.clone()));
        assert!(run(&h.agent, "go").0.is_ok());
        assert_eq!(p.is_tainted("s1"), expect_tainted, "trusted={trusted}");
        assert_eq!(asked.lock().unwrap().len(), usize::from(expect_tainted), "trusted={trusted}");
    }
}

#[test]
fn the_declared_risk_is_what_the_parent_permission_sees() {
    let c = child(vec![Step::text("x")], |b| b);
    let sub = Subagent::new("task", "d", c.agent.clone(), Risk::Exec);
    let asked = Arc::default();
    let p = Arc::new(Permission::new(PermissionMode::Ask, {
        let log: Arc<Mutex<Vec<(String, Risk)>>> = Arc::clone(&asked);
        move |r: &ApprovalRequest<'_>| {
            log.lock().unwrap().push((r.tool.to_string(), r.risk));
            Approval::Deny
        }
    }));
    let h = harness(vec![Step::tool_call("c", "task", r#"{"task":"t"}"#), Step::text("ok")], |b| b.tool(sub).middleware(p.clone()));
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(*asked.lock().unwrap(), vec![("task".to_string(), Risk::Exec)]);
    assert!(c.provider.requests().is_empty(), "denied: the child never ran");
}

fn persistent_child(dir: &TempDir, reply: &str) -> (Arc<ScriptedProvider>, Subagent) {
    let provider = Arc::new(ScriptedProvider::new(vec![Step::text(reply)]));
    let agent = Arc::new(Agent::builder(provider.clone(), Arc::new(JsonlStore::new(dir.path("child")))).build().unwrap());
    (provider, Subagent::new("task", "d", agent, Risk::Read))
}

fn call_once(sub: Subagent, args: &str) {
    let h = harness(vec![Step::tool_call("c", "task", args), Step::text("ok")], |b| b.tool(sub));
    run(&h.agent, "go").0.unwrap();
}

#[test]
fn a_persistent_child_store_never_mixes_jobs_across_instances_or_restarts() {
    let dir = TempDir::new("sub-persist");
    for (task, reply) in [("FIRST", "one"), ("SECOND", "two"), ("THIRD", "three")] {
        // A new Agent and Subagent each time stands in for a process restart.
        let (provider, sub) = persistent_child(&dir, reply);
        call_once(sub, &serde_json::json!({ "task": task }).to_string());
        let seen = &provider.requests()[0].messages;
        assert_eq!(seen.len(), 1, "{task}: the child saw an earlier job's history");
        assert_eq!(seen[0].text(), task);
    }
    assert_eq!(std::fs::read_dir(dir.path("child")).unwrap().count(), 3, "one transcript per job");
}

#[test]
fn on_finish_reports_each_child_session_even_when_the_job_fails() {
    let ids: Arc<Mutex<Vec<String>>> = Arc::default();
    let log = ids.clone();
    let c = child(vec![Step::text("fine"), Step::error(HarnessError::provider("model is down", false))], |b| b);
    let sub = delegate(&c, |s| s.on_finish(move |id| log.lock().unwrap().push(id.to_string())));
    let steps = vec![Step::tool_call("1", "task", r#"{"task":"a"}"#).and_tool_call("2", "task", r#"{"task":"b"}"#), Step::text("ok")];
    let h = harness(steps, |b| b.tool(sub));
    assert!(run(&h.agent, "go").0.is_ok());
    let mut reported = ids.lock().unwrap().clone();
    reported.sort();
    let mut stored = c.store.session_ids();
    stored.sort();
    assert_eq!(reported.len(), 2);
    assert_ne!(reported[0], reported[1]);
    assert_eq!(reported, stored, "the hook gets exactly the sessions the child ran in");
    assert!(reported.iter().all(|id| id.starts_with("s1#task-")));
}

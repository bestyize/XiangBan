mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::{harness, spec, FnTool};
use serde_json::{json, Value};
use std::sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc, Arc, Mutex};
use std::time::Duration;

// Resource ownership is process-wide. Each test deliberately starts its own competing runs.
static SERIAL: Mutex<()> = Mutex::new(());

fn execute(agent: &Agent, session: &str, control: RunControl, cancel: &Arc<AtomicBool>, event: &mut dyn FnMut(Event)) -> Result<RunSummary, HarnessError> {
    agent.run_parts_observed(session, vec![Part::Text { text: "go".into() }],
        RunOptions { control: Some(control), ..Default::default() }, &mut |record| {
            if let RunEventBody::Event { event: e } = record.body { event(e); }
            Ok(())
        }, cancel)
}

struct HeldTool {
    spec: ToolSpec,
    resource: ToolResource,
    ready: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}
impl Tool for HeldTool {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn resource(&self, _: &Value) -> Result<Option<ToolResource>, String> { Ok(Some(self.resource.clone())) }
    fn execute(&self, _: Value, _: &ToolCtx) -> Result<String, String> {
        self.ready.send(()).unwrap();
        self.release.lock().unwrap().recv_timeout(Duration::from_secs(3)).map_err(|e| e.to_string())?;
        Ok("changed".into())
    }
}
fn owner(resource: ToolResource) -> (std::thread::JoinHandle<Result<RunSummary, HarnessError>>, mpsc::Receiver<()>, mpsc::Sender<()>) {
    let (ready, rx) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let h = harness(vec![Step::tool_call("a", "hold", "{}"), Step::text("done")], |b| b.tool(HeldTool {
        spec: spec("hold").risk(Risk::Write).serial(), resource, ready, release: Mutex::new(wait),
    }));
    let worker = std::thread::spawn(move || execute(&h.agent, "owner", RunControl::unlimited(), &Arc::new(AtomicBool::new(false)), &mut |_| {}));
    (worker, rx, release)
}
fn contender(calls: Arc<AtomicUsize>) -> common::Harness {
    harness(vec![Step::tool_call("b", "change", "{}"), Step::text("done")], |b| b.tool(FnTool::new(
        spec("change").risk(Risk::Write), move |_, _| { calls.fetch_add(1, Ordering::SeqCst); Ok("changed".into()) },
    )))
}

#[test]
fn completed_mutation_releases_ownership_even_while_the_run_budget_is_retained() {
    let _guard = SERIAL.lock().unwrap();
    let h = contender(Arc::new(AtomicUsize::new(0)));
    let control = RunControl::unlimited();
    assert!(execute(&h.agent, "a", control.clone(), &Arc::new(AtomicBool::new(false)), &mut |_| {}).is_ok());
    assert!(RunControl::unlimited().claim().is_ok(), "retaining the budget must not retain a completed tool's lease");
}

#[test]
fn a_short_conflict_waits_then_dispatches_exactly_once_without_another_model_call() {
    let _guard = SERIAL.lock().unwrap();
    let (worker, ready, release) = owner(ToolResource::Global);
    ready.recv_timeout(Duration::from_secs(2)).unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let h = contender(count.clone());
    let mut events = Vec::new();
    let result = execute(&h.agent, "b", RunControl::unlimited(), &Arc::new(AtomicBool::new(false)), &mut |event| {
        if matches!(event, Event::ToolWaiting { waiting: true, .. }) { release.send(()).unwrap(); }
        events.push(event);
    });
    let _ = release.send(());
    assert!(worker.join().unwrap().is_ok());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(h.provider.requests().len(), 2, "waiting does not consume model retries");
    assert!(events.iter().any(|e| matches!(e, Event::ToolWaiting { waiting: true, owner_session: Some(s), .. } if s == "owner")));
    assert!(events.iter().any(|e| matches!(e, Event::ToolWaiting { waiting: false, .. })));
}

#[test]
fn cancelling_a_wait_does_not_dispatch_the_pending_operation() {
    let _guard = SERIAL.lock().unwrap();
    let (worker, ready, release) = owner(ToolResource::Global);
    ready.recv_timeout(Duration::from_secs(2)).unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let h = contender(count.clone());
    let cancel = Arc::new(AtomicBool::new(false));
    let mut dispatched = false;
    let result = execute(&h.agent, "b", RunControl::unlimited(), &cancel, &mut |event| {
        if matches!(event, Event::ToolWaiting { waiting: true, .. }) { cancel.store(true, Ordering::SeqCst); }
        if matches!(event, Event::ToolDispatched { .. }) { dispatched = true; }
    });
    release.send(()).unwrap();
    assert!(worker.join().unwrap().is_ok());
    assert!(matches!(result, Err(HarnessError::Cancelled)), "{result:?}");
    assert_eq!(count.load(Ordering::SeqCst), 0);
    assert!(!dispatched);
}

#[test]
fn waiting_respects_the_run_deadline() {
    let _guard = SERIAL.lock().unwrap();
    let (worker, ready, release) = owner(ToolResource::Global);
    ready.recv_timeout(Duration::from_secs(2)).unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let h = contender(count.clone());
    let result = execute(&h.agent, "b", RunControl::bounded(Duration::from_millis(60), 1000), &Arc::new(AtomicBool::new(false)), &mut |_| {});
    release.send(()).unwrap();
    assert!(worker.join().unwrap().is_ok());
    assert!(matches!(result, Err(HarnessError::BudgetExceeded(_))), "{result:?}");
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[test]
#[cfg(feature = "fs")]
fn app_state_and_computer_ownership_do_not_block_an_unrelated_file() {
    let _guard = SERIAL.lock().unwrap();
    for resource in [ToolResource::Named("app-state".into()), ToolResource::RunExclusive("computer".into())] {
        let (worker, ready, release) = owner(resource);
        ready.recv_timeout(Duration::from_secs(2)).unwrap();
        let dir = common::TempDir::new("independent-file");
        let h = harness(vec![Step::tool_call("f", "write_file", &json!({"path":"result.txt","content":"saved"}).to_string()), Step::text("done")], |mut b| {
            for tool in agent_kernel::builtin::fs::fs_write_tools(vec![dir.0.clone()]).unwrap() { b = b.tool_arc(tool); }
            b
        });
        let result = execute(&h.agent, "writer", RunControl::unlimited(), &Arc::new(AtomicBool::new(false)), &mut |_| {});
        release.send(()).unwrap();
        assert!(worker.join().unwrap().is_ok());
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(std::fs::read_to_string(dir.path("result.txt")).unwrap(), "saved");
    }
}

#[test]
fn computer_control_stays_exclusive_between_actions_and_reports_the_owner() {
    let _guard = SERIAL.lock().unwrap();
    let (worker, ready, release) = owner(ToolResource::RunExclusive("computer".into()));
    ready.recv_timeout(Duration::from_secs(2)).unwrap();
    // Even though the contender uses a call-scoped domain, it conflicts with the run-scoped one.
    let (ready2, _) = mpsc::channel();
    let (_, wait) = mpsc::channel();
    let h = harness(vec![Step::tool_call("b", "hold", "{}")], |b| b.tool(HeldTool {
        spec: spec("hold").risk(Risk::Write), resource: ToolResource::Named("computer".into()), ready: ready2, release: Mutex::new(wait),
    }));
    let result = execute(&h.agent, "b", RunControl::unlimited(), &Arc::new(AtomicBool::new(false)), &mut |_| {});
    release.send(()).unwrap();
    assert!(worker.join().unwrap().is_ok());
    assert!(matches!(result, Err(HarnessError::Paused(ref s)) if s == "resources_busy"));
    let message = h.store.load("b").unwrap().into_iter().find(|m| m.role == Role::Tool).unwrap();
    assert_eq!(message.structured_content.unwrap()["owner_session"], "owner");
}

#[test]
fn an_unconfirmed_worker_blocks_replay_instead_of_waiting_indefinitely() {
    let _guard = SERIAL.lock().unwrap();
    let (ready, rx) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let h = harness(vec![Step::tool_call("a", "hold", "{}")], |b| b.tool(HeldTool {
        spec: spec("hold").risk(Risk::Write).timeout(Duration::from_millis(10)),
        resource: ToolResource::Global, ready, release: Mutex::new(wait),
    }));
    let result = execute(&h.agent, "owner", RunControl::unlimited(), &Arc::new(AtomicBool::new(false)), &mut |_| {});
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(matches!(result, Err(HarnessError::Paused(ref s)) if s == "tool_outcome_needs_review"));
    let count = Arc::new(AtomicUsize::new(0));
    let next = contender(count.clone());
    let result = execute(&next.agent, "b", RunControl::bounded(Duration::from_secs(1), 1000), &Arc::new(AtomicBool::new(false)), &mut |_| {});
    release.send(()).unwrap();
    assert!(matches!(result, Err(HarnessError::Paused(ref s)) if s == "resources_busy"), "{result:?}");
    assert_eq!(count.load(Ordering::SeqCst), 0);
    // The detached worker releases its lease after its actual exit.
    let until = std::time::Instant::now() + Duration::from_secs(2);
    while RunControl::unlimited().claim().is_err() {
        assert!(std::time::Instant::now() < until);
        std::thread::yield_now();
    }
}

struct Deny;
impl Middleware for Deny {
    fn authorizes_tool(&self) -> bool { true }
    fn before_tool(&self, _: &ToolCtx, _: &ToolCall, _: &Value, _: &ToolSpec) -> ToolGate { ToolGate::Block("The user denied permission to run `change`".into()) }
}
#[test]
fn denied_authorization_never_reserves_a_resource_or_runs_mutation_hooks() {
    let _guard = SERIAL.lock().unwrap();
    struct Snapshot(Arc<AtomicUsize>);
    impl Middleware for Snapshot {
        fn before_tool(&self, _: &ToolCtx, _: &ToolCall, _: &Value, _: &ToolSpec) -> ToolGate {
            self.0.fetch_add(1, Ordering::SeqCst); ToolGate::Proceed
        }
    }
    let snapshots = Arc::new(AtomicUsize::new(0));
    let h = harness(vec![Step::tool_call("b", "change", "{}"), Step::text("denied")], |b| b
        .tool(FnTool::new(spec("change").risk(Risk::Write), |_, _| panic!("denied tool ran")))
        .middleware(Snapshot(snapshots.clone())).middleware(Arc::new(Deny)));
    let control = RunControl::unlimited();
    let result = execute(&h.agent, "b", control.clone(), &Arc::new(AtomicBool::new(false)), &mut |_| {});
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(snapshots.load(Ordering::SeqCst), 0);
    assert!(RunControl::unlimited().claim().is_ok());
}

mod common;

use agent_kernel::builtin::AskUser;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn cancel_after(ms: u64) -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = cancel.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(ms));
        trigger.store(true, Ordering::SeqCst);
    });
    cancel
}

fn writer(runs: &Arc<AtomicUsize>) -> FnTool {
    let runs = runs.clone();
    FnTool::new(spec("save").risk(Risk::Write), move |_, _| {
        runs.fetch_add(1, Ordering::SeqCst);
        Ok("saved".into())
    })
}

#[test]
fn an_approval_dialog_waiting_through_wait_for_closes_on_cancel() {
    let runs = Arc::new(AtomicUsize::new(0));
    let saw_cancel = Arc::new(AtomicBool::new(false));
    let seen = saw_cancel.clone();
    let permission = Arc::new(Permission::new(PermissionMode::Ask, move |request: &ApprovalRequest<'_>| {
        let (answer_tx, answer_rx) = mpsc::channel::<Approval>();
        let decision = request.wait_for(&answer_rx).unwrap_or(Approval::Deny);
        seen.store(request.is_cancelled(), Ordering::SeqCst);
        drop(answer_tx);
        decision
    }));
    let h = harness(vec![Step::tool_call("c", "save", "{}"), Step::text("never")], |b| b.tool(writer(&runs)).middleware(permission.clone()));
    let started = Instant::now();
    let (result, events) = run_with_cancel(&h.agent, "go", &cancel_after(200));
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(2), "{:?}", started.elapsed());
    assert!(saw_cancel.load(Ordering::SeqCst) && runs.load(Ordering::SeqCst) == 0);
    assert_eq!(events.last(), Some(&Event::Cancelled));
}

#[test]
fn an_approval_given_after_cancel_is_discarded() {
    let runs = Arc::new(AtomicUsize::new(0));
    let asked = Arc::new(AtomicUsize::new(0));
    let counter = asked.clone();
    let permission = Arc::new(Permission::new(PermissionMode::Ask, move |_: &ApprovalRequest<'_>| {
        counter.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(500));
        Approval::Always
    }));
    let steps = vec![Step::tool_call("c", "save", "{}"), Step::text("never"), Step::tool_call("d", "save", "{}"), Step::text("done")];
    let h = harness(steps, |b| b.tool(writer(&runs)).middleware(permission.clone()));
    let (result, _) = run_with_cancel(&h.agent, "go", &cancel_after(100));
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert_eq!(runs.load(Ordering::SeqCst), 0, "the tool did not run although the approver said yes");
    let history = h.store.load("s1").unwrap();
    assert_eq!(history.last().unwrap().text(), "cancelled", "the call is answered so the session stays consistent");

    let h2 = harness(vec![Step::tool_call("d", "save", "{}"), Step::text("done")], |b| b.tool(writer(&runs)).middleware(permission.clone()));
    let started = Arc::new(AtomicBool::new(false));
    let (result, _) = run_with_cancel(&h2.agent, "again", &started);
    assert!(result.is_ok());
    assert_eq!(asked.load(Ordering::SeqCst), 2, "the discarded answer was not remembered as an always-grant");
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}

#[test]
fn a_tool_never_starts_when_the_run_is_cancelled_while_a_gate_was_deciding() {
    struct SlowGate;
    impl Middleware for SlowGate {
        fn before_tool(&self, _ctx: &ToolCtx, _call: &ToolCall, _args: &serde_json::Value, _spec: &ToolSpec) -> ToolGate {
            std::thread::sleep(Duration::from_millis(300));
            ToolGate::Proceed
        }
    }
    let runs = Arc::new(AtomicUsize::new(0));
    let h = harness(vec![Step::tool_call("c", "save", "{}"), Step::text("never")], |b| b.tool(writer(&runs)).middleware(SlowGate));
    let (result, _) = run_with_cancel(&h.agent, "go", &cancel_after(100));
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

#[test]
fn ask_user_returns_the_answer_dismisses_on_hangup_and_is_interrupted_by_cancel() {
    let answers = Mutex::new(vec!["answered", "hangup", "waits"].into_iter());
    let ask = AskUser::new(move |_, _, ctx| {
        let (tx, rx) = mpsc::channel::<String>();
        match answers.lock().unwrap().next().unwrap() {
            "answered" => {
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(100));
                    let _ = tx.send("yes".to_string());
                });
                ctx.wait_for(&rx)
            }
            "hangup" => {
                drop(tx);
                ctx.wait_for(&rx)
            }
            _ => {
                let answer = ctx.wait_for(&rx);
                drop(tx);
                answer
            }
        }
    });
    let args = r#"{"question":"Proceed?"}"#;
    let steps = vec![
        Step::tool_call("1", "ask_user", args).and_tool_call("2", "ask_user", args),
        Step::tool_call("3", "ask_user", args),
        Step::text("never"),
    ];
    let h = harness(steps, |b| b.tool(ask));
    let started = Instant::now();
    let (result, _) = run_with_cancel(&h.agent, "go", &cancel_after(400));
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    let history = h.store.load("s1").unwrap();
    let results: Vec<String> = history.iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect();
    assert_eq!(results[0], "The user answered: yes");
    assert!(results[1].contains("dismissed"), "{results:?}");
    assert_eq!(results[2], "cancelled", "the interrupted call is answered as cancelled: {results:?}");
}

#[test]
fn a_detached_context_is_never_cancelled_and_waits_for_values() {
    let ctx = ToolCtx::detached("s9");
    assert_eq!(ctx.session(), "s9");
    assert!(!ctx.is_cancelled());
    let (tx, rx) = mpsc::channel();
    tx.send(7).unwrap();
    assert_eq!(ctx.wait_for(&rx), Some(7));
    drop(tx);
    assert_eq!(ctx.wait_for(&rx), None, "a dropped sender ends the wait");
}

#[test]
fn waiting_for_a_person_does_not_spend_the_execution_time_budget() {
    let ask=AskUser::new(|_,_,_| {std::thread::sleep(Duration::from_millis(240));Some("yes".into())});
    assert_eq!(ask.spec().timeout,Duration::MAX);
    let h=harness(vec![Step::tool_call("q","ask_user",r#"{"question":"continue?"}"#),Step::text("done")],|b|b.tool(ask));
    let control=RunControl::bounded(Duration::from_millis(160),1000);
    let result=h.agent.run_parts_observed("s1",vec![Part::Text{text:"go".into()}],RunOptions{control:Some(control.clone()),..Default::default()},&mut |_|Ok(()),&Arc::new(AtomicBool::new(false)));
    assert!(result.is_ok(),"{result:?}");assert!(control.excluded_wait()>=Duration::from_millis(240));
}
#[test]
fn question_checkpoint_failure_is_an_error_not_a_user_dismissal() {
    let ask=AskUser::new_fallible(|_,_,_| Err("cannot persist question".into()));
    let result=ask.execute(serde_json::json!({"question":"q"}),&ToolCtx::detached("s"));
    assert_eq!(result,Err("cannot persist question".into()));
}

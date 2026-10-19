mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct Fixture {
    runs: Arc<AtomicUsize>,
    asked: Arc<Mutex<Vec<(String, String, Risk)>>>,
}

fn tools(fx: &Fixture) -> Vec<FnTool> {
    ["reader", "writer", "runner"]
        .iter()
        .zip([Risk::Read, Risk::Write, Risk::Exec])
        .map(|(name, risk)| {
            let runs = fx.runs.clone();
            FnTool::new(spec(name).risk(risk), move |_, _| {
                runs.fetch_add(1, Ordering::SeqCst);
                Ok("ran".into())
            })
        })
        .collect()
}

fn setup(mode: PermissionMode, answers: Vec<Approval>, customize: impl FnOnce(Permission) -> Permission) -> (Fixture, Arc<Permission>) {
    let fx = Fixture { runs: Arc::new(AtomicUsize::new(0)), asked: Arc::new(Mutex::new(Vec::new())) };
    let (asked, queue) = (fx.asked.clone(), Mutex::new(answers.into_iter()));
    let approver = move |req: &ApprovalRequest<'_>| {
        asked.lock().unwrap().push((req.session.to_string(), req.tool.to_string(), req.risk));
        queue.lock().unwrap().next().unwrap_or(Approval::Deny)
    };
    (fx, Arc::new(customize(Permission::new(mode, approver))))
}

fn run_calls(fx: &Fixture, permission: &Arc<Permission>, session: &str, calls: &[&str]) -> Vec<String> {
    let mut first = Step::tool_call("c0", calls[0], "{}");
    for (i, name) in calls.iter().enumerate().skip(1) {
        first = first.and_tool_call(&format!("c{i}"), name, "{}");
    }
    let h = harness(vec![first, Step::text("ok")], |mut b| {
        for t in tools(fx) {
            b = b.tool(t);
        }
        b.middleware(permission.clone())
    });
    let mut events = Vec::new();
    h.agent.run(session, "go", &mut |e| events.push(e), &Arc::new(Default::default())).unwrap();
    tool_messages(&h).into_iter().map(|(_, text)| text).collect()
}

#[test]
fn read_only_mode_blocks_everything_but_reads() {
    let (fx, p) = setup(PermissionMode::ReadOnly, vec![], |p| p);
    let out = run_calls(&fx, &p, "s1", &["reader", "writer", "runner"]);
    assert_eq!(out[0], "ran");
    assert!(out[1].contains("Read-only mode") && out[2].contains("Read-only mode"));
    assert_eq!(fx.runs.load(Ordering::SeqCst), 1);
    assert!(fx.asked.lock().unwrap().is_empty(), "read-only never asks");
}

#[test]
fn ask_mode_asks_for_writes_and_honours_the_answer() {
    let (fx, p) = setup(PermissionMode::Ask, vec![Approval::Once, Approval::Deny], |p| p);
    let out = run_calls(&fx, &p, "s1", &["reader", "writer", "runner"]);
    assert_eq!(out[0], "ran");
    assert_eq!(out[1], "ran");
    assert!(out[2].contains("denied permission"));
    let asked = fx.asked.lock().unwrap().clone();
    assert_eq!(asked, vec![("s1".into(), "writer".into(), Risk::Write), ("s1".into(), "runner".into(), Risk::Exec)]);
}

#[test]
fn always_is_remembered_per_session_and_never_for_exec() {
    let (fx, p) = setup(PermissionMode::Ask, vec![Approval::Always, Approval::Always, Approval::Always, Approval::Once], |p| p);
    run_calls(&fx, &p, "s1", &["writer", "runner"]);
    run_calls(&fx, &p, "s1", &["writer", "runner"]);
    let asked: Vec<String> = fx.asked.lock().unwrap().iter().map(|a| format!("{}:{}", a.0, a.1)).collect();
    assert_eq!(asked, vec!["s1:writer", "s1:runner", "s1:runner"], "writer asked once; exec asked every time");
    run_calls(&fx, &p, "s2", &["writer"]);
    assert_eq!(fx.asked.lock().unwrap().last().unwrap().0, "s2", "grants do not leak across sessions");
    p.forget_session("s1");
    run_calls(&fx, &p, "s1", &["writer"]);
    assert_eq!(fx.asked.lock().unwrap().last().unwrap().1, "writer", "forgotten sessions ask again");
}

#[test]
fn allow_deny_lists_and_auto_mode() {
    let (fx, p) = setup(PermissionMode::Ask, vec![], |p| p.allow_tool("writer").deny_tool("runner"));
    let out = run_calls(&fx, &p, "s1", &["writer", "runner"]);
    assert_eq!(out[0], "ran");
    assert!(out[1].contains("disabled by policy"));
    assert!(fx.asked.lock().unwrap().is_empty());

    p.set_mode(PermissionMode::Auto);
    assert_eq!(p.mode(), PermissionMode::Auto);
    let (fx, p) = setup(PermissionMode::Auto, vec![], |p| p.deny_tool("runner"));
    let out = run_calls(&fx, &p, "s1", &["writer", "runner"]);
    assert_eq!(out[0], "ran");
    assert!(out[1].contains("disabled by policy"), "deny wins even in auto mode");
}

fn net_tool(name: &str, risk: Risk, untrusted: bool, always_ask: bool, runs: &Arc<AtomicUsize>) -> FnTool {
    let mut s = spec(name).risk(risk);
    if untrusted {
        s = s.untrusted_output();
    }
    if always_ask {
        s = s.always_ask();
    }
    if name == "own" {
        s = s.self_managed();
    }
    let runs = runs.clone();
    FnTool::new(s, move |_, _| {
        runs.fetch_add(1, Ordering::SeqCst);
        Ok("ran".into())
    })
}

fn run_set(p: &Arc<Permission>, session: &str, calls: &[&str], runs: &Arc<AtomicUsize>) -> Vec<String> {
    let mut first = Step::tool_call("c0", calls[0], "{}");
    for (i, name) in calls.iter().enumerate().skip(1) {
        first = first.and_tool_call(&format!("c{i}"), name, "{}");
    }
    let h = harness(vec![first, Step::text("ok")], |b| {
        b.tool(net_tool("fetch", Risk::Network, true, false, runs))
            .tool(net_tool("writer", Risk::Write, false, false, runs))
            .tool(net_tool("saver", Risk::Write, false, true, runs))
            .tool(net_tool("own", Risk::Write, false, false, runs))
            .tool(net_tool("reader", Risk::Read, false, false, runs))
            .middleware(p.clone())
    });
    h.agent.run(session, "go", &mut |_| {}, &Arc::new(Default::default())).unwrap();
    tool_messages(&h).into_iter().map(|(_, t)| t).collect()
}

#[test]
fn reading_external_content_ends_auto_approval_for_that_session_only() {
    let runs = Arc::new(AtomicUsize::new(0));
    let (fx, p) = setup(PermissionMode::Auto, vec![Approval::Deny, Approval::Once], |p| p);
    let out = run_set(&p, "s1", &["writer", "fetch", "writer"], &runs);
    assert_eq!(out[0], "ran", "before any external content, auto mode does not ask");
    assert_eq!(out[1], "ran");
    assert!(out[2].contains("denied"), "after the fetch the same call needs approval: {out:?}");
    assert!(p.is_tainted("s1") && !p.is_tainted("s2"));
    let asked = fx.asked.lock().unwrap().clone();
    assert_eq!(asked, vec![("s1".into(), "writer".into(), Risk::Write)]);

    run_set(&p, "s2", &["writer"], &runs);
    assert_eq!(fx.asked.lock().unwrap().len(), 1, "another session is unaffected");
    p.forget_session("s1");
    assert!(!p.is_tainted("s1"));
}

#[test]
fn taint_overrides_allow_lists_and_remembered_grants_but_not_reads() {
    let runs = Arc::new(AtomicUsize::new(0));
    let (fx, p) = setup(PermissionMode::Ask, vec![Approval::Always, Approval::Once], |p| p.allow_tool("fetch"));
    run_set(&p, "s1", &["writer"], &runs);
    run_set(&p, "s1", &["writer"], &runs);
    assert_eq!(fx.asked.lock().unwrap().len(), 1, "remembered while untainted");
    let out = run_set(&p, "s1", &["fetch", "writer", "reader"], &runs);
    assert_eq!(fx.asked.lock().unwrap().len(), 2, "tainted: asked again despite the remembered grant");
    assert_eq!((out[1].as_str(), out[2].as_str()), ("ran", "ran"));
    assert_eq!(fx.asked.lock().unwrap().last().unwrap().1, "writer", "the allow-listed fetch and the read were not asked");
}

#[test]
fn a_denied_fetch_does_not_taint() {
    let runs = Arc::new(AtomicUsize::new(0));
    let (_, p) = setup(PermissionMode::Ask, vec![Approval::Deny], |p| p);
    let out = run_set(&p, "s1", &["fetch"], &runs);
    assert!(out[0].contains("denied"));
    assert!(!p.is_tainted("s1"));
    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

#[test]
fn always_ask_tools_ask_in_every_mode_and_ignore_remembering() {
    let runs = Arc::new(AtomicUsize::new(0));
    let (fx, p) = setup(PermissionMode::Auto, vec![Approval::Always, Approval::Always, Approval::Deny], |p| p.allow_tool("saver"));
    run_set(&p, "s1", &["saver"], &runs);
    run_set(&p, "s1", &["saver"], &runs);
    let out = run_set(&p, "s1", &["saver"], &runs);
    assert_eq!(fx.asked.lock().unwrap().len(), 3, "asked every time even in auto mode with an allow-list entry");
    assert!(out[0].contains("denied"));
    assert_eq!(runs.load(Ordering::SeqCst), 2);

    p.set_mode(PermissionMode::ReadOnly);
    assert!(run_set(&p, "s1", &["saver"], &runs)[0].contains("Read-only"));
}

#[test]
fn approval_requests_report_the_taint() {
    let runs = Arc::new(AtomicUsize::new(0));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let p = Arc::new(Permission::new(PermissionMode::Auto, move |r: &ApprovalRequest<'_>| {
        log.lock().unwrap().push((r.tool.to_string(), r.tainted));
        Approval::Once
    }));
    run_set(&p, "s1", &["saver", "fetch", "writer"], &runs);
    assert_eq!(*seen.lock().unwrap(), vec![("saver".to_string(), false), ("writer".to_string(), true)]);
}

fn flag_saver_and_runner(name: &str, _: &serde_json::Value) -> Option<String> {
    (name == "saver").then(|| "writes a thing that must not happen by itself".to_string())
}

#[test]
fn a_danger_check_lets_auto_mode_run_everything_except_what_it_flags() {
    let runs = Arc::new(AtomicUsize::new(0));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let p = Arc::new(
        Permission::new(PermissionMode::Auto, move |r: &ApprovalRequest<'_>| {
            log.lock().unwrap().push((r.tool.to_string(), r.tainted, r.danger.map(str::to_string)));
            Approval::Once
        })
        .danger_check(flag_saver_and_runner),
    );
    // Reading the web taints the session, and `saver` is an `always_ask` tool; with a check installed neither asks by itself.
    let out = run_set(&p, "s1", &["fetch", "writer", "reader", "saver"], &runs);
    assert!(out.iter().all(|text| text == "ran"), "{out:?}");
    assert!(p.is_tainted("s1"), "the taint is still recorded and reported");
    // Only the flagged call asks, and it says why and that the session was tainted.
    assert_eq!(*seen.lock().unwrap(), vec![("saver".to_string(), true, Some("writes a thing that must not happen by itself".to_string()))]);
}

#[test]
fn a_flagged_call_is_asked_every_time_and_a_refusal_stops_it() {
    let runs = Arc::new(AtomicUsize::new(0));
    let (fx, p) = setup(PermissionMode::Auto, vec![Approval::Always, Approval::Always, Approval::Deny], |p| p.danger_check(flag_saver_and_runner));
    run_set(&p, "s1", &["saver"], &runs);
    run_set(&p, "s1", &["saver"], &runs);
    let out = run_set(&p, "s1", &["saver"], &runs);
    assert_eq!(fx.asked.lock().unwrap().len(), 3, "\"always\" is not remembered for a flagged call");
    assert!(out[0].contains("denied"));
    assert_eq!(runs.load(Ordering::SeqCst), 2);
}

#[test]
fn a_danger_check_changes_nothing_outside_auto_mode() {
    let runs = Arc::new(AtomicUsize::new(0));
    let (fx, p) = setup(PermissionMode::Ask, vec![Approval::Once; 4], |p| p.danger_check(|_, _| None));
    run_set(&p, "s1", &["writer"], &runs);
    assert_eq!(fx.asked.lock().unwrap().len(), 1, "ask mode still asks for the write");
    run_set(&p, "s1", &["fetch", "writer"], &runs);
    let asked: Vec<String> = fx.asked.lock().unwrap().iter().map(|a| a.1.clone()).collect();
    assert_eq!(asked, vec!["writer", "fetch", "writer"], "the fetch asks, and the taint it leaves keeps forcing the next write");

    p.set_mode(PermissionMode::ReadOnly);
    assert!(run_set(&p, "s2", &["saver"], &runs)[0].contains("Read-only"));
}

#[test]
fn the_check_sees_the_arguments_and_never_runs_for_reads() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let log = calls.clone();
    let p = Arc::new(
        Permission::new(PermissionMode::Auto, |_: &ApprovalRequest<'_>| Approval::Deny).danger_check(move |name, args| {
            log.lock().unwrap().push((name.to_string(), args.clone()));
            None
        }),
    );
    let runs = Arc::new(AtomicUsize::new(0));
    run_set(&p, "s1", &["reader", "writer"], &runs);
    assert_eq!(*calls.lock().unwrap(), vec![("writer".to_string(), serde_json::json!({}))], "reads are never examined");
}

#[test]
fn self_managed_tools_run_unasked_in_ask_mode_and_after_external_content() {
    let runs = Arc::new(AtomicUsize::new(0));
    let (fx, p) = setup(PermissionMode::Ask, vec![Approval::Once, Approval::Deny], |p| p);
    let out = run_set(&p, "s1", &["own", "fetch", "writer", "own"], &runs);
    assert_eq!((out[0].as_str(), out[3].as_str()), ("ran", "ran"), "the app's own state never asks: {out:?}");
    assert!(p.is_tainted("s1"));
    assert!(out[2].contains("denied"), "the scripted answer denies the user's file write");
    let asked: Vec<String> = fx.asked.lock().unwrap().iter().map(|a| a.1.clone()).collect();
    assert_eq!(asked, vec!["fetch", "writer"], "the web read and the user's file write were asked; the app's own work was not");

    // Read-only still blocks it, and a danger check still asks.
    p.set_mode(PermissionMode::ReadOnly);
    assert!(run_set(&p, "s1", &["own"], &runs)[0].contains("Read-only"));
    let flagged = Arc::new(Permission::new(PermissionMode::Ask, |_: &ApprovalRequest<'_>| Approval::Deny).danger_check(|name, _| (name == "own").then(|| "flagged".to_string())));
    let out = run_set(&flagged, "s2", &["own"], &runs);
    assert!(out[0].contains("denied"), "a flagged call is asked and the refusal holds: {out:?}");
}

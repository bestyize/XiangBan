#![cfg(all(
    feature = "journal",
    feature = "mcp",
    any(feature = "openai", feature = "anthropic", feature = "web")
))]
mod common;
use agent_kernel::provider::scripted::{ScriptedProvider, Step};
use agent_kernel::*;
use common::*;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
#[test]
fn unknown_execution_is_persisted_and_holds_resources_until_worker_exit() {
    let count = Arc::new(AtomicUsize::new(0));
    let n = count.clone();
    let store = Arc::new(MemoryStore::new());
    let agent = Agent::builder(
        Arc::new(ScriptedProvider::new(vec![Step::tool_call(
            "a", "act", "{}",
        )
        .and_tool_call("b", "act", "{}")])),
        store.clone(),
    )
    .tool(FnTool::new(
        spec("act").serial().timeout(Duration::from_millis(10)),
        move |_, _| {
            n.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(650));
            Ok("changed".into())
        },
    ))
    .build()
    .unwrap();
    let dir = std::env::temp_dir().join(format!("uncertain-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let log = journal::JsonlRunLog::new(&dir);
    let mut id = String::new();
    let control = RunControl::bounded(Duration::from_secs(5), 1000);
    let result = agent.run_parts_observed(
        "s",
        vec![Part::Text {
            text: "do it".into(),
        }],
        RunOptions {
            control: Some(control.clone()),
            ..Default::default()
        },
        &mut |r| {
            id = r.run_id.clone();
            log.append(&r)
        },
        &Arc::new(AtomicBool::new(false)),
    );
    assert!(matches!(result, Err(HarnessError::Paused(_))));
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(
        recovery::classify(&log.inspect(&id).unwrap()),
        recovery::RecoveryDecision::NeedsReview
    );
    assert!(store.load("s").unwrap().iter().any(|m| m
        .structured_content
        .as_ref()
        .is_some_and(|v| v["harness_status"] == "outcome_unknown")));
    drop(control);
    assert!(RunControl::bounded(Duration::from_secs(5), 1000)
        .claim()
        .is_err());
    std::thread::sleep(Duration::from_millis(700));
    assert!(RunControl::bounded(Duration::from_secs(5), 1000)
        .claim()
        .is_ok());
    let _ = std::fs::remove_dir_all(dir);
}
struct Stalled;
impl ModelProvider for Stalled {
    fn chat(
        &self,
        _: &ChatRequest,
        _: &mut dyn FnMut(&str),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        while !cancel.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }
        Err(HarnessError::Cancelled)
    }
}
#[test]
fn elapsed_budget_interrupts_stalled_provider_and_is_not_user_cancellation() {
    let agent = Agent::builder(Arc::new(Stalled), Arc::new(MemoryStore::new()))
        .build()
        .unwrap();
    let started = Instant::now();
    let r = agent.run_parts_observed(
        "s",
        vec![Part::Text { text: "x".into() }],
        RunOptions {
            control: Some(RunControl::bounded(Duration::from_millis(50), 1000)),
            ..Default::default()
        },
        &mut |_| Ok(()),
        &Arc::new(AtomicBool::new(false)),
    );
    assert!(matches!(r, Err(HarnessError::BudgetExceeded(_))));
    assert!(started.elapsed() < Duration::from_secs(1));
}
#[test]
fn alternating_failed_strategies_stop_at_no_progress_budget() {
    let steps = (0..10)
        .map(|i| {
            Step::tool_call(
                &format!("c{i}"),
                if i % 2 == 0 { "one" } else { "two" },
                "{}",
            )
        })
        .collect();
    let h = harness(steps, |b| {
        b.tool(FnTool::new(spec("one"), |_, _| Err("unavailable".into())))
            .tool(FnTool::new(spec("two"), |_, _| Err("unavailable".into())))
    });
    let r = h.agent.run_parts_observed(
        "s",
        vec![Part::Text { text: "x".into() }],
        RunOptions {
            max_no_progress_rounds: Some(3),
            ..Default::default()
        },
        &mut |_| Ok(()),
        &Arc::new(AtomicBool::new(false)),
    );
    assert!(matches!(r,Err(HarnessError::Paused(ref s)) if s=="no_progress_tool_rounds"));
}
#[test]
#[cfg(unix)]
fn stdio_handshake_can_be_cancelled() {
    use agent_kernel::mcp::{McpClient, McpConfig};
    let cancel = Arc::new(AtomicBool::new(false));
    let c = cancel.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        c.store(true, Ordering::SeqCst);
    });
    let start = Instant::now();
    assert!(
        McpClient::spawn_cancellable(McpConfig::new("sh", &["-c", "sleep 15"]), cancel).is_err()
    );
    assert!(start.elapsed() < Duration::from_secs(2));
}
#[test]
fn http_initialization_drops_socket_on_cancel() {
    use agent_kernel::mcp::{HttpMcpClient, HttpMcpConfig};
    use std::{io::Read, net::TcpListener};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (ready, rx) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut b = [0; 8192];
        s.read(&mut b).unwrap();
        ready.send(()).unwrap();
        while s.read(&mut b).unwrap() != 0 {}
    });
    let cancel = Arc::new(AtomicBool::new(false));
    let c = cancel.clone();
    let worker = std::thread::spawn(move || {
        HttpMcpClient::connect_cancellable(
            HttpMcpConfig {
                url,
                headers: vec![],
                tool_prefix: String::new(),
                timeout: Duration::from_secs(15),
            },
            None,
            c,
        )
    });
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    cancel.store(true, Ordering::SeqCst);
    assert!(worker.join().unwrap().is_err());
    server.join().unwrap();
}

#[test]
#[cfg(unix)]
fn stdio_tool_discovery_reads_all_pages_and_rejects_repeated_cursors() {
    use agent_kernel::mcp::{McpClient, McpConfig};
    let script = r#"import sys,json
for line in sys.stdin:
 q=json.loads(line)
 if 'id' not in q: continue
 if q['method']=='initialize': r={'protocolVersion':'2024-11-05','capabilities':{}}
 else:
  second='cursor' in q.get('params',{})
  r={'tools':[{'name':'two' if second else 'one','inputSchema':{'type':'object'}}]}
  if not second or sys.argv[1]=='repeat': r['nextCursor']='page2'
 print(json.dumps({'jsonrpc':'2.0','id':q['id'],'result':r}),flush=True)
"#;
    let client =
        McpClient::spawn(McpConfig::new("python3", &["-u", "-c", script, "normal"])).unwrap();
    let mut names = client
        .tools()
        .into_iter()
        .map(|t| t.spec().name.clone())
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(names, vec!["one", "two"]);
    client.close();
    assert!(McpClient::spawn(McpConfig::new("python3", &["-u", "-c", script, "repeat"])).is_err());
}
#[test]
fn manual_network_policy_routes_web_fetch_through_proxy_and_direct_can_override_it() {
    use agent_kernel::builtin::WebFetch;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut buf = [0; 8192];
        let n = s.read(&mut buf).unwrap();
        s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 8\r\nConnection: close\r\n\r\nviaProxy").unwrap();
        String::from_utf8_lossy(&buf[..n]).into_owned()
    });
    agent_kernel::net::set_policy(agent_kernel::net::NetworkPolicy {
        mode: "manual".into(),
        proxy_url: format!("http://{address}"),
    })
    .unwrap();
    let text = WebFetch::new()
        .execute(
            serde_json::json!({"url":"http://93.184.216.34/task"}),
            &ToolCtx::detached("s"),
        )
        .unwrap();
    assert!(text.contains("viaProxy"));
    assert!(server
        .join()
        .unwrap()
        .starts_with("GET http://93.184.216.34/task"));
    agent_kernel::net::set_policy(agent_kernel::net::NetworkPolicy {
        mode: "direct".into(),
        proxy_url: String::new(),
    })
    .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut b = [0; 8192];
        s.read(&mut b).unwrap();
        s.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 6\r\nConnection: close\r\n\r\ndirect").unwrap();
    });
    let text = WebFetch::new()
        .trust_host("127.0.0.1")
        .execute(
            serde_json::json!({"url":format!("http://{address}/task")}),
            &ToolCtx::detached("s"),
        )
        .unwrap();
    assert!(text.contains("direct"));
    server.join().unwrap();
}

#[test]fn delegated_unknown_outcome_halts_parent_batch_even_when_time_budget_expires(){
 use agent_kernel::builtin::Subagent;
 let child=Agent::builder(Arc::new(ScriptedProvider::new(vec![Step::tool_call("inner","slow","{}")])),Arc::new(MemoryStore::new())).tool(FnTool::new(spec("slow").timeout(Duration::from_millis(10)),|_,_|{std::thread::sleep(Duration::from_millis(650));Ok("late action".into())})).build().unwrap();
 let count=Arc::new(AtomicUsize::new(0));let n=count.clone();
 let parent=Agent::builder(Arc::new(ScriptedProvider::new(vec![Step::tool_call("delegate","task",r#"{"task":"act"}"#).and_tool_call("follow","follow","{}")])),Arc::new(MemoryStore::new())).tool(Subagent::new("task","delegate",Arc::new(child),Risk::Exec)).tool(FnTool::new(spec("follow"),move|_,_|{n.fetch_add(1,Ordering::SeqCst);Ok("must not happen".into())})).build().unwrap();
 let result=parent.run_parts_observed("s",vec![Part::Text{text:"act".into()}],RunOptions{control:Some(RunControl::bounded(Duration::from_millis(50),1000)),..Default::default()},&mut |_|Ok(()),&Arc::new(AtomicBool::new(false)));
 assert!(matches!(result,Err(HarnessError::Paused(ref s))if s=="tool_outcome_needs_review"));assert_eq!(count.load(Ordering::SeqCst),0);std::thread::sleep(Duration::from_millis(700));
}

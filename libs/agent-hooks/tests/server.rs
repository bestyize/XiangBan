use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};
use xiaoban_agent_hooks::{
    events::HookDecision,
    receiver::{HookServer, ReceiverConfig},
};
fn post(port: u16, token: &str, event: &str, host: &str, body: &[u8]) -> (u16, Vec<u8>) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(s,"POST /h/{token}/{event} HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();
    s.write_all(body).unwrap();
    let mut out = Vec::new();
    s.read_to_end(&mut out).unwrap();
    let n = out.windows(4).position(|x| x == b"\r\n\r\n").unwrap();
    let status = std::str::from_utf8(&out[..n])
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    (status, out[n + 4..].to_vec())
}
#[tokio::test(flavor = "multi_thread")]
async fn receiver_and_forwarder() {
    let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let capture = log.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .without_time()
        .with_writer(move || CapturedLog(capture.clone()))
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XIAOBAN_HOOK_ENDPOINT", dir.path().join("endpoint.json"));
    let server = HookServer::start(ReceiverConfig {
        permission_timeout: Duration::from_millis(500),
        read_timeout: Duration::from_millis(200),
        ..Default::default()
    })
    .await
    .unwrap();
    let ep = server.endpoint();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.path().join("endpoint.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    #[cfg(windows)]
    {
        let output = std::process::Command::new("icacls.exe")
            .arg(dir.path().join("endpoint.json"))
            .output()
            .unwrap();
        assert!(output.status.success());
        let acl = String::from_utf8_lossy(&output.stdout);
        assert!(
            !acl.contains("(I)"),
            "endpoint ACL must not contain inherited grants"
        );
        assert!(acl.contains("(F)"), "current user must retain full control");
    }
    let mut events = server.events();
    let e = ep.clone();
    let (status, _) =
        tokio::task::spawn_blocking(move || post(e.port, "wrong", "Stop", "localhost", b"{}"))
            .await
            .unwrap();
    assert_eq!(status, 404);
    let e = ep.clone();
    let (status, _) =
        tokio::task::spawn_blocking(move || post(e.port, &e.token, "Stop", "evil.test", b"{}"))
            .await
            .unwrap();
    assert_eq!(status, 403);
    let e = ep.clone();
    let (status, _) = tokio::task::spawn_blocking(move || {
        post(
            e.port,
            &e.token,
            "Stop",
            "localhost",
            &vec![b'a'; 1024 * 1024 + 1],
        )
    })
    .await
    .unwrap();
    assert_eq!(status, 413);
    for (decision, expected) in [
        (
            HookDecision::Allow,
            json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}),
        ),
        (
            HookDecision::Deny {
                message: Some("no".into()),
            },
            json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"deny","message":"no"}}}),
        ),
        (HookDecision::NoOpinion, json!({})),
    ] {
        let e = ep.clone();
        let call = tokio::task::spawn_blocking(move || {
            post(
                e.port,
                &e.token,
                "PermissionRequest",
                "localhost",
                br#"{"session_id":"a","hook_event_name":"PermissionRequest"}"#,
            )
        });
        let ev = events.recv().await.unwrap();
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(
            !call.is_finished(),
            "permission request must remain pending until response"
        );
        server.respond(ev.id, decision);
        let (_, body) = call.await.unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&body).unwrap(), expected);
    }
    let e = ep.clone();
    let call = tokio::task::spawn_blocking(move || {
        post(
            e.port,
            &e.token,
            "PermissionRequest",
            "localhost",
            br#"{"session_id":"a","hook_event_name":"PermissionRequest"}"#,
        )
    });
    let _ = events.recv().await.unwrap();
    let (_, body) = call.await.unwrap();
    assert_eq!(body, b"{}");
    let a = ep.clone();
    let b = ep.clone();
    let first = tokio::task::spawn_blocking(move || {
        post(
            a.port,
            &a.token,
            "PermissionRequest",
            "localhost",
            br#"{"session_id":"one","hook_event_name":"PermissionRequest"}"#,
        )
    });
    let second = tokio::task::spawn_blocking(move || {
        post(
            b.port,
            &b.token,
            "PermissionRequest",
            "localhost",
            br#"{"session_id":"two","hook_event_name":"PermissionRequest"}"#,
        )
    });
    for _ in 0..2 {
        let ev = events.recv().await.unwrap();
        server.respond(
            ev.id,
            if ev.session_id == "one" {
                HookDecision::Allow
            } else {
                HookDecision::Deny { message: None }
            },
        );
    }
    let (_, body) = first.await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&body).unwrap()["hookSpecificOutput"]["decision"]
            ["behavior"],
        "allow"
    );
    let (_, body) = second.await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&body).unwrap()["hookSpecificOutput"]["decision"]
            ["behavior"],
        "deny"
    );
    server.queue_instruction("a", "next".into());
    for (session, active, expected) in [
        ("a", true, json!({})),
        ("b", false, json!({})),
        ("a", false, json!({"decision":"block","reason":"next"})),
        ("a", false, json!({})),
    ] {
        let e = ep.clone();
        let payload =
            json!({"session_id":session,"hook_event_name":"Stop","stop_hook_active":active})
                .to_string();
        let (_, body) = tokio::task::spawn_blocking(move || {
            post(e.port, &e.token, "Stop", "localhost", payload.as_bytes())
        })
        .await
        .unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&body).unwrap(), expected);
        let _ = events.recv().await.unwrap();
    }
    let exe = env!("CARGO_BIN_EXE_xiaoban-hook").to_string();
    let path = dir.path().join("endpoint.json");
    let child = tokio::task::spawn_blocking(move || {
        let mut child = std::process::Command::new(exe)
            .arg("PermissionRequest")
            .env("XIAOBAN_HOOK_ENDPOINT", path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(br#"{"session_id":"a","hook_event_name":"PermissionRequest"}"#)
            .unwrap();
        child.wait_with_output().unwrap()
    });
    let ev = events.recv().await.unwrap();
    server.respond(ev.id, HookDecision::Allow);
    let output = child.await.unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}})
    );
    for (decision, expected) in [
        (
            HookDecision::Deny {
                message: Some("no".into()),
            },
            json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"deny","message":"no"}}}),
        ),
        (HookDecision::NoOpinion, json!({})),
    ] {
        let path = dir.path().join("endpoint.json");
        let child = tokio::task::spawn_blocking(move || {
            let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_xiaoban-hook"))
                .arg("PermissionRequest")
                .env("XIAOBAN_HOOK_ENDPOINT", path)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            child
                .stdin
                .take()
                .unwrap()
                .write_all(br#"{"session_id":"a","hook_event_name":"PermissionRequest"}"#)
                .unwrap();
            child.wait_with_output().unwrap()
        });
        let ev = events.recv().await.unwrap();
        server.respond(ev.id, decision);
        let output = child.await.unwrap();
        assert!(output.status.success() && output.stderr.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            expected
        );
    }
    server.queue_instruction("forwarded", "resume".into());
    let path = dir.path().join("endpoint.json");
    let child = tokio::task::spawn_blocking(move || {
        let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_xiaoban-hook"))
            .arg("Stop")
            .env("XIAOBAN_HOOK_ENDPOINT", path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        c.stdin
            .take()
            .unwrap()
            .write_all(
                br#"{"session_id":"forwarded","hook_event_name":"Stop","stop_hook_active":false}"#,
            )
            .unwrap();
        c.wait_with_output().unwrap()
    });
    let ev = events.recv().await.unwrap();
    assert_eq!(ev.session_id, "forwarded");
    let output = child.await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!({"decision":"block","reason":"resume"})
    );
    let e = ep.clone();
    let slow_body = tokio::task::spawn_blocking(move || {
        let mut s = TcpStream::connect(("127.0.0.1", e.port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        write!(s,"POST /h/{}/Stop HTTP/1.1\r\nHost: localhost\r\nContent-Length: 50\r\nConnection: close\r\n\r\n{{",e.token).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    });
    assert!(slow_body.await.unwrap().starts_with("HTTP/1.1 408"));
    let exe = env!("CARGO_BIN_EXE_xiaoban-hook").to_string();
    let path = dir.path().join("endpoint.json");
    let timed = tokio::task::spawn_blocking(move || {
        let mut child = std::process::Command::new(exe)
            .args(["PermissionRequest", "--wait", "1"])
            .env("XIAOBAN_HOOK_ENDPOINT", path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(br#"{"session_id":"a","hook_event_name":"PermissionRequest"}"#)
            .unwrap();
        child.wait_with_output().unwrap()
    });
    let _ = events.recv().await.unwrap();
    let output = timed.await.unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"{}");
    let e = ep.clone();
    let secret_body=br#"{"session_id":"log-secret-body-marker","hook_event_name":"Notification","notification_type":"agent_needs_input","new_field":{"key":42}}"#;
    let (status, body) = tokio::task::spawn_blocking(move || {
        post(e.port, &e.token, "Notification", "localhost", secret_body)
    })
    .await
    .unwrap();
    assert_eq!(status, 200);
    assert_eq!(body, b"{}");
    let event = events.recv().await.unwrap();
    assert_eq!(
        event.notification_type.as_deref(),
        Some("agent_needs_input")
    );
    assert_eq!(event.extra["new_field"]["key"], 42);
    server.shutdown().await;
    let captured = String::from_utf8(log.lock().unwrap().clone()).unwrap();
    assert!(!captured.contains(&ep.token));
    assert!(!captured.contains("log-secret-body-marker"));
    assert!(!dir.path().join("endpoint.json").exists());
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_xiaoban-hook"))
        .arg("Stop")
        .env("XIAOBAN_HOOK_ENDPOINT", dir.path().join("missing"))
        .output()
        .unwrap();
    assert!(output.status.success() && output.stdout.is_empty() && output.stderr.is_empty());
}

struct CapturedLog(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
impl Write for CapturedLog {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[test]
fn forwarder_closed_port_and_unresponsive_server_are_silent() {
    use std::{
        net::TcpListener,
        process::{Command, Stdio},
        time::Instant,
    };
    let d = tempfile::tempdir().unwrap();
    let endpoint = d.path().join("endpoint.json");
    let closed = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = closed.local_addr().unwrap().port();
    drop(closed);
    std::fs::write(
        &endpoint,
        json!({"port":port,"token":"abcd","pid":1}).to_string(),
    )
    .unwrap();
    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_xiaoban-hook"))
        .arg("Stop")
        .env("XIAOBAN_HOOK_ENDPOINT", &endpoint)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success() && output.stdout.is_empty() && output.stderr.is_empty());
    assert!(start.elapsed() < Duration::from_millis(700));
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    std::fs::write(
        &endpoint,
        json!({"port":port,"token":"abcd","pid":1}).to_string(),
    )
    .unwrap();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut buf = [0; 1024];
        let _ = socket.read(&mut buf);
        std::thread::sleep(Duration::from_millis(1500));
    });
    let start = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_xiaoban-hook"))
        .args(["PermissionRequest", "--wait", "1"])
        .env("XIAOBAN_HOOK_ENDPOINT", &endpoint)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"{}").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success() && output.stdout.is_empty() && output.stderr.is_empty());
    assert!(start.elapsed() < Duration::from_millis(1400));
    server.join().unwrap();
}
#[test]
fn forwarder_bounds_stdin_and_uses_deadline() {
    use std::process::{Command, Stdio};
    let d = tempfile::tempdir().unwrap();
    let endpoint = d.path().join("endpoint.json");
    std::fs::write(
        &endpoint,
        json!({"port":1,"token":"abcd","pid":1}).to_string(),
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_xiaoban-hook"))
        .arg("Stop")
        .env("XIAOBAN_HOOK_ENDPOINT", &endpoint)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&vec![b'a'; 1024 * 1024 + 1]).unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success() && output.stdout.is_empty() && output.stderr.is_empty());
    let mut child = Command::new(env!("CARGO_BIN_EXE_xiaoban-hook"))
        .args(["PermissionRequest", "--wait", "1"])
        .env("XIAOBAN_HOOK_ENDPOINT", &endpoint)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let held_stdin = child.stdin.take().unwrap();
    let start = std::time::Instant::now();
    let output = child.wait_with_output().unwrap();
    drop(held_stdin);
    assert!(output.status.success() && output.stdout.is_empty() && output.stderr.is_empty());
    assert!(start.elapsed() < Duration::from_millis(1400));
}

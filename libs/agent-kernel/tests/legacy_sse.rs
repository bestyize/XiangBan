#![cfg(all(
    feature = "mcp",
    any(feature = "openai", feature = "anthropic", feature = "web")
))]
mod legacy_sse_support;
use agent_kernel::{
    mcp::{HttpAuthorization, HttpMcpConfig, SseMcpClient},
    ToolCtx,
};
use legacy_sse_support::{Options, Server};
use serde_json::json;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

fn config(server: &Server, timeout: Duration) -> HttpMcpConfig {
    HttpMcpConfig {
        url: server.url(),
        headers: vec![("Authorization".into(), "Bearer secret".into())],
        tool_prefix: "legacy_".into(),
        timeout,
    }
}
fn until(check: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !check() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn handshake_relative_and_absolute_endpoints_pagination_and_multiline_data() {
    for absolute in [false, true] {
        let server = Server::new(Options {
            absolute,
            token: Some("secret".into()),
            ..Default::default()
        });
        let client = SseMcpClient::connect(config(&server, Duration::from_secs(3))).unwrap();
        let tools = client.tools();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].spec().name, "legacy_read");
        assert!(tools[0].spec().always_ask && tools[0].spec().untrusted_output);
        assert_eq!(
            tools[0]
                .execute(json!({"text":"hello"}), &ToolCtx::detached("call"))
                .unwrap(),
            "hello"
        );
        until(|| {
            server
                .requests
                .lock()
                .unwrap()
                .iter()
                .any(|(_, r)| r["id"] == "server-ping" && r.get("result").is_some())
        });
        client.close();
        until(|| server.stream_closed.load(Ordering::SeqCst));
        assert!(!client.is_connected());
    }
}
#[test]
fn rejects_cross_origin_credentials_and_fragment_endpoints_before_post() {
    for endpoint in [
        "https://other.invalid/rpc",
        "//127.0.0.1:1/rpc",
        "http://user:pass@127.0.0.1/rpc",
        "/rpc#fragment",
    ] {
        let server = Server::new(Options {
            endpoint: Some(endpoint.into()),
            ..Default::default()
        });
        let error = SseMcpClient::connect(config(&server, Duration::from_secs(2)))
            .err()
            .unwrap();
        assert!(error.contains("same-origin"), "{error}");
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}
#[test]
fn concurrent_responses_are_dispatched_out_of_order() {
    let server = Server::new(Options {
        reorder: true,
        ..Default::default()
    });
    let client = SseMcpClient::connect(config(&server, Duration::from_secs(3))).unwrap();
    let tool = client.tools()[0].clone();
    let second = tool.clone();
    let first = std::thread::spawn(move || {
        tool.execute(json!({"text":"first"}), &ToolCtx::detached("one"))
    });
    let other = std::thread::spawn(move || {
        second.execute(json!({"text":"second"}), &ToolCtx::detached("two"))
    });
    assert_eq!(first.join().unwrap().unwrap(), "first");
    assert_eq!(other.join().unwrap().unwrap(), "second");
    client.close();
}
#[test]
fn interrupted_stream_fails_calls_without_reconnect_and_keeps_review_gate() {
    let server = Server::new(Options {
        drop_call: true,
        ..Default::default()
    });
    let client = SseMcpClient::connect(config(&server, Duration::from_secs(3))).unwrap();
    let result = client.tools()[0].execute_result(json!({}), &ToolCtx::detached("drop"));
    assert!(result.is_error);
    assert!(!client.is_connected());
    assert!(client.needs_review());
    assert_eq!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(r, _)| r.starts_with("GET"))
            .count(),
        1
    );
    client.close();
    assert!(!client.needs_review());
}
#[test]
fn timeout_closes_stream_and_unblocks_all_pending_calls() {
    let server = Server::new(Options {
        stall_call: true,
        ..Default::default()
    });
    let client = SseMcpClient::connect(config(&server, Duration::from_millis(500))).unwrap();
    let tool = client.tools()[0].clone();
    let other = tool.clone();
    let start = Instant::now();
    let first = std::thread::spawn(move || {
        tool.execute_result(json!({}), &ToolCtx::detached("timeout-one"))
    });
    let second = std::thread::spawn(move || {
        other.execute_result(json!({}), &ToolCtx::detached("timeout-two"))
    });
    assert!(first.join().unwrap().is_error && second.join().unwrap().is_error);
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(!client.is_connected());
    until(|| server.stream_closed.load(Ordering::SeqCst));
}
#[test]
fn endpoint_wait_honors_startup_cancellation_and_timeout() {
    for cancel_startup in [false, true] {
        let server = Server::new(Options {
            stall_endpoint: true,
            ..Default::default()
        });
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let thread = std::thread::spawn(move || {
            if cancel_startup {
                std::thread::sleep(Duration::from_millis(80));
                flag.store(true, Ordering::SeqCst);
            }
        });
        let start = Instant::now();
        assert!(SseMcpClient::connect_cancellable(
            config(&server, Duration::from_millis(300)),
            None,
            cancel
        )
        .is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
        until(|| server.stream_closed.load(Ordering::SeqCst));
        thread.join().unwrap();
    }
}
#[test]
fn oversized_unterminated_event_disconnects_client() {
    let server = Server::new(Options {
        oversized: true,
        ..Default::default()
    });
    let client = SseMcpClient::connect(config(&server, Duration::from_secs(5))).unwrap();
    let result = client.tools()[0].execute_result(json!({}), &ToolCtx::detached("large"));
    assert!(result.is_error, "{}", result.output);
    assert!(result.output.contains("exceeds 8 MiB"), "{}", result.output);
    assert!(!client.is_connected());
}
struct Auth(AtomicUsize);
impl HttpAuthorization for Auth {
    fn access_token(&self, refresh: bool) -> Result<String, String> {
        if refresh {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
        Ok("secret".into())
    }
}
#[test]
fn get_and_post_401_refresh_once_with_host_authorization() {
    for get in [false, true] {
        let server = Server::new(Options {
            reject_get_once: get,
            reject_post_once: !get,
            token: Some("secret".into()),
            ..Default::default()
        });
        let auth = Arc::new(Auth(AtomicUsize::new(0)));
        let client = SseMcpClient::connect_with_authorization(
            config(&server, Duration::from_secs(3)),
            auth.clone(),
        )
        .unwrap();
        assert_eq!(auth.0.load(Ordering::SeqCst), 1);
        client.close();
    }
}

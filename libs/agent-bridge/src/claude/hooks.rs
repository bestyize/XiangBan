use crate::{classify_risk, AgentEvent, Approval, ApprovalKind, ApprovalOption, Decision, Risk};
use rand::RngCore;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
    time::timeout,
};

pub(super) struct Hooks {
    pub state: Arc<State>,
    pub settings: String,
    task: JoinHandle<()>,
}
pub(super) struct State {
    pub tx: mpsc::UnboundedSender<AgentEvent>,
    pub pending: Mutex<HashMap<String, oneshot::Sender<Decision>>>,
    pub steering: Mutex<VecDeque<String>>,
    pub cwd: PathBuf,
    next: AtomicU64,
    approval_timeout: Mutex<Duration>,
}
impl Drop for Hooks {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Hooks {
    pub async fn start(
        cwd: PathBuf,
        tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let state = Arc::new(State {
            tx,
            pending: Mutex::new(HashMap::new()),
            steering: Mutex::new(VecDeque::new()),
            cwd,
            next: AtomicU64::new(1),
            approval_timeout: Mutex::new(Duration::from_secs(300)),
        });
        let mut all = serde_json::Map::new();
        for name in [
            "PreToolUse",
            "PermissionRequest",
            "Stop",
            "Notification",
            "SessionEnd",
        ] {
            all.insert(name.into(),json!([{"hooks":[{"type":"http","url":format!("http://127.0.0.1:{port}/h/{token}/{name}"),"timeout":if name=="PermissionRequest" || name=="PreToolUse" {600} else {30}}]}]));
        }
        let settings = json!({"hooks":all}).to_string();
        let shared = state.clone();
        let task = tokio::spawn(async move {
            let mut requests = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let Ok((stream, addr)) = accepted else { break };
                        if !addr.ip().is_loopback() { continue; }
                        let state = shared.clone();
                        let token = token.clone();
                        requests.spawn(async move { let _ = handle(stream, state, &token).await; });
                    }
                    _ = requests.join_next(), if !requests.is_empty() => {}
                }
            }
        });
        Ok(Self {
            state,
            settings,
            task,
        })
    }
}
async fn respond(mut stream: TcpStream, status: &str, body: &str) -> std::io::Result<()> {
    let head=format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body.as_bytes()).await
}
async fn handle(mut stream: TcpStream, state: Arc<State>, token: &str) -> std::io::Result<()> {
    let read = timeout(Duration::from_secs(10), async {
        let mut bytes = Vec::new();
        let mut buf = [0u8; 4096];
        let header_end = loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                return Ok(None);
            }
            bytes.extend_from_slice(&buf[..n]);
            if bytes.len() > 16384 {
                return Ok(None);
            }
            if let Some(i) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                break i + 4;
            }
        };
        let header = String::from_utf8_lossy(&bytes[..header_end]);
        let mut lines = header.lines();
        let first = lines.next().unwrap_or("");
        let parts: Vec<_> = first.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(None);
        }
        let path = parts[1].to_owned();
        let len = lines
            .filter_map(|l| l.split_once(':'))
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, v)| v.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if len > 1024 * 1024 {
            return Ok(Some((path, None)));
        }
        while bytes.len() < header_end + len {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                return Ok(None);
            }
            bytes.extend_from_slice(&buf[..n]);
        }
        Ok::<_, std::io::Error>(Some((
            path,
            Some(bytes[header_end..header_end + len].to_vec()),
        )))
    })
    .await;
    let Ok(Ok(Some((path, body)))) = read else {
        return Ok(());
    };
    let prefix = format!("/h/{token}/");
    if !path.starts_with(&prefix) {
        return respond(stream, "404 Not Found", "{}").await;
    }
    let Some(body) = body else {
        return respond(stream, "413 Payload Too Large", "{}").await;
    };
    let Ok(v) = serde_json::from_slice::<Value>(&body) else {
        return respond(stream, "400 Bad Request", "{}").await;
    };
    let event = &path[prefix.len()..];
    let response = match event {
        "Stop" => {
            if v.get("stop_hook_active")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                json!({})
            } else if let Some(next) = state.steering.lock().await.pop_front() {
                json!({"decision":"block","reason":next})
            } else {
                json!({})
            }
        }
        "PreToolUse" | "PermissionRequest" => {
            let name = v.get("tool_name").and_then(Value::as_str).unwrap_or("");
            if event == "PreToolUse" && matches!(name, "Read" | "Glob" | "Grep") {
                json!({})
            } else {
                let id = format!("hook-{}", state.next.fetch_add(1, Ordering::Relaxed));
                let detail = v
                    .get("tool_input")
                    .map(Value::to_string)
                    .unwrap_or_default();
                let kind = if name == "Bash" {
                    ApprovalKind::Command
                } else {
                    ApprovalKind::Permission
                };
                let risk = if kind == ApprovalKind::Permission {
                    Risk::High
                } else {
                    classify_risk(&detail, &state.cwd)
                };
                let (tx, rx) = oneshot::channel();
                state.pending.lock().await.insert(id.clone(), tx);
                let _ = state.tx.send(AgentEvent::ApprovalRequested(Approval {
                    id: id.clone(),
                    kind,
                    title: name.into(),
                    detail,
                    risk,
                    options: vec![
                        ApprovalOption {
                            id: "allow".into(),
                            label: "Allow".into(),
                        },
                        ApprovalOption {
                            id: "deny".into(),
                            label: "Deny".into(),
                        },
                    ],
                }));
                let wait = *state.approval_timeout.lock().await;
                let decision = timeout(wait, rx).await;
                state.pending.lock().await.remove(&id);
                if event == "PreToolUse" {
                    let allow = matches!(
                        decision,
                        Ok(Ok(Decision::Allow | Decision::AllowForSession))
                    );
                    json!({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":if allow {"allow"} else {"deny"}}})
                } else {
                    match decision {
                        Ok(Ok(Decision::Allow | Decision::AllowForSession)) => {
                            json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}})
                        }
                        Ok(Ok(Decision::Deny)) => {
                            json!({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"deny"}}})
                        }
                        _ => json!({}),
                    }
                }
            }
        }
        _ => json!({}),
    };
    respond(stream, "200 OK", &response.to_string()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hook_url(h: &Hooks, name: &str) -> String {
        serde_json::from_str::<Value>(&h.settings).unwrap()["hooks"][name][0]["hooks"][0]["url"]
            .as_str()
            .unwrap()
            .to_owned()
    }
    async fn post(url: &str, body: &str, length: Option<usize>) -> (String, Value) {
        let url = url.strip_prefix("http://").unwrap();
        let (host, path) = url.split_once('/').unwrap();
        let mut stream = TcpStream::connect(host).await.unwrap();
        let bytes=format!("POST /{path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",length.unwrap_or(body.len()));
        stream.write_all(bytes.as_bytes()).await.unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8(response).unwrap();
        let (header, body) = response.split_once("\r\n\r\n").unwrap();
        (
            header.lines().next().unwrap().to_owned(),
            serde_json::from_str(body).unwrap(),
        )
    }
    #[tokio::test]
    async fn hook_binding_wrong_token_and_body_limit() {
        let (tx, _) = mpsc::unbounded_channel();
        let hooks = Hooks::start(std::env::current_dir().unwrap(), tx)
            .await
            .unwrap();
        let url = hook_url(&hooks, "Stop");
        assert!(url.starts_with("http://127.0.0.1:"));
        let bad = url.replace("/h/", "/h/wrong-");
        assert!(post(&bad, "{}", None).await.0.contains("404"));
        assert!(post(&url, "", Some(1024 * 1024 + 1))
            .await
            .0
            .contains("413"));
    }
    #[tokio::test]
    async fn stop_active_does_not_consume_queued_instruction() {
        let (tx, _) = mpsc::unbounded_channel();
        let hooks = Hooks::start(std::env::current_dir().unwrap(), tx)
            .await
            .unwrap();
        hooks
            .state
            .steering
            .lock()
            .await
            .push_back("follow up".into());
        let url = hook_url(&hooks, "Stop");
        assert_eq!(
            post(&url, r#"{"stop_hook_active":true}"#, None).await.1,
            json!({})
        );
        assert_eq!(
            post(&url, r#"{"stop_hook_active":false}"#, None).await.1,
            json!({"decision":"block","reason":"follow up"})
        );
        assert_eq!(
            post(&url, r#"{"stop_hook_active":false}"#, None).await.1,
            json!({})
        );
    }
    #[tokio::test]
    async fn approval_timeout_returns_no_allow_and_removes_pending() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let hooks = Hooks::start(std::env::current_dir().unwrap(), tx)
            .await
            .unwrap();
        *hooks.state.approval_timeout.lock().await = Duration::from_millis(40);
        let url = hook_url(&hooks, "PermissionRequest");
        let pending = tokio::spawn(async move {
            post(
                &url,
                r#"{"tool_name":"Bash","tool_input":{"command":"touch file"}}"#,
                None,
            )
            .await
        });
        assert!(matches!(
            rx.recv().await,
            Some(AgentEvent::ApprovalRequested(_))
        ));
        let (_, body) = pending.await.unwrap();
        assert_eq!(body, json!({}));
        assert!(hooks.state.pending.lock().await.is_empty());
    }
    #[tokio::test]
    async fn cannot_connect_through_non_loopback_interface_when_available() {
        let (tx, _) = mpsc::unbounded_channel();
        let hooks = Hooks::start(std::env::current_dir().unwrap(), tx)
            .await
            .unwrap();
        let url = hook_url(&hooks, "Stop");
        let authority = url
            .strip_prefix("http://")
            .unwrap()
            .split('/')
            .next()
            .unwrap();
        let port = authority
            .rsplit(':')
            .next()
            .unwrap()
            .parse::<u16>()
            .unwrap();
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
        if socket.connect("192.0.2.1:9").is_ok() {
            let addr = socket.local_addr().unwrap().ip();
            if !addr.is_loopback() && !addr.is_unspecified() {
                let connection = timeout(
                    Duration::from_millis(250),
                    TcpStream::connect(std::net::SocketAddr::new(addr, port)),
                )
                .await;
                assert!(!matches!(connection, Ok(Ok(_))));
                return;
            }
        }
        eprintln!("No non-loopback IPv4 interface; connection probe not run");
    }
    #[tokio::test]
    async fn dropping_receiver_aborts_pending_requests() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let hooks = Hooks::start(std::env::current_dir().unwrap(), tx)
            .await
            .unwrap();
        let url = hook_url(&hooks, "PermissionRequest");
        let request = tokio::spawn(async move {
            let parsed = url.strip_prefix("http://").unwrap();
            let (host, path) = parsed.split_once('/').unwrap();
            let mut stream = TcpStream::connect(host).await.unwrap();
            let body = r#"{"tool_name":"Bash"}"#;
            let message = format!(
                "POST /{path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(message.as_bytes()).await.unwrap();
            let mut result = Vec::new();
            let _ = stream.read_to_end(&mut result).await;
            result
        });
        assert!(matches!(
            rx.recv().await,
            Some(AgentEvent::ApprovalRequested(_))
        ));
        drop(hooks);
        let bytes = timeout(Duration::from_secs(1), request)
            .await
            .unwrap()
            .unwrap();
        assert!(bytes.is_empty());
    }
}

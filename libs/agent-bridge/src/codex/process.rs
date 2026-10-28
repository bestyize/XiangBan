use super::protocol;
use crate::{
    AgentEvent, AgentKind, Availability, BridgeError, Decision, ExternalAgent, PermissionProfile,
    Session, SessionInfo, StartRequest,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin},
    sync::{mpsc, oneshot, Mutex},
    time::timeout,
};

/// Codex CLI app-server backend.
#[derive(Clone)]
pub struct Codex {
    executable: PathBuf,
    request_timeout: Duration,
}
impl Default for Codex {
    fn default() -> Self {
        Self::new(crate::process::default_path("codex"))
    }
}
impl Codex {
    /// Use a specified executable (also useful for test doubles).
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            request_timeout: Duration::from_secs(60),
        }
    }
    /// Configure the maximum wait for a protocol response.
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.request_timeout = duration;
        self
    }
    fn profiles_verified() -> bool {
        false
    }
    async fn connect(&self, cwd: PathBuf) -> Result<Connection, BridgeError> {
        let mut cmd = crate::process::command(&self.executable);
        cmd.arg("app-server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BridgeError::NotInstalled
            } else {
                BridgeError::ProcessExited
            }
        })?;
        let stdin = child.stdin.take().ok_or(BridgeError::ProcessExited)?;
        let stdout = child.stdout.take().ok_or(BridgeError::ProcessExited)?;
        let (tx, rx) = mpsc::unbounded_channel();
        let shared = Arc::new(Shared {
            stdin: Mutex::new(Some(stdin)),
            pending: Mutex::new(HashMap::new()),
            approvals: Mutex::new(HashMap::new()),
            turn: Mutex::new(None),
            tx,
            cwd,
            next: AtomicU64::new(1),
            duration: self.request_timeout,
        });
        tokio::spawn(read_loop(stdout, shared.clone()));
        let c = Connection {
            child,
            shared,
            rx: Some(rx),
        };
        c.request("initialize", json!({"clientInfo":{"name":"xiaoban-agent-bridge","title":"Xiaoban Agent Bridge","version":env!("CARGO_PKG_VERSION")}})).await?;
        c.write(json!({"method":"initialized"})).await?;
        Ok(c)
    }
}
type Pending = HashMap<u64, oneshot::Sender<Result<Value, BridgeError>>>;
type Approvals = HashMap<String, (Value, String, Value)>;
struct Shared {
    stdin: Mutex<Option<ChildStdin>>,
    pending: Mutex<Pending>,
    approvals: Mutex<Approvals>,
    turn: Mutex<Option<String>>,
    tx: mpsc::UnboundedSender<AgentEvent>,
    cwd: PathBuf,
    next: AtomicU64,
    duration: Duration,
}
struct Connection {
    child: Child,
    shared: Arc<Shared>,
    rx: Option<mpsc::UnboundedReceiver<AgentEvent>>,
}
impl Connection {
    async fn write(&self, value: Value) -> Result<(), BridgeError> {
        write(&self.shared, value).await
    }
    async fn request(&self, method: &str, params: Value) -> Result<Value, BridgeError> {
        request(&self.shared, method, params).await
    }
    async fn stop(&mut self) -> Result<(), BridgeError> {
        self.shared.stdin.lock().await.take();
        crate::process::shutdown(&mut self.child).await
    }
}
async fn write(s: &Shared, value: Value) -> Result<(), BridgeError> {
    let mut lock = s.stdin.lock().await;
    let stdin = lock.as_mut().ok_or(BridgeError::ProcessExited)?;
    let mut line = serde_json::to_vec(&value).map_err(|_| BridgeError::Protocol)?;
    line.push(b'\n');
    stdin
        .write_all(&line)
        .await
        .map_err(|_| BridgeError::ProcessExited)
}
async fn request(s: &Shared, method: &str, params: Value) -> Result<Value, BridgeError> {
    let id = s.next.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel();
    s.pending.lock().await.insert(id, tx);
    if let Err(e) = write(s, json!({"id":id,"method":method,"params":params})).await {
        s.pending.lock().await.remove(&id);
        return Err(e);
    }
    let result = timeout(s.duration, rx).await;
    s.pending.lock().await.remove(&id);
    match result {
        Err(_) => Err(BridgeError::Timeout),
        Ok(Err(_)) => Err(BridgeError::ProcessExited),
        Ok(Ok(r)) => r,
    }
}
fn string(v: &Value, pointer: &str) -> String {
    v.pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}
fn key(id: &Value) -> String {
    match id {
        Value::String(s) => s.clone(),
        _ => id.to_string(),
    }
}
async fn read_loop(stdout: tokio::process::ChildStdout, s: Arc<Shared>) {
    let mut lines = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            let _ = s.tx.send(AgentEvent::Error {
                code: "protocol".into(),
                message: "Invalid JSON from agent".into(),
            });
            break;
        };
        let method = v.get("method").and_then(Value::as_str);
        let id = v.get("id");
        if let (Some(method), Some(id)) = (method, id) {
            let params = v.get("params").cloned().unwrap_or(json!({}));
            if matches!(
                method,
                "item/commandExecution/requestApproval"
                    | "item/fileChange/requestApproval"
                    | "item/permissions/requestApproval"
                    | "item/tool/requestUserInput"
                    | "mcpServer/elicitation/request"
                    | "applyPatchApproval"
                    | "execCommandApproval"
            ) {
                let k = key(id);
                s.approvals
                    .lock()
                    .await
                    .insert(k.clone(), (id.clone(), method.into(), params.clone()));
                let _ = s.tx.send(AgentEvent::ApprovalRequested(protocol::approval(
                    k, method, &params, &s.cwd,
                )));
            } else {
                let _ = write(
                    &s,
                    json!({"id":id,"error":{"code":-32601,"message":"Unsupported request"}}),
                )
                .await;
            }
        } else if let Some(method) = method {
            notify(&s, method, v.get("params").unwrap_or(&Value::Null)).await;
        } else if let Some(id) = id.and_then(Value::as_u64) {
            if let Some(tx) = s.pending.lock().await.remove(&id) {
                let result = if v.get("error").is_some() {
                    Err(BridgeError::Protocol)
                } else {
                    Ok(v.get("result").cloned().unwrap_or(Value::Null))
                };
                let _ = tx.send(result);
            }
        }
    }
    s.pending.lock().await.clear();
    let _ = s.tx.send(AgentEvent::Closed);
}
async fn notify(s: &Shared, method: &str, p: &Value) {
    let mut events = Vec::new();
    match method {
        "turn/started" => {
            *s.turn.lock().await = Some(string(p, "/turn/id"));
            events.push(AgentEvent::Status { busy: true });
        }
        "turn/completed" => {
            *s.turn.lock().await = None;
            events.push(AgentEvent::Status { busy: false });
            events.push(AgentEvent::TurnCompleted {
                final_text: None,
                usage: None,
            });
        }
        "item/agentMessage/delta" => events.push(AgentEvent::TextDelta {
            text: string(p, "/delta"),
        }),
        "item/started" => {
            let item = &p["item"];
            let kind = string(item, "/type");
            if kind != "agentMessage" {
                events.push(AgentEvent::ToolStarted {
                    id: string(item, "/id"),
                    name: kind,
                    summary: String::new(),
                });
            }
        }
        "item/completed" => {
            let item = &p["item"];
            let kind = string(item, "/type");
            if kind == "agentMessage" {
                events.push(AgentEvent::Message {
                    text: string(item, "/text"),
                });
            } else {
                events.push(AgentEvent::ToolFinished {
                    id: string(item, "/id"),
                    ok: item.get("status").and_then(Value::as_str) != Some("failed"),
                    preview: string(item, "/aggregatedOutput")
                        .chars()
                        .take(2000)
                        .collect(),
                });
            }
        }
        "serverRequest/resolved" => {
            let id = key(&p["requestId"]);
            s.approvals.lock().await.remove(&id);
            events.push(AgentEvent::ApprovalResolved { id });
        }
        "error" => events.push(AgentEvent::Error {
            code: "agent".into(),
            message: "Agent reported an error".into(),
        }),
        _ => {}
    }
    for e in events {
        let _ = s.tx.send(e);
    }
}
#[async_trait]
impl ExternalAgent for Codex {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }
    async fn availability(&self) -> Availability {
        crate::process::probe(&self.executable).await
    }
    async fn list_sessions(&self, limit: usize) -> Result<Vec<SessionInfo>, BridgeError> {
        let mut c = self.connect(PathBuf::new()).await?;
        let result = c.request("thread/list", json!({"limit":limit})).await;
        let _ = c.stop().await;
        let value = result?;
        let data = value
            .get("data")
            .and_then(Value::as_array)
            .ok_or(BridgeError::Protocol)?;
        Ok(data
            .iter()
            .filter_map(|v| {
                Some(SessionInfo {
                    id: v.get("id")?.as_str()?.into(),
                    title: v.get("name").and_then(Value::as_str).map(str::to_owned),
                    cwd: v.get("cwd").and_then(Value::as_str).map(PathBuf::from),
                    updated_at: v.get("updatedAt").and_then(Value::as_i64),
                    kind: AgentKind::Codex,
                })
            })
            .collect())
    }
    async fn start(&self, req: StartRequest) -> Result<Box<dyn Session>, BridgeError> {
        if req.prompt.is_empty() || !req.cwd.is_absolute() {
            return Err(BridgeError::InvalidArgument);
        }
        if !Self::profiles_verified() && !cfg!(test) {
            return Err(BridgeError::Unsupported);
        }
        let c = self.connect(req.cwd.clone()).await?;
        let (policy, sandbox) = protocol::settings(req.profile);
        let r=c.request("thread/start",json!({"cwd":req.cwd,"approvalPolicy":policy,"sandbox":sandbox,"model":req.model,"ephemeral":false})).await?;
        let id = r
            .pointer("/thread/id")
            .and_then(Value::as_str)
            .ok_or(BridgeError::Protocol)?
            .to_owned();
        if let Some(name) = req.name {
            c.request("thread/name/set", json!({"threadId":id,"name":name}))
                .await?;
        }
        let session = CodexSession::new(id, c);
        session.send(&req.prompt).await?;
        Ok(Box::new(session))
    }
    async fn resume(
        &self,
        id: &str,
        cwd: &Path,
        profile: PermissionProfile,
    ) -> Result<Box<dyn Session>, BridgeError> {
        if id.is_empty() || !cwd.is_absolute() {
            return Err(BridgeError::InvalidArgument);
        }
        if !Self::profiles_verified() && !cfg!(test) {
            return Err(BridgeError::Unsupported);
        }
        let c = self.connect(cwd.to_path_buf()).await?;
        let (policy, sandbox) = protocol::settings(profile);
        c.request(
            "thread/resume",
            json!({"threadId":id,"cwd":cwd,"approvalPolicy":policy,"sandbox":sandbox}),
        )
        .await?;
        Ok(Box::new(CodexSession::new(id.into(), c)))
    }
}
struct CodexSession {
    id: String,
    connection: Connection,
}
impl CodexSession {
    fn new(id: String, connection: Connection) -> Self {
        let _ = connection.shared.tx.send(AgentEvent::SessionStarted {
            session: id.clone(),
        });
        Self { id, connection }
    }
}
#[async_trait]
impl Session for CodexSession {
    fn id(&self) -> &str {
        &self.id
    }
    fn events(&mut self) -> mpsc::Receiver<AgentEvent> {
        let (tx, rx) = mpsc::channel(128);
        if let Some(mut source) = self.connection.rx.take() {
            tokio::spawn(async move {
                while let Some(v) = source.recv().await {
                    if tx.send(v).await.is_err() {
                        break;
                    }
                }
            });
        }
        rx
    }
    async fn send(&self, text: &str) -> Result<(), BridgeError> {
        if text.is_empty() {
            return Err(BridgeError::InvalidArgument);
        }
        self.connection
            .request(
                "turn/start",
                json!({"threadId":self.id,"input":[{"type":"text","text":text}]}),
            )
            .await?;
        Ok(())
    }
    async fn steer(&self, text: &str) -> Result<(), BridgeError> {
        let turn = self
            .connection
            .shared
            .turn
            .lock()
            .await
            .clone()
            .ok_or(BridgeError::InvalidArgument)?;
        self.connection.request("turn/steer",json!({"threadId":self.id,"expectedTurnId":turn,"input":[{"type":"text","text":text}]})).await?;
        Ok(())
    }
    async fn interrupt(&self) -> Result<(), BridgeError> {
        let turn = self
            .connection
            .shared
            .turn
            .lock()
            .await
            .clone()
            .ok_or(BridgeError::InvalidArgument)?;
        self.connection
            .request("turn/interrupt", json!({"threadId":self.id,"turnId":turn}))
            .await?;
        Ok(())
    }
    async fn answer(&self, id: &str, decision: Decision) -> Result<(), BridgeError> {
        let mut approvals = self.connection.shared.approvals.lock().await;
        let (original, method, params) = approvals.get(id).ok_or(BridgeError::InvalidArgument)?;
        let result = protocol::response(method, params, decision)?;
        self.connection
            .write(json!({"id":original,"result":result}))
            .await?;
        approvals.remove(id);
        Ok(())
    }
    async fn close(mut self: Box<Self>) -> Result<(), BridgeError> {
        self.connection.stop().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn agent() -> Codex {
        Codex::new(crate::test_support::fake("codex")).with_timeout(Duration::from_secs(2))
    }
    fn request() -> StartRequest {
        StartRequest {
            cwd: std::env::current_dir().unwrap(),
            prompt: "hello".into(),
            profile: PermissionProfile::Strict,
            model: None,
            name: None,
        }
    }
    async fn next(rx: &mut mpsc::Receiver<AgentEvent>) -> AgentEvent {
        timeout(Duration::from_secs(3), rx.recv())
            .await
            .expect("event timed out")
            .expect("event stream closed")
    }
    #[tokio::test]
    async fn fake_codex_start_steer_interrupt_and_decisions() {
        for decision in [Decision::Allow, Decision::Deny] {
            let agent = agent();
            let mut session = agent.start(request()).await.unwrap();
            assert_eq!(session.id(), "test-thread");
            let mut rx = session.events();
            let mut text = false;
            let id = loop {
                match next(&mut rx).await {
                    AgentEvent::TextDelta { text: t } => text = t == "hello",
                    AgentEvent::ApprovalRequested(a) => break a.id,
                    _ => {}
                }
            };
            assert!(text);
            session.steer("extra").await.unwrap();
            session.interrupt().await.unwrap();
            let expected = matches!(decision, Decision::Allow);
            session.answer(&id, decision).await.unwrap();
            let mut actual = None;
            loop {
                match next(&mut rx).await {
                    AgentEvent::ToolFinished { ok, .. } => actual = Some(ok),
                    AgentEvent::TurnCompleted { .. } => break,
                    _ => {}
                }
            }
            assert_eq!(actual, Some(expected));
            session.close().await.unwrap();
            let resumed = agent
                .resume(
                    "old",
                    &std::env::current_dir().unwrap(),
                    PermissionProfile::AskForWrites,
                )
                .await
                .unwrap();
            assert_eq!(resumed.id(), "old");
            resumed.close().await.unwrap();
        }
    }
    #[tokio::test]
    async fn fake_codex_sessions_are_isolated() {
        let agent = agent();
        let mut a = agent.start(request()).await.unwrap();
        let mut b = agent.start(request()).await.unwrap();
        let mut ar = a.events();
        let mut br = b.events();
        for rx in [&mut ar, &mut br] {
            loop {
                if matches!(next(rx).await, AgentEvent::ApprovalRequested(_)) {
                    break;
                }
            }
        }
        a.answer("91", Decision::Allow).await.unwrap();
        b.answer("91", Decision::Deny).await.unwrap();
        for rx in [&mut ar, &mut br] {
            loop {
                if matches!(next(rx).await, AgentEvent::TurnCompleted { .. }) {
                    break;
                }
            }
        }
        a.close().await.unwrap();
        b.close().await.unwrap();
    }
    #[tokio::test]
    async fn fake_codex_exit_invalid_json_and_timeout() {
        let mut request = request();
        request.prompt = "timeout".into();
        let short = agent().with_timeout(Duration::from_millis(150));
        assert!(matches!(
            short.start(request.clone()).await,
            Err(BridgeError::Timeout)
        ));
        request.prompt = "exit".into();
        assert!(matches!(
            short.start(request.clone()).await,
            Err(BridgeError::ProcessExited)
        ));
        request.prompt = "bad-json".into();
        let mut session = short.start(request).await.unwrap();
        let mut rx = session.events();
        loop {
            match next(&mut rx).await {
                AgentEvent::Error { code, .. } => {
                    assert_eq!(code, "protocol");
                    break;
                }
                AgentEvent::Closed => panic!("no protocol error"),
                _ => {}
            }
        }
        session.close().await.unwrap();
    }
    #[tokio::test]
    async fn resolution_from_server_invalidates_pending_approval() {
        let mut req = request();
        req.prompt = "resolved".into();
        let mut session = agent().start(req).await.unwrap();
        let mut rx = session.events();
        loop {
            if let AgentEvent::ApprovalResolved { id } = next(&mut rx).await {
                assert_eq!(id, "91");
                break;
            }
        }
        assert!(matches!(
            session.answer("91", Decision::Allow).await,
            Err(BridgeError::InvalidArgument)
        ));
        session.close().await.unwrap();
    }
    #[tokio::test]
    async fn token_refresh_is_rejected_without_any_credentials() {
        let mut req = request();
        req.prompt = "unsupported".into();
        let mut session = agent().start(req).await.unwrap();
        let mut rx = session.events();
        loop {
            if let AgentEvent::TextDelta { text } = next(&mut rx).await {
                if text == "unsupported-rejected" {
                    break;
                }
            }
        }
        session.answer("91", Decision::Deny).await.unwrap();
        session.close().await.unwrap();
    }
    #[tokio::test]
    async fn close_kills_and_reaps_a_child_that_ignores_eof() {
        let mut req = request();
        req.prompt = "linger".into();
        let session = agent().start(req).await.unwrap();
        let start = std::time::Instant::now();
        timeout(Duration::from_secs(5), session.close())
            .await
            .unwrap()
            .unwrap();
        assert!(start.elapsed() >= Duration::from_millis(1800));
    }
}

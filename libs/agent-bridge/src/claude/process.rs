use super::{hooks::Hooks, stream};
use crate::{
    AgentEvent, AgentKind, Availability, BridgeError, Decision, ExternalAgent, PermissionProfile,
    Session, SessionInfo, StartRequest,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
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

/// Claude Code streaming backend. Its session registry lives only in memory.
#[derive(Clone)]
pub struct ClaudeCode {
    executable: PathBuf,
    timeout: Duration,
    sessions: Arc<Mutex<Vec<SessionInfo>>>,
}
impl Default for ClaudeCode {
    fn default() -> Self {
        Self::new(crate::process::default_path("claude"))
    }
}
impl ClaudeCode {
    /// Use the given executable; primarily useful for a local installation or test double.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            executable: path.into(),
            timeout: Duration::from_secs(60),
            sessions: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// Set the initialization timeout.
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }
    fn hooks_verified() -> bool {
        false
    }
    async fn launch(
        &self,
        cwd: &Path,
        profile: PermissionProfile,
        resume: Option<&str>,
        model: Option<&str>,
        name: Option<&str>,
        prompt: Option<&str>,
    ) -> Result<ClaudeSession, BridgeError> {
        if !cwd.is_absolute() {
            return Err(BridgeError::InvalidArgument);
        }
        // Fail closed until hook enforcement and failure behavior have been verified end to end.
        if !Self::hooks_verified() && !cfg!(test) {
            return Err(BridgeError::Unsupported);
        }
        let (tx, rx) = mpsc::unbounded_channel();
        let hooks = Hooks::start(cwd.to_path_buf(), tx.clone())
            .await
            .map_err(|_| BridgeError::ProcessExited)?;
        let (mode, allowed) = permission_settings(profile);
        let mut cmd = crate::process::command(&self.executable);
        cmd.current_dir(cwd)
            .arg("-p")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--permission-mode")
            .arg(mode)
            .arg("--allowedTools")
            .arg(allowed)
            .arg("--settings")
            .arg(&hooks.settings)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(id) = resume {
            cmd.arg("--resume").arg(id);
        }
        if let Some(model) = model {
            cmd.arg("--model").arg(model);
        }
        if let Some(name) = name {
            cmd.arg("--name").arg(name);
        }
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BridgeError::NotInstalled
            } else {
                BridgeError::ProcessExited
            }
        })?;
        let stdin = child.stdin.take().ok_or(BridgeError::ProcessExited)?;
        let stdout = child.stdout.take().ok_or(BridgeError::ProcessExited)?;
        let busy = Arc::new(AtomicBool::new(false));
        let (init_tx, init_rx) = oneshot::channel();
        tokio::spawn(reader(stdout, tx, busy.clone(), init_tx));
        let stdin = Arc::new(Mutex::new(Some(stdin)));
        if let Some(prompt) = prompt {
            busy.store(true, Ordering::SeqCst);
            send_input(&stdin, prompt).await?;
        }
        let id = if let Some(id) = resume {
            id.to_owned()
        } else {
            match timeout(self.timeout, init_rx).await {
                Ok(Ok(id)) => id,
                Ok(Err(_)) => return Err(BridgeError::ProcessExited),
                Err(_) => return Err(BridgeError::Timeout),
            }
        };
        self.sessions.lock().await.push(SessionInfo {
            id: id.clone(),
            title: name.map(str::to_owned),
            cwd: Some(cwd.to_path_buf()),
            updated_at: None,
            kind: AgentKind::ClaudeCode,
        });
        Ok(ClaudeSession {
            id,
            child: Mutex::new(child),
            stdin,
            busy,
            hooks,
            rx: Some(rx),
        })
    }
}
fn permission_settings(_profile: PermissionProfile) -> (&'static str, &'static str) {
    ("default", "Read,Glob,Grep")
}
async fn reader(
    stdout: tokio::process::ChildStdout,
    tx: mpsc::UnboundedSender<AgentEvent>,
    busy: Arc<AtomicBool>,
    init: oneshot::Sender<String>,
) {
    let mut init = Some(init);
    let mut lines = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            let _ = tx.send(AgentEvent::Error {
                code: "protocol".into(),
                message: "Invalid JSON from agent".into(),
            });
            break;
        };
        if v.get("type").and_then(Value::as_str) == Some("system")
            && v.get("subtype").and_then(Value::as_str) == Some("init")
        {
            if let (Some(id), Some(init)) =
                (v.get("session_id").and_then(Value::as_str), init.take())
            {
                let _ = init.send(id.into());
                let _ = tx.send(AgentEvent::SessionStarted { session: id.into() });
            }
        }
        if v.get("type").and_then(Value::as_str) == Some("result") {
            busy.store(false, Ordering::SeqCst);
            let _ = tx.send(AgentEvent::Status { busy: false });
        }
        for event in stream::parse(&v) {
            let _ = tx.send(event);
        }
    }
    let _ = tx.send(AgentEvent::Closed);
}
async fn send_input(stdin: &Mutex<Option<ChildStdin>>, text: &str) -> Result<(), BridgeError> {
    if text.is_empty() {
        return Err(BridgeError::InvalidArgument);
    }
    let mut guard = stdin.lock().await;
    let input = guard.as_mut().ok_or(BridgeError::ProcessExited)?;
    let mut line = serde_json::to_vec(
        &json!({"type":"user","message":{"role":"user","content":[{"type":"text","text":text}]}}),
    )
    .map_err(|_| BridgeError::Protocol)?;
    line.push(b'\n');
    input
        .write_all(&line)
        .await
        .map_err(|_| BridgeError::ProcessExited)
}
#[async_trait]
impl ExternalAgent for ClaudeCode {
    fn kind(&self) -> AgentKind {
        AgentKind::ClaudeCode
    }
    async fn availability(&self) -> Availability {
        crate::process::probe(&self.executable).await
    }
    async fn list_sessions(&self, limit: usize) -> Result<Vec<SessionInfo>, BridgeError> {
        Ok(self
            .sessions
            .lock()
            .await
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect())
    }
    async fn start(&self, req: StartRequest) -> Result<Box<dyn Session>, BridgeError> {
        if req.prompt.is_empty() {
            return Err(BridgeError::InvalidArgument);
        }
        Ok(Box::new(
            self.launch(
                &req.cwd,
                req.profile,
                None,
                req.model.as_deref(),
                req.name.as_deref(),
                Some(&req.prompt),
            )
            .await?,
        ))
    }
    async fn resume(
        &self,
        id: &str,
        cwd: &Path,
        profile: PermissionProfile,
    ) -> Result<Box<dyn Session>, BridgeError> {
        if id.is_empty() {
            return Err(BridgeError::InvalidArgument);
        }
        Ok(Box::new(
            self.launch(cwd, profile, Some(id), None, None, None)
                .await?,
        ))
    }
}
struct ClaudeSession {
    id: String,
    child: Mutex<Child>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    busy: Arc<AtomicBool>,
    hooks: Hooks,
    rx: Option<mpsc::UnboundedReceiver<AgentEvent>>,
}
#[async_trait]
impl Session for ClaudeSession {
    fn id(&self) -> &str {
        &self.id
    }
    fn events(&mut self) -> mpsc::Receiver<AgentEvent> {
        let (tx, rx) = mpsc::channel(128);
        if let Some(mut source) = self.rx.take() {
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
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err(BridgeError::InvalidArgument);
        }
        if let Err(e) = send_input(&self.stdin, text).await {
            self.busy.store(false, Ordering::SeqCst);
            return Err(e);
        }
        let _ = self.hooks.state.tx.send(AgentEvent::Status { busy: true });
        Ok(())
    }
    async fn steer(&self, text: &str) -> Result<(), BridgeError> {
        if text.is_empty() {
            return Err(BridgeError::InvalidArgument);
        }
        if self.busy.load(Ordering::SeqCst) {
            self.hooks
                .state
                .steering
                .lock()
                .await
                .push_back(text.into());
            Ok(())
        } else {
            self.send(text).await
        }
    }
    async fn interrupt(&self) -> Result<(), BridgeError> {
        self.stdin.lock().await.take();
        let mut child = self.child.lock().await;
        child.start_kill().map_err(|_| BridgeError::ProcessExited)?;
        child.wait().await.map_err(|_| BridgeError::ProcessExited)?;
        self.busy.store(false, Ordering::SeqCst);
        self.hooks.state.pending.lock().await.clear();
        Ok(())
    }
    async fn answer(&self, id: &str, decision: Decision) -> Result<(), BridgeError> {
        if matches!(decision, Decision::Answer(_)) {
            return Err(BridgeError::Unsupported);
        }
        self.hooks
            .state
            .pending
            .lock()
            .await
            .remove(id)
            .ok_or(BridgeError::InvalidArgument)?
            .send(decision)
            .map_err(|_| BridgeError::InvalidArgument)
    }
    async fn close(self: Box<Self>) -> Result<(), BridgeError> {
        self.stdin.lock().await.take();
        crate::process::shutdown(&mut *self.child.lock().await).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    fn agent() -> ClaudeCode {
        ClaudeCode::new(crate::test_support::fake("claude")).with_timeout(Duration::from_secs(2))
    }
    fn request(prompt: &str) -> StartRequest {
        StartRequest {
            cwd: std::env::current_dir().unwrap(),
            prompt: prompt.into(),
            profile: PermissionProfile::Strict,
            model: None,
            name: None,
        }
    }
    async fn next(rx: &mut mpsc::Receiver<AgentEvent>) -> AgentEvent {
        timeout(Duration::from_secs(3), rx.recv())
            .await
            .expect("event timed out")
            .expect("stream closed")
    }
    async fn approval(rx: &mut mpsc::Receiver<AgentEvent>) -> String {
        loop {
            match next(rx).await {
                AgentEvent::ApprovalRequested(a) => return a.id,
                AgentEvent::Closed => panic!("agent closed before approval"),
                _ => {}
            }
        }
    }
    async fn result(rx: &mut mpsc::Receiver<AgentEvent>) -> (bool, Vec<String>) {
        let mut ok = None;
        let mut messages = Vec::new();
        loop {
            match next(rx).await {
                AgentEvent::ToolFinished { ok: value, .. } => ok = Some(value),
                AgentEvent::Message { text } => messages.push(text),
                AgentEvent::TurnCompleted { final_text, usage } => {
                    assert_eq!(final_text.as_deref(), Some("done"));
                    assert_eq!(usage.unwrap().input_tokens, Some(1));
                    return (ok.expect("tool result"), messages);
                }
                _ => {}
            }
        }
    }
    #[tokio::test]
    async fn fake_claude_allow_steer_and_resume() {
        let agent = agent();
        let mut session = agent.start(request("first")).await.unwrap();
        let id = session.id().to_owned();
        let mut rx = session.events();
        let approval_id = approval(&mut rx).await;
        session.steer("follow up").await.unwrap();
        session.answer(&approval_id, Decision::Allow).await.unwrap();
        let (ok, messages) = result(&mut rx).await;
        assert!(ok);
        assert!(messages.contains(&"follow up".to_string()));
        session.close().await.unwrap();
        assert_eq!(agent.list_sessions(10).await.unwrap()[0].id, id);
        let mut resumed = agent
            .resume(
                &id,
                &std::env::current_dir().unwrap(),
                PermissionProfile::Strict,
            )
            .await
            .unwrap();
        assert_eq!(resumed.id(), id);
        let mut rx = resumed.events();
        resumed.send("second").await.unwrap();
        let id = approval(&mut rx).await;
        resumed.answer(&id, Decision::Deny).await.unwrap();
        assert!(!result(&mut rx).await.0);
        resumed.close().await.unwrap();
    }
    #[tokio::test]
    async fn fake_claude_concurrent_sessions_are_isolated() {
        let agent = agent();
        let mut a = agent.start(request("alpha")).await.unwrap();
        let mut b = agent.start(request("beta")).await.unwrap();
        assert_ne!(a.id(), b.id());
        let mut ar = a.events();
        let mut br = b.events();
        let ai = approval(&mut ar).await;
        let bi = approval(&mut br).await;
        a.answer(&ai, Decision::Allow).await.unwrap();
        b.answer(&bi, Decision::Deny).await.unwrap();
        assert!(result(&mut ar).await.0);
        assert!(!result(&mut br).await.0);
        a.close().await.unwrap();
        b.close().await.unwrap();
    }
    #[tokio::test]
    async fn fake_claude_interrupt_and_bad_json() {
        let agent = agent();
        let mut session = agent.start(request("first")).await.unwrap();
        let mut rx = session.events();
        let _ = approval(&mut rx).await;
        session.interrupt().await.unwrap();
        loop {
            if matches!(next(&mut rx).await, AgentEvent::Closed) {
                break;
            }
        }
        session.close().await.unwrap();
        let mut session = agent.start(request("bad-json")).await.unwrap();
        let mut rx = session.events();
        loop {
            if let AgentEvent::Error { code, .. } = next(&mut rx).await {
                assert_eq!(code, "protocol");
                break;
            }
        }
        session.close().await.unwrap();
    }
    #[tokio::test]
    async fn fake_claude_init_timeout_and_unexpected_exit() {
        let mut req = request("first");
        req.model = Some("no-init".into());
        let agent = agent().with_timeout(Duration::from_millis(150));
        assert!(matches!(agent.start(req).await, Err(BridgeError::Timeout)));
        let mut session = agent.start(request("exit")).await.unwrap();
        let mut rx = session.events();
        loop {
            if matches!(next(&mut rx).await, AgentEvent::Closed) {
                break;
            }
        }
        session.close().await.unwrap();
    }
    #[test]
    fn candidate_permission_profiles_use_restricted_tool_list() {
        for profile in [PermissionProfile::Strict, PermissionProfile::AskForWrites] {
            assert_eq!(permission_settings(profile), ("default", "Read,Glob,Grep"));
        }
    }
    /// Opt-in only: launches the installed CLI and spends model usage.
    #[tokio::test]
    async fn real_claude_touch_requires_explicit_approval() {
        if std::env::var("XIAOBAN_REAL_CLAUDE").as_deref() != Ok("1") {
            return;
        }
        let agent = ClaudeCode::default();
        if !agent.availability().await.installed {
            eprintln!("Real Claude test not run: CLI not installed");
            return;
        }
        let dir = std::env::temp_dir().join(format!(
            "xiaoban-bridge-real-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let target = dir.join("bridge-permission-test.txt");
        let mut session=agent.start(StartRequest{cwd:dir.clone(),prompt:"Use the Bash tool to run exactly `touch bridge-permission-test.txt` in the current directory, then stop.".into(),profile:PermissionProfile::Strict,model:None,name:None}).await.expect("could not start Claude; ensure the CLI is logged in");
        let mut rx = session.events();
        let mut approved = false;
        let completed = timeout(Duration::from_secs(60), async {
            while let Some(event) = rx.recv().await {
                match event {
                    AgentEvent::ApprovalRequested(a) => {
                        let input: Value = serde_json::from_str(&a.detail).unwrap_or(Value::Null);
                        let exact = !target.exists()
                            && a.title == "Bash"
                            && input.get("command").and_then(Value::as_str).map(str::trim)
                                == Some("touch bridge-permission-test.txt");
                        let _ = session
                            .answer(
                                &a.id,
                                if exact {
                                    Decision::Allow
                                } else {
                                    Decision::Deny
                                },
                            )
                            .await;
                        if exact {
                            approved = true;
                        }
                    }
                    AgentEvent::TurnCompleted { .. } => return true,
                    AgentEvent::Closed | AgentEvent::Error { .. } => return false,
                    _ => {}
                }
            }
            false
        })
        .await
        .unwrap_or(false);
        session.close().await.unwrap();
        let created = target.is_file();
        if created {
            std::fs::remove_file(&target).unwrap();
        }
        std::fs::remove_dir(&dir).unwrap();
        assert!(
            approved,
            "the expected command did not produce an approval request"
        );
        assert!(
            completed && created,
            "the approved command did not complete and create the file"
        );
    }
}

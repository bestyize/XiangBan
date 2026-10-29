//! The phone's agent engine: [`agent_kernel`] with the network and the tools left to the host.
//!
//! The host talks to the engine in JSON lines (UTF-8, one object per line), the same shape as
//! `agent_kernel::stdio`, plus a few extra messages for what the host does on the engine's behalf.
//!
//! Host to engine ([`Engine::send`]):
//! - the `stdio` protocol: `run`, `cancel`, `approval`, `answer`
//! - `{"type":"model_line","id":"m1","line":"data: {...}"}` one line of the streamed reply to a `model_request`
//! - `{"type":"model_end","id":"m1","status":200,"error":null,"body":null}` the reply ended; `error` is a connection
//!   failure, `body` the response text when `status` is not 2xx
//! - `{"type":"tool_result","id":"t1","ok":true,"output":".."}` the outcome of a `tool_request`
//! - `{"type":"set_model","model":".."}` the model (tier) used from the next request on; empty means the server's default
//!
//! Engine to host ([`Engine::poll`]):
//! - the `stdio` protocol: `hello`, `event`, `run_end`, `approval_request`, `question`, `error`
//! - `{"type":"model_request","id":"m1","body":"{..}"}` POST this OpenAI-style chat-completions body (already streaming)
//!   and send the reply back line by line; the host adds the address and the credentials
//! - `{"type":"model_cancel","id":"m1"}` stop that request
//! - `{"type":"tool_request","id":"t1","name":"set_reminder","arguments":{..}}` run a tool the host declared
//! - `{"type":"closed"}` once, after the engine has stopped

use agent_kernel::builtin::TimeNow;
use agent_kernel::provider::stream::{SseParser, StreamAccumulator};
use agent_kernel::provider::wire::{build_request_body, ProviderProfile};
use agent_kernel::stdio::StdioBridge;
use agent_kernel::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufReader, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub mod jni_api;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

const POLL_STEP: Duration = Duration::from_millis(100);

#[derive(Deserialize)]
pub struct Config {
    /// The model (tier) to start with. Empty lets the server pick its default.
    #[serde(default)]
    pub model: String,
    /// Where conversations are kept (one `.jsonl` file per session).
    pub sessions_dir: String,
    #[serde(default)]
    pub system_prompt: String,
    /// Estimated tokens kept in the conversation sent to the model.
    #[serde(default = "default_context_tokens")]
    pub context_tokens: usize,
    /// Tools the host runs for the engine.
    #[serde(default)]
    pub tools: Vec<HostToolDef>,
}

fn default_context_tokens() -> usize {
    60_000
}

#[derive(Deserialize, Clone)]
pub struct HostToolDef {
    pub name: String,
    pub description: String,
    /// JSON Schema of the arguments object.
    pub parameters: Value,
    /// `read`, `write`, `network` or `exec`. Anything but `read` is confirmed with the user first.
    #[serde(default = "default_risk")]
    pub risk: String,
    #[serde(default = "default_tool_timeout")]
    pub timeout_secs: u64,
}

fn default_risk() -> String {
    "write".into()
}

fn default_tool_timeout() -> u64 {
    60
}

fn parse_risk(name: &str) -> Result<Risk, String> {
    match name {
        "read" => Ok(Risk::Read),
        "write" => Ok(Risk::Write),
        "network" => Ok(Risk::Network),
        "exec" => Ok(Risk::Exec),
        other => Err(format!("unknown risk `{other}` (use read, write, network or exec)")),
    }
}

// ───────────── talking to the host ─────────────

enum ModelMsg {
    Line(String),
    End { status: u16, error: Option<String>, body: Option<String> },
}

struct ToolMsg {
    ok: bool,
    output: String,
}

/// What the engine says to the host, and who is waiting for the host's answers.
struct Rpc {
    out: Mutex<Sender<String>>,
    /// Set when the host stops the engine: everything that waits for the host gives up.
    stopping: AtomicBool,
    next: AtomicU64,
    models: Mutex<HashMap<String, Sender<ModelMsg>>>,
    tools: Mutex<HashMap<String, Sender<ToolMsg>>>,
}

impl Rpc {
    fn id(&self, prefix: &str) -> String {
        format!("{prefix}{}", self.next.fetch_add(1, Ordering::Relaxed) + 1)
    }

    fn emit(&self, value: &Value) {
        let _ = lock(&self.out).send(value.to_string());
    }
}

/// Lets the kernel's JSON-lines writer feed the host's output queue: whole lines only.
struct LineWriter {
    tx: Sender<String>,
    buf: Vec<u8>,
}

impl Write for LineWriter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(data);
        while let Some(i) = self.buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.buf.drain(..=i).collect();
            let text = String::from_utf8_lossy(&line[..line.len() - 1]).into_owned();
            if !text.is_empty() && self.tx.send(text).is_err() {
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "the host stopped reading"));
            }
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// The kernel's input: lines the host sent, ending when the host stops the engine.
struct LineReader {
    rx: Receiver<Vec<u8>>,
    current: Vec<u8>,
    pos: usize,
}

impl Read for LineReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        while self.pos >= self.current.len() {
            match self.rx.recv() {
                Ok(next) => {
                    self.current = next;
                    self.pos = 0;
                }
                Err(_) => return Ok(0),
            }
        }
        let n = out.len().min(self.current.len() - self.pos);
        out[..n].copy_from_slice(&self.current[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

// ───────────── the model, over the host's network ─────────────

struct HostModel {
    rpc: Arc<Rpc>,
    model: Arc<Mutex<String>>,
}

fn http_error(status: u16, body: &str) -> HarnessError {
    let detail = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().or_else(|| v["message"].as_str()).map(str::to_string))
        .unwrap_or_else(|| body.chars().take(300).collect());
    let retryable = status == 429 || status >= 500;
    HarnessError::provider(format!("HTTP {status}: {detail}"), retryable)
}

impl ModelProvider for HostModel {
    fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
        let profile = ProviderProfile::new("host", "", lock(&self.model).clone());
        let mut body = build_request_body(&profile, request);
        if profile.model.trim().is_empty() {
            // No model chosen: leave the field out so the server uses its default.
            if let Some(fields) = body.as_object_mut() {
                fields.remove("model");
            }
        }
        let body = body.to_string();
        let id = self.rpc.id("m");
        let (tx, rx) = mpsc::channel();
        lock(&self.rpc.models).insert(id.clone(), tx);
        let finish = |rpc: &Rpc| {
            lock(&rpc.models).remove(&id);
        };
        self.rpc.emit(&json!({ "type": "model_request", "id": id, "body": body }));

        let (mut sse, mut acc) = (SseParser::default(), StreamAccumulator::default());
        let outcome = loop {
            if cancel.load(Ordering::SeqCst) || self.rpc.stopping.load(Ordering::SeqCst) {
                self.rpc.emit(&json!({ "type": "model_cancel", "id": id }));
                break Err(HarnessError::Cancelled);
            }
            match rx.recv_timeout(POLL_STEP) {
                Ok(ModelMsg::Line(line)) => {
                    if let Some(payload) = sse.push_line(&line) {
                        if payload.trim() == "[DONE]" {
                            acc.mark_done();
                            break Ok(());
                        }
                        let chunk: Value = match serde_json::from_str(&payload) {
                            Ok(v) => v,
                            Err(e) => break Err(HarnessError::provider(format!("bad stream chunk: {e}"), false)),
                        };
                        if let Err(e) = acc.push(&chunk, on_text) {
                            break Err(e);
                        }
                    }
                }
                Ok(ModelMsg::End { status, error, body }) => {
                    if let Some(payload) = sse.flush() {
                        if payload.trim() != "[DONE]" {
                            if let Ok(chunk) = serde_json::from_str::<Value>(&payload) {
                                let _ = acc.push(&chunk, on_text);
                            }
                        }
                    }
                    if let Some(message) = error {
                        break Err(HarnessError::provider(message, true));
                    }
                    if !(200..300).contains(&status) {
                        break Err(http_error(status, body.as_deref().unwrap_or("")));
                    }
                    acc.mark_done();
                    break Ok(());
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break Err(HarnessError::provider("the host stopped answering", true)),
            }
        };
        finish(&self.rpc);
        outcome.map(|()| acc.finish())
    }
}

// ───────────── tools the host runs ─────────────

struct HostTool {
    spec: ToolSpec,
    rpc: Arc<Rpc>,
}

impl Tool for HostTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let id = self.rpc.id("t");
        let (tx, rx) = mpsc::channel();
        lock(&self.rpc.tools).insert(id.clone(), tx);
        self.rpc.emit(&json!({ "type": "tool_request", "id": id, "name": self.spec.name, "arguments": args }));
        let started = Instant::now();
        let outcome = loop {
            if ctx.is_cancelled() || self.rpc.stopping.load(Ordering::SeqCst) {
                break Err("cancelled".to_string());
            }
            if started.elapsed() > self.spec.timeout {
                break Err("the tool did not answer in time".to_string());
            }
            match rx.recv_timeout(POLL_STEP) {
                Ok(msg) => break if msg.ok { Ok(msg.output) } else { Err(msg.output) },
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break Err("the host stopped answering".to_string()),
            }
        };
        lock(&self.rpc.tools).remove(&id);
        outcome
    }
}

// ───────────── the engine ─────────────

pub enum Polled {
    Line(String),
    Timeout,
    Closed,
}

pub struct Engine {
    input: Mutex<Option<Sender<Vec<u8>>>>,
    output: Mutex<Receiver<String>>,
    rpc: Arc<Rpc>,
    model: Arc<Mutex<String>>,
    worker: Mutex<Option<JoinHandle<()>>>,
    closed_sent: AtomicBool,
}

impl Engine {
    pub fn start(config_json: &str) -> Result<Arc<Engine>, String> {
        let config: Config = serde_json::from_str(config_json).map_err(|e| format!("bad config: {e}"))?;
        let (out_tx, out_rx) = mpsc::channel::<String>();
        let rpc = Arc::new(Rpc {
            out: Mutex::new(out_tx.clone()),
            stopping: AtomicBool::new(false),
            next: AtomicU64::new(0),
            models: Mutex::new(HashMap::new()),
            tools: Mutex::new(HashMap::new()),
        });
        let model = Arc::new(Mutex::new(config.model.clone()));
        let provider: Arc<dyn ModelProvider> = Arc::new(HostModel { rpc: rpc.clone(), model: model.clone() });
        let bridge = StdioBridge::new(LineWriter { tx: out_tx.clone(), buf: Vec::new() });

        let mut builder = Agent::builder(provider, Arc::new(JsonlStore::new(config.sessions_dir.clone())))
            .system_prompt(config.system_prompt.clone())
            .tool(TimeNow::new())
            .middleware(Permission::new(PermissionMode::Ask, bridge.approver()))
            .middleware(ContextTrim::new(config.context_tokens).reserve_output(4096));
        for def in &config.tools {
            let risk = parse_risk(&def.risk).map_err(|e| format!("tool `{}`: {e}", def.name))?;
            let spec = ToolSpec::new(def.name.clone(), def.description.clone(), def.parameters.clone())
                .risk(risk)
                .timeout(Duration::from_secs(def.timeout_secs.max(1)));
            builder = builder.tool(HostTool { spec, rpc: rpc.clone() });
        }
        let agent = Arc::new(builder.build().map_err(|e| format!("cannot build the agent: {e}"))?);

        let (in_tx, in_rx) = mpsc::channel::<Vec<u8>>();
        let closed_tx = out_tx;
        let worker = thread::Builder::new()
            .name("agent-mobile".into())
            .spawn(move || {
                let reader = BufReader::new(LineReader { rx: in_rx, current: Vec::new(), pos: 0 });
                let _ = bridge.serve(agent, reader);
                let _ = closed_tx.send(json!({ "type": "closed" }).to_string());
            })
            .map_err(|e| format!("cannot start the agent thread: {e}"))?;

        Ok(Arc::new(Engine {
            input: Mutex::new(Some(in_tx)),
            output: Mutex::new(out_rx),
            rpc,
            model,
            worker: Mutex::new(Some(worker)),
            closed_sent: AtomicBool::new(false),
        }))
    }

    /// One message from the host. Answers to what the engine asked for are delivered to whoever waits for
    /// them; everything else goes to the kernel's own protocol.
    pub fn send(&self, line: &str) -> Result<(), String> {
        let value: Value = serde_json::from_str(line).map_err(|e| format!("not JSON: {e}"))?;
        let id = value["id"].as_str().unwrap_or("").to_string();
        match value["type"].as_str().unwrap_or("") {
            "model_line" => {
                if let Some(tx) = lock(&self.rpc.models).get(&id) {
                    let _ = tx.send(ModelMsg::Line(value["line"].as_str().unwrap_or("").to_string()));
                }
                Ok(())
            }
            "model_end" => {
                if let Some(tx) = lock(&self.rpc.models).get(&id) {
                    let _ = tx.send(ModelMsg::End {
                        status: value["status"].as_u64().unwrap_or(0) as u16,
                        error: value["error"].as_str().map(str::to_string),
                        body: value["body"].as_str().map(str::to_string),
                    });
                }
                Ok(())
            }
            "tool_result" => {
                if let Some(tx) = lock(&self.rpc.tools).get(&id) {
                    let _ = tx.send(ToolMsg { ok: value["ok"].as_bool().unwrap_or(false), output: value["output"].as_str().unwrap_or("").to_string() });
                }
                Ok(())
            }
            "set_model" => {
                *lock(&self.model) = value["model"].as_str().unwrap_or("").trim().to_string();
                Ok(())
            }
            _ => {
                let mut bytes = line.as_bytes().to_vec();
                bytes.push(b'\n');
                match &*lock(&self.input) {
                    Some(tx) => tx.send(bytes).map_err(|_| "the engine has stopped".to_string()),
                    None => Err("the engine has stopped".into()),
                }
            }
        }
    }

    /// The next line for the host, waiting up to `timeout`.
    pub fn poll(&self, timeout: Duration) -> Polled {
        match lock(&self.output).recv_timeout(timeout) {
            Ok(line) => {
                if line.contains("\"type\":\"closed\"") {
                    self.closed_sent.store(true, Ordering::SeqCst);
                }
                Polled::Line(line)
            }
            Err(RecvTimeoutError::Timeout) => Polled::Timeout,
            Err(RecvTimeoutError::Disconnected) => {
                if self.closed_sent.swap(true, Ordering::SeqCst) {
                    Polled::Closed
                } else {
                    Polled::Line(json!({ "type": "closed" }).to_string())
                }
            }
        }
    }

    /// Ends the engine: running work is cancelled, waiting tools and requests are released.
    pub fn stop(&self) {
        self.rpc.stopping.store(true, Ordering::SeqCst);
        lock(&self.input).take();
        lock(&self.rpc.models).clear();
        lock(&self.rpc.tools).clear();
        if let Some(worker) = lock(&self.worker).take() {
            let _ = worker.join();
        }
    }
}

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

use agent_kernel::builtin::{AskUser, TimeNow, Todo};
use agent_kernel::provider::stream::{SseParser, StreamAccumulator};
use agent_kernel::provider::wire::{encode_request_body, ProviderProfile};
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
mod policy;
mod media;

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
    /// The phone's current local UTC offset, e.g. 480 for UTC+08:00. Old hosts keep UTC.
    #[serde(default)]
    pub utc_offset_minutes: i32,
    /// Estimated tokens kept in the conversation sent to the model.
    #[serde(default = "default_context_tokens")]
    pub context_tokens: usize,
    /// Tools the host runs for the engine.
    #[serde(default)]
    pub tools: Vec<HostToolDef>,
}

fn default_context_tokens() -> usize {
    256_000
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
    #[serde(default)] pub always_ask: bool,
    #[serde(default)] pub untrusted_output: bool,
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
    End { status: u16, error: Option<String>, body: Option<String>, verified_tool_terminal: bool },
}

struct ToolMsg {
    ok: bool,
    output: String,
    cancelled: bool,
    outcome_unknown: bool,
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
    settings: policy::Settings,
    attachments: std::path::PathBuf,
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
    fn enforces_request_token_budget(&self) -> bool { true }

    fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
        self.chat_with_reasoning(request, on_text, &mut |_, _| {}, cancel)
    }

    fn chat_with_reasoning(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(ReasoningKind, &str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
        self.chat_with_progress(request, on_text, on_reasoning, &mut |_| {}, cancel)
    }

    fn chat_with_progress(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(ReasoningKind, &str), on_tool: &mut dyn FnMut(&ToolProgress), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
        self.chat_with_output_budget(request, None, on_text, on_reasoning, on_tool, cancel)
    }

    fn chat_with_output_budget(&self, request: &ChatRequest, output_limit: Option<u32>, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(ReasoningKind, &str), on_tool: &mut dyn FnMut(&ToolProgress), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
        let session = policy::MODEL_SESSION.with(|s| s.borrow().clone());
        let settings = lock(&self.settings).get(&session).cloned();
        let mut profile = ProviderProfile::new("host", "", settings.as_ref().map(|s| s.model.clone()).unwrap_or_else(|| lock(&self.model).clone()));
        profile.max_request_bytes = Some(8 * 1024 * 1024 - 4096);
        if let Some(settings) = &settings { profile.supports_tools = settings.supports_tools; profile.max_output_tokens = settings.max_output_tokens; }
        if let Some(configured) = profile.max_output_tokens {
            let limit = configured.min(output_limit.unwrap_or(configured));
            profile.extra_body.insert("max_tokens".into(), json!(limit));
        }
        let mut hydrated;
        let request = if media::has_references(&request.messages) {
            hydrated = request.clone();
            media::hydrate(&self.attachments, &session, &mut hydrated)?;
            &hydrated
        } else { request };
        let mut body: Value = serde_json::from_slice(&encode_request_body(&profile, request)?).map_err(|e|HarnessError::Config(e.to_string()))?;
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
        self.rpc.emit(&json!({ "type": "model_request", "id": id, "session": session, "body": body }));

        let (mut sse, mut acc) = (SseParser::default(), StreamAccumulator::default());
        let mut received_done = false;
        let mut consume = |acc: &mut StreamAccumulator, received_done: &mut bool, payload: &str| -> Result<(), HarnessError> {
            if payload.trim() == "[DONE]" {
                *received_done = true;
                acc.mark_done();
                return Ok(());
            }
            let chunk: Value = serde_json::from_str(payload)
                .map_err(|e| HarnessError::provider(format!("bad stream chunk: {e}"), false))?;
            acc.push_with_reasoning(&chunk, on_text, on_reasoning)?;
            if let Some(progress) = acc.pending_tool_call() { on_tool(&progress); }
            Ok(())
        };
        let outcome = loop {
            if cancel.load(Ordering::SeqCst) || self.rpc.stopping.load(Ordering::SeqCst) {
                self.rpc.emit(&json!({ "type": "model_cancel", "id": id }));
                break Err(HarnessError::Cancelled);
            }
            match rx.recv_timeout(POLL_STEP) {
                Ok(ModelMsg::Line(line)) => {
                    if let Some(payload) = sse.push_line(&line) {
                        if let Err(error) = consume(&mut acc, &mut received_done, &payload) { break Err(error); }
                    }
                }
                Ok(ModelMsg::End { status, error, body, verified_tool_terminal }) => {
                    if let Some(payload) = sse.flush() {
                        if let Err(error) = consume(&mut acc, &mut received_done, &payload) { break Err(error); }
                    }
                    if let Some(message) = error {
                        break Err(HarnessError::provider(message, status == 0 || status == 429 || status >= 500));
                    }
                    if !(200..300).contains(&status) {
                        break Err(http_error(status, body.as_deref().unwrap_or("")));
                    }
                    if let Some(body) = body {
                        let result = serde_json::from_str::<Value>(&body).ok().and_then(|mut chunk| {
                            let choice = chunk["choices"].get_mut(0)?.as_object_mut()?;
                            let message = choice.remove("message")?;
                            if choice.get("finish_reason").is_none_or(Value::is_null) && message["tool_calls"].is_null() {
                                choice.insert("finish_reason".into(), json!("stop"));
                            }
                            choice.insert("delta".into(), message);
                            Some(chunk)
                        });
                        let Some(chunk) = result else {
                            break Err(HarnessError::provider("模型返回了无效的 JSON 回复", false));
                        };
                        if let Err(error) = consume(&mut acc, &mut received_done, &chunk.to_string()) { break Err(error); }
                    }
                    if verified_tool_terminal && received_done
                        && matches!(acc.finish_reason(), Some("tool_calls" | "function_call")) {
                        acc.fill_empty_tool_arguments(|name| request.tools.iter().any(|tool| {
                            tool.name == name
                                && tool.parameters["type"] == "object"
                                && tool.parameters["additionalProperties"] == false
                                && tool.parameters["properties"].as_object().is_some_and(|properties| properties.is_empty())
                                && (tool.parameters["required"].is_null()
                                    || tool.parameters["required"].as_array().is_some_and(|required| required.is_empty()))
                        }));
                    }
                    let has_calls = acc.pending_tool_call().is_some();
                    if has_calls && (!profile.supports_tools || !verified_tool_terminal || !acc.tool_calls_complete()
                        || !matches!(acc.finish_reason(), Some("tool_calls" | "function_call" | "stop"))) {
                        break Err(HarnessError::provider("工具调用未完整结束或网关版本过旧，未执行操作", false));
                    }
                    if !acc.is_finished() || matches!(acc.finish_reason(), Some("content_filter")) {
                        break Err(HarnessError::provider("模型回复未完整结束，请重试", false));
                    }
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
    permission: Arc<Permission>,
    trust: std::path::PathBuf,
}

impl Tool for HostTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn cancel_grace(&self) -> Duration { Duration::from_secs(3) }

    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        let id = self.rpc.id("t");
        let (tx, rx) = mpsc::channel();
        lock(&self.rpc.tools).insert(id.clone(), tx);
        self.rpc.emit(&json!({ "type": "tool_request", "id": id, "session": ctx.session(), "run_id": ctx.run_id(), "name": self.spec.name, "arguments": args }));
        let started = Instant::now();
        let mut cancelling = None;
        let result = loop {
            if cancelling.is_none() && (ctx.is_cancelled() || self.rpc.stopping.load(Ordering::SeqCst) || started.elapsed() > self.spec.timeout) {
                self.rpc.emit(&json!({"type":"tool_cancel","id":id,"session":ctx.session()}));
                cancelling = Some(Instant::now());
            }
            if cancelling.is_some_and(|at: Instant| at.elapsed() > Duration::from_secs(2)) {
                break ToolResult::unknown("phone tool did not acknowledge cancellation");
            }
            match rx.recv_timeout(POLL_STEP) {
                Ok(msg) if msg.outcome_unknown => break ToolResult::unknown(&msg.output),
                Ok(msg) if msg.cancelled => break ToolResult::error("cancelled before a confirmed side effect"),
                Ok(msg) => break if msg.ok { ToolResult::ok(msg.output) } else { ToolResult::error(msg.output) },
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break ToolResult::unknown("the phone stopped answering"),
            }
        };
        lock(&self.rpc.tools).remove(&id);
        if result.structured_content.as_ref().is_some_and(|v|v["harness_status"] == "outcome_unknown") {
            let _ = self.permission.restore_trust(&self.trust, ctx.session(), false);
            let _ = self.permission.require_review(ctx.session());
        }
        result
    }
}

// ───────────── the engine ─────────────

pub enum Polled {
    Line(String),
    Timeout,
    Closed,
}

struct MobileSessionStore {
    history: JsonlStore,
    deleted: std::path::PathBuf,
    attachments: std::path::PathBuf,
}

impl SessionStore for MobileSessionStore {
    fn load(&self, session: &str) -> Result<Vec<Message>, HarnessError> {
        if policy::valid_session(session) && self.deleted.join(session).is_file() {
            return Err(HarnessError::Store("会话已删除".into()));
        }
        self.history.load(session)
    }

    fn append(&self, session: &str, messages: &[Message]) -> Result<(), HarnessError> {
        if policy::valid_session(session) && self.deleted.join(session).is_file() {
            return Err(HarnessError::Store("会话已删除".into()));
        }
        if messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part, Part::Image { url } if url.starts_with("data:image/"))) {
            let mut compact = messages.to_vec();
            media::compact(&self.attachments, session, &mut compact)?;
            self.history.append(session, &compact)
        } else {
            self.history.append(session, messages)
        }
    }
}

pub struct Engine {
    input: Mutex<Option<Sender<Vec<u8>>>>,
    output: Mutex<Receiver<String>>,
    rpc: Arc<Rpc>,
    model: Arc<Mutex<String>>,
    worker: Mutex<Option<JoinHandle<()>>>,
    closed_sent: AtomicBool,
    permission: Arc<Permission>,
    settings: policy::Settings,
    trust: std::path::PathBuf,
    deleted: std::path::PathBuf,
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
        let settings: policy::Settings = Arc::new(Mutex::new(HashMap::new()));
        let attachments = std::path::Path::new(&config.sessions_dir).join("attachments");
        let provider: Arc<dyn ModelProvider> = Arc::new(HostModel { rpc: rpc.clone(), model: model.clone(), settings: settings.clone(), attachments: attachments.clone() });
        let bridge = StdioBridge::with_no_progress_limit(LineWriter { tx: out_tx.clone(), buf: Vec::new() }, Some(6));
        // Progress is independent of chat trimming and uses the same per-session isolation.
        let todo = Arc::new(Todo::new().persistent(std::path::Path::new(&config.sessions_dir).join("progress")));

        let always_confirm: std::collections::HashSet<_> = config.tools.iter().filter(|tool|tool.always_ask).map(|tool|tool.name.clone()).collect();
        let permission = Arc::new(Permission::new(PermissionMode::Ask, bridge.approver()).danger_check(move |tool,args| {
            if always_confirm.contains(tool) { Some("此工具每次执行都需要确认".into()) } else { policy::danger(tool,args) }
        }).check_reads().honor_grants_after_taint());
        let trust = std::path::Path::new(&config.sessions_dir).join("trust");
        let deleted = std::path::Path::new(&config.sessions_dir).join("deleted");
        let store = Arc::new(MobileSessionStore { history: JsonlStore::new(&config.sessions_dir), deleted: deleted.clone(), attachments });
        let spill = std::path::Path::new(&config.sessions_dir).join("tool-output");
        let mut builder = Agent::builder(provider, store.clone())
            .system_prompt(config.system_prompt.clone())
            .tool(TimeNow::new().utc_offset_minutes(config.utc_offset_minutes))
            .middleware(todo.clone())
            .tool_arc(todo)
            .tool(AskUser::new(bridge.ask_handler()))
            .middleware(permission.clone())
            .middleware(RepeatGuard::new())
            .middleware(policy::MobileContext { settings: settings.clone(), fallback_context: config.context_tokens })
            .middleware(policy::MobileSpill(spill.clone()))
            .tool(policy::OutputRead::new(spill))
            .tool(policy::HistoryRead::new(store))
            .middleware(ContinueGuard::new())
            .retry(RetryPolicy { max_retries: 3, ..Default::default() })
            .unlimited_turns();
        for def in &config.tools {
            let risk = parse_risk(&def.risk).map_err(|e| format!("tool `{}`: {e}", def.name))?;
            let mut spec = ToolSpec::new(def.name.clone(), def.description.clone(), def.parameters.clone())
                .risk(risk)
                .timeout(Duration::from_secs(def.timeout_secs.max(1)));
            spec.always_ask = def.always_ask;
            spec.untrusted_output = def.untrusted_output;
            builder = builder.tool(HostTool { spec, rpc: rpc.clone(), permission: permission.clone(), trust: trust.clone() });
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
            permission, settings, trust, deleted,
        }))
    }

    /// One message from the host. Answers to what the engine asked for are delivered to whoever waits for
    /// them; everything else goes to the kernel's own protocol.
    pub fn send(&self, line: &str) -> Result<(), String> {
        let value: Value = serde_json::from_str(line).map_err(|e| format!("not JSON: {e}"))?;
        let id = value["id"].as_str().unwrap_or("").to_string();
        let kind = value["type"].as_str().unwrap_or("");
        if matches!(kind, "run" | "configure_session" | "reviewed_session") {
            if let Some(session) = value["session"].as_str().filter(|s| policy::valid_session(s)) {
                if self.deleted.join(session).is_file() { return Err("会话已删除".into()); }
            }
        }
        match kind {
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
                        verified_tool_terminal: value["verified_tool_terminal"].as_bool().unwrap_or(true),
                    });
                }
                Ok(())
            }
            "tool_result" => {
                if let Some(tx) = lock(&self.rpc.tools).get(&id) {
                    let _ = tx.send(ToolMsg { ok: value["ok"].as_bool().unwrap_or(false), output: value["output"].as_str().unwrap_or("").to_string(), cancelled: value["cancelled"].as_bool().unwrap_or(false), outcome_unknown: value["outcome_unknown"].as_bool().unwrap_or(false) });
                }
                Ok(())
            }
            "reviewed_session" => {
                let session = value["session"].as_str().ok_or("missing session")?;
                if !policy::valid_session(session) { return Err("invalid session identity".into()); }
                self.permission.clear_review(session)
            }
            "configure_session" => {
                let session = value["session"].as_str().ok_or("missing session")?;
                let next = serde_json::from_value(value["settings"].clone()).map_err(|e|format!("invalid session settings: {e}"))?;
                policy::configure(&self.settings, &self.permission, &self.trust, session, next)
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

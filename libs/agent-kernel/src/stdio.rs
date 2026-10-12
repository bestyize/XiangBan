//! JSON-lines bridge for hosts that are not written in Rust: one JSON object per line, UTF-8.
//!
//! Client to server:
//! - `{"type":"run","id":"r1","session":"s1","input":"hello"}` (or `"parts":[{"type":"text","text":".."},{"type":"image","url":".."}]`)
//! - `{"type":"cancel","id":"r1"}` (does nothing if the run already ended)
//! - `{"type":"approval","id":"a1","decision":"once"|"always"|"deny"}`
//! - `{"type":"answer","id":"q1","answer":"text"}` (`null` means the user dismissed the question)
//!
//! Server to client:
//! - `{"type":"hello","protocol":1}` once at the start
//! - `{"type":"event","run":"r1","event":{"type":"text_delta","text":".."}}`
//! - `{"type":"run_end","run":"r1","ok":true,"text":"..","turns":2,"finish":"stop","prompt_tokens":10,"completion_tokens":5}`
//!   (`finish` is `stop`, `length` when the answer may be cut off, `content_filter` or `other`) or `{"type":"run_end","run":"r1","ok":false,"error":".."}`; the run id can be reused after it
//! - `{"type":"approval_request","id":"a1","session":"s1","tool":"shell","risk":"exec","arguments":{..},"tainted":false}` (plus `"danger":".."` when the host's `danger_check` flagged the call)
//! - `{"type":"question","id":"q1","session":"s1","question":"..","options":[..]}`
//! - `{"type":"error","message":".."}` for malformed input; the server keeps going
//!
//! Build the agent with [`StdioBridge::approver`] (for `Permission`) and [`StdioBridge::ask_handler`]
//! (for `AskUser`), then call [`StdioBridge::serve`]. When the input ends, running work is left to
//! finish and any question that is still open is answered with a denial or a dismissal.
//! When a run is cancelled, its open approvals and questions are resolved the same way.

use crate::builtin::Question;
use crate::middleware::{Approval, ApprovalRequest, Approver};
use crate::tool::{Risk, ToolCtx};
use crate::types::{Part, PROTOCOL_VERSION};
use crate::Agent;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;
const MAX_ID_CHARS: usize = 128;
const MAX_SESSION_CHARS: usize = 256;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Incoming {
    Run { id: String, session: String, input: Option<String>, parts: Option<Vec<Part>> },
    Cancel { id: String },
    Approval { id: String, decision: String },
    Answer { id: String, answer: Option<String> },
}

struct RunInfo {
    session: String,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
struct Pending {
    approvals: HashMap<String, (String, Sender<Approval>)>,
    questions: HashMap<String, (String, Sender<Option<String>>)>,
}

pub struct StdioBridge {
    out: Mutex<Box<dyn Write + Send>>,
    broken: AtomicBool,
    input_closed: AtomicBool,
    next_id: AtomicU64,
    pending: Mutex<Pending>,
    runs: Mutex<HashMap<String, RunInfo>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

fn risk_name(risk: Risk) -> &'static str {
    match risk {
        Risk::Read => "read",
        Risk::Write => "write",
        Risk::Network => "network",
        Risk::Exec => "exec",
    }
}

enum Line {
    Data,
    TooLong,
    Eof,
}

/// Reads one line into `buf` without ever holding more than `cap` bytes of an over-long line.
fn read_line_capped(input: &mut impl BufRead, cap: usize, buf: &mut Vec<u8>) -> io::Result<Line> {
    buf.clear();
    let (mut too_long, mut seen_any) = (false, false);
    loop {
        let available = match input.fill_buf() {
            Ok(a) => a,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        if available.is_empty() {
            return Ok(if too_long { Line::TooLong } else if seen_any { Line::Data } else { Line::Eof });
        }
        seen_any = true;
        let (chunk, found_newline) = match available.iter().position(|&b| b == b'\n') {
            Some(i) => (&available[..i], true),
            None => (available, false),
        };
        if !too_long {
            if buf.len() + chunk.len() > cap {
                too_long = true;
                buf.clear();
            } else {
                buf.extend_from_slice(chunk);
            }
        }
        let used = chunk.len() + usize::from(found_newline);
        input.consume(used);
        if found_newline {
            return Ok(if too_long { Line::TooLong } else { Line::Data });
        }
    }
}

struct BridgeApprover(Arc<StdioBridge>);

impl Approver for BridgeApprover {
    fn approve(&self, request: &ApprovalRequest<'_>) -> Approval {
        self.0.request_approval(request)
    }
}

impl StdioBridge {
    pub fn new(out: impl Write + Send + 'static) -> Arc<Self> {
        Arc::new(Self {
            out: Mutex::new(Box::new(out)),
            broken: AtomicBool::new(false),
            input_closed: AtomicBool::new(false),
            next_id: AtomicU64::new(0),
            pending: Mutex::new(Pending::default()),
            runs: Mutex::new(HashMap::new()),
        })
    }

    /// For `Permission::new`: approval requests go to the client as `approval_request` lines.
    pub fn approver(self: &Arc<Self>) -> impl Approver + 'static {
        BridgeApprover(self.clone())
    }

    /// For `AskUser::new`: questions go to the client as `question` lines.
    pub fn ask_handler(self: &Arc<Self>) -> impl Fn(&str, &Question, &ToolCtx) -> Option<String> + Send + Sync + 'static {
        let bridge = self.clone();
        move |session, question, ctx| bridge.request_answer(session, question, ctx)
    }

    /// Reads commands until the input ends, then waits for running work and returns. A closed
    /// output is only noticed between input lines, so the host should close both together.
    /// Returns an error if reading failed or the output was closed by the other side.
    pub fn serve(self: &Arc<Self>, agent: Arc<Agent>, mut input: impl BufRead) -> io::Result<()> {
        self.send(&json!({ "type": "hello", "protocol": PROTOCOL_VERSION }));
        let mut handles: Vec<JoinHandle<()>> = Vec::new();
        let mut buf = Vec::new();
        let outcome = loop {
            if self.broken.load(Ordering::Relaxed) {
                break Ok(());
            }
            match read_line_capped(&mut input, MAX_LINE_BYTES, &mut buf) {
                Ok(Line::Eof) => break Ok(()),
                Ok(Line::TooLong) => self.error("line too long (the limit is 8 MiB)"),
                Ok(Line::Data) => self.handle_line(&agent, &buf, &mut handles),
                Err(e) => break Err(e),
            }
        };
        self.input_closed.store(true, Ordering::SeqCst);
        self.clear_pending(|_| true);
        for handle in handles {
            let _ = handle.join();
        }
        match outcome {
            Ok(()) if self.broken.load(Ordering::Relaxed) => Err(io::Error::new(io::ErrorKind::BrokenPipe, "the output was closed")),
            other => other,
        }
    }

    fn send(&self, value: &Value) -> bool {
        if self.broken.load(Ordering::Relaxed) {
            return false;
        }
        let mut line = serde_json::to_vec(value).unwrap_or_default();
        line.push(b'\n');
        let ok = {
            let mut out = lock(&self.out);
            out.write_all(&line).and_then(|()| out.flush()).is_ok()
        };
        if !ok {
            self.broken.store(true, Ordering::SeqCst);
            for run in lock(&self.runs).values() {
                run.cancel.store(true, Ordering::SeqCst);
            }
            self.clear_pending(|_| true);
        }
        ok
    }

    fn error(&self, message: &str) {
        self.send(&json!({ "type": "error", "message": message }));
    }

    fn clear_pending(&self, matches: impl Fn(&str) -> bool) {
        let mut pending = lock(&self.pending);
        pending.approvals.retain(|_, (session, _)| !matches(session));
        pending.questions.retain(|_, (session, _)| !matches(session));
    }

    fn unavailable(&self, session: &str) -> bool {
        self.broken.load(Ordering::SeqCst)
            || self.input_closed.load(Ordering::SeqCst)
            || lock(&self.runs).values().any(|r| {
                r.cancel.load(Ordering::SeqCst) && (session == r.session || session.starts_with(&format!("{}#", r.session)))
            })
    }

    fn next_id(&self, prefix: char) -> String {
        format!("{prefix}{}", self.next_id.fetch_add(1, Ordering::Relaxed) + 1)
    }

    fn request_approval(&self, request: &ApprovalRequest<'_>) -> Approval {
        if self.unavailable(request.session) {
            return Approval::Deny;
        }
        let id = self.next_id('a');
        let (tx, rx) = mpsc::channel();
        lock(&self.pending).approvals.insert(id.clone(), (request.session.to_string(), tx));
        // Re-checked after registering: a cancel or close that ran in between has already cleared the table.
        let mut message = json!({
            "type": "approval_request", "id": id, "session": request.session, "tool": request.tool,
            "risk": risk_name(request.risk), "arguments": request.arguments, "tainted": request.tainted,
        });
        if let Some(danger) = request.danger {
            message["danger"] = json!(danger);
        }
        let sent = !self.unavailable(request.session) && self.send(&message);
        if !sent {
            lock(&self.pending).approvals.remove(&id);
            return Approval::Deny;
        }
        let decision = request.wait_for(&rx).unwrap_or(Approval::Deny);
        lock(&self.pending).approvals.remove(&id);
        decision
    }

    fn request_answer(&self, session: &str, question: &Question, ctx: &ToolCtx) -> Option<String> {
        if self.unavailable(session) {
            return None;
        }
        let id = self.next_id('q');
        let (tx, rx) = mpsc::channel();
        lock(&self.pending).questions.insert(id.clone(), (session.to_string(), tx));
        let sent = !self.unavailable(session)
            && self.send(&json!({ "type": "question", "id": id, "session": session, "question": question.text, "options": question.options }));
        if !sent {
            lock(&self.pending).questions.remove(&id);
            return None;
        }
        let answer = ctx.wait_for(&rx).flatten();
        lock(&self.pending).questions.remove(&id);
        answer
    }

    fn handle_line(self: &Arc<Self>, agent: &Arc<Agent>, line: &[u8], handles: &mut Vec<JoinHandle<()>>) {
        if line.iter().all(u8::is_ascii_whitespace) {
            return;
        }
        match serde_json::from_slice::<Incoming>(line) {
            Err(e) => self.error(&format!("invalid message: {e}")),
            Ok(Incoming::Run { id, session, input, parts }) => {
                let input = match (input, parts) {
                    (Some(text), None) => vec![Part::Text { text }],
                    (None, Some(parts)) if !parts.is_empty() => parts,
                    _ => return self.error("a run needs exactly one of `input` (a string) or `parts` (a non-empty array)"),
                };
                if id.is_empty() || id.chars().count() > MAX_ID_CHARS || session.is_empty() || session.chars().count() > MAX_SESSION_CHARS {
                    return self.error("`id` must be 1 to 128 characters and `session` 1 to 256 characters");
                }
                handles.retain(|h| !h.is_finished());
                if let Some(handle) = self.start_run(agent, id, session, input) {
                    handles.push(handle);
                }
            }
            Ok(Incoming::Cancel { id }) => {
                let session = lock(&self.runs).get(&id).map(|run| {
                    run.cancel.store(true, Ordering::SeqCst);
                    run.session.clone()
                });
                if let Some(session) = session {
                    self.clear_pending(|s| s == session || s.starts_with(&format!("{session}#")));
                }
            }
            Ok(Incoming::Approval { id, decision }) => {
                let decision = match decision.as_str() {
                    "once" => Approval::Once,
                    "always" => Approval::Always,
                    "deny" => Approval::Deny,
                    other => return self.error(&format!("unknown decision `{other}`; use once, always or deny")),
                };
                let waiting = lock(&self.pending).approvals.remove(&id);
                match waiting {
                    Some((_, tx)) => {
                        let _ = tx.send(decision);
                    }
                    None => self.error(&format!("no open approval request `{id}`")),
                }
            }
            Ok(Incoming::Answer { id, answer }) => {
                let waiting = lock(&self.pending).questions.remove(&id);
                match waiting {
                    Some((_, tx)) => {
                        let _ = tx.send(answer);
                    }
                    None => self.error(&format!("no open question `{id}`")),
                }
            }
        }
    }

    fn start_run(self: &Arc<Self>, agent: &Arc<Agent>, id: String, session: String, input: Vec<Part>) -> Option<JoinHandle<()>> {
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut runs = lock(&self.runs);
            if runs.contains_key(&id) {
                drop(runs);
                self.error(&format!("run id `{id}` is already in use"));
                return None;
            }
            runs.insert(id.clone(), RunInfo { session: session.clone(), cancel: cancel.clone() });
        }
        let (bridge, agent) = (self.clone(), agent.clone());
        let thread_id = id.clone();
        let spawned = thread::Builder::new().name(format!("run-{id}")).spawn(move || {
            let result = agent.run_parts(
                &session,
                input,
                &mut |event| {
                    if !bridge.send(&json!({ "type": "event", "run": thread_id, "event": event })) {
                        cancel.store(true, Ordering::SeqCst);
                    }
                },
                &cancel,
            );
            lock(&bridge.runs).remove(&thread_id);
            let end = match result {
                Ok(s) => json!({
                    "type": "run_end", "run": thread_id, "ok": true, "text": s.text, "turns": s.turns, "finish": s.finish,
                    "prompt_tokens": s.usage.prompt_tokens, "completion_tokens": s.usage.completion_tokens,
                }),
                Err(e) => json!({ "type": "run_end", "run": thread_id, "ok": false, "error": e.to_string() }),
            };
            bridge.send(&end);
        });
        match spawned {
            Ok(handle) => Some(handle),
            Err(e) => {
                lock(&self.runs).remove(&id);
                self.error(&format!("cannot start the run: {e}"));
                None
            }
        }
    }
}

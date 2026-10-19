//! Optional, host-owned local process sessions. Commands run with the host user's privileges.
//! Run-scoped children are cleaned up by the start tool's on_run_end hook. Session-scoped
//! children require the host to retain this manager and close the session/application explicitly.

use crate::{Risk, RunEnd, RunState, Tool, ToolCtx, ToolResult, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(20);
static PROCESS_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
const BASE_ENV: &[&str] = &[
    "PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "LC_CTYPE", "TMPDIR", "TZ",
];
#[cfg(windows)]
const BASE_ENV: &[&str] = &[
    "PATH",
    "SystemRoot",
    "ComSpec",
    "PATHEXT",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "windir",
];

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub max_running: usize,
    pub max_retained: usize,
    pub output_bytes: usize,
    pub max_runtime: Duration,
    pub env: Vec<(String, String)>,
    pub inherited_env: Vec<String>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            max_running: 8,
            max_retained: 64,
            output_bytes: 256 * 1024,
            max_runtime: Duration::from_secs(3600),
            env: Vec::new(),
            inherited_env: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessScope {
    Run,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    Running,
    Exited,
    Stopped,
    Cancelled,
    TimedOut,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub id: String,
    pub run_id: String,
    pub state: ProcessState,
    pub scope: ProcessScope,
    pub pid: u32,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub output: String,
    /// Byte cursor; keep this and send it on the next read. It never resets.
    pub next_cursor: u64,
    pub dropped_bytes: u64,
    pub has_more: bool,
}

struct State {
    status: ProcessState,
    stop: Option<ProcessState>,
    exit_code: Option<i32>,
    error: Option<String>,
    output: VecDeque<u8>,
    first: u64,
    end: u64,
    input_cancel: Option<(Arc<AtomicBool>, Arc<AtomicBool>)>,
}

struct Shared {
    id: String,
    order: u64,
    session: String,
    run_id: String,
    scope: ProcessScope,
    pid: u32,
    state: Mutex<State>,
    changed: Condvar,
    stdin: Mutex<Option<ChildStdin>>,
    capacity: usize,
}

struct Entry {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
}

pub struct ProcessManager {
    root: PathBuf,
    config: ProcessConfig,
    entries: Mutex<HashMap<String, Entry>>,
    closed: AtomicBool,
}

impl ProcessManager {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, String> {
        Self::with_config(root, ProcessConfig::default())
    }

    pub fn with_config(root: impl AsRef<Path>, config: ProcessConfig) -> Result<Self, String> {
        let root = root
            .as_ref()
            .canonicalize()
            .map_err(|e| format!("process workspace: {e}"))?;
        if !root.is_dir()
            || config.max_running == 0
            || config.max_retained < config.max_running
            || config.output_bytes < 1024
            || config.max_runtime.as_secs() == 0
        {
            return Err("invalid process workspace or resource limits".into());
        }
        Ok(Self {
            root,
            config,
            entries: Mutex::new(HashMap::new()),
            closed: AtomicBool::new(false),
        })
    }

    pub fn workspace(&self) -> &Path {
        &self.root
    }

    pub fn start(
        &self,
        ctx: &ToolCtx,
        command: &str,
        scope: ProcessScope,
        timeout: Duration,
    ) -> Result<ProcessSnapshot, String> {
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        if command.trim().is_empty()
            || command.chars().count() > 8000
            || timeout.is_zero()
            || timeout > self.config.max_runtime
        {
            return Err("invalid command or process timeout".into());
        }
        let mut entries = lock(&self.entries);
        if self.closed.load(Ordering::SeqCst) {
            return Err("process manager is closed".into());
        }
        if entries
            .values()
            .filter(|entry| lock(&entry.shared.state).status == ProcessState::Running)
            .count()
            >= self.config.max_running
        {
            return Err(
                "running process limit reached; stop a process before starting another".into(),
            );
        }
        while entries.len() >= self.config.max_retained {
            let oldest = entries
                .iter()
                .filter(|(_, entry)| lock(&entry.shared.state).status != ProcessState::Running)
                .min_by_key(|(_, entry)| entry.shared.order)
                .map(|(id, _)| id.clone());
            let Some(id) = oldest else {
                return Err("process retention limit reached".into());
            };
            if let Some(mut old) = entries.remove(&id) {
                if let Some(worker) = old.worker.take() {
                    let _ = worker.join();
                }
            }
        }
        #[cfg(unix)]
        let mut cmd = {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
            cmd
        };
        #[cfg(windows)]
        let mut cmd = {
            use std::os::windows::process::CommandExt;
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
            cmd
        };
        cmd.current_dir(&self.root)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for key in BASE_ENV
            .iter()
            .copied()
            .chain(self.config.inherited_env.iter().map(String::as_str))
        {
            if let Some(value) = std::env::var_os(key) {
                cmd.env(key, value);
            }
        }
        cmd.envs(self.config.env.iter().map(|(k, v)| (k, v)));
        let mut owned = ManagedChild::spawn(&mut cmd)?;
        let id = crate::runtime::new_id("p");
        let shared = Arc::new(Shared {
            id: id.clone(),
            order: PROCESS_SEQUENCE.fetch_add(1, Ordering::Relaxed),
            session: ctx.session().into(),
            run_id: ctx.run_id().into(),
            scope,
            pid: owned.child.id(),
            capacity: self.config.output_bytes,
            stdin: Mutex::new(owned.child.stdin.take()),
            changed: Condvar::new(),
            state: Mutex::new(State {
                status: ProcessState::Running,
                stop: None,
                exit_code: None,
                error: None,
                output: VecDeque::new(),
                first: 0,
                end: 0,
                input_cancel: None,
            }),
        });
        let mut readers = Vec::new();
        if let Some(stdout) = owned.child.stdout.take() {
            readers.push(reader(stdout, shared.clone())?);
        }
        if let Some(stderr) = owned.child.stderr.take() {
            readers.push(reader(stderr, shared.clone())?);
        }
        let worker_shared = shared.clone();
        let run_cancel = ctx.run_cancel.clone();
        let worker = thread::Builder::new()
            .name(format!("process-{id}"))
            .spawn(move || supervise(owned, worker_shared, readers, run_cancel, timeout))
            .map_err(|e| format!("cannot supervise process: {e}"))?;
        entries.insert(
            id,
            Entry {
                shared: shared.clone(),
                worker: Some(worker),
            },
        );
        drop(entries);
        if ctx.is_cancelled() {
            request_stop(&shared, ProcessState::Cancelled);
        }
        snapshot(&shared, 0, 16_000)
    }

    fn owned(&self, ctx: &ToolCtx, id: &str) -> Result<Arc<Shared>, String> {
        let entries = lock(&self.entries);
        let entry = entries
            .get(id)
            .filter(|entry| entry.shared.session == ctx.session())
            .ok_or("unknown process in this session/workspace")?;
        Ok(entry.shared.clone())
    }

    pub fn read(
        &self,
        ctx: &ToolCtx,
        id: &str,
        cursor: u64,
        limit: usize,
        wait: Duration,
    ) -> Result<ProcessSnapshot, String> {
        let process = self.owned(ctx, id)?;
        let end = Instant::now() + wait.min(Duration::from_secs(10));
        let mut state = lock(&process.state);
        while state.status == ProcessState::Running && Instant::now() < end && !ctx.is_cancelled() {
            let result = process
                .changed
                .wait_timeout(state, POLL)
                .unwrap_or_else(|e| e.into_inner());
            state = result.0;
        }
        drop(state);
        snapshot(&process, cursor, limit.clamp(4, 32_000))
    }

    /// Pipe input, not a PTY. Input is bounded; stopping the child unblocks a blocked pipe write.
    pub fn write(&self, ctx: &ToolCtx, id: &str, input: &str, close: bool) -> Result<(), String> {
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        if input.len() > 8192 {
            return Err("process input exceeds 8 KiB".into());
        }
        let process = self.owned(ctx, id)?;
        if lock(&process.state).status != ProcessState::Running {
            return Err("process is no longer running".into());
        }
        let mut stdin = lock(&process.stdin);
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        let pipe = stdin.as_mut().ok_or("process input is closed")?;
        lock(&process.state).input_cancel = Some((ctx.run_cancel.clone(), ctx.call_cancel.clone()));
        let written = pipe
            .write_all(input.as_bytes())
            .and_then(|_| pipe.flush())
            .map_err(|e| format!("process input: {e}"));
        lock(&process.state).input_cancel = None;
        written?;
        if close {
            stdin.take();
        }
        Ok(())
    }

    pub fn stop(&self, ctx: &ToolCtx, id: &str) -> Result<ProcessSnapshot, String> {
        let process = self.owned(ctx, id)?;
        request_stop(&process, ProcessState::Stopped);
        if !wait_stopped(&process) {
            return Err("process has not confirmed termination within three seconds; query its status before retrying".into());
        }
        snapshot(&process, 0, 16_000)
    }

    pub fn stop_session(&self, session: &str) -> Result<(), String> {
        self.stop_matching(|process| process.session == session)
    }

    pub fn forget_session(&self, session: &str) -> Result<(), String> {
        self.stop_session(session)?;
        lock(&self.entries).retain(|_, entry| entry.shared.session != session);
        Ok(())
    }

    pub fn finish_run(&self, end: &RunEnd) -> Result<(), String> {
        if end.ephemeral_session {
            return self.forget_session(&end.session_id);
        }
        self.stop_matching(|process| {
            process.session == end.session_id
                && process.run_id == end.run_id
                && (process.scope == ProcessScope::Run || end.state != RunState::Completed)
        })
    }

    fn stop_matching(&self, matches: impl Fn(&Shared) -> bool) -> Result<(), String> {
        let processes: Vec<_> = lock(&self.entries)
            .values()
            .filter(|entry| matches(&entry.shared))
            .map(|entry| entry.shared.clone())
            .collect();
        for process in &processes {
            request_stop(process, ProcessState::Stopped);
        }
        let pending: Vec<_> = processes
            .iter()
            .filter(|process| !wait_stopped(process))
            .map(|process| process.id.clone())
            .collect();
        if pending.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "process termination was not confirmed: {}",
                pending.join(", ")
            ))
        }
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.closed.store(true, Ordering::SeqCst);
        self.stop_matching(|_| true)
    }

    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        let start = ToolSpec::new("process_start", "Start a foreground shell command and return its process id immediately. Read incremental logs with process_read. No PTY. Default run scope stops at the end of this answer; keep_alive=true retains it for this conversation after a successful answer. Cancellation and timeout still stop it.", json!({
            "type":"object", "properties":{"command":{"type":"string"},"keep_alive":{"type":"boolean"},"timeout_seconds":{"type":"integer","minimum":1,"maximum":self.config.max_runtime.as_secs()}},"required":["command"],"additionalProperties":false
        }));
        let read = ToolSpec::new("process_read", "Collect a process's new output and status. Reuse next_cursor to avoid duplicate logs. wait_ms collects output for up to ten seconds, returning early when the process exits; use 10000 for long builds to avoid rapid polling. dropped_bytes reports evicted output.", json!({
            "type":"object","properties":{"id":{"type":"string"},"cursor":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":4,"maximum":32000},"wait_ms":{"type":"integer","minimum":0,"maximum":10000}},"required":["id"],"additionalProperties":false
        })).risk(Risk::Read).parallel().max_output(64_000);
        let write = ToolSpec::new("process_write", "Send up to 8 KiB of pipe input to a process. Include a newline if needed. close_stdin=true sends EOF. This executes input in another process and requires execution permission.", json!({
            "type":"object","properties":{"id":{"type":"string"},"input":{"type":"string"},"close_stdin":{"type":"boolean"}},"required":["id","input"],"additionalProperties":false
        }));
        let stop = ToolSpec::new("process_stop", "Stop a process and its managed descendants. Already stopped processes are safe to query again.", json!({"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false})).max_output(64_000);
        [start, read, write, stop]
            .into_iter()
            .enumerate()
            .map(|(kind, spec)| {
                Arc::new(ProcessTool {
                    manager: self.clone(),
                    kind,
                    spec,
                }) as Arc<dyn Tool>
            })
            .collect()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        if !self.closed.load(Ordering::SeqCst) {
            let _ = self.shutdown();
        }
        for (_, mut entry) in lock(&self.entries).drain() {
            if let Some(worker) = entry.worker.take() {
                if worker.is_finished() {
                    let _ = worker.join();
                }
            }
        }
    }
}

struct ProcessTool {
    manager: Arc<ProcessManager>,
    kind: usize,
    spec: ToolSpec,
}

impl Tool for ProcessTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        let result = (|| -> Result<Value, String> {
            let id = || {
                args.get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "id is required".to_string())
            };
            match self.kind {
                0 => {
                    let command = args
                        .get("command")
                        .and_then(Value::as_str)
                        .ok_or("command is required")?;
                    let scope = if args
                        .get("keep_alive")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        ProcessScope::Session
                    } else {
                        ProcessScope::Run
                    };
                    let timeout = Duration::from_secs(
                        args.get("timeout_seconds")
                            .and_then(Value::as_u64)
                            .unwrap_or(600)
                            .min(self.manager.config.max_runtime.as_secs()),
                    );
                    serde_json::to_value(self.manager.start(ctx, command, scope, timeout)?)
                        .map_err(|e| e.to_string())
                }
                1 => serde_json::to_value(self.manager.read(
                    ctx,
                    id()?,
                    args["cursor"].as_u64().unwrap_or(0),
                    args["limit"].as_u64().unwrap_or(16_000) as usize,
                    Duration::from_millis(args["wait_ms"].as_u64().unwrap_or(0)),
                )?)
                .map_err(|e| e.to_string()),
                2 => {
                    self.manager.write(
                        ctx,
                        id()?,
                        args["input"].as_str().ok_or("input is required")?,
                        args["close_stdin"].as_bool().unwrap_or(false),
                    )?;
                    Ok(json!({"id":id()?,"input_written":true}))
                }
                _ => {
                    serde_json::to_value(self.manager.stop(ctx, id()?)?).map_err(|e| e.to_string())
                }
            }
        })();
        match result {
            Ok(mut value) => {
                // Logs remain text so SpillOutput can handle them; metadata remains small JSON.
                let output = value
                    .as_object_mut()
                    .and_then(|v| v.remove("output"))
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default();
                ToolResult::ok(output).with_structured_content(value)
            }
            Err(error) => ToolResult::error(error),
        }
    }

    fn on_run_end(&self, end: &RunEnd) -> Result<(), String> {
        if self.kind == 0 {
            self.manager.finish_run(end)
        } else {
            Ok(())
        }
    }
}

fn snapshot(process: &Shared, cursor: u64, limit: usize) -> Result<ProcessSnapshot, String> {
    let state = lock(&process.state);
    if cursor > state.end {
        return Err("cursor is beyond the process output".into());
    }
    let mut start = cursor.max(state.first);
    let offset = (start - state.first) as usize;
    let mut data: Vec<u8> = state
        .output
        .iter()
        .skip(offset)
        .take(limit)
        .copied()
        .collect();
    // An evicted/caller-supplied byte cursor may lie inside a UTF-8 character.
    while data.first().is_some_and(|byte| byte & 0xc0 == 0x80) {
        data.remove(0);
        start += 1;
    }
    if let Err(error) = std::str::from_utf8(&data) {
        if error.error_len().is_none()
            && (state.status == ProcessState::Running || start + (data.len() as u64) < state.end)
        {
            data.truncate(error.valid_up_to());
        }
    }
    let next = start + data.len() as u64;
    Ok(ProcessSnapshot {
        id: process.id.clone(),
        run_id: process.run_id.clone(),
        state: state.status,
        scope: process.scope,
        pid: process.pid,
        exit_code: state.exit_code,
        error: state.error.clone(),
        output: String::from_utf8_lossy(&data).into_owned(),
        next_cursor: next,
        dropped_bytes: start.saturating_sub(cursor),
        has_more: next < state.end,
    })
}

fn reader(
    mut source: impl Read + Send + 'static,
    shared: Arc<Shared>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name("process-output".into())
        .spawn(move || {
            let mut bytes = [0u8; 8192];
            loop {
                match source.read(&mut bytes) {
                    Ok(0) => break,
                    Err(error) => {
                        lock(&shared.state).error =
                            Some(format!("cannot read process output: {error}"));
                        break;
                    }
                    Ok(n) => {
                        let mut state = lock(&shared.state);
                        state.output.extend(&bytes[..n]);
                        state.end += n as u64;
                        if state.output.len() > shared.capacity {
                            let discard = state.output.len() - shared.capacity;
                            state.output.drain(..discard);
                            state.first += discard as u64;
                        }
                        shared.changed.notify_all();
                    }
                }
            }
        })
        .map_err(|e| format!("cannot start process output reader: {e}"))
}

fn request_stop(process: &Shared, reason: ProcessState) {
    let mut state = lock(&process.state);
    if state.status == ProcessState::Running && state.stop.is_none() {
        state.stop = Some(reason);
    }
    process.changed.notify_all();
}

fn wait_stopped(process: &Shared) -> bool {
    let end = Instant::now() + Duration::from_secs(3);
    let mut state = lock(&process.state);
    while state.status == ProcessState::Running && Instant::now() < end {
        state = process
            .changed
            .wait_timeout(state, POLL)
            .unwrap_or_else(|e| e.into_inner())
            .0;
    }
    state.status != ProcessState::Running
}

fn supervise(
    mut owned: ManagedChild,
    shared: Arc<Shared>,
    readers: Vec<JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
    timeout: Duration,
) {
    let started = Instant::now();
    let (status, exit, mut error) = loop {
        match owned.child.try_wait() {
            Ok(Some(status)) => break (ProcessState::Exited, Some(status), None),
            Err(error) => {
                break (
                    ProcessState::Failed,
                    None,
                    Some(format!("cannot wait for process: {error}")),
                )
            }
            Ok(None) => {}
        }
        let input_cancelled = lock(&shared.state)
            .input_cancel
            .as_ref()
            .is_some_and(|(run, call)| run.load(Ordering::SeqCst) || call.load(Ordering::SeqCst));
        let stop = if cancel.load(Ordering::SeqCst) || input_cancelled {
            Some(ProcessState::Cancelled)
        } else if started.elapsed() >= timeout {
            Some(ProcessState::TimedOut)
        } else {
            lock(&shared.state).stop
        };
        if let Some(reason) = stop {
            owned.terminate();
            break (reason, owned.child.wait().ok(), None);
        }
        thread::sleep(POLL);
    };
    owned.terminate(); // Also close descendants left behind by an exited shell.
    let _ = owned.child.wait();
    lock(&shared.stdin).take();
    let end = Instant::now() + Duration::from_secs(1);
    while !readers.iter().all(JoinHandle::is_finished) && Instant::now() < end {
        thread::sleep(POLL);
    }
    for reader in readers {
        if reader.is_finished() {
            if reader.join().is_err() {
                error = Some("process output reader panicked".into());
            }
        } else {
            error = Some("a descendant kept an output pipe open after termination".into());
        }
    }
    let mut state = lock(&shared.state);
    let error = error.or_else(|| state.error.take());
    state.status = if error.is_some() {
        ProcessState::Failed
    } else {
        status
    };
    state.exit_code = exit.and_then(|status| status.code());
    state.error = error;
    shared.changed.notify_all();
}

/// Owns termination even if supervision cannot be spawned. Never silently fall back when a
/// Windows job cannot be attached; the child is killed and start returns an error.
struct ManagedChild {
    child: Child,
    armed: bool,
    #[cfg(windows)]
    job: Job,
}

impl ManagedChild {
    fn spawn(command: &mut Command) -> Result<Self, String> {
        #[cfg(windows)]
        let job = Job::new()?;
        #[allow(unused_mut)]
        let mut child = command
            .spawn()
            .map_err(|e| format!("cannot start process: {e}"))?;
        #[cfg(windows)]
        if let Err(error) = job.attach(&child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        Ok(Self {
            child,
            armed: true,
            #[cfg(windows)]
            job,
        })
    }

    fn terminate(&mut self) {
        if !self.armed {
            return;
        }
        self.armed = false;
        #[cfg(unix)]
        unsafe {
            libc::killpg(self.child.id() as libc::pid_t, libc::SIGKILL);
        }
        #[cfg(windows)]
        self.job.terminate();
        let _ = self.child.kill();
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        self.terminate();
        let _ = self.child.wait();
    }
}

#[cfg(windows)]
struct Job(usize);

#[cfg(windows)]
impl Job {
    fn new() -> Result<Self, String> {
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return Err(format!(
                    "cannot create process job: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let job = Self(handle as usize);
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const _,
                std::mem::size_of_val(&limits) as u32,
            ) == 0
            {
                return Err(format!(
                    "cannot configure process job: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(job)
        }
    }
    fn attach(&self, child: &Child) -> Result<(), String> {
        use std::os::windows::io::AsRawHandle;
        unsafe {
            if windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(
                self.0 as _,
                child.as_raw_handle() as _,
            ) == 0
            {
                return Err(format!(
                    "cannot attach process job: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }
        Ok(())
    }
    fn terminate(&self) {
        unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.0 as _, 1);
        }
    }
}
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0 as _);
        }
    }
}

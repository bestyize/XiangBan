//! `shell`: run one command line and return its output.
//!
//! This is not a sandbox. The command runs with the host process's privileges, so the tool is
//! `Risk::Exec`: install `Permission` and keep the default of asking every time. What it does limit:
//! the environment is cleared (host secrets such as API keys do not leak to the command), stdin is
//! closed, output is bounded, and on timeout or cancel the whole process group is killed.
//! Background processes a command leaves behind are killed when it finishes.
//! Unix is tested; the Windows path (`cmd /C`, `taskkill`) has not been run.

use super::str_arg;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const HEAD_BYTES: usize = 64 * 1024;
const TAIL_BYTES: usize = 192 * 1024;
const MAX_COMMAND_CHARS: usize = 8000;
const READER_GRACE: Duration = Duration::from_millis(500);
const POLL: Duration = Duration::from_millis(20);


pub struct Shell {
    spec: ToolSpec,
    cwd: PathBuf,
    env: Vec<(String, String)>,
    extra_inherited: Vec<String>,
}

impl Shell {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": { "command": { "type": "string", "description": "One command line, run by the system shell." } },
            "required": ["command"]
        });
        let description = "Run a shell command in the working directory and return its combined output and exit code. \
                           There is no terminal and no input, so never start interactive programs. Prefer the file tools \
                           for reading and editing files.";
        let spec = ToolSpec::new("shell", description, schema).risk(Risk::Exec).serial().timeout(Duration::from_secs(120));
        Self { spec, cwd: cwd.into(), env: Vec::new(), extra_inherited: Vec::new() }
    }

    /// Sets one variable for the command. Everything not listed here or passed through is removed.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    /// Passes this process's value of `key` through to the command.
    pub fn inherit_env(mut self, key: &str) -> Self {
        self.extra_inherited.push(key.to_string());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.spec.timeout = timeout;
        self
    }

    pub fn max_output(mut self, bytes: usize) -> Self {
        self.spec.max_output = bytes;
        self
    }
}

/// Keeps the start and the end of the output; the middle is counted and dropped.
#[derive(Default)]
struct Capture {
    head: Vec<u8>,
    tail: VecDeque<u8>,
    dropped: u64,
}

impl Capture {
    fn push(&mut self, mut chunk: &[u8]) {
        if self.head.len() < HEAD_BYTES {
            let take = (HEAD_BYTES - self.head.len()).min(chunk.len());
            self.head.extend_from_slice(&chunk[..take]);
            chunk = &chunk[take..];
        }
        self.tail.extend(chunk);
        if self.tail.len() > TAIL_BYTES {
            let excess = self.tail.len() - TAIL_BYTES;
            self.tail.drain(..excess);
            self.dropped += excess as u64;
        }
    }

    fn text(&self) -> String {
        let head = String::from_utf8_lossy(&self.head);
        let tail_bytes: Vec<u8> = self.tail.iter().copied().collect();
        let tail = String::from_utf8_lossy(&tail_bytes);
        if self.dropped > 0 {
            format!("{head}\n[... {} bytes of output omitted ...]\n{tail}", self.dropped)
        } else {
            format!("{head}{tail}")
        }
    }
}

fn spawn_reader(mut source: impl Read + Send + 'static, capture: Arc<Mutex<Capture>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match source.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => capture.lock().unwrap_or_else(|e| e.into_inner()).push(&buf[..n]),
            }
        }
    })
}

fn kill_tree(pid: u32) {
    #[cfg(unix)]
    let _ = Command::new("sh").args(["-c", &format!("kill -KILL -- -{pid}")]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
    #[cfg(windows)]
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

fn describe(status: ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit code: {code}"),
        None => "terminated by a signal".to_string(),
    }
}

impl Tool for Shell {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let command = str_arg(&args, "command")?;
        if command.trim().is_empty() || command.chars().count() > MAX_COMMAND_CHARS {
            return Err(format!("`command` must be 1 to {MAX_COMMAND_CHARS} characters"));
        }

        #[cfg(windows)]
        let mut cmd = {
            let mut c = crate::child_env::windows_shell(command);
            crate::child_env::no_window(&mut c);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.args(["-c", command]);
            c
        };
        cmd.current_dir(&self.cwd).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        crate::child_env::apply(&mut cmd, &self.extra_inherited);
        cmd.envs(self.env.iter().map(|(k, v)| (k, v)));
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);

        let mut child = cmd.spawn().map_err(|e| format!("cannot start the shell in {}: {e}", self.cwd.display()))?;
        let pid = child.id();
        let capture = Arc::new(Mutex::new(Capture::default()));
        let readers: Vec<JoinHandle<()>> = [
            child.stdout.take().map(|s| spawn_reader(s, capture.clone())),
            child.stderr.take().map(|s| spawn_reader(s, capture.clone())),
        ]
        .into_iter()
        .flatten()
        .collect();

        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if ctx.is_cancelled() => break None,
                Ok(None) => thread::sleep(POLL),
                Err(e) => return Err(format!("cannot wait for the command: {e}")),
            }
        };
        if status.is_none() {
            kill_tree(pid);
            let _ = child.kill();
            let _ = child.wait();
        }

        let wait_for_readers = |limit: Duration| {
            let end = Instant::now() + limit;
            while Instant::now() < end && !readers.iter().all(JoinHandle::is_finished) {
                thread::sleep(Duration::from_millis(10));
            }
            readers.iter().all(JoinHandle::is_finished)
        };
        if !wait_for_readers(READER_GRACE) {
            kill_tree(pid);
            wait_for_readers(Duration::from_secs(1));
        }

        let output = capture.lock().unwrap_or_else(|e| e.into_inner()).text();
        let output = output.trim_end();
        match status {
            None => Err(format!("{output}\n[stopped: the command was cancelled or timed out and was killed]").trim_start().to_string()),
            Some(status) if status.success() => Ok(if output.is_empty() { "(no output)\n[exit code: 0]".to_string() } else { format!("{output}\n[exit code: 0]") }),
            Some(status) => Err(format!("{output}\n[{}]", describe(status)).trim_start().to_string()),
        }
    }
}

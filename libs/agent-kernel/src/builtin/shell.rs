//! `shell`: run one command line and return its output.
//!
//! This is not a sandbox. The command runs with the host process's privileges, so the tool is
//! `Risk::Exec`: install `Permission` and keep the default of asking every time. What it does limit:
//! the environment is cleared (host secrets such as API keys do not leak to the command), stdin is
//! closed, output is bounded, and on timeout or cancel the whole process group is killed.
//! Background processes a command leaves behind are killed when it finishes.
//! On Windows, output that is not
//! UTF-8 is decoded from the OEM code page.

use super::str_arg;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::Read;
use std::path::PathBuf;
#[cfg(windows)]
use std::process::Command;
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const HEAD_BYTES: usize = 64 * 1024;
const TAIL_BYTES: usize = 192 * 1024;
const MAX_COMMAND_CHARS: usize = 8000;
const READER_GRACE: Duration = Duration::from_millis(500);
const POLL: Duration = Duration::from_millis(20);
#[cfg(windows)]
const SHELL_NOTE: &str = "The shell is Windows cmd.exe (`cmd /C`), not bash or sh: use cmd syntax such as `&&`, `if exist`, \
                          `where`, `dir`, `%VAR%` and `for %i in (a b) do ...`; a call must be ONE line (cmd ignores everything after the first newline, so never send a multi-line `python -c`; write a file and run it); never bash syntax such as `for x in ...; do`, \
                          `command -v`, `ls`, `$(...)` or `$VAR`. Python defaults to UTF-8. For PowerShell UTF-8 files use \
                          `Get-Content -Encoding UTF8`; prefer read_file or Python over findstr to validate UTF-8 documents.";
#[cfg(not(windows))]
const SHELL_NOTE: &str = "The shell is /bin/sh.";


/// `cmd /C` runs only the first line of a command: everything after the first newline is silently dropped and the call
/// still reports success, so a multi-line `python -c "…"` looks like it ran. Refuse it, and say what to do instead.
#[cfg_attr(not(windows), allow(dead_code))]
fn multi_line_error(command: &str) -> Option<String> {
    command.trim_end().contains(['\n', '\r']).then(|| {
        "The command has several lines, but cmd.exe runs only the first one and ignores the rest, so nothing useful would run. Nothing was executed. Write the script to a file with write_file and run that file (for example `python script.py`), or join steps on one line with `&&`.".to_string()
    })
}

pub struct Shell {
    spec: ToolSpec,
    cwd: PathBuf,
    env: Vec<(String, String)>,
    extra_inherited: Vec<String>,
    sandbox: Option<crate::sandbox::SandboxHook>,
}

impl Shell {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": format!("One command line, run by the system shell, at most {MAX_COMMAND_CHARS} characters. Put a long script in a file instead.") },
                "outside_sandbox": { "type": "boolean", "description": crate::sandbox::OUTSIDE_SANDBOX_ARG }
            },
            "required": ["command"]
        });
        let description = format!(
            "Run a shell command in the working directory and return its combined output and exit code. {SHELL_NOTE} \
             There is no terminal and no input, so never start interactive programs. Prefer the file tools \
             for reading and editing files. Each call starts in the working directory; `cd` only affects that call."
        );
        let spec = ToolSpec::new("shell", &description, schema).risk(Risk::Exec).serial().timeout(Duration::from_secs(120));
        Self { spec, cwd: cwd.into(), env: Vec::new(), extra_inherited: Vec::new(), sandbox: None }
    }

    /// Runs commands for which `hook` returns a policy inside the OS sandbox (see `crate::sandbox`).
    pub fn sandbox(mut self, hook: crate::sandbox::SandboxHook) -> Self {
        self.sandbox = Some(hook);
        self
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
        let head = decode(&self.head);
        let tail_bytes: Vec<u8> = self.tail.iter().copied().collect();
        let tail = decode(&tail_bytes);
        if self.dropped > 0 {
            format!("{head}\n[... {} bytes of output omitted ...]\n{tail}", self.dropped)
        } else {
            format!("{head}{tail}")
        }
    }
}

/// UTF-8 when the bytes are UTF-8. Otherwise, on Windows, the OEM code page that `cmd` and console programs
/// write to a pipe (GBK on Chinese systems), so their messages are not turned into replacement characters.
fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        #[cfg(windows)]
        Err(_) => oem_decode(bytes).unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned()),
        #[cfg(not(windows))]
        Err(_) => String::from_utf8_lossy(bytes).into_owned(),
    }
}

#[cfg(windows)]
fn oem_decode(bytes: &[u8]) -> Option<String> {
    use windows_sys::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP};
    let len = i32::try_from(bytes.len()).ok()?;
    // SAFETY: the pointers and lengths describe live buffers; a null output asks for the needed size.
    let needed = unsafe { MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), len, std::ptr::null_mut(), 0) };
    if needed <= 0 {
        return None;
    }
    let mut wide = vec![0u16; needed as usize];
    let written = unsafe { MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), len, wide.as_mut_ptr(), needed) };
    (written > 0).then(|| String::from_utf16_lossy(&wide[..written as usize]))
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

fn kill_tree(pid: u32) { crate::process_tree::terminate(pid); }

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

    /// Killing the process tree (`taskkill` twice on Windows) and draining the pipes can take a couple of
    /// seconds, but it always finishes. Reporting "outcome unknown" for a command we just killed would
    /// pause the run for no reason.
    fn cancel_grace(&self) -> Duration {
        Duration::from_secs(4)
    }

    /// Commands take no shared mutation lease, so conversations can run commands at the same time.
    /// Like Claude Code and Hermes, shell isolation is left to the working folder (use a worktree) and the
    /// permission prompt; file writes still take their per-file lease.
    fn resource(&self, _args: &Value) -> Result<Option<crate::ToolResource>, String> {
        Ok(None)
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let command = str_arg(&args, "command")?;
        if command.trim().is_empty() || command.chars().count() > MAX_COMMAND_CHARS {
            return Err(format!("`command` must be 1 to {MAX_COMMAND_CHARS} characters"));
        }

        #[cfg(windows)]
        if let Some(error) = multi_line_error(command) {
            return Err(error);
        }
        let policy = self.sandbox.as_ref().and_then(|hook| hook.policy(ctx.session(), command));
        #[cfg(windows)]
        let mut cmd = {
            let mut c = crate::sandbox::windows_shell(command, policy.as_ref())?;
            crate::child_env::no_window(&mut c);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = crate::sandbox::unix_shell(command, policy.as_ref())?;
        cmd.current_dir(&self.cwd).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        crate::child_env::apply_command(&mut cmd, &self.extra_inherited);
        #[cfg(windows)]
        crate::sandbox::windows_launcher_env(&mut cmd, command, policy.as_ref());
        cmd.envs(self.env.iter().map(|(k, v)| (k, v)));
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);

        let mut child = crate::process_tree::ChildTree::spawn(&mut cmd).map_err(|e| format!("cannot start the shell in {}: {e}", self.cwd.display()))?;
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
        child.terminate();
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
        let blocked = policy.is_some() && crate::sandbox::looks_blocked(output);
        let output = if blocked { format!("{output}\n{}", crate::sandbox::BLOCKED_HINT) } else { output.to_string() };
        let output = output.as_str();
        match status {
            None => Err(format!("{output}\n[stopped: the command was cancelled or timed out and was killed]").trim_start().to_string()),
            Some(status) if status.success() => Ok(if output.is_empty() { "(no output)\n[exit code: 0]".to_string() } else { format!("{output}\n[exit code: 0]") }),
            Some(status) => Err(format!("{output}\n[{}]", describe(status)).trim_start().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::multi_line_error;

    #[test]
    fn a_command_with_a_second_line_is_refused_but_a_trailing_newline_is_fine() {
        assert!(multi_line_error("python -c \"\nprint(1)\n\"").unwrap().contains("Nothing was executed"));
        assert!(multi_line_error("dir\r\ndir").is_some());
        assert!(multi_line_error("python script.py").is_none());
        assert!(multi_line_error("python script.py\n").is_none());
        assert!(multi_line_error("a && b").is_none());
    }
}

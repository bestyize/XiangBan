//! User hooks: commands the user configures to run around the assistant's tool calls, like Claude Code's hooks.
//!
//! `chat/hooks.json` in the app's data folder:
//!
//! ```json
//! { "hooks": [
//!   { "event": "before_tool", "tools": ["shell", "write_file"], "command": "python3 ~/check.py", "timeout_seconds": 10 },
//!   { "event": "after_tool", "tools": ["edit_file"], "command": "npx prettier --check ." }
//! ] }
//! ```
//!
//! The call is given to the command as JSON on standard input (`event`, `session`, `tool`, `arguments`, and for
//! `after_tool` also `output` and `is_error`); the command runs in the working folder with the user's environment.
//! - `before_tool` runs after permission and resource acquisition, before execution. Exit code 0 lets the call go on; anything else (another code,
//!   a timeout, a command that cannot start) stops it, and standard error is what the model is told. A check the
//!   user set up on purpose fails closed.
//! - `after_tool`: a non-zero exit adds standard error to the tool's result, so the model sees the feedback (a
//!   formatter or linter that did not pass).
//!
//! The file is read again for every call, so an edit applies at once. `tools` left out or `"*"` matches every tool.

use agent_kernel::{Middleware, ToolCall, ToolCtx, ToolGate, ToolResult, ToolSpec};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT: u64 = 10;
const MAX_TIMEOUT: u64 = 120;
/// What a hook's error output may add to a block reason or a tool result.
const MAX_FEEDBACK_BYTES: usize = 4 * 1024;
/// How much of a tool's output is handed to an `after_tool` hook.
const MAX_OUTPUT_TO_HOOK: usize = 64 * 1024;
const PIPE_CLEANUP_GRACE: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Event {
    BeforeTool,
    AfterTool,
}

#[derive(Debug, Clone, Deserialize)]
struct Hook {
    event: Event,
    #[serde(default)]
    tools: Vec<String>,
    command: String,
    timeout_seconds: Option<u64>,
}

impl Hook {
    fn matches(&self, tool: &str) -> bool {
        self.tools.is_empty() || self.tools.iter().any(|t| t == "*" || t == tool)
    }
}

#[derive(Debug, Default, Deserialize)]
struct Config {
    #[serde(default)]
    hooks: Vec<Hook>,
}

/// What a hook run came to.
#[derive(Debug, PartialEq)]
enum Outcome {
    Passed,
    Cancelled,
    /// Exit code (or none: timed out, killed), and its standard error.
    Failed(Option<i32>, String),
    CouldNotRun(String),
}

/// Checks hooks configuration text before it is saved; `Ok` with the number of hooks.
pub fn validate(text: &str) -> Result<usize, String> {
    if text.trim().is_empty() {
        return Ok(0);
    }
    let config: Config = serde_json::from_str(text).map_err(|e| format!("不是有效的 hooks 配置：{e}"))?;
    if let Some(hook) = config.hooks.iter().find(|h| h.command.trim().is_empty()) {
        return Err(format!("有一条 {:?} hook 没有填写 command", hook.event));
    }
    Ok(config.hooks.len())
}

pub struct Hooks {
    file: PathBuf,
    workspace: PathBuf,
}

fn clip(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

impl Hooks {
    pub fn new(file: impl Into<PathBuf>, workspace: &Path) -> Self {
        Self { file: file.into(), workspace: workspace.to_path_buf() }
    }

    /// The hooks for this event and tool. A file that cannot be read as hooks makes every call stop, so a broken
    /// safety check is noticed instead of silently doing nothing.
    fn load(&self, event: Event, tool: &str) -> Result<Vec<Hook>, String> {
        let text = match std::fs::read_to_string(&self.file) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("cannot read {}: {e}", self.file.display())),
        };
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let config: Config = serde_json::from_str(&text).map_err(|e| format!("{} is not valid hooks configuration: {e}", self.file.display()))?;
        Ok(config.hooks.into_iter().filter(|h| h.event == event && h.matches(tool)).collect())
    }

    fn run(&self, ctx: &ToolCtx, hook: &Hook, input: &Value) -> Outcome {
        if ctx.is_cancelled() { return Outcome::Cancelled; }
        // The deadline covers input delivery and output draining as well as execution.
        let timeout = Duration::from_secs(hook.timeout_seconds.unwrap_or(DEFAULT_TIMEOUT).clamp(1, MAX_TIMEOUT));
        let end = Instant::now() + timeout;
        #[cfg(windows)]
        let mut cmd = {
            use std::os::windows::process::CommandExt;
            let mut cmd = Command::new("cmd");
            cmd.args(["/D", "/S", "/C"]).raw_arg(format!("\"{}\"", hook.command)).creation_flags(0x0800_0000);
            cmd
        };
        #[cfg(not(windows))]
        let mut cmd = {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", &hook.command]);
            cmd
        };
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
        cmd.current_dir(&self.workspace).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped());
        let mut child = match agent_kernel::process_tree::ChildTree::spawn(&mut cmd) {
            Ok(child) => child,
            Err(e) => return Outcome::CouldNotRun(format!("cannot start hook `{}`: {e}", hook.command)),
        };
        let mut stderr = child.stderr.take();
        let captured = Arc::new(Mutex::new(Vec::new()));
        let output = captured.clone();
        let reader = match std::thread::Builder::new().name("hook-stderr".into()).spawn(move || {
            if let Some(stderr) = stderr.as_mut() {
                let mut buffer = [0; 8192];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(count) => {
                            let mut output = output.lock().unwrap_or_else(|e| e.into_inner());
                            let keep = count.min(MAX_FEEDBACK_BYTES.saturating_sub(output.len()));
                            output.extend_from_slice(&buffer[..keep]);
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }
            }
        }) {
            Ok(reader) => reader,
            Err(error) => return Outcome::CouldNotRun(format!("cannot drain hook output: {error}")),
        };
        let mut stdin = child.stdin.take();
        let input = input.to_string();
        let writer = match std::thread::Builder::new().name("hook-stdin".into()).spawn(move || {
            if let Some(stdin) = stdin.as_mut() {
                // A hook may choose not to consume its input. Do not let a full
                // pipe prevent the supervising thread from enforcing cancellation.
                let _ = stdin.write_all(input.as_bytes());
            }
        }) {
            Ok(writer) => writer,
            Err(error) => return Outcome::CouldNotRun(format!("cannot deliver hook input: {error}")),
        };
        let outcome = loop {
            if ctx.is_cancelled() { break Outcome::Cancelled; }
            if Instant::now() >= end {
                break Outcome::Failed(None, format!("timed out after {}s", timeout.as_secs()));
            }
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break Outcome::Passed,
                Ok(Some(status)) => break Outcome::Failed(status.code(), String::new()),
                Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                Err(e) => break Outcome::CouldNotRun(format!("cannot wait for hook `{}`: {e}", hook.command)),
            }
        };
        // Also terminate descendants when their parent exits normally: inherited
        // pipes must not keep this call alive or perform later background writes.
        child.terminate();
        let _ = child.wait();
        let cleanup_end = Instant::now() + PIPE_CLEANUP_GRACE;
        while (!reader.is_finished() || !writer.is_finished()) && Instant::now() < cleanup_end {
            std::thread::sleep(Duration::from_millis(10));
        }
        let clean = reader.is_finished() && writer.is_finished();
        // A child escaping its group must not turn a bounded hook into an
        // unbounded join. Completed readers/writers are reaped; others detach.
        if reader.is_finished() { let _ = reader.join(); }
        if writer.is_finished() { let _ = writer.join(); }
        let stderr = String::from_utf8_lossy(&captured.lock().unwrap_or_else(|e| e.into_inner())).into_owned();
        let stderr = clip(stderr.trim(), MAX_FEEDBACK_BYTES).to_string();
        if ctx.is_cancelled() { return Outcome::Cancelled; }
        match outcome {
            Outcome::Passed if !clean => Outcome::Failed(None, "hook left pipes open after process cleanup".into()),
            Outcome::Failed(code, reason) => Outcome::Failed(code, [reason, stderr].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(". ")),
            other => other,
        }
    }
}

impl Middleware for Hooks {
    fn before_tool(&self, ctx: &ToolCtx, _call: &ToolCall, args: &Value, spec: &ToolSpec) -> ToolGate {
        let hooks = match self.load(Event::BeforeTool, &spec.name) {
            Ok(hooks) => hooks,
            Err(error) => return ToolGate::Block(format!("Stopped by the user's hooks configuration: {error}")),
        };
        let input = json!({ "event": "before_tool", "session": ctx.session(), "tool": spec.name, "arguments": args });
        for hook in &hooks {
            match self.run(ctx, hook, &input) {
                Outcome::Passed => {}
                Outcome::Cancelled => return ToolGate::Block("The user's hook was cancelled; this tool was not executed.".into()),
                Outcome::Failed(code, stderr) => {
                    let why = if stderr.is_empty() { code.map_or("it did not finish".to_string(), |c| format!("it exited with code {c}")) } else { stderr };
                    return ToolGate::Block(format!("The user's hook `{}` stopped this call: {why}", hook.command));
                }
                Outcome::CouldNotRun(error) => return ToolGate::Block(format!("Stopped because a user hook could not run: {error}")),
            }
        }
        ToolGate::Proceed
    }

    fn after_tool(&self, session: &str, call: &ToolCall, result: &mut ToolResult) {
        self.after_tool_with_context(&ToolCtx::detached(session), call, result);
    }

    fn after_tool_with_context(&self, ctx: &ToolCtx, call: &ToolCall, result: &mut ToolResult) {
        if ctx.is_cancelled() { return; }
        let Ok(hooks) = self.load(Event::AfterTool, &call.name) else { return };
        if hooks.is_empty() {
            return;
        }
        let arguments: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::String(call.arguments.clone()));
        let input = json!({
            "event": "after_tool", "session": ctx.session(), "tool": call.name, "arguments": arguments,
            "output": clip(&result.output, MAX_OUTPUT_TO_HOOK), "is_error": result.is_error,
        });
        for hook in &hooks {
            let feedback = match self.run(ctx, hook, &input) {
                Outcome::Passed => continue,
                Outcome::Cancelled => {
                    result.output.push_str("\n[hook: cancelled; the tool result above is unchanged]");
                    break;
                }
                Outcome::Failed(_, stderr) if stderr.is_empty() => format!("the user's hook `{}` reported a problem", hook.command),
                Outcome::Failed(_, stderr) => format!("the user's hook `{}` reported: {stderr}", hook.command),
                Outcome::CouldNotRun(error) => error,
            };
            result.output.push_str(&format!("\n[hook: {feedback}]"));
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn setup(tag: &str, config: Value) -> (PathBuf, Hooks) {
        let dir = std::env::temp_dir().join(format!("xiaoban-hooks-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("work")).unwrap();
        std::fs::write(dir.join("hooks.json"), config.to_string()).unwrap();
        let hooks = Hooks::new(dir.join("hooks.json"), &dir.join("work"));
        (dir, hooks)
    }

    fn hook(event: &str, tools: &[&str], command: &str) -> Value {
        json!({ "event": event, "tools": tools, "command": command, "timeout_seconds": 2 })
    }

    fn before(hooks: &Hooks, tool: &str, args: Value) -> ToolGate {
        let spec = ToolSpec::new(tool, "", json!({"type": "object"}));
        let call = ToolCall { id: "c".into(), name: tool.into(), arguments: args.to_string() };
        hooks.before_tool(&ToolCtx::detached("s1"), &call, &args, &spec)
    }

    #[test]
    fn a_before_hook_sees_the_call_and_can_stop_it() {
        let (dir, hooks) = setup("before", json!({ "hooks": [
            hook("before_tool", &["shell"], "cat > seen.json; grep -q 'rm -rf' seen.json && { echo '不允许删除' >&2; exit 2; }; exit 0"),
        ]}));
        assert!(matches!(before(&hooks, "shell", json!({"command": "ls"})), ToolGate::Proceed));
        let seen: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("work/seen.json")).unwrap()).unwrap();
        assert_eq!(seen, json!({"event": "before_tool", "session": "s1", "tool": "shell", "arguments": {"command": "ls"}}), "runs in the working folder with the call on stdin");
        match before(&hooks, "shell", json!({"command": "rm -rf x"})) {
            ToolGate::Block(reason) => assert!(reason.contains("不允许删除"), "{reason}"),
            ToolGate::Proceed => panic!("should be stopped"),
        }
        assert!(matches!(before(&hooks, "write_file", json!({"path": "rm -rf"})), ToolGate::Proceed), "other tools are not matched");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_broken_hook_or_configuration_fails_closed_and_no_file_means_no_hooks() {
        let (dir, hooks) = setup("broken", json!({ "hooks": [hook("before_tool", &["*"], "sleep 5")] }));
        assert!(matches!(before(&hooks, "read_file", json!({})), ToolGate::Block(r) if r.contains("timed out")));
        std::fs::write(dir.join("hooks.json"), "{ not json").unwrap();
        assert!(matches!(before(&hooks, "read_file", json!({})), ToolGate::Block(r) if r.contains("not valid hooks configuration")));
        std::fs::remove_file(dir.join("hooks.json")).unwrap();
        assert!(matches!(before(&hooks, "read_file", json!({})), ToolGate::Proceed));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_after_hook_adds_its_feedback_to_the_result() {
        let (dir, hooks) = setup("after", json!({ "hooks": [
            hook("after_tool", &["edit_file"], "grep -q '\"is_error\":false' && { echo '格式检查未通过：a.rs 第 3 行' >&2; exit 1; }; exit 0"),
            hook("after_tool", &["edit_file"], "exit 0"),
        ]}));
        let call = ToolCall { id: "c".into(), name: "edit_file".into(), arguments: json!({"path": "a.rs"}).to_string() };
        let mut result = ToolResult::ok("Edited a.rs (1 replacement).");
        hooks.after_tool("s1", &call, &mut result);
        assert_eq!(result.output, "Edited a.rs (1 replacement).\n[hook: the user's hook `grep -q '\"is_error\":false' && { echo '格式检查未通过：a.rs 第 3 行' >&2; exit 1; }; exit 0` reported: 格式检查未通过：a.rs 第 3 行]");
        let mut other = ToolResult::ok("ok");
        hooks.after_tool("s1", &ToolCall { id: "d".into(), name: "shell".into(), arguments: "{}".into() }, &mut other);
        assert_eq!(other.output, "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_hook_that_never_reads_large_stdin_still_times_out() {
        let (dir, hooks) = setup("blocked-stdin", json!({"hooks":[{
            "event":"before_tool","command":"sleep 30","timeout_seconds":1
        }]}));
        let began = Instant::now();
        let result = before(&hooks, "write_file", json!({"content":"x".repeat(256 * 1024)}));
        assert!(matches!(result, ToolGate::Block(reason) if reason.contains("timed out")));
        assert!(began.elapsed() < Duration::from_secs(3), "stdin delivery is inside the hook deadline");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn large_stderr_is_drained_while_delivering_input_without_cutting_the_pipe() {
        let (dir, hooks) = setup("two-pipes", json!({"hooks":[hook("before_tool", &["*"],
            "head -c 262144 /dev/zero >&2; cat > seen.json")]}));
        let result = before(&hooks, "write_file", json!({"content":"x".repeat(256 * 1024)}));
        if let ToolGate::Block(reason) = result { panic!("unexpected hook failure: {reason}"); }
        let seen: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("work/seen.json")).unwrap()).unwrap();
        assert_eq!(seen["arguments"]["content"].as_str().unwrap().len(), 256 * 1024);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn exiting_shell_cleans_up_descendants_and_does_not_wait_on_their_stderr() {
        let (dir, hooks) = setup("descendant", json!({"hooks":[hook("before_tool", &["*"],
            "(sleep 2; echo leaked > descendant-ran.txt) & exit 0")]}));
        let began = Instant::now();
        assert!(matches!(before(&hooks, "read_file", json!({})), ToolGate::Proceed));
        assert!(began.elapsed() < Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(2200));
        assert!(!dir.join("work/descendant-ran.txt").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn cancellation_reaches_before_and_after_hooks_through_the_real_agent() {
        use agent_kernel::{Agent, HarnessError, MemoryStore, Risk, Tool};
        use agent_kernel::provider::scripted::{ScriptedProvider, Step};
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        struct CountTool { spec: ToolSpec, count: Arc<AtomicUsize> }
        impl Tool for CountTool {
            fn spec(&self) -> &ToolSpec { &self.spec }
            fn execute(&self, _: Value, _: &ToolCtx) -> Result<String, String> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok("performed once".into())
            }
        }
        for event in ["before_tool", "after_tool"] {
            let (dir, hooks) = setup(&format!("cancel-{event}"), json!({"hooks":[
                {"event":event,"command":"touch started; sleep 30","timeout_seconds":30},
                {"event":event,"command":"touch second-hook","timeout_seconds":30}
            ]}));
            let count = Arc::new(AtomicUsize::new(0));
            let provider = Arc::new(ScriptedProvider::new(vec![Step::tool_call("c", "count", "{}"), Step::text("done")]));
            let agent = Agent::builder(provider, Arc::new(MemoryStore::new())).middleware(hooks)
                .tool(CountTool { spec: ToolSpec::new("count", "", json!({"type":"object"})).risk(Risk::Read), count:count.clone() }).build().unwrap();
            let cancel = Arc::new(AtomicBool::new(false));
            let flag = cancel.clone();
            let started = dir.join("work/started");
            let trigger = std::thread::spawn(move || {
                let end = Instant::now() + Duration::from_secs(2);
                while !started.exists() && Instant::now() < end { std::thread::sleep(Duration::from_millis(10)); }
                let observed = started.exists();
                flag.store(true, Ordering::SeqCst);
                observed
            });
            let began = Instant::now();
            assert_eq!(agent.run("s", "run", &mut |_| {}, &cancel), Err(HarnessError::Cancelled));
            assert!(trigger.join().unwrap(), "the test cancelled an active {event}");
            assert!(began.elapsed() < Duration::from_secs(3));
            assert_eq!(count.load(Ordering::SeqCst), usize::from(event == "after_tool"));
            assert!(!dir.join("work/second-hook").exists());
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

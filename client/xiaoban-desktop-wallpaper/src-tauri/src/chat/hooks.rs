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
//! - `before_tool` runs before the permission check. Exit code 0 lets the call go on; anything else (another code,
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
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT: u64 = 10;
const MAX_TIMEOUT: u64 = 120;
/// What a hook's error output may add to a block reason or a tool result.
const MAX_FEEDBACK_BYTES: usize = 4 * 1024;
/// How much of a tool's output is handed to an `after_tool` hook.
const MAX_OUTPUT_TO_HOOK: usize = 64 * 1024;

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
    /// Exit code (or none: timed out, killed), and its standard error.
    Failed(Option<i32>, String),
    CouldNotRun(String),
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

    fn run(&self, hook: &Hook, input: &Value) -> Outcome {
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
        cmd.current_dir(&self.workspace).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped());
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => return Outcome::CouldNotRun(format!("cannot start hook `{}`: {e}", hook.command)),
        };
        if let Some(mut stdin) = child.stdin.take() {
            // A hook that does not read its input closes the pipe early; that is its choice, not an error.
            let _ = stdin.write_all(input.to_string().as_bytes());
        }
        let stderr = child.stderr.take();
        let reader = std::thread::spawn(move || {
            let mut text = String::new();
            if let Some(mut stderr) = stderr {
                let _ = stderr.by_ref().take(MAX_FEEDBACK_BYTES as u64 * 4).read_to_string(&mut text);
            }
            text
        });
        let timeout = Duration::from_secs(hook.timeout_seconds.unwrap_or(DEFAULT_TIMEOUT).clamp(1, MAX_TIMEOUT));
        let end = Instant::now() + timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() < end => std::thread::sleep(Duration::from_millis(20)),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                Err(e) => return Outcome::CouldNotRun(format!("cannot wait for hook `{}`: {e}", hook.command)),
            }
        };
        let stderr = reader.join().unwrap_or_default();
        let stderr = clip(stderr.trim(), MAX_FEEDBACK_BYTES).to_string();
        match status {
            Some(status) if status.success() => Outcome::Passed,
            Some(status) => Outcome::Failed(status.code(), stderr),
            None => Outcome::Failed(None, format!("timed out after {}s. {stderr}", timeout.as_secs()).trim().to_string()),
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
            match self.run(hook, &input) {
                Outcome::Passed => {}
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
        let Ok(hooks) = self.load(Event::AfterTool, &call.name) else { return };
        if hooks.is_empty() {
            return;
        }
        let arguments: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::String(call.arguments.clone()));
        let input = json!({
            "event": "after_tool", "session": session, "tool": call.name, "arguments": arguments,
            "output": clip(&result.output, MAX_OUTPUT_TO_HOOK), "is_error": result.is_error,
        });
        for hook in &hooks {
            let feedback = match self.run(hook, &input) {
                Outcome::Passed => continue,
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
}

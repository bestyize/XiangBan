//! Windows document-command regressions. Python is optional for the kernel, so run explicitly:
//! cargo test --features shell,process --test windows_commands -- --include-ignored
#![cfg(all(windows, feature = "shell", feature = "process"))]

use agent_kernel::{builtin::Shell, process::{ProcessManager, ProcessScope, ProcessState}, Tool, ToolCtx};
use serde_json::json;
use std::{fs, path::PathBuf, time::{Duration, SystemTime, UNIX_EPOCH}};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("agent-doc-command-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let dir = root.join("我的 文档");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("订单.json"), r#"{"口令":"XB-DOC-4729","收入":400}"#).unwrap();
        fs::write(dir.join("核算.py"), r#"from pathlib import Path
import json
base = Path(__file__).resolve().parent
data = json.loads((base / '订单.json').read_text())
assert data['口令'] == 'XB-DOC-4729'
assert data['收入'] == 400
(base / 'summary.csv').write_text('净收入,400\n')
print('文档核验完成 😀 ' + data['口令'])
"#).unwrap();
        Self(root)
    }
    fn verify(&self, output: &str) {
        assert!(output.contains("文档核验完成 😀 XB-DOC-4729"), "{output}");
        let csv = fs::read_to_string(self.0.join("我的 文档/summary.csv")).unwrap();
        assert_eq!(csv.lines().collect::<Vec<_>>(), ["净收入,400"]);
        assert!(!self.0.join("我的 文档/我的 文档").exists());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}
const COMMAND: &str = "cd \"我的 文档\" && python \"核算.py\"";

#[test]
#[ignore = "requires Python on PATH"]
fn shell_round_trips_unicode_and_writes_beside_the_script() {
    let fixture = Fixture::new();
    let shell = Shell::new(&fixture.0);
    let ctx = ToolCtx::detached("docs");
    let result = shell.execute_result(json!({"command": COMMAND}), &ctx);
    assert!(!result.is_error, "{}", result.output);
    fixture.verify(&result.output);
    // A later call starts in the declared workspace, even after the first command used cd.
    let next = shell.execute_result(json!({"command": "if exist \"我的 文档\\summary.csv\" (echo FOUND) else (exit /b 1)"}), &ctx);
    assert!(!next.is_error && next.output.contains("FOUND"), "{}", next.output);
}

#[test]
#[ignore = "requires Python on PATH"]
fn managed_process_uses_the_same_utf8_command_environment() {
    let fixture = Fixture::new();
    let manager = ProcessManager::new(&fixture.0).unwrap();
    let ctx = ToolCtx::detached("docs");
    let started = manager.start(&ctx, COMMAND, ProcessScope::Run, Duration::from_secs(30)).unwrap();
    let result = manager.read(&ctx, &started.id, 0, 32000, Duration::from_secs(30)).unwrap();
    assert_eq!(result.state, ProcessState::Exited, "{result:?}");
    assert_eq!(result.exit_code, Some(0), "{result:?}");
    fixture.verify(&result.output);
}

#[test]
#[ignore = "requires Python on PATH"]
fn explicit_host_encoding_settings_override_the_defaults() {
    let fixture = Fixture::new();
    let result = Shell::new(&fixture.0).env("PYTHONUTF8", "0").env("PYTHONIOENCODING", "ascii")
        .execute_result(json!({"command": "python -c \"import sys; print(sys.flags.utf8_mode, sys.stdout.encoding)\""}), &ToolCtx::detached("docs"));
    assert!(!result.is_error, "{}", result.output);
    assert!(result.output.contains("0 ascii"), "{}", result.output);
}

/// cmd.exe runs only a command's first line and still reports success; the tool must refuse such a command instead.
#[test]
fn a_multi_line_command_is_refused_and_nothing_runs() {
    let root = std::env::temp_dir().join(format!("agent-multiline-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let marker = root.join("ran.txt");
    let shell = Shell::new(&root);
    let command = format!("echo first > first.txt\necho second > \"{}\"", marker.display());
    let error = shell.execute(json!({ "command": command }), &ToolCtx::detached("s")).unwrap_err();
    assert!(error.contains("Nothing was executed") && error.contains("write_file"), "{error}");
    assert!(!root.join("first.txt").exists() && !marker.exists(), "not even the first line ran");
    let ok = shell.execute(json!({ "command": "echo done" }), &ToolCtx::detached("s")).unwrap();
    assert!(ok.contains("done"), "{ok}");
    let _ = fs::remove_dir_all(&root);
}

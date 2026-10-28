#![cfg(all(feature = "shell", unix))]
mod common;

use agent_kernel::builtin::Shell;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn run_shell(shell: Shell, command: &str) -> (String, Vec<Event>) {
    let args = serde_json::json!({ "command": command }).to_string();
    let h = harness(vec![Step::tool_call("c", "shell", &args), Step::text("ok")], |b| b.tool(shell));
    let (result, events) = run(&h.agent, "go");
    assert!(result.is_ok());
    (tool_messages(&h).remove(0).1, events)
}

fn sh(dir: &TempDir, command: &str) -> String {
    run_shell(Shell::new(&dir.0), command).0
}

/// A killed process whose parent never reaps it stays a zombie (state `Z`); it is not running. That happens in
/// containers whose first process does not reap orphans.
fn process_alive(pid: &str) -> bool {
    let out = Command::new("ps").args(["-o", "stat=", "-p", pid]).stdin(Stdio::null()).stderr(Stdio::null()).output();
    out.is_ok_and(|out| {
        let state = String::from_utf8_lossy(&out.stdout);
        let state = state.trim();
        !state.is_empty() && !state.starts_with('Z')
    })
}

fn assert_dies(pid: &str) {
    let end = Instant::now() + Duration::from_secs(3);
    while process_alive(pid) && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!process_alive(pid), "process {pid} is still running");
}

fn pid_from(dir: &TempDir, file: &str) -> String {
    std::fs::read_to_string(dir.path(file)).unwrap().trim().to_string()
}

#[test]
fn captures_combined_output_and_exit_status() {
    let dir = TempDir::new("sh-basic");
    assert_eq!(sh(&dir, "echo hello"), "hello\n[exit code: 0]");
    assert_eq!(sh(&dir, "true"), "(no output)\n[exit code: 0]");
    let out = sh(&dir, "echo out; echo err >&2");
    assert!(out.contains("out") && out.contains("err") && out.ends_with("[exit code: 0]"), "{out}");
    let (out, events) = run_shell(Shell::new(&dir.0), "echo oops >&2; exit 3");
    assert_eq!(out, "oops\n[exit code: 3]");
    assert!(events.iter().any(|e| matches!(e, Event::ToolEnd { is_error: true, .. })), "a failing command is flagged as an error");
    assert!(sh(&dir, "kill -9 $$").contains("terminated by a signal"));
    assert!(sh(&dir, "nonexistent-command-xyz").contains("[exit code: 127]"));
}

#[test]
fn runs_in_the_configured_directory_with_stdin_closed() {
    let dir = TempDir::new("sh-cwd");
    dir.write("marker.txt", "x");
    let real = dir.0.canonicalize().unwrap();
    assert_eq!(sh(&dir, "pwd").lines().next().unwrap(), real.display().to_string());
    assert!(sh(&dir, "ls").contains("marker.txt"));
    assert_eq!(sh(&dir, "cat; echo finished"), "finished\n[exit code: 0]", "cat sees EOF instead of waiting");
    let missing = run_shell(Shell::new(dir.path("nope")), "pwd").0;
    assert!(missing.contains("cannot start the shell"), "{missing}");
}

#[test]
fn the_environment_is_cleared_except_for_what_the_host_allows() {
    let dir = TempDir::new("sh-env");
    std::env::set_var("AGENT_KERNEL_TEST_SECRET", "hunter2");
    std::env::set_var("AGENT_KERNEL_TEST_SHARED", "visible");
    let shell = Shell::new(&dir.0).env("PROJECT_MODE", "demo").inherit_env("AGENT_KERNEL_TEST_SHARED");
    let (out, _) = run_shell(shell, "env");
    assert!(!out.contains("hunter2") && !out.contains("AGENT_KERNEL_TEST_SECRET"), "{out}");
    assert!(out.contains("PROJECT_MODE=demo") && out.contains("AGENT_KERNEL_TEST_SHARED=visible"), "{out}");
    assert!(out.contains("PATH="), "PATH is passed through so commands can be found");
    std::env::remove_var("AGENT_KERNEL_TEST_SECRET");
    std::env::remove_var("AGENT_KERNEL_TEST_SHARED");
}

#[test]
fn a_timeout_kills_the_command_and_everything_it_started() {
    let dir = TempDir::new("sh-timeout");
    let pidfile = dir.path("bg.pid").display().to_string();
    let started = Instant::now();
    let (out, _) = run_shell(
        Shell::new(&dir.0).timeout(Duration::from_millis(400)),
        &format!("sleep 30 & echo $! > \"{pidfile}\"; echo before; sleep 30"),
    );
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
    assert!(out.contains("timed out"), "{out}");
    assert_dies(&pid_from(&dir, "bg.pid"));
}

#[test]
fn background_processes_left_by_a_finished_command_are_cleaned_up() {
    let dir = TempDir::new("sh-bg");
    let pidfile = dir.path("bg.pid").display().to_string();
    let started = Instant::now();
    let out = sh(&dir, &format!("sleep 30 & echo $! > \"{pidfile}\"; echo started"));
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
    assert_eq!(out, "started\n[exit code: 0]");
    assert_dies(&pid_from(&dir, "bg.pid"));
}

#[test]
fn cancelling_the_run_stops_the_command() {
    let dir = TempDir::new("sh-cancel");
    let pidfile = dir.path("fg.pid").display().to_string();
    let args = serde_json::json!({ "command": format!("echo $$ > \"{pidfile}\"; exec sleep 30") }).to_string();
    let h = harness(vec![Step::tool_call("c", "shell", &args), Step::text("never")], |b| b.tool(Shell::new(&dir.0)));
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = cancel.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        trigger.store(true, Ordering::SeqCst);
    });
    let started = Instant::now();
    let (result, _) = run_with_cancel(&h.agent, "go", &cancel);
    canceller.join().unwrap();
    assert_eq!(result.unwrap_err(), HarnessError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_dies(&pid_from(&dir, "fg.pid"));
}

#[test]
fn large_output_keeps_the_start_and_the_end() {
    let dir = TempDir::new("sh-big");
    let shell = Shell::new(&dir.0).max_output(2_000_000);
    let (out, _) = run_shell(shell, "echo START; head -c 1000000 /dev/zero | tr '\\0' 'a'; echo; echo END");
    assert!(out.starts_with("START\naaa"), "{}", &out[..20]);
    assert!(out.contains("bytes of output omitted ...]") && out.contains("\nEND\n[exit code: 0]"), "{}", &out[out.len() - 60..]);
    assert!(out.len() < 270 * 1024, "{}", out.len());
    let omitted: u64 = out.split("[... ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
    assert_eq!(omitted, 1_000_000 + "START\n\nEND\n".len() as u64 - 65_536 - 196_608);
}

#[test]
fn invalid_commands_never_reach_the_shell() {
    let dir = TempDir::new("sh-invalid");
    assert!(sh(&dir, "   ").contains("`command` must be"));
    assert!(sh(&dir, &"x".repeat(8001)).contains("`command` must be"));
}

#[test]
fn it_asks_before_running_and_is_blocked_in_read_only_mode() {
    let dir = TempDir::new("sh-perm");
    let marker = dir.path("ran").display().to_string();
    let args = serde_json::json!({ "command": format!("touch \"{marker}\"") }).to_string();
    for (mode, expect) in [(PermissionMode::Ask, "denied permission"), (PermissionMode::ReadOnly, "Read-only mode")] {
        let permission = Arc::new(Permission::new(mode, |r: &ApprovalRequest<'_>| {
            assert_eq!((r.tool, r.risk), ("shell", Risk::Exec));
            Approval::Deny
        }));
        let h = harness(vec![Step::tool_call("c", "shell", &args), Step::text("ok")], |b| b.tool(Shell::new(&dir.0)).middleware(permission.clone()));
        assert!(run(&h.agent, "go").0.is_ok());
        assert!(tool_messages(&h)[0].1.contains(expect));
        assert!(!dir.path("ran").exists());
    }
}

#[test]
fn a_working_folder_with_spaces_and_chinese_characters_is_used_as_is() {
    let base = TempDir::new("sh-unicode");
    let dir = base.path("我的 项目");
    std::fs::create_dir_all(&dir).unwrap();
    let real = dir.canonicalize().unwrap();
    let (out, _) = run_shell(Shell::new(&dir), "pwd; echo 内容 > \"新 文件.txt\"; ls");
    assert!(out.contains(&real.display().to_string()) && out.contains("新 文件.txt"), "{out}");
    assert_eq!(std::fs::read_to_string(dir.join("新 文件.txt")).unwrap(), "内容\n");
}

#[test]
fn shell_declares_the_one_shot_sandbox_escape() {
    let shell = Shell::new(std::env::temp_dir());
    assert_eq!(shell.spec().schema["properties"]["outside_sandbox"]["type"], "boolean");
    assert!(shell.spec().schema["required"].as_array().unwrap().iter().all(|k| k != "outside_sandbox"));
}

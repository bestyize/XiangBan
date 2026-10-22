//! Checks that need the real operating system: the Windows shell's quoting and environment, the macOS Seatbelt
//! sandbox and the Windows low-integrity sandbox. Elsewhere it has nothing to check and passes.
//!
//! A custom harness (`harness = false`), because on Windows this test program is also the sandbox launcher: the
//! sandboxed shell starts this same executable with the launcher flag.

#[allow(dead_code)]
mod common;

#[allow(unused_imports)]
use agent_kernel::builtin::Shell;
#[allow(unused_imports)]
use agent_kernel::provider::scripted::Step;
#[allow(unused_imports)]
use common::*;

#[allow(dead_code)]
fn shell(shell: Shell, command: &str) -> String {
    let args = serde_json::json!({ "command": command }).to_string();
    let h = harness(vec![Step::tool_call("c", "shell", &args), Step::text("ok")], |b| b.tool(shell));
    assert!(run(&h.agent, "go").0.is_ok());
    tool_messages(&h).remove(0).1
}

#[allow(dead_code)]
fn check(failures: &mut Vec<String>, ok: bool, what: &str, output: &str) {
    if ok {
        println!("ok   {what}");
    } else {
        println!("FAIL {what}\n{output}");
        failures.push(what.to_string());
    }
}

#[cfg(target_os = "macos")]
fn macos(failures: &mut Vec<String>) {
    use agent_kernel::sandbox::{SandboxHook, SandboxPolicy};
    let dir = TempDir::new("os-sandbox-mac");
    let home = std::path::PathBuf::from(std::env::var("HOME").unwrap());
    let probe = home.join(".ssh").join("xiaoban-sandbox-probe");
    std::fs::create_dir_all(probe.parent().unwrap()).unwrap();
    std::fs::write(&probe, "secret").unwrap();
    let policy = SandboxPolicy::for_workspace(&dir.0);
    let sandboxed = || Shell::new(&dir.0).sandbox(SandboxHook::new({ let p = policy.clone(); move |_, _| Some(p.clone()) }));

    let out = shell(sandboxed(), "echo ok > inside.txt && cat inside.txt");
    check(failures, dir.path("inside.txt").exists() && out.contains("exit code: 0"), "macOS sandbox: writing in the working folder works", &out);
    let outside = home.join("xiaoban-sandbox-probe.txt");
    let out = shell(sandboxed(), &format!("echo x > '{}'", outside.display()));
    check(failures, !outside.exists() && out.contains("[sandbox:"), "macOS sandbox: writing in the home folder is refused", &out);
    let out = shell(sandboxed(), &format!("cat '{}'", probe.display()));
    check(failures, !out.contains("secret"), "macOS sandbox: ~/.ssh cannot be read", &out);
    let out = shell(sandboxed(), "mkdir -p node_modules/x && echo '{}' > node_modules/x/package.json && ls node_modules/x");
    check(failures, out.contains("package.json"), "macOS sandbox: ordinary work in the folder is unaffected", &out);
    let _ = std::fs::remove_file(&probe);
    let _ = std::fs::remove_file(&outside);
}

#[cfg(windows)]
fn windows(failures: &mut Vec<String>) {
    use agent_kernel::sandbox::{label_low, SandboxHook, SandboxPolicy};
    let dir = TempDir::new("os-sandbox-win");

    let out = shell(Shell::new(&dir.0), r#"echo "a b""#);
    check(failures, out.contains(r#""a b""#) && !out.contains(r#"\""#), "Windows shell: quotes reach cmd unchanged", &out);
    let out = shell(Shell::new(&dir.0), "echo %APPDATA%");
    check(failures, !out.contains("%APPDATA%"), "Windows shell: APPDATA is passed to commands", &out);

    let work = dir.path("work");
    let low = dir.path("low");
    if let Err(error) = label_low(&work).and_then(|_| label_low(&low)) {
        check(failures, false, "Windows sandbox: label the folders low integrity", &error);
        return;
    }
    let policy = SandboxPolicy { launcher: std::env::current_exe().ok(), low_folder: Some(low.clone()), ..SandboxPolicy::for_workspace(&work) };
    let sandboxed = || Shell::new(&work).sandbox(SandboxHook::new({ let p = policy.clone(); move |_, _| Some(p.clone()) }));

    let out = shell(sandboxed(), "echo ok> inside.txt && type inside.txt");
    check(failures, work.join("inside.txt").exists() && out.contains("exit code: 0"), "Windows sandbox: writing in the labelled folder works", &out);
    let outside = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap()).join("xiaoban-sandbox-probe.txt");
    let _ = std::fs::remove_file(&outside);
    let out = shell(sandboxed(), &format!("echo x> \"{}\"", outside.display()));
    check(failures, !outside.exists(), "Windows sandbox: writing in the user folder is refused", &out);
    let out = shell(sandboxed(), "echo %TEMP%");
    check(failures, out.contains(&low.display().to_string()), "Windows sandbox: temporary files go to the low folder", &out);
    let _ = std::fs::remove_file(&outside);
}

fn main() {
    if let Some(code) = agent_kernel::sandbox::windows_launcher_main() {
        std::process::exit(code);
    }
    #[allow(unused_mut)]
    let mut failures: Vec<String> = Vec::new();
    #[cfg(target_os = "macos")]
    macos(&mut failures);
    #[cfg(windows)]
    windows(&mut failures);
    if failures.is_empty() {
        println!("os_sandbox: all checks passed (or none apply on this system)");
    } else {
        println!("os_sandbox: {} failed: {failures:?}", failures.len());
        std::process::exit(1);
    }
}

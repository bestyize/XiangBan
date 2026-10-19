#![cfg(feature = "fs")]
mod common;

use agent_kernel::builtin::fs::fs_tools;
use agent_kernel::provider::scripted::Step;
use agent_kernel::HarnessError;
use common::*;
use std::path::PathBuf;

struct Workspace {
    root: PathBuf,
    outside: PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let base = std::env::temp_dir().join(format!("agent-kernel-fs-{tag}-{}-{:?}", std::process::id(), std::thread::current().id()));
        let _ = std::fs::remove_dir_all(&base);
        let (root, outside) = (base.join("root"), base.join("outside"));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("target")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();
        std::fs::write(root.join("notes.txt"), "alpha\nBeta\r\ngamma\n").unwrap();
        std::fs::write(root.join("target/skip.txt"), "Hello from target\n").unwrap();
        std::fs::write(root.join("blob.bin"), b"Hello\0binary").unwrap();
        std::fs::write(outside.join("secret.txt"), "top secret\n").unwrap();
        Self { root, outside }
    }

    fn call(&self, tool: &str, args: &str) -> String {
        let tools = fs_tools(vec![self.root.clone()]).unwrap();
        let h = harness(vec![Step::tool_call("c", tool, args), Step::text("ok")], |mut b| {
            for t in tools {
                b = b.tool_arc(t);
            }
            b
        });
        assert!(run(&h.agent, "go").0.is_ok());
        tool_messages(&h).remove(0).1
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.root.parent().unwrap());
    }
}

#[test]
fn read_file_numbers_lines_and_pages() {
    let ws = Workspace::new("read");
    assert_eq!(ws.call("read_file", r#"{"path":"src/main.rs"}"#), "     1\tfn main() {\n     2\t    println!(\"Hello\");\n     3\t}\n");
    let page = ws.call("read_file", r#"{"path":"notes.txt","offset":2,"limit":1}"#);
    assert_eq!(page, "     2\tBeta\n[more lines follow; continue with offset=3]");
    assert!(ws.call("read_file", r#"{"path":"notes.txt","offset":50}"#).contains("the file has 3"));
    let abs = ws.root.join("notes.txt").display().to_string().replace('\\', "\\\\");
    assert!(ws.call("read_file", &format!(r#"{{"path":"{abs}"}}"#)).contains("alpha"));
}

#[test]
fn read_file_refuses_binary_directories_and_missing_files() {
    let ws = Workspace::new("readbad");
    assert!(ws.call("read_file", r#"{"path":"blob.bin"}"#).contains("binary"));
    assert!(ws.call("read_file", r#"{"path":"src"}"#).contains("list_dir"));
    assert!(ws.call("read_file", r#"{"path":"nope.txt"}"#).contains("cannot access"));
    assert!(ws.call("read_file", r#"{"offset":1}"#).contains("\"path\" is a required property"));
}

#[test]
fn read_file_caps_long_lines_and_large_output() {
    let ws = Workspace::new("readbig");
    std::fs::write(ws.root.join("long.txt"), format!("{}\n", "字".repeat(5000))).unwrap();
    let out = ws.call("read_file", r#"{"path":"long.txt"}"#);
    assert!(out.ends_with("…\n") && out.chars().count() < 2100);
    let many: String = (0..5000).map(|i| format!("line number {i} with some padding text\n")).collect();
    std::fs::write(ws.root.join("many.txt"), many).unwrap();
    let out = ws.call("read_file", r#"{"path":"many.txt","limit":5000}"#);
    assert!(out.len() < 45_000 && out.contains("continue with offset="));
}

#[test]
fn paths_outside_the_roots_are_refused() {
    let ws = Workspace::new("confine");
    let secret = ws.outside.join("secret.txt").display().to_string().replace('\\', "\\\\");
    for args in [format!(r#"{{"path":"{secret}"}}"#), r#"{"path":"../outside/secret.txt"}"#.to_string()] {
        let out = ws.call("read_file", &args);
        assert!(out.contains("outside the allowed directories"), "{out}");
        assert!(!out.contains("top secret"));
    }
    assert!(ws.call("list_dir", r#"{"path":"../outside"}"#).contains("outside the allowed"));
    assert!(ws.call("grep", r#"{"pattern":"secret","path":"../outside"}"#).contains("outside the allowed"));
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_be_used_to_escape() {
    let ws = Workspace::new("symlink");
    std::os::unix::fs::symlink(ws.outside.join("secret.txt"), ws.root.join("link.txt")).unwrap();
    std::os::unix::fs::symlink(&ws.outside, ws.root.join("linkdir")).unwrap();
    assert!(ws.call("read_file", r#"{"path":"link.txt"}"#).contains("outside the allowed"));
    assert!(ws.call("list_dir", r#"{"path":"linkdir"}"#).contains("outside the allowed"));
    assert!(ws.call("grep", r#"{"pattern":"secret"}"#).contains("No matches"), "grep does not follow symlinks");
}

#[test]
fn list_dir_sorts_and_marks_directories() {
    let ws = Workspace::new("list");
    let out = ws.call("list_dir", "{}");
    assert_eq!(out.lines().collect::<Vec<_>>(), vec!["blob.bin  (12 bytes)", "notes.txt  (18 bytes)", "src/", "target/"]);
    assert_eq!(ws.call("list_dir", r#"{"path":"src"}"#), "main.rs  (37 bytes)");
    std::fs::create_dir_all(ws.root.join("empty")).unwrap();
    assert_eq!(ws.call("list_dir", r#"{"path":"empty"}"#), "(empty directory)");
}

#[test]
fn grep_finds_literals_skips_noise_and_limits_results() {
    let ws = Workspace::new("grep");
    let out = ws.call("grep", r#"{"pattern":"Hello"}"#);
    assert!(out.contains("src/main.rs:2: println!(\"Hello\");") || out.contains("src\\main.rs:2: println!(\"Hello\");"), "{out}");
    assert!(!out.contains("skip.txt") && !out.contains("blob.bin"), "target dir and binary files are skipped: {out}");
    assert!(ws.call("grep", r#"{"pattern":"beta"}"#).contains("No matches"));
    assert!(ws.call("grep", r#"{"pattern":"beta","ignore_case":true}"#).contains("notes.txt:2: Beta"));
    assert!(ws.call("grep", r#"{"pattern":"a.c"}"#).contains("No matches"), "pattern is literal, not a regex");
    assert!(ws.call("grep", r#"{"pattern":"gamma","path":"notes.txt"}"#).contains("notes.txt:3: gamma"));
    assert!(ws.call("grep", r#"{"pattern":""}"#).contains("1 to 500"));

    let lines: String = (0..50).map(|i| format!("hit {i}\n")).collect();
    std::fs::write(ws.root.join("hits.txt"), lines).unwrap();
    let out = ws.call("grep", r#"{"pattern":"hit","max_results":5}"#);
    assert_eq!(out.lines().filter(|l| l.contains("hits.txt:")).count(), 5);
    assert!(out.contains("results limited"));
}

#[test]
fn roots_must_exist_and_be_directories() {
    assert!(matches!(fs_tools(vec![]), Err(HarnessError::Config(_))));
    assert!(matches!(fs_tools(vec![PathBuf::from("/definitely/not/here")]), Err(HarnessError::Config(_))));
    let file = std::env::temp_dir().join(format!("agent-kernel-rootfile-{}", std::process::id()));
    std::fs::write(&file, "x").unwrap();
    assert!(matches!(fs_tools(vec![file.clone()]), Err(HarnessError::Config(m)) if m.contains("not a directory")));
    let _ = std::fs::remove_file(file);
}

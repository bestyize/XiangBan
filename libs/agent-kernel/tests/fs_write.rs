#![cfg(feature = "fs")]
mod common;

use agent_kernel::builtin::fs::fs_write_tools;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;

struct Ws {
    base: TempDir,
}

impl Ws {
    fn new(tag: &str) -> Self {
        let base = TempDir::new(tag);
        std::fs::create_dir_all(base.path("root")).unwrap();
        std::fs::create_dir_all(base.path("outside")).unwrap();
        Self { base }
    }

    fn root(&self, rel: &str) -> std::path::PathBuf {
        self.base.path("root").join(rel)
    }

    fn call(&self, tool: &str, args: serde_json::Value) -> String {
        let tools = fs_write_tools(vec![self.base.path("root")]).unwrap();
        let h = harness(vec![Step::tool_call("c", tool, &args.to_string()), Step::text("ok")], |mut b| {
            for t in tools {
                b = b.tool_arc(t);
            }
            b
        });
        assert!(run(&h.agent, "go").0.is_ok());
        tool_messages(&h).remove(0).1
    }

    fn write(&self, path: &str, content: &str) -> String {
        self.call("write_file", serde_json::json!({"path": path, "content": content}))
    }

    fn edit(&self, path: &str, old: &str, new: &str, all: bool) -> String {
        self.call("edit_file", serde_json::json!({"path": path, "old_string": old, "new_string": new, "replace_all": all}))
    }

    fn leftovers(&self) -> Vec<String> {
        fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.ends_with(".tmp") {
                    out.push(name);
                }
                if e.file_type().unwrap().is_dir() {
                    walk(&e.path(), out);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.base.0, &mut out);
        out
    }
}

#[test]
fn write_file_creates_nested_files_and_replaces_existing_ones() {
    let ws = Ws::new("fsw-basic");
    assert_eq!(ws.write("a/b/new.txt", "hello"), "Created a/b/new.txt (5 bytes).");
    assert_eq!(std::fs::read_to_string(ws.root("a/b/new.txt")).unwrap(), "hello");
    assert_eq!(ws.write("a/b/new.txt", "替换"), "Replaced a/b/new.txt (6 bytes).");
    assert_eq!(std::fs::read_to_string(ws.root("a/b/new.txt")).unwrap(), "替换");
    let absolute = ws.root("abs.txt").display().to_string();
    assert!(ws.write(&absolute, "x").starts_with("Created abs.txt"));
    assert!(ws.leftovers().is_empty(), "no temp files remain: {:?}", ws.leftovers());
}

#[cfg(unix)]
#[test]
fn replacing_a_file_keeps_its_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let ws = Ws::new("fsw-mode");
    ws.write("script.sh", "#!/bin/sh\n");
    std::fs::set_permissions(ws.root("script.sh"), std::fs::Permissions::from_mode(0o750)).unwrap();
    ws.write("script.sh", "#!/bin/sh\necho hi\n");
    assert_eq!(std::fs::metadata(ws.root("script.sh")).unwrap().permissions().mode() & 0o777, 0o750);
}

#[test]
fn write_file_refuses_escapes_directories_and_oversized_content() {
    let ws = Ws::new("fsw-escape");
    let outside = ws.base.path("outside/target.txt").display().to_string();
    assert!(ws.write(&outside, "x").contains("outside the allowed directories"));
    assert!(ws.write("../outside/target.txt", "x").contains("`..` is not allowed"));
    assert!(ws.write("sub/../../outside/target.txt", "x").contains("`..` is not allowed"));
    assert!(!ws.base.path("outside/target.txt").exists());
    std::fs::create_dir_all(ws.root("adir")).unwrap();
    assert!(ws.write("adir", "x").contains("not a regular file"));
    assert!(ws.write("big.txt", &"x".repeat(1024 * 1024 + 1)).contains("larger than"));
    assert!(!ws.root("big.txt").exists());
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_be_written_through() {
    let ws = Ws::new("fsw-symlink");
    ws.base.write("outside/real.txt", "original");
    std::os::unix::fs::symlink(ws.base.path("outside/real.txt"), ws.root("link.txt")).unwrap();
    std::os::unix::fs::symlink(ws.base.path("outside/not-yet.txt"), ws.root("dangling.txt")).unwrap();
    std::os::unix::fs::symlink(ws.base.path("outside"), ws.root("linkdir")).unwrap();
    assert!(ws.write("link.txt", "pwned").contains("outside the allowed"));
    assert!(ws.write("dangling.txt", "pwned").contains("cannot access"));
    assert!(ws.write("linkdir/new.txt", "pwned").contains("outside the allowed"));
    assert!(ws.edit("link.txt", "original", "pwned", false).contains("outside the allowed"));
    assert_eq!(std::fs::read_to_string(ws.base.path("outside/real.txt")).unwrap(), "original");
    assert!(!ws.base.path("outside/not-yet.txt").exists() && !ws.base.path("outside/new.txt").exists());
}

#[cfg(unix)]
#[test]
fn a_symlink_that_stays_inside_the_root_updates_its_target() {
    let ws = Ws::new("fsw-inside");
    ws.write("real.txt", "one");
    std::os::unix::fs::symlink(ws.root("real.txt"), ws.root("alias.txt")).unwrap();
    assert!(ws.write("alias.txt", "two").starts_with("Replaced real.txt"));
    assert_eq!(std::fs::read_to_string(ws.root("real.txt")).unwrap(), "two");
    assert!(std::fs::symlink_metadata(ws.root("alias.txt")).unwrap().file_type().is_symlink(), "the link itself survives");
}

#[test]
fn edit_file_replaces_exact_text_once_or_everywhere() {
    let ws = Ws::new("fsw-edit");
    ws.write("f.txt", "alpha\r\nbeta\r\nalpha\r\n你好\r\n");
    assert!(ws.edit("f.txt", "beta", "BETA", false).starts_with("Edited f.txt (1 replacement)"));
    assert!(ws.edit("f.txt", "alpha", "A", false).contains("occurs 2 times"));
    assert!(ws.edit("f.txt", "alpha", "A", true).contains("2 replacements"));
    assert!(ws.edit("f.txt", "你好", "世界", false).starts_with("Edited"));
    assert_eq!(std::fs::read_to_string(ws.root("f.txt")).unwrap(), "A\r\nBETA\r\nA\r\n世界\r\n", "CRLF line endings are untouched");
    assert!(ws.edit("f.txt", "missing", "x", false).contains("was not found"));
    assert!(ws.edit("f.txt", "A", "A", false).contains("identical"));
    assert!(ws.edit("f.txt", "", "x", false).contains("must not be empty"));
    assert!(ws.edit("nope.txt", "a", "b", false).contains("cannot access"));
    assert!(ws.leftovers().is_empty());
}

#[test]
fn edit_file_refuses_binary_oversized_and_outside_files() {
    let ws = Ws::new("fsw-edit-bad");
    std::fs::write(ws.root("blob.bin"), b"abc\0def").unwrap();
    std::fs::write(ws.root("latin1.txt"), b"caf\xe9").unwrap();
    std::fs::write(ws.root("huge.txt"), vec![b'a'; 2 * 1024 * 1024 + 1]).unwrap();
    std::fs::create_dir_all(ws.root("adir")).unwrap();
    ws.base.write("outside/o.txt", "abc");
    assert!(ws.edit("blob.bin", "abc", "x", false).contains("binary"));
    assert!(ws.edit("latin1.txt", "caf", "x", false).contains("not valid UTF-8"));
    assert!(ws.edit("huge.txt", "a", "b", false).contains("larger than"));
    assert!(ws.edit("adir", "a", "b", false).contains("not a regular file"));
    assert!(ws.edit(&ws.base.path("outside/o.txt").display().to_string(), "abc", "x", false).contains("outside the allowed"));
    assert_eq!(std::fs::read(ws.root("blob.bin")).unwrap(), b"abc\0def");
}

#[test]
fn write_tools_are_blocked_in_read_only_mode() {
    let ws = Ws::new("fsw-perm");
    let tools = fs_write_tools(vec![ws.base.path("root")]).unwrap();
    let permission = std::sync::Arc::new(Permission::new(PermissionMode::ReadOnly, |_: &ApprovalRequest<'_>| Approval::Once));
    let steps = vec![Step::tool_call("c", "write_file", r#"{"path":"x.txt","content":"c"}"#), Step::text("ok")];
    let h = harness(steps, |mut b| {
        for t in tools {
            b = b.tool_arc(t);
        }
        b.middleware(permission.clone())
    });
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(tool_messages(&h)[0].1.contains("Read-only mode"));
    assert!(!ws.root("x.txt").exists());
}

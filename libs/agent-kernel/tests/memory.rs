#![cfg(feature = "memory")]
mod common;

use agent_kernel::memory::Memory;
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::sync::Arc;

fn call(memory: &Arc<Memory>, tool: &str, args: &str) -> String {
    let h = harness(vec![Step::tool_call("c", tool, args), Step::text("ok")], |mut b| {
        for t in memory.tools() {
            b = b.tool_arc(t);
        }
        b
    });
    assert!(run(&h.agent, "go").0.is_ok());
    tool_messages(&h).remove(0).1
}

fn setup() -> (TempDir, Arc<Memory>) {
    let dir = TempDir::new("memory");
    let notes = dir.path("notes");
    (dir, Arc::new(Memory::new(notes)))
}

#[test]
fn write_read_append_delete_round_trip() {
    let (dir, memory) = setup();
    assert_eq!(call(&memory, "memory_read", "{}"), "No notes saved yet.");
    assert!(!dir.path("notes").exists(), "nothing is created until the first write");
    assert_eq!(call(&memory, "memory_write", r#"{"name":"tastes","content":"Likes tea.\n"}"#), "Saved note `tastes` (11 bytes).");
    assert_eq!(call(&memory, "memory_read", r#"{"name":"tastes"}"#), "Likes tea.\n");
    call(&memory, "memory_write", r#"{"name":"tastes","action":"append","content":"Dislikes coffee."}"#);
    assert_eq!(call(&memory, "memory_read", r#"{"name":"tastes"}"#), "Likes tea.\nDislikes coffee.\n");
    call(&memory, "memory_write", r#"{"name":"fresh","action":"append","content":"made by append"}"#);
    call(&memory, "memory_write", r#"{"name":"tastes","content":"Replaced."}"#);
    assert_eq!(call(&memory, "memory_read", r#"{"name":"tastes"}"#), "Replaced.\n");
    let listing = call(&memory, "memory_read", "{}");
    assert_eq!(listing.lines().collect::<Vec<_>>(), vec!["fresh (15 bytes): made by append", "tastes (10 bytes): Replaced."]);
    assert_eq!(call(&memory, "memory_write", r#"{"name":"tastes","action":"delete"}"#), "Deleted note `tastes`.");
    assert!(call(&memory, "memory_read", r#"{"name":"tastes"}"#).contains("no note named"));
    assert!(call(&memory, "memory_write", r#"{"name":"tastes","action":"delete"}"#).contains("no note named"));
}

#[test]
fn names_actions_and_sizes_are_validated() {
    let (dir, memory) = setup();
    for bad in ["../x", "A", "a b", "", "-x", "a.md"] {
        let args = serde_json::json!({"name": bad, "content": "c"}).to_string();
        assert!(call(&memory, "memory_write", &args).contains("note names use"), "name {bad:?}");
        assert!(call(&memory, "memory_read", &serde_json::json!({"name": bad}).to_string()).contains("note names use") || bad.is_empty());
    }
    assert!(call(&memory, "memory_write", r#"{"name":"ok","action":"burn","content":"c"}"#).contains("unknown action"));
    assert!(call(&memory, "memory_write", r#"{"name":"ok","content":"   "}"#).contains("1 to 20000"));
    let big = serde_json::json!({"name": "ok", "content": "x".repeat(20_001)}).to_string();
    assert!(call(&memory, "memory_write", &big).contains("1 to 20000"));
    let append = serde_json::json!({"name": "ok", "action": "append", "content": "x".repeat(15_000)});
    assert!(call(&memory, "memory_write", &append.to_string()).starts_with("Saved"));
    assert!(call(&memory, "memory_write", &append.to_string()).contains("1 to 20000"), "append cannot grow a note past the limit");
    assert!(!dir.path("ok.md").exists() && !dir.path("../x.md").exists());
}

#[test]
fn the_note_count_is_limited() {
    let (_dir, memory) = setup();
    for i in 0..100 {
        let args = serde_json::json!({"name": format!("n{i:03}"), "content": "x"}).to_string();
        assert!(call(&memory, "memory_write", &args).starts_with("Saved"), "note {i}");
    }
    assert!(call(&memory, "memory_write", r#"{"name":"one-too-many","content":"x"}"#).contains("limit of 100"));
    assert!(call(&memory, "memory_write", r#"{"name":"n000","content":"still editable"}"#).starts_with("Saved"));
}

#[test]
fn prompt_section_previews_notes_and_respects_the_budget() {
    let (_dir, memory) = setup();
    let section = memory.prompt_section();
    assert_eq!(section(), "");
    call(&memory, "memory_write", r#"{"name":"pets","content":"\n\nHas a cat named Mimi.\nSecond line"}"#);
    let text = section();
    assert!(text.starts_with("## Saved notes") && text.contains("- pets: Has a cat named Mimi.") && !text.contains("Second line"));
    for i in 0..60 {
        let args = serde_json::json!({"name": format!("bulk-{i:02}"), "content": "y".repeat(100)}).to_string();
        call(&memory, "memory_write", &args);
    }
    let text = section();
    assert!(text.len() < 4300 && text.contains("more notes not listed"), "{}", text.len());
}

#[cfg(unix)]
#[test]
fn symlinked_notes_are_refused() {
    let (dir, memory) = setup();
    let outside = TempDir::new("memory-outside");
    outside.write("secret.txt", "do not touch");
    std::fs::create_dir_all(dir.path("notes")).unwrap();
    std::os::unix::fs::symlink(outside.path("secret.txt"), dir.path("notes/leak.md")).unwrap();
    assert!(call(&memory, "memory_read", r#"{"name":"leak"}"#).contains("not a regular note file"));
    assert!(call(&memory, "memory_write", r#"{"name":"leak","content":"overwrite"}"#).contains("not a regular note file"));
    assert!(call(&memory, "memory_write", r#"{"name":"leak","action":"delete"}"#).contains("not a regular note file"));
    assert_eq!(std::fs::read_to_string(outside.path("secret.txt")).unwrap(), "do not touch");
    assert!(memory.list().is_empty(), "symlinks are not listed");
}

#[test]
fn writes_follow_the_permission_rules() {
    let (dir, memory) = setup();
    let permission = Arc::new(Permission::new(PermissionMode::Ask, |_: &ApprovalRequest<'_>| Approval::Deny));
    let steps = vec![Step::tool_call("a", "memory_write", r#"{"name":"x","content":"c"}"#).and_tool_call("b", "memory_read", "{}"), Step::text("ok")];
    let h = harness(steps, |mut b| {
        for t in memory.tools() {
            b = b.tool_arc(t);
        }
        b.middleware(permission.clone())
    });
    assert!(run(&h.agent, "go").0.is_ok());
    let messages = tool_messages(&h);
    assert!(messages[0].1.contains("denied") && messages[1].1 == "No notes saved yet.");
    assert!(!dir.path("notes/x.md").exists());
}

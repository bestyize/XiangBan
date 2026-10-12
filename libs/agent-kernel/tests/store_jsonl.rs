use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("agent-kernel-{tag}-{}-{:?}", std::process::id(), std::thread::current().id()));
        let _ = std::fs::remove_dir_all(&dir);
        Self(dir)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sample() -> Vec<Message> {
    vec![
        Message::user(vec![Part::Text { text: "你好\n\"世界\"".into() }, Part::Image { url: "data:image/png;base64,AA".into() }]),
        Message::assistant("", vec![ToolCall { id: "c1".into(), name: "t".into(), arguments: r#"{"a":"line1\nline2"}"#.into() }]),
        Message::tool("c1", "output with\nnewlines"),
    ]
}

#[test]
fn round_trips_messages_across_instances_and_appends() {
    let dir = TempDir::new("rt");
    let messages = sample();
    JsonlStore::new(&dir.0).append("s1", &messages[..1]).unwrap();
    JsonlStore::new(&dir.0).append("s1", &messages[1..]).unwrap();
    assert_eq!(JsonlStore::new(&dir.0).load("s1").unwrap(), messages);
    assert!(JsonlStore::new(&dir.0).load("never-used").unwrap().is_empty());
}

#[test]
fn session_ids_cannot_escape_or_collide() {
    let dir = TempDir::new("ids");
    let store = JsonlStore::new(&dir.0);
    let ids = ["../escape", "a/b", "a%2Fb", "A", "a", "con", "会话"];
    for (i, id) in ids.iter().enumerate() {
        store.append(id, &[Message::user_text(format!("m{i}"))]).unwrap();
    }
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(store.load(id).unwrap()[0].text(), format!("m{i}"), "id {id:?}");
    }
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), ids.len());
    assert!(!dir.0.parent().unwrap().join("escape.jsonl").exists());
}

#[test]
fn half_written_tail_is_ignored_on_load_and_removed_on_append() {
    let dir = TempDir::new("partial");
    let store = JsonlStore::new(&dir.0);
    store.append("s", &sample()[..2]).unwrap();
    let path = dir.file("s-s.jsonl");
    OpenOptions::new().append(true).open(&path).unwrap().write_all(br#"{"v":1,"m":{"role":"tool","parts":[{"ty"#).unwrap();
    assert_eq!(store.load("s").unwrap().len(), 2, "partial record skipped");
    store.append("s", &sample()[2..]).unwrap();
    assert_eq!(store.load("s").unwrap(), sample(), "new record starts on a clean line");
    assert!(std::fs::read_to_string(&path).unwrap().ends_with('\n'));
}

#[test]
fn corrupt_middle_line_and_newer_format_are_errors() {
    let dir = TempDir::new("corrupt");
    let store = JsonlStore::new(&dir.0);
    store.append("s", &sample()[..1]).unwrap();
    let path = dir.file("s-s.jsonl");
    OpenOptions::new().append(true).open(&path).unwrap().write_all(b"not json\n").unwrap();
    store.append("s", &sample()[1..2]).unwrap();
    match store.load("s") {
        Err(HarnessError::Store(m)) => assert!(m.contains("line 2"), "{m}"),
        other => panic!("expected store error, got {other:?}"),
    }
    std::fs::write(dir.file("s-t.jsonl"), "{\"v\":99,\"m\":{\"role\":\"user\"}}\n").unwrap();
    assert!(matches!(store.load("t"), Err(HarnessError::Store(m)) if m.contains("newer version")));
}

#[cfg(unix)]
#[test]
fn session_files_are_private_to_the_owner() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new("perm");
    JsonlStore::new(&dir.0).append("s", &sample()[..1]).unwrap();
    let mode = std::fs::metadata(dir.file("s-s.jsonl")).unwrap().permissions().mode();
    assert_eq!(mode & 0o077, 0);
}

#[test]
fn agent_resumes_a_persisted_session_and_closes_dangling_tool_calls() {
    let dir = TempDir::new("resume");
    let store = Arc::new(JsonlStore::new(&dir.0));
    // A previous process died after the model asked for two tools but before either result was saved.
    let crashed = vec![
        Message::user_text("first"),
        Message::assistant("", vec![
            ToolCall { id: "a".into(), name: "t".into(), arguments: "{}".into() },
            ToolCall { id: "b".into(), name: "t".into(), arguments: "{}".into() },
        ]),
        Message::tool("a", "partial result"),
    ];
    store.append("s1", &crashed).unwrap();

    let provider = Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(vec![Step::text("welcome back")]));
    let agent = Agent::builder(provider.clone(), store.clone()).build().unwrap();
    let summary = agent.run("s1", "second", &mut |_| {}, &Arc::new(Default::default())).unwrap();
    assert_eq!(summary.text, "welcome back");

    let sent = &provider.requests()[0].messages;
    let roles: Vec<Role> = sent.iter().map(|m| m.role).collect();
    assert_eq!(roles, vec![Role::User, Role::Assistant, Role::Tool, Role::Tool, Role::User]);
    assert_eq!(sent[2].tool_call_id.as_deref(), Some("a"));
    assert_eq!(sent[3].tool_call_id.as_deref(), Some("b"));
    assert!(sent[3].text().contains("interrupted"));
    assert_eq!(store.load("s1").unwrap().len(), 6, "the repair is persisted too");
}

#[test]
fn removing_a_session_deletes_only_that_session_and_tolerates_missing_ones() {
    let dir = TempDir::new("remove");
    let store = JsonlStore::new(&dir.0);
    store.append("keep", &sample()[..1]).unwrap();
    store.append("drop", &sample()[..2]).unwrap();
    store.remove("drop").unwrap();
    assert!(store.load("drop").unwrap().is_empty());
    assert_eq!(store.load("keep").unwrap().len(), 1);
    store.remove("drop").unwrap();
    store.remove("never-existed").unwrap();
    store.append("drop", &sample()[..1]).unwrap();
    assert_eq!(store.load("drop").unwrap().len(), 1, "the id can be used again");
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 2);
}

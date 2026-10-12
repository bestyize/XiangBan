//! The assembled assistant, run with a scripted model: the real tool set, the real permission
//! rules, a real folder. Only the model and the desktop app are stand-ins.

use super::desktop_tools::{CompanionChange, Desktop};
use super::*;
use crate::reminders::Rule;
use agent_kernel::provider::scripted::{ScriptedProvider, Step};
use agent_kernel::{Event, Role};
use std::sync::atomic::AtomicUsize;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("xiaoban-chat-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, rel: &str) -> PathBuf {
        self.0.join(rel)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Default)]
struct FakeDesktop {
    calls: Mutex<Vec<String>>,
}

impl Desktop for FakeDesktop {
    fn status(&self) -> Result<Value, String> { Ok(json!({"mode": "floating"})) }
    fn wallpapers(&self) -> Result<Value, String> { Ok(json!({"activeId": "companion", "items": []})) }
    fn apply_wallpaper(&self, id: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("apply {id}"));
        Ok(())
    }
    fn effects(&self) -> Result<Value, String> { Ok(json!({"selected": "", "packs": []})) }
    fn set_effect(&self, id: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("effect {id}"));
        Ok(())
    }
    fn set_companion(&self, change: &CompanionChange) -> Result<(), String> {
        lock(&self.calls).push(format!("companion {change:?}"));
        Ok(())
    }
    fn care(&self) -> Result<Value, String> { Ok(json!({"config": {"enabled": true}, "rules": []})) }
    fn care_upsert(&self, rule: Rule) -> Result<(), String> {
        lock(&self.calls).push(format!("care {}", rule.text));
        Ok(())
    }
    fn care_delete(&self, id: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("care-delete {id}"));
        Ok(())
    }
    fn widgets(&self) -> Result<Value, String> { Ok(json!({"widgets": []})) }
    fn widget_add(&self, kind: &str) -> Result<Value, String> {
        lock(&self.calls).push(format!("widget-add {kind}"));
        Ok(json!({"widgets": [{"id": "wgt-1"}]}))
    }
    fn widget_update(&self, id: &str, patch: Value) -> Result<Value, String> {
        lock(&self.calls).push(format!("widget-update {id} {patch}"));
        Ok(json!({}))
    }
    fn widget_remove(&self, id: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("widget-remove {id}"));
        Ok(())
    }
    fn games(&self) -> Result<Value, String> { Ok(json!({"active": false, "games": []})) }
    fn game_open(&self, id: Option<&str>) -> Result<(), String> {
        lock(&self.calls).push(format!("game-open {}", id.unwrap_or("lobby")));
        Ok(())
    }
    fn game_close(&self) -> Result<(), String> {
        lock(&self.calls).push("game-close".to_string());
        Ok(())
    }
}

struct Rig {
    _dir: TempDir,
    workspace: PathBuf,
    dirs: Dirs,
    desktop: Arc<FakeDesktop>,
    asked: Arc<Mutex<Vec<(String, String)>>>,
    provider: Arc<ScriptedProvider>,
    agent: Agent,
    todo: Arc<Todo>,
}

/// `answer` decides every confirmation; `None` would be a user who never answers, so tests always give one.
fn rig(tag: &str, steps: Vec<Step>, answer: Approval) -> Rig {
    let dir = TempDir::new(tag);
    let workspace = dir.path("work");
    fs::create_dir_all(&workspace).unwrap();
    let dirs = Dirs { chat: dir.path("chat") };
    let asked: Arc<Mutex<Vec<(String, String)>>> = Arc::default();
    let log = asked.clone();
    let permission = Arc::new(Permission::new(PermissionMode::Ask, move |request: &ApprovalRequest<'_>| {
        lock(&log).push((request.tool.to_string(), risk_name(request.risk).to_string()));
        answer
    }));
    let provider = Arc::new(ScriptedProvider::new(steps));
    let desktop = Arc::new(FakeDesktop::default());
    let todo = Arc::new(Todo::new());
    let agent = assemble(Parts {
        provider: provider.clone(),
        dirs: &dirs,
        workspace: &workspace,
        permission,
        todo: todo.clone(),
        ask: AskUser::new(|_, question, _| Some(format!("回答：{}", question.text))),
        desktop: desktop.clone(),
        utc_offset_minutes: 480,
        open_computer_use: false,
    })
    .expect("the tool set assembles");
    Rig { _dir: dir, workspace, dirs, desktop, asked, provider, agent, todo }
}

fn run(rig: &Rig, text: &str) -> (Result<agent_kernel::RunSummary, agent_kernel::HarnessError>, Vec<Event>) {
    let mut events = Vec::new();
    let result = rig.agent.run("s1", text, &mut |e| events.push(e), &Arc::new(AtomicBool::new(false)));
    (result, events)
}

/// What the model was told about each tool call, in order.
fn tool_results(rig: &Rig) -> Vec<String> {
    let requests = rig.provider.requests();
    requests.last().unwrap().messages.iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect()
}

fn call(id: &str, tool: &str, args: Value) -> Step {
    Step::tool_call(id, tool, &args.to_string())
}

#[test]
fn the_assistant_has_every_tool_it_needs_and_no_others() {
    let rig = rig("tools", vec![Step::text("ok")], Approval::Once);
    let _ = run(&rig, "hi");
    let request = &rig.provider.requests()[0];
    let mut names: Vec<&str> = request.tools.iter().map(|t| t.name.as_str()).collect();
    names.sort();
    let mut expected = vec![
        "time_now", "todo", "ask_user", "read_file", "list_dir", "grep", "write_file", "edit_file", "shell", "web_fetch", "load_skill", "skill_manage",
        "memory_read", "memory_write", "desktop_status", "wallpaper_list", "wallpaper_apply", "effect_list", "effect_set", "companion_set", "care_list",
        "care_set", "care_delete", "widget_list", "widget_add", "widget_update", "widget_remove", "game_list", "game_open", "game_close",
    ];
    expected.sort();
    assert_eq!(names, expected);
    let system = request.messages[0].text();
    assert!(system.contains("小伴") && system.contains(&rig.workspace.display().to_string()), "the prompt names the working folder");
}

#[test]
fn reading_and_planning_need_no_confirmation_but_writing_a_file_does() {
    let steps = vec![
        call("1", "todo", json!({"todos": [{"content": "写笔记", "status": "in_progress"}]})).and_tool_call("2", "list_dir", "{}").and_tool_call("3", "time_now", "{}"),
        call("4", "write_file", json!({"path": "notes/今天.md", "content": "# 今天\n喝水"})),
        Step::text("写好了"),
    ];
    let rig = rig("write", steps, Approval::Once);
    let (result, _) = run(&rig, "记一下");
    assert_eq!(result.unwrap().text, "写好了");
    assert_eq!(*lock(&rig.asked), vec![("write_file".to_string(), "write".to_string())], "only the write asked");
    assert_eq!(fs::read_to_string(rig.workspace.join("notes/今天.md")).unwrap(), "# 今天\n喝水");
    assert_eq!(rig.todo.snapshot("s1").len(), 1);
    let results = tool_results(&rig);
    assert_eq!(results.len(), 4);
    assert!(results[3].starts_with("Created notes/今天.md"), "{}", results[3]);
    assert!(results[0].contains("[~] 写笔记") && results[2].contains("UTC+08:00"), "the offset given by the app is used: {}", results[2]);
}

#[test]
fn a_refused_write_changes_nothing_and_the_model_is_told() {
    let rig = rig("deny", vec![call("1", "write_file", json!({"path": "a.txt", "content": "x"})), Step::text("好吧")], Approval::Deny);
    let _ = run(&rig, "写");
    assert!(!rig.workspace.join("a.txt").exists());
    assert!(tool_results(&rig)[0].contains("denied permission"));
}

#[test]
fn files_outside_the_working_folder_cannot_be_read_or_written() {
    let outside = TempDir::new("outside");
    fs::write(outside.path("secret.txt"), "top secret").unwrap();
    let steps = vec![
        call("1", "read_file", json!({"path": outside.path("secret.txt").display().to_string()})).and_tool_call("2", "read_file", &json!({"path": "../outside/secret.txt"}).to_string()),
        call("3", "write_file", json!({"path": outside.path("planted.txt").display().to_string(), "content": "x"})),
        call("4", "grep", json!({"pattern": "secret", "path": outside.0.display().to_string()})),
        Step::text("done"),
    ];
    let rig = rig("confine", steps, Approval::Once);
    let _ = run(&rig, "看看");
    let results = tool_results(&rig);
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| r.contains("outside the allowed directories") || r.contains("cannot access") || r.contains("`..` is not allowed")), "{results:?}");
    assert!(results.iter().all(|r| !r.contains("top secret")));
    assert!(!outside.path("planted.txt").exists());
}

#[test]
fn very_long_output_is_saved_to_a_file_and_the_model_gets_its_path() {
    let lines: String = (0..3_000).map(|i| format!("line {i} with some text to search for\n")).collect();
    let steps = vec![
        call("1", "write_file", json!({"path": "big.txt", "content": lines})),
        call("2", "grep", json!({"pattern": "text", "path": "big.txt", "max_results": 500})),
        Step::text("ok"),
    ];
    let rig = rig("spill", steps, Approval::Once);
    let _ = run(&rig, "x");
    let grep = &tool_results(&rig)[1];
    let saved: Vec<PathBuf> = fs::read_dir(rig.dirs.spill()).unwrap().flatten().map(|e| e.path()).collect();
    assert_eq!(saved.len(), 1, "{grep}");
    assert!(grep.contains("bytes omitted") && grep.contains(&format!("is saved at {}", saved[0].display())), "{}", &grep[grep.len().saturating_sub(300)..]);
    assert!(fs::metadata(&saved[0]).unwrap().len() > 16_000);
    assert!(grep.len() < 6_000, "the model sees a preview, not the whole output");
}

#[test]
fn the_folder_for_saved_output_can_be_read_but_not_written() {
    let rig = rig("spill-read", vec![Step::text("unused")], Approval::Once);
    let saved = rig.dirs.spill().join("earlier-output.txt");
    fs::write(&saved, "saved earlier").unwrap();
    let steps = vec![
        call("1", "read_file", json!({"path": saved.display().to_string()})),
        call("2", "write_file", json!({"path": rig.dirs.spill().join("planted.txt").display().to_string(), "content": "x"})),
        Step::text("ok"),
    ];
    let reader = {
        let dirs = Dirs { chat: rig.dirs.chat.clone() };
        assemble(Parts {
            provider: Arc::new(ScriptedProvider::new(steps)),
            dirs: &dirs,
            workspace: &rig.workspace,
            permission: Arc::new(Permission::new(PermissionMode::Auto, |_: &ApprovalRequest<'_>| Approval::Once)),
            todo: Arc::new(Todo::new()),
            ask: AskUser::new(|_, _, _| None),
            desktop: Arc::new(FakeDesktop::default()),
            utc_offset_minutes: 0,
            open_computer_use: false,
        })
        .unwrap()
    };
    let _ = reader.run("s2", "go", &mut |_| {}, &Arc::new(AtomicBool::new(false)));
    let messages = JsonlStore::new(rig.dirs.sessions()).load("s2").unwrap();
    let results: Vec<String> = messages.iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect();
    assert!(results[0].contains("saved earlier"), "{results:?}");
    assert!(results[1].contains("outside the allowed directories"), "{results:?}");
    assert!(!rig.dirs.spill().join("planted.txt").exists());
}

#[cfg(unix)]
#[test]
fn a_command_always_asks_runs_in_the_working_folder_and_sees_none_of_our_secrets() {
    std::env::set_var("XIAOBAN_TEST_API_KEY", "sk-should-not-leak");
    let steps = vec![call("1", "shell", json!({"command": "pwd; echo \"[$XIAOBAN_TEST_API_KEY]\"; echo hi > made-by-shell.txt"})), Step::text("ok")];
    let rig = rig("shell", steps, Approval::Once);
    let _ = run(&rig, "跑一下");
    assert_eq!(*lock(&rig.asked), vec![("shell".to_string(), "exec".to_string())]);
    let output = &tool_results(&rig)[0];
    assert!(output.contains(rig.workspace.canonicalize().unwrap().to_str().unwrap()), "{output}");
    assert!(output.contains("[]") && !output.contains("sk-should-not-leak"), "{output}");
    assert!(rig.workspace.join("made-by-shell.txt").exists());
    std::env::remove_var("XIAOBAN_TEST_API_KEY");
}

#[cfg(unix)]
#[test]
fn a_refused_command_never_runs() {
    let rig = rig("shell-deny", vec![call("1", "shell", json!({"command": "touch should-not-exist.txt"})), Step::text("ok")], Approval::Deny);
    let _ = run(&rig, "跑");
    assert!(!rig.workspace.join("should-not-exist.txt").exists());
    assert!(tool_results(&rig)[0].contains("denied permission"));
}

#[test]
fn changing_the_desktop_asks_first_and_then_goes_through_the_desktop() {
    let steps = vec![
        call("1", "wallpaper_list", json!({})),
        call("2", "wallpaper_apply", json!({"id": "companion"})),
        call("3", "care_set", json!({"text": "喝水", "time": "15:00"})),
        Step::text("好了"),
    ];
    let rig = rig("desktop", steps, Approval::Once);
    let _ = run(&rig, "整理一下桌面");
    assert_eq!(*lock(&rig.asked), vec![("wallpaper_apply".to_string(), "write".to_string()), ("care_set".to_string(), "write".to_string())]);
    assert_eq!(*lock(&rig.desktop.calls), vec!["apply companion", "care 喝水"]);

    let refused = rig_denied_desktop();
    assert!(lock(&refused.desktop.calls).is_empty(), "a refused change does not reach the desktop");
}

fn rig_denied_desktop() -> Rig {
    let rig = rig("desktop-deny", vec![call("1", "effect_set", json!({"id": "snow"})), Step::text("ok")], Approval::Deny);
    let _ = run(&rig, "下雪");
    rig
}

#[test]
fn the_assistant_can_ask_the_user_and_remembers_notes_and_skills_in_its_own_folders() {
    let steps = vec![
        call("1", "ask_user", json!({"question": "放哪里？"})),
        call("2", "memory_write", json!({"name": "likes", "content": "喜欢喝茶"})),
        call("3", "skill_manage", json!({"action": "create", "name": "daily-note", "description": "写每日笔记", "content": "按日期建文件"})),
        Step::text("ok"),
    ];
    let rig = rig("memory", steps, Approval::Once);
    let _ = run(&rig, "x");
    assert_eq!(tool_results(&rig)[0], "The user answered: 回答：放哪里？");
    assert_eq!(fs::read_to_string(rig.dirs.memory().join("likes.md")).unwrap(), "喜欢喝茶\n");
    assert!(fs::read_to_string(rig.dirs.skills().join("daily-note/SKILL.md")).unwrap().contains("origin: agent"));
    let asked: Vec<String> = lock(&rig.asked).iter().map(|(tool, _)| tool.clone()).collect();
    assert_eq!(asked, vec!["memory_write", "skill_manage"]);
    assert!(rig.workspace.read_dir().unwrap().next().is_none(), "notes and skills stay out of the user's folder");
}

#[test]
fn a_conversation_is_saved_and_picked_up_again_by_a_new_run() {
    let rig = rig("persist", vec![Step::text("你好呀"), Step::text("记得")], Approval::Once);
    let _ = run(&rig, "第一句");
    let _ = run(&rig, "第二句");
    let second = &rig.provider.requests()[1].messages;
    let texts: Vec<String> = second.iter().map(|m| m.text()).collect();
    assert!(texts.contains(&"第一句".to_string()) && texts.contains(&"你好呀".to_string()), "{texts:?}");
    let saved = JsonlStore::new(rig.dirs.sessions()).load("s1").unwrap();
    assert_eq!(saved.len(), 4);
}

#[test]
fn long_values_are_cut_for_the_confirmation_card_but_short_ones_stay_whole() {
    let long = "字".repeat(5_000);
    let shown = shorten(&json!({"path": "a.txt", "content": long, "nested": {"list": [long.clone(), "short"]}}), 0);
    assert_eq!(shown["path"], "a.txt");
    let content = shown["content"].as_str().unwrap();
    assert!(content.chars().count() < 2_100 && content.ends_with("（共 5000 个字符）"), "{}", content.chars().count());
    assert!(shown["nested"]["list"][0].as_str().unwrap().contains("共 5000"));
    assert_eq!(shown["nested"]["list"][1], "short");
    let exact = "x".repeat(2_000);
    assert_eq!(shorten(&json!(exact.clone()), 0), json!(exact), "exactly the limit is kept");
    let wide = Value::Array((0..80).map(|i| json!(i)).collect());
    assert_eq!(shorten(&wide, 0).as_array().unwrap().len(), 50);
    assert_eq!(shorten(&json!(7), 0), json!(7));
}

#[test]
fn old_spilled_output_is_removed_and_recent_output_is_kept() {
    let dir = TempDir::new("spill-clean");
    let old = dir.path("old.txt");
    let fresh = dir.path("fresh.txt");
    fs::write(&old, "x").unwrap();
    fs::write(&fresh, "x").unwrap();
    fs::File::options().write(true).open(&old).unwrap().set_modified(SystemTime::now() - Duration::from_secs(10 * 86_400)).unwrap();
    clean_spill(&dir.0, SPILL_KEEP);
    assert!(!old.exists() && fresh.exists());
    clean_spill(&dir.path("missing"), SPILL_KEEP);
}

#[test]
fn each_protocol_builds_a_provider_and_unusable_settings_are_refused() {
    for protocol in ["chat", "responses", "anthropic", "something-else"] {
        let config = AiConfig { protocol: protocol.into(), base_url: "https://api.example.com/v1".into(), model: "m".into(), api_key: "k".into(), workspace: String::new(), permission_mode: "ask".into(), hide_tool_calls: false, open_computer_use: false };
        assert!(provider(&config).is_ok(), "{protocol}");
    }
    let empty = AiConfig::default();
    assert!(provider(&empty).err().is_some_and(|e| e.starts_with("模型配置不能用")));
}

#[test]
fn only_the_control_window_may_use_the_chat_commands() {
    for command in ["chat_overview", "chat_load_session", "chat_new_session", "chat_send", "chat_cancel", "chat_delete_session", "chat_answer_approval", "chat_answer_question", "chat_set_permission_mode"] {
        assert!(crate::content::permitted("control", command), "{command}");
        for label in ["pet", "wallpaper", "care", "games", "media-a", "media-b", "media-preview", "ritual", "widgets", "effects"] {
            assert!(!crate::content::permitted(label, command), "{label} must not reach {command}");
        }
    }
}

#[test]
fn dropping_a_conversation_releases_a_run_that_is_waiting_for_the_user() {
    let broker = Broker::default();
    let (_id, rx) = broker.open_approval("s1");
    let waited = Arc::new(AtomicUsize::new(0));
    let counter = waited.clone();
    let handle = std::thread::spawn(move || {
        let ctx = ToolCtx::detached("s1");
        let answer = ctx.wait_for(&rx);
        counter.fetch_add(1, Ordering::SeqCst);
        answer
    });
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(waited.load(Ordering::SeqCst), 0, "still waiting for the user");
    broker.drop_session("s1");
    assert_eq!(handle.join().unwrap(), None);
}

#[test]
fn in_auto_mode_all_write_tools_run_without_asking() {
    let steps = vec![
        call("1", "write_file", json!({"path": "auto.txt", "content": "auto created"})),
        call("2", "care_set", json!({"text": "auto reminder", "time": "12:00"})),
        Step::text("all done"),
    ];
    let dir = TempDir::new("auto-mode");
    let workspace = dir.path("work");
    fs::create_dir_all(&workspace).unwrap();
    let dirs = Dirs { chat: dir.path("chat") };
    let asked: Arc<Mutex<Vec<(String, String)>>> = Arc::default();
    let log = asked.clone();
    let permission = Arc::new(Permission::new(PermissionMode::Auto, move |request: &ApprovalRequest<'_>| {
        lock(&log).push((request.tool.to_string(), risk_name(request.risk).to_string()));
        Approval::Deny
    }));
    let provider = Arc::new(ScriptedProvider::new(steps));
    let desktop = Arc::new(FakeDesktop::default());
    let todo = Arc::new(Todo::new());
    let agent = assemble(Parts {
        provider: provider.clone(),
        dirs: &dirs,
        workspace: &workspace,
        permission: permission.clone(),
        todo: todo.clone(),
        ask: AskUser::new(|_, _, _| None),
        desktop: desktop.clone(),
        utc_offset_minutes: 0,
        open_computer_use: false,
    }).unwrap();

    let mut events = Vec::new();
    let (res, _) = (agent.run("s1", "auto run", &mut |e| events.push(e), &Arc::new(AtomicBool::new(false))), events);
    assert_eq!(res.unwrap().text, "all done");
    assert_eq!(*lock(&asked), Vec::<(String, String)>::new(), "no approval asked in auto mode");
    assert_eq!(fs::read_to_string(workspace.join("auto.txt")).unwrap(), "auto created");
    assert_eq!(*lock(&desktop.calls), vec!["care auto reminder"]);
}



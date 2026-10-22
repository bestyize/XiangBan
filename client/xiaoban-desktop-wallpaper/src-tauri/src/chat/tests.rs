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
    fn create_wallpaper(&self, title: &str, _desc: Option<&str>, _html: &str, apply_now: bool) -> Result<Value, String> {
        lock(&self.calls).push(format!("create-wallpaper {title} {apply_now}"));
        Ok(json!({"id": "local-mock", "title": title, "applied": apply_now}))
    }
    fn delete_wallpaper(&self, id: &str) -> Result<String, String> {
        lock(&self.calls).push(format!("delete-wallpaper {id}"));
        Ok("hash".to_string())
    }
    fn effects(&self) -> Result<Value, String> { Ok(json!({"selected": "", "packs": []})) }
    fn set_effect(&self, id: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("effect {id}"));
        Ok(())
    }
    fn create_effect(&self, pack: &Value, overwrite: bool) -> Result<String, String> {
        lock(&self.calls).push(format!("create-effect {} {overwrite}", pack["id"]));
        Ok(format!("{}.json", pack["id"].as_str().unwrap_or("?")))
    }
    fn delete_effect(&self, id: &str) -> Result<String, String> {
        lock(&self.calls).push(format!("delete-effect {id}"));
        Ok(format!("{id}.json"))
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
    fn custom_widgets(&self) -> Result<Value, String> { Ok(json!([])) }
    fn create_custom_widget(&self, pack: &Value, place_now: bool, overwrite: bool) -> Result<Value, String> {
        let id = pack["id"].as_str().unwrap_or("?");
        lock(&self.calls).push(format!("custom-widget {id} {place_now} {overwrite}"));
        Ok(json!({"id": id, "name": pack["name"], "file": format!("{id}.json"), "placed": place_now}))
    }
    fn delete_custom_widget(&self, id: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("delete-custom-widget {id}"));
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
    fn create_game(&self, id: &str, _name: &str, _description: &str, _files: &[(String, String)], _save: bool, _overwrite: bool) -> Result<serde_json::Value, String> {
        lock(&self.calls).push(format!("create-game {id}"));
        Ok(serde_json::json!({"id": id}))
    }
    fn delete_game(&self, id: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("delete-game {id}"));
        Ok(())
    }
    fn request_permissions(&self) -> Result<Value, String> {
        lock(&self.calls).push("request-permissions".to_string());
        Ok(json!({"ok": true}))
    }
    fn computer_use_repair_permissions(&self, permission: &str) -> Result<Value, String> {
        lock(&self.calls).push(format!("repair-permissions {permission}"));
        Ok(json!({"reset": true, "permission": permission, "permissionCheckRequired": true}))
    }
    fn export_pack(&self, kind: &str, id: &str) -> Result<Option<(String, u64)>, String> {
        lock(&self.calls).push(format!("export-pack {kind} {id}"));
        Ok(None)
    }
    fn open_url(&self, url: &str) -> Result<(), String> {
        lock(&self.calls).push(format!("open-url {url}"));
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
    build_rig(tag, steps, answer, false)
}

/// Computer control switched on, with a server that cannot be started.
fn rig_with_failing_computer_use(tag: &str) -> Rig {
    build_rig(tag, vec![Step::text("好")], Approval::Once, true)
}

fn no_computer_use() -> Result<Arc<agent_kernel::mcp::McpClient>, agent_kernel::HarnessError> {
    Err(agent_kernel::HarnessError::Config("内置电脑控制引擎缺失".into()))
}

fn build_rig(tag: &str, steps: Vec<Step>, answer: Approval, computer_use: bool) -> Rig {
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
        open_computer_use: computer_use,
        spawn_computer_use: &no_computer_use,
        budget: TokenBudget::resolve(ai_config::DEFAULT_CONTEXT_TOKENS, None),
        processes: None,
        mid_run: None,
    })
    .expect("the tool set assembles");
    Rig { _dir: dir, workspace, dirs, desktop, asked, provider, agent, todo }
}

fn run(rig: &Rig, text: &str) -> (Result<agent_kernel::RunSummary, agent_kernel::HarnessError>, Vec<Event>) {
    let mut events = Vec::new();
    let result = rig.agent.run("s1", text, &mut |e| events.push(e), &Arc::new(AtomicBool::new(false)));
    (result, events)
}

#[test]
fn observed_chat_run_persists_events_and_keeps_legacy_chat_stream() {
    let rig = rig("observed-journal", vec![Step::text("任务已回答")], Approval::Once);
    let journal = JsonlRunLog::new(rig.dirs.runs().join("s1"));
    let mut delivered: Vec<(String, Value)> = Vec::new();
    let result = rig.agent.run_parts_observed(
        "s1", vec![Part::Text { text: "测试".into() }],
        RunOptions { max_tool_calls: Some(256), ..Default::default() },
        &mut |record| run_history::forward(&journal, record, "secret-test-key", |channel, payload| {
            delivered.push((channel.into(), payload));
        }),
        &Arc::new(AtomicBool::new(false)),
    ).unwrap();
    let inspection = journal.inspect(&result.run_id).unwrap();
    assert!(inspection.terminal);
    assert_eq!(inspection.state, Some(agent_kernel::RunState::Completed));
    assert!(inspection.incomplete_tools.is_empty());
    assert!(inspection.last_sequence >= 3);
    let records = journal.read_after(&result.run_id, 0, 100).unwrap();
    assert!(records.windows(2).all(|items| items[1].sequence == items[0].sequence + 1));
    assert!(delivered.iter().any(|(kind, value)| kind == "chat-event" && value["event"]["type"] == "done"));
    assert!(delivered.iter().any(|(kind, value)| kind == "chat-run-state" && value["state"] == "completed"));
    assert!(delivered.iter().all(|(_, value)| value["session"] == "s1"));
}

/// What the model was told about each tool call, in order.
fn tool_results(rig: &Rig) -> Vec<String> {
    let requests = rig.provider.requests();
    requests.last().unwrap().messages.iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect()
}

fn call(id: &str, tool: &str, args: Value) -> Step {
    Step::tool_call(id, tool, &args.to_string())
}

/// A tiny valid PNG as a data URL, the shape the web side sends.
const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

#[test]
fn images_are_checked_before_a_message_is_accepted() {
    assert_eq!(clean_image(PNG).unwrap(), PNG, "a pasted PNG is kept as it came");
    assert_eq!(clean_image(&format!("  {PNG}  ")).unwrap(), PNG, "surrounding whitespace is dropped");
    for (bad, why) in [
        ("", "empty"),
        ("   ", "blank"),
        ("https://example.com/a.png", "remote"),
        ("data:image/png,iVBORw0KGgo=", "not base64 labelled"),
        ("data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=", "unsupported type"),
        ("data:image/png;base64,not base64!!", "not base64 data"),
        ("data:image/png;base64,", "no data"),
    ] {
        assert!(clean_image(bad).is_err(), "{why} must be refused: {bad:?}");
    }
    let huge = format!("data:image/png;base64,{}", "A".repeat(MAX_IMAGE_CHARS));
    assert!(clean_image(&huge).is_err(), "an image over the size limit is refused");
}

#[test]
fn a_message_keeps_its_text_its_images_and_nothing_more() {
    let with_text = user_parts("看这个", &[PNG.to_string()]);
    assert_eq!(with_text, vec![Part::Text { text: "看这个".into() }, Part::Image { url: PNG.into() }], "text comes first");
    let image_only = user_parts("   ", &[PNG.to_string()]);
    assert_eq!(image_only, vec![Part::Image { url: PNG.into() }], "a blank caption adds no empty text block");
    assert_eq!(user_parts("只有文字", &[]), vec![Part::Text { text: "只有文字".into() }]);
}

#[test]
fn an_image_reaches_the_model_in_the_form_its_protocol_needs() {
    let rig = rig("images", vec![Step::text("看到了")], Approval::Once);
    let mut events = Vec::new();
    let parts = user_parts("这是什么", &[PNG.to_string()]);
    rig.agent.run_parts("s1", parts, &mut |e| events.push(e), &Arc::new(AtomicBool::new(false))).unwrap();

    let request = &rig.provider.requests()[0];
    let sent = request.messages.iter().find(|m| m.role == Role::User).expect("the user turn reached the model");
    assert_eq!(
        sent.parts,
        vec![Part::Text { text: "这是什么".into() }, Part::Image { url: PNG.into() }],
        "the text and the image both arrived, in order"
    );
    assert!(matches!(events.last(), Some(Event::Done)), "the run finished normally");
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
        "memory_read", "memory_write", "desktop_status", "desktop_windows", "desktop_window_set", "wallpaper_list", "wallpaper_apply", "wallpaper_create", "wallpaper_delete", "effect_list", "effect_set", "effect_create", "effect_delete", "effect_read_format", "companion_set", "care_list",
        "care_set", "care_delete", "widget_list", "widget_add", "widget_create", "widget_delete_custom", "widget_update", "widget_remove", "game_list", "game_open", "game_close", "game_create", "game_delete",
        "request_accessibility_permission", "computer_use_diagnose", "computer_use_repair_permissions", "community_share", "community_open", "pet_read_format", "pet_catalog", "pet_theme_save", "pet_theme_delete", "pet_add", "pet_update", "pet_remove",
        "experience_recall", "skill_search", "skill_inspect", "skill_install", "skill_read", "skill_run",
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
    assert!(results.iter().all(|r| r.contains("outside the allowed directories") || r.contains("outside the authorized workspace") || r.contains("cannot access") || r.contains("`..` is not allowed") || r.contains("file unavailable")), "{results:?}");
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
            spawn_computer_use: &no_computer_use,
        budget: TokenBudget::resolve(ai_config::DEFAULT_CONTEXT_TOKENS, None),
        processes: None,
        mid_run: None,
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
fn changing_the_desktop_is_the_apps_own_work_and_goes_through_the_desktop_unasked() {
    let steps = vec![
        call("1", "wallpaper_list", json!({})),
        call("2", "wallpaper_apply", json!({"id": "companion"})),
        call("3", "care_set", json!({"text": "喝水", "time": "15:00"})),
        call("4", "widget_add", json!({"kind": "clock"})),
        call("5", "desktop_window_set", json!({"target": "assistant", "action": "collapse"})),
        Step::text("好了"),
    ];
    let rig = rig("desktop", steps, Approval::Deny);
    let (result, _) = run(&rig, "整理一下桌面");
    assert_eq!(result.unwrap().text, "好了");
    assert_eq!(*lock(&rig.asked), Vec::<(String, String)>::new(), "the app's own state needs no confirmation");
    assert_eq!(*lock(&rig.desktop.calls), vec!["apply companion", "care 喝水", "widget-add clock"]);
    assert!(tool_results(&rig).iter().all(|text| !text.contains("denied")), "{:?}", tool_results(&rig));
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
        let config = AiConfig { protocol: protocol.into(), base_url: "https://api.example.com/v1".into(), model: "m".into(), api_key: "k".into(), ..AiConfig::default() };
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
        spawn_computer_use: &no_computer_use,
        budget: TokenBudget::resolve(ai_config::DEFAULT_CONTEXT_TOKENS, None),
        processes: None,
        mid_run: None,
    }).unwrap();

    let mut events = Vec::new();
    let (res, _) = (agent.run("s1", "auto run", &mut |e| events.push(e), &Arc::new(AtomicBool::new(false))), events);
    assert_eq!(res.unwrap().text, "all done");
    assert_eq!(*lock(&asked), Vec::<(String, String)>::new(), "no approval asked in auto mode");
    assert_eq!(fs::read_to_string(workspace.join("auto.txt")).unwrap(), "auto created");
    assert_eq!(*lock(&desktop.calls), vec!["care auto reminder"]);
}



/// The real tool set with the app's own permission rules in "完全信任" mode. Anything asked about is refused,
/// so a test can never end up running a dangerous command.
fn trusted_rig(tag: &str, steps: Vec<Step>) -> (Rig, Arc<Mutex<Vec<(String, Option<String>)>>>) {
    let dir = TempDir::new(tag);
    let workspace = dir.path("work");
    fs::create_dir_all(&workspace).unwrap();
    let dirs = Dirs { chat: dir.path("chat") };
    let asked: Arc<Mutex<Vec<(String, Option<String>)>>> = Arc::default();
    let log = asked.clone();
    let shared = Arc::new(Mutex::new(Some(workspace.clone())));
    let permission = Arc::new(permission_with_danger_check(PermissionMode::Auto, move |request: &ApprovalRequest<'_>| {
        lock(&log).push((request.tool.to_string(), request.danger.map(str::to_string)));
        Approval::Deny
    }, shared));
    let provider = Arc::new(ScriptedProvider::new(steps));
    let desktop = Arc::new(FakeDesktop::default());
    let todo = Arc::new(Todo::new());
    let agent = assemble(Parts {
        provider: provider.clone(),
        dirs: &dirs,
        workspace: &workspace,
        permission,
        todo: todo.clone(),
        ask: AskUser::new(|_, _, _| None),
        desktop: desktop.clone(),
        utc_offset_minutes: 0,
        open_computer_use: false,
        spawn_computer_use: &no_computer_use,
        budget: TokenBudget::resolve(ai_config::DEFAULT_CONTEXT_TOKENS, None),
        processes: None,
        mid_run: None,
    })
    .unwrap();
    let rig = Rig { _dir: dir, workspace, dirs, desktop, asked: Arc::default(), provider, agent, todo };
    (rig, asked)
}

#[test]
fn computer_use_repair_requires_confirmation_even_in_full_trust() {
    let steps = vec![
        call("1", "computer_use_repair_permissions", json!({"permission": "accessibility"})),
        call("2", "computer_use_repair_permissions", json!({"permission": "All"})),
        Step::text("等待授权"),
    ];
    let (rig, asked) = trusted_rig("cu-repair-denied", steps);
    run(&rig, "修复旧授权").0.unwrap();
    let asked = lock(&asked);
    assert_eq!(asked.len(), 1, "invalid service is rejected before confirmation");
    assert_eq!(asked[0].0, "computer_use_repair_permissions");
    assert!(asked[0].1.as_ref().unwrap().contains("com.ifuryst.opencomputeruse"));
    assert!(lock(&rig.desktop.calls).is_empty(), "a denied repair must not reach the host");
    assert!(tool_results(&rig)[0].contains("denied"));
}

#[test]
fn computer_use_repair_dispatches_only_the_approved_permission() {
    let rig = rig("cu-repair-approved", vec![
        call("1", "computer_use_repair_permissions", json!({"permission": "accessibility"})),
        Step::text("请在系统界面重新授权"),
    ], Approval::Once);
    run(&rig, "清除旧的无障碍记录").0.unwrap();
    assert_eq!(*lock(&rig.desktop.calls), vec!["repair-permissions accessibility"]);
    assert!(tool_results(&rig)[0].contains("permissionCheckRequired"));
}

#[test]
fn full_trust_runs_everyday_commands_and_stops_only_for_the_dangerous_ones() {
    let steps = vec![
        call("1", "shell", json!({"command": "echo fine"})),
        call("2", "shell", json!({"command": "rm -rf build node_modules"})),
        call("3", "shell", json!({"command": "sudo true"})),
        call("4", "shell", json!({"command": "git push --force origin main"})),
        call("5", "write_file", json!({"path": "notes.txt", "content": "ok"})),
        Step::text("done"),
    ];
    let (rig, asked) = trusted_rig("trusted-danger", steps);
    let (result, _) = run(&rig, "go");
    assert_eq!(result.unwrap().text, "done");
    let asked = lock(&asked).clone();
    assert_eq!(
        asked.iter().map(|(tool, _)| tool.as_str()).collect::<Vec<_>>(),
        vec!["shell", "shell"],
        "only the two dangerous commands asked; the harmless ones and the file write ran on their own"
    );
    assert_eq!(asked[0].1.as_deref(), Some("以管理员权限运行"));
    assert_eq!(asked[1].1.as_deref(), Some("会丢失改动或改写远端历史的 git 操作"));
    let results = tool_results(&rig);
    assert_eq!(results.len(), 5);
    assert!(results[0].contains("fine"), "{}", results[0]);
    assert!(!results[1].contains("denied"), "a delete inside the working folder is not interrupted: {}", results[1]);
    assert!(results[2].contains("denied") && results[3].contains("denied"), "{results:?}");
    assert_eq!(fs::read_to_string(rig.workspace.join("notes.txt")).unwrap(), "ok");
}

#[test]
fn installed_skill_scripts_still_require_permission_in_full_trust() {
    let steps = vec![
        call("1", "skill_install", json!({
            "slug": "demo-skill", "version": "1.0.0", "sha256": "a".repeat(64)
        })),
        call("2", "skill_run", json!({
            "skill": "demo-skill", "script": "scripts/test.py", "args": ["--help"]
        })),
        Step::text("stopped"),
    ];
    let (rig, asked) = trusted_rig("trusted-3p-skill", steps);
    let (result, _) = run(&rig, "go");
    assert_eq!(result.unwrap().text, "stopped");
    let asked = lock(&asked).clone();
    assert_eq!(asked.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
        vec!["skill_install", "skill_run"]);
    for (_, reason) in &asked {
        assert!(reason.as_ref().is_some_and(|s| s.contains("必须逐次经过用户确认")));
    }
    assert!(!rig.dirs.skills().join("demo-skill").exists(),
        "denied third-party package was not installed");
}

#[test]
fn full_trust_is_not_taken_back_after_a_skill_change() {
    let steps = vec![
        call("1", "skill_manage", json!({"action": "create", "name": "tidy-notes", "description": "整理笔记", "content": "步骤"})),
        call("2", "write_file", json!({"path": "after.txt", "content": "x"})),
        Step::text("ok"),
    ];
    let (rig, asked) = trusted_rig("trusted-skill", steps);
    let (result, _) = run(&rig, "go");
    assert_eq!(result.unwrap().text, "ok");
    assert!(lock(&asked).is_empty(), "nothing was asked: {:?}", lock(&asked));
    assert!(rig.dirs.skills().join("tidy-notes/SKILL.md").exists());
}


#[test]
fn the_bundled_computer_use_skill_is_listed_loadable_and_follows_the_switch() {
    let dir = TempDir::new("ocu-skill");
    let skills_dir = dir.path("skills");
    let read = || agent_kernel::skills::Skills::new().read_only_root(&skills_dir);

    sync_computer_use_skill(&skills_dir, true);
    let info = read().find("open-computer-use").expect("the skill is found by its name");
    assert!(!info.agent_created, "the assistant cannot edit or delete it");
    assert!(info.description.contains("get_app_state"), "{}", info.description);
    let body = read().load("open-computer-use").unwrap();
    for rule in ["element_index", "text_limit", "max_tree_nodes", "set_value", "query", "perform_secondary_action", "Drag delivered via app_post"] {
        assert!(body.contains(rule), "the skill teaches {rule}");
    }

    // Switching the tools off takes the skill out of the prompt again.
    sync_computer_use_skill(&skills_dir, false);
    assert!(read().find("open-computer-use").is_none());
    assert!(!skills_dir.join("open-computer-use").exists(), "the empty folder goes too");
}

#[test]
fn the_computer_use_skill_refreshes_its_own_copy_but_never_a_skill_of_the_users() {
    let dir = TempDir::new("ocu-skill-own");
    let skills_dir = dir.path("skills");
    let file = skills_dir.join("open-computer-use/SKILL.md");
    fs::create_dir_all(file.parent().unwrap()).unwrap();

    let stale = COMPUTER_USE_SKILL.replace("改编自开源项目", "旧版：改编自开源项目");
    fs::write(&file, &stale).unwrap();
    sync_computer_use_skill(&skills_dir, true);
    assert_eq!(fs::read_to_string(&file).unwrap(), COMPUTER_USE_SKILL, "an older copy of ours is brought up to date");

    let theirs = "---\nname: open-computer-use\ndescription: 我自己写的\n---\n\n自己的做法\n";
    fs::write(&file, theirs).unwrap();
    sync_computer_use_skill(&skills_dir, true);
    sync_computer_use_skill(&skills_dir, false);
    assert_eq!(fs::read_to_string(&file).unwrap(), theirs, "a file that is not ours is left alone, on or off");
}

#[test]
fn a_computer_use_server_that_cannot_start_is_reported_to_the_model_not_hidden() {
    let rig = rig_with_failing_computer_use("ocu-fail");
    let _ = run(&rig, "帮我点一下浏览器");
    let system = rig.provider.requests()[0].messages[0].text();
    assert!(system.contains("没能启动") && system.contains("不要假装能操作"), "{system}");
    assert!(!system.contains("load_skill 加载 open-computer-use"), "no instructions for tools that are not there");
}

#[test]
fn the_community_skill_is_always_listed_loadable_and_never_overwrites_a_users_own() {
    let dir = TempDir::new("community-skill");
    let skills_dir = dir.path("skills");
    let read = || agent_kernel::skills::Skills::new().read_only_root(&skills_dir);

    sync_community_skill(&skills_dir);
    let info = read().find("xiaoban-community").expect("the skill is found by its name");
    assert!(!info.agent_created, "the assistant cannot edit or delete it");
    let body = read().load("xiaoban-community").unwrap();
    for fact in ["https://wallpaper.xiangban.xyz", "https://chat.xiangban.xyz", "xiaoban-wallpaper", "community_share", "community_open", "zip 资源包"] {
        assert!(body.contains(fact), "the skill teaches {fact}");
    }

    let file = skills_dir.join("xiaoban-community/SKILL.md");
    fs::write(&file, COMMUNITY_SKILL.replace("随程序更新", "旧版：随程序更新")).unwrap();
    sync_community_skill(&skills_dir);
    assert_eq!(fs::read_to_string(&file).unwrap(), COMMUNITY_SKILL, "an older copy of ours is brought up to date");

    let theirs = "---\nname: xiaoban-community\ndescription: 我自己写的\n---\n\n自己的做法\n";
    fs::write(&file, theirs).unwrap();
    sync_community_skill(&skills_dir);
    assert_eq!(fs::read_to_string(&file).unwrap(), theirs);
}

#[test]
fn a_compacted_conversation_is_sent_as_summary_then_the_kept_turn_then_the_new_question() {
    let rig = rig("compacted", vec![Step::text("好的")], Approval::Once);
    let file = JsonlStore::new(rig.dirs.sessions());
    file.append(
        "s9",
        &[Message::user_text("旧问题"), Message::assistant("旧回答", vec![]), Message::user_text("最近的问题"), Message::assistant("最近的回答", vec![]), compact::summary_message(2, "摘要正文")],
    )
    .unwrap();
    rig.agent.run("s9", "新问题", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();

    let sent: Vec<String> = rig.provider.requests()[0].messages.iter().map(|m| m.text()).collect();
    assert_eq!(sent.len(), 5, "{sent:?}");
    assert!(sent[0].len() > 100, "the instructions come first");
    assert!(sent[1].contains("摘要正文") && !sent[1].contains(compact::MARK));
    assert_eq!(&sent[2..], ["最近的问题", "最近的回答", "新问题"]);
    assert!(!sent.iter().any(|t| t.contains("旧问题") || t.contains("旧回答")), "the summarized messages are not sent");

    let saved = file.load("s9").unwrap();
    assert_eq!(saved.len(), 7, "the file keeps every message, the summary and the new exchange");
    assert_eq!(saved[0].text(), "旧问题");
    assert_eq!(saved[5].text(), "新问题");
}

#[test]
fn the_context_size_from_settings_is_what_the_agent_trims_to() {
    let steps = vec![Step::text("好")];
    let tiny = {
        let rig = rig("trim-tiny", steps.clone(), Approval::Once);
        let dirs = Dirs { chat: rig.dirs.chat.clone() };
        let provider = Arc::new(ScriptedProvider::new(steps));
        let agent = assemble(Parts {
            provider: provider.clone(),
            dirs: &dirs,
            workspace: &rig.workspace,
            permission: Arc::new(Permission::new(PermissionMode::Auto, |_: &ApprovalRequest<'_>| Approval::Once)),
            todo: Arc::new(Todo::new()),
            ask: AskUser::new(|_, _, _| None),
            desktop: Arc::new(FakeDesktop::default()),
            utc_offset_minutes: 0,
            open_computer_use: false,
            spawn_computer_use: &no_computer_use,
        budget: TokenBudget::resolve(32_000, None),
        processes: None,
        mid_run: None,
        })
        .unwrap();
        let file = JsonlStore::new(dirs.sessions());
        let long = "字".repeat(30_000);
        file.append("s8", &[Message::user_text(format!("第一个问题{long}")), Message::assistant(long.clone(), vec![]), Message::user_text("第二个问题"), Message::assistant("第二个回答", vec![])]).unwrap();
        agent.run("s8", "第三个问题", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();
        provider.requests()[0].messages.iter().map(|m| m.text()).collect::<Vec<_>>()
    };
    assert!(!tiny.iter().any(|t| t.contains("第一个问题")), "a small limit drops the oldest exchange: {}", tiny.len());
    assert!(tiny.iter().any(|t| t.contains("第三个问题")) && tiny.iter().any(|t| t.contains("第二个问题")));
}

/// A listener registered the usual way (the page's global `listen`) matches every targeted emit, so sending an
/// event once per window would hand it each event twice and the reply would be written twice.
#[test]
fn a_chat_event_reaches_a_global_listener_once_even_with_two_chat_windows() {
    use std::sync::atomic::Ordering;
    use tauri::{Listener, WebviewUrl, WebviewWindowBuilder};
    let app = tauri::test::mock_app();
    let handle = app.handle();
    for label in ["control", crate::float_assistant::LABEL] {
        WebviewWindowBuilder::new(handle, label, WebviewUrl::App("index.html".into())).build().unwrap();
    }
    let seen = Arc::new(AtomicUsize::new(0));
    let counter = seen.clone();
    handle.listen_any("chat-event", move |_| { counter.fetch_add(1, Ordering::SeqCst); });
    emit(handle, "chat-event", json!({ "session": "s", "event": { "type": "text_delta", "text": "你好" } }));
    assert_eq!(seen.load(Ordering::SeqCst), 1);
}


#[test]
fn game_draft_can_be_written_edited_and_installed_in_separate_tool_turns() {
    let steps = vec![
        call("write-page", "write_file", json!({"path":"game-drafts/test/index.html", "content":"<html><head></head><script src='game.js'></script></html>"})),
        call("write-script", "write_file", json!({"path":"game-drafts/test/game.js", "content":"let score = 0;"})),
        call("edit-script", "edit_file", json!({"path":"game-drafts/test/game.js", "old_string":"score = 0", "new_string":"score = 10"})),
        call("install", "game_create", json!({"id":"draft-game", "name":"测试游戏", "description":"分步创作", "source_dir":"game-drafts/test"})),
        Step::text("已保存"),
    ];
    let rig = rig("game-draft-workflow", steps, Approval::Once);
    let (result, events) = run(&rig, "分步做一个游戏");
    assert_eq!(result.unwrap().text, "已保存");
    assert_eq!(fs::read_to_string(rig.workspace.join("game-drafts/test/game.js")).unwrap(), "let score = 10;");
    assert!(lock(&rig.desktop.calls).iter().any(|c| c.starts_with("create-game draft-game")));
    assert!(!events.iter().any(|e| matches!(e, Event::ToolEnd { is_error: true, .. })));
}

#[test]
fn game_create_requires_exactly_one_source_and_rejects_paths_outside_workspace() {
    let steps = vec![
        call("both", "game_create", json!({"id":"draft-game", "name":"游戏", "description":"测试", "source_dir":"draft", "files":[{"path":"index.html", "content":"x"}]})),
        call("neither", "game_create", json!({"id":"draft-game", "name":"游戏", "description":"测试"})),
        call("outside", "game_create", json!({"id":"draft-game", "name":"游戏", "description":"测试", "source_dir":".."})),
        Step::text("未安装"),
    ];
    let rig = rig("game-draft-schema", steps, Approval::Once);
    let (result, events) = run(&rig, "测试错误参数");
    assert!(result.is_ok());
    assert_eq!(events.iter().filter(|e| matches!(e, Event::ToolEnd { is_error: true, .. })).count(), 3);
    assert!(!lock(&rig.desktop.calls).iter().any(|c| c.starts_with("create-game")));
}

#[test]
fn automatic_output_limits_are_protocol_specific_and_independent_of_context_headroom() {
    let mut config: AiConfig = serde_json::from_value(json!({
        "activeProvider":"custom-output", "protocol":"responses",
        "baseUrl":"https://model.example/v1", "model":"test-model", "contextTokens":256000,
        "providers":[{"id":"custom-output", "protocol":"responses",
            "baseUrl":"https://model.example/v1", "model":"test-model",
            "maxOutputTokens":1024,
            "models":[{"id":"test-model","maxOutputTokens":65536}]}]
    })).unwrap();
    let request = agent_kernel::provider::ChatRequest {
        messages: vec![Message::user(vec![Part::Text { text: "hello".into() }])], tools: vec![]
    };
    let body = agent_kernel::provider::responses::build_request_body(&provider_profile(&config), &request);
    assert!(body.get("max_output_tokens").is_none(), "Responses must use the endpoint default");
    assert_eq!(config.token_budget().trim_headroom, 16_384 + 12_800);
    assert_eq!(TokenBudget::resolve(16_000, None).trim_headroom, 4_000 + 800);

    config.protocol = "anthropic".into();
    config.providers[0].protocol = "anthropic".into();
    let body = agent_kernel::provider::anthropic::build_request_body(&provider_profile(&config), &request);
    assert_eq!(body["max_tokens"], 65_536, "Anthropic uses model metadata, not legacy overrides");

    config.protocol = "chat".into();
    config.providers[0].protocol = "chat".into();
    let body = agent_kernel::provider::wire::build_request_body(&provider_profile(&config), &request);
    assert!(body.get("max_tokens").is_none());
    assert!(body.get("max_completion_tokens").is_none());
}

#[test]
fn background_process_tools_are_registered_and_asked_like_the_shell() {
    let dir = TempDir::new("process-tools");
    let workspace = dir.path("work");
    fs::create_dir_all(&workspace).unwrap();
    let dirs = Dirs { chat: dir.path("chat") };
    let manager = Arc::new(ProcessManager::new(&workspace).unwrap());
    let asked: Arc<Mutex<Vec<String>>> = Arc::default();
    let log = asked.clone();
    let workspace_slot = Arc::new(Mutex::new(Some(workspace.clone())));
    let permission = Arc::new(permission_with_danger_check(PermissionMode::Auto, move |request: &ApprovalRequest<'_>| {
        lock(&log).push(request.tool.to_string());
        Approval::Deny
    }, workspace_slot));
    let steps = vec![call("1", "process_start", json!({"command": "sudo true"})), Step::text("好")];
    let agent = assemble(Parts {
        provider: Arc::new(ScriptedProvider::new(steps)),
        dirs: &dirs,
        workspace: &workspace,
        permission,
        todo: Arc::new(Todo::new()),
        ask: AskUser::new(|_, _, _| None),
        desktop: Arc::new(FakeDesktop::default()),
        utc_offset_minutes: 0,
        open_computer_use: false,
        spawn_computer_use: &no_computer_use,
        budget: TokenBudget::resolve(ai_config::DEFAULT_CONTEXT_TOKENS, None),
        processes: Some(manager.clone()),
        mid_run: None,
    })
    .unwrap();
    let names: Vec<String> = agent.tool_schemas().iter().map(|t| t.name.clone()).collect();
    for tool in ["process_start", "process_read", "process_write", "process_stop"] {
        assert!(names.contains(&tool.to_string()), "{tool} missing from {names:?}");
    }
    agent.run("s1", "go", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(*lock(&asked), vec!["process_start".to_string()], "full trust still asks before a flagged background command");
    manager.shutdown().unwrap();
}

#[test]
fn an_answer_that_wrote_files_can_be_undone() {
    let rig = rig("checkpoint-undo", vec![
        call("1", "write_file", json!({"path": "notes.md", "content": "小伴改过"})),
        call("2", "write_file", json!({"path": "new/plan.md", "content": "新建"})),
        Step::text("好了"),
    ], Approval::Once);
    fs::write(rig.workspace.join("notes.md"), "我自己写的").unwrap();
    let (result, _) = run(&rig, "整理笔记");
    result.unwrap();
    assert_eq!(fs::read_to_string(rig.workspace.join("notes.md")).unwrap(), "小伴改过");
    let runs = checkpoints::list(&rig.dirs.checkpoints(), "s1");
    assert_eq!(runs.len(), 1);
    let report = checkpoints::undo(&rig.dirs.checkpoints(), "s1", &runs[0].run_id).unwrap();
    assert_eq!((report.restored, report.removed), (vec!["notes.md".to_string()], vec!["new/plan.md".to_string()]));
    assert_eq!(fs::read_to_string(rig.workspace.join("notes.md")).unwrap(), "我自己写的");
    assert!(!rig.workspace.join("new/plan.md").exists());
}

#[test]
fn the_working_folder_s_agents_md_reaches_the_model_and_edits_apply_next_step() {
    let rig = rig("project-rules", vec![call("1", "time_now", json!({})), Step::text("好")], Approval::Once);
    fs::write(rig.workspace.join("AGENTS.md"), "测试命令：pnpm test").unwrap();
    let provider = rig.provider.clone();
    let (result, _) = run(&rig, "跑测试");
    result.unwrap();
    let requests = provider.requests();
    let system = requests[0].messages[0].text();
    assert!(system.contains("工作目录里的项目说明") && system.contains("测试命令：pnpm test"), "{system}");
}

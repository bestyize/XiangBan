//! The assistant: conversations, the agent that answers them, and the commands the chat page uses.
//!
//! Everything the assistant says, asks or runs is sent to the control window only
//! (`emit_to("control", ..)`): wallpaper pages run content from outside and must not see a chat.

mod broker;
mod desktop_tools;
mod prompt;
mod sessions;

use agent_kernel::builtin::fs::{fs_tools, fs_write_tools};
use agent_kernel::builtin::{AskUser, Question, Shell, TimeNow, Todo, WebFetch};
use agent_kernel::memory::Memory;
use agent_kernel::provider::anthropic::AnthropicMessages;
use agent_kernel::provider::openai::OpenAiCompat;
use agent_kernel::provider::responses::OpenAiResponses;
use agent_kernel::provider::wire::ProviderProfile;
use agent_kernel::skills::Skills;
use agent_kernel::{Agent, Approval, ApprovalRequest, ContextTrim, JsonlStore, Message, ModelProvider, Permission, PermissionMode, Risk, SessionStore, SpillOutput, ToolCtx};
use broker::Broker;
use desktop_tools::{Desktop, TauriDesktop};
use serde_json::{json, Value};
use sessions::{Index, SessionSummary};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager};

use crate::ai_config::{self, AiConfig};

/// Most of what one message may hold. Longer input should be a file.
const MAX_INPUT_CHARS: usize = 30_000;
/// Estimated tokens kept in a request before old tool output and old turns are dropped.
const CONTEXT_TOKENS: usize = 48_000;
const MAX_TURNS: usize = 40;
const SPILL_KEEP: Duration = Duration::from_secs(7 * 86_400);
const ARGUMENT_PREVIEW_CHARS: usize = 2_000;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

fn now() -> (i64, u128) {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    (d.as_secs() as i64, d.as_millis())
}

pub struct ChatState {
    broker: Arc<Broker>,
    permission: Arc<Permission>,
    todo: Arc<Todo>,
    runs: Mutex<HashMap<String, Arc<AtomicBool>>>,
    index: Mutex<Option<Index>>,
}

impl ChatState {
    pub fn new(app: &tauri::AppHandle) -> Self {
        let broker = Arc::new(Broker::default());
        let approver = {
            let (app, broker) = (app.clone(), broker.clone());
            move |request: &ApprovalRequest<'_>| ask_approval(&app, &broker, request)
        };
        let initial_mode = ai_config::current(app).ok()
            .map(|c| if c.permission_mode == "auto" { PermissionMode::Auto } else { PermissionMode::Ask })
            .unwrap_or(PermissionMode::Ask);
        Self {
            broker,
            permission: Arc::new(Permission::new(initial_mode, approver)),
            todo: Arc::new(Todo::new()),
            runs: Mutex::new(HashMap::new()),
            index: Mutex::new(None),
        }
    }
}

fn emit(app: &tauri::AppHandle, name: &str, payload: Value) {
    let _ = app.emit_to("control", name, payload);
}

/// Long values (a whole file about to be written) are cut for the confirmation card.
pub(crate) fn shorten(value: &Value, depth: usize) -> Value {
    match value {
        Value::String(text) if text.chars().count() > ARGUMENT_PREVIEW_CHARS => {
            let cut: String = text.chars().take(ARGUMENT_PREVIEW_CHARS).collect();
            Value::String(format!("{cut}…（共 {} 个字符）", text.chars().count()))
        }
        Value::Array(items) if depth < 6 => Value::Array(items.iter().take(50).map(|v| shorten(v, depth + 1)).collect()),
        Value::Object(map) if depth < 6 => Value::Object(map.iter().take(50).map(|(k, v)| (k.clone(), shorten(v, depth + 1))).collect()),
        other => other.clone(),
    }
}

fn risk_name(risk: Risk) -> &'static str {
    match risk {
        Risk::Read => "read",
        Risk::Write => "write",
        Risk::Network => "network",
        Risk::Exec => "exec",
    }
}

fn ask_approval(app: &tauri::AppHandle, broker: &Broker, request: &ApprovalRequest<'_>) -> Approval {
    let (id, rx) = broker.open_approval(request.session);
    emit(
        app,
        "chat-approval",
        json!({
            "id": id, "session": request.session, "tool": request.tool, "risk": risk_name(request.risk),
            "arguments": shorten(request.arguments, 0), "tainted": request.tainted,
            // Commands and skill changes are asked every time, so "always" would only mislead.
            "canRemember": request.risk != Risk::Exec && request.tool != "skill_manage",
        }),
    );
    let decision = request.wait_for(&rx).unwrap_or(Approval::Deny);
    broker.close_approval(&id);
    decision
}

fn ask_question(app: &tauri::AppHandle, broker: &Broker, session: &str, question: &Question, ctx: &ToolCtx) -> Option<String> {
    let (id, rx) = broker.open_question(session);
    emit(app, "chat-question", json!({ "id": id, "session": session, "question": question.text, "options": question.options }));
    let answer = ctx.wait_for(&rx).flatten();
    broker.close_question(&id);
    answer
}

struct Dirs {
    chat: PathBuf,
}

impl Dirs {
    fn of(app: &tauri::AppHandle) -> Result<Self, String> {
        Ok(Self { chat: crate::storage::root(app)?.join("chat") })
    }
    fn sessions(&self) -> PathBuf { self.chat.join("sessions") }
    fn spill(&self) -> PathBuf { self.chat.join("spill") }
    fn memory(&self) -> PathBuf { self.chat.join("memory") }
    fn skills(&self) -> PathBuf { self.chat.join("skills") }
}

pub(crate) fn provider(config: &AiConfig) -> Result<Arc<dyn ModelProvider>, String> {
    let profile = ProviderProfile::new(config.base_url.clone(), config.api_key.clone(), config.model.clone());
    let made: Result<Arc<dyn ModelProvider>, agent_kernel::HarnessError> = match config.protocol.as_str() {
        "responses" => OpenAiResponses::new(profile).map(|p| Arc::new(p) as Arc<dyn ModelProvider>),
        "anthropic" => AnthropicMessages::new(profile).map(|p| Arc::new(p) as Arc<dyn ModelProvider>),
        _ => OpenAiCompat::new(profile).map(|p| Arc::new(p) as Arc<dyn ModelProvider>),
    };
    made.map_err(|e| format!("模型配置不能用：{e}"))
}

/// Deletes spilled tool output nobody will look at again.

fn open_computer_use_config() -> agent_kernel::mcp::McpConfig {
    let in_path = std::process::Command::new("open-computer-use")
        .arg("-v")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success());

    #[cfg(windows)]
    let npx = "npx.cmd";
    #[cfg(not(windows))]
    let npx = "npx";

    let (cmd, args): (&str, Vec<&str>) = if in_path {
        ("open-computer-use", vec!["mcp"])
    } else {
        (npx, vec!["-y", "open-computer-use", "mcp"])
    };

    agent_kernel::mcp::McpConfig::new(cmd, &args)
        .default_risk(Risk::Exec)
        .timeout(Duration::from_secs(60))
}

pub fn spawn_open_computer_use() -> Result<Arc<agent_kernel::mcp::McpClient>, agent_kernel::HarnessError> {
    agent_kernel::mcp::McpClient::spawn(open_computer_use_config())
}

fn clean_spill(dir: &Path, keep: Duration) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let old = entry.metadata().ok().filter(|m| m.is_file()).and_then(|m| m.modified().ok()).and_then(|t| t.elapsed().ok()).is_some_and(|age| age > keep);
        if old {
            let _ = fs::remove_file(entry.path());
        }
    }
}

struct Parts<'a> {
    provider: Arc<dyn ModelProvider>,
    dirs: &'a Dirs,
    workspace: &'a Path,
    permission: Arc<Permission>,
    todo: Arc<Todo>,
    ask: AskUser,
    desktop: Arc<dyn Desktop>,
    utc_offset_minutes: i32,
    open_computer_use: bool,
}

/// Puts the kernel together: the model, the tools the assistant may use, and the rules around them.
fn assemble(parts: Parts<'_>) -> Result<Agent, String> {
    let Parts { provider, dirs, workspace, permission, todo, ask, desktop, utc_offset_minutes, open_computer_use } = parts;
    for dir in [dirs.sessions(), dirs.spill(), dirs.memory(), dirs.skills()] {
        fs::create_dir_all(&dir).map_err(|e| format!("无法创建 {}：{e}", dir.display()))?;
    }
    clean_spill(&dirs.spill(), SPILL_KEEP);
    let skills = Arc::new(Skills::new().writable_root(dirs.skills()));
    let memory = Arc::new(Memory::new(dirs.memory()));
    let environment = prompt::Environment { os: std::env::consts::OS, workspace, open_computer_use };

    let mut builder = Agent::builder(provider, Arc::new(JsonlStore::new(dirs.sessions())))
        .system_prompt(prompt::system_prompt(&environment))
        .prompt_section(skills.prompt_section())
        .prompt_section(memory.prompt_section())
        .tool(TimeNow::new().utc_offset_minutes(utc_offset_minutes))
        .tool_arc(todo)
        .tool(ask)
        .tool(Shell::new(workspace))
        .tool(WebFetch::new())
        .middleware(permission)
        .middleware(SpillOutput::new(dirs.spill()))
        .middleware(ContextTrim::new(CONTEXT_TOKENS))
        .max_turns(MAX_TURNS);
    let readable = fs_tools(vec![workspace.to_path_buf(), dirs.spill()]).map_err(|e| e.to_string())?;
    let writable = fs_write_tools(vec![workspace.to_path_buf()]).map_err(|e| e.to_string())?;
    for tool in readable.into_iter().chain(writable).chain(skills.tools()).chain(memory.tools()).chain(desktop_tools::tools(desktop)) {
        builder = builder.tool_arc(tool);
    }
    if open_computer_use {
        if let Ok(mcp_client) = spawn_open_computer_use() {
            for tool in mcp_client.tools() {
                builder = builder.tool_arc(tool);
            }
        }
    }
    builder.build().map_err(|e| e.to_string())
}

fn build_agent(app: &tauri::AppHandle, config: &AiConfig, workspace: &Path) -> Result<Agent, String> {
    let state = app.state::<ChatState>();
    let ask = {
        let (app, broker) = (app.clone(), state.broker.clone());
        AskUser::new(move |session, question, ctx| ask_question(&app, &broker, session, question, ctx))
    };
    assemble(Parts {
        provider: provider(config)?,
        dirs: &Dirs::of(app)?,
        workspace,
        permission: state.permission.clone(),
        todo: state.todo.clone(),
        ask,
        desktop: Arc::new(TauriDesktop { app: app.clone() }),
        utc_offset_minutes: chrono::Local::now().offset().local_minus_utc() / 60,
        open_computer_use: config.open_computer_use,
    })
}

/// Runs `change` on the list of conversations and saves it; the first use loads it from disk.
fn mutate<T>(app: &tauri::AppHandle, change: impl FnOnce(&mut Index) -> T) -> Result<T, String> {
    let state = app.state::<ChatState>();
    let mut guard = lock(&state.index);
    if guard.is_none() {
        let dirs = Dirs::of(app)?;
        fs::create_dir_all(dirs.sessions()).map_err(|e| format!("无法创建对话目录：{e}"))?;
        let store = JsonlStore::new(dirs.sessions());
        *guard = Some(Index::load(&dirs.chat, &dirs.sessions(), &store));
    }
    let index = guard.as_mut().expect("loaded above");
    let result = change(index);
    index.save().map_err(|e| format!("无法保存对话列表：{e}"))?;
    Ok(result)
}

fn list(app: &tauri::AppHandle) -> Result<Vec<SessionSummary>, String> {
    mutate(app, |index| index.list())
}

fn finish_run(app: &tauri::AppHandle, session: &str) {
    let state = app.state::<ChatState>();
    lock(&state.runs).remove(session);
    state.broker.drop_session(session);
    if let Ok(sessions) = mutate(app, |index| {
        index.touch(session, now().0);
        index.list()
    }) {
        emit(app, "chat-sessions-changed", json!(sessions));
    }
}

#[tauri::command]
pub fn chat_overview(app: tauri::AppHandle) -> Result<Value, String> {
    let config = ai_config::current(&app)?;
    let default = ai_config::default_workspace(&app)?;
    let running: Vec<String> = lock(&app.state::<ChatState>().runs).keys().cloned().collect();
    let mode = if app.state::<ChatState>().permission.mode() == PermissionMode::Auto { "auto" } else { "ask" };
    Ok(json!({
        "configured": config.is_complete(),
        "sessions": list(&app)?,
        "running": running,
        "workspace": config.workspace_or(&default).display().to_string(),
        "permissionMode": mode,
        "hideToolCalls": config.hide_tool_calls,
    }))
}

#[tauri::command]
pub fn chat_load_session(app: tauri::AppHandle, session_id: String) -> Result<Vec<Message>, String> {
    if !sessions::valid_id(&session_id) {
        return Err("对话编号不对".into());
    }
    let known = mutate(&app, |index| index.get(&session_id).is_some())?;
    if !known {
        return Ok(Vec::new());
    }
    JsonlStore::new(Dirs::of(&app)?.sessions()).load(&session_id).map_err(|e| e.to_string())
}

/// An empty conversation. The chat page makes one when the first message is sent, so the page
/// already knows the id when the answer's events start to arrive.
#[tauri::command]
pub fn chat_new_session(app: tauri::AppHandle) -> Result<SessionSummary, String> {
    let (seconds, millis) = now();
    mutate(&app, |index| index.create(seconds, millis))?
}

#[tauri::command]
pub fn chat_send(app: tauri::AppHandle, session_id: String, text: String) -> Result<SessionSummary, String> {
    if !sessions::valid_id(&session_id) {
        return Err("对话编号不对".into());
    }
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("请先输入内容".into());
    }
    if text.chars().count() > MAX_INPUT_CHARS {
        return Err(format!("一次最多发送 {MAX_INPUT_CHARS} 个字，更长的内容请放进文件再让小伴去读"));
    }
    let state = app.state::<ChatState>();
    if lock(&state.runs).contains_key(&session_id) {
        return Err("这个对话还在回答，请等它结束或先停止".into());
    }
    let config = ai_config::current(&app)?;
    if !config.is_complete() {
        return Err("还没有配置模型，请先到“设置 → AI 模型”填写".into());
    }
    let default = ai_config::default_workspace(&app)?;
    let workspace = config.workspace_or(&default).to_path_buf();
    if workspace == default {
        fs::create_dir_all(&workspace).map_err(|e| format!("无法创建工作目录：{e}"))?;
    }
    if !workspace.is_dir() {
        return Err(format!("工作目录不存在：{}，请到“设置 → AI 模型”重新选择", workspace.display()));
    }

    let (seconds, _) = now();
    let summary = mutate(&app, |index| -> Result<SessionSummary, String> {
        let summary = index.get(&session_id).cloned().ok_or("这个对话已经不存在")?;
        index.name_from_first_message(&summary.id, &text);
        index.touch(&summary.id, seconds);
        Ok(index.get(&summary.id).cloned().unwrap_or(summary))
    })??;

    let cancel = Arc::new(AtomicBool::new(false));
    if lock(&state.runs).insert(summary.id.clone(), cancel.clone()).is_some() {
        return Err("这个对话还在回答，请等它结束或先停止".into());
    }

    // Build the agent and run it entirely on a dedicated OS thread.
    // reqwest::blocking::Client internally manages its own current-thread runtime;
    // building or dropping it inside Tokio's async worker threads panics.
    let (handle, id, text_owned, config_owned, workspace_owned) = (app.clone(), summary.id.clone(), text, config, workspace);
    let started = std::thread::Builder::new().name(format!("chat-{id}")).spawn(move || {
        let sender = handle.clone();
        let session = id.clone();
        let agent = match build_agent(&handle, &config_owned, &workspace_owned) {
            Ok(agent) => agent,
            Err(error) => {
                emit(&sender, "chat-event", json!({ "session": session, "event": { "type": "error", "message": error } }));
                finish_run(&handle, &id);
                return;
            }
        };
        let _ = agent.run(&id, &text_owned, &mut |event| emit(&sender, "chat-event", json!({ "session": session, "event": event })), &cancel);
        finish_run(&handle, &id);
    });
    if let Err(error) = started {
        lock(&state.runs).remove(&summary.id);
        return Err(format!("无法开始回答：{error}"));
    }
    Ok(summary)
}

#[tauri::command]
pub fn chat_cancel(app: tauri::AppHandle, session_id: String) {
    let state = app.state::<ChatState>();
    let runs = lock(&state.runs);
    if let Some(cancel) = runs.get(&session_id) {
        cancel.store(true, Ordering::SeqCst);
    }
}

#[tauri::command]
pub fn chat_delete_session(app: tauri::AppHandle, session_id: String) -> Result<Vec<SessionSummary>, String> {
    if !sessions::valid_id(&session_id) {
        return Err("对话编号不对".into());
    }
    let state = app.state::<ChatState>();
    if lock(&state.runs).contains_key(&session_id) {
        return Err("这个对话还在回答，请先停止再删除".into());
    }
    JsonlStore::new(Dirs::of(&app)?.sessions()).remove(&session_id).map_err(|e| e.to_string())?;
    state.permission.forget_session(&session_id);
    state.todo.clear_session(&session_id);
    mutate(&app, |index| {
        index.remove(&session_id);
        index.list()
    })
}

#[tauri::command]
pub fn chat_answer_approval(app: tauri::AppHandle, id: String, decision: String) -> Result<(), String> {
    let decision = match decision.as_str() {
        "once" => Approval::Once,
        "always" => Approval::Always,
        "deny" => Approval::Deny,
        _ => return Err("不认识这个选择".into()),
    };
    app.state::<ChatState>().broker.answer_approval(&id, decision)
}

#[tauri::command]
pub fn chat_answer_question(app: tauri::AppHandle, id: String, answer: Option<String>) -> Result<(), String> {
    let answer = answer.map(|a| a.trim().to_string()).filter(|a| !a.is_empty());
    app.state::<ChatState>().broker.answer_question(&id, answer)
}

#[tauri::command]
pub fn chat_set_permission_mode(app: tauri::AppHandle, mode: String) -> Result<String, String> {
    if !["ask", "auto"].contains(&mode.as_str()) {
        return Err("不支持的权限模式".into());
    }
    let perm_mode = if mode == "auto" { PermissionMode::Auto } else { PermissionMode::Ask };
    let state = app.state::<ChatState>();
    state.permission.set_mode(perm_mode);

    let _ = ai_config::update_permission_mode(&app, &mode);

    emit(&app, "chat-permission-mode-changed", json!({ "mode": mode }));
    Ok(mode)
}

#[cfg(test)]
mod tests;

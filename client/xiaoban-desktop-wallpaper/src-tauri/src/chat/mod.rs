//! The assistant: conversations, the agent that answers them, and the commands the chat page uses.
//!
//! Everything the assistant says, asks or runs is sent to the control window and the floating assistant's
//! window only (one `emit_filter`): wallpaper pages run content from outside and must not see a chat.

mod broker;
mod community;
mod compact;
mod danger;
mod desktop_tools;
mod game_draft;
mod goals;
pub use goals::{chat_goal_get, chat_goal_set, chat_goal_start, chat_goal_pause, chat_goal_acknowledge, chat_goal_finish, chat_goal_delete};
mod prompt;
mod run_history;
pub(crate) mod task_outcome;
mod learning;
mod skill_evolution;
pub(crate) mod store;
mod sessions;

use agent_computer_use_tools::{SKILL as COMPUTER_USE_SKILL, SKILL_NAME as COMPUTER_USE_SKILL_NAME, SKILL_ORIGIN_MARKER};
use agent_kernel::builtin::fs::{fs_tools, fs_write_tools};
use agent_file_tools::{Attachment, AttachmentInput, AttachmentStore, AttachmentHistory, AttachmentContext, ReadFile};
use agent_kernel::builtin::{AskUser, Question, Shell, TimeNow, Todo, WebFetch};
use agent_kernel::memory::Memory;
use agent_kernel::provider::anthropic::AnthropicMessages;
use agent_kernel::provider::openai::OpenAiCompat;
use agent_kernel::provider::responses::OpenAiResponses;
use agent_kernel::provider::wire::ProviderProfile;
use agent_kernel::model::TokenBudget;
use agent_kernel::skills::Skills;
use agent_kernel::{Agent, Approval, ApprovalRequest, ContextTrim, ContinueGuard, JsonlStore, RepeatGuard, Message, ModelProvider, Part, Permission, PermissionMode, Risk, RunOptions, RunEventBody, SessionStore, SpillOutput, ToolCtx};
use agent_kernel::journal::JsonlRunLog;
use agent_learning::LearningDb;
use agent_skill_discovery::SkillDiscovery;
use broker::Broker;
use compact::{CompactingStore, ContextInfo};
use desktop_tools::{Desktop, TauriDesktop};
use serde_json::{json, Value};
use sessions::{Index, Library, SessionSummary};
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
use crate::ai_gateway::{self, KeySource};

/// Most of what one message may hold. Longer input should be a file.
const MAX_INPUT_CHARS: usize = 30_000;
/// How many images one message may carry.
const MAX_IMAGES: usize = 8;
/// Largest single image, measured in the encoded data URL. Base64 inflates by a third, so this is
/// roughly 3 MiB of original bytes — enough for a screenshot without bloating the session file.
const MAX_IMAGE_CHARS: usize = 4 * 1024 * 1024;
/// Image types the three protocols accept. SVG and friends are rejected: providers either refuse
/// them or need them rasterised first.
const IMAGE_MEDIA_TYPES: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];
/// Model calls in one answer. Building something real (a game, a set of files) can take dozens of steps;
/// `RepeatGuard` and `ContinueGuard`'s own cap keep a stuck model from using them all.
const MAX_TURNS: usize = 60;
const SPILL_KEEP: Duration = Duration::from_secs(7 * 86_400);
const ARGUMENT_PREVIEW_CHARS: usize = 2_000;

/// Checks one image and returns the `data:` URL to store, or why it was refused.
///
/// Every protocol the kernel speaks wants the image inline as a data URL: OpenAI puts it in
/// `image_url`, Anthropic takes the base64 plus its `media_type`, and Responses passes the URL
/// through. A remote `http(s)` URL would therefore be fetched by the provider, not by us, and
/// would leak a reference to whatever host it points at; only inline data is accepted here.
fn clean_image(raw: &str) -> Result<String, String> {
    let url = raw.trim();
    if url.is_empty() {
        return Err("图片内容为空".into());
    }
    if url.len() > MAX_IMAGE_CHARS {
        return Err("这张图片太大了，请换一张小于 3MB 的".into());
    }
    let Some((meta, data)) = url.strip_prefix("data:").and_then(|rest| rest.split_once(',')) else {
        return Err("只支持本机粘贴或选择的图片".into());
    };
    let Some(media_type) = meta.strip_suffix(";base64") else {
        return Err("图片必须是 base64 编码的".into());
    };
    if !IMAGE_MEDIA_TYPES.contains(&media_type) {
        return Err(format!("不支持这种图片格式（{media_type}），请用 PNG、JPEG、GIF 或 WebP"));
    }
    if data.is_empty() || !data.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=') {
        return Err("图片数据不完整，请重新粘贴".into());
    }
    Ok(url.to_string())
}

/// The user's parts in the order the protocols expect: text first, then images.
///
/// Anthropic and Responses both reject an empty text block, so text is dropped when blank; but a
/// message has to keep *something*, so a blank message with images keeps only its images.
fn user_parts(text: &str, images: &[String]) -> Vec<Part> {
    let mut parts = Vec::with_capacity(images.len() + 1);
    if !text.trim().is_empty() {
        parts.push(Part::Text { text: text.to_string() });
    }
    parts.extend(images.iter().map(|url| Part::Image { url: url.clone() }));
    parts
}

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
    /// The folder the shell works in, as of the latest message. The danger check needs it to tell work inside from outside.
    workspace: Arc<Mutex<Option<PathBuf>>>,
    todo: Arc<Todo>,
    runs: Mutex<HashMap<String, Arc<AtomicBool>>>,
    index: Mutex<Option<Index>>,
    /// Serializes goal record read/modify/write operations inside this host.
    goal_io: Mutex<()>,
    /// Conversations whose automatic compaction failed lately.
    auto_compact: compact::AutoCooldown,
    /// Actual schemas from the latest registration, keyed by whether native GUI tools are enabled.
    tool_tokens: Mutex<HashMap<bool, usize>>,
    connectors: Mutex<Option<Arc<agent_connector_tools::ConnectorManager>>>,
}

impl ChatState {
    pub fn new(app: &tauri::AppHandle) -> Self {
        let broker = Arc::new(Broker::default());
        let approver = {
            let (app, broker) = (app.clone(), broker.clone());
            move |request: &ApprovalRequest<'_>| ask_approval(&app, &broker, request)
        };
        let workspace = Arc::new(Mutex::new(None));
        Self {
            broker,
            // Each conversation carries its own mode and sets it again before every run.
            permission: Arc::new(permission_with_danger_check(PermissionMode::Ask, approver, workspace.clone())),
            workspace,
            todo: Arc::new(Todo::new()),
            runs: Mutex::new(HashMap::new()),
            index: Mutex::new(None),
            goal_io: Mutex::new(()),
            auto_compact: compact::AutoCooldown::default(),
            tool_tokens: Mutex::new(HashMap::new()),
            connectors: Mutex::new(None),
        }
    }
}

/// "完全信任" runs everything except what `danger::assess` flags, which is asked every time.
/// Shared by the app and its tests so the tests exercise the rules that really apply.
fn permission_with_danger_check(mode: PermissionMode, approver: impl agent_kernel::Approver + 'static, workspace: Arc<Mutex<Option<PathBuf>>>) -> Permission {
    Permission::new(mode, approver).danger_check(move |tool, args| {
        // In Auto mode, always_ask alone does not trigger approval when a danger
        // policy is installed; third-party skill installation must be flagged here.
        if tool == "skill_install" || tool == "skill_run" || tool.starts_with("mcp_ext_") {
            return Some("安装第三方 Skill、执行脚本或调用外部 MCP 工具必须逐次经过用户确认".to_string());
        }
        if tool == "computer_use_repair_permissions" {
            return Some("清除 com.ifuryst.opencomputeruse 的所选旧授权（同标识独立安装版也受影响），之后需重新授权；每次都需确认".to_string());
        }
        danger::assess(tool, args, lock(&workspace).as_deref()).map(str::to_string)
    })
}

fn emit<R: tauri::Runtime>(app: &tauri::AppHandle<R>, name: &str, payload: Value) {
    // One emit, not one per window: a listener registered the usual way (the page's global `listen`) matches every
    // targeted emit, so emitting to each window in turn would hand that listener every event twice.
    let _ = app.emit_filter(name, payload, |target| {
        use tauri::EventTarget::{AnyLabel, Webview, WebviewWindow, Window};
        matches!(target, AnyLabel { label } | Webview { label } | WebviewWindow { label } | Window { label }
            if label == "control" || label == crate::float_assistant::LABEL)
    });
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
    // In full trust a tainted conversation is not asked because of the taint, so the card must not say it is.
    let tainted_by_policy = request.tainted && app.state::<ChatState>().permission.mode() != PermissionMode::Auto;
    let (id, rx) = broker.open_approval(request.session);
    crate::proactivity::observe_task_state(app, agent_proactivity::context::TaskState::AwaitingUser);
    emit(
        app,
        "chat-approval",
        json!({
            "id": id, "session": request.session, "tool": request.tool, "risk": risk_name(request.risk),
            "arguments": shorten(request.arguments, 0), "tainted": tainted_by_policy, "danger": request.danger,
            // Commands, skill changes and flagged calls are asked every time, so "always" would only mislead.
            "canRemember": request.risk != Risk::Exec && request.tool != "skill_manage" && request.danger.is_none(),
        }),
    );
    let decision = request.wait_for(&rx).unwrap_or(Approval::Deny);
    broker.close_approval(&id);
    crate::proactivity::observe_task_state(app, agent_proactivity::context::TaskState::Running);
    decision
}

fn ask_question(app: &tauri::AppHandle, broker: &Broker, session: &str, question: &Question, ctx: &ToolCtx) -> Option<String> {
    let (id, rx) = broker.open_question(session);
    crate::proactivity::observe_task_state(app, agent_proactivity::context::TaskState::AwaitingUser);
    emit(app, "chat-question", json!({ "id": id, "session": session, "question": question.text, "options": question.options }));
    let answer = ctx.wait_for(&rx).flatten();
    broker.close_question(&id);
    crate::proactivity::observe_task_state(app, agent_proactivity::context::TaskState::Running);
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
    fn runs(&self) -> PathBuf { self.chat.join("runs") }
    fn outcomes(&self) -> PathBuf { self.chat.join("outcomes") }
    fn learning_db(&self) -> PathBuf { self.chat.join("agent-learning.sqlite") }
    fn spill(&self) -> PathBuf { self.chat.join("spill") }
    fn memory(&self) -> PathBuf { self.chat.join("memory") }
    fn skills(&self) -> PathBuf { self.chat.join("skills") }
}

pub(crate) fn provider(config: &AiConfig) -> Result<Arc<dyn ModelProvider>, String> {
    provider_with(config, None)
}

fn provider_profile(config: &AiConfig) -> ProviderProfile {
    let mut profile = ProviderProfile::new(config.base_url.clone(), config.api_key.clone(), config.model.clone());
    profile.extra_body = crate::ai_reasoning::body(&config.protocol, config.reply_preference(), &config.reasoning_levels());
    profile.max_request_bytes = Some(8 * 1024 * 1024);
    // Central model policy: compatible APIs use endpoint defaults; Anthropic
    // receives a required model/endpoint-specific max_tokens value.
    let capabilities = config.model_capabilities();
    profile.max_output_tokens = capabilities.request_output_limit();
    profile.supports_tools = capabilities.supports_tools.unwrap_or(true);
    profile
}

fn provider_with(config: &AiConfig, key_source: Option<KeySource>) -> Result<Arc<dyn ModelProvider>, String> {
    let mut profile = provider_profile(config);
    if let Some(source) = key_source {
        profile = profile.with_api_key_source(source);
    }
    let made: Result<Arc<dyn ModelProvider>, agent_kernel::HarnessError> = match config.protocol.as_str() {
        "responses" => OpenAiResponses::new(profile).map(|p| Arc::new(p) as Arc<dyn ModelProvider>),
        "anthropic" => AnthropicMessages::new(profile).map(|p| Arc::new(p) as Arc<dyn ModelProvider>),
        _ => OpenAiCompat::new(profile).map(|p| Arc::new(p) as Arc<dyn ModelProvider>),
    };
    made.map_err(|e| format!("模型配置不能用：{e}"))
}

/// Deletes spilled tool output nobody will look at again.
fn clean_spill(dir: &Path, keep: Duration) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let old = entry.metadata().ok().filter(|m| m.is_file()).and_then(|m| m.modified().ok()).and_then(|t| t.elapsed().ok()).is_some_and(|age| age > keep);
        if old {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Keeps the bundled computer-control skill in the assistant's skill folder while the tools are on, and removes
/// it when they are off so the prompt does not advertise something that is not there. Only a file that is ours is touched.
fn sync_computer_use_skill(skills: &Path, enabled: bool) {
    sync_bundled_skill(skills, COMPUTER_USE_SKILL_NAME, COMPUTER_USE_SKILL, SKILL_ORIGIN_MARKER, enabled);
}

/// What the assistant knows about the official website and the forum, shipped with the app so it stays in step with the tools.
const COMMUNITY_SKILL: &str = include_str!("../../skills/xiaoban-community/SKILL.md");
const COMMUNITY_SKILL_NAME: &str = "xiaoban-community";

fn sync_community_skill(skills: &Path) {
    sync_bundled_skill(skills, COMMUNITY_SKILL_NAME, COMMUNITY_SKILL, "小伴桌面内置技能", true);
}

/// Writes or removes a skill that ships with the app. A file counts as ours when it contains `marker`;
/// one the user wrote under the same name is never touched.
fn sync_bundled_skill(skills: &Path, name: &str, text: &str, marker: &str, enabled: bool) {
    let dir = skills.join(name);
    let file = dir.join("SKILL.md");
    let ours = fs::read_to_string(&file).map(|existing| existing.contains(marker)).unwrap_or(false);
    if enabled {
        if !file.exists() || ours {
            if fs::read_to_string(&file).map(|existing| existing != text).unwrap_or(true) {
                let _ = fs::create_dir_all(&dir);
                let _ = fs::write(&file, text);
            }
        }
    } else if ours {
        let _ = fs::remove_file(&file);
        let _ = fs::remove_dir(&dir);
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
    /// Starts the computer-control server; a parameter so tests never launch a real one.
    spawn_computer_use: &'a dyn Fn() -> Result<Arc<agent_kernel::mcp::McpClient>, agent_kernel::HarnessError>,
    /// Estimated tokens kept in a request before old tool output and old turns are dropped.
    budget: TokenBudget,
}

/// Puts the kernel together: the model, the tools the assistant may use, and the rules around them.
#[cfg(test)]
fn assemble(parts: Parts<'_>) -> Result<Agent, String> {
    assemble_with_files(parts, Vec::new())
}

fn assemble_with_files(parts: Parts<'_>, attachments: Vec<Attachment>) -> Result<Agent, String> {
    assemble_with_connectors(parts, attachments, Vec::new())
}
fn assemble_with_connectors(parts: Parts<'_>, attachments: Vec<Attachment>, connector_tools: Vec<Arc<dyn agent_kernel::Tool>>) -> Result<Agent, String> {
    let Parts { provider, dirs, workspace, permission, todo, ask, desktop, utc_offset_minutes, open_computer_use, spawn_computer_use, budget } = parts;
    for dir in [dirs.sessions(), dirs.spill(), dirs.memory(), dirs.skills()] {
        fs::create_dir_all(&dir).map_err(|e| format!("无法创建 {}：{e}", dir.display()))?;
    }
    clean_spill(&dirs.spill(), SPILL_KEEP);
    sync_computer_use_skill(&dirs.skills(), open_computer_use);
    sync_community_skill(&dirs.skills());
    let bundled = dirs.chat.join("bundled-skills");
    let file_skill = bundled.join("file-analysis");
    fs::create_dir_all(&file_skill).map_err(|e| e.to_string())?;
    fs::write(file_skill.join("SKILL.md"), agent_file_tools::ANALYSIS_SKILL).map_err(|e| e.to_string())?;
    fs::create_dir_all(file_skill.join("scripts")).map_err(|e| e.to_string())?;
    fs::write(file_skill.join("scripts/csv_summary.py"), agent_file_tools::CSV_SUMMARY_SCRIPT).map_err(|e| e.to_string())?;
    let skills = Arc::new(Skills::new().read_only_root(bundled).writable_root(dirs.skills()));
    let memory = Arc::new(Memory::new(dirs.memory()));
    let discovery = SkillDiscovery::new(dirs.skills(), workspace.to_path_buf());
    // Started before the prompt is written, so the prompt can say honestly whether the tools exist.
    let computer_use = if open_computer_use { Some(spawn_computer_use()) } else { None };
    let computer_use_error = match &computer_use { Some(Err(error)) => Some(error.to_string()), _ => None };
    let environment = prompt::Environment {
        os: std::env::consts::OS,
        workspace,
        open_computer_use: matches!(computer_use, Some(Ok(_))),
        open_computer_use_error: computer_use_error.as_deref(),
    };

    let mut builder = Agent::builder(provider, Arc::new(AttachmentHistory::new(CompactingStore::new(JsonlStore::new(dirs.sessions())), attachments.clone())))
        .system_prompt(prompt::system_prompt(&environment))
        .prompt_section(skills.prompt_section())
        .prompt_section(memory.prompt_section())
        .tool(TimeNow::new().utc_offset_minutes(utc_offset_minutes))
        .tool_arc(todo)
        .tool(ask)
        .tool(Shell::new(workspace))
        .tool(WebFetch::new())
        .tool(learning::Recall::new(dirs.learning_db()))
        .middleware(discovery.acquisition_policy())
        .middleware(permission)
        // Before spilling, so it compares real outputs rather than previews that name a fresh file each time.
        .middleware(RepeatGuard::new())
        .middleware(SpillOutput::new(dirs.spill()))
        .middleware(AttachmentContext::new(attachments))
        .middleware(ContextTrim::new(budget.context_limit).reserve_output(budget.trim_headroom))
        .middleware(ContinueGuard::new())
        .max_turns(MAX_TURNS);
    let roots = vec![workspace.to_path_buf(), dirs.spill()];
    let mut readable = fs_tools(roots.clone()).map_err(|e| e.to_string())?;
    readable.retain(|tool| tool.spec().name != "read_file");
    readable.push(Arc::new(ReadFile::new(roots, AttachmentStore::new(dirs.chat.join("attachments"))).map_err(|e| e.to_string())?));
    let writable = fs_write_tools(vec![workspace.to_path_buf()]).map_err(|e| e.to_string())?;
    for tool in readable.into_iter().chain(writable).chain(skills.tools()).chain(memory.tools()).chain(discovery.tools()).chain(desktop_tools::tools_in_workspace(desktop.clone(), workspace)) {
        builder = builder.tool_arc(tool);
    }
    if let Some(Ok(mcp_client)) = &computer_use {
        for tool in agent_computer_use_tools::checked_tools(mcp_client, &desktop.computer_use_runtime_root()?) {
            builder = builder.tool_arc(tool);
        }
    }
    for tool in connector_tools { builder = builder.tool_arc(tool); }
    builder.build().map_err(|e| e.to_string())
}

fn connector_manager(app: &tauri::AppHandle) -> Result<Arc<agent_connector_tools::ConnectorManager>, String> {
    let state=app.state::<ChatState>();
    let mut slot=lock(&state.connectors);
    if let Some(manager)=slot.as_ref(){return Ok(manager.clone());}
    let config=ai_config::current(app)?;
    let default=ai_config::default_workspace(app)?;
    let manager=Arc::new(agent_connector_tools::ConnectorManager::new(Dirs::of(app)?.chat.join("connectors"),config.workspace_or(&default).to_path_buf())?);
    *slot=Some(manager.clone());
    Ok(manager)
}
pub fn shutdown_connectors(app:&tauri::AppHandle){
    if let Some(manager)=lock(&app.state::<ChatState>().connectors).as_ref(){manager.shutdown();}
}

fn build_agent(app: &tauri::AppHandle, config: &AiConfig, key_source: Option<KeySource>, workspace: &Path, attachments: Vec<Attachment>) -> Result<Agent, String> {
    let state = app.state::<ChatState>();
    let ask = {
        let (app, broker) = (app.clone(), state.broker.clone());
        AskUser::new(move |session, question, ctx| ask_question(&app, &broker, session, question, ctx))
    };
    let spawn_computer_use = || {
        // Development resources live in the source tree; installed apps use Tauri's resource root.
        #[cfg(debug_assertions)]
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/computer-use");
        #[cfg(not(debug_assertions))]
        let root = app.path().resource_dir()
            .map_err(|e| agent_kernel::HarnessError::Config(format!("无法定位内置电脑控制资源：{e}")))?
            .join("agent-tools/computer-use");
        agent_computer_use_tools::spawn(&root)
    };
    let agent = assemble_with_connectors(Parts {
        provider: provider_with(config, key_source)?,
        dirs: &Dirs::of(app)?,
        workspace,
        permission: state.permission.clone(),
        todo: state.todo.clone(),
        ask,
        desktop: Arc::new(TauriDesktop { app: app.clone() }),
        utc_offset_minutes: chrono::Local::now().offset().local_minus_utc() / 60,
        open_computer_use: config.open_computer_use,
        spawn_computer_use: &spawn_computer_use,
        budget: config.token_budget(),
    }, attachments, connector_manager(app).map(|m|m.tools()).unwrap_or_default())?;
    lock(&state.tool_tokens).insert(config.open_computer_use, agent_kernel::estimate_tool_tokens(&agent.tool_schemas()));
    Ok(agent)
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
        "groups": mutate(&app, |index| index.groups())?,
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
    let summary = mutate(&app, |index| index.get(&session_id).cloned())?;
    let Some(summary) = summary else {
        return Ok(Vec::new());
    };
    let perm_mode = if summary.permission_mode == "auto" { PermissionMode::Auto } else { PermissionMode::Ask };
    app.state::<ChatState>().permission.set_mode(perm_mode);
    emit(&app, "chat-permission-mode-changed", json!({ "sessionId": session_id, "mode": summary.permission_mode }));
    JsonlStore::new(Dirs::of(&app)?.sessions()).load(&session_id).map_err(|e| e.to_string())
}

/// An empty conversation. The chat page makes one when the first message is sent, so the page
/// already knows the id when the answer's events start to arrive.
#[tauri::command]
pub fn chat_new_session(app: tauri::AppHandle, permission_mode: Option<String>) -> Result<SessionSummary, String> {
    let mode = match permission_mode.as_deref() {
        None => "ask",
        Some(mode @ ("ask" | "auto")) => mode,
        Some(_) => return Err("不支持的权限模式".into()),
    };
    let (seconds, millis) = now();
    mutate(&app, |index| index.create(seconds, millis, mode))?
}

#[tauri::command]
pub fn chat_send(app: tauri::AppHandle, session_id: String, text: String, images: Option<Vec<String>>, retry: Option<bool>, attachments: Option<Vec<AttachmentInput>>) -> Result<SessionSummary, String> {
    if !sessions::valid_id(&session_id) {
        return Err("对话编号不对".into());
    }
    let retry = retry.unwrap_or(false);
    let attachments = attachments.unwrap_or_default();
    let text = text.trim().to_string();
    let images = images.unwrap_or_default();
    if images.len() + attachments.len() > MAX_IMAGES {
        return Err(format!("一次最多发送 {MAX_IMAGES} 个附件"));
    }
    let images = images.iter().map(|raw| clean_image(raw)).collect::<Result<Vec<_>, _>>()?;
    if retry && (!attachments.is_empty() || !images.is_empty()) { return Err("重试不能添加新附件".into()); }
    if !retry && text.is_empty() && images.is_empty() && attachments.is_empty() {
        return Err("请先输入内容".into());
    }
    if text.chars().count() > MAX_INPUT_CHARS {
        return Err(format!("一次最多发送 {MAX_INPUT_CHARS} 个字，更长的内容请放进文件再让小伴去读"));
    }
    let state = app.state::<ChatState>();
    if lock(&state.runs).contains_key(&session_id) {
        return Err("这个对话还在回答或正在压缩，请等它结束或先停止".into());
    }
    if retry {
        let history = JsonlStore::new(Dirs::of(&app)?.sessions()).load(&session_id).map_err(|e| e.to_string())?;
        if !history.last().is_some_and(|m| m.role == agent_kernel::Role::User) {
            return Err("这条消息已产生回复或执行工具，无法直接重放。请发送新的消息继续。".into());
        }
    }
    let config = ai_config::current(&app)?;
    crate::ai_catalogue::usable(&app, &config)?;
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
        // An image-only message has no text to name the conversation after.
        if !text.is_empty() {
            index.name_from_first_message(&summary.id, &text);
        }
        index.touch(&summary.id, seconds);
        Ok(index.get(&summary.id).cloned().unwrap_or(summary))
    })??;

    let perm_mode = if summary.permission_mode == "auto" { PermissionMode::Auto } else { PermissionMode::Ask };
    state.permission.set_mode(perm_mode);
    *lock(&state.workspace) = Some(workspace.clone());

    let history_dir = Dirs::of(&app)?.sessions();
    let attachment_store = AttachmentStore::new(Dirs::of(&app)?.chat.join("attachments"));
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut runs = lock(&state.runs);
        if runs.contains_key(&summary.id) { return Err("这个对话还在回答或正在压缩，请等它结束或先停止".into()); }
        runs.insert(summary.id.clone(), cancel.clone());
    }
    let mut inputs = attachments;
    for (index, image) in images.iter().enumerate() {
        let extension = if image.starts_with("data:image/jpeg;") { "jpg" } else if image.starts_with("data:image/gif;") { "gif" } else if image.starts_with("data:image/webp;") { "webp" } else { "png" };
        inputs.push(AttachmentInput { name: format!("图片-{}.{extension}", index + 1), data_url: image.clone() });
    }
    let saved = match attachment_store.import(&session_id, &inputs) {
        Ok(saved) => saved,
        Err(error) => { lock(&state.runs).remove(&session_id); return Err(error); }
    };
    let saved_for_thread = saved.clone();

    // Build the agent and run it entirely on a dedicated OS thread.
    // reqwest::blocking::Client internally manages its own current-thread runtime;
    // building or dropping it inside Tokio's async worker threads panics.
    let parts = user_parts(&text, &images);
    let (handle, id, parts_owned, config_owned, workspace_owned) = (app.clone(), summary.id.clone(), parts, config, workspace);
    let started = std::thread::Builder::new().name(format!("chat-{id}")).spawn(move || {
        let sender = handle.clone();
        let session = id.clone();
        // 小伴 AI 要先拿到有效的账号令牌，可能需要联网刷新，所以放在这条线程里做，不卡住界面。
        let prepared = ai_gateway::prepare(&handle, config_owned).and_then(|(config, source)| {
            build_agent(&handle, &config, source.clone(), &workspace_owned, saved_for_thread.clone()).map(|agent| (agent, config, source))
        });
        if cancel.load(Ordering::SeqCst) {
            attachment_store.rollback(&saved_for_thread);
            emit(&sender, "chat-event", json!({ "session": session, "event": { "type": "cancelled" } }));
            finish_run(&handle, &id);
            return;
        }
        let (agent, config_owned, key_source) = match prepared {
            Ok(ready) => ready,
            Err(error) => {
                attachment_store.rollback(&saved_for_thread);
                emit(&sender, "chat-event", json!({ "session": session, "event": { "type": "error", "message": error } }));
                finish_run(&handle, &id);
                return;
            }
        };
        crate::proactivity::observe_task_state(&handle, agent_proactivity::context::TaskState::Started);
        // Persist the kernel's full sequenced stream before emitting compatible UI events.
        // If the journal cannot be written, do not continue dispatching tools.
        let mut terminal_delivered = false;
        let answered = match Dirs::of(&handle) {
            Ok(dirs) => {
                let journal = JsonlRunLog::new(dirs.runs().join(&id));
                let options = RunOptions { max_tool_calls: Some(256), ..Default::default() };
                let mut observed_run_id: Option<String> = None;
                let mut observe = |mut record: agent_kernel::RunEvent| {
                    observed_run_id = Some(record.run_id.clone());
                    let terminal = matches!(&record.body, RunEventBody::Event { event:
                        agent_kernel::Event::Done | agent_kernel::Event::Cancelled | agent_kernel::Event::Error { .. }
                    });
                    if let RunEventBody::Event { event: agent_kernel::Event::Error { message } } = &mut record.body {
                        if ai_gateway::is_active(&config_owned) {
                            *message = ai_gateway::readable_error(message);
                        }
                    }
                    run_history::forward(&journal, record.clone(), &config_owned.api_key, |channel, payload| {
                        emit(&sender, channel, payload);
                    })?;
                    goals::observe(&handle, &record).map_err(agent_kernel::HarnessError::Store)?;
                    if terminal { terminal_delivered = true; }
                    Ok(())
                };
                let executed = if retry {
                    agent.retry_last_observed(&id, options, &mut observe, &cancel)
                } else {
                    agent.run_parts_observed(&id, parts_owned, options, &mut observe, &cancel)
                };
                // The trace is the source of truth. Do not turn a successful model reply into
                // a verified business success automatically.
                if let Some(run) = observed_run_id {
                    if let Ok(outcome) = task_outcome::inspect(&journal, &id, &run) {
                        let _ = task_outcome::save(&dirs.outcomes(), &outcome);
                        if let Ok(db) = LearningDb::open(&dirs.learning_db()) {
                            let _ = db.record_outcome(&outcome.run_id, &outcome.session_id,
                                &format!("{:?}", outcome.status).to_lowercase(),
                                &outcome.validator, &outcome.evidence);
                        }
                        crate::proactivity::observe_task_outcome(&sender, &outcome);
                        emit(&sender, "chat-task-outcome", json!({ "session": id, "outcome": outcome }));
                    }
                }
                executed
            }
            Err(error) => Err(agent_kernel::HarnessError::Store(error)),
        };
        // An observer can fail before the kernel is able to publish its terminal event.
        if !terminal_delivered {
            if let Err(error) = &answered {
                let mut message = error.to_string();
                if !config_owned.api_key.is_empty() {
                    message = message.replace(&config_owned.api_key, "<hidden>");
                }
                if ai_gateway::is_active(&config_owned) {
                    message = ai_gateway::readable_error(&message);
                }
                emit(&sender, "chat-event", json!({ "session": session, "event": { "type": "error", "message": message } }));
                emit(&sender, "chat-run-state", json!({ "session": session, "state": "failed", "reason": message }));
            }
        }
        if answered.is_ok() {
            auto_compact(&handle, &id, &config_owned, key_source, &cancel);
        }
        // Keep files if their user message was saved, including failed model calls that can be retried.
        if let Ok(messages) = JsonlStore::new(history_dir).load(&id) {
            let persisted = messages.iter().any(|m| m.structured_content.as_ref()
                .and_then(|v| v.get("attachments")).and_then(Value::as_array)
                .is_some_and(|items| items.iter().any(|item| saved_for_thread.iter().any(|a| item["id"].as_str() == Some(a.id.as_str())))));
            if !persisted { attachment_store.rollback(&saved_for_thread); }
        }
        finish_run(&handle, &id);
    });
    if let Err(error) = started {
        AttachmentStore::new(Dirs::of(&app)?.chat.join("attachments")).rollback(&saved);
        lock(&state.runs).remove(&summary.id);
        return Err(format!("无法开始回答：{error}"));
    }
    Ok(summary)
}

/// Returns the persisted run trace for a console view, 50 events at a time.
/// Kernel completion is not proof that the user's task was successful.
#[tauri::command]
pub fn chat_run_history(
    app: tauri::AppHandle, window: tauri::WebviewWindow,
    session_id: String, run_id: String, after: Option<u64>,
) -> Result<Value, String> {
    if window.label() != "control" {
        return Err("运行轨迹仅可从控制台读取".into());
    }
    checked_session_id(&session_id)?;
    let log = JsonlRunLog::new(Dirs::of(&app)?.runs().join(&session_id));
    let inspection = log.inspect(&run_id).map_err(|e| e.to_string())?;
    let events = log.read_after(&run_id, after.unwrap_or(0), 50).map_err(|e| e.to_string())?;
    if events.iter().any(|event| event.session_id != session_id) {
        return Err("运行轨迹与会话不匹配".into());
    }
    let outcome = task_outcome::load(&Dirs::of(&app)?.outcomes(), &session_id, &run_id)?;
    Ok(json!({ "inspection": inspection, "events": events, "outcome": outcome }))
}

/// Verify one specifically requested file artifact. This does NOT approve an entire
/// multi-step task or let the model self-mark its result as successful.
#[tauri::command]
pub fn chat_verify_file_outcome(app: tauri::AppHandle, window: tauri::WebviewWindow,
    session_id: String, run_id: String, relative_path: String, expected_sha256: String
) -> Result<Value, String> {
    if window.label() != "control" { return Err("仅控制台可以验收任务".into()); }
    checked_session_id(&session_id)?;
    let dirs = Dirs::of(&app)?;
    let journal = JsonlRunLog::new(dirs.runs().join(&session_id));
    let previous = task_outcome::inspect(&journal, &session_id, &run_id)?;
    if !matches!(journal.inspect(&run_id).map_err(|e|e.to_string())?.state, Some(agent_kernel::RunState::Completed)) {
        return Err("只有正常结束的运行才允许文件验收".into());
    }
    if matches!(previous.status, task_outcome::Status::Partial | task_outcome::Status::Failed) {
        return Err("工具执行异常，请先调查运行记录".into());
    }
    let config = ai_config::current(&app)?;
    let default = ai_config::default_workspace(&app)?;
    let workspace = config.workspace_or(&default);
    let outcome=task_outcome::verify_file(&dirs.outcomes(), &session_id, &run_id, workspace, &relative_path, &expected_sha256)?;
    LearningDb::open(&dirs.learning_db())?.record_outcome(&run_id, &session_id,
        "succeeded", &outcome.validator, &outcome.evidence)?;
    crate::proactivity::observe_task_outcome(&app, &outcome);
    Ok(json!(outcome))
}

/// Explicit opt-in: run an isolated one-shot reflection model call. No auto approval.
#[tauri::command]
pub async fn chat_reflect_run(
    app: tauri::AppHandle, window: tauri::WebviewWindow,
    session_id: String, run_id: String
) -> Result<Value, String> {
    if window.label() != "control" { return Err("只有控制台可以启动学习".into()); }
    checked_session_id(&session_id)?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new().name("xiaoban-reflection".into()).spawn(move || {
        let result = learning::reflect_run(&app, &session_id, &run_id);
        let _ = tx.send(result);
    }).map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || rx.recv().unwrap_or_else(|_| Err("学习线程意外退出".into())))
        .await.map_err(|e|e.to_string())?
}

/// Only user control-plane review can promote a candidate into reusable memory.
#[tauri::command]
pub fn chat_review_experience(app: tauri::AppHandle, window: tauri::WebviewWindow,
    experience_id: i64, approve: bool,
) -> Result<Value,String> {
    if window.label() != "control" {return Err("只有控制台可以审核经验".into());}
    let db=LearningDb::open(&Dirs::of(&app)?.learning_db())?;
    let record=db.get_experience(experience_id)?.ok_or("未找到经验")?;
    if record.state!="candidate" {return Err("只有候选经验可以审核".into());}
    db.set_experience_state(experience_id,if approve{"approved"}else{"rejected"})?;
    Ok(json!(db.get_experience(experience_id)?))
}

/// Explicit proposal generation; reviewing and evaluating the proposal are separate steps.
#[tauri::command]
pub async fn chat_generate_skill_proposal(
    app: tauri::AppHandle, window: tauri::WebviewWindow,
    experience_id: i64, skill_name: String
) -> Result<Value,String> {
    if window.label() != "control" {return Err("只有控制台可以生成技能改进".into());}
    let (tx,rx)=std::sync::mpsc::channel();
    std::thread::Builder::new().name("xiaoban-skill-proposal".into()).spawn(move||{
        let _=tx.send(skill_evolution::generate(&app,experience_id,&skill_name));
    }).map_err(|e|e.to_string())?;
    tauri::async_runtime::spawn_blocking(move||rx.recv().unwrap_or_else(|_|Err("提议生成失败".into())))
       .await.map_err(|e|e.to_string())?
}

/// Records one external artifact-based eval; the model is not allowed to score itself.
#[tauri::command]
pub fn chat_record_skill_eval(app:tauri::AppHandle,window:tauri::WebviewWindow,
    proposal_id:i64, case_id:String, description:String,
    expected_sha256:String, artifact_path:String, baseline:bool,
) -> Result<Value,String> {
    if window.label()!="control"{return Err("只有控制台可以提交评测".into());}
    skill_evolution::record_eval(&app,proposal_id,&case_id,&description,
        &expected_sha256,&artifact_path,baseline)
}

#[tauri::command]
pub fn chat_apply_skill_proposal(app:tauri::AppHandle,window:tauri::WebviewWindow,
   proposal_id:i64,
) -> Result<Value,String>{
    if window.label()!="control"{return Err("只有控制台可以批准技能更新".into());}
    skill_evolution::apply(&app,proposal_id)
}

#[tauri::command]
pub fn chat_rollback_skill_proposal(app:tauri::AppHandle,window:tauri::WebviewWindow,
    proposal_id:i64,
) -> Result<Value,String>{
    if window.label()!="control"{return Err("只有控制台可以恢复技能版本".into());}
    skill_evolution::rollback(&app,proposal_id)
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
    AttachmentStore::new(Dirs::of(&app)?.chat.join("attachments")).remove_session(&session_id)?;
    let run_dir = Dirs::of(&app)?.runs().join(&session_id);
    if let Err(error) = fs::remove_dir_all(&run_dir) {
        if error.kind() != std::io::ErrorKind::NotFound {
            return Err(format!("无法删除运行记录：{error}"));
        }
    }
    let _ = fs::remove_dir_all(Dirs::of(&app)?.outcomes().join(&session_id));
    state.permission.forget_session(&session_id);
    state.todo.clear_session(&session_id);
    state.auto_compact.forget(&session_id);
    mutate(&app, |index| {
        index.remove(&session_id);
        index.list()
    })
}

/// What the agent sends before any conversation: its instructions, the skill list and the note index.
fn base_tokens(dirs: &Dirs, workspace: &Path, config: &AiConfig) -> usize {
    let environment = prompt::Environment { os: std::env::consts::OS, workspace, open_computer_use: config.open_computer_use, open_computer_use_error: None };
    let skills = Arc::new(Skills::new().read_only_root(dirs.chat.join("bundled-skills")).writable_root(dirs.skills()));
    let memory = Arc::new(Memory::new(dirs.memory()));
    let (skills_section, memory_section) = (skills.prompt_section(), memory.prompt_section());
    let text = [prompt::system_prompt(&environment), skills_section(), memory_section()]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    agent_kernel::estimate_tokens(&[Message::system(text)]) + config.token_budget().trim_headroom
}

fn context_of(app: &tauri::AppHandle, config: &AiConfig, raw: &[Message]) -> Result<ContextInfo, String> {
    let dirs = Dirs::of(app)?;
    let default = ai_config::default_workspace(app)?;
    // Until tools have been registered this launch, include a conservative allowance rather
    // than displaying zero schema cost. Registration replaces it with the actual schemas.
    let tool_tokens = lock(&app.state::<ChatState>().tool_tokens).get(&config.open_computer_use).copied().unwrap_or(32_000);
    Ok(compact::measure(raw, base_tokens(&dirs, config.workspace_or(&default), config) + tool_tokens, config.token_budget().context_limit))
}

/// How big the next request in a conversation would be, against the limit chosen in settings.
/// `None` is a conversation that has not been started yet.
#[tauri::command]
pub fn chat_context(app: tauri::AppHandle, session_id: Option<String>) -> Result<ContextInfo, String> {
    let config = ai_config::current(&app)?;
    let raw = match session_id.as_deref() {
        Some(id) => {
            checked_session_id(id)?;
            JsonlStore::new(Dirs::of(&app)?.sessions()).load(id).map_err(|e| e.to_string())?
        }
        None => Vec::new(),
    };
    context_of(&app, &config, &raw)
}

/// Has the model summarize the older part of a conversation and saves the summary. Returns its text.
fn write_summary(app: &tauri::AppHandle, config: &AiConfig, key_source: Option<KeySource>, id: &str, cancel: &AtomicBool) -> Result<String, String> {
    let file = JsonlStore::new(Dirs::of(app)?.sessions());
    let provider = provider_with(config, key_source)?;
    compact::compact_into(&file, provider.as_ref(), config.token_budget().context_limit, id, cancel)
        .map_err(|error| if ai_gateway::is_active(config) { ai_gateway::readable_error(&error) } else { error })
}

fn compact_session(app: &tauri::AppHandle, id: &str, cancel: &AtomicBool) -> Result<ContextInfo, String> {
    let config = ai_config::current(app)?;
    crate::ai_catalogue::usable(app, &config)?;
    if !config.is_complete() {
        return Err("还没有配置模型，请先到“设置 → AI 模型”填写".into());
    }
    let (config, key_source) = ai_gateway::prepare(app, config)?;
    write_summary(app, &config, key_source, id, cancel)?;
    let raw = JsonlStore::new(Dirs::of(app)?.sessions()).load(id).map_err(|e| e.to_string())?;
    context_of(app, &config, &raw)
}

/// After an answer went through: once the next request would use 90% of the context limit, the older part is
/// replaced by a summary, so a long conversation does not lose its beginning. Runs on the answer's thread, so the
/// conversation stays busy meanwhile. A failure is reported once and not tried again for a while; `ContextTrim`
/// still keeps every request under the limit.
fn auto_compact(app: &tauri::AppHandle, id: &str, config: &AiConfig, key_source: Option<KeySource>, cancel: &AtomicBool) {
    let Ok(dirs) = Dirs::of(app) else { return };
    let Ok(raw) = JsonlStore::new(dirs.sessions()).load(id) else { return };
    let Ok(info) = context_of(app, config, &raw) else { return };
    let state = app.state::<ChatState>();
    if !compact::due(&info) || !state.auto_compact.allowed(id, std::time::Instant::now()) {
        return;
    }
    emit(app, "chat-compacting", json!({ "session": id, "state": "start" }));
    match write_summary(app, config, key_source, id, cancel) {
        Ok(summary) => {
            state.auto_compact.forget(id);
            emit(app, "chat-compacting", json!({ "session": id, "state": "done", "summary": summary }));
        }
        Err(message) => {
            state.auto_compact.failed(id, std::time::Instant::now());
            emit(app, "chat-compacting", json!({ "session": id, "state": "failed", "message": message }));
        }
    }
}

/// Replaces the older part of a conversation, for the model only, with a summary the model writes.
/// The saved conversation and what the user sees are untouched. While it runs the conversation counts as busy.
#[tauri::command]
pub async fn chat_compact(app: tauri::AppHandle, session_id: String) -> Result<ContextInfo, String> {
    checked_session_id(&session_id)?;
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let state = app.state::<ChatState>();
        let mut runs = lock(&state.runs);
        if runs.contains_key(&session_id) {
            return Err("这个对话还在回答或正在压缩，请等它结束再试".into());
        }
        runs.insert(session_id.clone(), cancel.clone());
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    let (handle, id) = (app.clone(), session_id.clone());
    // The model client blocks, so it runs on its own thread, as a chat answer does.
    let started = std::thread::Builder::new().name(format!("compact-{id}")).spawn(move || {
        let result = compact_session(&handle, &id, &cancel);
        lock(&handle.state::<ChatState>().runs).remove(&id);
        let _ = sender.send(result);
    });
    if let Err(error) = started {
        lock(&app.state::<ChatState>().runs).remove(&session_id);
        return Err(format!("无法开始压缩：{error}"));
    }
    tauri::async_runtime::spawn_blocking(move || receiver.recv().unwrap_or_else(|_| Err("压缩被中断了".to_string())))
        .await
        .map_err(|e| e.to_string())?
}

fn checked_session_id(id: &str) -> Result<(), String> {
    if sessions::valid_id(id) { Ok(()) } else { Err("对话编号不对".into()) }
}

fn checked_group_id(id: &str) -> Result<(), String> {
    if sessions::valid_id(id) { Ok(()) } else { Err("分组编号不对".into()) }
}

/// Renaming works while the conversation is answering: the title is only a label.
#[tauri::command]
pub fn chat_rename_session(app: tauri::AppHandle, session_id: String, title: String) -> Result<Library, String> {
    checked_session_id(&session_id)?;
    mutate(&app, |index| index.rename(&session_id, &title).map(|()| index.library()))?
}

/// An empty or missing `group_id` takes the conversation out of its group.
#[tauri::command]
pub fn chat_move_session(app: tauri::AppHandle, session_id: String, group_id: Option<String>) -> Result<Library, String> {
    checked_session_id(&session_id)?;
    if let Some(group) = group_id.as_deref().filter(|g| !g.is_empty()) {
        checked_group_id(group)?;
    }
    mutate(&app, |index| index.move_session(&session_id, group_id.as_deref()).map(|()| index.library()))?
}

/// With `session_id`, the conversation moves into the new group in the same step, so a failed name leaves it where it was.
#[tauri::command]
pub fn chat_create_group(app: tauri::AppHandle, name: String, session_id: Option<String>) -> Result<Library, String> {
    if let Some(id) = session_id.as_deref() {
        checked_session_id(id)?;
    }
    let (_, millis) = now();
    mutate(&app, |index| -> Result<Library, String> {
        if let Some(id) = session_id.as_deref() {
            index.get(id).ok_or("这个对话已经不存在")?;
        }
        let group = index.create_group(millis, &name)?;
        if let Some(id) = session_id.as_deref() {
            index.move_session(id, Some(&group.id))?;
        }
        Ok(index.library())
    })?
}

#[tauri::command]
pub fn chat_rename_group(app: tauri::AppHandle, group_id: String, name: String) -> Result<Library, String> {
    checked_group_id(&group_id)?;
    mutate(&app, |index| index.rename_group(&group_id, &name).map(|()| index.library()))?
}

/// The conversations inside are kept and go back to the top level.
#[tauri::command]
pub fn chat_delete_group(app: tauri::AppHandle, group_id: String) -> Result<Library, String> {
    checked_group_id(&group_id)?;
    mutate(&app, |index| index.delete_group(&group_id).map(|()| index.library()))?
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
pub fn chat_set_permission_mode(app: tauri::AppHandle, session_id: Option<String>, mode: String) -> Result<String, String> {
    if !["ask", "auto"].contains(&mode.as_str()) {
        return Err("不支持的权限模式".into());
    }
    let perm_mode = if mode == "auto" { PermissionMode::Auto } else { PermissionMode::Ask };
    let state = app.state::<ChatState>();
    state.permission.set_mode(perm_mode);

    if let Some(ref sid) = session_id {
        let _ = mutate(&app, |index| {
            index.set_permission_mode(sid, &mode);
        });
    }

    emit(&app, "chat-permission-mode-changed", json!({ "sessionId": session_id, "mode": mode }));
    Ok(mode)
}

#[cfg(test)]
mod tests;

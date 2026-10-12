//! The model endpoint the assistant talks to: protocol, address, model name and an optional key.
//!
//! The key has its own file and is never part of `Settings`: settings are broadcast to every window
//! and shown in diagnostics, while this file is read by the host only. The web side can read back
//! no more than whether a key exists and its last four characters.

use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{Emitter, Manager};

pub const PROTOCOLS: &[&str] = &["chat", "responses", "anthropic"];
const MAX_URL: usize = 2048;
const MAX_MODEL: usize = 200;
const MAX_KEY: usize = 2048;

pub const DEFAULT_PROVIDERS: &[(&str, &str, &str, &str, &str)] = &[
    ("deepseek", "DeepSeek", "chat", "https://api.deepseek.com", "deepseek-chat"),
    ("zhipu", "智谱 BigModel", "chat", "https://open.bigmodel.cn/api/paas/v4", "glm-4-flash"),
    ("kimi", "Kimi", "chat", "https://api.moonshot.cn/v1", "moonshot-v1-auto"),
    ("minimax", "MiniMax", "chat", "https://api.minimax.chat/v1", "MiniMax-Text-01"),
];

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            id: "custom".into(),
            name: "自定义供应商".into(),
            protocol: "chat".into(),
            base_url: String::new(),
            model: String::new(),
            api_key: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigView {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    pub has_key: bool,
    pub key_hint: String,
    pub is_custom: bool,
}

fn make_key_hint(key: &str) -> String {
    match key.chars().count() {
        0 => String::new(),
        n if n >= 12 => format!("••••{}", key.chars().skip(n - 4).collect::<String>()),
        _ => "••••".into(),
    }
}

impl ProviderConfig {
    pub fn is_custom(&self) -> bool {
        !["deepseek", "zhipu", "kimi", "minimax"].contains(&self.id.as_str())
    }

    pub fn view(&self) -> ProviderConfigView {
        ProviderConfigView {
            id: self.id.clone(),
            name: if self.name.is_empty() {
                match self.id.as_str() {
                    "deepseek" => "DeepSeek".into(),
                    "zhipu" => "智谱 BigModel".into(),
                    "kimi" => "Kimi".into(),
                    "minimax" => "MiniMax".into(),
                    _ => "自定义供应商".into(),
                }
            } else {
                self.name.clone()
            },
            protocol: self.protocol.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            has_key: !self.api_key.is_empty(),
            key_hint: make_key_hint(&self.api_key),
            is_custom: self.is_custom(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AiConfig {
    pub active_provider: String,
    pub providers: Vec<ProviderConfig>,
    /// `chat` (/chat/completions), `responses` (/responses) or `anthropic` (/v1/messages).
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    /// Empty for local servers that need no key.
    pub api_key: String,
    /// The folder the assistant's file and command tools work in. Empty means the built-in one.
    pub workspace: String,
    /// Permission approval mode: `ask` (default) asks before writes/exec, `auto` trusts everything.
    pub permission_mode: String,
    /// Whether to hide intermediate tool call cards in the conversation.
    pub hide_tool_calls: bool,
    /// Whether open-computer-use MCP tools are enabled.
    pub open_computer_use: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        let mut cfg = Self {
            active_provider: "deepseek".into(),
            providers: Vec::new(),
            protocol: "chat".into(),
            base_url: String::new(),
            model: String::new(),
            api_key: String::new(),
            workspace: String::new(),
            permission_mode: "ask".into(),
            hide_tool_calls: false,
            open_computer_use: false,
        };
        cfg.normalize();
        cfg
    }
}

impl AiConfig {
    /// Enough to send a request.
    pub fn is_complete(&self) -> bool {
        !self.base_url.trim().is_empty() && !self.model.trim().is_empty()
    }

    pub fn workspace_or<'a>(&'a self, default: &'a Path) -> &'a Path {
        if self.workspace.is_empty() { default } else { Path::new(&self.workspace) }
    }
}

impl fmt::Debug for AiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AiConfig")
            .field("active_provider", &self.active_provider)
            .field("protocol", &self.protocol)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("workspace", &self.workspace)
            .field("api_key", &if self.api_key.is_empty() { "<none>" } else { "<redacted>" })
            .finish()
    }
}

/// What the web side may see.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfigView {
    pub active_provider: String,
    pub providers: Vec<ProviderConfigView>,
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    pub has_key: bool,
    /// Dots, plus the last four characters when the key is long enough for that to reveal little.
    pub key_hint: String,
    /// Empty when the built-in folder is used.
    pub workspace: String,
    pub default_workspace: String,
    /// `ask` or `auto`.
    pub permission_mode: String,
    pub hide_tool_calls: bool,
    pub open_computer_use: bool,
}

impl AiConfig {
    pub fn view(&self, default_workspace: &Path) -> AiConfigView {
        let providers_view = self.providers.iter().map(|p| p.view()).collect();
        AiConfigView {
            active_provider: self.active_provider.clone(),
            providers: providers_view,
            protocol: self.protocol.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            has_key: !self.api_key.is_empty(),
            key_hint: make_key_hint(&self.api_key),
            workspace: self.workspace.clone(),
            default_workspace: default_workspace.display().to_string(),
            permission_mode: self.permission_mode.clone(),
            hide_tool_calls: self.hide_tool_calls,
            open_computer_use: self.open_computer_use,
        }
    }

    fn normalize(&mut self) {
        if !PROTOCOLS.contains(&self.protocol.as_str()) {
            self.protocol = "chat".into();
        }
        if !["ask", "auto"].contains(&self.permission_mode.as_str()) {
            self.permission_mode = "ask".into();
        }

        for &(id, name, default_proto, default_url, default_model) in DEFAULT_PROVIDERS {
            if !self.providers.iter().any(|p| p.id == id) {
                self.providers.push(ProviderConfig {
                    id: id.into(),
                    name: name.into(),
                    protocol: default_proto.into(),
                    base_url: default_url.into(),
                    model: default_model.into(),
                    api_key: String::new(),
                });
            }
        }

        if self.active_provider.is_empty() {
            self.active_provider = if self.base_url.contains("deepseek.com") {
                "deepseek".into()
            } else if self.base_url.contains("bigmodel.cn") {
                "zhipu".into()
            } else if self.base_url.contains("moonshot.cn") {
                "kimi".into()
            } else if self.base_url.contains("minimax") {
                "minimax".into()
            } else if !self.base_url.is_empty() {
                "custom".into()
            } else {
                "deepseek".into()
            };
        }

        if let Some(active) = self.providers.iter_mut().find(|p| p.id == self.active_provider) {
            if !self.base_url.is_empty() && active.base_url.is_empty() {
                active.base_url = self.base_url.clone();
            }
            if !self.model.is_empty() && active.model.is_empty() {
                active.model = self.model.clone();
            }
            if !self.api_key.is_empty() && active.api_key.is_empty() {
                active.api_key = self.api_key.clone();
            }
            if !self.protocol.is_empty() {
                active.protocol = self.protocol.clone();
            }
        }
    }
}

pub struct Input<'a> {
    pub protocol: &'a str,
    pub base_url: &'a str,
    pub model: &'a str,
    /// `None` keeps the saved key.
    pub api_key: Option<&'a str>,
    pub clear_key: bool,
    /// `None` keeps the saved folder, `Some("")` goes back to the built-in one.
    pub workspace: Option<&'a str>,
    pub permission_mode: Option<&'a str>,
    pub hide_tool_calls: Option<bool>,
    pub open_computer_use: Option<bool>,
}

fn clean_base_url(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("请填写 API 地址".into());
    }
    if raw.len() > MAX_URL {
        return Err("API 地址太长".into());
    }
    let url = url::Url::parse(raw).map_err(|_| "API 地址格式不对，应类似 https://api.example.com/v1".to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("API 地址必须以 http:// 或 https:// 开头".into());
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err("API 地址缺少主机名".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("API 地址里不要带用户名或密码，密钥请填在下面的密钥栏".into());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("API 地址里不要带 ? 或 # 参数，只填到接口根路径".into());
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

fn clean_model(raw: &str) -> Result<String, String> {
    let model = raw.trim();
    if model.is_empty() {
        return Err("请填写模型名".into());
    }
    if model.chars().count() > MAX_MODEL || model.chars().any(char::is_control) {
        return Err("模型名不对，请按服务商给出的名字填写".into());
    }
    Ok(model.to_string())
}

fn clean_key(raw: &str) -> Result<String, String> {
    let key = raw.trim();
    if key.is_empty() {
        return Err("密钥不能为空。不想修改请留空不填，要删除请用“删除密钥”".into());
    }
    if key.chars().count() > MAX_KEY || key.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("密钥里不能有空格或换行，请重新复制粘贴".into());
    }
    Ok(key.to_string())
}

fn clean_workspace(raw: &str) -> Result<String, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Ok(String::new());
    }
    if text.len() > MAX_URL || text.chars().any(char::is_control) {
        return Err("工作目录路径不对".into());
    }
    let path = Path::new(text);
    if !path.is_absolute() {
        return Err("工作目录必须是完整路径，请用“选择文件夹”".into());
    }
    if !path.is_dir() {
        return Err("这个文件夹不存在，或者不是文件夹".into());
    }
    Ok(text.to_string())
}

/// Validates the form against what is saved and returns the new configuration.
pub fn apply(current: &AiConfig, input: &Input<'_>) -> Result<AiConfig, String> {
    if !PROTOCOLS.contains(&input.protocol) {
        return Err("不支持这种接口协议".into());
    }
    if input.clear_key && input.api_key.is_some() {
        return Err("不能同时设置和删除密钥".into());
    }
    let api_key = match (input.api_key, input.clear_key) {
        (Some(new), _) => clean_key(new)?,
        (None, true) => String::new(),
        (None, false) => current.api_key.clone(),
    };
    let base_url = clean_base_url(input.base_url)?;
    let model = clean_model(input.model)?;
    let protocol = input.protocol.to_string();

    let mut next = current.clone();
    next.protocol = protocol.clone();
    next.base_url = base_url.clone();
    next.model = model.clone();
    next.api_key = api_key.clone();

    if let Some(raw) = input.workspace {
        next.workspace = clean_workspace(raw)?;
    }
    if let Some(mode) = input.permission_mode {
        if !["ask", "auto"].contains(&mode) {
            return Err("不支持的权限模式".into());
        }
        next.permission_mode = mode.to_string();
    }
    if let Some(htc) = input.hide_tool_calls {
        next.hide_tool_calls = htc;
    }
    if let Some(ocu) = input.open_computer_use {
        next.open_computer_use = ocu;
    }

    if let Some(p) = next.providers.iter_mut().find(|p| p.id == next.active_provider) {
        p.protocol = protocol;
        p.base_url = base_url;
        p.model = model;
        p.api_key = api_key;
    }
    next.normalize();
    Ok(next)
}

pub fn load(path: &Path) -> io::Result<AiConfig> {
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(AiConfig::default()),
        Err(error) => return Err(error),
    };
    let mut value: AiConfig = serde_json::from_slice(&data).map_err(io::Error::other)?;
    value.normalize();
    Ok(value)
}

/// Written through a private temporary file and renamed, so a crash never leaves half a key behind.
/// On Unix the file is readable by its owner only; elsewhere it inherits the folder's permissions.
pub fn save(path: &Path, value: &AiConfig) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::other("配置文件没有父目录"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!("ai-{}.tmp", std::process::id()));
    let _ = fs::remove_file(&temporary);
    let result = (|| {
        let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[derive(Default)]
pub struct AiConfigState {
    io: Mutex<()>,
}

pub(crate) fn path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config/ai.json"))
}

/// Where the assistant works when the user has not picked a folder.
pub fn default_workspace(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("workspace"))
}

/// What is saved, for the host's own use (the chat). Not for the web side: it holds the key.
pub fn current(app: &tauri::AppHandle) -> Result<AiConfig, String> {
    let path = path(app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    load(&path).map_err(|e| format!("模型配置读取失败：{e}"))
}

pub fn update_permission_mode(app: &tauri::AppHandle, mode: &str) -> Result<(), String> {
    let path = path(app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    let mut config = load(&path).unwrap_or_default();
    config.permission_mode = mode.to_string();
    config.normalize();
    save(&path, &config).map_err(|e| format!("无法保存权限设置：{e}"))
}

#[tauri::command]
pub fn get_ai_config(app: tauri::AppHandle) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    let default = default_workspace(&app)?;
    load(&path).map(|config| config.view(&default)).map_err(|e| format!("模型配置读取失败：{e}"))
}

/// Opens the system folder picker for the workspace. Returns the choice without saving it.
#[tauri::command]
pub async fn pick_workspace(window: tauri::WebviewWindow) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let chosen = app.dialog().file().set_parent(&window).set_title("选择小伴的工作目录").blocking_pick_folder();
        let Some(chosen) = chosen else { return Ok(None) };
        let path = chosen.into_path().map_err(|e| e.to_string())?;
        Ok(Some(path.display().to_string()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn save_ai_config(
    app: tauri::AppHandle,
    protocol: String,
    base_url: String,
    model: String,
    api_key: Option<String>,
    clear_key: bool,
    workspace: Option<String>,
    permission_mode: Option<String>,
    hide_tool_calls: Option<bool>,
    open_computer_use: Option<bool>,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    // A damaged file must not stop the user from writing a new one over it.
    let current = load(&path).unwrap_or_default();
    let next = apply(&current, &Input { protocol: &protocol, base_url: &base_url, model: &model, api_key: api_key.as_deref(), clear_key, workspace: workspace.as_deref(), permission_mode: permission_mode.as_deref(), hide_tool_calls, open_computer_use })?;
    save(&path, &next).map_err(|e| format!("无法保存模型配置：{e}"))?;
    let _ = app.emit_to("control", "chat-hide-tool-calls-changed", serde_json::json!({ "enabled": next.hide_tool_calls }));
    Ok(next.view(&default_workspace(&app)?))
}

#[tauri::command]
pub fn save_ai_provider(
    app: tauri::AppHandle,
    provider_id: String,
    name: Option<String>,
    protocol: String,
    base_url: String,
    model: String,
    api_key: Option<String>,
    clear_key: bool,
    activate: bool,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    let clean_url = clean_base_url(&base_url)?;
    let clean_mdl = clean_model(&model)?;
    if !PROTOCOLS.contains(&protocol.as_str()) {
        return Err("不支持这种接口协议".into());
    }

    let existing_idx = config.providers.iter().position(|p| p.id == provider_id);
    let (current_name, current_key) = match existing_idx {
        Some(i) => (config.providers[i].name.clone(), config.providers[i].api_key.clone()),
        None => (String::new(), String::new()),
    };

    let new_name = match name {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => if current_name.is_empty() { "自定义供应商".into() } else { current_name },
    };

    let new_key = match (api_key.as_deref(), clear_key) {
        (Some(new), _) if !new.trim().is_empty() => clean_key(new)?,
        (None, true) => String::new(),
        _ => current_key,
    };

    let updated_entry = ProviderConfig {
        id: provider_id.clone(),
        name: new_name,
        protocol: protocol.clone(),
        base_url: clean_url.clone(),
        model: clean_mdl.clone(),
        api_key: new_key.clone(),
    };

    if let Some(idx) = existing_idx {
        config.providers[idx] = updated_entry;
    } else {
        config.providers.push(updated_entry);
    }

    if activate || config.active_provider == provider_id || config.base_url.is_empty() {
        config.active_provider = provider_id;
        config.protocol = protocol;
        config.base_url = clean_url;
        config.model = clean_mdl;
        config.api_key = new_key;
    }

    save(&path, &config).map_err(|e| format!("无法保存服务商配置：{e}"))?;
    Ok(config.view(&default_workspace(&app)?))
}

#[tauri::command]
pub fn delete_ai_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    if ["deepseek", "zhipu", "kimi", "minimax"].contains(&provider_id.as_str()) {
        return Err("内置服务商不能删除".into());
    }
    if config.active_provider == provider_id {
        return Err("当前正在使用该服务商，请先切换为其他服务商后再删除".into());
    }

    let initial_len = config.providers.len();
    config.providers.retain(|p| p.id != provider_id);
    if config.providers.len() == initial_len {
        return Err("未找到该服务商".into());
    }

    save(&path, &config).map_err(|e| format!("无法删除服务商：{e}"))?;
    Ok(config.view(&default_workspace(&app)?))
}

#[tauri::command]
pub fn switch_ai_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    let provider = config.providers.iter().find(|p| p.id == provider_id)
        .ok_or_else(|| format!("未找到服务商配置：{provider_id}"))?
        .clone();

    if provider.base_url.trim().is_empty() {
        return Err("该服务商尚未配置 API 地址，请先点击配置".into());
    }
    if provider.model.trim().is_empty() {
        return Err("该服务商尚未填写模型名，请先点击配置".into());
    }

    config.active_provider = provider_id;
    config.protocol = provider.protocol;
    config.base_url = provider.base_url;
    config.model = provider.model;
    config.api_key = provider.api_key;

    save(&path, &config).map_err(|e| format!("无法切换服务商：{e}"))?;
    Ok(config.view(&default_workspace(&app)?))
}

#[tauri::command]
pub fn save_ai_preferences(
    app: tauri::AppHandle,
    workspace: Option<String>,
    permission_mode: Option<String>,
    hide_tool_calls: Option<bool>,
    open_computer_use: Option<bool>,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap();
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    if let Some(raw) = workspace.as_deref() {
        config.workspace = clean_workspace(raw)?;
    }
    if let Some(mode) = permission_mode.as_deref() {
        if !["ask", "auto"].contains(&mode) {
            return Err("不支持的权限模式".into());
        }
        config.permission_mode = mode.to_string();
    }
    if let Some(htc) = hide_tool_calls {
        config.hide_tool_calls = htc;
    }
    if let Some(ocu) = open_computer_use {
        config.open_computer_use = ocu;
    }

    save(&path, &config).map_err(|e| format!("无法保存偏好设置：{e}"))?;
    let _ = app.emit_to("control", "chat-hide-tool-calls-changed", serde_json::json!({ "enabled": config.hide_tool_calls }));
    Ok(config.view(&default_workspace(&app)?))
}

pub fn candidate_models_urls(protocol: &str, base_url: &str) -> Vec<String> {
    let base = base_url.trim_end_matches('/');
    if protocol == "anthropic" {
        if base.ends_with("/v1") {
            vec![format!("{base}/models")]
        } else {
            vec![format!("{base}/v1/models"), format!("{base}/models")]
        }
    } else if base.ends_with("/v1") {
        vec![format!("{base}/models")]
    } else {
        vec![format!("{base}/models"), format!("{base}/v1/models")]
    }
}

pub fn parse_model_list_json(raw: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| format!("解析服务商返回的 JSON 失败：{e}"))?;
    let mut models = Vec::new();

    if let Some(data) = value.get("data").and_then(|v| v.as_array()) {
        for item in data {
            if let Some(id) = item.get("id").and_then(|id| id.as_str()) {
                models.push(id.trim().to_string());
            } else if let Some(id) = item.as_str() {
                models.push(id.trim().to_string());
            }
        }
    } else if let Some(items) = value.get("models").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(id) = item.get("name").and_then(|n| n.as_str()).or_else(|| item.get("id").and_then(|i| i.as_str())) {
                models.push(id.trim().to_string());
            } else if let Some(id) = item.as_str() {
                models.push(id.trim().to_string());
            }
        }
    } else if let Some(items) = value.as_array() {
        for item in items {
            if let Some(id) = item.get("id").and_then(|i| i.as_str()).or_else(|| item.get("name").and_then(|n| n.as_str())) {
                models.push(id.trim().to_string());
            } else if let Some(id) = item.as_str() {
                models.push(id.trim().to_string());
            }
        }
    }

    models.retain(|m| !m.is_empty());
    models.sort();
    models.dedup();

    if models.is_empty() {
        return Err("服务商返回的数据中未包含任何模型".into());
    }
    Ok(models)
}

#[tauri::command]
pub async fn fetch_ai_models(
    app: tauri::AppHandle,
    protocol: String,
    base_url: String,
    api_key: Option<String>,
) -> Result<Vec<String>, String> {
    let base_url = clean_base_url(&base_url)?;
    let protocol = if PROTOCOLS.contains(&protocol.as_str()) { protocol } else { "chat".into() };

    let effective_key = match api_key {
        Some(key) if !key.trim().is_empty() => clean_key(&key)?,
        _ => {
            let path = path(&app)?;
            let state = app.state::<AiConfigState>();
            let _io = state.io.lock().unwrap();
            let cfg = load(&path).unwrap_or_default();
            if let Some(p) = cfg.providers.iter().find(|p| !p.api_key.is_empty() && (clean_base_url(&p.base_url).ok().as_deref() == Some(&base_url) || p.base_url.is_empty())) {
                p.api_key.clone()
            } else {
                cfg.api_key
            }
        }
    };

    tauri::async_runtime::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("无法创建网络请求客户端：{e}"))?;

        let urls = candidate_models_urls(&protocol, &base_url);
        let mut last_error = String::new();

        for url in urls {
            let mut req = client.get(&url).header("User-Agent", "XiaoBanDesktop/1.0");
            if protocol == "anthropic" {
                req = req.header("anthropic-version", "2023-06-01");
                if !effective_key.is_empty() {
                    req = req.header("x-api-key", &effective_key)
                        .header("Authorization", format!("Bearer {effective_key}"));
                }
            } else if !effective_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {effective_key}"));
            }

            let resp = match req.send() {
                Ok(r) => r,
                Err(e) => {
                    last_error = format!("网络请求失败：{e}");
                    continue;
                }
            };

            let status = resp.status();
            if status == reqwest::StatusCode::NOT_FOUND {
                last_error = format!("未找到模型列表接口（HTTP 404，地址：{url}）");
                continue;
            }
            if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
                return Err(format!("鉴权失败（HTTP {status}）：请检查 API 密钥是否正确"));
            }
            if !status.is_success() {
                let text = resp.text().unwrap_or_default();
                let summary = text.chars().take(200).collect::<String>();
                return Err(format!("服务商返回错误（HTTP {status}）：{summary}"));
            }

            let body = resp.text().map_err(|e| format!("读取响应失败：{e}"))?;
            return parse_model_list_json(&body);
        }

        if last_error.is_empty() {
            Err("未能获取到模型列表".into())
        } else {
            Err(last_error)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "sk-abcdefghijklmnop1234";

    fn saved() -> AiConfig {
        let mut cfg = AiConfig { protocol: "chat".into(), base_url: "https://api.example.com/v1".into(), model: "m1".into(), api_key: KEY.into(), ..AiConfig::default() };
        cfg.normalize();
        cfg
    }

    fn input<'a>(base_url: &'a str, model: &'a str) -> Input<'a> {
        Input { protocol: "chat", base_url, model, api_key: None, clear_key: false, workspace: None, permission_mode: None, hide_tool_calls: None, open_computer_use: None }
    }

    fn temp(tag: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("xiaoban-ai-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn addresses_are_trimmed_and_lose_trailing_slashes() {
        for (typed, stored) in [
            ("https://api.openai.com/v1", "https://api.openai.com/v1"),
            ("  https://api.openai.com/v1/  ", "https://api.openai.com/v1"),
            ("https://api.anthropic.com", "https://api.anthropic.com"),
            ("https://api.anthropic.com/", "https://api.anthropic.com"),
            ("http://127.0.0.1:11434/v1", "http://127.0.0.1:11434/v1"),
            ("https://gw.example.com:8443/proxy/anthropic//", "https://gw.example.com:8443/proxy/anthropic"),
            ("HTTPS://API.Example.COM/V1", "https://api.example.com/V1"),
        ] {
            assert_eq!(clean_base_url(typed).unwrap(), stored, "{typed}");
        }
    }

    #[test]
    fn addresses_that_would_leak_or_misroute_are_rejected() {
        let long = format!("https://example.com/{}", "a".repeat(2100));
        for bad in [
            "", "   ", "api.openai.com/v1", "ftp://example.com/v1", "file:///etc/passwd", "javascript:alert(1)", "https://",
            "https://user:pass@example.com/v1", "https://user@example.com/v1", "https://example.com/v1?key=abc", "https://example.com/v1#frag",
            "not a url", long.as_str(),
        ] {
            assert!(clean_base_url(bad).is_err(), "{bad:?} should be rejected");
        }
        assert!(clean_base_url("https://user:pass@example.com").unwrap_err().contains("密钥"));
    }

    #[test]
    fn model_names_follow_the_providers_spelling() {
        for good in ["gpt-4o", "anthropic/some-model:free", "qwen2.5:7b", "  spaced  ", "模型-1"] {
            assert!(clean_model(good).is_ok(), "{good}");
        }
        assert_eq!(clean_model("  spaced  ").unwrap(), "spaced");
        for bad in ["", "   ", "a\nb", "a\u{0}b", &"m".repeat(201)] {
            assert!(clean_model(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn keys_may_not_hide_whitespace_or_control_characters() {
        assert_eq!(clean_key("  sk-abc123  ").unwrap(), "sk-abc123");
        assert_eq!(clean_key("sk-abc123\r\n").unwrap(), "sk-abc123", "a newline copied along with the key is dropped");
        for bad in ["", "   ", "sk abc", "sk-abc\nX-Evil: 1", "sk-abc\r\nX-Evil: 1", "sk\tabc", "sk-\u{7}x", &"k".repeat(2049)] {
            assert!(clean_key(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn saving_keeps_replaces_or_clears_the_key() {
        let keep = apply(&saved(), &input("https://other.example.com/v1", "m2")).unwrap();
        assert_eq!((keep.api_key.as_str(), keep.base_url.as_str(), keep.model.as_str()), (KEY, "https://other.example.com/v1", "m2"));

        let replace = apply(&saved(), &Input { api_key: Some("  sk-new-key-value  "), ..input("https://api.example.com/v1", "m1") }).unwrap();
        assert_eq!(replace.api_key, "sk-new-key-value");

        let clear = apply(&saved(), &Input { clear_key: true, ..input("https://api.example.com/v1", "m1") }).unwrap();
        assert_eq!(clear.api_key, "");

        let first = apply(&AiConfig::default(), &Input { api_key: Some("local-key-1"), ..input("http://localhost:1234/v1", "m") }).unwrap();
        assert_eq!(first.api_key, "local-key-1");
        let keyless = apply(&AiConfig::default(), &input("http://localhost:1234/v1", "m")).unwrap();
        assert_eq!(keyless.api_key, "", "a local server needs no key");

        assert!(apply(&saved(), &Input { api_key: Some("new-key-0123456"), clear_key: true, ..input("https://a.example.com", "m") }).is_err());
        assert!(apply(&saved(), &Input { api_key: Some("   "), ..input("https://a.example.com", "m") }).is_err(), "a blank key never silently erases the saved one");
        assert!(apply(&saved(), &Input { protocol: "grpc", ..input("https://a.example.com", "m") }).is_err());
    }

    #[test]
    fn a_rejected_form_changes_nothing() {
        let current = saved();
        assert!(apply(&current, &input("nonsense", "m")).is_err());
        assert!(apply(&current, &input("https://a.example.com", "")).is_err());
        assert_eq!(current, saved());
    }

    #[test]
    fn the_web_side_never_receives_the_key() {
        let view = saved().view(Path::new("/d"));
        let json = serde_json::to_string(&view).unwrap();
        assert!(!json.contains(KEY) && !json.contains("sk-abc") && !json.to_lowercase().contains("apikey"), "{json}");
        assert_eq!((view.has_key, view.key_hint.as_str()), (true, "••••1234"));
        let short = AiConfig { api_key: "secret-1".into(), ..saved() }.view(Path::new("/d"));
        assert_eq!((short.has_key, short.key_hint.as_str()), (true, "••••"), "a short key reveals nothing");
        let none = AiConfig { api_key: String::new(), ..saved() }.view(Path::new("/d"));
        assert_eq!((none.has_key, none.key_hint.as_str()), (false, ""));
        let wide = AiConfig { api_key: "密钥".repeat(8), ..saved() }.view(Path::new("/d"));
        assert_eq!(wide.key_hint, "••••密钥密钥", "hints are cut on character boundaries");
    }

    #[test]
    fn the_workspace_is_kept_changed_or_reset_and_must_be_a_real_folder() {
        let folder = temp("workspace");
        fs::create_dir_all(&folder).unwrap();
        let file = folder.join("a-file.txt");
        fs::write(&file, "x").unwrap();
        let chosen = folder.display().to_string();
        let base = || input("https://a.example.com", "m");

        let set = apply(&saved(), &Input { workspace: Some(&format!("  {chosen}  ")), ..base() }).unwrap();
        assert_eq!(set.workspace, chosen, "stored without the surrounding spaces");
        let kept = apply(&set, &base()).unwrap();
        assert_eq!(kept.workspace, chosen, "not mentioned means unchanged");
        let reset = apply(&set, &Input { workspace: Some(""), ..base() }).unwrap();
        assert_eq!(reset.workspace, "", "an empty choice goes back to the built-in folder");

        for bad in ["relative/folder", "~/Documents", "./here", &file.display().to_string(), &folder.join("missing").display().to_string(), "bad\u{0}path"] {
            assert!(apply(&set, &Input { workspace: Some(bad), ..base() }).is_err(), "{bad:?}");
        }
        assert_eq!(set.workspace_or(Path::new("/default")), Path::new(&chosen));
        assert_eq!(reset.workspace_or(Path::new("/default")), Path::new("/default"));
        let view = set.view(Path::new("/default"));
        assert_eq!((view.workspace.as_str(), view.default_workspace.as_str()), (chosen.as_str(), "/default"));
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn a_configuration_needs_an_address_and_a_model_to_be_usable() {
        assert!(!AiConfig::default().is_complete());
        assert!(saved().is_complete());
        assert!(!AiConfig { model: "  ".into(), ..saved() }.is_complete());
        assert!(!AiConfig { base_url: String::new(), ..saved() }.is_complete());
    }

    #[test]
    fn debug_output_redacts_the_key() {
        let text = format!("{:?}", saved());
        assert!(!text.contains(KEY) && !text.contains("abcdefgh") && text.contains("<redacted>"), "{text}");
        assert!(format!("{:?}", AiConfig::default()).contains("<none>"));
    }

    #[test]
    fn a_saved_configuration_survives_restart_and_is_private() {
        let directory = temp("roundtrip");
        let file = directory.join("config/ai.json");
        assert_eq!(load(&file).unwrap(), AiConfig::default(), "a missing file means not configured");
        save(&file, &saved()).unwrap();
        assert_eq!(load(&file).unwrap(), saved());
        let replacement = AiConfig { model: "m9".into(), ..saved() };
        save(&file, &replacement).unwrap();
        assert_eq!(load(&file).unwrap(), replacement);
        let leftovers: Vec<_> = fs::read_dir(file.parent().unwrap()).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().ends_with(".tmp")).collect();
        assert!(leftovers.is_empty());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&file).unwrap().permissions().mode() & 0o077, 0, "group and others get nothing");
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn a_stale_loose_temporary_file_cannot_weaken_the_saved_file() {
        let directory = temp("stale");
        fs::create_dir_all(&directory).unwrap();
        let file = directory.join("ai.json");
        let stale = directory.join(format!("ai-{}.tmp", std::process::id()));
        fs::write(&stale, "old").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&stale, fs::Permissions::from_mode(0o666)).unwrap();
        }
        save(&file, &saved()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&file).unwrap().permissions().mode() & 0o077, 0);
        }
        assert!(!stale.exists());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn older_partial_or_damaged_files_are_handled() {
        let directory = temp("damaged");
        fs::create_dir_all(&directory).unwrap();
        let file = directory.join("ai.json");
        fs::write(&file, r#"{"baseUrl":"https://x.example.com","protocol":"telepathy"}"#).unwrap();
        let loaded = load(&file).unwrap();
        assert_eq!((loaded.protocol.as_str(), loaded.base_url.as_str(), loaded.model.as_str()), ("chat", "https://x.example.com", ""));
        fs::write(&file, "{ this is not json").unwrap();
        let error = load(&file).unwrap_err().to_string();
        assert!(!error.contains("this is not json"), "errors never echo file contents: {error}");
        let recovered = apply(&load(&file).unwrap_or_default(), &Input { api_key: Some("fresh-key-12345"), ..input("https://a.example.com", "m") }).unwrap();
        save(&file, &recovered).unwrap();
        assert_eq!(load(&file).unwrap(), recovered);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn only_the_control_window_may_read_or_change_it() {
        for command in ["get_ai_config", "save_ai_config", "pick_workspace", "fetch_ai_models", "save_ai_provider", "switch_ai_provider", "delete_ai_provider", "save_ai_preferences"] {
            assert!(crate::content::permitted("control", command), "{command}");
            for label in ["pet", "wallpaper", "care", "games", "media-a", "media-preview", "ritual", "widgets"] {
                assert!(!crate::content::permitted(label, command), "{label} must not reach {command}");
            }
        }
    }

    #[test]
    fn candidate_urls_are_constructed_correctly() {
        assert_eq!(
            candidate_models_urls("chat", "https://api.openai.com/v1"),
            vec!["https://api.openai.com/v1/models"]
        );
        assert_eq!(
            candidate_models_urls("chat", "https://api.deepseek.com"),
            vec!["https://api.deepseek.com/models", "https://api.deepseek.com/v1/models"]
        );
        assert_eq!(
            candidate_models_urls("anthropic", "https://api.anthropic.com"),
            vec!["https://api.anthropic.com/v1/models", "https://api.anthropic.com/models"]
        );
        assert_eq!(
            candidate_models_urls("anthropic", "https://api.anthropic.com/v1"),
            vec!["https://api.anthropic.com/v1/models"]
        );
    }

    #[test]
    fn parses_various_provider_model_responses() {
        // OpenAI / Anthropic 标准 data 格式
        let openai_json = r#"{"data":[{"id":"gpt-4o"},{"id":"gpt-4o-mini"}]}"#;
        assert_eq!(parse_model_list_json(openai_json).unwrap(), vec!["gpt-4o", "gpt-4o-mini"]);

        // Ollama / 某些平台的 models 格式
        let ollama_json = r#"{"models":[{"name":"llama3:latest"},{"name":"qwen2.5:7b"}]}"#;
        assert_eq!(parse_model_list_json(ollama_json).unwrap(), vec!["llama3:latest", "qwen2.5:7b"]);

        // 顶层数组格式
        let list_json = r#"[{"id":"claude-3-5-sonnet"},{"id":"claude-3-5-haiku"}]"#;
        assert_eq!(parse_model_list_json(list_json).unwrap(), vec!["claude-3-5-haiku", "claude-3-5-sonnet"]);

        // 纯字符串数组格式
        let str_list = r#"["model-b","model-a","model-b"]"#;
        assert_eq!(parse_model_list_json(str_list).unwrap(), vec!["model-a", "model-b"]);

        // 空或无模型应返回 Err
        assert!(parse_model_list_json(r#"{"data":[]}"#).is_err());
        assert!(parse_model_list_json(r#"{"other":"field"}"#).is_err());
    }

    #[test]
    fn default_providers_are_initialized_and_accessible() {
        let mut cfg = AiConfig::default();
        cfg.normalize();
        assert_eq!(cfg.providers.len(), 4);
        assert!(cfg.providers.iter().any(|p| p.id == "deepseek"));
        assert!(cfg.providers.iter().any(|p| p.id == "zhipu"));
        assert!(cfg.providers.iter().any(|p| p.id == "kimi"));
        assert!(cfg.providers.iter().any(|p| p.id == "minimax"));

        // 添加一个自定义供应商
        cfg.providers.push(ProviderConfig {
            id: "custom_1".into(),
            name: "我的本地 Ollama".into(),
            protocol: "chat".into(),
            base_url: "http://localhost:11434/v1".into(),
            model: "llama3".into(),
            api_key: String::new(),
        });

        let view = cfg.view(Path::new("/work"));
        assert_eq!(view.providers.len(), 5);
        let custom = view.providers.iter().find(|p| p.id == "custom_1").unwrap();
        assert_eq!(custom.name, "我的本地 Ollama");
        assert!(custom.is_custom);
    }
}

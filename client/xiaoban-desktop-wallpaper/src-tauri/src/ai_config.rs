//! The model endpoint the assistant talks to: protocol, address, model name and an optional key.
//!
//! The key has its own file and is never part of `Settings`: settings are broadcast to every window
//! and shown in diagnostics, while this file is read by the host only. The web side can read back
//! no more than whether a key exists and its last four characters.

use serde::{Deserialize, Serialize};
use crate::ai_reasoning::{self, ReplyPreference};
use agent_kernel::model::{ModelCapabilities, ModelProtocol, TokenBudget};
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{Emitter, Manager};

pub const PROTOCOLS: &[&str] = &["responses", "chat", "anthropic"];
const MAX_URL: usize = 2048;
const MAX_MODEL: usize = 200;
const MAX_KEY: usize = 2048;
/// The context limit is chosen with a slider (`chat/format.ts` has the same range).
pub const MIN_CONTEXT_TOKENS: usize = 16_000;
pub const MAX_CONTEXT_TOKENS: usize = 1_000_000;
pub const DEFAULT_CONTEXT_TOKENS: usize = 256_000;

/// The built-in providers a fresh install starts with: id, display name, and the chat-completions
/// address and default model. Each vendor also serves the Responses and Anthropic wire formats on
/// its own path; the web side switches the address with the protocol, and the host only needs a
/// usable default here. See `src/ui/ai-config.ts` for the full list of endpoints.
pub const DEFAULT_PROVIDERS: &[(&str, &str, &str, &str, &str)] = &[
    ("deepseek", "DeepSeek", "responses", "https://api.deepseek.com", "deepseek-v4-pro"),
    ("zhipu", "智谱 BigModel", "responses", "https://open.bigmodel.cn/api/v1", "glm-5.3"),
    ("kimi", "Kimi", "responses", "https://api.moonshot.cn/v1", "kimi-k3"),
    ("minimax", "MiniMax", "responses", "https://api.minimax.cn/v1", "MiniMax-M3"),
];

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProviderConfig {
    pub reasoning_efforts: Option<Vec<String>>,
    pub reply_preference: ReplyPreference,
    #[serde(rename = "type")] pub provider_type: String,
    pub remote: bool,
    pub available: bool,
    pub description: String,
    pub default_model: String,
    pub models: Vec<ModelConfig>,
    pub endpoints: std::collections::BTreeMap<String, String>,
    pub console_url: String,
    pub console_text: String,
    pub selection_mode: String,
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ModelConfig {
    /// Request visible Responses summaries independently of effort selection.
    #[serde(alias = "reasoning_summary", skip_serializing_if = "Option::is_none")]
    pub reasoning_summary: Option<bool>,
    /// Explicit Messages thinking protocol for private aliases and legacy budget models.
    #[serde(alias = "thinking_mode", skip_serializing_if = "Option::is_none")]
    pub thinking_mode: Option<ai_reasoning::ThinkingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_efforts: Option<Vec<String>>,
    pub id: String, pub name: String, pub description: String,
    #[serde(alias = "max_output_tokens", skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    /// Optional service-declared total context window. Unknown means unknown,
    /// not the client slider value; aliases accept common provider catalogue formats.
    #[serde(alias = "contextLength", alias = "context_length", alias = "contextWindow", alias = "context_window_tokens", skip_serializing_if = "Option::is_none")]
    pub context_window_tokens: Option<usize>,
    #[serde(alias = "supportsToolCalling", alias = "toolCalling", skip_serializing_if = "Option::is_none")]
    pub supports_tools: Option<bool>,
    #[serde(alias = "vision", skip_serializing_if = "Option::is_none")]
    pub supports_vision: Option<bool>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            reasoning_efforts: None,
            reply_preference: ReplyPreference::Default,
            provider_type: "custom".into(), remote: false, available: true,
            description: String::new(), default_model: String::new(), models: Vec::new(),
            endpoints: Default::default(), console_url: String::new(), console_text: String::new(),
            selection_mode: "manual".into(),
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
    pub reasoning_efforts: Option<Vec<String>>,
    pub reply_preference: ReplyPreference,
    #[serde(rename = "type")] pub provider_type: String,
    pub remote: bool, pub available: bool, pub description: String,
    pub default_model: String, pub models: Vec<ModelConfig>,
    pub endpoints: std::collections::BTreeMap<String, String>,
    pub console_url: String, pub console_text: String, pub selection_mode: String,
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
        !self.remote && !["deepseek", "zhipu", "kimi", "minimax"].contains(&self.id.as_str())
    }

    /// Enough to send a request: an address, a model, and a key unless this is a local custom
    /// service or an official free service that needs none. The web side draws the same line.
    pub fn is_usable(&self) -> bool {
        self.available
            && !self.base_url.trim().is_empty()
            && !self.model.trim().is_empty()
            && (!self.api_key.trim().is_empty() || self.is_custom() || self.provider_type == "free")
    }

    pub fn view(&self) -> ProviderConfigView {
        ProviderConfigView {
            reasoning_efforts: self.reasoning_efforts.clone(),
            reply_preference: self.reply_preference,
            provider_type: if self.is_custom() { "custom".into() } else if self.provider_type == "custom" { "third".into() } else { self.provider_type.clone() },
            remote: self.remote, available: self.available, description: self.description.clone(),
            default_model: self.default_model.clone(), models: self.models.clone(), endpoints: self.endpoints.clone(),
            console_url: self.console_url.clone(), console_text: self.console_text.clone(), selection_mode: self.selection_mode.clone(),
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
    pub gateway_reply_preference: ReplyPreference,
    pub gateway_reasoning_efforts: Vec<String>,
    pub catalogue_received: bool,
    pub catalogue_owner: String,
    pub catalogue_fetched_at: u64,
    pub catalogue_etag: String,
    pub catalogue_refresh_after: u64,
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
    /// Whether to hide intermediate tool call cards in the conversation.
    pub hide_tool_calls: bool,
    /// Whether open-computer-use MCP tools are enabled.
    pub open_computer_use: bool,
    /// The tier chosen for 小伴 AI. Used only while `active_provider` is the gateway; the
    /// short-lived account token is never saved here.
    pub gateway_tier: String,
    /// How much conversation is sent to the model before the oldest part is dropped, in estimated tokens.
    pub context_tokens: usize,
    /// The 小伴 AI entry is on in the catalogue the site delivers. On until the site says otherwise.
    pub gateway_enabled: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        let mut cfg = Self {
            gateway_reply_preference: ReplyPreference::Default,
            gateway_reasoning_efforts: Vec::new(),
            catalogue_received: false, catalogue_owner: String::new(), catalogue_fetched_at: 0,
            catalogue_etag: String::new(), catalogue_refresh_after: 21600,
            active_provider: String::new(),
            providers: Vec::new(),
            protocol: "responses".into(),
            base_url: String::new(),
            model: String::new(),
            api_key: String::new(),
            workspace: String::new(),
            hide_tool_calls: false,
            open_computer_use: false,
            gateway_tier: String::new(),
            context_tokens: DEFAULT_CONTEXT_TOKENS,
            gateway_enabled: true,
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

    /// Resolve only the currently selected endpoint, never capabilities from another
    /// service's model with the same ID. Protocol fallback belongs to the kernel.
    pub(crate) fn model_capabilities(&self) -> ModelCapabilities {
        let protocol = match self.protocol.as_str() {
            "anthropic" => ModelProtocol::AnthropicMessages,
            "responses" => ModelProtocol::Responses,
            _ => ModelProtocol::ChatCompletions,
        };
        let provider = self.providers.iter().find(|p| p.id == self.active_provider
            && p.base_url == self.base_url && p.model == self.model && p.protocol == self.protocol);
        let declared = provider.and_then(|p| p.models.iter().find(|m| m.id == self.model));
        ModelCapabilities::resolve(
            protocol, &self.model,
            declared.and_then(|m| m.context_window_tokens),
            declared.and_then(|m| m.max_output_tokens),
        ).with_declared_features(
            declared.and_then(|m| m.supports_tools),
            declared.and_then(|m| m.supports_vision),
            provider.and_then(|p| p.reasoning_efforts.clone())
                .or_else(|| declared.and_then(|m| m.reasoning_efforts.clone())),
        )
    }

    pub(crate) fn token_budget(&self) -> TokenBudget {
        TokenBudget::resolve(self.context_tokens, self.model_capabilities().context_window_tokens)
    }

    /// Whether the selected provider is ready to use. Saving another provider must not steal the
    /// selection while this is true. 小伴 AI counts while the site still offers it.
    pub fn has_usable_active_provider(&self) -> bool {
        if !self.is_complete() {
            return false;
        }
        if self.active_provider == crate::ai_gateway::GATEWAY_ID {
            return self.gateway_enabled;
        }
        self.providers.iter().find(|p| p.id == self.active_provider).is_some_and(ProviderConfig::is_usable)
    }

    pub fn workspace_or<'a>(&'a self, default: &'a Path) -> &'a Path {
        if self.workspace.is_empty() { default } else { Path::new(&self.workspace) }
    }

    /// Read from the provider actually in effect, rather than a global preference that could
    /// leak into 小伴 AI or a different provider after switching or a catalogue update.
    pub(crate) fn reply_preference(&self) -> ReplyPreference {
        if self.active_provider == crate::ai_gateway::GATEWAY_ID { return self.gateway_reply_preference; }
        self.providers.iter().find(|p| p.id == self.active_provider
            && p.base_url == self.base_url && p.model == self.model && p.protocol == self.protocol)
            .map(|p| p.reply_preference).unwrap_or_default()
    }

    pub(crate) fn reasoning_body(&self) -> serde_json::Map<String, serde_json::Value> {
        // Gateway models use their server-managed capabilities, never local overrides.
        let declared = if self.active_provider == crate::ai_gateway::GATEWAY_ID { None } else {
            self.providers.iter().find(|p| p.id == self.active_provider
                && p.base_url == self.base_url && p.model == self.model && p.protocol == self.protocol)
                .and_then(|p| p.models.iter().find(|m| m.id == self.model))
        };
        ai_reasoning::body_with_capabilities(&self.protocol, self.reply_preference(), &self.reasoning_levels(),
            declared.and_then(|m| m.reasoning_summary), declared.and_then(|m| m.thinking_mode.as_ref()))
    }

    pub(crate) fn reasoning_levels(&self) -> Vec<String> {
        if self.active_provider == crate::ai_gateway::GATEWAY_ID { return self.gateway_reasoning_efforts.clone(); }
        let capabilities = self.model_capabilities();
        ai_reasoning::levels(&self.model, &self.protocol, capabilities.reasoning_efforts.as_deref())
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
    pub gateway_reply_preference: ReplyPreference,
    pub gateway_reasoning_efforts: Vec<String>,
    pub catalogue_received: bool,
    pub catalogue_fetched_at: u64,
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
    pub hide_tool_calls: bool,
    pub open_computer_use: bool,
    pub gateway_tier: String,
    pub gateway_enabled: bool,
}

impl AiConfig {
    pub fn view(&self, default_workspace: &Path) -> AiConfigView {
        let providers_view = self.providers.iter().map(|p| p.view()).collect();
        AiConfigView {
            gateway_reply_preference: self.gateway_reply_preference,
            gateway_reasoning_efforts: self.gateway_reasoning_efforts.clone(),
            catalogue_received: self.catalogue_received, catalogue_fetched_at: self.catalogue_fetched_at,
            active_provider: self.active_provider.clone(),
            providers: providers_view,
            protocol: self.protocol.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            has_key: !self.api_key.is_empty(),
            key_hint: make_key_hint(&self.api_key),
            workspace: self.workspace.clone(),
            default_workspace: default_workspace.display().to_string(),
            hide_tool_calls: self.hide_tool_calls,
            open_computer_use: self.open_computer_use,
            gateway_tier: self.gateway_tier.clone(),
            gateway_enabled: self.gateway_enabled,
        }
    }

    fn normalize(&mut self) {
        self.context_tokens = if self.context_tokens == 0 { DEFAULT_CONTEXT_TOKENS } else { self.context_tokens.clamp(MIN_CONTEXT_TOKENS, MAX_CONTEXT_TOKENS) };
        if !PROTOCOLS.contains(&self.protocol.as_str()) {
            self.protocol = "responses".into();
        }

        for &(id, name, default_proto, default_url, default_model) in DEFAULT_PROVIDERS {
            if !self.catalogue_received && !self.providers.iter().any(|p| p.id == id) {
                self.providers.push(ProviderConfig {
                    id: id.into(),
                    name: name.into(),
                    protocol: default_proto.into(),
                    base_url: default_url.into(),
                    model: default_model.into(),
                    api_key: String::new(), provider_type: "third".into(),
                    selection_mode: "recommended".into(), ..ProviderConfig::default()
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
                String::new()
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

fn clean_context_tokens(size: u32) -> Result<usize, String> {
    let size = size as usize;
    if (MIN_CONTEXT_TOKENS..=MAX_CONTEXT_TOKENS).contains(&size) {
        Ok(size)
    } else {
        Err(format!("上下文大小需要在 {}K 到 {}M 之间", MIN_CONTEXT_TOKENS / 1000, MAX_CONTEXT_TOKENS / 1_000_000))
    }
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
        p.selection_mode = "manual".into();
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
    pub(crate) io: Mutex<()>,
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
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    load(&path).map_err(|e| format!("模型配置读取失败：{e}"))
}

/// Notify only the local configuration and chat windows, once, without exposing credentials.
pub(crate) fn emit_changed<R: tauri::Runtime>(app: &tauri::AppHandle<R>, view: &AiConfigView) {
    let _ = app.emit_filter("ai-config-changed", view, configuration_target);
}

fn configuration_target(target: &tauri::EventTarget) -> bool {
    use tauri::EventTarget::{AnyLabel, Webview, WebviewWindow, Window};
    matches!(target, AnyLabel { label } | Webview { label } | WebviewWindow { label } | Window { label }
        if label == "control" || label == crate::float_assistant::LABEL)
}

#[tauri::command]
pub fn get_ai_config(app: tauri::AppHandle) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
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
    hide_tool_calls: Option<bool>,
    open_computer_use: Option<bool>,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    // A damaged file must not stop the user from writing a new one over it.
    let current = load(&path).unwrap_or_default();
    let next = apply(&current, &Input { protocol: &protocol, base_url: &base_url, model: &model, api_key: api_key.as_deref(), clear_key, workspace: workspace.as_deref(), hide_tool_calls, open_computer_use })?;
    save(&path, &next).map_err(|e| format!("无法保存模型配置：{e}"))?;
    if next.workspace != current.workspace {
        let chosen = next.workspace_or(&default_workspace(&app)?).display().to_string();
        crate::chat::remember_workspace(&app, &chosen);
    }
    let _ = app.emit_to("control", "chat-hide-tool-calls-changed", serde_json::json!({ "enabled": next.hide_tool_calls }));
    let view = next.view(&default_workspace(&app)?);
    emit_changed(&app, &view);
    Ok(view)
}

/// Whether saving a provider should also make it the one in effect: the user pressed
/// 保存并启用, re-saved the provider already in effect, or nothing usable is in effect yet and the
/// saved one is ready — in that last case it takes over by itself instead of waiting for a click.
fn selecting_after_save(config: &AiConfig, provider_id: &str, activate: bool) -> bool {
    if activate || config.active_provider == provider_id {
        return true;
    }
    !config.has_usable_active_provider()
        && config.providers.iter().any(|p| p.id == provider_id && p.is_usable())
}

/// Whether the facts a model carries are sane; shared by the one discovered model and the edited list.
fn model_facts_valid(model: &ModelConfig) -> bool {
    !(!ai_reasoning::validate_thinking(model.thinking_mode.as_ref(), model.max_output_tokens)
        || model.id.len() > MAX_MODEL
        || model.max_output_tokens.is_some_and(|n| n == 0 || n > 2_000_000)
        || model.context_window_tokens.is_some_and(|n| n == 0 || n > 16_000_000)
        || model.name.len() > 250 || model.description.len() > 500
        || model.reasoning_efforts.as_ref().is_some_and(|v| ai_reasoning::validate(v).is_err()))
}

/// The model list edited in the provider dialog: every id valid and unique, every fact sane, and the
/// selected model among them. Order is kept (the first is just the first the user added).
pub(crate) fn clean_model_list(list: Vec<ModelConfig>, selected: &str) -> Result<Vec<ModelConfig>, String> {
    if list.len() > 200 { return Err("模型列表太长，最多 200 个".into()); }
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(list.len());
    for mut model in list {
        model.id = clean_model(&model.id)?;
        if !model_facts_valid(&model) { return Err("服务商模型能力数据无效".into()); }
        if seen.insert(model.id.clone()) { out.push(model); }
    }
    if !out.iter().any(|m| m.id == selected) { return Err("默认模型必须在模型列表中".into()); }
    Ok(out)
}

fn clean_reply_preference(previous: &ProviderConfig, _url: &str, model: &str, protocol: &str, requested: Option<ReplyPreference>) -> Result<ReplyPreference, String> {
    let declared = previous.reasoning_efforts.as_deref().or_else(|| previous.models.iter().find(|m| m.id == model).and_then(|m| m.reasoning_efforts.as_deref()));
    let levels = ai_reasoning::levels(model, protocol, declared);
    let available = !levels.is_empty();
    let preference = requested.unwrap_or(previous.reply_preference);
    if preference.effort().is_some_and(|effort| !levels.iter().any(|l| l == effort)) {
        if requested.is_some() { return Err("此模型未声明支持所选思考等级，请检查模型能力配置".into()); }
        return Ok(ReplyPreference::Default);
    }
    Ok(if available { preference } else { ReplyPreference::Default })
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
    selection_mode: Option<String>,
    reply_preference: Option<ReplyPreference>,
    reasoning_efforts: Option<Vec<String>>,
    clear_reasoning_efforts: Option<bool>,
    discovered_model: Option<ModelConfig>,
    models: Option<Vec<ModelConfig>>,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    let clean_url = clean_base_url(&base_url)?;
    let clean_mdl = clean_model(&model)?;
    if !PROTOCOLS.contains(&protocol.as_str()) {
        return Err("不支持这种接口协议".into());
    }

    let existing_idx = config.providers.iter().position(|p| p.id == provider_id);
    let mut previous = existing_idx.map(|i| config.providers[i].clone()).unwrap_or_default();
    let mode = selection_mode.as_deref().unwrap_or("manual");
    if !["manual", "recommended"].contains(&mode) { return Err("模型选择方式无效".into()); }
    let clean_mdl = if mode == "recommended" && !previous.default_model.is_empty() { previous.default_model.clone() } else { clean_mdl };
    if previous.remote && previous.provider_type != "third" && (reasoning_efforts.is_some() || clear_reasoning_efforts.unwrap_or(false)) {
        return Err("官方服务的模型能力由服务端管理".into());
    }
    if previous.model != clean_mdl || previous.protocol != protocol {
        previous.reasoning_efforts = None;
    }
    // Model facts are scoped to the exact endpoint. An old provider catalogue
    // must never leak capabilities into a newly configured proxy URL.
    if (previous.base_url != clean_url || previous.protocol != protocol)
        && (!previous.remote || previous.provider_type == "third")
    {
        previous.models.clear();
    }
    if let Some(model) = discovered_model {
        if previous.remote && previous.provider_type != "third" {
            return Err("官方模型能力由服务端目录管理".into());
        }
        if model.id != clean_mdl || !model_facts_valid(&model) {
            return Err("服务商模型能力数据无效".into());
        }
        // Only the selected discovered model is persisted; the list of IDs
        // can be large, and all other models will be re-resolved when selected.
        previous.models.retain(|m| m.id != model.id);
        previous.models.push(model);
    }
    // The whole list the user edited (several models, each with its own thinking levels) replaces the
    // saved one. Official services keep their catalogue; a catalogue refresh re-adds its own models.
    if let Some(list) = models {
        if previous.remote && previous.provider_type != "third" {
            return Err("官方模型列表由服务端目录管理".into());
        }
        previous.models = clean_model_list(list, &clean_mdl)?;
    }
    if let Some(levels) = reasoning_efforts { ai_reasoning::validate(&levels)?; previous.reasoning_efforts = Some(levels); }
    if clear_reasoning_efforts.unwrap_or(false) { previous.reasoning_efforts = None; }
    if previous.remote && !previous.available { return Err("该供应商已停用，请选择其他模型".into()); }
    if previous.remote && previous.provider_type != "third" {
        if clean_url != previous.base_url || protocol != previous.protocol || api_key.is_some() || clear_key {
            return Err("官方服务的地址、协议和凭证由服务端管理".into());
        }
        if !previous.models.iter().any(|m| m.id == clean_mdl) { return Err("该模型已停用，请选择其他模型".into()); }
    }
    let preference = clean_reply_preference(&previous, &clean_url, &clean_mdl, &protocol, reply_preference)?;
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
        reply_preference: preference,
        id: provider_id.clone(),
        name: new_name,
        protocol: protocol.clone(),
        base_url: clean_url.clone(),
        model: clean_mdl.clone(),
        api_key: new_key.clone(),
        selection_mode: mode.into(), ..previous
    };

    if let Some(idx) = existing_idx {
        config.providers[idx] = updated_entry;
    } else {
        config.providers.push(updated_entry);
    }

    // With no provider in effect, the one the user just configured takes over by itself, so they
    // never have to come back and press 启用. An incomplete entry is not put in effect yet.
    let auto_activate = selecting_after_save(&config, &provider_id, activate);
    if auto_activate {
        config.active_provider = provider_id;
        config.protocol = protocol;
        config.base_url = clean_url;
        config.model = clean_mdl;
        config.api_key = new_key;
    }

    save(&path, &config).map_err(|e| format!("无法保存服务商配置：{e}"))?;
    let view = config.view(&default_workspace(&app)?);
    emit_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn delete_ai_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    if config.providers.iter().any(|p| p.id == provider_id && !p.is_custom()) {
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
    let view = config.view(&default_workspace(&app)?);
    emit_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn switch_ai_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    let provider = config.providers.iter().find(|p| p.id == provider_id)
        .ok_or_else(|| format!("未找到服务商配置：{provider_id}"))?
        .clone();

    if !provider.available { return Err("该供应商已停用，请切换模型".into()); }
    if provider.remote && provider.provider_type != "third" && !provider.models.iter().any(|m| m.id == provider.model) {
        return Err("该模型已停用，请选择其他模型".into());
    }
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
    let view = config.view(&default_workspace(&app)?);
    emit_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn save_ai_preferences(
    app: tauri::AppHandle,
    workspace: Option<String>,
    hide_tool_calls: Option<bool>,
    open_computer_use: Option<bool>,
) -> Result<AiConfigView, String> {
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    let mut config = load(&path).unwrap_or_default();
    config.normalize();

    if let Some(raw) = workspace.as_deref() {
        config.workspace = clean_workspace(raw)?;
    }
    if let Some(htc) = hide_tool_calls {
        config.hide_tool_calls = htc;
    }
    if let Some(ocu) = open_computer_use {
        config.open_computer_use = ocu;
    }

    save(&path, &config).map_err(|e| format!("无法保存偏好设置：{e}"))?;
    let _ = app.emit_to("control", "chat-hide-tool-calls-changed", serde_json::json!({ "enabled": config.hide_tool_calls }));
    let view = config.view(&default_workspace(&app)?);
    emit_changed(&app, &view);
    Ok(view)
}

/// The context limit the chat page's slider chose. Saved on its own so moving the slider never touches the other settings.
#[tauri::command]
pub fn set_context_tokens(app: tauri::AppHandle, tokens: u32) -> Result<(), String> {
    let size = clean_context_tokens(tokens)?;
    let path = path(&app)?;
    let state = app.state::<AiConfigState>();
    let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    let mut config = load(&path).unwrap_or_default();
    config.context_tokens = size;
    save(&path, &config).map_err(|e| format!("无法保存上下文大小：{e}"))
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

/// Parse optional capabilities from the *selected endpoint* rather than deriving
/// them from hard-coded model names. Many /models APIs return IDs only; keep all
/// absent fields as None so the kernel can use protocol-specific recovery.
pub fn parse_model_details_json(raw: &str, protocol: &str) -> Result<Vec<ModelConfig>, String> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| format!("解析服务商返回的 JSON 失败：{e}"))?;
    let models = value.get("data").and_then(|v| v.as_array())
        .or_else(|| value.get("models").and_then(|v| v.as_array()))
        .or_else(|| value.as_array())
        .ok_or_else(|| "服务商返回的数据中未包含模型列表".to_string())?;

    fn number(value: &serde_json::Value, keys: &[&str], ceiling: u64) -> Option<u64> {
        keys.iter().filter_map(|key| value.get(*key).and_then(|v| v.as_u64()))
            .find(|n| *n > 0 && *n <= ceiling)
    }
    fn flag(value: &serde_json::Value, keys: &[&str]) -> Option<bool> {
        keys.iter().find_map(|key| value.get(*key).and_then(|v| v.as_bool()))
    }
    let mut parsed = Vec::new();
    for item in models {
        let id = item.as_str().or_else(|| item.get("id").and_then(|v| v.as_str()))
            .or_else(|| item.get("name").and_then(|v| v.as_str()))
            .unwrap_or("").trim();
        if id.is_empty() || id.len() > MAX_MODEL || id.chars().any(char::is_control) { continue; }
        let mut model = ModelConfig { id: id.into(), name: id.into(), ..ModelConfig::default() };
        if let Some(name) = item.get("display_name").or_else(|| item.get("displayName"))
            .and_then(|v| v.as_str()).filter(|v| !v.is_empty() && v.len() <= 250)
        {
            model.name = name.into();
        }
        if let Some(description) = item.get("description").and_then(|v| v.as_str())
            .filter(|v| v.len() <= 500)
        {
            model.description = description.into();
        }
        let output_keys = if protocol == "anthropic" {
            &["max_output_tokens", "maxOutputTokens", "max_completion_tokens", "max_tokens"][..]
        } else {
            &["max_output_tokens", "maxOutputTokens", "max_completion_tokens", "maxCompletionTokens"][..]
        };
        model.max_output_tokens = number(item, output_keys, 2_000_000).map(|n| n as u32)
            .or_else(|| item.get("top_provider")
                .and_then(|v| number(v, &["max_completion_tokens"], 2_000_000))
                .map(|n| n as u32));
        let context_keys = if protocol == "anthropic" {
            &["max_input_tokens", "context_window_tokens", "context_window", "context_length", "contextLength"][..]
        } else {
            &["context_window_tokens", "context_window", "context_length", "contextLength", "max_model_len"][..]
        };
        model.context_window_tokens = number(item, context_keys, 16_000_000).map(|n| n as usize);
        model.supports_tools = flag(item, &["supports_tools", "supportsTools", "tool_calling", "toolCalling"]);
        model.supports_vision = flag(item, &["supports_vision", "supportsVision", "vision"]);
        model.reasoning_summary = flag(item, &["reasoning_summary", "reasoningSummary"]);
        model.reasoning_efforts = item.get("reasoning_efforts").or_else(|| item.get("reasoningEfforts"))
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
            .filter(|levels| ai_reasoning::validate(levels).is_ok());
        model.thinking_mode = item.get("thinking_mode").or_else(|| item.get("thinkingMode"))
            .and_then(|v| serde_json::from_value::<ai_reasoning::ThinkingMode>(v.clone()).ok())
            .filter(|mode| ai_reasoning::validate_thinking(Some(mode), model.max_output_tokens));
        // Models APIs may also expose a nested capabilities object.
        if let Some(capabilities) = item.get("capabilities") {
            model.supports_tools = model.supports_tools.or_else(|| capabilities.get("tool_use")
                .and_then(|v| flag(v, &["supported"])));
            model.supports_vision = model.supports_vision.or_else(|| capabilities.get("vision")
                .and_then(|v| flag(v, &["supported"])));
        }
        parsed.push(model);
    }
    parsed.sort_by(|a, b| a.id.cmp(&b.id));
    parsed.dedup_by(|a, b| a.id == b.id);
    if parsed.is_empty() { return Err("服务商返回的数据中未包含任何模型".into()); }
    Ok(parsed)
}

/// Backwards-compatible ID-only parser used by earlier clients and tests.
pub fn parse_model_list_json(raw: &str) -> Result<Vec<String>, String> {
    Ok(parse_model_details_json(raw, "chat")?.into_iter().map(|m| m.id).collect())
}

#[tauri::command]
pub async fn fetch_ai_models(
    app: tauri::AppHandle,
    protocol: String,
    base_url: String,
    api_key: Option<String>,
) -> Result<Vec<ModelConfig>, String> {
    let base_url = clean_base_url(&base_url)?;
    let protocol = if PROTOCOLS.contains(&protocol.as_str()) { protocol } else { "chat".into() };

    let effective_key = match api_key {
        Some(key) if !key.trim().is_empty() => clean_key(&key)?,
        _ => {
            let path = path(&app)?;
            let state = app.state::<AiConfigState>();
            let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
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
            return parse_model_details_json(&body, &protocol);
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
        let mut cfg = AiConfig { active_provider: "deepseek".into(), protocol: "responses".into(), base_url: "https://api.example.com/v1".into(), model: "m1".into(), api_key: KEY.into(), ..AiConfig::default() };
        cfg.normalize();
        cfg
    }

    fn input<'a>(base_url: &'a str, model: &'a str) -> Input<'a> {
        Input { protocol: "chat", base_url, model, api_key: None, clear_key: false, workspace: None, hide_tool_calls: None, open_computer_use: None }
    }

    fn temp(tag: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("xiaoban-ai-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn model_config_notifications_only_target_the_two_trusted_windows() {
        use tauri::EventTarget;
        for label in ["control", crate::float_assistant::LABEL, "pet", "wallpaper", "preview", "game"] {
            let allowed = label == "control" || label == crate::float_assistant::LABEL;
            for target in [
                EventTarget::AnyLabel { label: label.into() },
                EventTarget::Window { label: label.into() },
                EventTarget::Webview { label: label.into() },
                EventTarget::WebviewWindow { label: label.into() },
            ] {
                assert_eq!(configuration_target(&target), allowed, "{label}");
            }
        }
        assert!(!configuration_target(&EventTarget::Any));
        assert!(!configuration_target(&EventTarget::App));
    }

    #[test]
    fn model_config_notifications_are_redacted_and_delivered_once() {
        use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
        use tauri::{Listener, WebviewUrl, WebviewWindowBuilder};
        let app = tauri::test::mock_app();
        for label in ["control", crate::float_assistant::LABEL, "wallpaper"] {
            WebviewWindowBuilder::new(app.handle(), label, WebviewUrl::App("index.html".into())).build().unwrap();
        }
        let seen = Arc::new(AtomicUsize::new(0));
        let counter = seen.clone();
        app.handle().listen_any("ai-config-changed", move |event| {
            counter.fetch_add(1, Ordering::SeqCst);
            assert!(!event.payload().contains(KEY));
            let value: serde_json::Value = serde_json::from_str(event.payload()).unwrap();
            assert_eq!(value["hasKey"], true);
            assert!(value.get("apiKey").is_none());
            assert!(value["providers"].as_array().unwrap().iter().all(|p| p.get("apiKey").is_none()));
        });
        emit_changed(app.handle(), &saved().view(Path::new("/work")));
        assert_eq!(seen.load(Ordering::SeqCst), 1);
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
    fn a_freshly_configured_provider_is_enabled_without_a_second_click() {
        // Nothing is in effect yet: the saved provider takes over by itself. The check runs on the
        // configuration after the save, so deepseek here already carries the key just typed.
        let mut fresh = AiConfig::default();
        assert!(!fresh.has_usable_active_provider());
        fresh.providers.iter_mut().find(|p| p.id == "deepseek").unwrap().api_key = KEY.into();
        assert!(!fresh.has_usable_active_provider(), "nothing is selected yet");
        assert!(selecting_after_save(&fresh, "deepseek", false));

        // A provider that is already in effect keeps its place when another one is configured.
        let working = saved();
        assert!(working.has_usable_active_provider());
        assert!(!selecting_after_save(&working, "zhipu", false), "another provider does not steal the selection");
        assert!(selecting_after_save(&working, "deepseek", false), "re-saving the one in effect keeps it");

        // The user's explicit choice always wins.
        assert!(selecting_after_save(&working, "zhipu", true));

        // 小伴 AI counts as in effect while the site still offers it.
        let mut gateway = AiConfig { active_provider: crate::ai_gateway::GATEWAY_ID.into(), base_url: "https://ai.example.com/v1".into(), model: "standard".into(), ..AiConfig::default() };
        gateway.gateway_enabled = true;
        assert!(gateway.has_usable_active_provider());
        assert!(!selecting_after_save(&gateway, "deepseek", false));
        gateway.gateway_enabled = false;
        assert!(!gateway.has_usable_active_provider(), "a closed entry leaves room for a key provider");
        gateway.providers.iter_mut().find(|p| p.id == "deepseek").unwrap().api_key = KEY.into();
        assert!(selecting_after_save(&gateway, "deepseek", false));
    }

    #[test]
    fn an_incomplete_provider_is_not_put_in_effect() {
        let local = ProviderConfig { id: "custom_x".into(), base_url: "https://a.example.com/v1".into(), model: "m".into(), ..ProviderConfig::default() };
        assert!(local.is_usable(), "a local custom service needs no key");
        let keyed = ProviderConfig { id: "deepseek".into(), base_url: "https://api.deepseek.com".into(), model: "m".into(), ..ProviderConfig::default() };
        assert!(!keyed.is_usable(), "a vendor preset without its key is not usable");
        let keyed = ProviderConfig { api_key: KEY.into(), ..keyed };
        assert!(keyed.is_usable());
        assert!(!ProviderConfig { available: false, ..keyed.clone() }.is_usable());
        assert!(!ProviderConfig { model: "  ".into(), ..keyed }.is_usable());
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
        assert_eq!((loaded.protocol.as_str(), loaded.base_url.as_str(), loaded.model.as_str()), ("responses", "https://x.example.com", ""));
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
        for command in ["refresh_ai_providers", "get_ai_config", "save_ai_config", "pick_workspace", "fetch_ai_models", "save_ai_provider", "switch_ai_provider", "delete_ai_provider", "save_ai_preferences"] {
            assert!(crate::content::permitted("control", command), "{command}");
            for label in ["pet", "wallpaper", "care", "games", "media-a", "media-preview", "ritual", "widgets"] {
                assert!(!crate::content::permitted(label, command), "{label} must not reach {command}");
            }
        }
    }

    #[test]
    fn an_edited_model_list_is_cleaned_and_must_contain_the_default() {
        let model = |id: &str, efforts: Option<Vec<&str>>| ModelConfig { id: id.into(), name: id.into(), reasoning_efforts: efforts.map(|v| v.into_iter().map(String::from).collect()), ..ModelConfig::default() };
        let list = clean_model_list(vec![model(" a ", None), model("b", Some(vec!["low", "high"])), model("a", None), model("c", Some(vec![]))], "b").unwrap();
        assert_eq!(list.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), ["a", "b", "c"], "trimmed, duplicates dropped, order kept");
        assert_eq!(list[1].reasoning_efforts, Some(vec!["low".to_string(), "high".to_string()]));
        assert_eq!(list[2].reasoning_efforts, Some(vec![]), "an empty declaration means no thinking levels");
        assert!(clean_model_list(vec![model("a", None)], "b").is_err(), "the default must be listed");
        assert!(clean_model_list(vec![model("", None)], "a").is_err());
        assert!(clean_model_list(vec![model("a", Some(vec!["low", "low"]))], "a").is_err(), "levels must be valid and unique");
        let mut huge = model("a", None);
        huge.max_output_tokens = Some(0);
        assert!(clean_model_list(vec![huge], "a").is_err());
        assert!(clean_model_list((0..201).map(|i| model(&format!("m{i}"), None)).collect(), "m0").is_err());
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
    fn model_discovery_retains_live_capabilities_without_guessing_from_names() {
        let anthropic = r#"{"data":[{
            "id":"new-claude-family-2030", "display_name":"Claude Future",
            "max_input_tokens":1000000, "max_tokens":128000,
            "capabilities":{"tool_use":{"supported":true},"vision":{"supported":false}}
        }]}"#;
        let models = parse_model_details_json(anthropic, "anthropic").unwrap();
        assert_eq!(models.len(), 1);
        let model = &models[0];
        assert_eq!(model.id, "new-claude-family-2030");
        assert_eq!(model.name, "Claude Future");
        assert_eq!(model.context_window_tokens, Some(1_000_000));
        assert_eq!(model.max_output_tokens, Some(128_000));
        assert_eq!(model.supports_tools, Some(true));
        assert_eq!(model.supports_vision, Some(false));

        let proxied = r#"{"data":[
          {"id":"future-model","context_length":262144,"top_provider":{"max_completion_tokens":65536}},
          {"id":"id-only","max_tokens":1024}
        ]}"#;
        let models = parse_model_details_json(proxied, "chat").unwrap();
        assert_eq!(models[0].id, "future-model");
        assert_eq!(models[0].context_window_tokens, Some(262_144));
        assert_eq!(models[0].max_output_tokens, Some(65_536));
        assert_eq!(models[1].max_output_tokens, None,
            "an arbitrary OpenAI-compatible max_tokens field is not trusted as a cap");
        assert_eq!(parse_model_details_json(r#"{"models":[{"name":"local:latest"}]}"#, "responses")
            .unwrap()[0].max_output_tokens, None);
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
        // Every built-in starts on chat completions with a working address and model.
        for p in &cfg.providers {
            assert_eq!(p.protocol, "responses", "{} starts on the responses protocol", p.id);
            assert!(!p.base_url.is_empty() && !p.model.is_empty(), "{} has a usable default", p.id);
        }

        // 添加一个自定义供应商
        cfg.providers.push(ProviderConfig {
            id: "custom_1".into(),
            name: "我的本地 Ollama".into(),
            protocol: "chat".into(),
            base_url: "http://localhost:11434/v1".into(),
            model: "llama3".into(),
            api_key: String::new(),
            ..ProviderConfig::default()
        });

        let view = cfg.view(Path::new("/work"));
        assert_eq!(view.providers.len(), 5);
        let custom = view.providers.iter().find(|p| p.id == "custom_1").unwrap();
        assert_eq!(custom.name, "我的本地 Ollama");
        assert!(custom.is_custom);
    }

    #[test]
    fn a_builtin_provider_may_be_switched_to_any_of_the_three_protocols() {
        // The vendors serve all three wire formats on their own paths; the host only stores what
        // the form sent, so every protocol has to be accepted for a built-in id.
        for protocol in PROTOCOLS {
            let next = apply(&saved(), &Input { protocol, ..input("https://api.deepseek.com/anthropic", "m") }).unwrap();
            assert_eq!(next.protocol, *protocol, "{protocol} is accepted");
            assert_eq!(next.providers.iter().find(|p| p.id == next.active_provider).unwrap().protocol, *protocol, "the provider entry follows");
        }
        assert!(apply(&saved(), &Input { protocol: "grpc", ..input("https://a.example.com", "m") }).is_err());
    }

    #[test]
    fn the_context_size_defaults_for_old_files_and_is_kept_inside_its_range() {
        let old: AiConfig = serde_json::from_str(r#"{"activeProvider":"deepseek","baseUrl":"https://a.example.com","model":"m"}"#).unwrap();
        assert_eq!(old.context_tokens, DEFAULT_CONTEXT_TOKENS, "a file written before the setting existed gets the default");
        assert_eq!(AiConfig::default().context_tokens, 256_000);

        // Keep valid smaller windows from older builds; zero means use the default.
        for (stored, expected) in [(0, DEFAULT_CONTEXT_TOKENS), (48_000, 48_000), (8_000, MIN_CONTEXT_TOKENS), (300_000, 300_000), (9_999_999, MAX_CONTEXT_TOKENS)] {
            let mut config = AiConfig { context_tokens: stored, ..AiConfig::default() };
            config.normalize();
            assert_eq!(config.context_tokens, expected, "stored {stored}");
        }
    }

    #[test]
    fn a_context_size_from_the_form_must_be_inside_the_range() {
        assert_eq!(clean_context_tokens(512_000), Ok(512_000));
        assert_eq!(clean_context_tokens(256_000), Ok(256_000));
        assert_eq!(clean_context_tokens(1_000_000), Ok(1_000_000));
        assert_eq!(clean_context_tokens(16_000), Ok(16_000));
        assert_eq!(clean_context_tokens(128_000), Ok(128_000));
        for bad in [0, 15_999, 1_000_001, u32::MAX] {
            assert!(clean_context_tokens(bad).unwrap_err().contains("16K 到 1M"), "{bad}");
        }
    }
}

#[cfg(test)]
mod reply_preference_tests {
    use super::*;

    fn configured() -> AiConfig {
        let mut cfg = AiConfig {
            active_provider: "custom_openai".into(), protocol: "responses".into(),
            base_url: "https://api.openai.com/v1".into(), model: "gpt-6.1-sol".into(),
            ..AiConfig::default()
        };
        cfg.providers.push(ProviderConfig {
            id: cfg.active_provider.clone(), protocol: cfg.protocol.clone(), base_url: cfg.base_url.clone(),
            model: cfg.model.clone(), reply_preference: ReplyPreference::Deeper, ..ProviderConfig::default()
        });
        cfg
    }

    #[test]
    fn managed_models_and_gateway_keep_declared_levels_and_preferences() {
        let mut cfg = configured();
        let idx = cfg.providers.iter().position(|p| p.id == cfg.active_provider).unwrap();
        cfg.providers[idx].remote = true;
        cfg.providers[idx].provider_type = "free".into();
        assert_eq!(cfg.reply_preference(), ReplyPreference::Deeper);
        assert!(cfg.reasoning_levels().contains(&"max".into()));
        cfg.providers[idx].reasoning_efforts = Some(vec!["high".into()]);
        assert_eq!(cfg.reasoning_levels(), vec!["high"]);
        assert_eq!(clean_reply_preference(&cfg.providers[idx], &cfg.base_url, &cfg.model, &cfg.protocol, Some(ReplyPreference::Max)).unwrap_err(), "此模型未声明支持所选思考等级，请检查模型能力配置");
        cfg.active_provider = crate::ai_gateway::GATEWAY_ID.into();
        cfg.gateway_reasoning_efforts = vec!["low".into(), "high".into()];
        cfg.gateway_reply_preference = ReplyPreference::Deeper;
        let restored: AiConfig = serde_json::from_value(serde_json::to_value(cfg).unwrap()).unwrap();
        assert_eq!(restored.reply_preference(), ReplyPreference::Deeper);
        assert_eq!(ai_reasoning::body("responses", restored.reply_preference(), &restored.reasoning_levels())["reasoning"]["effort"], "high");
    }

    #[test]
    fn reply_preferences_survive_storage_and_old_files_keep_supplier_defaults() {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("xiaoban-reply-pref-{}-{nonce}", std::process::id()));
        let path = dir.join("ai.json");
        let cfg = configured();
        save(&path, &cfg).unwrap();
        assert_eq!(load(&path).unwrap().reply_preference(), ReplyPreference::Deeper);
        let mut old = serde_json::to_value(&cfg).unwrap();
        for provider in old["providers"].as_array_mut().unwrap() { provider.as_object_mut().unwrap().remove("replyPreference"); }
        std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        assert_eq!(load(&path).unwrap().reply_preference(), ReplyPreference::Default);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn switching_provider_or_model_does_not_reuse_another_preference() {
        let mut cfg = configured();
        assert_eq!(cfg.reply_preference(), ReplyPreference::Deeper);
        cfg.active_provider = "deepseek".into();
        assert_eq!(cfg.reply_preference(), ReplyPreference::Default);
        cfg.active_provider = crate::ai_gateway::GATEWAY_ID.into();
        assert_eq!(cfg.reply_preference(), ReplyPreference::Default);
        cfg.active_provider = "custom_openai".into();
        cfg.model = "some-other-model".into();
        assert_eq!(cfg.reply_preference(), ReplyPreference::Default);
    }

    #[test]
    fn proxy_preferences_work_and_unsupported_model_preferences_reset_on_save() {
        let p = configured().providers.pop().unwrap();
        assert_eq!(clean_reply_preference(&p, "https://gateway.example/v1", &p.model, "responses", Some(ReplyPreference::Deeper)).unwrap(), ReplyPreference::Deeper);
        assert_eq!(clean_reply_preference(&p, "https://gateway.example/v1", "unknown", "responses", None).unwrap(), ReplyPreference::Default);
        assert_eq!(clean_reply_preference(&p, &p.base_url, &p.model, &p.protocol, None).unwrap(), ReplyPreference::Deeper);
    }
}


#[cfg(test)]
mod output_budget_tests {
    use super::*;
    use agent_kernel::model::CapabilitySource;

    fn config() -> AiConfig {
        serde_json::from_value(serde_json::json!({
            "activeProvider": "custom-output", "protocol": "anthropic",
            "baseUrl": "https://model.example/v1", "model": "test-model",
            "providers": [{
                "id":"custom-output", "protocol":"anthropic",
                "baseUrl":"https://model.example/v1", "model":"test-model",
                "models":[{"id":"test-model","maxOutputTokens":65536, "contextLength":128000}]
            }]
        })).unwrap()
    }

    #[test]
    fn provider_metadata_changes_context_window_not_output_reservation() {
        let mut config = config();
        assert_eq!(config.model_capabilities().request_output_limit(), Some(65_536));
        assert_eq!(config.model_capabilities().context_window_tokens, Some(128_000));
        config.context_tokens = 256_000;
        assert_eq!(config.token_budget().context_limit, 128_000);
        assert_eq!(config.token_budget().trim_headroom, 22_784);
        config.context_tokens = 16_000;
        assert_eq!(config.model_capabilities().request_output_limit(), Some(65_536));
        assert_eq!(config.token_budget().context_limit, 16_000);
    }

    #[test]
    fn legacy_saved_manual_output_limits_are_ignored() {
        let mut value = serde_json::to_value(config()).unwrap();
        value["providers"][0]["maxOutputTokens"] = serde_json::json!(1024);
        let loaded: AiConfig = serde_json::from_value(value).unwrap();
        assert_eq!(loaded.model_capabilities().request_output_limit(), Some(65_536));
        let saved = serde_json::to_value(loaded).unwrap();
        assert!(saved["providers"][0].get("maxOutputTokens").is_none());
    }

    #[test]
    fn capability_does_not_leak_between_providers_or_models() {
        let mut config = config();
        config.active_provider = "different".into();
        assert_eq!(config.model_capabilities().output_source, CapabilitySource::ProtocolFallback);
        assert_eq!(config.model_capabilities().context_window_tokens, None);
        config.active_provider = "custom-output".into();
        config.model = "other".into();
        assert_eq!(config.model_capabilities().context_window_tokens, None);
        config.model = "test-model".into();
        assert_eq!(config.model_capabilities().output_source, CapabilitySource::ProviderMetadata);
        assert_eq!(config.model_capabilities().request_output_limit(), Some(65_536));
    }
}

#[cfg(test)]
mod reasoning_metadata_tests {
    use super::*;

    #[test]
    fn model_discovery_preserves_independent_reasoning_metadata() {
        let models = parse_model_details_json(r#"{"data":[{"id":"private-alias","reasoningSummary":false,"reasoningEfforts":[],"thinkingMode":{"type":"enabled","budget_tokens":2048},"max_output_tokens":8192}]}"#, "anthropic").unwrap();
        let model = &models[0];
        assert_eq!(model.reasoning_summary, Some(false));
        assert_eq!(model.reasoning_efforts, Some(vec![]));
        assert_eq!(model.thinking_mode, Some(ai_reasoning::ThinkingMode::Enabled { budget_tokens: 2048 }));
        let restored: ModelConfig = serde_json::from_value(serde_json::to_value(model).unwrap()).unwrap();
        assert_eq!(&restored, model);
        assert!(!model_facts_valid(&ModelConfig { thinking_mode: Some(ai_reasoning::ThinkingMode::Enabled { budget_tokens: 8192 }), ..model.clone() }));
    }
    #[test]
    fn alias_summary_capability_is_scoped_to_the_selected_endpoint() {
        let mut config = AiConfig::default();
        config.active_provider = "alias-provider".into();
        config.protocol = "responses".into();
        config.base_url = "https://gateway.example/v1".into();
        config.model = "private-alias".into();
        config.providers = vec![ProviderConfig {
            id: config.active_provider.clone(), protocol: config.protocol.clone(),
            base_url: config.base_url.clone(), model: config.model.clone(),
            models: vec![ModelConfig { id: config.model.clone(), reasoning_summary: Some(true), reasoning_efforts: Some(vec![]), ..Default::default() }],
            ..Default::default()
        }];
        assert!(config.reasoning_levels().is_empty());
        assert_eq!(config.reasoning_body()["reasoning"]["summary"], "auto");
        config.base_url = "https://other.example/v1".into();
        assert!(config.reasoning_body().is_empty());
    }

}

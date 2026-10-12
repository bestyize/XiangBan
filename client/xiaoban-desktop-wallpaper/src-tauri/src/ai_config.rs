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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AiConfig {
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
        Self { protocol: "chat".into(), base_url: String::new(), model: String::new(), api_key: String::new(), workspace: String::new(), permission_mode: "ask".into(), hide_tool_calls: false, open_computer_use: false }
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
        let key_hint = match self.api_key.chars().count() {
            0 => String::new(),
            n if n >= 12 => format!("••••{}", self.api_key.chars().skip(n - 4).collect::<String>()),
            _ => "••••".into(),
        };
        AiConfigView {
            protocol: self.protocol.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            has_key: !self.api_key.is_empty(),
            key_hint,
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
    Ok(AiConfig {
        protocol: input.protocol.to_string(),
        base_url: clean_base_url(input.base_url)?,
        model: clean_model(input.model)?,
        api_key,
        workspace: match input.workspace {
            Some(raw) => clean_workspace(raw)?,
            None => current.workspace.clone(),
        },
        permission_mode: match input.permission_mode {
            Some(mode) => {
                if !["ask", "auto"].contains(&mode) {
                    return Err("不支持的权限模式".into());
                }
                mode.to_string()
            }
            None => current.permission_mode.clone(),
        },
        hide_tool_calls: input.hide_tool_calls.unwrap_or(current.hide_tool_calls),
        open_computer_use: input.open_computer_use.unwrap_or(current.open_computer_use),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "sk-abcdefghijklmnop1234";

    fn saved() -> AiConfig {
        AiConfig { protocol: "chat".into(), base_url: "https://api.example.com/v1".into(), model: "m1".into(), api_key: KEY.into(), workspace: String::new(), permission_mode: "ask".into(), hide_tool_calls: false, open_computer_use: false }
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
        for command in ["get_ai_config", "save_ai_config", "pick_workspace"] {
            assert!(crate::content::permitted("control", command), "{command}");
            for label in ["pet", "wallpaper", "care", "games", "media-a", "media-preview", "ritual", "widgets"] {
                assert!(!crate::content::permitted(label, command), "{label} must not reach {command}");
            }
        }
    }
}

//! Each conversation works in a folder of its own, chosen on the chat page under the message box.
//!
//! A new conversation starts in the folder used last (the recent list in `agent-settings.json`), or the default
//! working folder from the settings the first time. A conversation made before this existed has no folder saved and
//! keeps working in the default one. Switching a conversation that already has messages adds a note for the model
//! (`SWITCH_MARK`), so it does not go on using paths from the old folder; the page shows that note as a notice.

use super::*;

/// Starts the note added to a conversation when its folder changes; the page shows such a note as a notice.
pub const SWITCH_MARK: &str = "【工作区已切换】";

/// A folder the user chose: a full path to a folder that exists.
pub fn checked(raw: &str) -> Result<String, String> {
    let text = raw.trim();
    if text.is_empty() || text.chars().count() > 1024 || text.chars().any(char::is_control) {
        return Err("文件夹路径不对".into());
    }
    let path = Path::new(text);
    if !path.is_absolute() {
        return Err("请选择一个完整路径的文件夹".into());
    }
    if !path.is_dir() {
        return Err(format!("找不到这个文件夹：{text}"));
    }
    Ok(text.to_string())
}

/// The folder a conversation works in.
pub fn of(summary: &SessionSummary, config: &AiConfig, default: &Path) -> PathBuf {
    if summary.workspace.is_empty() { config.workspace_or(default).to_path_buf() } else { PathBuf::from(&summary.workspace) }
}

/// Where a new conversation starts: the folder used last that still exists, else the default working folder.
pub fn for_new_conversation(app: &tauri::AppHandle) -> Result<String, String> {
    let settings = agent_settings::load(&Dirs::of(app)?.chat);
    if let Some(recent) = settings.recent_workspaces.iter().find(|p| Path::new(p).is_dir()) {
        return Ok(recent.clone());
    }
    let config = ai_config::current(app)?;
    let default = ai_config::default_workspace(app)?;
    Ok(config.workspace_or(&default).display().to_string())
}

pub fn remember(app: &tauri::AppHandle, folder: &str) {
    let Ok(dirs) = Dirs::of(app) else { return };
    let mut settings = agent_settings::load(&dirs.chat);
    settings.remember_workspace(folder);
    let _ = agent_settings::save(&dirs.chat, &settings);
}

/// One folder for the page's menu.
fn describe(path: &str) -> Value {
    let folder = Path::new(path);
    let name = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string());
    json!({ "path": path, "name": name, "exists": folder.is_dir(), "wide": danger::too_wide(folder) })
}

/// The recently used folders, newest first, for the folder menu.
pub fn recent(app: &tauri::AppHandle) -> Result<Vec<Value>, String> {
    let settings = agent_settings::load(&Dirs::of(app)?.chat);
    Ok(settings.recent_workspaces.iter().map(|p| describe(p)).collect())
}

/// The note the model reads after the folder changed.
pub fn switch_note(from: &Path, to: &Path) -> String {
    format!(
        "{SWITCH_MARK}从现在起这个对话在“{}”里工作（之前是“{}”）。前面提到的文件路径指的是旧文件夹，需要时先重新查看新文件夹里的内容。",
        to.display(),
        from.display()
    )
}

/// Moves a conversation to another folder. Refused while it is answering. A conversation with messages gets a note
/// for the model (see the module comment). Returns the conversation and whether the folder is too wide to be safe.
#[tauri::command]
pub fn chat_set_workspace(app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String, workspace: String) -> Result<Value, String> {
    if window.label() != "control" {
        return Err("请在聊天页切换工作区".into());
    }
    checked_session_id(&session_id)?;
    let folder = checked(&workspace)?;
    let state = app.state::<ChatState>();
    if lock(&state.runs).contains_key(&session_id) {
        return Err("这个对话还在回答，请等它结束或先停止，再切换工作区".into());
    }
    let summary = mutate(&app, |index| index.get(&session_id).cloned())?.ok_or("这个对话已经不存在")?;
    let config = ai_config::current(&app)?;
    let default = ai_config::default_workspace(&app)?;
    let before = of(&summary, &config, &default);
    let after = PathBuf::from(&folder);
    let dirs = Dirs::of(&app)?;
    if before != after {
        let store = JsonlStore::new(dirs.sessions());
        if !store.load(&session_id).map_err(|e| e.to_string())?.is_empty() {
            store.append(&session_id, &[Message::reminder(switch_note(&before, &after))]).map_err(|e| e.to_string())?;
        }
    }
    let summary = mutate(&app, |index| {
        index.set_workspace(&session_id, &folder);
        index.get(&session_id).cloned()
    })?
    .ok_or("这个对话已经不存在")?;
    remember(&app, &folder);
    if let Ok(sessions) = mutate(&app, |index| index.list()) {
        emit(&app, "chat-sessions-changed", json!(sessions));
    }
    Ok(json!({ "session": summary, "wide": danger::too_wide(&after), "recentWorkspaces": recent(&app)? }))
}

/// Shows a conversation's folder (or, without one, where a new conversation would start) in Finder or Explorer.
#[tauri::command]
pub fn chat_reveal_workspace(app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: Option<String>) -> Result<(), String> {
    if window.label() != "control" {
        return Err("请在聊天页打开工作区".into());
    }
    let config = ai_config::current(&app)?;
    let default = ai_config::default_workspace(&app)?;
    let folder = match session_id.as_deref() {
        Some(id) => {
            checked_session_id(id)?;
            let summary = mutate(&app, |index| index.get(id).cloned())?.ok_or("这个对话已经不存在")?;
            of(&summary, &config, &default)
        }
        None => PathBuf::from(for_new_conversation(&app)?),
    };
    if folder == default {
        fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    }
    crate::storage::show_in_file_manager(&folder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_old_conversation_keeps_the_default_folder_and_a_new_one_its_own() {
        let old: SessionSummary = serde_json::from_str(r#"{"id":"a","title":"旧对话","createdAt":1,"updatedAt":1,"permissionMode":"ask","group":""}"#).unwrap();
        assert_eq!(old.workspace, "", "saved before conversations had folders");
        let config = AiConfig { workspace: "/projects/default".into(), ..AiConfig::default() };
        assert_eq!(of(&old, &config, Path::new("/app/workspace")), PathBuf::from("/projects/default"));
        assert_eq!(of(&old, &AiConfig::default(), Path::new("/app/workspace")), PathBuf::from("/app/workspace"));
        let own = SessionSummary { workspace: "/projects/site".into(), ..old };
        assert_eq!(of(&own, &config, Path::new("/app/workspace")), PathBuf::from("/projects/site"));
    }

    #[test]
    fn a_chosen_folder_must_be_a_full_path_that_exists() {
        let dir = std::env::temp_dir();
        assert_eq!(checked(&format!("  {}  ", dir.display())).unwrap(), dir.display().to_string());
        assert!(checked("relative/folder").is_err());
        assert!(checked("/no/such/folder/xiaoban").unwrap_err().contains("找不到"));
        assert!(checked("").is_err());
    }

    #[test]
    fn recent_folders_are_newest_first_without_repeats_and_bounded() {
        let mut settings = agent_settings::AgentSettings::default();
        for i in 0..10 {
            settings.remember_workspace(&format!("/p/{i}"));
        }
        settings.remember_workspace("/p/5");
        assert_eq!(settings.recent_workspaces.len(), agent_settings::RECENT_WORKSPACES);
        assert_eq!(settings.recent_workspaces[0], "/p/5");
        assert_eq!(settings.recent_workspaces.iter().filter(|p| *p == "/p/5").count(), 1);
    }

    #[test]
    fn the_switch_note_names_both_folders_and_is_marked_for_the_page() {
        let note = switch_note(Path::new("/a"), Path::new("/b"));
        assert!(note.starts_with(SWITCH_MARK) && note.contains("“/b”") && note.contains("“/a”"));
        let wide = describe("/");
        assert_eq!(wide["wide"], true);
    }
}

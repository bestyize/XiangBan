//! 资料库 → 记忆: everything 小伴 keeps about the user, in one place they can read, correct and clear.
//! Notes are the assistant's own Markdown notes (also written by the user here), experiences are reviewed
//! lessons from earlier tasks (a candidate is used only once approved), and goals are the conversations' goals.
use super::Dirs;
use agent_kernel::goals::GoalStore;
use agent_kernel::memory::Memory;
use agent_learning::LearningDb;
use serde_json::{json, Value};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const SCOPE: &str = "personal";

fn require_control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "control" { return Err("请在主窗口的资料库里管理记忆".into()); }
    Ok(())
}

fn paused_marker(dirs: &Dirs) -> std::path::PathBuf { dirs.chat.join("memory-paused") }

/// Whether 小伴 may use and add to its memory. On unless the user switched it off.
pub(super) fn remembering(dirs: &Dirs) -> bool { !paused_marker(dirs).exists() }

fn notes(dirs: &Dirs) -> Memory { Memory::new(dirs.memory()) }

#[tauri::command]
pub fn chat_memory_overview(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    require_control(&window)?;
    let dirs = Dirs::of(&app)?;
    let memory = notes(&dirs);
    let notes: Vec<Value> = memory.list().into_iter().map(|note| {
        let text = memory.read(&note.name).unwrap_or_default();
        json!({ "name": note.name, "preview": note.preview, "text": text.trim_end(), "bytes": note.bytes })
    }).collect();
    let (mut candidates, mut approved) = (Vec::new(), Vec::new());
    for (experience, created) in LearningDb::open(&dirs.learning_db())?.list_experiences(SCOPE)? {
        let item = json!({ "id": experience.id, "kind": experience.kind, "content": experience.content, "evidence": experience.evidence, "createdMs": created * 1000 });
        if experience.state == "candidate" { candidates.push(item) } else { approved.push(item) }
    }
    let goal_dir = dirs.chat.join("goals");
    let store = GoalStore::new(&goal_dir);
    let mut goals: Vec<(u64, Value)> = fs::read_dir(&goal_dir).map(|read| read.flatten().filter_map(|entry| {
        let id = entry.file_name().to_str()?.strip_suffix(".json")?.to_string();
        let record = store.load(&id).ok()??;
        Some((record.updated_at_ms, json!({ "sessionId": record.session_id, "outcome": record.contract.outcome, "state": record.state, "updatedAtMs": record.updated_at_ms })))
    }).collect()).unwrap_or_default();
    goals.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(json!({
        "remembering": remembering(&dirs),
        "notes": notes, "candidates": candidates, "approved": approved,
        "goals": goals.into_iter().map(|(_, goal)| goal).collect::<Vec<_>>(),
    }))
}

#[tauri::command]
pub fn chat_memory_set_remembering(app: tauri::AppHandle, window: tauri::WebviewWindow, enabled: bool) -> Result<(), String> {
    require_control(&window)?;
    let dirs = Dirs::of(&app)?;
    let marker = paused_marker(&dirs);
    if enabled {
        if marker.exists() { fs::remove_file(&marker).map_err(|e| e.to_string())?; }
    } else {
        fs::create_dir_all(&dirs.chat).map_err(|e| e.to_string())?;
        fs::write(&marker, b"").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn chat_memory_edit_experience(app: tauri::AppHandle, window: tauri::WebviewWindow, id: i64, content: String) -> Result<(), String> {
    require_control(&window)?;
    let content = content.trim();
    if content.is_empty() || content.chars().count() > 500 { return Err("一条记忆需要 1 到 500 个字".into()); }
    LearningDb::open(&Dirs::of(&app)?.learning_db())?.update_experience(id, content)
}

#[tauri::command]
pub fn chat_memory_forget_experience(app: tauri::AppHandle, window: tauri::WebviewWindow, id: i64) -> Result<(), String> {
    require_control(&window)?;
    LearningDb::open(&Dirs::of(&app)?.learning_db())?.delete_experience(id)
}

/// Writes a note. Without a name a new one is made, so a note the user writes is kept like the assistant's own.
#[tauri::command]
pub fn chat_memory_write_note(app: tauri::AppHandle, window: tauri::WebviewWindow, name: Option<String>, text: String) -> Result<String, String> {
    require_control(&window)?;
    if text.trim().is_empty() || text.chars().count() > 2000 { return Err("一条记忆需要 1 到 2000 个字".into()); }
    let name = match name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()) {
        Some(name) => name,
        None => format!("note-{}", SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or_default()),
    };
    notes(&Dirs::of(&app)?).write(&name, &text).map_err(|e| if e.contains("limit") { "记忆最多 100 条，请先删掉一些".to_string() } else { e })?;
    Ok(name)
}

#[tauri::command]
pub fn chat_memory_delete_note(app: tauri::AppHandle, window: tauri::WebviewWindow, name: String) -> Result<(), String> {
    require_control(&window)?;
    notes(&Dirs::of(&app)?).remove(&name)
}

/// Forgets every note and experience. Goals belong to their conversations and stay.
#[tauri::command]
pub fn chat_memory_clear(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
    require_control(&window)?;
    let dirs = Dirs::of(&app)?;
    let memory = notes(&dirs);
    for note in memory.list() { memory.remove(&note.name)?; }
    LearningDb::open(&dirs.learning_db())?.clear_experiences(SCOPE)
}

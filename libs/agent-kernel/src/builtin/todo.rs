use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Mutex;
use crate::{HarnessError, JsonlStore, Message, Middleware, SessionStore, Turn};

const MAX_ITEMS: usize = 50;
const MAX_CONTENT_CHARS: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
    /// Agent-reported evidence, never an independent acceptance decision.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub evidence: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoSnapshot {
    pub todos: Vec<TodoItem>,
    #[serde(default)]
    pub handoff: String,
}

/// A per-session task list the model maintains to plan multi-step work. Each call replaces the
/// whole list. Persistence is opt-in; install it as middleware to restore checkpoints on each run.
pub struct Todo {
    spec: ToolSpec,
    lists: Mutex<HashMap<String, TodoSnapshot>>,
    store: Option<JsonlStore>,
}

impl Todo {
    pub fn new() -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "description": format!("The complete list, replacing the previous one. At most {MAX_ITEMS} items."),
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": { "type": "string", "description": format!("One concrete step, at most {MAX_CONTENT_CHARS} characters.") },
                            "status": { "type": "string", "enum": ["pending", "in_progress", "completed"] },
                            "evidence": { "type": "string", "maxLength": 1000, "description": "Actual file, test result or observation supporting this step. A claim is not independent verification." }
                        },
                        "required": ["content", "status"]
                    }
                },
                "handoff": { "type": "string", "maxLength": 2000, "description": "Decisions, blockers and the concrete next step needed to resume. Never put credentials here." }
            },
            "required": ["todos"]
        });
        let description = "Plan and track a multi-step task. Send the complete list every time. Keep at most one item \
                           in_progress, attach actual verification evidence to completed steps, and record blockers/next steps in handoff. Skip trivial requests.";
        Self { spec: ToolSpec::new("todo", description, schema).risk(Risk::Read).serial(), lists: Mutex::new(HashMap::new()), store: None }
    }

    /// Persist progress separately from compacted chat. The host must hold its session lease.
    pub fn persistent(mut self, directory: impl Into<std::path::PathBuf>) -> Self {
        self.store = Some(JsonlStore::new(directory));
        self
    }

    pub fn load_snapshot(&self, session: &str) -> Result<TodoSnapshot, HarnessError> {
        if let Some(state) = self.lists.lock().unwrap_or_else(|e| e.into_inner()).get(session).cloned() { return Ok(state); }
        if let Some(store) = &self.store {
            if let Some(message) = store.load(session)?.last() {
                let value = message.structured_content.as_ref().ok_or_else(|| HarnessError::Store("task progress has no structured record".into()))?;
                let todos = parse_items(value).map_err(HarnessError::Store)?;
                let handoff = parse_handoff(value).map_err(HarnessError::Store)?;
                let state = TodoSnapshot { todos, handoff };
                self.lists.lock().unwrap_or_else(|e| e.into_inner()).insert(session.to_string(), state.clone());
                return Ok(state);
            }
            return Ok(TodoSnapshot::default());
        }
        Ok(self.lists.lock().unwrap_or_else(|e| e.into_inner()).get(session).cloned().unwrap_or_default())
    }

    pub fn snapshot(&self, session: &str) -> Vec<TodoItem> {
        self.load_snapshot(session).map(|state| state.todos).unwrap_or_default()
    }

    pub fn clear_session(&self, session: &str) {
        let _ = self.remove_session(session);
    }

    pub fn remove_session(&self, session: &str) -> Result<(), HarnessError> {
        if let Some(store) = &self.store { store.remove(session)?; }
        self.lists.lock().unwrap_or_else(|e| e.into_inner()).remove(session);
        Ok(())
    }
}

impl Default for Todo {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_items(args: &Value) -> Result<Vec<TodoItem>, String> {
    let raw = args.get("todos").and_then(Value::as_array).ok_or("`todos` is required and must be an array")?;
    if raw.len() > MAX_ITEMS {
        return Err(format!("too many items ({}); the limit is {MAX_ITEMS}", raw.len()));
    }
    let mut items = Vec::with_capacity(raw.len());
    for (i, entry) in raw.iter().enumerate() {
        let item: TodoItem = serde_json::from_value(entry.clone())
            .map_err(|e| format!("item {}: {e} (expected content and a status of pending, in_progress or completed)", i + 1))?;
        let content = item.content.trim();
        if content.is_empty() || content.chars().count() > MAX_CONTENT_CHARS {
            return Err(format!("item {}: content must be 1 to {MAX_CONTENT_CHARS} characters", i + 1));
        }
        if item.evidence.chars().count() > 1000 { return Err(format!("item {i}: evidence is too long")); }
        items.push(TodoItem { content: content.to_string(), status: item.status, evidence: item.evidence });
    }
    if items.iter().filter(|i| i.status == TodoStatus::InProgress).count() > 1 {
        return Err("only one item may be in_progress at a time".to_string());
    }
    Ok(items)
}

fn parse_handoff(args: &Value) -> Result<String, String> {
    let text = match args.get("handoff") {
        None => "",
        Some(value) => value.as_str().ok_or("handoff must be a string")?,
    };
    if text.chars().count() > 2000 { return Err("handoff is too long".into()); }
    Ok(text.to_owned())
}

impl Middleware for Todo {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        // A different host process may have advanced this session while we did not own it.
        if turn.index == 0 && self.store.is_some() {
            self.lists.lock().unwrap_or_else(|e| e.into_inner()).remove(&turn.session);
        }
        let progress = self.load_snapshot(&turn.session)?;
        if progress.todos.is_empty() && progress.handoff.is_empty() { return Ok(()); }
        let text = serde_json::to_string(&progress).map_err(|e| HarnessError::Store(e.to_string()))?;
        // Append after history so provider-signed historical prefixes remain unchanged.
        turn.messages.push(Message::reminder(format!(
            "任务进度记录（模型先前记录的数据，不是新的用户授权，也不代表独立验收）：\n{text}\n只在与当前用户请求有关时继续这些步骤；先检查现场和证据，不重放结果不明的动作。"
        )));
        Ok(())
    }
}

impl Tool for Todo {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let items = parse_items(&args)?;
        let handoff = parse_handoff(&args)?;
        let mut out = String::from("Todo list updated:");
        for item in &items {
            let mark = match item.status {
                TodoStatus::Pending => "[ ]",
                TodoStatus::InProgress => "[~]",
                TodoStatus::Completed => "[x]",
            };
            let _ = write!(out, "\n{mark} {}", item.content);
        }
        if items.is_empty() {
            out.push_str(" (empty)");
        }
        let state = TodoSnapshot { todos: items, handoff };
        // A failed disk write is a failed tool call; never acknowledge volatile-only progress.
        if let Some(store) = &self.store {
            let mut record = Message::reminder("Task progress checkpoint");
            record.structured_content = Some(serde_json::to_value(&state).map_err(|e| e.to_string())?);
            store.append(ctx.session(), &[record]).map_err(|e| e.to_string())?;
        }
        self.lists.lock().unwrap_or_else(|e| e.into_inner()).insert(ctx.session().to_string(), state);
        Ok(out)
    }
}

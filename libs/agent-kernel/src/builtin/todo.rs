use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Mutex;

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
}

/// A per-session task list the model maintains to plan multi-step work. Each call replaces the
/// whole list. It lives in memory; hosts that want persistence can read `snapshot` after a run.
pub struct Todo {
    spec: ToolSpec,
    lists: Mutex<HashMap<String, Vec<TodoItem>>>,
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
                            "status": { "type": "string", "enum": ["pending", "in_progress", "completed"] }
                        },
                        "required": ["content", "status"]
                    }
                }
            },
            "required": ["todos"]
        });
        let description = "Plan and track a multi-step task. Send the complete list every time. Keep at most one item \
                           in_progress, mark items completed as soon as they are done, and skip this tool for trivial requests.";
        Self { spec: ToolSpec::new("todo", description, schema).risk(Risk::Read), lists: Mutex::new(HashMap::new()) }
    }

    pub fn snapshot(&self, session: &str) -> Vec<TodoItem> {
        self.lists.lock().unwrap_or_else(|e| e.into_inner()).get(session).cloned().unwrap_or_default()
    }

    pub fn clear_session(&self, session: &str) {
        self.lists.lock().unwrap_or_else(|e| e.into_inner()).remove(session);
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
        items.push(TodoItem { content: content.to_string(), status: item.status });
    }
    if items.iter().filter(|i| i.status == TodoStatus::InProgress).count() > 1 {
        return Err("only one item may be in_progress at a time".to_string());
    }
    Ok(items)
}

impl Tool for Todo {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let items = parse_items(&args)?;
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
        self.lists.lock().unwrap_or_else(|e| e.into_inner()).insert(ctx.session().to_string(), items);
        Ok(out)
    }
}

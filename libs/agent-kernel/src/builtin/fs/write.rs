use super::{looks_binary, Roots};
use crate::builtin::str_arg;
use crate::error::HarnessError;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use crate::util::atomic_write;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

const MAX_WRITE_BYTES: usize = 1024 * 1024;
const MAX_EDIT_FILE_BYTES: u64 = 2 * 1024 * 1024;

/// `write_file` and `edit_file`, limited to the same kind of roots as `fs_tools`. Both are
/// `Risk::Write`, so `Permission` decides whether they run. Missing parent directories are created.
pub fn fs_write_tools(roots: Vec<PathBuf>) -> Result<Vec<Arc<dyn Tool>>, HarnessError> {
    let roots = Roots::open(roots)?;
    Ok(vec![Arc::new(WriteFile::new(roots.clone())), Arc::new(EditFile::new(roots))])
}

struct WriteFile {
    spec: ToolSpec,
    roots: Arc<Roots>,
}

impl WriteFile {
    fn new(roots: Arc<Roots>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": { "path": { "type": "string" }, "content": { "type": "string" } },
            "required": ["path", "content"]
        });
        let description = "Create a file, or replace an existing file completely, with the given text. To change part of an \
                           existing file use edit_file instead.";
        Self { spec: ToolSpec::new("write_file", description, schema).risk(Risk::Write).serial(), roots }
    }
}

impl Tool for WriteFile {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        let content = str_arg(&args, "content")?;
        if content.len() > MAX_WRITE_BYTES {
            return Err(format!("content is larger than {} KiB", MAX_WRITE_BYTES / 1024));
        }
        let (path, existed) = self.roots.resolve_new(str_arg(&args, "path")?)?;
        if existed && !fs::metadata(&path).map_err(|e| e.to_string())?.is_file() {
            return Err("that path is not a regular file".to_string());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("cannot create directory: {e}"))?;
        }
        atomic_write(&path, content.as_bytes()).map_err(|e| format!("cannot write file: {e}"))?;
        Ok(format!("{} {} ({} bytes).", if existed { "Replaced" } else { "Created" }, self.roots.display(&path), content.len()))
    }
}

struct EditFile {
    spec: ToolSpec,
    roots: Arc<Roots>,
}

impl EditFile {
    fn new(roots: Arc<Roots>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "old_string": { "type": "string", "description": "Exact text to replace, including whitespace. Must occur exactly once unless replace_all is true." },
                "new_string": { "type": "string" },
                "replace_all": { "type": "boolean" }
            },
            "required": ["path", "old_string", "new_string"]
        });
        let description = "Replace exact text in an existing text file. Read the file first so old_string matches exactly.";
        Self { spec: ToolSpec::new("edit_file", description, schema).risk(Risk::Write).serial(), roots }
    }
}

impl Tool for EditFile {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        let old = str_arg(&args, "old_string")?;
        let new = str_arg(&args, "new_string")?;
        let replace_all = args.get("replace_all").and_then(Value::as_bool).unwrap_or(false);
        if old.is_empty() {
            return Err("`old_string` must not be empty".to_string());
        }
        if old == new {
            return Err("`old_string` and `new_string` are identical".to_string());
        }
        let path = self.roots.resolve(str_arg(&args, "path")?)?;
        let meta = fs::metadata(&path).map_err(|e| format!("cannot access file: {e}"))?;
        if !meta.is_file() {
            return Err("that path is not a regular file".to_string());
        }
        if meta.len() > MAX_EDIT_FILE_BYTES {
            return Err(format!("the file is larger than {} MiB", MAX_EDIT_FILE_BYTES / 1024 / 1024));
        }
        let bytes = fs::read(&path).map_err(|e| format!("cannot read file: {e}"))?;
        if looks_binary(&bytes) {
            return Err("this looks like a binary file".to_string());
        }
        let text = String::from_utf8(bytes).map_err(|_| "the file is not valid UTF-8".to_string())?;
        let count = text.matches(old).count();
        match (count, replace_all) {
            (0, _) => return Err("`old_string` was not found; read the file again and copy the text exactly".to_string()),
            (n, false) if n > 1 => return Err(format!("`old_string` occurs {n} times; add surrounding lines to make it unique, or set replace_all")),
            _ => {}
        }
        let updated = if replace_all { text.replace(old, new) } else { text.replacen(old, new, 1) };
        if updated.len() > MAX_WRITE_BYTES * 2 {
            return Err("the edited file would be too large".to_string());
        }
        atomic_write(&path, updated.as_bytes()).map_err(|e| format!("cannot write file: {e}"))?;
        Ok(format!("Edited {} ({count} replacement{}).", self.roots.display(&path), if count == 1 { "" } else { "s" }))
    }
}

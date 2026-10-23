use super::{looks_binary, Roots};
use crate::builtin::str_arg;
use crate::error::HarnessError;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use crate::util::atomic_write;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use sha2::{Digest, Sha256};

const MAX_WRITE_BYTES: usize = 1024 * 1024;
const MAX_EDIT_FILE_BYTES: u64 = 2 * 1024 * 1024;

type PathLocks = std::collections::HashMap<PathBuf, Weak<Mutex<()>>>;
static PATH_LOCKS: OnceLock<Mutex<PathLocks>> = OnceLock::new();

fn path_lock(path: &Path) -> Arc<Mutex<()>> {
    let mut locks = PATH_LOCKS.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    locks.retain(|_, lock| lock.strong_count() != 0);
    if let Some(lock) = locks.get(path).and_then(Weak::upgrade) { return lock; }
    let lock = Arc::new(Mutex::new(()));
    locks.insert(path.into(), Arc::downgrade(&lock));
    lock
}

fn check_version(path: &Path, args: &Value) -> Result<(), String> {
    let Some(expected) = args.get("expected_sha256") else { return Ok(()); };
    let expected = expected.as_str().ok_or("expected_sha256 must be a SHA256 or 'missing'")?;
    match fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && expected == "missing" => return Ok(()),
        Ok(_) if expected == "missing" => return Err("file version conflict; the file already exists".into()),
        Ok(meta) if meta.len() > MAX_EDIT_FILE_BYTES => return Err("file is too large for version-checked text writes".into()),
        Err(e) => return Err(format!("cannot verify current file version: {e}")),
        _ => {}
    }
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && expected == "missing" => return Ok(()),
        Err(e) => return Err(format!("cannot verify current file version: {e}")),
    };
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if expected.len() != 64 || !actual.eq_ignore_ascii_case(expected) {
        return Err("file version conflict; read the file again before changing it".into());
    }
    Ok(())
}

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
            "properties": { "path": { "type": "string" }, "content": { "type": "string" },
                "expected_sha256": { "type": "string", "description": "Version returned by read_file; use missing when creating a new file. A stale version rejects the write." } },
            "required": ["path", "content"]
        });
        let description = "Create a file, or replace an existing file completely, with the given text. To change part of an \
                           existing file use edit_file instead.";
        Self { spec: ToolSpec::new("write_file", description, schema).risk(Risk::Write).serial(), roots }
    }
}

impl Tool for WriteFile {
    fn resource(&self, args: &Value) -> Result<Option<crate::ToolResource>, String> {
        Ok(Some(crate::ToolResource::File(self.roots.resolve_new(str_arg(args, "path")?)?.0)))
    }
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let content = str_arg(&args, "content")?;
        if content.len() > MAX_WRITE_BYTES {
            return Err(format!("content is larger than {} KiB", MAX_WRITE_BYTES / 1024));
        }
        let (path, existed) = self.roots.resolve_new(str_arg(&args, "path")?)?;
        let lock = path_lock(&path);
        let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        if ctx.is_cancelled() { return Err("cancelled before file write".into()); }
        check_version(&path, &args)?;
        if existed && !fs::metadata(&path).map_err(|e| e.to_string())?.is_file() {
            return Err("that path is not a regular file".to_string());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("cannot create directory: {e}"))?;
        }
        if ctx.is_cancelled() { return Err("cancelled before file commit".into()); }
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
                "replace_all": { "type": "boolean" },
                "expected_sha256": { "type": "string", "description": "Version returned by read_file; stale input is rejected." }
            },
            "required": ["path", "old_string", "new_string"]
        });
        let description = "Replace exact text in an existing text file. Read the file first so old_string matches exactly.";
        Self { spec: ToolSpec::new("edit_file", description, schema).risk(Risk::Write).serial(), roots }
    }
}

impl Tool for EditFile {
    fn resource(&self, args: &Value) -> Result<Option<crate::ToolResource>, String> {
        Ok(Some(crate::ToolResource::File(self.roots.resolve(str_arg(args, "path")?)?)))
    }
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
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
        let lock = path_lock(&path);
        let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        if ctx.is_cancelled() { return Err("cancelled before file edit".into()); }
        check_version(&path, &args)?;
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
        // A GBK/GB18030 file is edited as text and written back in its own encoding, not silently turned into UTF-8.
        let original = bytes.clone();
        let (text, gb18030) = match String::from_utf8(bytes) {
            Ok(text) => (text, false),
            Err(error) => match encoding_rs::GB18030.decode_without_bom_handling_and_without_replacement(error.as_bytes()) {
                Some(text) => (text.into_owned(), true),
                None => return Err("the file is neither UTF-8 nor GBK/GB18030 text; it cannot be edited safely".to_string()),
            },
        };
        // `read_file` shows lines without their `\r`, so on a Windows (CRLF) file a multi-line `old_string` written
        // from what the model saw never matches. Such a file is edited with plain newlines and written back as CRLF.
        let crlf = !old.contains('\r') && old.contains('\n') && text.contains("\r\n") && !text.replace("\r\n", "").contains('\n');
        let (text, old, new) = if crlf {
            (text.replace("\r\n", "\n"), old.to_string(), new.replace("\r\n", "\n"))
        } else {
            (text, old.to_string(), new.to_string())
        };
        let (old, new) = (old.as_str(), new.as_str());
        let count = text.matches(old).count();
        match (count, replace_all) {
            (0, _) => return Err("`old_string` was not found; read the file again and copy the text exactly".to_string()),
            (n, false) if n > 1 => return Err(format!("`old_string` occurs {n} times; add surrounding lines to make it unique, or set replace_all")),
            _ => {}
        }
        let updated = if replace_all { text.replace(old, new) } else { text.replacen(old, new, 1) };
        let updated = if crlf { updated.replace('\n', "\r\n") } else { updated };
        if updated.len() > MAX_WRITE_BYTES * 2 {
            return Err("the edited file would be too large".to_string());
        }
        let bytes = if gb18030 { encoding_rs::GB18030.encode(&updated).0.into_owned() } else { updated.into_bytes() };
        if ctx.is_cancelled() { return Err("cancelled before file commit".into()); }
        if fs::read(&path).map_err(|e| e.to_string())? != original {
            return Err("file changed during editing; read it again before retrying".into());
        }
        atomic_write(&path, &bytes).map_err(|e| format!("cannot write file: {e}"))?;
        let kept = if gb18030 { " The file stays GBK/GB18030." } else { "" };
        Ok(format!("Edited {} ({count} replacement{}).{kept}", self.roots.display(&path), if count == 1 { "" } else { "s" }))
    }
}

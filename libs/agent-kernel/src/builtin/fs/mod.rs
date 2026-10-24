//! File tools confined to directories the host declares: `fs_tools` reads, `fs_write_tools` changes.

mod write;

pub use write::fs_write_tools;

use super::{opt_str_arg, opt_uint_arg, str_arg};
use crate::error::HarnessError;
use crate::tool::{Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use crate::util::nearest_boundary;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

const SKIP_DIRS: [&str; 3] = [".git", "node_modules", "target"];
const BINARY_SNIFF: usize = 8000;
const READ_SCAN_LIMIT: u64 = 64 * 1024 * 1024;
const READ_OUTPUT_LIMIT: usize = 40_000;
const READ_DEFAULT_LINES: u64 = 2000;
const READ_MAX_LINES: u64 = 5000;
const LINE_CHAR_LIMIT: usize = 2000;
const LIST_LIMIT: usize = 500;
const GREP_DEFAULT_RESULTS: u64 = 100;
const GREP_MAX_RESULTS: u64 = 500;
const GREP_FILE_LIMIT: usize = 20_000;
const GREP_FILE_BYTES: u64 = 2 * 1024 * 1024;
const GREP_MAX_DEPTH: usize = 20;
const GREP_LINE_CHARS: usize = 300;

/// One line of a file as text. UTF-8 first; otherwise GB18030 (a superset of GBK and GB2312, still common for
/// Chinese text saved on Windows) when it decodes without errors. Its multi-byte sequences never contain a newline
/// byte, so decoding line by line is safe. `None` when neither fits.
pub(crate) fn decode_line(bytes: &[u8]) -> (std::borrow::Cow<'_, str>, LineEncoding) {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return (text.into(), LineEncoding::Utf8);
    }
    match encoding_rs::GB18030.decode_without_bom_handling_and_without_replacement(bytes) {
        Some(text) => (text, LineEncoding::Gb18030),
        None => (String::from_utf8_lossy(bytes), LineEncoding::Unknown),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineEncoding {
    Utf8,
    Gb18030,
    /// Neither: shown with replacement characters.
    Unknown,
}

struct Roots {
    list: Vec<PathBuf>,
}

impl Roots {
    fn check_inside(&self, real: PathBuf, requested: &str) -> Result<PathBuf, String> {
        if self.list.iter().any(|root| real.starts_with(root)) {
            return Ok(real);
        }
        let allowed: Vec<String> = self.list.iter().map(|r| r.display().to_string()).collect();
        Err(format!("`{requested}` is outside the allowed directories: {}", allowed.join(", ")))
    }

    fn join(&self, requested: &str) -> PathBuf {
        let path = Path::new(requested);
        if path.is_absolute() { path.to_path_buf() } else { self.list[0].join(path) }
    }

    /// Resolves to a canonical path that must lie inside a root. Symlinks that point outside fail.
    fn resolve(&self, requested: &str) -> Result<PathBuf, String> {
        let real = self.join(requested).canonicalize().map_err(|e| format!("cannot access `{requested}`: {e}"))?;
        self.check_inside(real, requested)
    }

    /// Like `resolve`, but the file (and some parent directories) may not exist yet. Returns the
    /// final path and whether it already existed. `..` is rejected outright, and a dangling
    /// symlink counts as existing so it can never be written through.
    fn resolve_new(&self, requested: &str) -> Result<(PathBuf, bool), String> {
        if Path::new(requested).components().any(|c| matches!(c, Component::ParentDir)) {
            return Err("`..` is not allowed in paths that are written".to_string());
        }
        let joined = self.join(requested);
        if fs::symlink_metadata(&joined).is_ok() {
            return Ok((self.check_inside(joined.canonicalize().map_err(|e| format!("cannot access `{requested}`: {e}"))?, requested)?, true));
        }
        let mut tail = Vec::new();
        let mut ancestor = joined.as_path();
        while fs::symlink_metadata(ancestor).is_err() {
            tail.push(ancestor.file_name().ok_or_else(|| format!("`{requested}` is not a valid path"))?);
            ancestor = ancestor.parent().ok_or_else(|| format!("`{requested}` is not a valid path"))?;
        }
        let mut real = ancestor.canonicalize().map_err(|e| format!("cannot access `{requested}`: {e}"))?;
        real.extend(tail.iter().rev());
        Ok((self.check_inside(real, requested)?, false))
    }

    /// Relative to the first root when possible, so the model can pass the path straight back.
    fn display(&self, path: &Path) -> String {
        path.strip_prefix(&self.list[0]).unwrap_or(path).display().to_string()
    }

    fn open(roots: Vec<PathBuf>) -> Result<Arc<Self>, HarnessError> {
        if roots.is_empty() {
            return Err(HarnessError::Config("at least one root directory is required".into()));
        }
        let mut list = Vec::with_capacity(roots.len());
        for root in &roots {
            let real = root.canonicalize().map_err(|e| HarnessError::Config(format!("root {}: {e}", root.display())))?;
            if !real.is_dir() {
                return Err(HarnessError::Config(format!("root {} is not a directory", root.display())));
            }
            list.push(real);
        }
        Ok(Arc::new(Self { list }))
    }
}

/// `read_file`, `list_dir` and `grep`, limited to `roots`. Relative paths resolve against the
/// first root. Add the directory used by `SpillOutput` as a root so the model can read spilled output.
pub fn fs_tools(roots: Vec<PathBuf>) -> Result<Vec<Arc<dyn Tool>>, HarnessError> {
    let roots = Roots::open(roots)?;
    Ok(vec![Arc::new(ReadFile::new(roots.clone())), Arc::new(ListDir::new(roots.clone())), Arc::new(Grep::new(roots))])
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(BINARY_SNIFF).any(|&b| b == 0)
}

fn clip_chars(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text.to_string(),
    }
}

struct ReadFile {
    spec: ToolSpec,
    roots: Arc<Roots>,
}

impl ReadFile {
    fn new(roots: Arc<Roots>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "offset": { "type": "integer", "description": "First line to read, starting at 1." },
                "limit": { "type": "integer", "description": "Maximum number of lines." }
            },
            "required": ["path"]
        });
        let description = "Read a text file with line numbers. Long files are returned in parts; the result \
                           says which offset to use for the next part.";
        Self { spec: ToolSpec::new("read_file", description, schema).risk(Risk::Read).parallel().max_output(READ_OUTPUT_LIMIT * 2), roots }
    }
}

impl Tool for ReadFile {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        use sha2::{Digest, Sha256};
        // Version evidence for editable files must describe the same bytes the model
        // read. Do not attach a hash from a later, independently changed version.
        let snapshot = str_arg(&args, "path").ok().and_then(|requested| self.roots.resolve(requested).ok())
            .filter(|path| fs::metadata(path).is_ok_and(|m| m.is_file() && m.len() <= 2 * 1024 * 1024))
            .and_then(|path| fs::read(&path).ok().map(|bytes| (path, bytes)));
        match self.execute(args, ctx) {
            Err(error) => ToolResult::error(error),
            Ok(text) => {
                if let Some((path, before)) = snapshot {
                    if fs::read(&path).ok().as_deref() != Some(before.as_slice()) {
                        return ToolResult::error("file changed while reading; read it again before editing");
                    }
                    ToolResult::ok(text).with_structured_content(json!({
                        "path": self.roots.display(&path), "sha256": format!("{:x}", Sha256::digest(&before))
                    }))
                } else { ToolResult::ok(text) }
            }
        }
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let path = self.roots.resolve(str_arg(&args, "path")?)?;
        let offset = opt_uint_arg(&args, "offset")?.unwrap_or(1).max(1);
        let limit = opt_uint_arg(&args, "limit")?.unwrap_or(READ_DEFAULT_LINES).clamp(1, READ_MAX_LINES);
        // Windows cannot open directories as ordinary files; return the same
        // actionable tool error before File::open on every platform.
        if fs::metadata(&path).map_err(|e| format!("cannot access file: {e}"))?.is_dir() {
            return Err("this is a directory; use list_dir".into());
        }
        let file = File::open(&path).map_err(|e| format!("cannot open file: {e}"))?;
        if file.metadata().map_err(|e| e.to_string())?.is_dir() {
            return Err("this is a directory; use list_dir".to_string());
        }
        let mut reader = BufReader::new(file.take(READ_SCAN_LIMIT));
        if looks_binary(reader.fill_buf().map_err(|e| e.to_string())?) {
            return Err("this looks like a binary file".to_string());
        }

        let mut out = String::new();
        let (mut number, mut emitted) = (0u64, 0u64);
        let mut line = Vec::new();
        let mut more_from = None;
        let (mut gb18030, mut undecodable) = (false, false);
        loop {
            if ctx.is_cancelled() {
                return Err("cancelled".to_string());
            }
            line.clear();
            if reader.read_until(b'\n', &mut line).map_err(|e| e.to_string())? == 0 {
                break;
            }
            number += 1;
            if number < offset {
                continue;
            }
            if emitted >= limit || out.len() >= READ_OUTPUT_LIMIT {
                more_from = Some(number);
                break;
            }
            let (text, encoding) = decode_line(&line);
            gb18030 |= encoding == LineEncoding::Gb18030;
            undecodable |= encoding == LineEncoding::Unknown;
            out.push_str(&format!("{number:>6}\t{}\n", clip_chars(text.trim_end_matches(['\n', '\r']), LINE_CHAR_LIMIT)));
            emitted += 1;
        }
        if emitted == 0 {
            return Ok(if number == 0 { "(empty file)".to_string() } else { format!("(no lines at offset {offset}; the file has {number})") });
        }
        if let Some(next) = more_from {
            out.push_str(&format!("[more lines follow; continue with offset={next}]"));
        }
        if undecodable {
            out.push_str("\n[encoding: some lines are neither UTF-8 nor GBK/GB18030 and were shown with \u{FFFD} in place of what could not be decoded; do not copy those characters into edits or claim to know them]");
        } else if gb18030 {
            out.push_str("\n[encoding: this file is GBK/GB18030, shown decoded; edit_file keeps that encoding when it writes]");
        }
        Ok(out)
    }
}

struct ListDir {
    spec: ToolSpec,
    roots: Arc<Roots>,
}

impl ListDir {
    fn new(roots: Arc<Roots>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": { "path": { "type": "string", "description": "Directory to list. Defaults to the first allowed root." } }
        });
        let spec = ToolSpec::new("list_dir", "List the entries of a directory. Directories end with /.", schema).risk(Risk::Read).parallel();
        Self { spec, roots }
    }
}

impl Tool for ListDir {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        let path = self.roots.resolve(opt_str_arg(&args, "path")?.unwrap_or("."))?;
        let mut entries = Vec::new();
        for entry in fs::read_dir(&path).map_err(|e| format!("cannot list directory: {e}"))?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let line = match entry.metadata() {
                Ok(meta) if meta.is_dir() => format!("{name}/"),
                Ok(meta) if entry.file_type().is_ok_and(|t| t.is_symlink()) => format!("{name}@  ({} bytes)", meta.len()),
                Ok(meta) => format!("{name}  ({} bytes)", meta.len()),
                Err(_) => format!("{name}  (unreadable)"),
            };
            entries.push((name, line));
        }
        entries.sort();
        if entries.is_empty() {
            return Ok("(empty directory)".to_string());
        }
        let total = entries.len();
        let mut out: Vec<String> = entries.into_iter().take(LIST_LIMIT).map(|(_, line)| line).collect();
        if total > LIST_LIMIT {
            out.push(format!("[... {} more entries not shown]", total - LIST_LIMIT));
        }
        Ok(out.join("\n"))
    }
}

struct Grep {
    spec: ToolSpec,
    roots: Arc<Roots>,
}

impl Grep {
    fn new(roots: Arc<Roots>) -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Literal text to find (not a regular expression)." },
                "path": { "type": "string", "description": "File or directory to search. Defaults to the first allowed root." },
                "ignore_case": { "type": "boolean" },
                "max_results": { "type": "integer" }
            },
            "required": ["pattern"]
        });
        let description = "Search files for a literal string and return matching lines as `path:line: text`. \
                           Skips binary files, large files, symlinks and .git, node_modules, target.";
        Self { spec: ToolSpec::new("grep", description, schema).risk(Risk::Read).parallel().timeout(Duration::from_secs(60)), roots }
    }
}

struct Search<'a> {
    needle: String,
    ignore_case: bool,
    max_results: usize,
    roots: &'a Roots,
    ctx: &'a ToolCtx,
    hits: Vec<String>,
    files_seen: usize,
    capped: bool,
}

impl Search<'_> {
    fn done(&self) -> bool {
        self.capped || self.ctx.is_cancelled()
    }

    fn file(&mut self, path: &Path) {
        if self.files_seen >= GREP_FILE_LIMIT {
            self.capped = true;
            return;
        }
        self.files_seen += 1;
        let Ok(file) = File::open(path) else { return };
        let mut bytes = Vec::new();
        if file.take(GREP_FILE_BYTES + 1).read_to_end(&mut bytes).is_err()
            || bytes.len() as u64 > GREP_FILE_BYTES
            || looks_binary(&bytes)
        {
            return;
        }
        let shown = self.roots.display(path);
        // Decoded like read_file, so a Chinese pattern also finds GBK files.
        let (text, _) = decode_line(&bytes);
        for (i, line) in text.lines().enumerate() {
            let found = if self.ignore_case { line.to_lowercase().contains(&self.needle) } else { line.contains(&self.needle) };
            if !found {
                continue;
            }
            if self.hits.len() >= self.max_results {
                self.capped = true;
                return;
            }
            self.hits.push(format!("{shown}:{}: {}", i + 1, clip_chars(line.trim(), GREP_LINE_CHARS)));
        }
    }

    fn dir(&mut self, path: &Path, depth: usize) {
        if depth > GREP_MAX_DEPTH {
            return;
        }
        let Ok(read) = fs::read_dir(path) else { return };
        let mut entries: Vec<_> = read.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            if self.done() {
                return;
            }
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                if !SKIP_DIRS.iter().any(|s| entry.file_name() == *s) {
                    self.dir(&entry.path(), depth + 1);
                }
            } else if kind.is_file() {
                self.file(&entry.path());
            }
        }
    }
}

impl Tool for Grep {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let pattern = str_arg(&args, "pattern")?;
        if pattern.is_empty() || pattern.chars().count() > 500 {
            return Err("`pattern` must be 1 to 500 characters".to_string());
        }
        let ignore_case = args.get("ignore_case").and_then(Value::as_bool).unwrap_or(false);
        let max_results = opt_uint_arg(&args, "max_results")?.unwrap_or(GREP_DEFAULT_RESULTS).clamp(1, GREP_MAX_RESULTS) as usize;
        let start = self.roots.resolve(opt_str_arg(&args, "path")?.unwrap_or("."))?;
        let needle = if ignore_case { pattern.to_lowercase() } else { pattern.to_string() };
        let mut search = Search { needle, ignore_case, max_results, roots: &self.roots, ctx, hits: Vec::new(), files_seen: 0, capped: false };
        if start.is_dir() {
            search.dir(&start, 0);
        } else {
            search.file(&start);
        }
        if ctx.is_cancelled() {
            return Err("cancelled".to_string());
        }
        if search.hits.is_empty() {
            return Ok(if search.capped { "No matches found before the search limit was reached.".to_string() } else { "No matches.".to_string() });
        }
        let mut out = search.hits.join("\n");
        if search.capped {
            out.push_str("\n[results limited; narrow the path or pattern to see more]");
        }
        let cut = nearest_boundary(&out, out.len(), false);
        out.truncate(cut);
        Ok(out)
    }
}

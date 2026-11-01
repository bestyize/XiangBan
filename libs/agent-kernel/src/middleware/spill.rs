use super::Middleware;
use crate::tool::ToolResult;
use crate::types::ToolCall;
use crate::util::{file_safe, nearest_boundary};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PREVIEW: usize = 2000;

/// Saves oversized tool output to a file and gives the model a short preview plus the path.
/// The host owns `dir`: it must be readable by the model's file tools and cleaned up by the host.
pub struct SpillOutput {
    dir: PathBuf,
    threshold: usize,
    counter: AtomicU64,
}

impl SpillOutput {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), threshold: 16_000, counter: AtomicU64::new(0) }
    }

    /// Outputs longer than this many bytes are spilled.
    pub fn threshold(mut self, bytes: usize) -> Self {
        self.threshold = bytes.max(PREVIEW * 2);
        self
    }

    fn write(&self, session: &str, call: &ToolCall, output: &str) -> Option<PathBuf> {
        std::fs::create_dir_all(&self.dir).ok()?;
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
        for _ in 0..5 {
            let n = self.counter.fetch_add(1, Ordering::Relaxed);
            let name = format!("{}-{millis}-{n}-{}.txt", file_safe(session), file_safe(&call.id));
            let path = self.dir.join(name.chars().take(180).collect::<String>());
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => return file.write_all(output.as_bytes()).is_ok().then_some(path),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
                Err(_) => return None,
            }
        }
        None
    }
}

impl Middleware for SpillOutput {
    fn after_tool(&self, session: &str, call: &ToolCall, result: &mut ToolResult) {
        let output = &result.output;
        if output.len() <= self.threshold {
            return;
        }
        let Some(path) = self.write(session, call, output) else {
            return; // the loop's plain truncation still applies
        };
        let head = nearest_boundary(output, PREVIEW, false);
        let tail = nearest_boundary(output, output.len() - PREVIEW, true);
        // Compact JSON is often one long line, so offer structured paging.
        let json_pointer = if output.len() <= 4 * 1024 * 1024 {
            serde_json::from_str::<serde_json::Value>(output).ok().and_then(|value| {
                if value.is_array() { Some("") } else if value.get("items").is_some_and(serde_json::Value::is_array) { Some("/items") } else { None }
            })
        } else { None };
        result.output = format!(
            "{}\n[... {} bytes omitted ...]\n{}\n[The full output ({} bytes) is saved at {}. Read it in parts with a file tool, or search it.]",
            &output[..head],
            tail - head,
            &output[tail..],
            output.len(),
            path.display()
        );
        if let Some(pointer) = json_pointer {
            result.output.push_str(&format!("\n[This JSON contains an array. Use read_file with json_pointer={pointer:?}, json_fields, json_offset/json_limit to inspect it without shell; json_group_by counts a field across the saved array only, not other API pages. Check the original API total and fetch remaining pages if needed. Missing projected fields are null.]"));
        }
    }
}

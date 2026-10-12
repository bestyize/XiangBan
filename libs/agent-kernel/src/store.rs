use crate::error::HarnessError;
use crate::types::Message;
use crate::util::file_safe;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The host decides where sessions live. The kernel only loads and appends.
pub trait SessionStore: Send + Sync {
    fn load(&self, session: &str) -> Result<Vec<Message>, HarnessError>;
    fn append(&self, session: &str, messages: &[Message]) -> Result<(), HarnessError>;
}

#[derive(Default)]
pub struct MemoryStore {
    sessions: Mutex<HashMap<String, Vec<Message>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ids of every session that has messages, sorted.
    pub fn session_ids(&self) -> Vec<String> {
        let map = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let mut ids: Vec<String> = map.keys().cloned().collect();
        ids.sort();
        ids
    }
}

impl SessionStore for MemoryStore {
    fn load(&self, session: &str) -> Result<Vec<Message>, HarnessError> {
        let map = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        Ok(map.get(session).cloned().unwrap_or_default())
    }

    fn append(&self, session: &str, messages: &[Message]) -> Result<(), HarnessError> {
        let mut map = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        map.entry(session.to_string()).or_default().extend_from_slice(messages);
        Ok(())
    }
}

const RECORD_VERSION: u32 = 1;

#[derive(Serialize)]
struct RecordRef<'a> {
    v: u32,
    m: &'a Message,
}

#[derive(Deserialize)]
struct Record {
    v: u32,
    m: Message,
}

/// One append-only JSONL file per session inside a host-chosen directory.
///
/// A crash can leave a half-written last line. `load` ignores it and the next `append` removes it,
/// so earlier records are never damaged. Files hold the full conversation; on Unix they are
/// created readable by the owner only.
pub struct JsonlStore {
    dir: PathBuf,
}

impl JsonlStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// The `s-` prefix keeps ids such as `con` or `nul` from becoming reserved Windows device names.
    fn path(&self, session: &str) -> PathBuf {
        self.dir.join(format!("s-{}.jsonl", file_safe(session)))
    }

    /// Deletes a session's file. A session that was never saved is not an error. The caller must
    /// make sure no `run` is working on that session.
    pub fn remove(&self, session: &str) -> Result<(), HarnessError> {
        let path = self.path(session);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Self::io_error("delete", &path, e)),
        }
    }

    fn io_error(action: &str, path: &Path, e: io::Error) -> HarnessError {
        HarnessError::Store(format!("{action} {}: {e}", path.display()))
    }
}

/// Cuts a trailing line that never got its newline.
fn trim_partial_tail(file: &mut File) -> io::Result<()> {
    let len = file.metadata()?.len();
    let mut buf = [0u8; 4096];
    let mut end = len;
    while end > 0 {
        let start = end.saturating_sub(buf.len() as u64);
        let n = (end - start) as usize;
        file.seek(SeekFrom::Start(start))?;
        file.read_exact(&mut buf[..n])?;
        if let Some(i) = buf[..n].iter().rposition(|&b| b == b'\n') {
            let keep = start + i as u64 + 1;
            if keep < len {
                file.set_len(keep)?;
            }
            return Ok(());
        }
        end = start;
    }
    if len > 0 {
        file.set_len(0)?;
    }
    Ok(())
}

fn open_for_append(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)
}

impl SessionStore for JsonlStore {
    fn load(&self, session: &str) -> Result<Vec<Message>, HarnessError> {
        let path = self.path(session);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(Self::io_error("read", &path, e)),
        };
        // Everything after the last newline is an unfinished write.
        let complete = bytes.iter().rposition(|&b| b == b'\n').map_or(&bytes[..0], |i| &bytes[..=i]);
        let mut messages = Vec::new();
        for (number, line) in complete.split(|&b| b == b'\n').enumerate() {
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let bad = |why: String| HarnessError::Store(format!("{} line {}: {why}", path.display(), number + 1));
            let record: Record = serde_json::from_slice(line).map_err(|e| bad(e.to_string()))?;
            if record.v > RECORD_VERSION {
                return Err(bad(format!("written by a newer version (format {})", record.v)));
            }
            messages.push(record.m);
        }
        Ok(messages)
    }

    fn append(&self, session: &str, messages: &[Message]) -> Result<(), HarnessError> {
        if messages.is_empty() {
            return Ok(());
        }
        let path = self.path(session);
        let mut data = Vec::new();
        for message in messages {
            serde_json::to_writer(&mut data, &RecordRef { v: RECORD_VERSION, m: message })
                .map_err(|e| HarnessError::Store(format!("encode message: {e}")))?;
            data.push(b'\n');
        }
        fs::create_dir_all(&self.dir).map_err(|e| Self::io_error("create", &self.dir, e))?;
        let mut file = open_for_append(&path).map_err(|e| Self::io_error("open", &path, e))?;
        trim_partial_tail(&mut file).map_err(|e| Self::io_error("repair", &path, e))?;
        file.seek(SeekFrom::End(0)).map_err(|e| Self::io_error("seek", &path, e))?;
        file.write_all(&data).and_then(|()| file.flush()).map_err(|e| Self::io_error("write", &path, e))
    }
}

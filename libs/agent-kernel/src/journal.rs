//! Optional local execution journal. It observes runs; it never rewrites model context or replays tools.

use crate::{Event, HarnessError, RunEvent, RunEventBody, RunState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_RECORD: u64 = 8 * 1024 * 1024;

struct Writer {
    file: File,
    session: String,
    parent: Option<String>,
    sequence: u64,
    poisoned: bool,
    last_sync: std::time::Instant,
    unsynced_bytes: usize,
}

#[derive(Deserialize)]
struct Record {
    v: u32,
    event: RunEvent,
}

#[derive(Serialize)]
struct RecordRef<'a> {
    v: u32,
    event: &'a RunEvent,
}

/// Share one instance among runs. A run's first append exclusively creates its file; a second
/// writer cannot attach to it. This is a per-run writer lease, not a cross-process session lock.
pub struct JsonlRunLog {
    dir: PathBuf,
    writers: Mutex<HashMap<String, Writer>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncompleteTool {
    pub id: String,
    pub name: String,
    pub dispatched: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunInspection {
    pub last_sequence: u64,
    /// Last persisted state, not proof that a writer/process is still alive.
    pub state: Option<RunState>,
    pub terminal: bool,
    pub incomplete_tools: Vec<IncompleteTool>,
}

fn error(e: impl std::fmt::Display) -> HarnessError {
    HarnessError::Store(format!("run log: {e}"))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

impl JsonlRunLog {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            writers: Mutex::new(HashMap::new()),
        }
    }

    fn path(&self, run_id: &str) -> Result<PathBuf, HarnessError> {
        if !valid_id(run_id) {
            return Err(error("invalid run id"));
        }
        Ok(self.dir.join(format!("run-{run_id}.jsonl")))
    }

    /// State and tool boundaries sync immediately. Text and reasoning deltas sync
    /// together at most once per 500 ms or 64 KiB, and at the next boundary.
    /// A failed write poisons that writer.
    pub fn append(&self, event: &RunEvent) -> Result<(), HarnessError> {
        let path = self.path(&event.run_id)?;
        let mut data = serde_json::to_vec(&RecordRef { v: 1, event }).map_err(error)?;
        if data.len() as u64 >= MAX_RECORD {
            return Err(error("event exceeds 8 MiB"));
        }
        data.push(b'\n');
        let mut writers = self.writers.lock().unwrap_or_else(|e| e.into_inner());
        if !writers.contains_key(&event.run_id) {
            if event.sequence != 1 {
                return Err(error("a new writer requires sequence 1"));
            }
            fs::create_dir_all(&self.dir).map_err(error)?;
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
            let file = options.open(path).map_err(error)?;
            writers.insert(
                event.run_id.clone(),
                Writer {
                    file,
                    session: event.session_id.clone(),
                    parent: event.parent_run_id.clone(),
                    sequence: 0,
                    poisoned: false,
                    last_sync: std::time::Instant::now(),
                    unsynced_bytes: 0,
                },
            );
        }
        let writer = writers
            .get_mut(&event.run_id)
            .ok_or_else(|| error("writer unavailable"))?;
        if writer.poisoned {
            return Err(error("writer is unusable after an earlier I/O failure"));
        }
        if writer.session != event.session_id
            || writer.parent != event.parent_run_id
            || event.sequence != writer.sequence + 1
        {
            return Err(error("session or sequence does not match the writer"));
        }
        let sync = !matches!(
            &event.body,
            RunEventBody::Event {
                event: Event::TextDelta { .. } | Event::ReasoningDelta { .. }
            }
        );
        writer.unsynced_bytes = writer.unsynced_bytes.saturating_add(data.len());
        let sync = sync || writer.unsynced_bytes >= 64 * 1024 || writer.last_sync.elapsed() >= std::time::Duration::from_millis(500);
        if let Err(e) = writer.file.write_all(&data).and_then(|_| {
            if sync {
                writer.file.sync_data()
            } else {
                Ok(())
            }
        }) {
            writer.poisoned = true;
            return Err(error(e));
        }
        if sync { writer.last_sync = std::time::Instant::now(); writer.unsynced_bytes = 0; }
        writer.sequence = event.sequence;
        if matches!(event.body, RunEventBody::State { state, .. } if state != RunState::Running) {
            writers.remove(&event.run_id);
        }
        Ok(())
    }

    /// Cursor paging is independent of whether the run is active. A partial final line is ignored.
    pub fn read_after(
        &self,
        run_id: &str,
        sequence: u64,
        limit: usize,
    ) -> Result<Vec<RunEvent>, HarnessError> {
        let mut out = Vec::new();
        self.scan(run_id, |event| {
            if event.sequence > sequence {
                out.push(event);
            }
            out.len() < limit.clamp(1, 100)
        })?;
        Ok(out)
    }

    /// Visit a validated trace in one pass without buffering it. A partial final
    /// record is ignored just as in read_after; no tools or observers are invoked.
    pub fn visit(&self, run_id: &str, mut accept: impl FnMut(RunEvent)) -> Result<(), HarnessError> {
        self.scan(run_id, |event| { accept(event); true })
    }

    pub fn inspect(&self, run_id: &str) -> Result<RunInspection, HarnessError> {
        let mut inspection = RunInspection {
            last_sequence: 0,
            state: None,
            terminal: false,
            incomplete_tools: Vec::new(),
        };
        let mut pending = BTreeMap::new();
        self.scan(run_id, |event| {
            inspection.last_sequence = event.sequence;
            match event.body {
                RunEventBody::State { state, .. } => {
                    inspection.state = Some(state);
                    inspection.terminal = state != RunState::Running;
                }
                RunEventBody::Event {
                    event: Event::ToolStart { id, name, .. },
                } => {
                    pending.insert(
                        id.clone(),
                        IncompleteTool {
                            id,
                            name,
                            dispatched: false,
                        },
                    );
                }
                RunEventBody::Event {
                    event: Event::ToolDispatched { id },
                } => {
                    if let Some(tool) = pending.get_mut(&id) {
                        tool.dispatched = true;
                    }
                }
                RunEventBody::Event {
                    event: Event::ToolEnd { id, structured_content, .. },
                } => {
                    if structured_content.as_ref().and_then(|v|v.get("harness_status")).and_then(serde_json::Value::as_str)!=Some("outcome_unknown"){pending.remove(&id);}
                }
                _ => {}
            }
            true
        })?;
        inspection.incomplete_tools = pending.into_values().collect();
        Ok(inspection)
    }

    fn scan(
        &self,
        run_id: &str,
        mut accept: impl FnMut(RunEvent) -> bool,
    ) -> Result<(), HarnessError> {
        scan_file(&self.path(run_id)?, run_id, &mut accept)
    }
}

fn scan_file(
    path: &Path,
    run_id: &str,
    accept: &mut dyn FnMut(RunEvent) -> bool,
) -> Result<(), HarnessError> {
    let mut reader = BufReader::new(File::open(path).map_err(error)?);
    let mut sequence = 0;
    let mut session = None;
    let mut parent = None;
    let mut terminal = false;
    loop {
        let mut line = Vec::new();
        let n = (&mut reader)
            .take(MAX_RECORD + 1)
            .read_until(b'\n', &mut line)
            .map_err(error)?;
        if n == 0 {
            return Ok(());
        }
        if n as u64 > MAX_RECORD {
            return Err(error("record exceeds 8 MiB"));
        }
        if line.last() != Some(&b'\n') {
            return Ok(());
        }
        let record: Record = serde_json::from_slice(&line).map_err(error)?;
        if record.v != 1 {
            return Err(error("unsupported run log version"));
        }
        let event = record.event;
        if terminal
            || event.run_id != run_id
            || event.sequence != sequence + 1
            || session.as_ref().is_some_and(|s| s != &event.session_id)
            || parent.as_ref().is_some_and(|p| p != &event.parent_run_id)
        {
            return Err(error("invalid run identity or event sequence"));
        }
        sequence = event.sequence;
        session = Some(event.session_id.clone());
        parent = Some(event.parent_run_id.clone());
        terminal =
            matches!(event.body, RunEventBody::State { state, .. } if state != RunState::Running);
        if !accept(event) {
            return Ok(());
        }
    }
}

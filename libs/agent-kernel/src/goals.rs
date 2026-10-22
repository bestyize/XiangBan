//! Persistent goal contracts. This is *not* a background scheduler: a model
//! stopping is never proof that a goal passed its acceptance criteria.
//! The host owns dispatch, evidence collection and explicit completion.
use serde::{Deserialize, Serialize};
use std::{fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}};
static TEMP: AtomicU64 = AtomicU64::new(0);
const MAX_BYTES: u64 = 32 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalContract {
    pub outcome: String,
    #[serde(default)]
    pub verification: String,
    #[serde(default)]
    pub constraints: String,
    #[serde(default)]
    pub boundaries: String,
    #[serde(default)]
    pub stop_when: String,
}
impl GoalContract {
    pub fn validate(&self) -> Result<(), String> {
        let fields = [
            ("outcome", &self.outcome, 2_000),
            ("verification", &self.verification, 2_000),
            ("constraints", &self.constraints, 2_000),
            ("boundaries", &self.boundaries, 2_000),
            ("stop_when", &self.stop_when, 2_000),
        ];
        for (label, value, max) in fields {
            if value.chars().count() > max || value.chars().any(|c| c == '\0') {
                return Err(format!("goal field {label} is invalid or too long"));
            }
        }
        if self.outcome.trim().is_empty() { return Err("goal outcome is required".into()); }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalState {
    Draft,
    Running,
    AwaitingVerification,
    Paused,
    Interrupted,
    NeedsReview,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalRecord {
    pub version: u32,
    pub session_id: String,
    pub contract: GoalContract,
    pub state: GoalState,
    pub attempts: u32,
    pub last_run_id: Option<String>,
    pub updated_at_ms: u64,
    #[serde(default)]
    pub note: String,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 &&
        id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

/// Replaces an existing record without exposing partially written JSON.
/// Windows std::fs::rename cannot reliably replace an existing destination;
/// use MoveFileExW with REPLACE_EXISTING and WRITE_THROUGH instead.
#[cfg(windows)]
fn replace_atomically(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }
    let src: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let dst: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    const MOVEFILE_REPLACE_EXISTING: u32 = 1;
    const MOVEFILE_WRITE_THROUGH: u32 = 8;
    if unsafe { MoveFileExW(src.as_ptr(), dst.as_ptr(),
        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) } == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(())
}
#[cfg(not(windows))]
fn replace_atomically(from: &Path, to: &Path) -> Result<(), String> {
    fs::rename(from, to).map_err(|e| e.to_string())
}

/// Disk-backed goal metadata. Atomic file replacement prevents torn JSON;
/// the embedding host must serialize writes per session. It is *not* a
/// cross-process transaction or a distributed lease.
pub struct GoalStore { root: PathBuf }
impl GoalStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    fn path(&self, id: &str) -> Result<PathBuf, String> {
        if !valid_id(id) { return Err("invalid session id".into()); }
        Ok(self.root.join(format!("{id}.json")))
    }
    pub fn load(&self, id: &str) -> Result<Option<GoalRecord>, String> {
        let path = self.path(id)?;
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES {
            return Err("goal record exceeds size limit".into());
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_BYTES { return Err("goal record exceeds size limit".into()); }
        let record: GoalRecord = serde_json::from_slice(&bytes).map_err(|e| format!("invalid goal record: {e}"))?;
        if record.version != 1 || record.session_id != id {
            return Err("goal record version or session does not match".into());
        }
        record.contract.validate()?;
        Ok(Some(record))
    }
    pub fn save(&self, record: &GoalRecord) -> Result<(), String> {
        let path = self.path(&record.session_id)?;
        if record.version != 1 { return Err("unsupported goal record version".into()); }
        record.contract.validate()?;
        let bytes = serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_BYTES { return Err("goal record exceeds size limit".into()); }
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let suffix = TEMP.fetch_add(1, Ordering::Relaxed);
        let temp = self.root.join(format!(".{}.{}.{suffix}.tmp", record.session_id, std::process::id()));
        let write = (|| -> Result<(), String> {
            let mut options = OpenOptions::new();
            options.create_new(true).write(true);
            #[cfg(unix)]
            { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
            let mut file = options.open(&temp).map_err(|e| e.to_string())?;
            file.write_all(&bytes).and_then(|_| file.sync_all()).map_err(|e| e.to_string())?;
            replace_atomically(&temp, &path)?;
            #[cfg(unix)]
            { let _ = File::open(&self.root).and_then(|f| f.sync_all()); }
            // The file has already been atomically published. Failure to sync
            // the parent directory on a filesystem that forbids directory
            // fsync must not be reported as an aborted transaction.
            Ok(())
        })();
        if write.is_err() { let _ = fs::remove_file(&temp); }
        write
    }
    pub fn delete(&self, id: &str) -> Result<(), String> {
        match fs::remove_file(self.path(id)?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// A goal is only complete after an external verifier or a human accepts the
/// evidence. Run completion can *only* transition to AwaitingVerification.
impl GoalRecord {
    pub fn draft(session_id: String, contract: GoalContract, now_ms: u64) -> Result<Self, String> {
        if !valid_id(&session_id) { return Err("invalid session id".into()); }
        contract.validate()?;
        Ok(Self { version: 1, session_id, contract, state: GoalState::Draft,
            attempts: 0, last_run_id: None, updated_at_ms: now_ms, note: String::new() })
    }
    pub fn begin(&mut self, now_ms: u64) -> Result<(), String> {
        if !matches!(self.state, GoalState::Draft | GoalState::Paused | GoalState::AwaitingVerification) {
            return Err(format!("cannot begin goal in state {:?}", self.state));
        }
        self.state = GoalState::Running;
        self.updated_at_ms = now_ms;
        self.attempts = self.attempts.saturating_add(1);
        self.last_run_id = None;
        self.note.clear();
        Ok(())
    }
    pub fn observe(&mut self, run_id: &str, state: crate::RunState, now_ms: u64) -> Result<(), String> {
        if !valid_id(run_id) { return Err("invalid run id".into()); }
        if self.state != GoalState::Running { return Ok(()); }
        if self.last_run_id.as_deref().is_some_and(|id| id != run_id) { return Err("goal run identity changed".into()); }
        self.last_run_id = Some(run_id.into());
        self.updated_at_ms = now_ms;
        match state {
            crate::RunState::Running => (),
            crate::RunState::Completed => self.state = GoalState::AwaitingVerification,
            crate::RunState::Paused | crate::RunState::Failed | crate::RunState::Cancelled => self.state = GoalState::Paused,
        }
        Ok(())
    }
    pub fn pause(&mut self, now_ms: u64) -> Result<(), String> {
        if matches!(self.state, GoalState::Completed | GoalState::NeedsReview) {
            return Err("completed or uncertain goals cannot be paused".into());
        }
        self.state = GoalState::Paused;
        self.updated_at_ms = now_ms;
        Ok(())
    }
    pub fn finish_verified(&mut self, evidence: &str, now_ms: u64) -> Result<(), String> {
        if self.state != GoalState::AwaitingVerification { return Err("goal has no result awaiting verification".into()); }
        if evidence.trim().is_empty() || evidence.chars().count() > 2_000 { return Err("completion evidence required".into()); }
        self.state = GoalState::Completed;
        self.note = evidence.into();
        self.updated_at_ms = now_ms;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> GoalContract {
        GoalContract { outcome: "修复测试失败".into(), verification: "测试全部通过".into(),
            constraints: "不修改公开接口".into(), boundaries: "libs/agent-kernel".into(),
            stop_when: "需要新的密钥".into() }
    }
    fn dir() -> PathBuf {
        let name = format!("agent-goal-test-{}-{}",
            std::process::id(), TEMP.fetch_add(1, Ordering::Relaxed));
        std::env::temp_dir().join(name)
    }
    #[test]
    fn persisted_goal_is_roundtrippable_and_model_completion_is_not_acceptance() {
        let root = dir();
        let store = GoalStore::new(&root);
        let mut goal = GoalRecord::draft("session-1".into(), contract(), 1).unwrap();
        store.save(&goal).unwrap();
        assert_eq!(store.load("session-1").unwrap(), Some(goal.clone()));
        goal.begin(2).unwrap();
        goal.observe("run-1", crate::RunState::Running, 3).unwrap();
        goal.observe("run-1", crate::RunState::Completed, 4).unwrap();
        assert_eq!(goal.state, GoalState::AwaitingVerification);
        assert!(goal.finish_verified("", 5).is_err());
        store.save(&goal).unwrap(); // exercises replacing an existing file
        assert_eq!(store.load("session-1").unwrap(), Some(goal.clone()));
        goal.finish_verified("已核验测试报告，全部通过", 6).unwrap();
        assert_eq!(goal.state, GoalState::Completed);
        assert!(goal.begin(7).is_err());
        store.save(&goal).unwrap();
        store.delete("session-1").unwrap();
        assert_eq!(store.load("session-1").unwrap(), None);
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn unexpected_identity_or_corrupt_data_fail_closed() {
        let root = dir();
        let store = GoalStore::new(&root);
        assert!(store.load("../elsewhere").is_err());
        let mut goal = GoalRecord::draft("session-2".into(), contract(), 1).unwrap();
        goal.begin(2).unwrap();
        assert!(goal.observe("run-x", crate::RunState::Running, 3).is_ok());
        assert!(goal.observe("run-y", crate::RunState::Completed, 4).is_err());
        store.save(&goal).unwrap();
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("session-2.json")).unwrap()).unwrap();
        data["sessionId"] = serde_json::json!("different");
        fs::write(root.join("session-2.json"), serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(store.load("session-2").is_err());
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn invalid_or_unbounded_contract_is_rejected() {
        let mut c = contract();
        c.outcome = " ".into();
        assert!(c.validate().is_err());
        c.outcome = "a".repeat(2_001);
        assert!(c.validate().is_err());
    }
}

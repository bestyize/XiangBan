//! Cross-process session ownership for an active run.
//! The OS releases a file lock when its process dies, unlike a persistent
//! "running" flag or a PID file. It is a local-machine advisory lease, not
//! distributed exactly-once execution and not a replacement for sandboxing.
use std::{fs::{self, File, OpenOptions}, path::{Path, PathBuf}};

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 &&
        id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

#[derive(Debug)]
pub struct SessionLease {
    _handle: File,
}
impl SessionLease {
    /// Returns Busy if another process (or agent instance) owns the session.
    /// Lockfiles are deliberately left on disk: only their OS lock matters,
    /// and deleting/recreating a live lockfile can break mutual exclusion.
    pub fn acquire(root: impl AsRef<Path>, session_id: &str) -> Result<Self, String> {
        if !valid_id(session_id) { return Err("invalid session id".into()); }
        let root = root.as_ref();
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let path: PathBuf = root.join(format!("{session_id}.lock"));
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create(true);
        #[cfg(unix)]
        { use std::os::unix::fs::OpenOptionsExt; opts.mode(0o600); }
        let file = opts.open(path).map_err(|e| e.to_string())?;
        // Since Rust 1.89 `try_lock` reports a held lock as `TryLockError::WouldBlock`, not as an io::Error kind.
        file.try_lock().map_err(|e| match e {
            fs::TryLockError::WouldBlock => "session is already owned by another process".to_string(),
            fs::TryLockError::Error(e) => format!("cannot acquire session lease: {e}"),
        })?;
        Ok(Self { _handle: file })
    }
    /// A check for other-process ownership, used before classifying an old
    /// Running record as crashed. A successful probe releases the lock at once.
    /// There is still a race: the real run must acquire and hold its own lease.
    pub fn held(root: impl AsRef<Path>, session_id: &str) -> Result<bool, String> {
        match Self::acquire(root, session_id) {
            Ok(_lease) => Ok(false),
            Err(e) if e == "session is already owned by another process" => Ok(true),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_second_writer_cannot_claim_the_same_session_and_crash_releases_os_lock() {
        let root = std::env::temp_dir().join(format!("agent-session-lease-{}-{}", std::process::id(),
            crate::runtime::now_ms()));
        let first = SessionLease::acquire(&root, "conversation-1").unwrap();
        assert!(SessionLease::held(&root, "conversation-1").unwrap());
        assert!(SessionLease::acquire(&root, "conversation-1").is_err());
        let different = SessionLease::acquire(&root, "conversation-2").unwrap();
        drop(first);
        assert!(!SessionLease::held(&root, "conversation-1").unwrap());
        let another = SessionLease::acquire(&root, "conversation-1").unwrap();
        drop(another);
        drop(different);
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn invalid_session_ids_cannot_escape_the_lease_root() {
        assert!(SessionLease::acquire(std::env::temp_dir(), "../escape").is_err());
    }
}

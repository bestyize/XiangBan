//! An OS sandbox for the shell and managed processes, so "run without asking" can mean it.
//!
//! The danger heuristics in a host read a command line; they cannot see what `python script.py` will do once the
//! model has written the script. A sandbox limits what any command can do, whatever it is called.
//!
//! macOS only for now: commands run under `sandbox-exec` with a Seatbelt profile that allows everything except
//! writing outside the working folder, the temporary folders and package caches, and reading well-known secret
//! folders. Network stays open unless the policy closes it (installing dependencies needs it). The profile follows
//! the "permissive-open" profile Gemini CLI ships. On other systems `available()` is false and a host should not
//! ask for a sandbox; when one is asked for anyway, the command is refused rather than run unprotected.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// What a sandboxed command may do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxPolicy {
    /// Folders (and their contents) that may be written.
    pub writable: Vec<PathBuf>,
    /// Folders that may not even be read.
    pub unreadable: Vec<PathBuf>,
    /// Outbound network beyond the local machine.
    pub network: bool,
}

impl SandboxPolicy {
    /// The usual policy for a working folder: write there, in the temporary folders and in package-manager caches;
    /// never read SSH, cloud and keychain folders; network open.
    pub fn for_workspace(workspace: &Path) -> Self {
        let mut writable = vec![workspace.to_path_buf(), std::env::temp_dir(), PathBuf::from("/private/tmp"), PathBuf::from("/private/var/folders")];
        let mut unreadable = Vec::new();
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            // Caches only: `~/.cargo/bin` and friends are on PATH, and writing there would outlive the sandbox.
            writable.extend(
                [".npm", ".cache", ".cargo/registry", ".cargo/git", ".rustup/tmp", ".gradle/caches", ".m2/repository", ".yarn", "Library/Caches", "Library/pnpm", "go/pkg/mod"]
                    .iter()
                    .map(|dir| home.join(dir)),
            );
            unreadable.extend([".ssh", ".aws", ".gnupg", ".kube", ".docker", ".config/gh", ".config/gcloud", ".azure", ".password-store", "Library/Keychains"].iter().map(|dir| home.join(dir)));
        }
        Self { writable, unreadable, network: true }
    }
}

/// Decides per call whether, and how, a command is sandboxed: gets the session and the command line.
#[derive(Clone)]
pub struct SandboxHook(Arc<dyn Fn(&str, &str) -> Option<SandboxPolicy> + Send + Sync>);

impl SandboxHook {
    pub fn new(decide: impl Fn(&str, &str) -> Option<SandboxPolicy> + Send + Sync + 'static) -> Self {
        Self(Arc::new(decide))
    }

    pub(crate) fn policy(&self, session: &str, command: &str) -> Option<SandboxPolicy> {
        (self.0)(session, command)
    }
}

impl fmt::Debug for SandboxHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SandboxHook")
    }
}

/// Whether this system can sandbox commands.
pub fn available() -> bool {
    cfg!(target_os = "macos") && Path::new(SANDBOX_EXEC).exists()
}

const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";

/// Seatbelt compares real paths (`/tmp` is `/private/tmp`), so every path is resolved first; a folder that does
/// not exist yet keeps its name.
fn real(path: &Path) -> String {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf()).to_string_lossy().into_owned()
}

fn quoted(path: &Path) -> String {
    format!("\"{}\"", real(path).replace('\\', "\\\\").replace('"', "\\\""))
}

/// The Seatbelt profile for `policy`. Later rules win, so the denials come after `allow default`.
pub fn seatbelt_profile(policy: &SandboxPolicy) -> String {
    let mut profile = String::from("(version 1)\n(allow default)\n(deny file-write*)\n(allow file-write*\n");
    for dir in &policy.writable {
        profile.push_str(&format!("    (subpath {})\n", quoted(dir)));
    }
    profile.push_str("    (literal \"/dev/null\") (literal \"/dev/zero\") (literal \"/dev/stdout\") (literal \"/dev/stderr\")\n");
    profile.push_str("    (regex #\"^/dev/tty\") (regex #\"^/dev/fd/\")\n)\n");
    if !policy.unreadable.is_empty() {
        profile.push_str("(deny file-read*\n");
        for dir in &policy.unreadable {
            profile.push_str(&format!("    (subpath {})\n", quoted(dir)));
        }
        profile.push_str(")\n");
    }
    if !policy.network {
        profile.push_str("(deny network-outbound)\n(allow network-outbound (remote unix-socket) (remote ip \"localhost:*\"))\n");
    }
    profile
}

/// Appended to a failed sandboxed command whose output looks like the sandbox stopped it, so the model asks the
/// user instead of trying other ways around it.
pub(crate) const BLOCKED_HINT: &str = "[sandbox: this command ran in a sandbox that only allows writing inside the working folder, \
temporary folders and package caches, and blocks reading credential folders. \"Operation not permitted\" above most likely \
comes from it. Do not look for a way around it; if the change is really needed, tell the user what and why so they can approve it.]";

pub(crate) fn looks_blocked(output: &str) -> bool {
    output.contains("Operation not permitted")
}

/// `sh -c command`, inside the sandbox when `policy` is given. Refuses to run when a sandbox is asked for and this
/// system has none.
#[cfg(unix)]
pub(crate) fn unix_shell(command: &str, policy: Option<&SandboxPolicy>) -> Result<std::process::Command, String> {
    use std::process::Command;
    match policy {
        None => {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            Ok(cmd)
        }
        Some(policy) if available() => {
            let mut cmd = Command::new(SANDBOX_EXEC);
            cmd.args(["-p", &seatbelt_profile(policy), "/bin/sh", "-c", command]);
            Ok(cmd)
        }
        Some(_) => Err("a sandbox was required for this command, but this system cannot provide one; it was not run".into()),
    }
}

#[cfg(windows)]
pub(crate) fn refuse_on_windows(policy: Option<&SandboxPolicy>) -> Result<(), String> {
    match policy {
        None => Ok(()),
        Some(_) => Err("a sandbox was required for this command, but Windows sandboxing is not supported yet; it was not run".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_profile_denies_writes_except_listed_folders_and_escapes_paths() {
        let policy = SandboxPolicy {
            writable: vec![PathBuf::from("/work/my \"proj\"")],
            unreadable: vec![PathBuf::from("/Users/a/.ssh")],
            network: true,
        };
        let profile = seatbelt_profile(&policy);
        let allow_default = profile.find("(allow default)").unwrap();
        let deny_write = profile.find("(deny file-write*)").unwrap();
        let allow_write = profile.find("(allow file-write*").unwrap();
        assert!(allow_default < deny_write && deny_write < allow_write, "later rules win, so the order matters");
        assert!(profile.contains("(subpath \"/work/my \\\"proj\\\"\")"), "{profile}");
        assert!(profile.contains("(deny file-read*\n    (subpath \"/Users/a/.ssh\")"));
        assert!(!profile.contains("network-outbound"), "network stays open by default");
        let closed = seatbelt_profile(&SandboxPolicy { network: false, ..policy });
        assert!(closed.contains("(deny network-outbound)") && closed.contains("localhost:*"));
    }

    #[test]
    fn the_workspace_policy_writes_caches_but_not_folders_on_path() {
        let policy = SandboxPolicy::for_workspace(Path::new("/work"));
        assert_eq!(policy.writable[0], PathBuf::from("/work"));
        let text: Vec<String> = policy.writable.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        assert!(text.iter().any(|p| p.ends_with(".npm")));
        assert!(!text.iter().any(|p| p.ends_with(".cargo/bin") || p.ends_with(".cargo")), "{text:?}");
        if std::env::var_os("HOME").is_some() {
            assert!(policy.unreadable.iter().any(|p| p.ends_with(".ssh")));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn asking_for_a_sandbox_where_there_is_none_refuses_to_run() {
        assert!(!available());
        let policy = SandboxPolicy::for_workspace(Path::new("/work"));
        assert!(unix_shell("true", Some(&policy)).is_err());
        assert!(unix_shell("true", None).is_ok());
    }
}

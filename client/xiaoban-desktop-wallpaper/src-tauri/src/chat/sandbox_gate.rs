//! When the shell and background processes run inside the OS sandbox.
//!
//! In "完全信任" the user is not asked about ordinary commands, so on systems that have a sandbox (macOS) those
//! commands run inside it: writes stay in the working folder, temporary folders and package caches. In "默认权限"
//! every command is shown to the user first, so it runs as before.
//!
//! A command the danger check flagged is asked even in full trust; when the user allows it, they mean that very
//! command, which usually reaches outside the folder (`rm -rf ~/Downloads/old`). It runs once outside the sandbox.

use agent_kernel::sandbox::{self, SandboxHook, SandboxPolicy};
use agent_kernel::{Permission, PermissionMode};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

/// "完全信任时允许命令联网" in settings; on by default, since installing dependencies needs it.
static NETWORK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn set_network(allowed: bool) {
    NETWORK.store(allowed, std::sync::atomic::Ordering::Relaxed);
}

fn escapes() -> &'static Mutex<HashSet<(String, String)>> {
    static ESCAPES: OnceLock<Mutex<HashSet<(String, String)>>> = OnceLock::new();
    ESCAPES.get_or_init(Default::default)
}

/// The user allowed this flagged command once in this conversation: it runs outside the sandbox once.
pub fn allow_once(session: &str, command: &str) {
    escapes().lock().unwrap_or_else(|e| e.into_inner()).insert((session.to_string(), command.to_string()));
}

fn take(session: &str, command: &str) -> bool {
    escapes().lock().unwrap_or_else(|e| e.into_inner()).remove(&(session.to_string(), command.to_string()))
}

/// The decision itself, apart from the system and the shared state, so it can be tested anywhere.
fn decide(available: bool, mode: PermissionMode, allowed_once: impl FnOnce() -> bool, policy: &SandboxPolicy) -> Option<SandboxPolicy> {
    if !available || mode != PermissionMode::Auto || allowed_once() {
        return None;
    }
    Some(policy.clone())
}

pub fn hook(permission: Arc<Permission>, workspace: &Path) -> SandboxHook {
    let policy = SandboxPolicy::for_workspace(workspace);
    SandboxHook::new(move |session, command| {
        let policy = SandboxPolicy { network: NETWORK.load(std::sync::atomic::Ordering::Relaxed), ..policy.clone() };
        decide(sandbox::available(), permission.mode_for(session), || take(session, command), &policy)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_unasked_commands_on_a_system_with_a_sandbox_are_sandboxed() {
        let policy = SandboxPolicy::for_workspace(Path::new("/work"));
        assert_eq!(decide(true, PermissionMode::Auto, || false, &policy), Some(policy.clone()));
        assert_eq!(decide(true, PermissionMode::Ask, || false, &policy), None, "every command is shown to the user in ask mode");
        assert_eq!(decide(false, PermissionMode::Auto, || false, &policy), None, "no sandbox on this system");
        assert_eq!(decide(true, PermissionMode::Auto, || true, &policy), None, "the user allowed this flagged command");
    }

    #[test]
    fn an_allowed_flagged_command_escapes_once_in_its_own_conversation() {
        allow_once("s-escape", "rm -rf ~/Downloads/old");
        assert!(!take("s-other", "rm -rf ~/Downloads/old"));
        assert!(!take("s-escape", "rm -rf ~/Downloads/older"));
        assert!(take("s-escape", "rm -rf ~/Downloads/old"));
        assert!(!take("s-escape", "rm -rf ~/Downloads/old"), "only once");
    }
}

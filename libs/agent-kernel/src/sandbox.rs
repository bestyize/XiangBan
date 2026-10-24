//! An OS sandbox for the shell and managed processes, so "run without asking" can mean it.
//!
//! The danger heuristics in a host read a command line; they cannot see what `python script.py` will do once the
//! model has written the script. A sandbox limits what any command can do, whatever it is called.
//!
//! macOS: commands run under `sandbox-exec` with a Seatbelt profile that allows everything except writing outside
//! the working folder, the temporary folders and package caches, and reading well-known secret folders. Network
//! stays open unless the policy closes it (installing dependencies needs it). The profile follows the
//! "permissive-open" profile Gemini CLI ships.
//!
//! Windows (experimental, feature `sandbox`): commands run at low integrity, the way browsers isolate their
//! renderers. A low-integrity process can only write where the folder is labelled low, so the host labels the
//! working folder once (`label_low`) and gives the command a low temporary folder. The host's own executable acts as
//! the launcher (`windows_launcher_main`): it starts `cmd` with a low-integrity copy of its token and passes its
//! standard input and output through, so pipes, timeouts and killing the process tree work as for any command.
//! Reading is not limited on Windows, unlike macOS.
//!
//! Elsewhere `available()` is false and a host should not ask for a sandbox; when one is asked for anyway, the
//! command is refused rather than run unprotected.

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
    /// Windows: the executable that runs `windows_launcher_main`, and a folder labelled low for temporary files
    /// and package caches. Without a launcher, a sandbox cannot be provided on Windows.
    pub launcher: Option<PathBuf>,
    pub low_folder: Option<PathBuf>,
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
        Self { writable, unreadable, network: true, launcher: None, low_folder: None }
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

/// The argument that makes the host's executable run as the launcher.
pub const WINDOWS_LAUNCHER_FLAG: &str = "--agent-sandbox-exec";
/// How the command and the low folder reach the launcher: environment variables set after the child environment
/// is cleared, so quoting never matters, and removed again before `cmd` starts.
#[cfg_attr(not(windows), allow(dead_code))]
const COMMAND_VAR: &str = "AGENT_SANDBOX_COMMAND";
#[cfg_attr(not(windows), allow(dead_code))]
const LOW_FOLDER_VAR: &str = "AGENT_SANDBOX_LOW_FOLDER";

/// `cmd` for `command`, through the launcher when `policy` asks for a sandbox. Refuses without a launcher.
#[cfg(windows)]
pub(crate) fn windows_shell(command: &str, policy: Option<&SandboxPolicy>) -> Result<std::process::Command, String> {
    match policy {
        None => Ok(crate::child_env::windows_shell(command)),
        Some(SandboxPolicy { launcher: Some(launcher), .. }) => {
            let mut cmd = std::process::Command::new(launcher);
            cmd.arg(WINDOWS_LAUNCHER_FLAG);
            Ok(cmd)
        }
        Some(_) => Err("a sandbox was required for this command, but no Windows sandbox launcher is set up; it was not run".into()),
    }
}

/// Hands the command to the launcher; call after the child's environment has been set.
#[cfg(windows)]
pub(crate) fn windows_launcher_env(cmd: &mut std::process::Command, command: &str, policy: Option<&SandboxPolicy>) {
    if let Some(policy) = policy {
        cmd.env(COMMAND_VAR, command);
        if let Some(folder) = &policy.low_folder {
            cmd.env(LOW_FOLDER_VAR, folder);
        }
    }
}

/// Labels `folder`, what is in it and what will be created in it, as low integrity, so sandboxed commands may write
/// there. Changes the folder's security label for good; the host asks the user first.
pub fn label_low(folder: &Path) -> Result<(), String> {
    if !cfg!(windows) {
        return Err("only Windows uses integrity labels".into());
    }
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let output = std::process::Command::new("icacls")
        .arg(folder)
        .args(["/setintegritylevel", "(OI)(CI)low", "/T", "/C", "/Q"])
        .output()
        .map_err(|e| format!("cannot run icacls: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("icacls failed: {}", String::from_utf8_lossy(&output.stderr).trim()))
    }
}

/// When this process was started as the launcher, runs the command and returns its exit code; `None` otherwise.
/// The host calls it first thing in `main`.
pub fn windows_launcher_main() -> Option<i32> {
    if std::env::args().nth(1).as_deref() != Some(WINDOWS_LAUNCHER_FLAG) {
        return None;
    }
    #[cfg(all(windows, feature = "sandbox"))]
    {
        return Some(match launcher::run() {
            Ok(code) => code as i32,
            Err(error) => {
                eprintln!("[sandbox] {error}");
                126
            }
        });
    }
    #[allow(unreachable_code)]
    {
        eprintln!("[sandbox] this build cannot run sandboxed commands");
        Some(126)
    }
}

#[cfg(all(windows, feature = "sandbox"))]
mod launcher {
    use super::{COMMAND_VAR, LOW_FOLDER_VAR};
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, HANDLE};
    use windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW;
    use windows_sys::Win32::Security::{
        DuplicateTokenEx, SetTokenInformation, SecurityImpersonation, TokenIntegrityLevel, TokenPrimary, PSID, SID_AND_ATTRIBUTES,
        TOKEN_ADJUST_DEFAULT, TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
    use windows_sys::Win32::System::Threading::{
        CreateProcessAsUserW, GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, WaitForSingleObject, CREATE_NO_WINDOW, INFINITE,
        PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOW,
    };

    /// `SE_GROUP_INTEGRITY`, the attribute an integrity label's SID carries.
    const SE_GROUP_INTEGRITY: u32 = 0x20;

    fn wide(text: &str) -> Vec<u16> {
        std::ffi::OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
    }

    fn last_error(what: &str) -> String {
        format!("{what} failed: {}", std::io::Error::last_os_error())
    }

    struct Handle(HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CloseHandle(self.0) };
            }
        }
    }

    /// A primary token like this process's, at low integrity.
    fn low_token() -> Result<Handle, String> {
        unsafe {
            let mut own: HANDLE = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ADJUST_DEFAULT | TOKEN_ASSIGN_PRIMARY, &mut own) == 0 {
                return Err(last_error("OpenProcessToken"));
            }
            let own = Handle(own);
            let mut token: HANDLE = std::ptr::null_mut();
            let access = TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ADJUST_DEFAULT | TOKEN_ASSIGN_PRIMARY;
            if DuplicateTokenEx(own.0, access, std::ptr::null(), SecurityImpersonation, TokenPrimary, &mut token) == 0 {
                return Err(last_error("DuplicateTokenEx"));
            }
            let token = Handle(token);
            let mut sid: PSID = std::ptr::null_mut();
            // S-1-16-4096: the low mandatory level.
            if ConvertStringSidToSidW(wide("S-1-16-4096").as_ptr(), &mut sid) == 0 {
                return Err(last_error("ConvertStringSidToSidW"));
            }
            let label = TOKEN_MANDATORY_LABEL { Label: SID_AND_ATTRIBUTES { Sid: sid, Attributes: SE_GROUP_INTEGRITY } };
            let size = std::mem::size_of::<TOKEN_MANDATORY_LABEL>() as u32;
            let set = SetTokenInformation(token.0, TokenIntegrityLevel, &label as *const _ as *const c_void, size);
            LocalFree(sid as _);
            if set == 0 {
                return Err(last_error("SetTokenInformation"));
            }
            Ok(token)
        }
    }

    pub(super) fn run() -> Result<u32, String> {
        let command = std::env::var(COMMAND_VAR).map_err(|_| "no command was given".to_string())?;
        std::env::remove_var(COMMAND_VAR);
        // Temporary files and package caches go to the low folder, the only writable place besides the working folder.
        if let Some(folder) = std::env::var_os(LOW_FOLDER_VAR) {
            std::env::remove_var(LOW_FOLDER_VAR);
            let folder = std::path::PathBuf::from(folder);
            for (var, sub) in [("TEMP", "temp"), ("TMP", "temp"), ("npm_config_cache", "npm-cache"), ("PIP_CACHE_DIR", "pip-cache"), ("YARN_CACHE_FOLDER", "yarn-cache")] {
                let dir = folder.join(sub);
                let _ = std::fs::create_dir_all(&dir);
                std::env::set_var(var, dir);
            }
        }
        let token = low_token()?;
        let mut line = wide(&format!("cmd /D /S /C \"{command}\""));
        unsafe {
            let mut startup: STARTUPINFOW = std::mem::zeroed();
            startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
            startup.dwFlags = STARTF_USESTDHANDLES;
            startup.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
            startup.hStdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
            startup.hStdError = GetStdHandle(STD_ERROR_HANDLE);
            let mut info: PROCESS_INFORMATION = std::mem::zeroed();
            // Inherit handles: the pipes this launcher was given are passed on as they are. The environment and the
            // working folder are this process's.
            let created = CreateProcessAsUserW(
                token.0, std::ptr::null(), line.as_mut_ptr(), std::ptr::null(), std::ptr::null(), 1, CREATE_NO_WINDOW,
                std::ptr::null(), std::ptr::null(), &startup, &mut info,
            );
            if created == 0 {
                return Err(last_error("CreateProcessAsUserW"));
            }
            let (process, _thread) = (Handle(info.hProcess), Handle(info.hThread));
            WaitForSingleObject(process.0, INFINITE);
            let mut code = 0u32;
            if GetExitCodeProcess(process.0, &mut code) == 0 {
                return Err(last_error("GetExitCodeProcess"));
            }
            Ok(code)
        }
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
            launcher: None,
            low_folder: None,
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
        if std::env::var_os("HOME").is_some() {
            assert!(text.iter().any(|p| p.ends_with(".npm")));
        }
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

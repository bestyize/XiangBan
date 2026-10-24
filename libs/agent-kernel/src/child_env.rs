//! The environment every child process the kernel starts gets: the shell, managed processes and MCP servers.
//!
//! Children start with an empty environment so host secrets (API keys) never leak to them, and get back only
//! what ordinary tools need to work. Two things go beyond the variables themselves:
//! - On Windows, npm, pip and most installers need the profile folders (`APPDATA`, `LOCALAPPDATA`, …).
//! - On macOS an app started from the Dock or Finder gets `PATH=/usr/bin:/bin:/usr/sbin:/sbin`, so Homebrew's
//!   `node`, `npx` and `python3` are not found. The user's login-shell `PATH` is read once and used instead.

use std::ffi::OsString;
use std::process::Command;

#[cfg(not(windows))]
pub(crate) const BASE_ENV: &[&str] = &["PATH", "HOME", "USER", "LOGNAME", "SHELL", "LANG", "LC_ALL", "LC_CTYPE", "TMPDIR", "TZ"];
#[cfg(windows)]
pub(crate) const BASE_ENV: &[&str] = &[
    "PATH", "SystemRoot", "SystemDrive", "ComSpec", "PATHEXT", "TEMP", "TMP", "USERPROFILE", "windir", "APPDATA", "LOCALAPPDATA",
    "PROGRAMDATA", "ProgramFiles", "ProgramFiles(x86)", "ProgramW6432", "CommonProgramFiles", "CommonProgramFiles(x86)", "USERNAME",
    "USERDOMAIN", "HOMEDRIVE", "HOMEPATH", "COMPUTERNAME", "OS", "NUMBER_OF_PROCESSORS", "PROCESSOR_ARCHITECTURE", "PSModulePath",
];

/// Clears `cmd`'s environment and passes through `BASE_ENV` plus `extra`, with `PATH` completed as described above.
pub(crate) fn apply(cmd: &mut Command, extra: &[String]) {
    cmd.env_clear();
    for key in BASE_ENV.iter().copied().chain(extra.iter().map(String::as_str)) {
        if key.eq_ignore_ascii_case("PATH") {
            continue;
        }
        if let Some(value) = std::env::var_os(key) {
            cmd.env(key, value);
        }
    }
    if let Some(path) = path() {
        cmd.env("PATH", path);
    }
}

/// Command tools exchange UTF-8 text with the model. Python otherwise uses the Windows
/// ANSI code page for redirected output and default file IO, which can fail on Chinese text.
/// Explicit host settings still win; MCP servers keep their own environment contract.
#[cfg(any(feature = "shell", feature = "process"))]
pub(crate) fn apply_command(cmd: &mut Command, extra: &[String]) {
    apply(cmd, extra);
    #[cfg(windows)]
    for (key, value) in [("PYTHONUTF8", "1"), ("PYTHONIOENCODING", "utf-8")] {
        if !cmd.get_envs().any(|(name, value)| name.to_string_lossy().eq_ignore_ascii_case(key) && value.is_some()) {
            cmd.env(key, value);
        }
    }
}

/// The `PATH` children get.
pub fn path() -> Option<OsString> {
    #[cfg(target_os = "macos")]
    {
        static PATH: std::sync::OnceLock<Option<OsString>> = std::sync::OnceLock::new();
        PATH.get_or_init(|| merge(std::env::var_os("PATH"), login_shell_path(), &fallback_dirs())).clone()
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var_os("PATH")
    }
}

/// The current entries first, then the login shell's, then well-known install folders; each folder once.
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
fn merge(current: Option<OsString>, login: Option<OsString>, fallback: &[std::path::PathBuf]) -> Option<OsString> {
    let mut seen = std::collections::HashSet::new();
    let mut dirs = Vec::new();
    let lists = [current, login];
    for dir in lists.iter().flatten().flat_map(std::env::split_paths).chain(fallback.iter().cloned()) {
        if !dir.as_os_str().is_empty() && seen.insert(dir.clone()) {
            dirs.push(dir);
        }
    }
    if dirs.is_empty() {
        return None;
    }
    std::env::join_paths(dirs).ok()
}

#[cfg(target_os = "macos")]
fn fallback_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<std::path::PathBuf> = ["/opt/homebrew/bin", "/opt/homebrew/sbin", "/usr/local/bin"].iter().map(Into::into).collect();
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        dirs.extend([".cargo/bin", ".local/bin", ".volta/bin", ".bun/bin"].iter().map(|d| home.join(d)));
    }
    dirs.retain(|d| d.is_dir());
    dirs
}

/// Asks the user's login shell for its `PATH`, giving up after a few seconds (a slow or broken profile must not
/// stall a tool call). The output is framed by markers because profiles often print banners.
#[cfg(target_os = "macos")]
fn login_shell_path() -> Option<OsString> {
    use std::io::Read;
    use std::time::{Duration, Instant};
    const MARK: &str = "__XIAOBAN_PATH__";
    let shell = std::env::var("SHELL").ok().filter(|s| s.starts_with('/')).unwrap_or_else(|| "/bin/zsh".into());
    let mut child = Command::new(&shell)
        .args(["-ilc", &format!("printf '%s%s%s' {MARK} \"$PATH\" {MARK}")])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let end = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < end => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    let mut out = String::new();
    child.stdout.take()?.read_to_string(&mut out).ok()?;
    let inner = out.split(MARK).nth(1)?.trim();
    (!inner.is_empty()).then(|| inner.into())
}

/// The program to start for `command`. On Windows, `Command::new("npx")` finds only `npx.exe`, never the `npx.cmd`
/// Node actually installs, so the usual MCP configurations (`npx`, `uvx`, `pnpm`) fail to start. A bare name is
/// looked up on the children's `PATH` with `PATHEXT`; Rust then starts a found `.cmd` with safe batch quoting.
pub(crate) fn program(command: &str) -> OsString {
    #[cfg(windows)]
    {
        let bare = !command.contains(['/', '\\', ':']) && std::path::Path::new(command).extension().is_none();
        if bare {
            let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
            for dir in path().iter().flat_map(std::env::split_paths) {
                for ext in exts.split(';').filter(|e| !e.is_empty()) {
                    let candidate = dir.join(format!("{command}{ext}"));
                    if candidate.is_file() {
                        return candidate.into_os_string();
                    }
                }
            }
        }
    }
    command.into()
}

/// Hides the console window Windows would otherwise open for every console child of a GUI app.
#[cfg(windows)]
pub(crate) fn no_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

/// `cmd.exe` running one command line exactly as written.
///
/// Rust quotes each argument the way C programs parse them (`"` becomes `\"`), but cmd.exe does not undo that, so
/// `echo "a b"` would print `\"a b\"`. The line is passed raw instead, after `/S /C` with one outer pair of quotes,
/// which cmd strips before running the rest unchanged. `/D` skips AutoRun scripts from the registry.
#[cfg(windows)]
pub(crate) fn windows_shell(command: &str) -> Command {
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("cmd");
    cmd.args(["/D", "/S", "/C"]).raw_arg(format!("\"{command}\""));
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn merging_keeps_the_current_order_first_and_drops_repeats() {
        let join = |dirs: &[&str]| std::env::join_paths(dirs.iter().map(PathBuf::from)).unwrap();
        let merged = merge(Some(join(&["/usr/bin", "/bin"])), Some(join(&["/opt/homebrew/bin", "/usr/bin"])), &[PathBuf::from("/bin"), PathBuf::from("/x")]).unwrap();
        let dirs: Vec<PathBuf> = std::env::split_paths(&merged).collect();
        assert_eq!(dirs, ["/usr/bin", "/bin", "/opt/homebrew/bin", "/x"].map(PathBuf::from));
        assert_eq!(merge(None, None, &[]), None);
    }

    #[test]
    fn children_get_the_base_variables_but_not_secrets() {
        let mut cmd = Command::new("env");
        apply(&mut cmd, &[]);
        let names: Vec<String> = cmd.get_envs().map(|(k, _)| k.to_string_lossy().into_owned()).collect();
        assert!(names.iter().all(|name| BASE_ENV.iter().any(|base| base.eq_ignore_ascii_case(name))), "{names:?}");
    }
}

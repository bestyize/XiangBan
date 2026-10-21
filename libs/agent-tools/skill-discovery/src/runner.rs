//! Explicit, user-approved script execution for installed SkillHub packages.
//! This is NOT a sandbox: interpreter processes retain normal user filesystem/network rights.
use crate::package::{is_script, valid_path};
use agent_kernel::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const MAX_OUTPUT: u64 = 64 * 1024;
const MAX_ARGS: usize = 32;
const MAX_RUNTIME: Duration = Duration::from_secs(120);

pub(crate) struct SkillRun {
    spec: ToolSpec,
    root: PathBuf,
    workspace: PathBuf,
}
impl SkillRun {
    pub(crate) fn new(root: PathBuf, workspace: PathBuf) -> Self {
        let schema = json!({"type":"object","properties":{
            "skill":{"type":"string","description":"Installed SkillHub slug"},
            "script":{"type":"string","description":"Relative scripts/ path of a reviewed file"},
            "args":{"type":"array","items":{"type":"string"},"maxItems":32}
        },"required":["skill","script"]});
        let spec = ToolSpec::new("skill_run",
            "Execute a verified installed SkillHub script with an installed interpreter (Python, Node, shell, PowerShell or Ruby). Always ask user permission; this is not sandboxed. Arguments are literal, never shell-interpreted.",
            schema).risk(Risk::Exec).serial().always_ask().timeout(MAX_RUNTIME);
        Self { spec, root, workspace }
    }
}
impl Tool for SkillRun {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() { return Err("cancelled".into()); }
        let slug = args["skill"].as_str().ok_or("skill is required")?;
        let script = args["script"].as_str().ok_or("script is required")?;
        if slug.len() < 2 || slug.len() > 64 || !slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || slug.starts_with('-') || slug.ends_with('-') {
            return Err("invalid skill name".into());
        }
        let script_path = verified_script(&self.root, slug, script)?;
        let parameters = args.get("args").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        if parameters.len() > MAX_ARGS {
            return Err("too many script arguments".into());
        }
        let mut command = match script.rsplit('.').next().unwrap_or("") {
            "py" => Command::new(if cfg!(windows) { "python" } else { "python3" }),
            "js" | "mjs" | "cjs" => Command::new("node"),
            "sh" => Command::new("sh"),
            "ps1" => Command::new(if cfg!(windows) { "powershell" } else { "pwsh" }),
            "rb" => Command::new("ruby"),
            _ => return Err("unsupported script runtime".into()),
        };
        if script.ends_with(".ps1") {
            command.args(["-NoProfile", "-NonInteractive", "-File"]);
        }
        command.arg(&script_path);
        for p in parameters {
            let value = p.as_str().ok_or("script arguments must be strings")?;
            if value.len() > 2000 || value.contains('\0') {
                return Err("invalid script argument".into());
            }
            command.arg(value);
        }
        command.current_dir(&self.workspace).env_clear().env("XIABAN_SKILL_ROOT", self.root.join(slug))
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(windows)]
        const PASS: &[&str] = &["PATH", "SystemRoot", "ComSpec", "PATHEXT", "TEMP", "TMP", "USERPROFILE"];
        #[cfg(not(windows))]
        const PASS: &[&str] = &["PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "LC_CTYPE", "TMPDIR", "TZ"];
        for key in PASS {
            if let Some(val) = std::env::var_os(key) { command.env(key, val); }
        }
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command.spawn().map_err(|e| format!("unable to start script interpreter: {e}"))?;
        let pid = child.id();
        let (tx, rx) = mpsc::channel();
        for (label, pipe) in [
            ("stdout", child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>)),
            ("stderr", child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>)),
        ] {
            if let Some(mut pipe) = pipe {
                let sender = tx.clone();
                thread::spawn(move || {
                    let mut buf = Vec::new();
                    let result = pipe.take(MAX_OUTPUT + 1).read_to_end(&mut buf)
                        .map(|_| String::from_utf8_lossy(&buf).into_owned())
                        .map_err(|e| e.to_string());
                    let _ = sender.send((label, result));
                });
            }
        }
        drop(tx);
        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if ctx.is_cancelled() || start.elapsed() >= MAX_RUNTIME => {
                    terminate_tree(pid);
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err("script cancelled or timed out".to_string());
                },
                Ok(None) => thread::sleep(Duration::from_millis(40)),
                Err(e) => {
                    terminate_tree(pid);
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(e.to_string());
                }
            }
        };
        let mut outputs = Vec::new();
        for _ in 0..2 {
            if let Ok((kind, result)) = rx.recv_timeout(Duration::from_secs(1)) {
                outputs.push(format!("{kind}: {}", result?));
            }
        }
        let text = outputs.join("\n");
        let status = status?;
        if status.success() {
            Ok(format!("{text}\n[exit code: 0]"))
        } else {
            Err(format!("{text}\n[script exited: {status}]"))
        }
    }
}

/// Read a referenced source or document without granting general filesystem access.
pub(crate) struct SkillRead { spec: ToolSpec, root: PathBuf }
impl SkillRead {
    pub(crate) fn new(root: PathBuf) -> Self {
        let spec = ToolSpec::new("skill_read",
            "Read one UTF-8 file from a verified installed SkillHub package (references, scripts, assets or SKILL.md); binary assets are not supported by this reader.",
            json!({"type":"object","properties":{"skill":{"type":"string"},"path":{"type":"string"}},
                "required":["skill","path"]}))
            .risk(Risk::Read).parallel().untrusted_output();
        Self { spec, root }
    }
}
impl Tool for SkillRead {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() { return Err("cancelled".into()); }
        let slug = args["skill"].as_str().ok_or("skill is required")?;
        let path = args["path"].as_str().ok_or("path is required")?;
        if slug.len() < 2 || slug.len() > 64 ||
            !slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || slug.starts_with('-') || slug.ends_with('-') || !valid_path(path) {
            return Err("invalid skill or path".into());
        }
        let file = verified_package_file(&self.root, slug, path)?;
        if fs::metadata(&file).map_err(|e|e.to_string())?.len() > 100 * 1024 {
            return Err("skill_read only supports files <=100 KiB".into());
        }
        fs::read_to_string(file).map_err(|_| "not a UTF-8 text file".into())
    }
}

fn verified_script(root: &Path, skill: &str, path: &str) -> Result<PathBuf, String> {
    if !valid_path(path) || !is_script(path) {
        return Err("only files under scripts/ with a supported extension may run".into());
    }
    verified_package_file(root, skill, path)
}

pub(crate) fn verified_package_file(root: &Path, skill: &str, path: &str) -> Result<PathBuf, String> {
    if !valid_path(path) { return Err("invalid skill file path".into()); }
    let dir = root.join(skill);
    let root_real = dir.canonicalize().map_err(|_| "SkillHub skill is not installed")?;
    let lock: Value = serde_json::from_slice(&fs::read(dir.join(".skillhub-lock.json"))
        .map_err(|_| "this skill has no verified SkillHub manifest; manual skills use the normal shell tool")?)
        .map_err(|e| format!("invalid SkillHub lock: {e}"))?;
    if lock["provider"] != "skillhub" || lock["slug"] != skill {
        return Err("SkillHub lock identity mismatch".into());
    }
    let items = lock["files"].as_array().ok_or("SkillHub lock missing files")?;
    if items.is_empty() || items.len() > 100 { return Err("invalid SkillHub lock".into()); }
    let mut script_exists = false;
    for item in items {
        let name = item["path"].as_str().ok_or("invalid locked path")?;
        let expected = item["sha256"].as_str().ok_or("invalid locked hash")?;
        if !valid_path(name) || expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid SkillHub manifest entry".into());
        }
        // Resolve symlinks but reject any path escaping the installed skill.
        let file = dir.join(name).canonicalize().map_err(|e| format!("missing SkillHub file {name}: {e}"))?;
        if !file.starts_with(&root_real) || !file.is_file() {
            return Err(format!("unsafe SkillHub file: {name}"));
        }
        let metadata = fs::metadata(&file).map_err(|e| e.to_string())?;
        if metadata.len() > 4 * 1024 * 1024 {
            return Err(format!("SkillHub file too large: {name}"));
        }
        let bytes = fs::read(&file).map_err(|e| e.to_string())?;
        if format!("{:x}", Sha256::digest(&bytes)) != expected.to_ascii_lowercase() {
            return Err(format!("SkillHub file changed after installation: {name}"));
        }
        if name == path { script_exists = true; }
    }
    if !script_exists { return Err("file not listed in verified SkillHub package".into()); }
    let script = dir.join(path).canonicalize().map_err(|e| e.to_string())?;
    if !script.starts_with(root_real) { return Err("script escapes skill directory".into()); }
    Ok(script)
}

fn terminate_tree(pid: u32) {
    #[cfg(unix)]
    let _ = Command::new("sh").args(["-c", &format!("kill -KILL -- -{pid}")])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
    #[cfg(windows)]
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_gate_blocks_unlisted_and_escaped_scripts() {
        let temp = std::env::temp_dir().join(format!("skill-run-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        assert!(verified_script(&temp, "demo", "../evil.py").is_err());
        assert!(verified_script(&temp, "demo", "assets/code.py").is_err());
        assert!(verified_script(&temp, "demo", "scripts/evil.py").is_err());
        fs::remove_dir_all(&temp).unwrap();
    }
}

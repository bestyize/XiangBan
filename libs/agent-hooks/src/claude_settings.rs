//! Plan and safely apply Claude Code settings changes.
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
/// Errors encountered when preparing or applying a settings change.
#[derive(Debug, Error)]
pub enum HooksError {
    /// Filesystem failure.
    #[error("settings I/O: {0}")]
    Io(#[from] io::Error),
    /// JSON or JSONC is not valid JSON.
    #[error("settings must be valid JSON (JSONC is not supported): {0}")]
    Json(#[from] serde_json::Error),
    /// Refusing to overwrite a structure whose shape is unknown.
    #[error("unsupported settings structure: {0}")]
    Structure(String),
    /// The file changed after the plan was created.
    #[error("settings changed since the plan was created")]
    Stale,
}
/// Configuration scope; project paths point to the project root.
#[derive(Clone, Debug)]
pub enum Scope {
    /// User configuration, honoring CLAUDE_CONFIG_DIR.
    User,
    /// Shared project settings.
    Project(PathBuf),
    /// Local project settings.
    ProjectLocal(PathBuf),
}
/// Desired executable and events.
#[derive(Clone, Debug)]
pub struct InstallSpec {
    /// Absolute executable path.
    pub forwarder: PathBuf,
    /// Events to install.
    pub events: Vec<String>,
    /// Timeout in seconds for ordinary events.
    pub timeout_secs: u32,
}
impl InstallSpec {
    /// Defaults to the eight low-noise events and a 30-second ordinary timeout.
    pub fn new(forwarder: PathBuf) -> Self {
        Self {
            forwarder,
            events: [
                "PermissionRequest",
                "Notification",
                "Stop",
                "StopFailure",
                "SubagentStop",
                "UserPromptSubmit",
                "SessionStart",
                "SessionEnd",
            ]
            .iter()
            .map(ToString::to_string)
            .collect(),
            timeout_secs: 30,
        }
    }
}
/// A human-readable change.
#[derive(Clone, Debug)]
pub struct Change {
    /// Event being changed.
    pub event: String,
    /// Short operation description.
    pub description: String,
}
/// Immutable plan to review before applying.
#[derive(Clone, Debug)]
pub struct Plan {
    /// Target settings file.
    pub path: PathBuf,
    /// Original file contents.
    pub before: String,
    /// Proposed file contents.
    pub after: String,
    /// Summary of proposed changes.
    pub changes: Vec<Change>,
    existed: bool,
    created: Vec<Created>,
    uninstall: bool,
}
/// Applied result with the backup path, when a file existed.
#[derive(Clone, Debug)]
pub struct Applied {
    /// Path of the backup, if an existing file changed.
    pub backup: Option<PathBuf>,
}
/// Hook installation status.
#[derive(Clone, Debug, Default)]
pub struct Status {
    /// Event, executable path and whether the path exists.
    pub installed: Vec<(String, PathBuf, bool)>,
    /// Whether any other handler was observed.
    pub other_hooks: bool,
}
/// Detected third-party integration.
#[derive(Clone, Debug)]
pub struct Coexisting {
    /// Detected integration name.
    pub name: String,
    /// File containing the matching configuration.
    pub source: PathBuf,
}
fn path(scope: &Scope) -> Result<PathBuf, HooksError> {
    Ok(match scope {
        Scope::User => std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .map(|p| PathBuf::from(p).join(".claude"))
            })
            .ok_or_else(|| HooksError::Structure("home directory unavailable".into()))?
            .join("settings.json"),
        Scope::Project(p) => p.join(".claude/settings.json"),
        Scope::ProjectLocal(p) => p.join(".claude/settings.local.json"),
    })
}
fn load(path: &Path) -> Result<(String, Value, bool), HooksError> {
    match fs::read_to_string(path) {
        Ok(s) => {
            let v: Value = serde_json::from_str(&s)?;
            if !v.is_object() {
                return Err(HooksError::Structure("root must be an object".into()));
            }
            Ok((s, v, true))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok((String::new(), json!({}), false)),
        Err(e) => Err(e.into()),
    }
}
fn owned(v: &Value) -> bool {
    let command = v.get("command").and_then(Value::as_str).unwrap_or("");
    let name = command.rsplit(['/', '\\']).next().unwrap_or("");
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    v.get("type").and_then(Value::as_str) == Some("command")
        && stem == "xiaoban-hook"
        && v.get("args")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(Value::as_str)
            .is_some()
}
fn format_value(v: &Value, before: &str) -> Result<String, HooksError> {
    let indent = before
        .lines()
        .skip(1)
        .find_map(|line| {
            let prefix = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
            if prefix.is_empty() {
                None
            } else if prefix.starts_with('\t') {
                Some("\t")
            } else if prefix.len() >= 4 {
                Some("    ")
            } else {
                Some("  ")
            }
        })
        .unwrap_or("  ");
    let mut out = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(
        &mut out,
        serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes()),
    );
    serde::Serialize::serialize(v, &mut ser)?;
    let mut text =
        String::from_utf8(out).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if before.ends_with("\r\n") {
        text = text.replace('\n', "\r\n");
        text.push_str("\r\n");
    } else if before.ends_with('\n') || before.is_empty() {
        text.push('\n');
    }
    Ok(text)
}
fn groups_mut<'a>(root: &'a mut Value, event: &str) -> Result<&'a mut Vec<Value>, HooksError> {
    let obj = root.as_object_mut().unwrap();
    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| HooksError::Structure("hooks is not an object".into()))?;
    hooks
        .entry(event)
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| HooksError::Structure(format!("hooks.{event} is not an array")))
}
/// Build an install plan without writing any files.
pub fn plan_install(scope: &Scope, spec: &InstallSpec) -> Result<Plan, HooksError> {
    if !spec.forwarder.is_absolute()
        || !owned(
            &json!({"type":"command","command":spec.forwarder.to_string_lossy(),"args":["x"]}),
        )
    {
        return Err(HooksError::Structure(
            "forwarder must be an absolute path named xiaoban-hook[.exe]".into(),
        ));
    }
    if spec.timeout_secs == 0 {
        return Err(HooksError::Structure("timeout must be positive".into()));
    }
    let path = path(scope)?;
    let (before, mut root, existed) = load(&path)?;
    let mut changes = Vec::new();
    let mut created = Vec::new();
    for event in spec.events.iter().collect::<BTreeSet<_>>() {
        if event.is_empty()
            || !event
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err(HooksError::Structure("invalid event name".into()));
        }
        let hooks_created = root.get("hooks").is_none();
        let event_created = root
            .get("hooks")
            .and_then(Value::as_object)
            .is_none_or(|h| !h.contains_key(event));
        let groups = groups_mut(&mut root, event)?;
        let mut found = false;
        for group in groups.iter_mut() {
            if let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                for handler in handlers.iter_mut() {
                    if owned(handler)
                        && handler
                            .get("args")
                            .and_then(Value::as_array)
                            .and_then(|a| a.first())
                            .and_then(Value::as_str)
                            == Some(event)
                    {
                        let desired = json!({"type":"command","command":spec.forwarder.to_string_lossy(),"args":[event],"timeout":if event == "PermissionRequest" {130} else {spec.timeout_secs}});
                        if *handler != desired {
                            *handler = desired;
                            changes.push(Change {
                                event: event.clone(),
                                description: "update".into(),
                            });
                        }
                        found = true;
                    }
                }
            }
        }
        if !found {
            let index = groups.len();
            groups.push(json!({"hooks":[{"type":"command","command":spec.forwarder.to_string_lossy(),"args":[event],"timeout":if event == "PermissionRequest" {130} else {spec.timeout_secs}}]}));
            created.push(Created {
                event: event.clone(),
                index,
                event_created,
                hooks_created,
            });
            changes.push(Change {
                event: event.clone(),
                description: "append".into(),
            });
        }
    }
    let after = if changes.is_empty() {
        before.clone()
    } else {
        format_value(&root, &before)?
    };
    Ok(Plan {
        path,
        before,
        after,
        changes,
        existed,
        created,
        uninstall: false,
    })
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Created {
    event: String,
    index: usize,
    event_created: bool,
    hooks_created: bool,
}
fn metadata_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.xiaoban-state", path.display()))
}
fn created_events(path: &Path) -> Vec<Created> {
    fs::read(metadata_path(path))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
/// Build an uninstall plan, retaining other tools and unknown structures.
pub fn plan_uninstall(scope: &Scope) -> Result<Plan, HooksError> {
    let path = path(scope)?;
    let (before, mut root, existed) = load(&path)?;
    let created = created_events(&path);
    let mut changes = Vec::new();
    if let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) {
        let keys: Vec<String> = hooks.keys().cloned().collect();
        for key in keys {
            if let Some(groups) = hooks.get_mut(&key).and_then(Value::as_array_mut) {
                let owned_indices: Vec<usize> = created
                    .iter()
                    .filter(|c| c.event == key)
                    .map(|c| c.index)
                    .collect();
                let mut remove_indices = Vec::new();
                for (i, g) in groups.iter_mut().enumerate() {
                    let removable = owned_indices.contains(&i)
                        && g.as_object().is_some_and(|o| o.len() == 1)
                        && g.get("hooks")
                            .and_then(Value::as_array)
                            .is_some_and(|a| a.len() == 1 && owned(&a[0]));
                    if let Some(handlers) = g.get_mut("hooks").and_then(Value::as_array_mut) {
                        let n = handlers.len();
                        handlers.retain(|v| !owned(v));
                        if handlers.len() != n {
                            changes.push(Change {
                                event: key.clone(),
                                description: "remove".into(),
                            });
                        }
                    }
                    if removable {
                        remove_indices.push(i);
                    }
                }
                for i in remove_indices.into_iter().rev() {
                    groups.remove(i);
                }
                if groups.is_empty() && created.iter().any(|c| c.event == key && c.event_created) {
                    hooks.remove(&key);
                }
            }
        }
        if hooks.is_empty() && created.iter().any(|c| c.hooks_created) {
            root.as_object_mut().unwrap().remove("hooks");
        }
    }
    let mut after = if changes.is_empty() {
        before.clone()
    } else {
        format_value(&root, &before)?
    };
    // Restore exact original formatting only when a backup is semantically identical.
    if !changes.is_empty() {
        for b in backups(&path)? {
            if let Ok(s) = fs::read_to_string(b) {
                if serde_json::from_str::<Value>(&s).ok().as_ref() == Some(&root) {
                    after = s;
                    break;
                }
            }
        }
    }
    Ok(Plan {
        path,
        before,
        after,
        changes,
        existed,
        created: Vec::new(),
        uninstall: true,
    })
}
fn backups(path: &Path) -> Result<Vec<PathBuf>, HooksError> {
    let Some(parent) = path.parent() else {
        return Ok(Vec::new());
    };
    if !parent.exists() {
        return Ok(Vec::new());
    }
    let prefix = format!(
        "{}.xiaoban-backup-",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    let mut found = Vec::new();
    for e in fs::read_dir(parent)? {
        let e = e?;
        if e.file_name().to_string_lossy().starts_with(&prefix) {
            found.push(e.path());
        }
    }
    found.sort();
    found.reverse();
    Ok(found)
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), HooksError> {
    let parent = path
        .parent()
        .ok_or_else(|| HooksError::Structure("missing parent".into()))?;
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    for attempt in 0..3 {
        match temp.persist(path) {
            Ok(_) => return Ok(()),
            Err(e) => {
                temp = e.file;
                if attempt == 2 {
                    return Err(e.error.into());
                }
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
        }
    }
    unreachable!()
}
/// Back up and atomically apply a plan, rejecting stale plans.
pub fn apply(plan: &Plan) -> Result<Applied, HooksError> {
    let (current, _, exists) = load(&plan.path)?;
    if current != plan.before || exists != plan.existed {
        return Err(HooksError::Stale);
    }
    if plan.before == plan.after {
        return Ok(Applied { backup: None });
    }
    fs::create_dir_all(
        plan.path
            .parent()
            .ok_or_else(|| HooksError::Structure("missing parent".into()))?,
    )?;
    let backup = if exists {
        let name = format!(
            "{}.xiaoban-backup-{}",
            plan.path.display(),
            Utc::now().format("%Y%m%dT%H%M%S%.9fZ")
        );
        let b = PathBuf::from(name);
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&b)?;
        f.write_all(current.as_bytes())?;
        f.sync_all()?;
        Some(b)
    } else {
        None
    };
    let mut created = created_events(&plan.path);
    for e in &plan.created {
        created.push(e.clone());
    }
    // Persist provenance before changing settings; a failure leaves the user's file intact.
    if !created.is_empty() {
        atomic_write(&metadata_path(&plan.path), &serde_json::to_vec(&created)?)?;
    }
    atomic_write(&plan.path, plan.after.as_bytes())?;
    if plan.uninstall {
        let _ = fs::remove_file(metadata_path(&plan.path));
    }
    for b in backups(&plan.path)?.into_iter().skip(5) {
        fs::remove_file(b)?;
    }
    Ok(Applied { backup })
}
/// Inspect installed handlers and other hook presence.
pub fn status(scope: &Scope) -> Result<Status, HooksError> {
    let (_, root, _) = load(&path(scope)?)?;
    let mut status = Status::default();
    if let Some(hooks) = root.get("hooks").and_then(Value::as_object) {
        for (event, groups) in hooks {
            if let Some(groups) = groups.as_array() {
                for group in groups {
                    if let Some(handlers) = group.get("hooks").and_then(Value::as_array) {
                        for h in handlers {
                            if owned(h) {
                                let p = PathBuf::from(
                                    h.get("command").and_then(Value::as_str).unwrap_or(""),
                                );
                                status
                                    .installed
                                    .push((event.clone(), p.clone(), p.exists()));
                            } else {
                                status.other_hooks = true;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(status)
}
/// Detect known voice integrations in user settings and MCP registration files, read-only.
pub fn detect_coexisting() -> Vec<Coexisting> {
    let mut paths = Vec::new();
    if let Ok(p) = path(&Scope::User) {
        paths.push(p);
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        paths.push(PathBuf::from(home).join(".claude.json"));
    }
    let mut result = Vec::new();
    for path in paths {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<Value>(&content) {
                let s = v.to_string().to_lowercase();
                for name in ["mcp-voice-hooks", "voicemode"] {
                    if s.contains(name) {
                        result.push(Coexisting {
                            name: name.into(),
                            source: path.clone(),
                        });
                    }
                }
            }
        }
    }
    result
}
/// Restore the newest backup, first backing up the current settings.
pub fn restore_latest_backup(scope: &Scope) -> Result<(), HooksError> {
    let path = path(scope)?;
    let backup = backups(&path)?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no backup"))?;
    let after = fs::read_to_string(backup)?;
    let _: Value = serde_json::from_str(&after)?;
    let (before, _, existed) = load(&path)?;
    apply(&Plan {
        path,
        before,
        after,
        changes: Vec::new(),
        existed,
        created: Vec::new(),
        uninstall: false,
    })?;
    Ok(())
}

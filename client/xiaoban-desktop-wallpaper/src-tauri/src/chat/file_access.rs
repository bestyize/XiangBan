//! One-operation access to an explicitly named file or directory outside the conversation workspace.
//! The normal filesystem tools remain rooted in the workspace. An approval only creates a temporary
//! tool for the requested canonical path; it never adds a persistent directory grant.

use agent_file_tools::{AttachmentStore, ReadFile};
use agent_kernel::builtin::fs::{fs_tools, fs_write_tools};
use agent_kernel::{Risk, Tool, ToolCtx, ToolResource, ToolResult, ToolSpec};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use std::collections::HashSet;

pub(super) type ApprovePath = dyn Fn(&str, &Path, Risk, &Value, &ToolCtx) -> bool + Send + Sync;
pub const EXTERNAL_WRITE_NOTICE: &str = "修改工作区外文件，必须确认具体目标路径";

/// Evaluate live session policy so changing modes never leaves an old per-agent grant behind.
pub fn trusted_path_access(permission: &agent_kernel::Permission, ctx: &ToolCtx) -> bool {
    !ctx.is_cancelled() && permission.mode_for(ctx.session()) == agent_kernel::PermissionMode::Auto
}

type PathGrant = (String, String, String, PathBuf);
fn path_grants() -> &'static Mutex<HashSet<PathGrant>> {
    static GRANTS: OnceLock<Mutex<HashSet<PathGrant>>> = OnceLock::new();
    GRANTS.get_or_init(Default::default)
}

/// Permission approved this precise outside-workspace operation. The file tool consumes it once.
pub fn allow_path_once(session: &str, run: &str, tool: &str, path: PathBuf) {
    path_grants().lock().unwrap_or_else(|e| e.into_inner()).insert((session.into(), run.into(), tool.into(), path));
}

pub fn take_path_approval(session: &str, run: &str, tool: &str, path: &Path) -> bool {
    path_grants().lock().unwrap_or_else(|e| e.into_inner()).remove(&(session.into(), run.into(), tool.into(), path.to_path_buf()))
}

pub fn forget_session(session: &str) {
    path_grants().lock().unwrap_or_else(|e| e.into_inner()).retain(|(s, _, _, _)| s.as_str() != session);
}

pub fn resolve_requested(workspace: &Path, tool: &str, path: &str) -> Result<PathBuf, String> {
    if path.is_empty() { return Err("path must not be empty".into()); }
    let requested = Path::new(path);
    if matches!(tool, "write_file" | "edit_file") && requested.components().any(|c| c == Component::ParentDir) {
        return Err("`..` is not allowed in paths that are written".into());
    }
    let joined = if requested.is_absolute() { requested.to_path_buf() } else { workspace.join(requested) };
    if tool == "write_file" && !joined.exists() {
        let mut ancestor = joined.as_path();
        let mut tail = Vec::new();
        while !ancestor.exists() {
            tail.push(ancestor.file_name().ok_or("invalid file path")?.to_os_string());
            ancestor = ancestor.parent().ok_or("invalid file path")?;
        }
        let mut canonical = ancestor.canonicalize().map_err(|e| format!("cannot access path: {e}"))?;
        for part in tail.iter().rev() { canonical.push(part); }
        Ok(canonical)
    } else {
        joined.canonicalize().map_err(|e| format!("cannot access path: {e}"))
    }
}

pub fn is_external_write(tool: &str, args: &Value, workspace: &Path) -> bool {
    if !matches!(tool, "write_file" | "edit_file") { return false; }
    let Some(requested) = args.get("path").and_then(Value::as_str) else { return false; };
    let Ok(base) = workspace.canonicalize() else { return true; };
    // When uncertain, require confirmation. The subsequent tool still validates the path.
    resolve_requested(workspace, tool, requested).map_or(true, |path| !path.starts_with(base))
}

pub struct ScopedFileTool {
    original: Arc<dyn Tool>,
    spec: ToolSpec,
    workspace: PathBuf,
    roots: Vec<PathBuf>,
    attachments: PathBuf,
    approve: Arc<ApprovePath>,
}

impl ScopedFileTool {
    pub fn new(
        original: Arc<dyn Tool>, workspace: &Path, roots: Vec<PathBuf>, attachments: PathBuf,
        approve: Arc<ApprovePath>,
    ) -> Self {
        let mut spec = original.spec().clone();
        // Asking the user happens in the tool, not the permission middleware. Leave time to answer.
        spec.timeout = Duration::from_secs(600);
        Self { original, spec, workspace: workspace.to_path_buf(), roots, attachments, approve }
    }

    fn named_path(&self, args: &Value) -> Result<PathBuf, String> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        resolve_requested(&self.workspace, &self.spec.name, path)
    }

    fn inside(&self, path: &Path) -> bool {
        self.roots.iter().filter_map(|root| root.canonicalize().ok()).any(|root| path.starts_with(root))
    }

    fn outside_tool(&self, requested: &Path) -> Result<Arc<dyn Tool>, String> {
        let root = if self.spec.name == "list_dir" && requested.is_dir() || self.spec.name == "grep" && requested.is_dir() {
            requested.to_path_buf()
        } else {
            // The parent is a transient implementation detail, not an authorized shared root.
            // For a new nested file, use the nearest existing ancestor.
            let mut parent = requested.parent().ok_or("file has no parent")?;
            while !parent.is_dir() { parent = parent.parent().ok_or("file has no existing parent")?; }
            parent.to_path_buf()
        };
        let roots = vec![self.workspace.clone(), root];
        let tools = if self.spec.risk == Risk::Write {
            fs_write_tools(roots).map_err(|e| e.to_string())?
        } else if self.spec.name == "read_file" {
            vec![Arc::new(ReadFile::new(roots, AttachmentStore::new(&self.attachments)).map_err(|e| e.to_string())?) as Arc<dyn Tool>]
        } else {
            fs_tools(roots).map_err(|e| e.to_string())?
        };
        tools.into_iter().find(|tool| tool.spec().name == self.spec.name).ok_or_else(|| "file tool unavailable".into())
    }
}

impl Tool for ScopedFileTool {
    fn spec(&self) -> &ToolSpec { &self.spec }

    fn resource(&self, args: &Value) -> Result<Option<ToolResource>, String> {
        if self.spec.risk == Risk::Write {
            return Ok(Some(ToolResource::File(self.named_path(args)?)));
        }
        self.original.resource(args)
    }

    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        let requested = match self.named_path(&args) {
            Ok(path) => path,
            Err(error) => return ToolResult::error(error),
        };
        // Original read_file also recognizes verified installed skill files and per-session attachments.
        // Check it first, so those established capabilities never generate an unnecessary prompt.
        if self.inside(&requested) || self.spec.name == "read_file" {
            let result = self.original.execute_result(args.clone(), ctx);
            if self.inside(&requested) || !result.is_error || self.spec.name != "read_file"
                || !result.output.contains("outside the authorized workspace") {
                return result;
            }
        }
        if ctx.is_cancelled() { return ToolResult::error("cancelled"); }
        if !(self.approve)(&self.spec.name, &requested, self.spec.risk, &args, ctx) || ctx.is_cancelled() {
            return ToolResult::error("用户未授权访问此工作区外路径；不要改用其他工具绕过拒绝。");
        }
        // Refuse a switched symlink or newly resolved destination after consent.
        if self.named_path(&args).ok().as_deref() != Some(requested.as_path()) {
            return ToolResult::error("file path changed after authorization; request access again");
        }
        match self.outside_tool(&requested) {
            Ok(tool) => tool.execute_result(args, ctx),
            Err(error) => ToolResult::error(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn fixture() -> (PathBuf, PathBuf, PathBuf) {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let base = std::env::temp_dir().join(format!("file-access-{}-{nanos}", std::process::id()));
        let workspace = base.join("workspace");
        let outside = base.join("outside");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        (base, workspace, outside)
    }

    #[test]
    fn full_trust_accesses_external_files_and_reverts_immediately_in_ask_mode() {
        let (base, workspace, outside) = fixture();
        let permission = Arc::new(agent_kernel::Permission::new(agent_kernel::PermissionMode::Ask, |_: &agent_kernel::ApprovalRequest<'_>| agent_kernel::Approval::Deny).children_inherit());
        permission.set_session_mode("trusted", agent_kernel::PermissionMode::Auto);
        let p = permission.clone();
        let approve: Arc<ApprovePath> = Arc::new(move |_, _, _, _, ctx| trusted_path_access(&p, ctx));
        let original = fs_write_tools(vec![workspace.clone()]).unwrap().into_iter().find(|t| t.spec().name == "write_file").unwrap();
        let scoped = ScopedFileTool::new(original, &workspace, vec![workspace.clone()], base.join("attachments"), approve.clone());
        let target = outside.join("created.txt");
        let args = serde_json::json!({"path":target,"content":"trusted write"});
        let ctx = ToolCtx::detached("trusted#team-worker");
        assert!(!scoped.execute_result(args.clone(), &ctx).is_error);
        let original = ReadFile::new(vec![workspace.clone()], AttachmentStore::new(base.join("attachments"))).unwrap();
        let read = ScopedFileTool::new(Arc::new(original), &workspace, vec![workspace.clone()], base.join("attachments"), approve);
        assert!(read.execute_result(serde_json::json!({"path":target}), &ctx).output.contains("trusted write"));
        assert!(scoped.execute_result(args.clone(), &ToolCtx::detached("other")).is_error);
        permission.set_session_mode("trusted", agent_kernel::PermissionMode::Ask);
        assert!(scoped.execute_result(args, &ctx).is_error);
        assert!(read.execute_result(serde_json::json!({"path":target}), &ctx).is_error);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn outside_read_needs_exact_approval_and_denial_does_not_fall_back() {
        let (base, workspace, outside) = fixture();
        std::fs::write(outside.join("secret.txt"), "external content").unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let approve: Arc<ApprovePath> = Arc::new(move |_, _, _, _, _| { counter.fetch_add(1, Ordering::SeqCst); false });
        let roots = vec![workspace.clone()];
        let original: Arc<dyn Tool> = Arc::new(ReadFile::new(roots.clone(), AttachmentStore::new(base.join("attachments"))).unwrap());
        let tool = ScopedFileTool::new(original, &workspace, roots, base.join("attachments"), approve);
        let result = tool.execute_result(serde_json::json!({"path":outside.join("secret.txt")}), &ToolCtx::detached("s"));
        assert!(result.is_error && !result.output.contains("external content"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn approved_file_does_not_grant_a_sibling_or_another_operation() {
        let (base, workspace, outside) = fixture();
        std::fs::write(outside.join("one.txt"), "allowed text").unwrap();
        std::fs::write(outside.join("two.txt"), "different text").unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let approve: Arc<ApprovePath> = Arc::new(move |_, _, _, _, _| { counter.fetch_add(1, Ordering::SeqCst); true });
        let roots = vec![workspace.clone()];
        let original: Arc<dyn Tool> = Arc::new(ReadFile::new(roots.clone(), AttachmentStore::new(base.join("attachments"))).unwrap());
        let tool = ScopedFileTool::new(original, &workspace, roots, base.join("attachments"), approve);
        let ctx = ToolCtx::detached("s");
        for name in ["one.txt", "two.txt"] {
            let result = tool.execute_result(serde_json::json!({"path":outside.join(name)}), &ctx);
            assert!(!result.is_error, "{}", result.output);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 2, "each external file must be approved independently");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn approved_external_directory_search_and_new_file_write() {
        let (base, workspace, outside) = fixture();
        std::fs::write(outside.join("note.txt"), "unique needle").unwrap();
        let approve: Arc<ApprovePath> = Arc::new(|_, _, _, _, _| true);
        let grep = fs_tools(vec![workspace.clone()]).unwrap().into_iter().find(|t| t.spec().name == "grep").unwrap();
        let scoped = ScopedFileTool::new(grep, &workspace, vec![workspace.clone()], base.join("attachments"), approve.clone());
        let result = scoped.execute_result(serde_json::json!({"path":outside,"pattern":"unique needle"}), &ToolCtx::detached("s"));
        assert!(!result.is_error && result.output.contains("unique needle"), "{}", result.output);

        let write = fs_write_tools(vec![workspace.clone()]).unwrap().into_iter().find(|t| t.spec().name == "write_file").unwrap();
        let scoped = ScopedFileTool::new(write, &workspace, vec![workspace.clone()], base.join("attachments"), approve);
        let target = outside.join("new").join("created.txt");
        let result = scoped.execute_result(serde_json::json!({"path":target,"content":"written"}), &ToolCtx::detached("s"));
        assert!(!result.is_error, "{}", result.output);
        assert_eq!(std::fs::read_to_string(target).unwrap(), "written");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn outside_write_requires_a_run_bound_exact_path_ticket() {
        let (base, workspace, outside) = fixture();
        let first = outside.join("first.txt");
        let second = outside.join("second.txt");
        std::fs::write(&first, "one").unwrap();
        assert!(is_external_write("edit_file", &serde_json::json!({"path":first}), &workspace));
        assert!(!is_external_write("write_file", &serde_json::json!({"path":workspace.join("local.txt")}), &workspace));
        allow_path_once("s", "r", "edit_file", first.clone());
        assert!(!take_path_approval("s", "other", "edit_file", &first));
        assert!(!take_path_approval("s", "r", "edit_file", &second));
        assert!(take_path_approval("s", "r", "edit_file", &first));
        assert!(!take_path_approval("s", "r", "edit_file", &first));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_inside_the_workspace_still_targets_the_external_file() {
        let (base, workspace, outside) = fixture();
        let target = outside.join("linked.txt");
        std::fs::write(&target, "out-of-workspace").unwrap();
        std::os::unix::fs::symlink(&target, workspace.join("link.txt")).unwrap();
        assert!(is_external_write("edit_file", &serde_json::json!({"path":workspace.join("link.txt")}), &workspace));
        assert_eq!(resolve_requested(&workspace, "read_file", "link.txt").unwrap(), target);
        let _ = std::fs::remove_dir_all(&base);
    }
}

#[cfg(test)]
mod wait_regressions {
    use super::*;
    use agent_kernel::{Agent, MemoryStore, RunControl, RunOptions, Part};
    use agent_kernel::provider::scripted::{ScriptedProvider, Step};
    use std::sync::atomic::AtomicBool;
    #[test]
    fn waiting_for_external_read_consent_does_not_spend_the_run_budget() {
        let base = std::env::temp_dir().join(format!("file-consent-budget-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let work = base.join("work"); std::fs::create_dir_all(&work).unwrap();
        let external = base.join("external.txt"); std::fs::write(&external, "approved data").unwrap();
        let original = ReadFile::new(vec![work.clone()], AttachmentStore::new(base.join("attachments"))).unwrap();
        let approve: Arc<ApprovePath> = Arc::new(|_, _, _, _, ctx| {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(450)); let _ = tx.send(true); });
            ctx.wait_for_user(&rx).unwrap_or(false)
        });
        let scoped = ScopedFileTool::new(Arc::new(original), &work, vec![work.clone()], base.join("attachments"), approve);
        let provider = Arc::new(ScriptedProvider::new(vec![Step::tool_call("read", "read_file", &serde_json::json!({"path":external}).to_string()), Step::text("done")]));
        let agent = Agent::builder(provider, Arc::new(MemoryStore::new())).tool(scoped).build().unwrap();
        let control = RunControl::bounded(Duration::from_millis(300), 1000);
        let result = agent.run_parts_observed("s1", vec![Part::Text { text:"read".into() }], RunOptions { control:Some(control.clone()), ..Default::default() }, &mut |_| Ok(()), &Arc::new(AtomicBool::new(false)));
        assert!(result.is_ok(), "{result:?}"); assert!(control.excluded_wait() >= Duration::from_millis(400));
        std::fs::remove_dir_all(base).unwrap();
    }
}

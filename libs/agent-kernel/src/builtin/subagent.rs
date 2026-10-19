use super::str_arg;
use crate::agent::Agent;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use crate::types::Event;
use crate::HarnessError;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const MAX_TASK_CHARS: usize = 20_000;
const POLL: Duration = Duration::from_millis(20);

type Observer = Box<dyn Fn(&str, &Event) + Send + Sync>;
type FinishHook = Box<dyn Fn(&str) + Send + Sync>;
type RunObserver = Box<dyn Fn(&crate::RunEvent) -> Result<(), HarnessError> + Send + Sync>;

static CHILD_SEQ: AtomicU64 = AtomicU64::new(0);

/// Unique across calls, instances and restarts, so a child never picks up another job's history
/// from a persistent store.
fn child_session_id(parent: &str, tool: &str) -> String {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    format!("{parent}#{tool}-{millis:x}-{:x}-{}", std::process::id(), CHILD_SEQ.fetch_add(1, Ordering::Relaxed))
}

/// Lets the model hand a self-contained job to a second agent and get back only its final answer,
/// which keeps the parent's context small.
///
/// The host builds the child `Agent` with whatever tools, prompt and store it should have. A child
/// can never contain the tool that wraps it, so delegation depth is fixed when the host builds it.
/// The child runs under its own middleware: pass it the same `Arc<Permission>` as the parent, or
/// its tools are not gated. Every call runs in a fresh session named
/// `<parent session>#<tool name>-<time>-<pid>-<n>`, so a persistent child store never mixes jobs;
/// use `MemoryStore` for the child unless the transcripts should be kept, and `on_finish` to drop
/// per-session state (permission grants, todo lists, stored messages) when a job ends. Token usage
/// of the child is not added to the parent's.
pub struct Subagent {
    spec: ToolSpec,
    child: Arc<Agent>,
    observer: Option<Observer>,
    on_finish: Option<FinishHook>,
    run_observer: Option<RunObserver>,
}

impl Subagent {
    /// `risk` is what a call to this tool counts as for the parent's `Permission`. Use `Risk::Read`
    /// for a read-only child whose own tools are gated, and a higher level when the child can act
    /// without further checks.
    pub fn new(name: &str, description: &str, child: Arc<Agent>, risk: Risk) -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "task": { "type": "string", "description": "Complete instructions. The sub-agent sees nothing else from this conversation, so include every detail it needs and say what to report back." }
            },
            "required": ["task"]
        });
        let spec = ToolSpec::new(name, description, schema).risk(risk).untrusted_output().timeout(Duration::from_secs(600));
        Self { spec, child, observer: None, on_finish: None, run_observer: None }
    }

    /// Called with the parent session id and every event of the child run, e.g. to show progress.
    pub fn observe(mut self, observer: impl Fn(&str, &Event) + Send + Sync + 'static) -> Self {
        self.observer = Some(Box::new(observer));
        self
    }

    /// Observe full child run identity, including parent_run_id. Errors fail closed before
    /// dispatch, just as the root run observer does.
    pub fn observe_run(mut self, observer: impl Fn(&crate::RunEvent) -> Result<(), HarnessError> + Send + Sync + 'static) -> Self {
        self.run_observer = Some(Box::new(observer));
        self
    }

    /// Called with the child session id after every job, whatever its outcome, for cleanup such as
    /// `Permission::forget_session`, `Todo::clear_session` or deleting stored messages.
    pub fn on_finish(mut self, hook: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_finish = Some(Box::new(hook));
        self
    }

    /// By default the answer counts as untrusted (see `ToolSpec::untrusted_output`) because the
    /// child may have read web pages. Call this when the child has no tools that fetch outside content.
    pub fn trusted_output(mut self) -> Self {
        self.spec.untrusted_output = false;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.spec.timeout = timeout;
        self
    }

    /// Enable only when child jobs may safely share their tools and workspace concurrently.
    pub fn parallel(mut self) -> Self {
        self.spec.parallel_safe = true;
        self
    }
}

impl Tool for Subagent {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let task = str_arg(&args, "task")?.trim();
        if task.is_empty() || task.chars().count() > MAX_TASK_CHARS {
            return Err(format!("`task` must be 1 to {MAX_TASK_CHARS} characters"));
        }
        let child_session = child_session_id(ctx.session(), &self.spec.name);
        let cancel = Arc::new(AtomicBool::new(false));
        let finished = AtomicBool::new(false);

        let result = thread::scope(|scope| {
            scope.spawn(|| {
                while !finished.load(Ordering::Relaxed) {
                    if ctx.is_cancelled() {
                        cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                    thread::sleep(POLL);
                }
            });
            let options = crate::RunOptions { parent_run_id: Some(ctx.run_id().into()), ephemeral_session: true, ..Default::default() };
            let result = self.child.run_parts_observed(&child_session, vec![crate::Part::Text { text: task.into() }], options, &mut |record| {
                if let Some(observer) = &self.run_observer { observer(&record)?; }
                if let (Some(observer), crate::RunEventBody::Event { event }) = (&self.observer, record.body) {
                    if !matches!(event, Event::ToolDispatched { .. }) { observer(ctx.session(), &event); }
                }
                Ok(())
            }, &cancel);
            finished.store(true, Ordering::Relaxed);
            result
        });

        if let Some(hook) = &self.on_finish {
            hook(&child_session);
        }
        match result {
            Ok(summary) if summary.text.trim().is_empty() => Ok("(the sub-agent finished without a text answer)".to_string()),
            Ok(summary) => Ok(summary.text),
            Err(HarnessError::Cancelled) => Err("cancelled".to_string()),
            Err(error) => Err(format!("sub-agent failed: {error}")),
        }
    }
}

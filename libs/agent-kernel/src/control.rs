//! Shared run budget and conservative tool isolation, including delegated/still-running workers.
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, Weak, OnceLock,
    },
    time::{Duration, Instant},
};
#[derive(Debug)]
struct Inner {
    isolate: bool,
    deadline: Mutex<Clock>,
    tokens: AtomicU64,
    limit: u64,
}
#[derive(Debug, Default)]
struct Clock { end: Option<Instant>, waiting: usize, since: Option<Instant>, excluded: Duration }
/// Nested user waits exclude wall time without resetting the remaining budget.
pub struct UserWait(RunControl);
impl Drop for UserWait {
    fn drop(&mut self) {
        let mut clock = self.0.0.deadline.lock().unwrap_or_else(|e| e.into_inner());
        clock.waiting = clock.waiting.saturating_sub(1);
        if clock.waiting == 0 {
            if let Some(since) = clock.since.take() {
                let elapsed = since.elapsed();
                clock.excluded += elapsed;
                clock.end = clock.end.and_then(|end| end.checked_add(elapsed));
            }
        }
    }
}
/// A tool's shared mutation domain. File paths must be resolved by the tool,
/// rather than trusting a model-supplied relative path or alias.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolResource {
    /// Unknown mutation domain: conflicts with all other mutation resources.
    Global,
    /// Keep file ownership for the run, so competing conversations cannot overwrite its work.
    File(std::path::PathBuf),
    /// An application-owned domain, exclusively held only while the call is executing.
    Named(String),
    /// A sequence-sensitive domain (such as the user's desktop), exclusively held for the run.
    RunExclusive(String),
}
impl ToolResource {
    fn conflicts(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Global, _) | (_, Self::Global) => true,
            (Self::File(a), Self::File(b)) => a == b,
            (Self::Named(a) | Self::RunExclusive(a), Self::Named(b) | Self::RunExclusive(b)) => a == b,
            _ => false,
        }
    }
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Global => "shared mutation resource".into(),
            Self::File(path) => path.display().to_string(),
            Self::Named(name) | Self::RunExclusive(name) => name.clone(),
        }
    }
    pub(crate) fn held_for_run(&self) -> bool {
        matches!(self, Self::File(_) | Self::RunExclusive(_))
    }
}
#[derive(Debug)]
struct ResourceOwner {
    run: Weak<Inner>,
    transient: bool,
    uncertain: AtomicBool,
}
struct Ownership {
    resource: ToolResource,
    owner: Weak<ResourceOwner>,
    session: Option<String>,
}
#[derive(Default)]
struct Owners { entries: Vec<Ownership> }
struct Conflict {
    persistent: bool,
    session: Option<String>,
}
static ACTIVE: OnceLock<Mutex<Owners>> = OnceLock::new();
#[derive(Clone, Debug)]
pub struct RunControl(Arc<Inner>, Arc<ResourceOwner>);
impl Default for RunControl {
    fn default() -> Self {
        Self::new(Inner {
            isolate: false,
            deadline: Mutex::new(Clock::default()),
            tokens: AtomicU64::new(0),
            limit: u64::MAX,
        })
    }
}
impl RunControl {
    fn new(inner: Inner) -> Self {
        let inner = Arc::new(inner);
        let owner = Arc::new(ResourceOwner { run: Arc::downgrade(&inner), transient: false, uncertain: AtomicBool::new(false) });
        Self(inner, owner)
    }
    /// Shares the run budget, but releases call-owned resources when the last worker/transport exits.
    pub(crate) fn tool_scope(&self) -> Self {
        Self(self.0.clone(), Arc::new(ResourceOwner {
            run: Arc::downgrade(&self.0), transient: true, uncertain: AtomicBool::new(false),
        }))
    }
    pub(crate) fn mark_uncertain(&self) { self.1.uncertain.store(true, Ordering::SeqCst); }
    /// No time/token cap, while retaining mutation-domain isolation.
    pub fn unlimited() -> Self {
        Self::new(Inner { isolate: true, deadline: Mutex::new(Clock::default()),
            tokens: AtomicU64::new(0), limit: u64::MAX })
    }

    pub fn limits(duration: Option<Duration>, tokens: Option<u64>) -> Self {
        Self::new(Inner { isolate: true,
            deadline: Mutex::new(Clock { end: duration.map(|d| Instant::now() + d), ..Default::default() }),
            tokens: AtomicU64::new(0), limit: tokens.unwrap_or(u64::MAX) })
    }

    pub(crate) fn tool_timeout(timeout: Duration) -> Self {
        Self::new(Inner { isolate: false,
            deadline: Mutex::new(Clock { end: Instant::now().checked_add(timeout), ..Default::default() }),
            tokens: AtomicU64::new(0), limit: u64::MAX })
    }
    pub(crate) fn time_expired_at(&self, at: Instant) -> bool {
        let clock = self.0.deadline.lock().unwrap_or_else(|e| e.into_inner());
        clock.end.is_some_and(|end| clock.since.unwrap_or(at) >= end)
    }

    pub fn reported_tokens(&self) -> u64 { self.0.tokens.load(Ordering::SeqCst) }

    pub fn bounded(duration: Duration, tokens: u64) -> Self {
        Self::new(Inner {
            isolate: true,
            deadline: Mutex::new(Clock { end: Some(Instant::now() + duration), ..Default::default() }),
            tokens: AtomicU64::new(0),
            limit: tokens,
        })
    }
    pub fn claim(&self) -> Result<(), String> {
        self.claim_resource(&ToolResource::Global)
    }
    pub fn claim_resource(&self, resource: &ToolResource) -> Result<(), String> {
        self.try_claim(resource, None).map_err(|conflict| format!(
            "another task{} still owns {}; inspect its state or resume after it has stopped",
            conflict.session.map(|s| format!(" (session {s})")).unwrap_or_default(), resource.label(),
        ))
    }
    fn try_claim(&self, resource: &ToolResource, session: Option<&str>) -> Result<(), Conflict> {
        if !self.0.isolate {
            return Ok(());
        }
        let mut active = ACTIVE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
        active.entries.retain(|entry| entry.owner.strong_count() != 0);
        let mut conflict: Option<Conflict> = None;
        for entry in &active.entries {
            let Some(owner) = entry.owner.upgrade() else { continue };
            if owner.run.ptr_eq(&self.1.run) || !entry.resource.conflicts(resource) { continue; }
            let persistent = !owner.transient || owner.uncertain.load(Ordering::SeqCst);
            if conflict.as_ref().map_or(true, |old| persistent && !old.persistent) {
                conflict = Some(Conflict { persistent, session: entry.session.clone() });
            }
        }
        if let Some(conflict) = conflict { return Err(conflict); }
        if !active.entries.iter().any(|entry| entry.resource == *resource && entry.owner.ptr_eq(&Arc::downgrade(&self.1))) {
            active.entries.push(Ownership { resource: resource.clone(), owner: Arc::downgrade(&self.1), session: session.map(String::from) });
        }
        Ok(())
    }
    /// Wait only for in-flight calls. Run-owned files and uncertain actions require an explicit
    /// decision: waiting on those could deadlock two runs that already own different files.
    pub(crate) fn claim_tool_resource(
        &self, resource: &ToolResource, session: &str, cancel: &AtomicBool,
        mut waiting: impl FnMut(bool, Option<&str>),
    ) -> Result<(), crate::ToolResult> {
        let mut notified = false;
        loop {
            if cancel.load(Ordering::Relaxed) { return Err(crate::ToolResult::error("cancelled while waiting for resource")); }
            if let Err(error) = self.check() { return Err(crate::ToolResult::error(error.to_string())); }
            match self.try_claim(resource, Some(session)) {
                Ok(()) => {
                    if notified { waiting(false, None); }
                    return Ok(());
                }
                Err(conflict) if conflict.persistent => return Err(crate::ToolResult::error(format!(
                    "Resource {} is held by another task{} for an active run or an unconfirmed action. No operation was dispatched. For conflicting project edits, use a separate worktree.",
                    resource.label(), conflict.session.as_ref().map(|s| format!(" (session {s})")).unwrap_or_default(),
                )).with_structured_content(serde_json::json!({"harness_status":"resource_busy","resource":resource.label(),"owner_session":conflict.session,"executed":false}))),
                Err(conflict) => {
                    if !notified { waiting(true, conflict.session.as_deref()); notified = true; }
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
        }
    }
    pub fn record(&self, u: crate::Usage) {
        self.0.tokens.fetch_add(u.budget_units(), Ordering::SeqCst);
    }
    pub fn check(&self) -> Result<(), crate::HarnessError> {
        let clock = self.0.deadline.lock().unwrap_or_else(|e| e.into_inner());
        if clock.end.is_some_and(|d| clock.since.unwrap_or_else(Instant::now) >= d) {
            return Err(crate::HarnessError::BudgetExceeded("elapsed time".into()));
        }
        if self.0.tokens.load(Ordering::SeqCst) >= self.0.limit {
            return Err(crate::HarnessError::BudgetExceeded(
                "reported tokens".into(),
            ));
        }
        Ok(())
    }
    pub fn wait_for_user(&self) -> UserWait {
        let mut clock = self.0.deadline.lock().unwrap_or_else(|e| e.into_inner());
        if clock.waiting == 0 { clock.since = Some(Instant::now()); }
        clock.waiting += 1;
        UserWait(self.clone())
    }
    pub fn excluded_wait(&self) -> Duration {
        let clock = self.0.deadline.lock().unwrap_or_else(|e| e.into_inner());
        clock.excluded + clock.since.map_or(Duration::ZERO, |since| since.elapsed())
    }
    pub fn watch(&self, cancel: Arc<AtomicBool>) -> StopWatch {
        let c = self.clone();
        StopWatch::new(
            move || c.check().is_err(),
            move || cancel.store(true, Ordering::SeqCst),
        )
    }
}
/// A bounded poller; dropping joins it and never waits on network or a tool worker.
pub struct StopWatch {
    done: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl StopWatch {
    pub fn new(
        stop: impl Fn() -> bool + Send + 'static,
        action: impl FnOnce() + Send + 'static,
    ) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let d = done.clone();
        let thread = std::thread::spawn(move || {
            while !d.load(Ordering::SeqCst) {
                if stop() {
                    action();
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        Self {
            done,
            thread: Some(thread),
        }
    }
}
impl Drop for StopWatch {
    fn drop(&mut self) {
        self.done.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod user_wait_tests {
    use super::*;
    #[test]
    fn nested_human_waits_suspend_time_but_not_token_or_cancel_limits() {
        let c=RunControl::bounded(Duration::from_millis(80),100);
        let pause=c.wait_for_user();let nested=c.wait_for_user();
        std::thread::sleep(Duration::from_millis(100));
        assert!(c.check().is_ok());drop(nested);assert!(c.check().is_ok());drop(pause);
        assert!(c.excluded_wait()>=Duration::from_millis(100));assert!(c.check().is_ok());
        std::thread::sleep(Duration::from_millis(100));assert!(c.check().is_err());
        let c=RunControl::bounded(Duration::from_secs(5),1);let _wait=c.wait_for_user();
        c.record(crate::Usage{prompt_tokens:1,..Default::default()});assert!(c.check().is_err());
    }

    #[test]
    fn overlapping_call_scopes_keep_each_lease_and_share_the_run_budget() {
        let run = RunControl::unlimited();
        let a = run.tool_scope();
        let b = run.tool_scope();
        let resource = ToolResource::Named("overlapping-unit-test".into());
        a.claim_resource(&resource).unwrap();
        b.claim_resource(&resource).unwrap();
        a.record(crate::Usage { prompt_tokens: 10, ..Default::default() });
        assert_eq!(run.reported_tokens(), 10);
        drop(b);
        assert!(RunControl::unlimited().claim_resource(&resource).is_err());
        drop(a);
        assert!(RunControl::unlimited().claim_resource(&resource).is_ok());
    }
}

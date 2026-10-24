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
    deadline: Option<Instant>,
    tokens: AtomicU64,
    limit: u64,
}
/// A tool's shared mutation domain. File paths must be resolved by the tool,
/// rather than trusting a model-supplied relative path or alias.
#[derive(Clone, Debug)]
pub enum ToolResource {
    Global,
    File(std::path::PathBuf),
}
#[derive(Default)]
struct Owners {
    global: Option<Weak<Inner>>,
    files: std::collections::HashMap<std::path::PathBuf, Weak<Inner>>,
}
static ACTIVE: OnceLock<Mutex<Owners>> = OnceLock::new();
#[derive(Clone, Debug)]
pub struct RunControl(Arc<Inner>);
impl Default for RunControl {
    fn default() -> Self {
        Self(Arc::new(Inner {
            isolate: false,
            deadline: None,
            tokens: AtomicU64::new(0),
            limit: u64::MAX,
        }))
    }
}
impl RunControl {
    /// No time/token cap, while retaining mutation-domain isolation.
    pub fn unlimited() -> Self {
        Self(Arc::new(Inner { isolate: true, deadline: None,
            tokens: AtomicU64::new(0), limit: u64::MAX }))
    }

    pub fn limits(duration: Option<Duration>, tokens: Option<u64>) -> Self {
        Self(Arc::new(Inner { isolate: true,
            deadline: duration.map(|d| Instant::now() + d),
            tokens: AtomicU64::new(0), limit: tokens.unwrap_or(u64::MAX) }))
    }

    pub fn reported_tokens(&self) -> u64 { self.0.tokens.load(Ordering::SeqCst) }

    pub fn bounded(duration: Duration, tokens: u64) -> Self {
        Self(Arc::new(Inner {
            isolate: true,
            deadline: Some(Instant::now() + duration),
            tokens: AtomicU64::new(0),
            limit: tokens,
        }))
    }
    pub fn claim(&self) -> Result<(), String> {
        self.claim_resource(&ToolResource::Global)
    }
    pub fn claim_resource(&self, resource: &ToolResource) -> Result<(), String> {
        if !self.0.isolate {
            return Ok(());
        }
        let mut active = ACTIVE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
        active.files.retain(|_, owner| owner.strong_count() != 0);
        let foreign = |owner: &Weak<Inner>| owner.upgrade().is_some_and(|old| !Arc::ptr_eq(&old, &self.0));
        let conflict = active.global.as_ref().is_some_and(foreign) || match resource {
            ToolResource::Global => active.files.values().any(foreign),
            ToolResource::File(path) => active.files.get(path).is_some_and(foreign),
        };
        if conflict {
            return Err("another task still owns this tool resource; inspect its state or resume after it has stopped".into());
        }
        match resource {
            ToolResource::Global => active.global = Some(Arc::downgrade(&self.0)),
            ToolResource::File(path) => { active.files.insert(path.clone(), Arc::downgrade(&self.0)); }
        }
        Ok(())
    }
    pub fn record(&self, u: crate::Usage) {
        self.0.tokens.fetch_add(u.budget_units(), Ordering::SeqCst);
    }
    pub fn check(&self) -> Result<(), crate::HarnessError> {
        if self.0.deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(crate::HarnessError::BudgetExceeded("elapsed time".into()));
        }
        if self.0.tokens.load(Ordering::SeqCst) >= self.0.limit {
            return Err(crate::HarnessError::BudgetExceeded(
                "reported tokens".into(),
            ));
        }
        Ok(())
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

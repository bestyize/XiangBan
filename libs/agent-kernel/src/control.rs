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
            deadline: Mutex::new(Clock::default()),
            tokens: AtomicU64::new(0),
            limit: u64::MAX,
        }))
    }
}
impl RunControl {
    /// No time/token cap, while retaining mutation-domain isolation.
    pub fn unlimited() -> Self {
        Self(Arc::new(Inner { isolate: true, deadline: Mutex::new(Clock::default()),
            tokens: AtomicU64::new(0), limit: u64::MAX }))
    }

    pub fn limits(duration: Option<Duration>, tokens: Option<u64>) -> Self {
        Self(Arc::new(Inner { isolate: true,
            deadline: Mutex::new(Clock { end: duration.map(|d| Instant::now() + d), ..Default::default() }),
            tokens: AtomicU64::new(0), limit: tokens.unwrap_or(u64::MAX) }))
    }

    pub(crate) fn tool_timeout(timeout: Duration) -> Self {
        Self(Arc::new(Inner { isolate: false,
            deadline: Mutex::new(Clock { end: Instant::now().checked_add(timeout), ..Default::default() }),
            tokens: AtomicU64::new(0), limit: u64::MAX }))
    }
    pub(crate) fn time_expired_at(&self, at: Instant) -> bool {
        let clock = self.0.deadline.lock().unwrap_or_else(|e| e.into_inner());
        clock.end.is_some_and(|end| clock.since.unwrap_or(at) >= end)
    }

    pub fn reported_tokens(&self) -> u64 { self.0.tokens.load(Ordering::SeqCst) }

    pub fn bounded(duration: Duration, tokens: u64) -> Self {
        Self(Arc::new(Inner {
            isolate: true,
            deadline: Mutex::new(Clock { end: Some(Instant::now() + duration), ..Default::default() }),
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
}

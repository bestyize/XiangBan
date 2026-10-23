//! Shared run budget and conservative tool isolation, including delegated/still-running workers.
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, Weak,
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
static ACTIVE: Mutex<Option<Weak<Inner>>> = Mutex::new(None);
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
    pub fn bounded(duration: Duration, tokens: u64) -> Self {
        Self(Arc::new(Inner {
            isolate: true,
            deadline: Some(Instant::now() + duration),
            tokens: AtomicU64::new(0),
            limit: tokens,
        }))
    }
    pub fn claim(&self) -> Result<(), String> {
        if !self.0.isolate {
            return Ok(());
        }
        let mut active = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(old) = active.as_ref().and_then(Weak::upgrade) {
            if !Arc::ptr_eq(&old, &self.0) {
                return Err(
                    "another task still owns shared tool resources; resume after it has stopped"
                        .into(),
                );
            }
        }
        *active = Some(Arc::downgrade(&self.0));
        Ok(())
    }
    pub fn record(&self, u: crate::Usage) {
        self.0.tokens.fetch_add(
            u64::from(u.prompt_tokens) + u64::from(u.completion_tokens),
            Ordering::SeqCst,
        );
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

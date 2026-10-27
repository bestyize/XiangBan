//! Run identity and observation, independent of conversation history and model context.

use crate::{Event, HarnessError};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Host mailbox. Poll only at complete model/tool boundaries; acknowledge only after history is saved.
/// `closing` seals an empty mailbox so a late submission can be handled by the host's next run.
pub trait RunInput: Send + Sync + std::fmt::Debug {
    fn poll(&self, session: &str, closing: bool) -> Result<Vec<crate::Message>, HarnessError>;
    fn accepted(&self, session: &str, messages: &[crate::Message]) -> Result<(), HarnessError>;
}

static RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub input: Option<Arc<dyn RunInput>>,
    /// Trusted host metadata for the initial user message (e.g. durable delivery identity).
    pub input_metadata: Option<serde_json::Value>,
    pub control: Option<crate::RunControl>,
    pub max_no_progress_rounds: Option<usize>,
    pub parent_run_id: Option<String>,
    /// Release session-owned runtime resources at run end (for temporary delegated sessions).
    /// Conversation retention remains the SessionStore/host's decision.
    pub ephemeral_session: bool,
    /// Counts requested tool calls, including invalid or denied ones. An oversized batch is
    /// rejected in its entirety and the run pauses, without executing a partial batch.
    pub max_tool_calls: Option<usize>,
    /// A budget shared with the run that started this one: a sub-agent's calls count against its parent's
    /// `max_tool_calls`. `Subagent` passes it on by itself; hosts rarely set it.
    pub shared_budget: Option<Arc<ToolBudget>>,
}

/// Tool calls left for a run and the sub-agent runs it starts.
#[derive(Debug)]
pub struct ToolBudget {
    limit: usize,
    used: AtomicUsize,
}

impl ToolBudget {
    pub fn new(limit: usize) -> Self {
        Self { limit, used: AtomicUsize::new(0) }
    }

    /// Counts `calls` and returns whether they still fit. Calls that do not fit are counted too, so once the
    /// budget is spent every later batch is refused as well.
    pub fn take(&self, calls: usize) -> bool {
        self.used.fetch_add(calls, Ordering::SeqCst).saturating_add(calls) <= self.limit
    }

    pub fn used(&self) -> usize {
        self.used.load(Ordering::SeqCst)
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Running,
    /// The model loop finished. This is not a claim that a business acceptance test passed.
    Completed,
    Paused,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunEventBody {
    State {
        state: RunState,
        reason: Option<String>,
    },
    Event {
        event: Event,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunEvent {
    pub run_id: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    /// Starts at one; never resets during this run.
    pub sequence: u64,
    pub timestamp_ms: u64,
    #[serde(flatten)]
    pub body: RunEventBody,
}

#[derive(Debug, Clone)]
pub struct RunEnd {
    pub run_id: String,
    pub session_id: String,
    pub state: RunState,
    pub ephemeral_session: bool,
}

pub(crate) struct RunEmitter<'a> {
    pub run_id: String,
    session: String,
    parent: Option<String>,
    sequence: u64,
    observe: &'a mut dyn FnMut(RunEvent) -> Result<(), HarnessError>,
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub(crate) fn new_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{prefix}-{nanos:x}-{:x}-{:x}",
        std::process::id(),
        RUN_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

impl<'a> RunEmitter<'a> {
    pub fn new(
        session: &str,
        options: &RunOptions,
        observe: &'a mut dyn FnMut(RunEvent) -> Result<(), HarnessError>,
    ) -> Self {
        let run_id = new_id("r");
        Self {
            run_id,
            session: session.into(),
            parent: options.parent_run_id.clone(),
            sequence: 0,
            observe,
        }
    }

    pub fn emit(&mut self, body: RunEventBody) -> Result<(), HarnessError> {
        self.sequence += 1;
        (self.observe)(RunEvent {
            run_id: self.run_id.clone(),
            session_id: self.session.clone(),
            parent_run_id: self.parent.clone(),
            sequence: self.sequence,
            timestamp_ms: now_ms(),
            body,
        })
    }
}

pub(crate) fn state_of(result: &Result<crate::RunSummary, HarnessError>) -> RunState {
    match result {
        Ok(_) => RunState::Completed,
        Err(HarnessError::Cancelled) => RunState::Cancelled,
        Err(
            HarnessError::Paused(_) | HarnessError::MaxTurns(_) | HarnessError::BudgetExceeded(_),
        ) => RunState::Paused,
        Err(_) => RunState::Failed,
    }
}

use crate::error::HarnessError;
use crate::tool::{ToolCtx, ToolGate, ToolResult, ToolSpec};
use crate::types::{FinishReason, Message, ToolCall};
use serde_json::Value;
use std::sync::Arc;

mod context;
mod continuation;
mod permission;
mod repeat;
mod spill;

pub use context::{estimate_tokens, ContextTrim};
pub use continuation::ContinueGuard;
pub use permission::{Approval, ApprovalRequest, Approver, Permission, PermissionMode};
pub use repeat::RepeatGuard;
pub use spill::SpillOutput;

/// The run at the moment the model replied without calling a tool, which would end it.
pub struct Stop<'a> {
    pub session: &'a str,
    /// The stored conversation, ending with the reply that would end the run.
    pub history: &'a [Message],
    /// Where this run starts in `history`: the user message it answers.
    pub run_start: usize,
    pub finish: FinishReason,
    /// How many times middleware has already kept this run going.
    pub continuations: usize,
    pub continuations_without_progress: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopDecision {
    Finish,
    Continue(Nudge),
    Pause(String),
}

/// Keeps a run going: `message` is stored as a synthetic user message and the model is called again.
#[derive(Debug, Clone, PartialEq)]
pub struct Nudge {
    /// Short code reported to the host in `Event::Continuing`.
    pub reason: String,
    pub message: String,
}

/// The model-facing view for one request. Edits here are not written back to the session.
pub struct Turn {
    pub session: String,
    pub index: usize,
    pub messages: Vec<Message>,
    pub tools: Vec<crate::ToolSchema>,
}

/// Hooks run in registration order. Permission, context trimming, output spilling and budgets
/// are all middleware; the loop itself knows none of them.
pub trait Middleware: Send + Sync {
    fn before_model(&self, _turn: &mut Turn) -> Result<(), HarnessError> {
        Ok(())
    }

    fn after_model(&self, _turn: &Turn, _reply: &mut Message) -> Result<(), HarnessError> {
        Ok(())
    }

    /// Runs on the agent thread, one call at a time, so it may block on user approval. Anything
    /// that blocks should wait through `ctx.wait_for` (or poll `ctx.is_cancelled`) so cancelling
    /// the run can interrupt it.
    fn before_tool(&self, _ctx: &ToolCtx, _call: &ToolCall, _args: &Value, _spec: &ToolSpec) -> ToolGate {
        ToolGate::Proceed
    }

    fn after_tool(&self, _session: &str, _call: &ToolCall, _result: &mut ToolResult) {}

    /// The first non-Finish decision wins; `max_turns` still bounds continuations.
    fn before_stop(&self, _stop: &Stop<'_>) -> StopDecision {
        StopDecision::Finish
    }
}

/// Lets a host keep a handle (for example to switch the permission mode) while the agent owns another.
impl<T: Middleware + ?Sized> Middleware for Arc<T> {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        (**self).before_model(turn)
    }

    fn after_model(&self, turn: &Turn, reply: &mut Message) -> Result<(), HarnessError> {
        (**self).after_model(turn, reply)
    }

    fn before_tool(&self, ctx: &ToolCtx, call: &ToolCall, args: &Value, spec: &ToolSpec) -> ToolGate {
        (**self).before_tool(ctx, call, args, spec)
    }

    fn after_tool(&self, session: &str, call: &ToolCall, result: &mut ToolResult) {
        (**self).after_tool(session, call, result)
    }

    fn before_stop(&self, stop: &Stop<'_>) -> StopDecision {
        (**self).before_stop(stop)
    }
}

use crate::error::HarnessError;
use crate::tool::{ToolCtx, ToolGate, ToolResult, ToolSpec};
use crate::types::{Message, ToolCall};
use serde_json::Value;
use std::sync::Arc;

mod context;
mod permission;
mod spill;

pub use context::{estimate_tokens, ContextTrim};
pub use permission::{Approval, ApprovalRequest, Approver, Permission, PermissionMode};
pub use spill::SpillOutput;

/// The model-facing view for one request. Edits here are not written back to the session.
pub struct Turn {
    pub session: String,
    pub index: usize,
    pub messages: Vec<Message>,
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
}

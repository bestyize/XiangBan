//! Product-agnostic agent harness kernel.
//!
//! The host supplies a [`ModelProvider`], a [`SessionStore`] and its own [`Tool`]s; everything
//! else (permissions, context trimming, budgets) plugs in as [`Middleware`].

mod agent;
pub mod builtin;
mod error;
mod exec;
#[cfg(feature = "memory")]
pub mod memory;
mod middleware;
pub mod provider;
mod run;
#[cfg(feature = "skills")]
pub mod skills;
#[cfg(feature = "mcp")]
pub mod mcp;
#[cfg(feature = "stdio")]
pub mod stdio;
mod store;
mod tool;
mod types;
mod util;

pub use agent::{Agent, AgentBuilder, RetryPolicy};
pub use error::HarnessError;
pub use middleware::{
    estimate_tokens, Approval, ApprovalRequest, Approver, ContextTrim, ContinueGuard, Middleware, Nudge, Permission,
    PermissionMode, RepeatGuard, SpillOutput, Stop, Turn,
};
pub use provider::{ChatRequest, ModelProvider, Reply, ToolSchema};
pub use run::RunSummary;
pub use store::{JsonlStore, MemoryStore, SessionStore};
pub use tool::{Risk, Tool, ToolCtx, ToolGate, ToolResult, ToolSpec, DEFAULT_MAX_OUTPUT};
pub use types::{Event, FinishReason, Message, Part, Role, ToolCall, Usage, PROTOCOL_VERSION};

//! Product-agnostic agent harness kernel.
//!
//! The host supplies a [`ModelProvider`], a [`SessionStore`] and its own [`Tool`]s; everything
//! else (permissions, context trimming, budgets) plugs in as [`Middleware`].

mod agent;
#[allow(dead_code)]
mod child_env;
pub mod sandbox;
pub mod builtin;
mod error;
mod exec;
#[cfg(feature = "memory")]
pub mod memory;
mod middleware;
pub mod provider;
pub mod model;
pub mod goals;
pub mod supervisor;
#[cfg(feature = "journal")]
pub mod recovery;
mod run;
mod runtime;
#[cfg(feature = "journal")]
pub mod journal;
#[cfg(feature = "process")]
pub mod process;
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
    estimate_tokens, estimate_tool_tokens, Approval, ApprovalRequest, Approver, ContextTrim, ContinueGuard, Middleware, Nudge, Permission,
    PermissionMode, RepeatGuard, SpillOutput, Stop, StopDecision, Turn,
};
pub use provider::{ChatRequest, ModelProvider, Reply, ToolSchema};
pub use run::RunSummary;
pub use runtime::{RunEnd, RunEvent, RunEventBody, RunOptions, RunState, ToolBudget};
pub use store::{JsonlStore, MemoryStore, SessionStore};
pub use tool::{Risk, Tool, ToolCtx, ToolGate, ToolResult, ToolSpec, DEFAULT_MAX_OUTPUT};
pub use types::{ContextSummary, Event, FinishReason, Message, Part, Role, ToolCall, Usage, ReasoningKind, ReasoningSegment, PROTOCOL_VERSION};

//! A host-agnostic proactivity decision engine.
//!
//! The engine does not read OS state, invoke an LLM, schedule threads, or deliver notifications.
//! A host supplies approved events and an explicit context, then owns all delivery and permissions.
//! Raw user activity content is deliberately not stored.
pub mod attention;
pub mod engine;
pub mod event;
pub mod feedback;
pub mod policy;

pub use engine::{Action, Decision, Engine, EngineState, Observation, Suppression};
pub use event::{Event, EventKind, EventSource};
pub use feedback::Feedback;
pub use policy::{ActionKind, Candidate, Channel, Context, Policy};

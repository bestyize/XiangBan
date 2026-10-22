//! Provider-neutral model runtime policy.
//!
//! Endpoint-specific capabilities are supplied by the host; unknown values stay
//! unknown rather than being inferred from context settings or a model name.
//! Only protocols that require an explicit generation budget receive a fallback.
mod capabilities;
mod budget;
pub mod recovery;

pub use capabilities::{CapabilitySource, ModelCapabilities, ModelProtocol};
pub use budget::TokenBudget;

use serde::{Deserialize, Serialize};
use super::Source;

/// Completion of a model loop is *not* proof of task success.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState { Started, Succeeded, Failed, Partial, Cancelled, Unknown }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskContext {
    pub state: TaskState,
    pub verified: bool,
    pub at_ms: i64,
    pub source: Source,
}
impl TaskContext {
    pub fn observed(state: TaskState, verified: bool, at_ms: i64) -> Option<Self> {
        if at_ms < 0 || (state == TaskState::Succeeded && !verified) { return None; }
        Some(Self { state, verified, at_ms, source: Source::AgentRun })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_completion_is_never_claimed_as_verified_success() {
        assert!(TaskContext::observed(TaskState::Succeeded,false,100).is_none());
        assert_eq!(TaskContext::observed(TaskState::Unknown,false,100).unwrap().state,TaskState::Unknown);
    }
}

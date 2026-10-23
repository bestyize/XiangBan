//! Reconciliation of a persisted run after its owning process disappeared.
//! Never infer that a dispatched external write did or did not happen from
//! an incomplete event stream. Never replay external side effects here.
use crate::{goals::{GoalRecord, GoalState}, journal::RunInspection, RunState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryDecision {
    /// A terminal run was recorded; business verification is separate.
    Terminal,
    /// No dispatched tool is left unresolved. A *new* user-authorized turn
    /// may inspect state, but must not replay the previous request.
    Interrupted,
    /// At least one dispatched action lacks a durable result. Requires human
    /// or domain-specific reconciliation before any continuation.
    NeedsReview,
}

pub fn classify(inspection: &RunInspection) -> RecoveryDecision {
    if inspection.incomplete_tools.iter().any(|t| t.dispatched) {
        RecoveryDecision::NeedsReview
    } else if inspection.terminal {
        RecoveryDecision::Terminal
    } else {
        RecoveryDecision::Interrupted
    }
}

/// Reconcile a GoalRecord that was still Running when its host stopped.
/// This is called on the next host load, *never* from a concurrent live run.
pub fn reconcile(goal: &mut GoalRecord, trace: Option<&RunInspection>, now_ms: u64) -> RecoveryDecision {
    if goal.state != GoalState::Running {
        return RecoveryDecision::Terminal;
    }
    let decision = trace.map(classify).unwrap_or(RecoveryDecision::NeedsReview);
    if let Some(inspect) = trace {
        if inspect.terminal {
            goal.state = match inspect.state {
                Some(RunState::Completed) => GoalState::AwaitingVerification,
                _ => GoalState::Paused,
            };
            goal.note = "已从运行日志恢复终态，仍需核验结果".into();
        }
    }
    if decision != RecoveryDecision::Terminal {
        goal.state = match decision {
            RecoveryDecision::NeedsReview => GoalState::NeedsReview,
            _ => GoalState::Interrupted,
        };
        goal.note = if decision == RecoveryDecision::NeedsReview {
            "上次执行中断，可能有已派发但未记录结果的外部动作；请先核验，禁止自动重放".into()
        } else {
            "上次执行中断；可在检查现场后创建新的继续回合，不重放旧工具调用".into()
        };
    }
    goal.updated_at_ms = now_ms;
    decision
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{goals::GoalContract, journal::IncompleteTool};
    fn goal() -> GoalRecord {
        let mut g = GoalRecord::draft("session-1".into(), GoalContract {
            outcome:"build".into(), verification:"cargo test".into(), constraints:String::new(),
            boundaries:String::new(), stop_when:String::new(),
        }, 1).unwrap();
        g.begin(2).unwrap();
        g
    }
    fn inspection(terminal: bool, state: Option<RunState>, dispatched: bool) -> RunInspection {
        RunInspection { last_sequence: 4, state, terminal,
            incomplete_tools: vec![IncompleteTool {
                id:"call-1".into(), name:"shell".into(), dispatched,
            }],
        }
    }
    #[test]
    fn ambiguous_dispatched_side_effects_never_resume() {
        let mut g = goal();
        assert_eq!(reconcile(&mut g, Some(&inspection(false, Some(RunState::Running), true)), 3),
            RecoveryDecision::NeedsReview);
        assert_eq!(g.state, GoalState::NeedsReview);
        assert!(g.begin(4).is_err());
    }
    #[test]
    fn unissued_tool_call_is_interrupted_not_completed() {
        let mut g = goal();
        assert_eq!(reconcile(&mut g, Some(&inspection(false, Some(RunState::Running), false)), 3),
            RecoveryDecision::Interrupted);
        assert_eq!(g.state, GoalState::Interrupted);
        assert!(g.begin(4).is_err());
    }
    #[test]
    fn durable_terminal_requires_external_verification() {
        let mut g = goal();
        assert_eq!(reconcile(&mut g, Some(&RunInspection{incomplete_tools:vec![],..inspection(true,Some(RunState::Completed),false)}), 3),
            RecoveryDecision::Terminal);
        assert_eq!(g.state, GoalState::AwaitingVerification);
        assert!(g.begin(4).is_ok());
    }
    #[test]
    fn missing_trace_is_never_assumed_safe() {
        let mut g = goal();
        assert_eq!(reconcile(&mut g, None, 3), RecoveryDecision::NeedsReview);
        assert_eq!(g.state, GoalState::NeedsReview);
    }
}

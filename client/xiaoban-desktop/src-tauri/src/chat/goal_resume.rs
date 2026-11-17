//! Shared goal-resume transaction for typed, queued and button submissions.
use agent_kernel::goals::{GoalRecord, GoalStore};

pub(super) fn is_resume_request(text: &str) -> bool {
    matches!(text.trim(), "请接着上面的内容继续写完。" | "继续" | "继续目标" | "继续任务" | "继续执行" | "接着做")
}

/// The host serializes this transaction and owns the goal metadata lease.
/// A refused dispatch restores the entire prior record, including accounting.
pub(super) fn resume<T>(db: &GoalStore, mut goal: GoalRecord, now: u64,
    dispatch: impl FnOnce(&GoalRecord) -> Result<T, String>,
    changed: impl Fn(&GoalRecord)) -> Result<T, String> {
    let previous = goal.clone();
    goal.begin(now)?;
    db.save(&goal)?;
    match dispatch(&goal) {
        Ok(result) => { changed(&goal); Ok(result) }
        Err(error) => { db.save(&previous)?; changed(&previous); Err(error) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::goals::{GoalContract, GoalState};
    use std::sync::atomic::{AtomicU64, Ordering};
    static IDS: AtomicU64 = AtomicU64::new(0);

    fn goal() -> GoalRecord {
        let mut goal = GoalRecord::draft("s1".into(), GoalContract {
            outcome: "交付可运行的页面".into(), verification: "可访问".into(), constraints: String::new(),
            boundaries: "限当前项目".into(), stop_when: String::new(), verification_command: String::new(), auto_resume: false,
        }, 10).unwrap();
        goal.state = GoalState::Paused;
        goal.spent_tokens = 42_000;
        goal.elapsed_ms = 90_000;
        goal.attempts = 2;
        goal.last_run_id = Some("previous".into());
        goal.note = "等待继续".into();
        goal
    }

    #[test]
    fn every_continue_entry_preserves_contract_and_cumulative_budget() {
        for text in ["继续", "继续目标", "继续任务", "继续执行", "接着做", "请接着上面的内容继续写完。"] {
            assert!(is_resume_request(text));
        }
        assert!(!is_resume_request("继续解释这段代码"));
        let root = std::env::temp_dir().join(format!("goal-resume-{}-{}", std::process::id(), IDS.fetch_add(1, Ordering::Relaxed)));
        let db = GoalStore::new(&root);
        let original = goal();
        let resumed = resume(&db, original.clone(), 100_000, |active| {
            assert_eq!(db.load("s1")?.as_ref(), Some(active), "the worker sees the resumed goal before it starts");
            Ok(active.clone())
        }, |_| {}).unwrap();
        assert_eq!(resumed.state, GoalState::Running);
        assert_eq!(resumed.contract, original.contract);
        assert_eq!((resumed.spent_tokens, resumed.elapsed_ms), (42_000, 90_000));
        assert_eq!(resumed.attempts, 3);
        assert_eq!(resumed.run_started_at_ms, 100_000);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_resume_restores_accounting_and_review_state() {
        let root = std::env::temp_dir().join(format!("goal-resume-{}-{}", std::process::id(), IDS.fetch_add(1, Ordering::Relaxed)));
        let db = GoalStore::new(&root);
        let original = goal();
        let result: Result<(), String> = resume(&db, original.clone(), 100_000,
            |_| Err("model preparation refused".into()), |restored| assert_eq!(restored, &original));
        assert_eq!(result.unwrap_err(), "model preparation refused");
        assert_eq!(db.load("s1").unwrap(), Some(original.clone()));
        let mut needs_review = original;
        needs_review.state = GoalState::NeedsReview;
        let blocked: Result<(), String> = resume(&db, needs_review, 100_001, |_| panic!("review must not be bypassed"), |_| {});
        assert!(blocked.is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}

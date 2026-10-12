//! Questions the assistant asks the user while it works: "may I run this?" and "which one?".
//!
//! The run waits on a channel; the web side answers by id. Answers for ids that no longer exist
//! (the run was cancelled, or the id is made up) are refused instead of being delivered to someone else.

use agent_kernel::Approval;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, MutexGuard};

type Waiting<T> = HashMap<String, (String, Sender<T>)>;

#[derive(Default)]
pub struct Broker {
    approvals: Mutex<Waiting<Approval>>,
    questions: Mutex<Waiting<Option<String>>>,
    next: AtomicU64,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

impl Broker {
    fn id(&self, prefix: char) -> String {
        format!("{prefix}{}", self.next.fetch_add(1, Ordering::Relaxed) + 1)
    }

    pub fn open_approval(&self, session: &str) -> (String, Receiver<Approval>) {
        let (tx, rx) = mpsc::channel();
        let id = self.id('a');
        lock(&self.approvals).insert(id.clone(), (session.to_string(), tx));
        (id, rx)
    }

    pub fn answer_approval(&self, id: &str, decision: Approval) -> Result<(), String> {
        let waiting = lock(&self.approvals).remove(id);
        match waiting {
            Some((_, tx)) => tx.send(decision).map_err(|_| "这个确认已经结束".to_string()),
            None => Err("这个确认已经结束".into()),
        }
    }

    pub fn close_approval(&self, id: &str) {
        lock(&self.approvals).remove(id);
    }

    pub fn open_question(&self, session: &str) -> (String, Receiver<Option<String>>) {
        let (tx, rx) = mpsc::channel();
        let id = self.id('q');
        lock(&self.questions).insert(id.clone(), (session.to_string(), tx));
        (id, rx)
    }

    pub fn answer_question(&self, id: &str, answer: Option<String>) -> Result<(), String> {
        let waiting = lock(&self.questions).remove(id);
        match waiting {
            Some((_, tx)) => tx.send(answer).map_err(|_| "这个问题已经结束".to_string()),
            None => Err("这个问题已经结束".into()),
        }
    }

    pub fn close_question(&self, id: &str) {
        lock(&self.questions).remove(id);
    }

    /// Ends everything a conversation is waiting on; the runs see their waits fail.
    pub fn drop_session(&self, session: &str) {
        lock(&self.approvals).retain(|_, (s, _)| s != session);
        lock(&self.questions).retain(|_, (s, _)| s != session);
    }

    #[cfg(test)]
    pub fn pending(&self) -> usize {
        lock(&self.approvals).len() + lock(&self.questions).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn an_approval_reaches_the_waiting_run_once() {
        let broker = Broker::default();
        let (id, rx) = broker.open_approval("s1");
        broker.answer_approval(&id, Approval::Always).unwrap();
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), Approval::Always);
        assert!(broker.answer_approval(&id, Approval::Once).is_err(), "a second answer is refused");
        assert_eq!(broker.pending(), 0);
    }

    #[test]
    fn a_question_can_be_answered_or_dismissed() {
        let broker = Broker::default();
        let (a, rx_a) = broker.open_question("s1");
        let (b, rx_b) = broker.open_question("s1");
        broker.answer_question(&a, Some("蓝色".into())).unwrap();
        broker.answer_question(&b, None).unwrap();
        assert_eq!(rx_a.recv_timeout(Duration::from_secs(1)).unwrap(), Some("蓝色".to_string()));
        assert_eq!(rx_b.recv_timeout(Duration::from_secs(1)).unwrap(), None);
    }

    #[test]
    fn ids_are_unique_and_unknown_or_foreign_ids_are_refused() {
        let broker = Broker::default();
        let (approval, _keep) = broker.open_approval("s1");
        let (question, _keep2) = broker.open_question("s1");
        assert_ne!(approval, question);
        assert!(broker.answer_approval("a999", Approval::Once).is_err());
        assert!(broker.answer_question("q999", None).is_err());
        assert!(broker.answer_approval(&question, Approval::Once).is_err(), "a question id is not an approval id");
        assert!(broker.answer_question(&approval, None).is_err());
        assert_eq!(broker.pending(), 2, "wrong-kind answers do not consume anything");
    }

    #[test]
    fn dropping_a_conversation_ends_only_its_own_waits() {
        let broker = Broker::default();
        let (mine, rx_mine) = broker.open_approval("s1");
        let (other, rx_other) = broker.open_approval("s2");
        let (question, rx_q) = broker.open_question("s1");
        broker.drop_session("s1");
        assert!(rx_mine.recv_timeout(Duration::from_millis(50)).is_err(), "the sender is gone, so the run's wait ends");
        assert!(rx_q.recv_timeout(Duration::from_millis(50)).is_err());
        assert!(broker.answer_approval(&mine, Approval::Once).is_err());
        assert!(broker.answer_question(&question, None).is_err());
        broker.answer_approval(&other, Approval::Deny).unwrap();
        assert_eq!(rx_other.recv_timeout(Duration::from_secs(1)).unwrap(), Approval::Deny);
    }

    #[test]
    fn closing_removes_a_wait_that_nobody_will_answer() {
        let broker = Broker::default();
        let (id, _rx) = broker.open_approval("s1");
        broker.close_approval(&id);
        assert!(broker.answer_approval(&id, Approval::Once).is_err());
        let (q, _rx) = broker.open_question("s1");
        broker.close_question(&q);
        assert!(broker.answer_question(&q, None).is_err());
        assert_eq!(broker.pending(), 0);
    }

    #[test]
    fn answering_after_the_run_stopped_listening_is_reported() {
        let broker = Broker::default();
        let (id, rx) = broker.open_approval("s1");
        drop(rx);
        assert!(broker.answer_approval(&id, Approval::Once).is_err());
    }
}

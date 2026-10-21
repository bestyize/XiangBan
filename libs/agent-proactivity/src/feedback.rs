use serde::{Deserialize, Serialize};
use crate::policy::Policy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feedback { Engaged, Dismissed, Ignored, Snoozed }

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FeedbackState {
    pub(crate) consecutive_dismissals: u8,
    pub(crate) snoozed_until_ms: i64,
}
impl FeedbackState {
    pub fn snoozed_until_ms(&self) -> i64 { self.snoozed_until_ms }
    pub(crate) fn record(&mut self, kind: Feedback, now: i64, policy: &Policy) {
        match kind {
            Feedback::Engaged => { self.consecutive_dismissals = 0; }
            Feedback::Dismissed => {
                self.consecutive_dismissals = self.consecutive_dismissals.saturating_add(1);
                if self.consecutive_dismissals >= 3 {
                    self.snoozed_until_ms = self.snoozed_until_ms.max(now.saturating_add(policy.dismissal_snooze_ms.max(0)));
                    self.consecutive_dismissals = 0;
                }
            }
            Feedback::Ignored => {} // ignoring a bubble is not proof of dislike
            Feedback::Snoozed => {
                self.snoozed_until_ms = self.snoozed_until_ms.max(now.saturating_add(policy.dismissal_snooze_ms.max(0)));
                self.consecutive_dismissals = 0;
            }
        }
    }
}

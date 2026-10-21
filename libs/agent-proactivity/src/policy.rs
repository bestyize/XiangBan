use serde::{Deserialize, Serialize};

/// Hosts MUST provide fresh, truthful flags. All false is deliberately NOT an active context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Context {
    /// A local calendar-day key in YYYYMMDD form. The host owns timezone conversion.
    pub local_day: i32,
    pub opted_in: bool,
    pub user_active: bool,
    pub pet_visible: bool,
    pub locked: bool,
    pub quiet: bool,
    pub game: bool,
    pub fullscreen: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind { PetMotion, CompanionGreeting, CareReminder, AgentSuggestion }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel { Ambient, Bubble, Question, Voice }
impl Channel {
    /// A silent animation does not consume interruption budget, but is still policy-gated.
    pub const fn cost(self) -> u16 {
        match self { Self::Ambient => 0, Self::Bubble => 2, Self::Question => 3, Self::Voice => 4 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub kind: ActionKind,
    pub channel: Channel,
    /// Stable, non-sensitive content category (e.g. `welcome_back`), never a message body.
    pub topic_id: Option<String>,
    pub expires_at_ms: Option<i64>,
}
impl Candidate {
    pub fn new(kind: ActionKind, channel: Channel) -> Self {
        Self { kind, channel, topic_id: None, expires_at_ms: None }
    }
    pub fn topic(mut self, topic: impl Into<String>) -> Self { self.topic_id = Some(topic.into()); self }
    pub fn expires_at(mut self, at_ms: i64) -> Self { self.expires_at_ms = Some(at_ms); self }
}

/// All fields are host-owned configuration; no model may override these in its output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub enabled: bool,
    pub max_daily_units: u16,
    pub min_gap_ms: i64,
    pub topic_gap_ms: i64,
    pub dismissal_snooze_ms: i64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: false, // opt-in product rollout: never silently activate new UI
            max_daily_units: 6,
            min_gap_ms: 90 * 60 * 1000,
            topic_gap_ms: 24 * 60 * 60 * 1000,
            dismissal_snooze_ms: 24 * 60 * 60 * 1000,
        }
    }
}

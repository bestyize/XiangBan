use serde::{Deserialize, Serialize};

/// A deliberately small vocabulary: no keystrokes, app titles, screen pixels or message bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    FirstActivityOfDay,
    UserReturned,
    LateNightActive,
    PetHovered,
    PetPoked,
    ConversationFinished,
    TaskFinished,
    TaskStarted,
    TaskFailed,
    TaskPartiallyFinished,
    TaskCancelled,
    TaskOutcomeUnknown,
    LocalDayChanged,
    CalendarClassificationChanged,
    WeatherConditionChanged,
    AttentionModeChanged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource { Desktop, Pet, Agent }

/// Ephemeral input; `subject_id` is intentionally excluded from persisted observation counts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub kind: EventKind,
    pub source: EventSource,
    pub at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}
impl Event {
    pub fn new(kind: EventKind, source: EventSource, at_ms: i64) -> Self {
        Self { kind, source, at_ms, subject_id: None, duration_ms: None }
    }
    pub fn subject(mut self, id: impl Into<String>) -> Self { self.subject_id = Some(id.into()); self }
    pub fn duration(mut self, ms: u64) -> Self { self.duration_ms = Some(ms); self }

    pub fn valid(&self) -> bool {
        self.at_ms >= 0 && self.subject_id.as_deref().is_none_or(valid_key)
    }
}
/// Keys are opaque identifiers, never personal text.
pub(crate) fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 64 &&
    key.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'-' | b'_' | b'.'))
}

//! Short-lived, host-supplied environmental context. Never persisted as an activity trace.
//! Sources stay outside this crate: no OS APIs, calendar networks, LLM, or weather transport.
pub mod activity;
pub mod calendar;
pub mod task;
pub mod temporal;
pub mod weather;

use serde::{Deserialize, Serialize};
pub use activity::{ActivityContext, AttentionMode};
pub use calendar::{CalendarContext, CalendarRules, DayKind, HolidaySpan};
pub use task::{TaskContext, TaskState};
pub use temporal::TemporalContext;
pub use weather::{WeatherCondition, WeatherContext};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    System,
    VerifiedCalendar,
    ExistingWeatherCache,
    AgentRun,
}

/// No identifying app names, messages, locations, or raw OS input samples.
/// Updated atomically by the host and dropped on restart; historical only for decision deltas.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ContextSnapshot {
    pub temporal: Option<TemporalContext>,
    pub calendar: Option<CalendarContext>,
    pub weather: Option<WeatherContext>,
    pub activity: Option<ActivityContext>,
    pub task: Option<TaskContext>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextChange { LocalDay, CalendarClassification, WeatherCondition, AttentionMode, TaskState }

/// Never synthesize a change from a missing previous sample or stale weather.
pub fn changes(previous: &ContextSnapshot, next: &ContextSnapshot, now_ms: i64) -> Vec<ContextChange> {
    let mut events = Vec::new();
    if let (Some(a), Some(b)) = (&previous.temporal, &next.temporal) {
        if a.local_day != b.local_day { events.push(ContextChange::LocalDay); }
    }
    if let (Some(a), Some(b)) = (&previous.calendar, &next.calendar) {
        if a.local_day != b.local_day || a.day_kind != b.day_kind || a.holiday_code != b.holiday_code {
            events.push(ContextChange::CalendarClassification);
        }
    }
    if let (Some(a), Some(b)) = (&previous.weather, &next.weather) {
        if a.is_fresh(now_ms) && b.is_fresh(now_ms) && a.condition != b.condition {
            events.push(ContextChange::WeatherCondition);
        }
    }
    if let (Some(a), Some(b)) = (&previous.activity, &next.activity) {
        if a.mode != b.mode { events.push(ContextChange::AttentionMode); }
    }
    if let (Some(a), Some(b)) = (&previous.task, &next.task) {
        if a.at_ms != b.at_ms && a.state != b.state { events.push(ContextChange::TaskState); }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_or_first_sample_does_not_generate_a_spurious_event() {
        let next = ContextSnapshot { temporal: Some(TemporalContext::new(20261021,3,14,30,540,1000).unwrap()), ..Default::default() };
        assert!(changes(&ContextSnapshot::default(),&next,1000).is_empty());
        assert!(changes(&next,&next,1000).is_empty());
    }
    #[test]
    fn a_real_local_day_transition_is_detected_once() {
        let first = ContextSnapshot { temporal: Some(TemporalContext::new(20261021,3,23,59,540,1000).unwrap()), ..Default::default() };
        let second = ContextSnapshot { temporal: Some(TemporalContext::new(20261022,4,0,0,540,2000).unwrap()), ..Default::default() };
        assert_eq!(changes(&first,&second,2000), vec![ContextChange::LocalDay]);
    }
    #[test]
    fn stale_weather_never_produces_a_change() {
        let before = ContextSnapshot { weather: Some(WeatherContext::from_cache(WeatherCondition::Clear, 100, 200)), ..Default::default() };
        let after = ContextSnapshot { weather: Some(WeatherContext::from_cache(WeatherCondition::Rain, 100, 200)), ..Default::default() };
        assert!(changes(&before,&after,500).is_empty());
        assert_eq!(changes(&before,&after,150), vec![ContextChange::WeatherCondition]);
    }
}

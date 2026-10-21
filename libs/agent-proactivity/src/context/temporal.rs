use serde::{Deserialize, Serialize};
use super::Source;

/// Local date and clock from the host. Weekday is ISO 1 (Mon) through 7 (Sun).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporalContext {
    pub local_day: i32,
    pub iso_weekday: u8,
    pub hour: u8,
    pub minute: u8,
    pub utc_offset_minutes: i16,
    pub sampled_at_ms: i64,
    pub source: Source,
}
impl TemporalContext {
    pub fn new(day: i32, weekday: u8, hour: u8, minute: u8, offset: i16, now_ms: i64) -> Option<Self> {
        if !(20_000_101..=99_991_231).contains(&day) || !(1..=7).contains(&weekday)
            || hour > 23 || minute > 59 || !(-840..=840).contains(&offset) || now_ms < 0 {
            return None;
        }
        Some(Self { local_day: day, iso_weekday: weekday, hour, minute,
            utc_offset_minutes: offset, sampled_at_ms: now_ms, source: Source::System })
    }
    pub fn is_weekend(&self) -> bool { self.iso_weekday >= 6 }
    pub fn period(&self) -> &'static str {
        match self.hour { 5..=10 => "morning", 11..=13 => "midday", 14..=17 => "afternoon", 18..=21 => "evening", _ => "night" }
    }
}

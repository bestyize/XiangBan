use serde::{Deserialize, Serialize};
use super::Source;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayKind { Unknown, Workday, RestDay, Holiday, MakeupWorkday }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarContext {
    pub local_day: i32,
    /// Explicitly selected jurisdiction, not inferred from language, timezone, or GPS.
    pub region: Option<String>,
    pub day_kind: DayKind,
    pub holiday_code: Option<String>,
    pub officially_covered: bool,
    pub source: Option<Source>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HolidaySpan {
    pub code: String,
    pub start_day: i32,
    pub end_day: i32,
    pub makeup_days: Vec<i32>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CalendarRules {
    pub region: String,
    pub covered_years: Vec<i32>,
    pub spans: Vec<HolidaySpan>,
}

impl CalendarRules {
    /// Caller must load authoritative, versioned regional holiday data.
    /// Missing region or year means UNKNOWN, never assumed legal workday.
    pub fn resolve(&self, region: Option<&str>, local_day: i32, iso_weekday: u8) -> CalendarContext {
        let selected = region.map(str::to_string);
        let covered = region == Some(self.region.as_str()) &&
            self.covered_years.contains(&(local_day / 10000)) &&
            (1..=7).contains(&iso_weekday);
        let mut result = CalendarContext { local_day, region: selected, day_kind: DayKind::Unknown,
            holiday_code: None, officially_covered: covered,
            source: covered.then_some(Source::VerifiedCalendar) };
        if !covered { return result; }
        if let Some(span) = self.spans.iter().find(|s| (s.start_day..=s.end_day).contains(&local_day)) {
            result.day_kind = DayKind::Holiday;
            result.holiday_code = Some(span.code.clone());
        } else if let Some(span) = self.spans.iter().find(|s| s.makeup_days.contains(&local_day)) {
            result.day_kind = DayKind::MakeupWorkday;
            result.holiday_code = Some(span.code.clone());
        } else {
            result.day_kind = if iso_weekday >= 6 { DayKind::RestDay } else { DayKind::Workday };
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rules() -> CalendarRules {
        CalendarRules { region: "CN".into(), covered_years: vec![2026], spans: vec![
            HolidaySpan { code:"national_day".into(),start_day:20261001,end_day:20261007,makeup_days:vec![20261010] },
        ] }
    }
    #[test]
    fn makeup_weekend_overrides_simple_weekend() {
        let info = rules().resolve(Some("CN"),20261010,6);
        assert_eq!(info.day_kind, DayKind::MakeupWorkday);
        assert!(info.officially_covered);
        assert_eq!(info.holiday_code.as_deref(), Some("national_day"));
    }
    #[test]
    fn holiday_weekday_overrides_workweek() {
        assert_eq!(rules().resolve(Some("CN"),20261005,1).day_kind, DayKind::Holiday);
    }
    #[test]
    fn unknown_jurisdiction_or_year_is_not_guessed() {
        for region in [None, Some("KR"), Some("JP")] {
            let status = rules().resolve(region,20261010,6);
            assert_eq!(status.day_kind,DayKind::Unknown);
            assert!(!status.officially_covered);
        }
        assert_eq!(rules().resolve(Some("CN"),20271010,7).day_kind,DayKind::Unknown);
    }
}

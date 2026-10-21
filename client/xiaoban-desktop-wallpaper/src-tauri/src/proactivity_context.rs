//! Read-only adapters for already existing desktop signals.
//! No additional weather calls, calendar requests, OS hooks or screen inspection.
use agent_proactivity::context::{
    ActivityContext, CalendarContext, CalendarRules, ContextSnapshot, HolidaySpan,
    TaskContext, TaskState, TemporalContext, WeatherCondition, WeatherContext,
};
use chrono::{DateTime, Datelike, Local, Offset, Timelike};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
struct YearFile { year: i32, holidays: Vec<HolidayFile> }
#[derive(Deserialize)]
struct HolidayFile { code: String, start: String, end: String, makeup: Vec<String> }

fn day_key(value: &str) -> Option<i32> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?
        .format("%Y%m%d").to_string().parse().ok()
}
fn china_rules() -> &'static CalendarRules {
    static RULES: OnceLock<CalendarRules> = OnceLock::new();
    RULES.get_or_init(|| {
        // Shared, checked-in data also used by the calendar widget. Currently covers CN 2026 only.
        let text = include_str!("../../src/widgets/holidays.json");
        let Some(years) = serde_json::from_str::<Vec<YearFile>>(text).ok() else {
            return CalendarRules::default();
        };
        let mut rules = CalendarRules { region: "CN".into(), ..CalendarRules::default() };
        for year in years {
            let mut spans = Vec::new();
            for item in year.holidays {
                let (Some(start_day), Some(end_day)) = (day_key(&item.start), day_key(&item.end)) else { continue };
                let makeups: Vec<_> = item.makeup.iter().filter_map(|v| day_key(v)).collect();
                if makeups.len() != item.makeup.len() || start_day > end_day ||
                    start_day / 10000 != year.year || end_day / 10000 != year.year { continue; }
                spans.push(HolidaySpan { code: item.code, start_day, end_day, makeup_days: makeups });
            }
            if !spans.is_empty() {
                rules.covered_years.push(year.year);
                rules.spans.extend(spans);
            }
        }
        rules
    })
}

pub fn calendar(now: DateTime<Local>, region: Option<&str>) -> CalendarContext {
    let day: i32 = now.format("%Y%m%d").to_string().parse().unwrap_or(0);
    china_rules().resolve(region, day, now.weekday().number_from_monday() as u8)
}

pub fn weather_from_cache(record: &crate::care_weather::Weather, now_ms: i64) -> Option<WeatherContext> {
    // The existing weather module creates a controlled Chinese description from a weather code.
    // Consume only that known format; don't interpret arbitrary text or invent forecasts.
    let condition = record.text.split_once('：')?.1;
    let condition = if condition.starts_with("雷雨，") { WeatherCondition::Thunderstorm }
        else if condition.starts_with("晴，") { WeatherCondition::Clear }
        else if condition.starts_with("多云，") { WeatherCondition::Cloudy }
        else if condition.starts_with("雾，") { WeatherCondition::Fog }
        else if condition.starts_with("雨，") { WeatherCondition::Rain }
        else if condition.starts_with("雪，") { WeatherCondition::Snow }
        else { return None };
    let observed = record.updated_at.checked_mul(1000)?;
    let expiry = record.model_at.checked_add(7200)?
        .min(record.updated_at.checked_add(1800)?).checked_mul(1000)?;
    let sample = WeatherContext::from_cache(condition, observed, expiry);
    sample.is_fresh(now_ms).then_some(sample)
}

pub fn collect(
    now: DateTime<Local>,
    region: Option<&str>,
    activity: &crate::care_system::Activity,
    busy: bool,
    cached_weather: Option<&crate::care_weather::Weather>,
) -> ContextSnapshot {
    let at_ms = now.timestamp_millis();
    let local_day = now.format("%Y%m%d").to_string().parse().unwrap_or(0);
    let offset_minutes = now.offset().fix().local_minus_utc() / 60;
    let temporal = TemporalContext::new(local_day, now.weekday().number_from_monday() as u8,
        now.hour() as u8, now.minute() as u8, offset_minutes as i16, at_ms);
    ContextSnapshot {
        temporal,
        calendar: Some(calendar(now, region)),
        activity: Some(ActivityContext::from_system(activity.available,activity.locked,
            activity.idle_seconds,busy,None,at_ms)),
        weather: cached_weather.and_then(|weather| weather_from_cache(weather,at_ms)),
        task: None, // Preserve the latest task state separately when merging an OS sample.
    }
}

pub fn task_from_outcome(outcome: &crate::chat::task_outcome::TaskOutcome, at_ms: i64) -> TaskContext {
    use crate::chat::task_outcome::Status;
    let state = match outcome.status {
        Status::Succeeded => TaskState::Succeeded,
        Status::Failed => TaskState::Failed,
        Status::Partial => TaskState::Partial,
        Status::Cancelled => TaskState::Cancelled,
        Status::Unknown => TaskState::Unknown,
    };
    let verified = outcome.verified_at_ms.is_some() && outcome.validator != "run_trace_only";
    TaskContext::observed(state, verified, at_ms)
        .unwrap_or_else(|| TaskContext::observed(TaskState::Unknown,false,at_ms).expect("valid system timestamp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_proactivity::context::DayKind;
    use chrono::{LocalResult, TimeZone};
    fn local(y:i32,m:u32,d:u32) -> DateTime<Local> {
        match Local.with_ymd_and_hms(y,m,d,12,0,0) {
            LocalResult::Single(v) => v, LocalResult::Ambiguous(v,_) => v, LocalResult::None => panic!("missing local noon"),
        }
    }
    #[test]
    fn official_cn_makeup_overrides_saturday_but_unknown_region_remains_unknown() {
        let day = local(2026,10,10);
        assert_eq!(calendar(day,Some("CN")).day_kind,DayKind::MakeupWorkday);
        assert_eq!(calendar(day,Some("KR")).day_kind,DayKind::Unknown);
        assert_eq!(calendar(local(2027,10,10),Some("CN")).day_kind,DayKind::Unknown);
    }
    #[test]
    fn basic_system_context_does_not_assume_fullscreen_or_force_weather_network() {
        let sample = collect(local(2026,10,21),None,&crate::care_system::Activity{
            idle_seconds: 1.0,locked:false,available:true
        },false,None);
        assert_eq!(sample.activity.unwrap().full_screen,None);
        assert_eq!(sample.calendar.unwrap().day_kind,DayKind::Unknown);
        assert!(sample.weather.is_none());
    }
}

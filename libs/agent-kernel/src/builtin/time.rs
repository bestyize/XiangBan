use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

const WEEKDAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

/// Days since 1970-01-01 to (year, month, day), proleptic Gregorian.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

pub(crate) fn format_time(unix_secs: i64, offset_minutes: i32) -> String {
    let local = unix_secs + i64::from(offset_minutes) * 60;
    let days = local.div_euclid(86_400);
    let secs = local.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let weekday = WEEKDAYS[(days + 4).rem_euclid(7) as usize];
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let abs = offset_minutes.unsigned_abs();
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} (UTC{sign}{:02}:{:02}), {weekday}",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60,
        abs / 60,
        abs % 60
    )
}

/// Reports the current date and time. The kernel has no timezone database, so the host passes
/// the user's UTC offset; the default is UTC.
pub struct TimeNow {
    spec: ToolSpec,
    offset_minutes: i32,
}

impl TimeNow {
    pub fn new() -> Self {
        Self {
            spec: ToolSpec::new(
                "time_now",
                "Get the current date, time and weekday. Use it whenever the answer depends on today's date.",
                json!({ "type": "object", "properties": {} }),
            ).risk(Risk::Read).parallel(),
            offset_minutes: 0,
        }
    }

    /// Local offset from UTC in minutes, e.g. 480 for UTC+08:00.
    pub fn utc_offset_minutes(mut self, minutes: i32) -> Self {
        self.offset_minutes = minutes.clamp(-14 * 60, 14 * 60);
        self
    }
}

impl Default for TimeNow {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for TimeNow {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, _args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| format!("system clock is before 1970: {e}"))?;
        let secs = now.as_secs() as i64;
        Ok(format!("{}. Unix time: {secs}.", format_time(secs, self.offset_minutes)))
    }
}

#[cfg(test)]
mod tests {
    use super::format_time;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_time(0, 0), "1970-01-01 00:00:00 (UTC+00:00), Thursday");
        assert_eq!(format_time(-1, 0), "1969-12-31 23:59:59 (UTC+00:00), Wednesday");
        assert_eq!(format_time(951_782_400, 0), "2000-02-29 00:00:00 (UTC+00:00), Tuesday");
        assert_eq!(format_time(1_700_000_000, 0), "2023-11-14 22:13:20 (UTC+00:00), Tuesday");
        assert_eq!(format_time(1_700_000_000, 480), "2023-11-15 06:13:20 (UTC+08:00), Wednesday");
        assert_eq!(format_time(1_700_000_000, -330), "2023-11-14 16:43:20 (UTC-05:30), Tuesday");
    }
}

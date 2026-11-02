use crate::error::AppError;
use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Schedule {
    Once { at: DateTime<Utc> },
    Daily { time: String },
    Weekly { time: String, weekday: u32 },
}
impl Schedule {
    pub fn next(
        &self,
        zone: &str,
        after: DateTime<Utc>,
    ) -> Result<Option<DateTime<Utc>>, AppError> {
        let tz: Tz = zone
            .parse()
            .map_err(|_| AppError::BadRequest("请选择有效的 IANA 时区".into()))?;
        let (time, weekday) = match self {
            Self::Once { at } => return Ok((*at > after).then_some(*at)),
            Self::Daily { time } => (time, None),
            Self::Weekly { time, weekday } if *weekday <= 6 => (time, Some(*weekday)),
            _ => {
                return Err(AppError::BadRequest(
                    "星期必须为 0（周一）至 6（周日）".into(),
                ))
            }
        };
        if time.len() != 5 {
            return Err(AppError::BadRequest("时间格式应为 HH:mm".into()));
        }
        let time = NaiveTime::parse_from_str(time, "%H:%M")
            .map_err(|_| AppError::BadRequest("时间格式应为 HH:mm".into()))?;
        let date = after.with_timezone(&tz).date_naive();
        for day in 0..15 {
            let date = date + Duration::days(day);
            if weekday.is_some_and(|w| w != date.weekday().num_days_from_monday()) {
                continue;
            }
            // DST: skip nonexistent local times; use first occurrence of a repeated wall time.
            if let Some(local) = tz.from_local_datetime(&date.and_time(time)).earliest() {
                let next = local.with_timezone(&Utc);
                if next > after {
                    return Ok(Some(next));
                }
            }
        }
        Err(AppError::BadRequest("无法计算下次发送时间".into()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn dt(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }
    #[test]
    fn timezone_and_dst() {
        let daily = Schedule::Daily {
            time: "08:00".into(),
        };
        assert_eq!(
            daily
                .next("Asia/Shanghai", dt("2026-11-01T00:00:00Z"))
                .unwrap(),
            Some(dt("2026-11-02T00:00:00Z"))
        );
        let spring = Schedule::Daily {
            time: "02:30".into(),
        };
        assert_eq!(
            spring
                .next("America/New_York", dt("2026-03-08T05:00:00Z"))
                .unwrap(),
            Some(dt("2026-03-09T06:30:00Z"))
        );
        let fall = Schedule::Daily {
            time: "01:30".into(),
        };
        assert_eq!(
            fall.next("America/New_York", dt("2026-11-01T05:31:00Z"))
                .unwrap(),
            Some(dt("2026-11-02T06:30:00Z"))
        );
        assert!(daily.next("invalid", Utc::now()).is_err());
    }
    #[test]
    fn weekly_once_and_invalid() {
        let weekly = Schedule::Weekly {
            time: "09:00".into(),
            weekday: 0,
        };
        assert_eq!(
            weekly.next("UTC", dt("2026-11-01T23:59:00Z")).unwrap(),
            Some(dt("2026-11-02T09:00:00Z"))
        );
        assert!(Schedule::Weekly {
            time: "09:00".into(),
            weekday: 7
        }
        .next("UTC", Utc::now())
        .is_err());
        assert!(Schedule::Daily {
            time: "25:00".into()
        }
        .next("UTC", Utc::now())
        .is_err());
        assert!(Schedule::Once {
            at: dt("2026-01-01T00:00:00Z")
        }
        .next("UTC", Utc::now())
        .unwrap()
        .is_none());
    }
}

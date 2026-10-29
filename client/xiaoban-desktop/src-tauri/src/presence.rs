//! Companion presence: greets after the user comes back, on the first activity of a day, and when the
//! computer is still in use late at night. Pure state machine over the idle/lock readings that care
//! reminders already sample; it reads nothing else and never talks to the network.
use chrono::{NaiveDate, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const AWAY_SECONDS: f64 = 10.0 * 60.0;
pub const ACTIVE_IDLE_SECONDS: f64 = 60.0;
pub const WELCOME_DAILY_MAX: u32 = 3;
pub const WELCOME_GAP_SECONDS: i64 = 30 * 60;
pub const LATE_NIGHT_SECONDS: f64 = 20.0 * 60.0;
const LATE_NIGHT_END_HOUR: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind { WelcomeBack, FirstToday, LateNight }
impl Kind {
    pub fn as_str(self) -> &'static str {
        match self { Kind::WelcomeBack => "welcome_back", Kind::FirstToday => "first_today", Kind::LateNight => "late_night" }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub local: NaiveDateTime,
    pub unix: i64,
    pub idle_seconds: f64,
    /// Seconds of continuous sampling since the previous call; 0 after a sleep or clock jump.
    pub elapsed: f64,
    pub locked: bool,
    pub available: bool,
    /// Quiet hours, care master switch off, or companion hidden.
    pub quiet: bool,
    pub game: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Presence {
    date: Option<NaiveDate>,
    first_today: Option<NaiveDate>,
    welcomes: u32,
    last_welcome: Option<i64>,
    away: bool,
    late_seconds: f64,
    late_done: bool,
}

impl Presence {
    pub fn with_first_today(last: Option<NaiveDate>) -> Self { Self { first_today: last, ..Self::default() } }
    pub fn first_today_date(&self) -> Option<NaiveDate> { self.first_today }

    /// A clock that moves backwards never reopens a day that already ran.
    fn roll_date(&mut self, date: NaiveDate) -> NaiveDate {
        match self.date {
            Some(current) if date <= current => current,
            _ => {
                self.date = Some(date);
                self.welcomes = 0;
                self.late_seconds = 0.0;
                self.late_done = false;
                date
            }
        }
    }

    pub fn step(&mut self, input: Input) -> Option<Kind> {
        if !input.available { return None; }
        let today = self.roll_date(input.local.date());
        if input.idle_seconds >= AWAY_SECONDS { self.away = true; }
        if input.locked { return None; }
        let active = input.idle_seconds < ACTIVE_IDLE_SECONDS;
        if !active { return None; }
        let muted = input.quiet || input.game;
        let returned = std::mem::take(&mut self.away);
        let first_pending = self.first_today.is_none_or(|day| day < today);
        if first_pending { self.first_today = Some(today); }
        if muted { return None; }
        if first_pending {
            self.last_welcome = Some(input.unix);
            return Some(Kind::FirstToday);
        }
        if returned && self.welcomes < WELCOME_DAILY_MAX && self.last_welcome.is_none_or(|at| input.unix - at >= WELCOME_GAP_SECONDS) {
            self.welcomes += 1;
            self.last_welcome = Some(input.unix);
            return Some(Kind::WelcomeBack);
        }
        if input.local.hour() < LATE_NIGHT_END_HOUR && !self.late_done {
            self.late_seconds += input.elapsed.max(0.0);
            if self.late_seconds >= LATE_NIGHT_SECONDS {
                self.late_done = true;
                return Some(Kind::LateNight);
            }
        }
        None
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Saved { last_first_today: Option<NaiveDate> }

/// Unreadable or corrupt state counts as "not greeted yet"; a future date is a clock accident and is ignored.
pub fn load_first_today(path: &Path, today: NaiveDate) -> Option<NaiveDate> {
    let saved: Saved = std::fs::read(path).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default();
    saved.last_first_today.filter(|day| *day <= today)
}

pub fn save_first_today(path: &Path, day: NaiveDate) -> Result<(), String> {
    crate::library::save_json(path, &Saved { last_first_today: Some(day) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, TimeZone, Utc};

    fn at(day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap().and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
    }
    fn input(local: NaiveDateTime, idle: f64) -> Input {
        Input { local, unix: Utc.from_utc_datetime(&local).timestamp(), idle_seconds: idle, elapsed: 5.0, locked: false, available: true, quiet: false, game: false }
    }
    fn greeted_today() -> Presence { Presence::with_first_today(NaiveDate::from_ymd_opt(2026, 10, 9)) }
    fn settled() -> Presence {
        let mut p = Presence::default();
        assert_eq!(p.step(input(at(9, 9, 0), 1.0)), Some(Kind::FirstToday));
        p
    }
    fn away_then_back(p: &mut Presence, local: NaiveDateTime) -> Option<Kind> {
        p.step(input(local, AWAY_SECONDS));
        p.step(input(local + chrono::Duration::seconds(5), 0.0))
    }

    #[test] fn first_activity_of_the_day_greets_once() {
        let mut p = Presence::default();
        assert_eq!(p.step(input(at(9, 8, 0), 1.0)), Some(Kind::FirstToday));
        assert_eq!(p.step(input(at(9, 8, 1), 1.0)), None);
        assert_eq!(p.step(input(at(10, 8, 0), 1.0)), Some(Kind::FirstToday));
    }
    #[test] fn idle_user_is_not_first_activity() {
        let mut p = Presence::default();
        assert_eq!(p.step(input(at(9, 8, 0), ACTIVE_IDLE_SECONDS)), None);
        assert_eq!(p.step(input(at(9, 8, 5), ACTIVE_IDLE_SECONDS - 0.1)), Some(Kind::FirstToday));
    }
    #[test] fn return_threshold_is_inclusive() {
        let mut p = settled();
        p.step(input(at(9, 10, 0), AWAY_SECONDS - 1.0));
        assert_eq!(p.step(input(at(9, 10, 5), 0.0)), None);
        assert_eq!(away_then_back(&mut p, at(9, 11, 0)), Some(Kind::WelcomeBack));
    }
    #[test] fn welcome_needs_thirty_minutes_between() {
        let mut p = settled();
        assert_eq!(away_then_back(&mut p, at(9, 10, 0)), Some(Kind::WelcomeBack));
        assert_eq!(away_then_back(&mut p, at(9, 10, 29)), None);
        assert_eq!(away_then_back(&mut p, at(9, 10, 40)), Some(Kind::WelcomeBack));
    }
    #[test] fn welcome_is_capped_at_three_a_day_and_resets_next_day() {
        let mut p = settled();
        for hour in [10, 12, 14] { assert_eq!(away_then_back(&mut p, at(9, hour, 0)), Some(Kind::WelcomeBack)); }
        assert_eq!(away_then_back(&mut p, at(9, 16, 0)), None);
        assert_eq!(p.step(input(at(10, 9, 0), 0.0)), Some(Kind::FirstToday));
        assert_eq!(away_then_back(&mut p, at(10, 10, 0)), Some(Kind::WelcomeBack));
    }
    #[test] fn first_today_and_return_make_one_greeting() {
        let mut p = Presence::default();
        p.step(input(at(9, 8, 0), AWAY_SECONDS * 2.0));
        assert_eq!(p.step(input(at(9, 8, 1), 0.0)), Some(Kind::FirstToday));
        assert_eq!(p.step(input(at(9, 8, 6), 0.0)), None);
    }
    #[test] fn quiet_hours_and_game_stay_silent_and_do_not_replay() {
        for (quiet, game) in [(true, false), (false, true)] {
            let mut p = Presence::default();
            let mut first = input(at(9, 8, 0), 1.0); first.quiet = quiet; first.game = game;
            assert_eq!(p.step(first), None);
            assert_eq!(p.step(input(at(9, 9, 0), 1.0)), None);
            let mut back = input(at(9, 10, 0), AWAY_SECONDS); back.quiet = quiet; back.game = game;
            p.step(back);
            back.idle_seconds = 0.0; back.local = at(9, 10, 1); back.unix += 1;
            assert_eq!(p.step(back), None);
            assert_eq!(p.step(input(at(9, 10, 2), 0.0)), None);
        }
    }
    #[test] fn lock_and_sleep_resume_greets_once_without_replay() {
        let mut p = settled();
        let mut locked = input(at(9, 10, 0), AWAY_SECONDS * 3.0);
        locked.locked = true;
        assert_eq!(p.step(locked), None);
        locked.local = at(9, 14, 0); locked.idle_seconds = AWAY_SECONDS * 20.0;
        assert_eq!(p.step(locked), None);
        assert_eq!(p.step(input(at(9, 14, 5), AWAY_SECONDS * 20.0)), None);
        assert_eq!(p.step(input(at(9, 14, 10), 0.5)), Some(Kind::WelcomeBack));
        assert_eq!(p.step(input(at(9, 14, 15), 0.5)), None);
    }
    #[test] fn unreadable_activity_changes_nothing() {
        let mut p = Presence::default();
        let mut blind = input(at(9, 8, 0), 0.0); blind.available = false;
        assert_eq!(p.step(blind), None);
        assert_eq!(p.step(input(at(9, 8, 5), 0.0)), Some(Kind::FirstToday));
    }
    #[test] fn late_night_after_twenty_active_minutes_once_per_night() {
        let mut p = Presence::default();
        p.step(input(at(9, 0, 0), 0.0));
        let mut fired = Vec::new();
        for step in 1..=300 {
            let mut tick = input(at(9, 0, 0) + chrono::Duration::seconds(step * 5), 0.0); tick.elapsed = 5.0;
            if let Some(kind) = p.step(tick) { fired.push((step, kind)); }
        }
        assert_eq!(fired, vec![(240, Kind::LateNight)]);
    }
    #[test] fn late_night_ignores_idle_quiet_and_daytime() {
        let mut p = settled();
        let mut tick = input(at(10, 2, 0), 120.0); tick.elapsed = 5000.0;
        assert_eq!(p.step(tick), None);
        tick = input(at(10, 2, 1), 1.0); tick.quiet = true; tick.elapsed = 5000.0;
        assert_eq!(p.step(tick), None);
        tick = input(at(10, 5, 0), 1.0); tick.elapsed = 5000.0;
        assert_eq!(p.step(tick), None);
        tick = input(at(10, 12, 0), 1.0); tick.elapsed = 5000.0;
        assert_eq!(p.step(tick), None);
        tick = input(at(10, 4, 59), 1.0); tick.elapsed = LATE_NIGHT_SECONDS;
        assert_eq!(p.step(tick), Some(Kind::LateNight));
        assert_eq!(p.step(tick), None);
    }
    #[test] fn sleep_gap_with_zero_elapsed_is_not_counted_as_late_night() {
        let mut p = greeted_today();
        let mut tick = input(at(9, 3, 0), 1.0); tick.elapsed = 0.0;
        for _ in 0..1000 { assert_eq!(p.step(tick), None); }
    }
    #[test] fn crossing_midnight_resets_counts_and_allows_a_new_night() {
        let mut p = settled();
        let mut tick = input(at(9, 23, 59), 1.0); tick.elapsed = 5.0;
        assert_eq!(p.step(tick), None);
        tick = input(at(10, 0, 1), 1.0); tick.elapsed = LATE_NIGHT_SECONDS;
        assert_eq!(p.step(tick), Some(Kind::FirstToday));
        tick.local = at(10, 0, 2);
        assert_eq!(p.step(tick), Some(Kind::LateNight));
        tick.local = at(11, 0, 2);
        assert_eq!(p.step(tick), Some(Kind::FirstToday));
        tick.local = at(11, 0, 3);
        assert_eq!(p.step(tick), Some(Kind::LateNight));
    }
    #[test] fn clock_rollback_does_not_repeat_anything() {
        let mut p = settled();
        assert_eq!(away_then_back(&mut p, at(9, 10, 0)), Some(Kind::WelcomeBack));
        assert_eq!(away_then_back(&mut p, at(9, 9, 30)), None);
        assert_eq!(p.step(input(at(8, 9, 0), 0.0)), None);
        let mut late = input(at(9, 3, 0), 0.0); late.elapsed = LATE_NIGHT_SECONDS;
        assert_eq!(p.step(late), Some(Kind::LateNight));
        late.local = at(8, 3, 0);
        assert_eq!(p.step(late), None);
    }
    #[test] fn saved_day_prevents_a_second_greeting_after_restart() {
        let mut p = Presence::with_first_today(NaiveDate::from_ymd_opt(2026, 10, 9));
        assert_eq!(p.step(input(at(9, 9, 0), 0.0)), None);
        assert_eq!(p.first_today_date(), NaiveDate::from_ymd_opt(2026, 10, 9));
    }
    #[test] fn persisted_day_round_trips_and_failures_are_harmless() {
        let dir = std::env::temp_dir().join(format!("xiaoban-presence-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("config/presence.json");
        let today = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
        assert_eq!(load_first_today(&path, today), None);
        save_first_today(&path, today).unwrap();
        assert_eq!(load_first_today(&path, today), Some(today));
        assert_eq!(load_first_today(&path, today.pred_opt().unwrap()), None);
        std::fs::write(&path, b"{not json").unwrap();
        assert_eq!(load_first_today(&path, today), None);
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(save_first_today(&path, today).is_err());
        assert_eq!(load_first_today(&path, today), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

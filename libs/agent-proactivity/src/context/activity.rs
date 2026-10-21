use serde::{Deserialize, Serialize};
use super::Source;

/// Unknown means the OS has not supplied proof; never claim "not fullscreen".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionMode { Available, Idle, Locked, Busy, Unknown }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityContext {
    pub mode: AttentionMode,
    pub full_screen: Option<bool>,
    pub sampled_at_ms: i64,
    pub source: Source,
}
impl ActivityContext {
    pub fn from_system(available: bool, locked: bool, idle_seconds: f64,
        quiet_or_focus_or_game: bool, full_screen: Option<bool>, now_ms: i64) -> Self {
        let mode = if locked { AttentionMode::Locked }
            else if quiet_or_focus_or_game || full_screen == Some(true) { AttentionMode::Busy }
            else if !available || !idle_seconds.is_finite() || idle_seconds < 0.0 { AttentionMode::Unknown }
            else if idle_seconds >= 60.0 { AttentionMode::Idle }
            else { AttentionMode::Available };
        Self { mode, full_screen, sampled_at_ms: now_ms, source: Source::System }
    }
    pub fn can_interrupt(&self, now_ms: i64) -> bool {
        self.mode == AttentionMode::Available && self.sampled_at_ms >= 0 &&
            now_ms >= self.sampled_at_ms && now_ms.saturating_sub(self.sampled_at_ms) <= 15_000
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_activity_is_not_assumed_free() {
        assert_eq!(ActivityContext::from_system(false,false,f64::INFINITY,false,None,100).mode,AttentionMode::Unknown);
        assert!(!ActivityContext::from_system(true,false,0.0,true,None,100).can_interrupt(100));
        assert!(!ActivityContext::from_system(true,true,0.0,false,None,100).can_interrupt(100));
        assert_eq!(ActivityContext::from_system(true,false,0.0,false,Some(true),100).mode,AttentionMode::Busy);
        assert!(ActivityContext::from_system(true,false,0.0,false,None,100).can_interrupt(100));
        assert!(!ActivityContext::from_system(true,false,0.0,false,None,100).can_interrupt(16_000));
    }
}

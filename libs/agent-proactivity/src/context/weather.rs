use serde::{Deserialize, Serialize};
use super::Source;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherCondition { Clear, Cloudy, Fog, Rain, Snow, Thunderstorm }

/// Only coarse conditions from an already cached, user-configured weather source.
/// No location, temperature inference, hourly forecasts, or automatic network fetch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherContext {
    pub condition: WeatherCondition,
    pub observed_at_ms: i64,
    pub expires_at_ms: i64,
    pub source: Source,
}
impl WeatherContext {
    pub fn from_cache(condition: WeatherCondition, observed_at_ms: i64, expires_at_ms: i64) -> Self {
        Self { condition, observed_at_ms, expires_at_ms, source: Source::ExistingWeatherCache }
    }
    pub fn is_fresh(&self, now_ms: i64) -> bool {
        self.observed_at_ms >= 0 && now_ms >= self.observed_at_ms && now_ms < self.expires_at_ms &&
            self.expires_at_ms.saturating_sub(self.observed_at_ms) <= 2 * 60 * 60 * 1000
    }
}

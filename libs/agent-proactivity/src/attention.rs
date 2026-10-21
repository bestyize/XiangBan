use serde::{Deserialize, Serialize};
use crate::policy::{Candidate, Policy};
use crate::engine::Suppression;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TopicSeen { id: String, at_ms: i64 }

/// Persistable *aggregates*, not raw events or content. Counts and topic keys are bounded.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Attention {
    day_key: i32,
    units_today: u16,
    last_interruption_ms: Option<i64>,
    topics: Vec<TopicSeen>,
}
impl Attention {
    pub fn units_today(&self) -> u16 { self.units_today }
    pub fn last_interruption_ms(&self) -> Option<i64> { self.last_interruption_ms }
    pub(crate) fn check(&self, candidate: &Candidate, policy: &Policy, day: i32, now: i64) -> Option<Suppression> {
        let cost = candidate.channel.cost();
        if day < self.day_key || self.last_interruption_ms.is_some_and(|last| now < last) {
            return Some(Suppression::ClockRollback);
        }
        if cost == 0 { return None; }
        if cost > policy.max_daily_units { return Some(Suppression::DailyBudget); }
        let used = if day > self.day_key { 0 } else { self.units_today };
        if used.saturating_add(cost) > policy.max_daily_units { return Some(Suppression::DailyBudget); }
        if self.last_interruption_ms.is_some_and(|last| now.saturating_sub(last) < policy.min_gap_ms.max(0)) {
            return Some(Suppression::Cooldown);
        }
        if candidate.topic_id.as_deref().is_some_and(|topic| self.topics.iter().any(|seen| {
            seen.id == topic && now.saturating_sub(seen.at_ms) < policy.topic_gap_ms.max(0)
        })) { return Some(Suppression::RepeatedTopic); }
        None
    }
    pub(crate) fn reserve(&mut self, candidate: &Candidate, day: i32, now: i64) {
        if day > self.day_key { self.day_key = day; self.units_today = 0; }
        let cost = candidate.channel.cost();
        if cost == 0 { return; }
        self.units_today = self.units_today.saturating_add(cost);
        self.last_interruption_ms = Some(now);
        self.topics.retain(|seen| now.saturating_sub(seen.at_ms) < 48 * 60 * 60 * 1000);
        if let Some(topic) = &candidate.topic_id {
            self.topics.retain(|seen| &seen.id != topic);
            self.topics.push(TopicSeen { id: topic.clone(), at_ms: now });
            if self.topics.len() > 64 { self.topics.drain(..self.topics.len()-64); }
        }
    }
}

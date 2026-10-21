use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::attention::Attention;
use crate::event::{valid_key, Event, EventKind};
use crate::feedback::{Feedback, FeedbackState};
use crate::policy::{ActionKind, Candidate, Channel, Context, Policy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suppression {
    Disabled, NoConsent, Unavailable, Busy, Snoozed, Invalid, Expired,
    ClockRollback, DailyBudget, Cooldown, RepeatedTopic,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub id: u64,
    pub kind: ActionKind,
    pub channel: Channel,
    pub topic_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision { Allowed(Action), Suppressed(Suppression) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation { Accepted, Rejected }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Receipt { id: u64, feedback_recorded: bool }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EngineState {
    /// Counts per class only. No actor identity, content, raw pointer locations, or event log.
    observed: BTreeMap<EventKind, u64>,
    attention: Attention,
    feedback: FeedbackState,
    next_id: u64,
    receipts: Vec<Receipt>,
}
impl Default for EngineState {
    fn default() -> Self {
        Self { observed: BTreeMap::new(), attention: Attention::default(), feedback: FeedbackState::default(), next_id: 1, receipts: Vec::new() }
    }
}
impl EngineState {
    pub fn count(&self, kind: EventKind) -> u64 { *self.observed.get(&kind).unwrap_or(&0) }
    pub fn attention(&self) -> &Attention { &self.attention }
    pub fn feedback(&self) -> &FeedbackState { &self.feedback }
}

pub struct Engine { policy: Policy, state: EngineState }
impl Engine {
    pub fn new(policy: Policy) -> Self { Self { policy, state: EngineState::default() } }
    pub fn restore(policy: Policy, state: EngineState) -> Self { Self { policy, state } }
    pub fn snapshot(&self) -> EngineState { self.state.clone() }

    /// Analyze only the event category. Never retain a subject or sampled private data.
    pub fn observe(&mut self, event: &Event) -> Observation {
        if !event.valid() { return Observation::Rejected; }
        let counter = self.state.observed.entry(event.kind).or_insert(0);
        *counter = counter.saturating_add(1);
        Observation::Accepted
    }

    /// An explicit authorization+reservation step. A returned action grants no OS permission.
    /// A failed delivery may conservatively consume budget; this prevents rapid repeated popups.
    pub fn consider(&mut self, candidate: Candidate, ctx: &Context, now: i64) -> Decision {
        if !self.policy.enabled { return Decision::Suppressed(Suppression::Disabled); }
        if !ctx.opted_in { return Decision::Suppressed(Suppression::NoConsent); }
        if now < 0 || !(20_000_000..=99_999_999).contains(&ctx.local_day)
            || candidate.topic_id.as_deref().is_some_and(|id| !valid_key(id)) {
            return Decision::Suppressed(Suppression::Invalid);
        }
        if ctx.locked || !ctx.user_active { return Decision::Suppressed(Suppression::Unavailable); }
        // The engine also serves non-pet actions: hiding pets must not disable all suggestions.
        if matches!(candidate.kind, ActionKind::PetMotion | ActionKind::CompanionGreeting) && !ctx.pet_visible {
            return Decision::Suppressed(Suppression::Unavailable);
        }
        if ctx.quiet || ctx.game || ctx.fullscreen { return Decision::Suppressed(Suppression::Busy); }
        if candidate.expires_at_ms.is_some_and(|expires| now > expires) {
            return Decision::Suppressed(Suppression::Expired);
        }
        if now < self.state.feedback.snoozed_until_ms { return Decision::Suppressed(Suppression::Snoozed); }
        if let Some(reason) = self.state.attention.check(&candidate, &self.policy, ctx.local_day, now) {
            return Decision::Suppressed(reason);
        }
        if self.state.next_id == u64::MAX { return Decision::Suppressed(Suppression::Invalid); }
        let id = self.state.next_id;
        self.state.next_id += 1;
        self.state.attention.reserve(&candidate, ctx.local_day, now);
        self.state.receipts.push(Receipt { id, feedback_recorded: false });
        if self.state.receipts.len() > 64 { let excess = self.state.receipts.len() - 64; self.state.receipts.drain(..excess); }
        Decision::Allowed(Action { id, kind: candidate.kind, channel: candidate.channel, topic_id: candidate.topic_id })
    }

    /// Feedback only applies to actions actually issued by this engine, once per action.
    pub fn feedback(&mut self, action_id: u64, kind: Feedback, now: i64) -> bool {
        if now < 0 { return false; }
        let Some(receipt) = self.state.receipts.iter_mut().find(|r| r.id == action_id && !r.feedback_recorded) else { return false; };
        receipt.feedback_recorded = true;
        self.state.feedback.record(kind, now, &self.policy);
        true
    }
}

/// Suggested actions are proposals only; the host must enable policy and explicitly consider them.
/// In particular, a pointer hover below 800 ms should never generate a text suggestion.
pub fn suggest(event: &Event) -> Option<Candidate> {
    if !event.valid() { return None; }
    match event.kind {
        EventKind::FirstActivityOfDay => Some(Candidate::new(ActionKind::CompanionGreeting, Channel::Bubble).topic("first_today")),
        EventKind::UserReturned => Some(Candidate::new(ActionKind::CompanionGreeting, Channel::Bubble).topic("welcome_back")),
        EventKind::LateNightActive => Some(Candidate::new(ActionKind::PetMotion, Channel::Ambient)),
        EventKind::PetHovered if event.duration_ms.unwrap_or(0) >= 800 =>
            Some(Candidate::new(ActionKind::CompanionGreeting, Channel::Bubble).topic("pet_hover")),
        EventKind::PetPoked => Some(Candidate::new(ActionKind::PetMotion, Channel::Ambient)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventSource;
    fn on() -> Policy { Policy { enabled: true, ..Policy::default() } }
    fn ctx(day: i32) -> Context { Context { local_day: day, opted_in: true, user_active: true, pet_visible: true, ..Context::default() } }
    fn candidate(topic: &str) -> Candidate { Candidate::new(ActionKind::CompanionGreeting, Channel::Bubble).topic(topic) }
    fn emitted(result: Decision) -> u64 { match result { Decision::Allowed(a) => a.id, other => panic!("expected action, got {other:?}") } }
    const DAY: i32 = 20261021;
    const HOUR: i64 = 3_600_000;

    #[test] fn starts_disabled_and_requires_explicit_consent() {
        let mut e = Engine::new(Policy::default());
        assert_eq!(e.consider(candidate("welcome"), &ctx(DAY), 1000), Decision::Suppressed(Suppression::Disabled));
        let mut e = Engine::new(on()); let mut c = ctx(DAY); c.opted_in = false;
        assert_eq!(e.consider(candidate("welcome"), &c, 1000), Decision::Suppressed(Suppression::NoConsent));
    }
    #[test] fn quiet_lock_game_fullscreen_and_inactivity_block_all_channels() {
        for i in 0..6 {
            let mut c = ctx(DAY);
            match i {
                0 => c.locked = true, 1 => c.quiet = true, 2 => c.game = true,
                3 => c.fullscreen = true, 4 => c.user_active = false, _ => c.pet_visible = false,
            }
            assert!(matches!(Engine::new(on()).consider(Candidate::new(ActionKind::PetMotion, Channel::Ambient), &c, 1000), Decision::Suppressed(_)));
        }
    }
    #[test] fn non_pet_actions_do_not_require_a_visible_pet() {
        let mut e = Engine::new(on());
        let mut context = ctx(DAY);
        context.pet_visible = false;
        let suggestion = Candidate::new(ActionKind::AgentSuggestion, Channel::Bubble).topic("task_update");
        emitted(e.consider(suggestion, &context, HOUR));
        assert_eq!(e.consider(candidate("pet_only"), &context, 3*HOUR), Decision::Suppressed(Suppression::Unavailable));
    }
    #[test] fn global_cooldown_budget_and_topic_repeat() {
        let mut e = Engine::new(on());
        emitted(e.consider(candidate("morning"), &ctx(DAY), 1000));
        assert_eq!(e.consider(candidate("other"), &ctx(DAY), 1000 + HOUR), Decision::Suppressed(Suppression::Cooldown));
        assert_eq!(e.consider(candidate("morning"), &ctx(DAY), 1000 + 2*HOUR), Decision::Suppressed(Suppression::RepeatedTopic));
        emitted(e.consider(candidate("afternoon"), &ctx(DAY), 1000 + 2*HOUR));
        emitted(e.consider(candidate("evening"), &ctx(DAY), 1000 + 4*HOUR));
        assert_eq!(e.snapshot().attention().units_today(), 6);
        assert_eq!(e.consider(candidate("night"), &ctx(DAY), 1000 + 6*HOUR), Decision::Suppressed(Suppression::DailyBudget));
    }
    #[test] fn calendar_changes_reset_daily_budget_but_not_topic_repeats() {
        let mut e = Engine::new(on());
        emitted(e.consider(candidate("same"), &ctx(DAY), HOUR));
        assert_eq!(e.consider(candidate("same"), &ctx(20261022), HOUR + 20*HOUR), Decision::Suppressed(Suppression::RepeatedTopic));
        emitted(e.consider(candidate("next"), &ctx(20261022), HOUR + 20*HOUR));
        assert_eq!(e.snapshot().attention().units_today(), 2);
        assert_eq!(e.consider(candidate("prior_day"), &ctx(DAY), HOUR + 23*HOUR), Decision::Suppressed(Suppression::ClockRollback));
    }
    #[test] fn ignored_is_not_dismissed_and_feedback_is_idempotent() {
        let mut e = Engine::new(on());
        let first = emitted(e.consider(candidate("one"), &ctx(DAY), HOUR));
        assert!(e.feedback(first, Feedback::Ignored, HOUR+10));
        assert!(!e.feedback(first, Feedback::Dismissed, HOUR+11));
        let second = emitted(e.consider(candidate("two"), &ctx(DAY), 3*HOUR));
        assert!(e.feedback(second, Feedback::Dismissed, 3*HOUR));
        let third = emitted(e.consider(candidate("three"), &ctx(DAY), 5*HOUR));
        assert!(e.feedback(third, Feedback::Engaged, 5*HOUR));
        assert_eq!(e.snapshot().feedback().snoozed_until_ms(), 0);
    }
    #[test] fn repeated_dismissals_snooze_and_survive_snapshot_restart() {
        let mut e = Engine::new(on());
        for (i,t) in [HOUR,3*HOUR,5*HOUR].into_iter().enumerate() {
            let id = emitted(e.consider(candidate(&format!("topic_{i}")), &ctx(DAY), t));
            assert!(e.feedback(id, Feedback::Dismissed, t+10));
        }
        let state: EngineState = serde_json::from_str(&serde_json::to_string(&e.snapshot()).unwrap()).unwrap();
        let mut restored = Engine::restore(on(), state);
        assert_eq!(restored.consider(candidate("four"), &ctx(20261022), 7*HOUR), Decision::Suppressed(Suppression::Snoozed));
    }
    #[test] fn no_actor_ids_or_content_are_persisted_from_events() {
        let mut e = Engine::new(on());
        let event = Event::new(EventKind::UserReturned, EventSource::Desktop, HOUR).subject("my-pet").duration(2*HOUR as u64);
        assert_eq!(e.observe(&event), Observation::Accepted);
        assert_eq!(e.snapshot().count(EventKind::UserReturned), 1);
        let snapshot = serde_json::to_string(&e.snapshot()).unwrap();
        assert!(!snapshot.contains("my-pet"));
        assert!(!snapshot.contains("7200000"));
        assert_eq!(e.observe(&Event::new(EventKind::PetHovered, EventSource::Pet, HOUR).subject("../secret")), Observation::Rejected);
    }
    #[test] fn hover_threshold_and_suggestion_are_separate_from_delivery() {
        let short = Event::new(EventKind::PetHovered, EventSource::Pet, 100).duration(799);
        assert!(suggest(&short).is_none());
        let long = Event::new(EventKind::PetHovered, EventSource::Pet, 200).duration(800);
        assert_eq!(suggest(&long).unwrap().topic_id.as_deref(), Some("pet_hover"));
        let e = Engine::new(Policy::default());
        assert_eq!(e.snapshot().attention().units_today(), 0);
    }
    #[test] fn expired_invalid_and_rollback_events_fail_closed() {
        let mut e = Engine::new(on());
        assert_eq!(e.consider(candidate("bad key"), &ctx(DAY), HOUR), Decision::Suppressed(Suppression::Invalid));
        assert_eq!(e.consider(candidate("past").expires_at(123), &ctx(DAY), HOUR), Decision::Suppressed(Suppression::Expired));
        emitted(e.consider(candidate("good"), &ctx(DAY), HOUR));
        assert_eq!(e.consider(candidate("later"), &ctx(DAY), HOUR-1), Decision::Suppressed(Suppression::ClockRollback));
    }
    #[test] fn silent_motion_never_consumes_budget_or_minimum_gap() {
        let mut e = Engine::new(on());
        let motion = Candidate::new(ActionKind::PetMotion, Channel::Ambient);
        for _ in 0..20 { emitted(e.consider(motion.clone(), &ctx(DAY), HOUR)); }
        assert_eq!(e.snapshot().attention().units_today(), 0);
        emitted(e.consider(candidate("first"), &ctx(DAY), HOUR));
    }
}

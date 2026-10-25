use crate::agent::{Agent, RetryPolicy};
use crate::error::HarnessError;
use crate::middleware::{estimate_tokens, estimate_tool_tokens, ContextTrim, Middleware, Stop, StopDecision, Turn};
use crate::provider::{ChatRequest, Reply};
use crate::types::{Event, FinishReason, Message, Part, Role, Usage};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub struct RunSummary {
    pub run_id: String,
    pub text: String,
    pub turns: usize,
    pub usage: Usage,
    /// Why the last reply ended. `Length` means the final answer may be cut off.
    pub finish: FinishReason,
}

const CUT_OFF_MESSAGE: &str = "Not run: your reply hit the output length limit, so this tool call may be incomplete. \
                               No calls in this batch were executed. Do not continue or concatenate partial JSON. \
                               Retry with complete, smaller tool calls: write files separately and use targeted edits \
                               instead of sending the entire artifact again. Keep all previously successful work.";

struct SessionGuard<'a> {
    agent: &'a Agent,
    id: String,
}

impl Drop for SessionGuard<'_> {
    fn drop(&mut self) {
        self.agent.running.lock().unwrap_or_else(|e| e.into_inner()).remove(&self.id);
    }
}

/// A crash between a tool call and its result leaves history that providers reject. Close it with
/// error results; only the tail is handled, because that is the only place a crash can leave a gap.
fn dangling_tool_results(history: &[Message]) -> Vec<Message> {
    let Some(pos) = history.iter().rposition(|m| m.role == Role::Assistant && !m.tool_calls.is_empty()) else {
        return Vec::new();
    };
    let tail = &history[pos + 1..];
    if tail.iter().any(|m| m.role != Role::Tool) {
        return Vec::new();
    }
    let answered: HashSet<&str> = tail.iter().filter_map(|m| m.tool_call_id.as_deref()).collect();
    history[pos]
        .tool_calls
        .iter()
        .filter(|c| !answered.contains(c.id.as_str()))
        .map(|c| crate::ToolResult::error("this tool call was interrupted before its result was saved; the action may already have happened. Check actual state before retrying; do not replay automatically.").message(c.id.as_str()))
        .collect()
}

fn backoff(policy: &RetryPolicy, attempt: u32) -> Duration {
    let factor = 1u32 << (attempt - 1).min(16);
    policy.base_delay.saturating_mul(factor).min(policy.max_delay)
}

fn sleep_unless_cancelled(delay: Duration, cancel: &AtomicBool) -> Result<(), HarnessError> {
    let end = Instant::now() + delay;
    while Instant::now() < end {
        if cancel.load(Ordering::Relaxed) {
            return Err(HarnessError::Cancelled);
        }
        std::thread::sleep(Duration::from_millis(20).min(end - Instant::now()));
    }
    Ok(())
}

impl Agent {
    /// Runs one user message to completion: model calls and tool rounds until a plain reply.
    /// The terminal event (`Done`, `Cancelled` or `Error`) is always emitted before returning.
    pub fn run(
        &self,
        session: &str,
        input: &str,
        on_event: &mut dyn FnMut(Event),
        cancel: &Arc<AtomicBool>,
    ) -> Result<RunSummary, HarnessError> {
        self.run_parts(session, vec![Part::Text { text: input.to_string() }], on_event, cancel)
    }

    pub fn run_parts(
        &self,
        session: &str,
        input: Vec<Part>,
        on_event: &mut dyn FnMut(Event),
        cancel: &Arc<AtomicBool>,
    ) -> Result<RunSummary, HarnessError> {
        self.run_parts_observed(session, input, crate::RunOptions::default(), &mut |record| {
            if let crate::RunEventBody::Event { event } = record.body {
                if !matches!(event, Event::ToolDispatched { .. }) { on_event(event); }
            }
            Ok(())
        }, cancel)
    }

    /// Observe typed run states and sequenced events. Observer failures cancel further dispatch.
    /// The observer may persist records before returning Ok, without involving message storage.
    pub fn run_parts_observed(
        &self, session: &str, input: Vec<Part>, options: crate::RunOptions,
        observe: &mut dyn FnMut(crate::RunEvent) -> Result<(), HarnessError>, cancel: &Arc<AtomicBool>,
    ) -> Result<RunSummary, HarnessError> {
        self.run_observed_inner(session, Some(input), options, observe, cancel)
    }

    /// Retry only a user message which has not produced a persisted assistant/tool message.
    pub fn retry_last(&self, session: &str, on_event: &mut dyn FnMut(Event), cancel: &Arc<AtomicBool>) -> Result<RunSummary, HarnessError> {
        self.retry_last_observed(session, crate::RunOptions::default(), &mut |record| {
            if let crate::RunEventBody::Event { event } = record.body {
                if !matches!(event, Event::ToolDispatched { .. }) { on_event(event); }
            }
            Ok(())
        }, cancel)
    }

    pub fn retry_last_observed(
        &self, session: &str, options: crate::RunOptions,
        observe: &mut dyn FnMut(crate::RunEvent) -> Result<(), HarnessError>, cancel: &Arc<AtomicBool>,
    ) -> Result<RunSummary, HarnessError> {
        self.run_observed_inner(session, None, options, observe, cancel)
    }

    fn run_observed_inner(
        &self, session: &str, input: Option<Vec<Part>>, mut options: crate::RunOptions,
        observe: &mut dyn FnMut(crate::RunEvent) -> Result<(), HarnessError>, cancel: &Arc<AtomicBool>,
    ) -> Result<RunSummary, HarnessError> {
        use crate::{RunEnd, RunEventBody, RunState};
        let control=options.control.get_or_insert_with(Default::default).clone();
        let _watch=control.watch(cancel.clone());
        let mut emitter = crate::runtime::RunEmitter::new(session, &options, observe);
        let run_id = emitter.run_id.clone();
        // Keep the session locked through cleanup and terminal observation. A rejected competing
        // run must never clean up resources belonging to the run that owns the session.
        let _guard = match self.acquire(session) {
            Ok(guard) => guard,
            Err(error) => {
                emitter.emit(RunEventBody::Event { event: Event::Error { message: error.to_string() } })?;
                emitter.emit(RunEventBody::State { state: RunState::Failed, reason: Some(error.to_string()) })?;
                return Err(error);
            }
        };
        emitter.emit(RunEventBody::State { state: RunState::Running, reason: None })?;
        let mut observation_error = None;
        let mut result = {
            let mut forward = |event| {
                if observation_error.is_none() {
                    if let Err(error) = emitter.emit(RunEventBody::Event { event }) {
                        observation_error = Some(error);
                        cancel.store(true, Ordering::SeqCst);
                    }
                }
            };
            self.run_inner(session, &run_id, input, &options, &mut forward, cancel)
        };
        if let Some(error) = observation_error { result = Err(error); }
        if !matches!(&result,Err(HarnessError::Paused(reason))if reason=="tool_outcome_needs_review"){if let Err(e)=control.check(){result=Err(e);}}
        let end = RunEnd { run_id, session_id: session.into(), state: crate::runtime::state_of(&result), ephemeral_session: options.ephemeral_session };
        for tool in &self.tools {
            if let Err(error) = tool.on_run_end(&end) {
                result = Err(HarnessError::Middleware(format!("run cleanup failed: {error}")));
            }
        }
        if crate::runtime::state_of(&result) != end.state {
            let failed = RunEnd { state: RunState::Failed, ..end.clone() };
            for tool in &self.tools { let _ = tool.on_run_end(&failed); }
        }
        let event = match &result {
            Ok(_) => Event::Done,
            Err(HarnessError::Cancelled) => Event::Cancelled,
            Err(error) => Event::Error { message: error.to_string() },
        };
        let finalized = emitter.emit(RunEventBody::Event { event }).and_then(|_| {
            emitter.emit(RunEventBody::State { state: crate::runtime::state_of(&result), reason: result.as_ref().err().map(ToString::to_string) })
        });
        if let Err(error) = finalized {
            let failed = RunEnd { state: RunState::Failed, ..end };
            for tool in &self.tools { let _ = tool.on_run_end(&failed); }
            return Err(error);
        }
        result
    }

    fn acquire(&self, session: &str) -> Result<SessionGuard<'_>, HarnessError> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if !running.insert(session.to_string()) {
            return Err(HarnessError::SessionBusy(session.to_string()));
        }
        Ok(SessionGuard { agent: self, id: session.to_string() })
    }

    fn run_inner(
        &self,
        session: &str,
        run_id: &str,
        input: Option<Vec<Part>>,
        options: &crate::RunOptions,
        on_event: &mut dyn FnMut(Event),
        cancel: &Arc<AtomicBool>,
    ) -> Result<RunSummary, HarnessError> {
        let mut history = self.store.load(session)?;
        if input.is_none() && !history.last().is_some_and(|m| m.role == Role::User) {
            return Err(HarnessError::Config("The last message has already produced output or tool actions; it cannot be replayed.".into()));
        }
        let repair = dangling_tool_results(&history);
        if !repair.is_empty() {
            self.store.append(session, &repair)?;
            history.extend(repair);
        }
        if let Some(input) = input {
            let user = Message::user(input);
            self.store.append(session, std::slice::from_ref(&user))?;
            history.push(user);
        }
        let run_start = history.len().saturating_sub(1);

        let mut usage = Usage::default();
        let mut continuations = 0;
        let mut continuations_without_progress = 0;
        let mut successful_calls = HashSet::new();
        let mut truncated_tool_retries = 0usize;
        let mut tool_calls = 0usize;
        // Shared with sub-agents this run starts, so their calls count against `max_tool_calls` too.
        let budget = options.shared_budget.clone().or_else(|| options.max_tool_calls.map(|limit| Arc::new(crate::ToolBudget::new(limit))));
        let own_limit = options.max_tool_calls.filter(|_| options.shared_budget.is_some());
        let mut recovered_context_budget = None;
        let control=options.control.clone().unwrap_or_default();
        let mut idle_rounds=0;
        for index in 0..self.max_turns.unwrap_or(usize::MAX) {
            control.check()?;
            if cancel.load(Ordering::Relaxed) {
                return Err(HarnessError::Cancelled);
            }
            let mut messages = Vec::with_capacity(history.len() + 1);
            let system = self.system_text();
            if !system.is_empty() {
                messages.push(Message::system(system));
            }
            messages.extend_from_slice(&history);
            let mut turn = Turn { session: session.to_string(), index, messages, tools: self.tool_schemas() };
            for mw in &self.middleware {
                mw.before_model(&mut turn)?;
            }
            let mut request = ChatRequest { messages: turn.messages, tools: turn.tools };
            let mut reply = self.chat_with_retry(session, run_id, &mut request, &mut recovered_context_budget, on_event, cancel)?;
            let turn = Turn { session: turn.session, index, messages: request.messages, tools: request.tools };
            for mw in &self.middleware {
                mw.after_model(&turn, &mut reply.message)?;
            }
            if let Some(u) = reply.usage {
                usage += u;
                control.record(u);
                on_event(Event::Usage { prompt_tokens: u.prompt_tokens, completion_tokens: u.completion_tokens });
            }

            let finish = reply.finish;
            let mut assistant = reply.message;
            let cut_off = finish == FinishReason::Length;
            // Some providers (DeepSeek over Responses) end a length-limited reply as a normal tool call.
            // Arguments that stop mid-JSON can only be a cut-off reply, so they get the same handling.
            let eof_cut = assistant.tool_calls.iter().any(|c| {
                !c.arguments.trim().is_empty()
                    && serde_json::from_str::<serde_json::Value>(&c.arguments).is_err_and(|e| e.is_eof())
            });
            let cut_calls = (cut_off || eof_cut) && !assistant.tool_calls.is_empty();
            if cut_calls {
                // Half-written arguments would be sent back to the model as history; keep them parseable.
                for call in &mut assistant.tool_calls {
                    if !serde_json::from_str::<serde_json::Value>(&call.arguments).is_ok_and(|v| v.is_object()) {
                        call.arguments = "{}".to_string();
                    }
                }
            }
            let mut identities=HashSet::new();
            let invalid_ids=assistant.tool_calls.iter().any(|c|c.id.trim().is_empty()||!identities.insert(c.id.clone()));
            if invalid_ids {
                // Never persist an invalid protocol history that makes the next authorized turn fail too.
                assistant.structured_content=Some(serde_json::json!({"rejected_model_calls":assistant.tool_calls}));
                assistant.provider_state=None;
                for (i,call) in assistant.tool_calls.iter_mut().enumerate(){
                    call.id=format!("rejected_{}_{}_{}",run_id.replace('-',"_"),index,i);
                    if !serde_json::from_str::<serde_json::Value>(&call.arguments).is_ok_and(|v|v.is_object()){call.arguments="{}".into();}
                }
            }
            self.store.append(session, std::slice::from_ref(&assistant))?;
            let (calls, text) = (assistant.tool_calls.clone(), assistant.text());
            history.push(assistant);
            control.check()?;
            if matches!(finish, FinishReason::ContentFilter | FinishReason::Other) || invalid_ids
                || (finish == FinishReason::ToolCalls && calls.is_empty()) {
                let results = self.reject_calls(session, &calls, "Not run: abnormal model terminal state or duplicate/empty tool identity. The entire batch was rejected.", on_event);
                let messages: Vec<_> = calls.iter().zip(results).map(|(c,r)|r.message(&c.id)).collect();
                self.store.append(session, &messages)?;
                return Err(HarnessError::Paused("invalid_model_terminal_or_call_identity".into()));
            }
            if calls.is_empty() {
                if cancel.load(Ordering::Relaxed) {
                    return Err(HarnessError::Cancelled);
                }
                let stop = Stop { session, history: &history, run_start, finish, continuations, continuations_without_progress };
                let decision = self.middleware.iter().map(|mw| mw.before_stop(&stop))
                    .find(|decision| !matches!(decision, StopDecision::Finish))
                    .unwrap_or(StopDecision::Finish);
                let nudge = match decision {
                    StopDecision::Finish => {
                        if cut_off {
                            on_event(Event::Truncated);
                        }
                        return Ok(RunSummary { run_id: run_id.to_string(), text, turns: index + 1, usage, finish });
                    }
                    StopDecision::Pause(reason) => return Err(HarnessError::Paused(reason)),
                    StopDecision::Continue(_) if self.max_turns == Some(index + 1) => return Err(HarnessError::MaxTurns(index + 1)),
                    StopDecision::Continue(nudge) => nudge,
                };
                let reminder = Message::reminder(nudge.message);
                self.store.append(session, std::slice::from_ref(&reminder))?;
                history.push(reminder);
                on_event(Event::Continuing { reason: nudge.reason });
                continuations += 1;
                continuations_without_progress += 1;
                continue;
            }

            let over_own = own_limit.is_some_and(|limit| tool_calls.saturating_add(calls.len()) > limit);
            let over_budget = !budget.as_ref().map_or(true, |budget| budget.take(calls.len())) || over_own;
            tool_calls = tool_calls.saturating_add(calls.len());
            let results = if over_budget {
                self.reject_calls(session, &calls, "Not run: this batch exceeds the run tool-call budget. No calls in the batch were executed.", on_event)
            } else if cut_calls {
                self.reject_calls(session, &calls, CUT_OFF_MESSAGE, on_event)
            } else {
                self.execute_calls(session, run_id, &calls, cancel, budget.as_ref(), &control, on_event)
            };
            let tool_messages: Vec<Message> =
                calls.iter().zip(&results).map(|(c, r)| r.message(c.id.as_str())).collect();
            self.store.append(session, &tool_messages)?;
            history.extend(tool_messages);
            if results.iter().any(crate::ToolResult::halts_dispatch){let unknown=results.iter().any(|r|r.structured_content.as_ref().is_some_and(|v|v["harness_status"]=="outcome_unknown"));return Err(HarnessError::Paused(if unknown{"tool_outcome_needs_review"}else{"resources_busy"}.into()));}
            if over_budget { return Err(HarnessError::BudgetExceeded("tool calls".into())); }
            if cut_calls {
                if cancel.load(Ordering::Relaxed) { return Err(HarnessError::Cancelled); }
                if truncated_tool_retries >= self.max_truncated_tool_retries {
                    return Err(HarnessError::Paused("output_length_limit".into()));
                }
                if self.max_turns == Some(index + 1) { return Err(HarnessError::MaxTurns(index + 1)); }
                truncated_tool_retries += 1;
                on_event(Event::Continuing { reason: "tool_call_length".into() });
                continue;
            }
            let previous_successes=successful_calls.len();
            for (call, result) in calls.iter().zip(&results) {
                if result.is_error || matches!(call.name.as_str(), "todo" | "ask_user" | "time_now") {
                    continue;
                }
                let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.arguments) else { continue };
                use std::hash::{Hash,Hasher};
                let mut hash=std::collections::hash_map::DefaultHasher::new();
                result.output.hash(&mut hash);
                if let Some(v)=&result.structured_content{v.to_string().hash(&mut hash);}
                let key = (call.name.clone(), args,hash.finish());
                if successful_calls.insert(key) {
                    continuations_without_progress = 0;
                    truncated_tool_retries = 0;
                }
            }
            // Waiting for a live owned process is not failed work. Require a real
            // bounded wait so zero-delay polling cannot evade the idle guard.
            let waiting = !calls.is_empty() && calls.iter().zip(&results).all(|(call, result)| {
                call.name == "process_read" && !result.is_error
                    && serde_json::from_str::<serde_json::Value>(&call.arguments).ok()
                        .and_then(|args| args["wait_ms"].as_u64()).is_some_and(|ms| ms >= 1000)
                    && result.structured_content.as_ref().is_some_and(|v| v["state"] == "running")
            });
            idle_rounds=if previous_successes!=successful_calls.len(){0}else if waiting{idle_rounds}else{idle_rounds+1};
            if options.max_no_progress_rounds.is_some_and(|limit|idle_rounds>=limit){return Err(HarnessError::Paused("no_progress_tool_rounds".into()));}
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(HarnessError::Cancelled);
        }
        Err(HarnessError::MaxTurns(self.max_turns.unwrap_or(usize::MAX)))
    }

    /// Retries only while no answer text has been streamed: reasoning shown so far is dropped by the page on `Retrying`,
    /// but a half answer is never replayed.
    fn chat_with_retry(
        &self,
        session: &str,
        run_id: &str,
        request: &mut ChatRequest,
        recovered_budget: &mut Option<usize>,
        on_event: &mut dyn FnMut(Event),
        cancel: &AtomicBool,
    ) -> Result<Reply, HarnessError> {
        let mut attempt = 0;
        let mut recoveries = 0;
        let reduce = |request: &mut ChatRequest, budget| -> Result<(), HarnessError> {
            let mut turn = Turn { session: String::new(), index: 0, messages: request.messages.clone(), tools: request.tools.clone() };
            ContextTrim::new(budget).keep_recent(2).before_model(&mut turn)?;
            request.messages = turn.messages;
            Ok(())
        };
        if let Some(budget) = *recovered_budget { reduce(request, budget)?; }
        loop {
            let streamed_flag = std::cell::Cell::new(false);
            let text_flag = std::cell::Cell::new(false);
            let mut visible_reasoning: Vec<crate::ReasoningSegment> = Vec::new();
            let mut visible_text = String::new();
            // Both provider callbacks share one observer without aliasing two &mut borrows.
            let result = {
                let sink = std::cell::RefCell::new(&mut *on_event);
                self.provider.chat_with_reasoning(
                    request,
                    &mut |text| {
                        if text.is_empty() { return; }
                        visible_text.push_str(text);
                        streamed_flag.set(true);
                        text_flag.set(true);
                        (*sink.borrow_mut())(Event::TextDelta { text: text.to_string() });
                    },
                    &mut |kind, text| {
                        if text.is_empty() { return; }
                        streamed_flag.set(true); // Persisted as interrupted if the call fails for good; a retry before any text discards it.
                        if let Some(last) = visible_reasoning.last_mut().filter(|part| part.kind == kind) {
                            last.text.push_str(text);
                        } else {
                            visible_reasoning.push(crate::ReasoningSegment { kind, text: text.to_owned() });
                        }
                        (*sink.borrow_mut())(Event::ReasoningDelta { kind, text: text.to_owned() });
                    },
                    cancel,
                )
            };
            let streamed = streamed_flag.get();
            match result {
                Err(error) if !streamed && error.is_context_limit() && recoveries < 2 => {
                    let used = estimate_tokens(&request.messages) + estimate_tool_tokens(&request.tools);
                    let budget = used.saturating_mul(3) / 4;
                    let before = request.messages.clone();
                    if reduce(request, budget).is_err() || request.messages == before {
                        return Err(HarnessError::provider("context_length_exceeded: 当前输入和必要工具定义仍超出模型窗口，请调低上下文上限、减少本次输入或手动压缩会话。", false));
                    }
                    *recovered_budget = Some(budget);
                    recoveries += 1;
                    on_event(Event::Retrying { attempt: recoveries, delay_ms: 0, message: "模型上下文超限，已缩减旧工具结果和历史，正在重试；已执行的操作不会重放。".into() });
                }
                Err(error) if error.is_context_limit() && !streamed => {
                    return Err(HarnessError::provider("context_length_exceeded: 缩减旧内容后仍超过模型窗口，请调低上下文上限或手动压缩会话后继续。", false));
                }
                Err(HarnessError::Provider { message, retryable: true })
                    // Reasoning shown so far is thrown away with the failed attempt (the page drops it on `Retrying`), so a stall
                    // before any answer text is as safe to retry as a failure before anything arrived. Text, and with it any
                    // call the model had started, is never replayed.
                    if !text_flag.get() && attempt < self.retry.max_retries =>
                {
                    attempt += 1;
                    let delay = backoff(&self.retry, attempt);
                    on_event(Event::Retrying { attempt, delay_ms: delay.as_millis() as u64, message });
                    sleep_unless_cancelled(delay, cancel)?;
                }
                Ok(mut reply) => {
                    if !visible_reasoning.is_empty() { reply.message.reasoning = visible_reasoning; }
                    return Ok(reply);
                }
                Err(error) => {
                    // A failed decoder has no safe, complete tool calls or opaque continuation
                    // state. Persist only what was already exposed to the user. This also
                    // prevents retry_last from silently replaying a partially answered turn.
                    if streamed {
                        let mut partial = Message::assistant(visible_text, Vec::new());
                        partial.reasoning = visible_reasoning;
                        partial.structured_content = Some(serde_json::json!({"interrupted": true, "recovery_run_id": run_id}));
                        self.store.append(session, &[partial])?;
                    }
                    return Err(error);
                }
            }
        }
    }
}

#[cfg(test)]
mod interrupted_output_tests {
    use super::*;
    use crate::store::SessionStore;
    use crate::provider::ModelProvider;
    use std::sync::atomic::AtomicUsize;

    struct InterruptedProvider {
        cancelled: bool,
        attempts: AtomicUsize,
    }
    impl ModelProvider for InterruptedProvider {
        fn chat(&self, request: &ChatRequest, on_text: &mut dyn FnMut(&str), cancel: &AtomicBool) -> Result<Reply, HarnessError> {
            self.chat_with_reasoning(request, on_text, &mut |_, _| {}, cancel)
        }
        fn chat_with_reasoning(&self, _: &ChatRequest, on_text: &mut dyn FnMut(&str), on_reasoning: &mut dyn FnMut(crate::ReasoningKind, &str), _: &AtomicBool) -> Result<Reply, HarnessError> {
            self.attempts.fetch_add(1, Ordering::SeqCst);
            on_reasoning(crate::ReasoningKind::Text, "checking ");
            on_reasoning(crate::ReasoningKind::Text, "state");
            on_text("partial answer");
            Err(if self.cancelled { HarnessError::Cancelled } else { HarnessError::provider("connection reset", true) })
        }
    }

    #[test]
    fn cancelled_and_failed_output_survive_reload_without_automatic_replay() {
        for cancelled in [false, true] {
            let dir = std::env::temp_dir().join(format!("agent-interrupted-{}-{}-{}", std::process::id(), cancelled,
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            let store = Arc::new(crate::JsonlStore::new(&dir));
            let provider = Arc::new(InterruptedProvider { cancelled, attempts: AtomicUsize::new(0) });
            let agent = Agent::builder(provider.clone(), store.clone()).build().unwrap();
            let cancel = Arc::new(AtomicBool::new(false));
            assert!(agent.run("s", "question", &mut |_| {}, &cancel).is_err());
            let saved = crate::JsonlStore::new(&dir).load("s").unwrap();
            assert_eq!(saved.len(), 2);
            assert_eq!(saved[1].text(), "partial answer");
            assert_eq!(saved[1].reasoning[0].text, "checking state");
            assert_eq!(saved[1].structured_content.as_ref().unwrap()["interrupted"], true);
            assert!(saved[1].tool_calls.is_empty());
            assert!(saved[1].provider_state.is_none());
            assert!(agent.retry_last("s", &mut |_| {}, &cancel).is_err());
            assert_eq!(provider.attempts.load(Ordering::SeqCst), 1);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }
}

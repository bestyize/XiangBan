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
        &self, session: &str, input: Option<Vec<Part>>, options: crate::RunOptions,
        observe: &mut dyn FnMut(crate::RunEvent) -> Result<(), HarnessError>, cancel: &Arc<AtomicBool>,
    ) -> Result<RunSummary, HarnessError> {
        use crate::{RunEnd, RunEventBody, RunState};
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
        let mut successful_calls = Vec::new();
        let mut truncated_tool_retries = 0usize;
        let mut tool_calls = 0usize;
        let mut recovered_context_budget = None;
        for index in 0..self.max_turns {
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
            let mut reply = self.chat_with_retry(&mut request, &mut recovered_context_budget, on_event, cancel)?;
            let turn = Turn { session: turn.session, index, messages: request.messages, tools: request.tools };
            for mw in &self.middleware {
                mw.after_model(&turn, &mut reply.message)?;
            }
            if let Some(u) = reply.usage {
                usage += u;
                on_event(Event::Usage { prompt_tokens: u.prompt_tokens, completion_tokens: u.completion_tokens });
            }

            let finish = reply.finish;
            let mut assistant = reply.message;
            let cut_off = finish == FinishReason::Length;
            let cut_calls = cut_off && !assistant.tool_calls.is_empty();
            if cut_calls {
                // Half-written arguments would be sent back to the model as history; keep them parseable.
                for call in &mut assistant.tool_calls {
                    if !serde_json::from_str::<serde_json::Value>(&call.arguments).is_ok_and(|v| v.is_object()) {
                        call.arguments = "{}".to_string();
                    }
                }
            }
            self.store.append(session, std::slice::from_ref(&assistant))?;
            let (calls, text) = (assistant.tool_calls.clone(), assistant.text());
            history.push(assistant);
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
                    StopDecision::Continue(_) if index + 1 == self.max_turns => return Err(HarnessError::MaxTurns(self.max_turns)),
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

            let over_budget = options.max_tool_calls.is_some_and(|limit| tool_calls.saturating_add(calls.len()) > limit);
            tool_calls = tool_calls.saturating_add(calls.len());
            let results = if over_budget {
                self.reject_calls(session, &calls, "Not run: this batch exceeds the run tool-call budget. No calls in the batch were executed.", on_event)
            } else if cut_calls {
                self.reject_calls(session, &calls, CUT_OFF_MESSAGE, on_event)
            } else {
                self.execute_calls(session, run_id, &calls, cancel, on_event)
            };
            let tool_messages: Vec<Message> =
                calls.iter().zip(&results).map(|(c, r)| r.message(c.id.as_str())).collect();
            self.store.append(session, &tool_messages)?;
            history.extend(tool_messages);
            if over_budget { return Err(HarnessError::BudgetExceeded("tool calls".into())); }
            if cut_calls {
                if cancel.load(Ordering::Relaxed) { return Err(HarnessError::Cancelled); }
                if truncated_tool_retries >= self.max_truncated_tool_retries {
                    return Err(HarnessError::Paused("output_length_limit".into()));
                }
                if index + 1 == self.max_turns { return Err(HarnessError::MaxTurns(self.max_turns)); }
                truncated_tool_retries += 1;
                on_event(Event::Continuing { reason: "tool_call_length".into() });
                continue;
            }
            for (call, result) in calls.iter().zip(&results) {
                if result.is_error || matches!(call.name.as_str(), "todo" | "ask_user" | "time_now") {
                    continue;
                }
                let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.arguments) else { continue };
                let key = (call.name.clone(), args);
                if !successful_calls.contains(&key) {
                    successful_calls.push(key);
                    continuations_without_progress = 0;
                    truncated_tool_retries = 0;
                }
            }
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(HarnessError::Cancelled);
        }
        Err(HarnessError::MaxTurns(self.max_turns))
    }

    /// Retries only when nothing was streamed yet, so the UI never sees a half answer replayed.
    fn chat_with_retry(
        &self,
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
            let mut visible_reasoning: Vec<crate::ReasoningSegment> = Vec::new();
            // Both provider callbacks share one observer without aliasing two &mut borrows.
            let result = {
                let sink = std::cell::RefCell::new(&mut *on_event);
                self.provider.chat_with_reasoning(
                    request,
                    &mut |text| {
                        streamed_flag.set(true);
                        (sink.borrow_mut())(Event::TextDelta { text: text.to_string() });
                    },
                    &mut |kind, text| {
                        if text.is_empty() { return; }
                        streamed_flag.set(true); // Do not replay a partially displayed reasoning stream.
                        if let Some(last) = visible_reasoning.last_mut().filter(|part| part.kind == kind) {
                            last.text.push_str(text);
                        } else {
                            visible_reasoning.push(crate::ReasoningSegment { kind, text: text.to_owned() });
                        }
                        (sink.borrow_mut())(Event::ReasoningDelta { kind, text: text.to_owned() });
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
                    if !streamed && attempt < self.retry.max_retries =>
                {
                    attempt += 1;
                    let delay = backoff(&self.retry, attempt);
                    on_event(Event::Retrying { attempt, delay_ms: delay.as_millis() as u64, message });
                    sleep_unless_cancelled(delay, cancel)?;
                }
                Ok(mut reply) => {
                    reply.message.reasoning = visible_reasoning;
                    return Ok(reply);
                }
                other => return other,
            }
        }
    }
}

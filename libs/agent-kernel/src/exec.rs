use crate::agent::Agent;
use crate::tool::{Tool, ToolCtx, ToolGate, ToolResult, DEFAULT_MAX_OUTPUT};
use crate::types::{Event, ToolCall};
use crate::util::truncate_middle;
use serde_json::{Map, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(20);

struct Inflight {
    index: usize,
    rx: Receiver<(Instant, ToolResult)>,
    clock: crate::RunControl,
    scope: crate::RunControl,

    call_cancel: Arc<AtomicBool>,
    grace: Duration,
    _watch: crate::StopWatch,
}

enum Prepared {
    Done(ToolResult),
    Run { tool: Arc<dyn Tool>, args: Value, ctx: ToolCtx },
}

/// Keep dispatcher events/hooks stable while execution and resource ownership use the pinned target.
struct ResolvedTool {
    spec: crate::ToolSpec,
    target: Arc<dyn Tool>,
    args: Value,
}
impl Tool for ResolvedTool {
    fn spec(&self) -> &crate::ToolSpec { &self.spec }
    fn resource(&self, _: &Value) -> Result<Option<crate::ToolResource>, String> { self.target.resource(&self.args) }
    fn cancel_grace(&self) -> Duration { self.target.cancel_grace() }
    fn execute_result(&self, _: Value, ctx: &ToolCtx) -> ToolResult { self.target.execute_result(self.args.clone(), ctx) }
}

fn spawn(index: usize, tool: Arc<dyn Tool>, args: Value, mut ctx: ToolCtx) -> Result<Inflight, ToolResult> {
    let timeout = tool.spec().timeout;
    let call_cancel = ctx.call_cancel.clone();
    let grace = tool.cancel_grace();
    let clock = crate::RunControl::tool_timeout(timeout);
    ctx.call_control = clock.clone();
    let watched_clock = clock.clone();
    let run_cancel = ctx.run_cancel.clone();
    let worker_cancel = call_cancel.clone();
    let scope = ctx.control.clone();
    // Each worker owns a watchdog; waiting for another result or an approval
    // must not postpone this call's cancellation.
    let watch = crate::StopWatch::new(
        move || run_cancel.load(Ordering::Relaxed) || watched_clock.time_expired_at(Instant::now()),
        move || worker_cancel.store(true, Ordering::SeqCst),
    );
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name(format!("tool-{}", tool.spec().name))
        .spawn(move || {
            let result = tool.execute_result(args, &ctx);
            if result.outcome_unknown() { ctx.control.mark_uncertain(); }
            drop(ctx);
            let _ = tx.send((Instant::now(), result));
        })
        .map_err(|e| ToolResult::error(format!("could not start tool: {e}")))?;
    Ok(Inflight { index, rx, clock, scope, call_cancel, grace, _watch: watch })
}

fn reported(call: &Inflight, finished: Instant, mut result: ToolResult) -> ToolResult {
    if call.clock.time_expired_at(finished) {
        result.is_error = true;
        result.output = format!("Tool timed out; final worker report (action may have completed; inspect state before retrying): {}", result.output);
    }
    result
}

fn await_call(call: &Inflight, cancel: &AtomicBool) -> ToolResult {
    loop {
        match call.rx.recv_timeout(POLL) {
            Ok((finished, result)) => return reported(call, finished, result),
            Err(RecvTimeoutError::Disconnected) => return ToolResult::unknown("worker crashed without a result"),
            Err(RecvTimeoutError::Timeout) => {
                if cancel.load(Ordering::Relaxed) || call.clock.time_expired_at(Instant::now()) {
                    call.call_cancel.store(true, Ordering::SeqCst);
                    return match call.rx.recv_timeout(call.grace) {
                        Ok((finished, result)) => reported(call, finished, result),
                        Err(_) => ToolResult::unknown("worker did not acknowledge cancellation/deadline"),
                    };
                }
            }
        }
    }
}

impl Agent {
    /// Results come back in call order. Parallel-safe calls overlap; a serial call runs alone.
    pub(crate) fn execute_calls(
        &self,
        session: &str,
        run_id: &str,
        calls: &[ToolCall],
        cancel: &Arc<AtomicBool>,
        budget: Option<&Arc<crate::ToolBudget>>,
        control: &crate::RunControl,
        on_event: &mut dyn FnMut(Event),
    ) -> Vec<ToolResult> {
        let mut results: Vec<Option<ToolResult>> = vec![None; calls.len()];
        let mut inflight = Vec::new();
        for (i, call) in calls.iter().enumerate() {
            on_event(Event::ToolStart { id: call.id.clone(), name: call.name.clone(), arguments: call.arguments.clone() });
            if results.iter().flatten().any(ToolResult::halts_dispatch){results[i]=Some(self.finalize(session,call,ToolResult::error("Not dispatched: earlier action needs review"),on_event));continue;}
            match self.prepare(session, run_id, call, cancel, budget, control) {
                Prepared::Done(result) => results[i] = Some(self.finalize(session, call, result, on_event)),
                Prepared::Run { tool, args, mut ctx } => {
                    let serial = !tool.spec().parallel_safe;
                    if serial {
                        self.drain(session, &mut inflight, &mut results, calls, cancel, on_event);
                    }
                    if results.iter().flatten().any(ToolResult::halts_dispatch){results[i]=Some(self.finalize(session,call,ToolResult::error("Not dispatched: earlier action needs review"),on_event));continue;}
                    let resource = match tool.resource(&args) {
                        Ok(resource) => resource,
                        Err(error) => {
                            results[i] = Some(self.finalize(session, call, ToolResult::error(error), on_event));
                            continue;
                        }
                    };
                    if let Some(resource) = resource {
                        // Permissions have already been resolved. File/sequence ownership lasts
                        // through the run; ordinary mutation ownership belongs to this worker.
                        if resource.held_for_run() { ctx.control = control.clone(); }
                        let claimed = ctx.control.claim_tool_resource(&resource, session, cancel, |waiting, owner| {
                            on_event(Event::ToolWaiting { id: call.id.clone(), resource: resource.label(), owner_session: owner.map(String::from), waiting });
                        });
                        if let Err(result) = claimed {
                            results[i] = Some(self.finalize(session, call, result, on_event));
                            continue;
                        }
                    }
                    if cancel.load(Ordering::Relaxed) {
                        results[i] = Some(self.finalize(session, call, ToolResult::error("cancelled before dispatch"), on_event));
                        continue;
                    }
                    let blocked = self.middleware.iter().filter(|mw| !mw.authorizes_tool()).find_map(|mw| {
                        match mw.before_tool(&ctx, call, &args, tool.spec()) {
                            ToolGate::Proceed => None,
                            ToolGate::Block(reason) => Some(ToolResult::error(reason)),
                        }
                    });
                    if let Some(result) = blocked {
                        results[i] = Some(self.finalize(session, call, result, on_event));
                        continue;
                    }
                    on_event(Event::ToolDispatched { id: call.id.clone() });
                    if cancel.load(Ordering::Relaxed) {
                        results[i] = Some(self.finalize(session, call, ToolResult::error("cancelled before dispatch"), on_event));
                        continue;
                    }
                    match spawn(i, tool, args, ctx) {
                        Ok(call_in_flight) => inflight.push(call_in_flight),
                        Err(result) => results[i] = Some(self.finalize(session, call, result, on_event)),
                    }
                    if serial {
                        self.drain(session, &mut inflight, &mut results, calls, cancel, on_event);
                    }
                }
            }
        }
        self.drain(session, &mut inflight, &mut results, calls, cancel, on_event);
        results.into_iter().map(|r| r.unwrap_or_else(|| ToolResult::error("no result"))).collect()
    }

    /// Answers every call with the same error without running anything, still reporting the calls
    /// to the UI and the middleware as ordinary failed calls.
    pub(crate) fn reject_calls(&self, session: &str, calls: &[ToolCall], message: &str, on_event: &mut dyn FnMut(Event)) -> Vec<ToolResult> {
        calls
            .iter()
            .map(|call| {
                on_event(Event::ToolStart { id: call.id.clone(), name: call.name.clone(), arguments: call.arguments.clone() });
                self.finalize(session, call, ToolResult::error(message), on_event)
            })
            .collect()
    }

    fn prepare(&self, session: &str, run_id: &str, call: &ToolCall, cancel: &Arc<AtomicBool>, budget: Option<&Arc<crate::ToolBudget>>,control:&crate::RunControl) -> Prepared {
        let Some(&index) = self.tool_index.get(&call.name) else {
            let mut names: Vec<&str> = self.tools.iter().map(|tool| tool.spec().name.as_str()).collect();
            names.sort_unstable();
            let available = if names.is_empty() { "(none)".to_string() } else { names.join(", ") };
            return Prepared::Done(ToolResult::error(format!(
                "unknown tool `{}`. No tool was executed. Available tools: {}. \
                 Use an exact registered name and its argument schema from the current tool definitions; \
                 do not invent names or reuse names from another client. If no suitable tool is available, explain that limitation.",
                call.name, available,
            )));
        };
        if cancel.load(Ordering::Relaxed) {
            return Prepared::Done(ToolResult::error("cancelled"));
        }
        if let Err(error) = control.check() { return Prepared::Done(ToolResult::error(error.to_string())); }
        let args = if call.arguments.trim().is_empty() {
            Value::Object(Map::new())
        } else {
            match serde_json::from_str(&call.arguments) {
                Ok(args) => args,
                Err(e) if e.is_eof() => return Prepared::Done(ToolResult::error(format!(
                    "arguments are not valid JSON: {e}. They end mid-way, so the reply was probably cut off by the output length limit.                      Send smaller calls (write files separately, use targeted edits) instead of repeating the same large call."))),
                Err(e) => return Prepared::Done(ToolResult::error(format!("arguments are not valid JSON: {e}"))),
            }
        };
        let mut tool = self.tools[index].clone();
        if !args.is_object() {
            return Prepared::Done(ToolResult::error("arguments must be a JSON object"));
        }
        let errors: Vec<_> = self.validators[index].iter_errors(&args).take(16)
            .map(|error| format!("at ${}: {}", error.instance_path, error)).collect();
        if !errors.is_empty() {
            return Prepared::Done(ToolResult::error(format!(
                "invalid arguments for `{}`: {}. No tool was executed. Correct all listed errors using the tool schema before calling again. Required properties: {}.",
                call.name, errors.join("; "), tool.spec().schema.get("required").unwrap_or(&Value::Null)
            )).with_structured_content(serde_json::json!({"harness_status":"invalid_arguments","executed":false})));
        }
        let mut authorization = None;
        match tool.resolve_call(&args) {
            Ok(Some((target, target_args))) => {
                let validator = match jsonschema::options().with_retriever(crate::agent::NoExternalSchemas).build(&target.spec().schema) {
                    Ok(validator) => validator,
                    Err(error) => return Prepared::Done(ToolResult::error(format!("invalid target schema: {error}"))),
                };
                if !target_args.is_object() || !validator.is_valid(&target_args) {
                    return Prepared::Done(ToolResult::error("invalid arguments for resolved tool; no tool was executed")
                        .with_structured_content(serde_json::json!({"harness_status":"invalid_arguments","executed":false})));
                }
                authorization = Some((target.spec().clone(), target_args.clone()));
                let mut spec = tool.spec().clone();
                spec.timeout = target.spec().timeout;
                spec.max_output = target.spec().max_output;
                tool = Arc::new(ResolvedTool { spec, target, args: target_args });
            }
            Ok(None) => {}
            Err(error) => return Prepared::Done(ToolResult::error(error)),
        }
        let ctx = ToolCtx {
            session: session.to_string(),
            run_id: run_id.to_string(),
            run_cancel: cancel.clone(),
            call_cancel: Arc::new(AtomicBool::new(false)),
            budget: budget.cloned(),
            control: control.tool_scope(),
            call_control: Default::default(),
        };
        for mw in self.middleware.iter().filter(|mw| mw.authorizes_tool()) {
            let (spec, permission_args) = authorization.as_ref().map(|(spec,args)| (spec,args)).unwrap_or((tool.spec(), &args));
            if let ToolGate::Block(reason) = mw.before_tool(&ctx, call, permission_args, spec) {
                let denied = reason.starts_with("The user denied permission to run `");
                let result = ToolResult::error(reason);
                return Prepared::Done(if denied { result.with_structured_content(serde_json::json!({"harness_status":"permission_denied"})) } else { result });
            }
        }
        if cancel.load(Ordering::Relaxed) {
            return Prepared::Done(ToolResult::error("cancelled"));
        }
        Prepared::Run { tool, args, ctx }
    }

    fn drain(
        &self,
        session: &str,
        inflight: &mut Vec<Inflight>,
        results: &mut [Option<ToolResult>],
        calls: &[ToolCall],
        cancel: &AtomicBool,
        on_event: &mut dyn FnMut(Event),
    ) {
        for call in inflight.drain(..) {
            let result = await_call(&call, cancel);
            if result.outcome_unknown() { call.scope.mark_uncertain(); }
            results[call.index] = Some(self.finalize(session, &calls[call.index], result, on_event));
        }
    }

    fn finalize(
        &self,
        session: &str,
        call: &ToolCall,
        mut result: ToolResult,
        on_event: &mut dyn FnMut(Event),
    ) -> ToolResult {
        // Do not truncate binary data or JSON into unusable fragments. Keep media below the
        // JSONL bridge's 8 MiB frame budget, with space for text and envelope fields.
        let media_bytes: usize = result.parts.iter().map(|p| match p {
            crate::Part::Text { text } => text.len(),
            crate::Part::Image { url } => url.len(),
        }).sum();
        let max = self.tool_index.get(&call.name).map_or(DEFAULT_MAX_OUTPUT, |&i| self.tools[i].spec().max_output);
        if media_bytes > 6 * 1024 * 1024 {
            result = ToolResult::error("Tool returned too much media (limit: 6 MiB); request a smaller result. The tool may already have performed its action.");
        } else if result.structured_content.as_ref().is_some_and(|v| v.to_string().len() > max) {
            result = ToolResult::error("Tool returned structured data larger than its output limit; request a smaller result. The tool may already have performed its action.");
        }
        let mut images = Vec::new();
        for part in std::mem::take(&mut result.parts) {
            match part {
                crate::Part::Text { text } => { result.output.push('\n'); result.output.push_str(&text); }
                image => images.push(image),
            }
        }
        result.parts = images;
        for mw in &self.middleware {
            mw.after_tool(session, call, &mut result);
        }
        result.output = truncate_middle(&result.output, max);
        on_event(Event::ToolEnd {
            id: call.id.clone(), output: result.output.clone(), is_error: result.is_error,
            parts: result.parts.clone(), structured_content: result.structured_content.clone(),
        });
        result
    }
}


#[cfg(test)]
mod user_wait_regressions {
    use super::*;
    use crate::{Risk, ToolSpec, RunControl};
    struct WaitingTool { spec: ToolSpec }
    impl Tool for WaitingTool {
        fn spec(&self) -> &ToolSpec { &self.spec }
        fn execute_result(&self, _: Value, ctx: &ToolCtx) -> ToolResult {
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(250)); let _ = tx.send("approved"); });
            match ctx.wait_for_user(&rx) { Some(_) => ToolResult::ok("continued"), None => ToolResult::error("cancelled") }
        }
    }
    fn human() -> Arc<dyn Tool> {
        Arc::new(WaitingTool { spec: ToolSpec::new("human", "wait", serde_json::json!({"type":"object"})).risk(Risk::Read).timeout(Duration::from_millis(100)) })
    }
    #[test]
    fn human_input_preserves_both_call_and_run_active_time() {
        let mut ctx = ToolCtx::detached("s"); ctx.control = RunControl::bounded(Duration::from_millis(150), 1000);
        let budget = ctx.control.clone(); let call = spawn(0, human(), Value::Null, ctx).unwrap();
        let result = await_call(&call, &AtomicBool::new(false));
        assert!(!result.is_error, "{}", result.output);
        assert!(budget.check().is_ok()); assert!(budget.excluded_wait() >= Duration::from_millis(200));
    }
    #[test]
    fn waiting_for_a_person_does_not_pause_a_different_calls_timeout() {
        struct Slow(ToolSpec);
        impl Tool for Slow {
            fn spec(&self) -> &ToolSpec { &self.0 }
            fn execute_result(&self, _: Value, ctx: &ToolCtx) -> ToolResult {
                while !ctx.is_cancelled() { std::thread::sleep(Duration::from_millis(5)); }
                ToolResult::error("stopped")
            }
        }
        let ctx = ToolCtx::detached("s"); let mut other = ToolCtx::detached("s");
        other.control = ctx.control.clone(); other.run_cancel = ctx.run_cancel.clone();
        let person = spawn(0, human(), Value::Null, ctx).unwrap();
        let slow = Arc::new(Slow(ToolSpec::new("slow", "slow", serde_json::json!({"type":"object"})).timeout(Duration::from_millis(70))));
        let began = Instant::now(); let slow = spawn(1, slow, Value::Null, other).unwrap();
        assert!(await_call(&slow, &AtomicBool::new(false)).is_error);
        assert!(began.elapsed() < Duration::from_millis(200));
        assert!(!await_call(&person, &AtomicBool::new(false)).is_error);
    }
    #[test]
    fn cancellation_interrupts_a_human_wait_even_while_its_clocks_are_paused() {
        let ctx = ToolCtx::detached("s"); let cancel = ctx.run_cancel.clone();
        let call = spawn(0, human(), Value::Null, ctx).unwrap();
        cancel.store(true, Ordering::SeqCst);
        assert!(await_call(&call, &cancel).is_error);
    }
}

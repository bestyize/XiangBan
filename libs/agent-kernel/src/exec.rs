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
    rx: Receiver<Result<String, String>>,
    deadline: Option<Instant>,
    timeout: Duration,
    call_cancel: Arc<AtomicBool>,
}

enum Prepared {
    Done(ToolResult),
    Run { tool: Arc<dyn Tool>, args: Value, ctx: ToolCtx },
}

fn spawn(index: usize, tool: Arc<dyn Tool>, args: Value, ctx: ToolCtx) -> Result<Inflight, ToolResult> {
    let timeout = tool.spec().timeout;
    let call_cancel = ctx.call_cancel.clone();
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name(format!("tool-{}", tool.spec().name))
        .spawn(move || {
            let _ = tx.send(tool.execute(args, &ctx));
        })
        .map_err(|e| ToolResult::error(format!("could not start tool: {e}")))?;
    Ok(Inflight { index, rx, deadline: Instant::now().checked_add(timeout), timeout, call_cancel })
}

fn await_call(call: &Inflight, cancel: &AtomicBool) -> ToolResult {
    loop {
        match call.rx.recv_timeout(POLL) {
            Ok(Ok(output)) => return ToolResult::ok(output),
            Ok(Err(error)) => return ToolResult::error(error),
            Err(RecvTimeoutError::Disconnected) => return ToolResult::error("tool crashed before returning a result"),
            Err(RecvTimeoutError::Timeout) => {
                if cancel.load(Ordering::Relaxed) {
                    call.call_cancel.store(true, Ordering::Relaxed);
                    return ToolResult::error("cancelled");
                }
                if call.deadline.is_some_and(|d| Instant::now() >= d) {
                    call.call_cancel.store(true, Ordering::Relaxed);
                    return ToolResult::error(format!("tool timed out after {}s", call.timeout.as_secs()));
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
        calls: &[ToolCall],
        cancel: &Arc<AtomicBool>,
        on_event: &mut dyn FnMut(Event),
    ) -> Vec<ToolResult> {
        let mut results: Vec<Option<ToolResult>> = vec![None; calls.len()];
        let mut inflight = Vec::new();
        for (i, call) in calls.iter().enumerate() {
            on_event(Event::ToolStart { id: call.id.clone(), name: call.name.clone(), arguments: call.arguments.clone() });
            match self.prepare(session, call, cancel) {
                Prepared::Done(result) => results[i] = Some(self.finalize(session, call, result, on_event)),
                Prepared::Run { tool, args, ctx } => {
                    let serial = !tool.spec().parallel_safe;
                    if serial {
                        self.drain(session, &mut inflight, &mut results, calls, cancel, on_event);
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

    fn prepare(&self, session: &str, call: &ToolCall, cancel: &Arc<AtomicBool>) -> Prepared {
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
        let args = if call.arguments.trim().is_empty() {
            Value::Object(Map::new())
        } else {
            match serde_json::from_str(&call.arguments) {
                Ok(args) => args,
                Err(e) => return Prepared::Done(ToolResult::error(format!("arguments are not valid JSON: {e}"))),
            }
        };
        let tool = self.tools[index].clone();
        let ctx = ToolCtx {
            session: session.to_string(),
            run_cancel: cancel.clone(),
            call_cancel: Arc::new(AtomicBool::new(false)),
        };
        for mw in &self.middleware {
            if let ToolGate::Block(reason) = mw.before_tool(&ctx, call, &args, tool.spec()) {
                return Prepared::Done(ToolResult::error(reason));
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
        for mw in &self.middleware {
            mw.after_tool(session, call, &mut result);
        }
        let max = self.tool_index.get(&call.name).map_or(DEFAULT_MAX_OUTPUT, |&i| self.tools[i].spec().max_output);
        result.output = truncate_middle(&result.output, max);
        on_event(Event::ToolEnd { id: call.id.clone(), output: result.output.clone(), is_error: result.is_error });
        result
    }
}


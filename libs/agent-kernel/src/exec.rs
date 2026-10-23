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
    rx: Receiver<ToolResult>,
    deadline: Option<Instant>,

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
            let _ = tx.send(tool.execute_result(args, &ctx));
        })
        .map_err(|e| ToolResult::error(format!("could not start tool: {e}")))?;
    Ok(Inflight { index, rx, deadline: Instant::now().checked_add(timeout), call_cancel })
}

fn await_call(call:&Inflight,cancel:&AtomicBool)->ToolResult{
 loop{match call.rx.recv_timeout(POLL){Ok(result)=>return result,Err(RecvTimeoutError::Disconnected)=>return ToolResult::unknown("worker crashed without a result"),Err(RecvTimeoutError::Timeout)=>{
  if cancel.load(Ordering::Relaxed)||call.deadline.is_some_and(|d|Instant::now()>=d){call.call_cancel.store(true,Ordering::SeqCst);let timed_out=!cancel.load(Ordering::Relaxed);
                    return match call.rx.recv_timeout(Duration::from_millis(250)){Ok(mut result)=>{if timed_out {result.output=format!("Tool timed out; final worker report (action may have completed): {}",result.output);}result},Err(_)=>ToolResult::unknown("worker did not acknowledge cancellation/deadline")};}
 }}}
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
                Prepared::Run { tool, args, ctx } => {
                    let serial = !tool.spec().parallel_safe;
                    if serial {
                        self.drain(session, &mut inflight, &mut results, calls, cancel, on_event);
                    }
                    if results.iter().flatten().any(ToolResult::halts_dispatch){results[i]=Some(self.finalize(session,call,ToolResult::error("Not dispatched: earlier action needs review"),on_event));continue;}
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
        let args = if call.arguments.trim().is_empty() {
            Value::Object(Map::new())
        } else {
            match serde_json::from_str(&call.arguments) {
                Ok(args) => args,
                Err(e) => return Prepared::Done(ToolResult::error(format!("arguments are not valid JSON: {e}"))),
            }
        };
        let tool = self.tools[index].clone();
        if !args.is_object() {
            return Prepared::Done(ToolResult::error("arguments must be a JSON object"));
        }
        if let Err(error) = self.validators[index].validate(&args) {
            return Prepared::Done(ToolResult::error(format!(
                "invalid arguments for `{}` at ${}: {}. No tool was executed.",
                call.name, error.instance_path, error
            )));
        }
        let ctx = ToolCtx {
            session: session.to_string(),
            run_id: run_id.to_string(),
            run_cancel: cancel.clone(),
            call_cancel: Arc::new(AtomicBool::new(false)),
            budget: budget.cloned(),
            control: control.clone(),
        };
        if let Err(e)=control.claim(){return Prepared::Done(ToolResult::error(e).with_structured_content(serde_json::json!({"harness_status":"resource_busy"})));}
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


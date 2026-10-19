use super::{Middleware, Turn};
use crate::error::HarnessError;
use crate::tool::{ToolCtx, ToolGate, ToolResult, ToolSpec};
use crate::types::ToolCall;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct Streak {
    name: String,
    args: Value,
    output: String,
    parts: Vec<crate::Part>,
    structured_content: Option<Value>,
    is_error: bool,
    count: usize,
}

/// Catches a model stuck repeating itself: the same tool with the same arguments returning the same
/// result, call after call. From `warn_after` repeats the result carries a note telling the model so;
/// from `block_after` the call is refused. The streak resets with each new user message.
pub struct RepeatGuard {
    warn_after: usize,
    block_after: usize,
    streaks: Mutex<HashMap<String, Streak>>,
}

impl RepeatGuard {
    pub fn new() -> Self {
        Self { warn_after: 3, block_after: 5, streaks: Mutex::new(HashMap::new()) }
    }

    pub fn warn_after(mut self, calls: usize) -> Self {
        self.warn_after = calls.max(2);
        self
    }

    pub fn block_after(mut self, calls: usize) -> Self {
        self.block_after = calls.max(2);
        self
    }

    pub fn forget_session(&self, session: &str) {
        self.streaks.lock().unwrap_or_else(|e| e.into_inner()).remove(session);
    }
}

impl Default for RepeatGuard {
    fn default() -> Self {
        Self::new()
    }
}

fn parse(arguments: &str) -> Value {
    serde_json::from_str(arguments).unwrap_or_else(|_| Value::String(arguments.to_string()))
}

impl Middleware for RepeatGuard {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        if turn.index == 0 {
            self.forget_session(&turn.session);
        }
        Ok(())
    }

    fn before_tool(&self, ctx: &ToolCtx, call: &ToolCall, args: &Value, _spec: &ToolSpec) -> ToolGate {
        let streaks = self.streaks.lock().unwrap_or_else(|e| e.into_inner());
        match streaks.get(ctx.session()) {
            Some(s) if s.count >= self.block_after && s.name == call.name && &s.args == args => ToolGate::Block(format!(
                "Not run: `{}` was already called {} times in a row with these exact arguments and returned the same result each time. \
                 Repeating it again will not help. Try a different approach, or tell the user what is blocking you.",
                call.name, s.count
            )),
            _ => ToolGate::Proceed,
        }
    }

    fn after_tool(&self, session: &str, call: &ToolCall, result: &mut ToolResult) {
        let args = parse(&call.arguments);
        let mut streaks = self.streaks.lock().unwrap_or_else(|e| e.into_inner());
        let streak = streaks.entry(session.to_string()).or_default();
        let same_call = streak.count > 0 && streak.name == call.name && streak.args == args;
        if same_call && streak.count >= self.block_after {
            // Refused in `before_tool`; the refusal must not reset the streak.
            return;
        }
        if same_call && streak.output == result.output && streak.parts == result.parts
            && streak.structured_content == result.structured_content && streak.is_error == result.is_error {
            streak.count += 1;
        } else {
            *streak = Streak {
                name: call.name.clone(), args, output: result.output.clone(), parts: result.parts.clone(),
                structured_content: result.structured_content.clone(), is_error: result.is_error, count: 1,
            };
        }
        if streak.count >= self.warn_after {
            result.output.push_str(&format!(
                "\n\n[Note from the harness: this exact call has now returned the same result {} times in a row. \
                 Do not repeat it; change the approach or explain the problem to the user.]",
                streak.count
            ));
        }
    }
}

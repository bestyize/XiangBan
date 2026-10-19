use super::str_arg;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::time::Duration;

const MAX_QUESTION_CHARS: usize = 2000;
const MAX_OPTIONS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub text: String,
    /// Suggested answers. The user may still type something else.
    pub options: Vec<String>,
}

type Handler = Box<dyn Fn(&str, &Question, &ToolCtx) -> Option<String> + Send + Sync>;

/// Lets the model ask the user a question mid-run. The host shows it in its own UI; the handler
/// blocks until the user answers and returns `None` if they dismiss it.
pub struct AskUser {
    spec: ToolSpec,
    handler: Handler,
}

impl AskUser {
    /// `handler` receives the session id, the question and the call context. Wait for the answer
    /// with `ctx.wait_for(&rx)` so cancelling the run can close the dialog; a handler that ignores
    /// cancellation holds the cancel up until the user answers.
    pub fn new(handler: impl Fn(&str, &Question, &ToolCtx) -> Option<String> + Send + Sync + 'static) -> Self {
        let schema = json!({
            "type": "object",
            "properties": {
                "question": { "type": "string" },
                "options": { "type": "array", "items": { "type": "string" }, "description": "Optional suggested answers." }
            },
            "required": ["question"]
        });
        let description = "Ask the user a question when you need a decision or missing information that you cannot \
                           find yourself. Ask one clear question at a time.";
        let spec = ToolSpec::new("ask_user", description, schema).risk(Risk::Read).serial().timeout(Duration::from_secs(600));
        Self { spec, handler: Box::new(handler) }
    }
}

fn parse_question(args: &Value) -> Result<Question, String> {
    let text = str_arg(args, "question")?.trim();
    if text.is_empty() || text.chars().count() > MAX_QUESTION_CHARS {
        return Err(format!("`question` must be 1 to {MAX_QUESTION_CHARS} characters"));
    }
    let options = match args.get("options") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(list)) => list
            .iter()
            .map(|v| v.as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
            .collect::<Option<Vec<_>>>()
            .ok_or("`options` must be non-empty strings")?,
        Some(_) => return Err("`options` must be an array of strings".to_string()),
    };
    if options.len() > MAX_OPTIONS {
        return Err(format!("at most {MAX_OPTIONS} options"));
    }
    Ok(Question { text: text.to_string(), options })
}

impl Tool for AskUser {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let question = parse_question(&args)?;
        let answer = (self.handler)(ctx.session(), &question, ctx);
        // wait_for returns None both for cancellation and dismissal. Cancellation must never
        // become a user's decision merely because this worker wins the race with the harness.
        if ctx.is_cancelled() { return Err("cancelled".into()); }
        Ok(match answer {
            Some(answer) => format!("The user answered: {answer}"),
            None => "The user dismissed the question without answering.".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn cancellation_during_the_handler_is_not_reported_as_user_dismissal() {
        let ctx = ToolCtx::detached("s");
        let cancel = ctx.run_cancel.clone();
        let ask = AskUser::new(move |_, _, _| {
            cancel.store(true, Ordering::SeqCst);
            None
        });
        assert_eq!(ask.execute(json!({"question": "continue?"}), &ctx), Err("cancelled".into()));
    }
}

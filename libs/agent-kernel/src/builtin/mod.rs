//! Tools that ship with the kernel. None of them knows about any product.

use serde_json::Value;

mod ask;
mod search;
mod subagent;
mod team;
mod time;
mod todo;

#[cfg(feature = "shell")]
mod shell;
#[cfg(feature = "web")]
mod web;

#[cfg(feature = "fs")]
pub mod fs;

pub use ask::{AskUser, Question};
pub use search::{SearchBackend, SearchHit, WebSearch};
pub use subagent::Subagent;
pub use team::{Outbox, Team, TeamMember};
#[cfg(feature = "shell")]
pub use shell::Shell;
#[cfg(feature = "web")]
pub use web::WebFetch;
pub use time::TimeNow;
pub use todo::{Todo, TodoItem, TodoSnapshot, TodoStatus};

pub(crate) fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("`{key}` is required and must be a string"))
}

#[cfg(any(feature = "fs", feature = "skills", feature = "memory"))]
pub(crate) fn opt_str_arg<'a>(args: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(format!("`{key}` must be a string")),
    }
}

pub(crate) fn opt_uint_arg(args: &Value, key: &str) -> Result<Option<u64>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v.as_u64().map(Some).ok_or_else(|| format!("`{key}` must be a non-negative integer")),
    }
}

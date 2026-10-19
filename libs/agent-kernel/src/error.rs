use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum HarnessError {
    Config(String),
    Provider { message: String, retryable: bool },
    Store(String),
    Middleware(String),
    SessionBusy(String),
    MaxTurns(usize),
    Paused(String),
    BudgetExceeded(String),
    Cancelled,
}

impl HarnessError {
    pub fn provider(message: impl Into<String>, retryable: bool) -> Self {
        Self::Provider { message: message.into(), retryable }
    }
}

impl fmt::Display for HarnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(m) => write!(f, "config error: {m}"),
            Self::Provider { message, .. } => write!(f, "provider error: {message}"),
            Self::Store(m) => write!(f, "session store error: {m}"),
            Self::Middleware(m) => write!(f, "middleware error: {m}"),
            Self::SessionBusy(id) => write!(f, "session `{id}` already has a run in progress"),
            Self::MaxTurns(n) => write!(f, "stopped after reaching the turn limit ({n})"),
            Self::Paused(reason) => write!(f, "run paused by continuation guard: {reason}"),
            Self::BudgetExceeded(reason) => write!(f, "run budget exceeded: {reason}"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::error::Error for HarnessError {}

use super::Middleware;
use crate::tool::{Risk, ToolCtx, ToolGate, ToolSpec};
use crate::types::ToolCall;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    /// Only `Risk::Read` tools run.
    ReadOnly,
    /// `Risk::Read` tools run; everything else asks the host.
    Ask,
    /// Everything runs without asking.
    Auto,
}

impl PermissionMode {
    fn to_u8(self) -> u8 {
        match self {
            Self::ReadOnly => 0,
            Self::Ask => 1,
            Self::Auto => 2,
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::ReadOnly,
            1 => Self::Ask,
            _ => Self::Auto,
        }
    }
}

pub struct ApprovalRequest<'a> {
    pub session: &'a str,
    pub tool: &'a str,
    pub risk: Risk,
    pub arguments: &'a Value,
    /// The session has already read external content, so this approval is not skippable by policy.
    pub tainted: bool,
    /// Why the host's `danger_check` flagged this call, if it did. Such a call is asked even in `Auto` mode
    /// and is never remembered.
    pub danger: Option<&'a str>,
    /// For a command tool: what "always" would remember (see `Permission::remember_exec_as`), e.g. `npm test`.
    /// `None` means "always" is not offered for this call: it is asked every time.
    pub remember_as: Option<&'a str>,
    pub(crate) ctx: &'a ToolCtx,
}

impl ApprovalRequest<'_> {
    /// True once the run was cancelled. The answer is then ignored, so a dialog can be closed.
    pub fn is_cancelled(&self) -> bool {
        self.ctx.is_cancelled()
    }

    /// Waits for the user's answer from `rx`; returns `None` if the run is cancelled first or the
    /// sender is dropped. Use it so that cancelling can close a pending dialog.
    pub fn wait_for<T>(&self, rx: &Receiver<T>) -> Option<T> {
        self.ctx.wait_for(rx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Approval {
    Once,
    /// Skip asking for this tool for the rest of this session. `Risk::Exec` tools ignore it unless the host
    /// named the kind of call to remember (`Permission::remember_exec_as`); then only that kind is remembered.
    Always,
    Deny,
}

/// Implemented by the host UI. The call blocks the agent thread until the user answers. Wait with
/// `request.wait_for(&rx)` (or poll `request.is_cancelled()`) so cancelling the run can interrupt it;
/// an approver that ignores cancellation holds up the cancel until it returns, and its answer is
/// then discarded: the tool does not run and no "always" grant is recorded.
pub trait Approver: Send + Sync {
    fn approve(&self, request: &ApprovalRequest<'_>) -> Approval;
}

impl<F> Approver for F
where
    F: Fn(&ApprovalRequest<'_>) -> Approval + Send + Sync,
{
    fn approve(&self, request: &ApprovalRequest<'_>) -> Approval {
        self(request)
    }
}

type DangerCheck = Arc<dyn Fn(&str, &Value) -> Option<String> + Send + Sync>;
type RememberKey = Arc<dyn Fn(&str, &Value) -> Option<String> + Send + Sync>;

pub struct Permission {
    mode: AtomicU8,
    approver: Arc<dyn Approver>,
    danger: Option<DangerCheck>,
    remember: Option<RememberKey>,
    allow: HashSet<String>,
    deny: HashSet<String>,
    always: Mutex<HashSet<(String, String)>>,
    tainted: Mutex<HashSet<String>>,
    /// Modes set for single sessions. They win over `mode`, so concurrent sessions never see each other's choice.
    session_modes: Mutex<HashMap<String, PermissionMode>>,
}

impl Permission {
    pub fn new(mode: PermissionMode, approver: impl Approver + 'static) -> Self {
        Self {
            mode: AtomicU8::new(mode.to_u8()),
            approver: Arc::new(approver),
            danger: None,
            remember: None,
            allow: HashSet::new(),
            deny: HashSet::new(),
            always: Mutex::new(HashSet::new()),
            tainted: Mutex::new(HashSet::new()),
            session_modes: Mutex::new(HashMap::new()),
        }
    }

    /// Never asks for this tool in `Ask` mode. Has no effect in `ReadOnly` mode.
    pub fn allow_tool(mut self, name: &str) -> Self {
        self.allow.insert(name.to_string());
        self
    }

    /// Lets `Auto` mode mean what it says. `check` gets the tool name and its arguments and returns a reason
    /// when the call is too dangerous to run unattended. With a check installed, `Auto` asks only for those
    /// calls; reading external content (taint) and `always_ask` no longer force a prompt there. The other modes
    /// behave as before, and a flagged call is reported in `ApprovalRequest::danger` and never remembered.
    /// Without a check, `Auto` keeps the stricter behaviour described in the README.
    pub fn danger_check(mut self, check: impl Fn(&str, &Value) -> Option<String> + Send + Sync + 'static) -> Self {
        self.danger = Some(Arc::new(check));
        self
    }

    /// Lets "always" apply to a kind of command instead of never applying to `Risk::Exec` tools. `key` gets the
    /// tool name and arguments and returns what to remember, e.g. `npm test` for `npm test -- --watch`, or `None`
    /// for a call that must be asked every time. A later call is skipped only when its key is exactly the same, in the
    /// same session. Flagged (`danger_check`), `always_ask` and tainted calls are still asked.
    pub fn remember_exec_as(mut self, key: impl Fn(&str, &Value) -> Option<String> + Send + Sync + 'static) -> Self {
        self.remember = Some(Arc::new(key));
        self
    }

    /// Always refuses this tool, in every mode.
    pub fn deny_tool(mut self, name: &str) -> Self {
        self.deny.insert(name.to_string());
        self
    }

    pub fn mode(&self) -> PermissionMode {
        PermissionMode::from_u8(self.mode.load(Ordering::Relaxed))
    }

    /// The default for sessions without a mode of their own.
    pub fn set_mode(&self, mode: PermissionMode) {
        self.mode.store(mode.to_u8(), Ordering::Relaxed);
    }

    /// The mode that applies to `session`: its own when one was set, the default otherwise.
    pub fn mode_for(&self, session: &str) -> PermissionMode {
        self.session_modes.lock().unwrap_or_else(|e| e.into_inner()).get(session).copied().unwrap_or_else(|| self.mode())
    }

    /// Sets the mode of one session only. A host running several sessions at once should use this
    /// instead of `set_mode`, which every session without its own mode shares.
    pub fn set_session_mode(&self, session: &str, mode: PermissionMode) {
        self.session_modes.lock().unwrap_or_else(|e| e.into_inner()).insert(session.to_string(), mode);
    }

    /// Drops the "always allow" grants and the taint of one session, e.g. when it is closed.
    pub fn forget_session(&self, session: &str) {
        self.always.lock().unwrap_or_else(|e| e.into_inner()).retain(|(s, _)| s != session);
        self.tainted.lock().unwrap_or_else(|e| e.into_inner()).remove(session);
        self.session_modes.lock().unwrap_or_else(|e| e.into_inner()).remove(session);
    }

    /// Records an "always" grant, as if the user had answered "always": `kind` is what `remember_exec_as`
    /// named for a command tool, `None` for any other tool. A host uses it to restore grants it saved, so
    /// remembering survives a restart; it never applies to flagged, `always_ask` or tainted calls.
    pub fn grant(&self, session: &str, tool: &str, kind: Option<&str>) {
        self.always.lock().unwrap_or_else(|e| e.into_inner()).insert(Self::grant_key(session, tool, kind));
    }

    /// Takes back one grant made by an "always" answer or by `grant`.
    pub fn revoke(&self, session: &str, tool: &str, kind: Option<&str>) {
        self.always.lock().unwrap_or_else(|e| e.into_inner()).remove(&Self::grant_key(session, tool, kind));
    }

    fn grant_key(session: &str, tool: &str, kind: Option<&str>) -> (String, String) {
        match kind {
            Some(kind) => (session.to_string(), format!("{tool}\u{0}{kind}")),
            None => (session.to_string(), tool.to_string()),
        }
    }

    /// Taint is kept in memory only: after a restart a resumed session starts untainted.
    pub fn is_tainted(&self, session: &str) -> bool {
        self.tainted.lock().unwrap_or_else(|e| e.into_inner()).contains(session)
    }
}

impl Middleware for Permission {
    fn before_tool(&self, ctx: &ToolCtx, _call: &ToolCall, args: &Value, spec: &ToolSpec) -> ToolGate {
        let session = ctx.session();
        let name = spec.name.as_str();
        if self.deny.contains(name) {
            return ToolGate::Block(format!("`{name}` is disabled by policy."));
        }
        let mode = self.mode_for(session);
        if mode == PermissionMode::ReadOnly && spec.risk != Risk::Read {
            return ToolGate::Block(format!("Read-only mode: `{name}` is not allowed because it can change state."));
        }

        let tainted = self.is_tainted(session);
        let danger = match &self.danger {
            Some(check) if spec.risk != Risk::Read => check(name, args),
            _ => None,
        };
        let exec_key = match &self.remember {
            Some(key) if spec.risk == Risk::Exec && !spec.always_ask && danger.is_none() => key(name, args),
            _ => None,
        };
        let sticky = (spec.risk == Risk::Exec && exec_key.is_none()) || spec.always_ask || danger.is_some();
        let key = Self::grant_key(session, name, exec_key.as_deref());
        let needs_ask = if self.danger.is_some() && mode == PermissionMode::Auto {
            danger.is_some()
        } else if spec.always_ask {
            true
        } else if spec.risk == Risk::Read {
            false
        } else if spec.self_managed {
            // Only the host app's own state changes, so neither a flag nor external content makes this a user decision.
            danger.is_some()
        } else if tainted {
            true
        } else {
            let remembered = !sticky && self.always.lock().unwrap_or_else(|e| e.into_inner()).contains(&key);
            !(mode == PermissionMode::Auto || self.allow.contains(name) || remembered)
        };

        if needs_ask {
            let request = ApprovalRequest { session, tool: name, risk: spec.risk, arguments: args, tainted, danger: danger.as_deref(), remember_as: exec_key.as_deref(), ctx };
            let decision = self.approver.approve(&request);
            if ctx.is_cancelled() {
                return ToolGate::Block("cancelled".to_string());
            }
            match decision {
                Approval::Once => {}
                Approval::Always => {
                    if !sticky {
                        self.always.lock().unwrap_or_else(|e| e.into_inner()).insert(key);
                    }
                }
                Approval::Deny => return ToolGate::Block(format!("The user denied permission to run `{name}`.")),
            }
        }
        if spec.untrusted_output {
            self.tainted.lock().unwrap_or_else(|e| e.into_inner()).insert(session.to_string());
        }
        ToolGate::Proceed
    }
}

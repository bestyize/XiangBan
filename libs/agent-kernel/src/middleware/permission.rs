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
    permission: &'a Permission,
    spec: &'a ToolSpec,
}

impl ApprovalRequest<'_> {
    /// Match the kernel's actual grant rules so a host never offers a grant that cannot take effect.
    pub fn can_remember(&self) -> bool {
        self.danger.is_none() && !self.spec.always_ask && !self.permission.needs_review(self.session)
            && (self.risk != Risk::Exec || self.remember_as.is_some())
    }
    /// Ordinary pending calls resume after a switch to Auto. Flagged calls and unknown side effects still wait.
    pub fn wait_for_approval(&self, rx: &Receiver<Approval>) -> Option<Approval> {
        self.ctx.wait_for_user_or(rx, || {
            self.permission.can_auto_approve(self.session, self.arguments, self.spec).then_some(Approval::Once)
        })
    }
    /// Identifies the run that is waiting for this approval, for single-call host grants.
    pub fn run_id(&self) -> &str {
        self.ctx.run_id()
    }
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

type DangerCheck = Arc<dyn Fn(PermissionMode, &str, &str, &Value) -> Option<String> + Send + Sync>;
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
    trust_dir: Mutex<Option<std::path::PathBuf>>,
    /// Modes set for single sessions. They win over `mode`, so concurrent sessions never see each other's choice.
    session_modes: Mutex<HashMap<String, PermissionMode>>,
    /// Child sessions (`<session>#<job>`) follow their parent's mode, grants, taint and trust.
    children_inherit: bool,
    check_reads: bool,
    honor_grants_after_taint: bool,
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
            trust_dir: Mutex::new(None),
            session_modes: Mutex::new(HashMap::new()),
            children_inherit: false,
            check_reads: false,
            honor_grants_after_taint: false,
        }
    }

    /// Sessions of delegated jobs (`<session>#<job>`, such as a team member's) count as their parent session:
    /// same mode, same "always" grants, same taint, and approval cards that appear in the parent conversation.
    /// Off by default, because a host may use `#` in its own session ids.
    pub fn children_inherit(mut self) -> Self {
        self.children_inherit = true;
        self
    }

    /// The session whose settings apply to `session`.
    pub fn root_session<'a>(&self, session: &'a str) -> &'a str {
        if self.children_inherit { session.split('#').next().unwrap_or(session) } else { session }
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
    pub fn danger_check(self, check: impl Fn(&str, &Value) -> Option<String> + Send + Sync + 'static) -> Self {
        self.danger_check_in_session(move |_, tool, args| check(tool, args))
    }

    /// `danger_check` for a host whose rules depend on the conversation, e.g. each one working in its own folder:
    /// `check` also gets the session id first.
    pub fn danger_check_in_session(self, check: impl Fn(&str, &str, &Value) -> Option<String> + Send + Sync + 'static) -> Self {
        self.danger_check_in_mode(move |_, session, tool, args| check(session, tool, args))
    }

    /// A host policy that also sees the current session's effective mode, including inherited child modes.
    pub fn danger_check_in_mode(mut self, check: impl Fn(PermissionMode, &str, &str, &Value) -> Option<String> + Send + Sync + 'static) -> Self {
        self.danger = Some(Arc::new(check));
        self
    }

    /// Include sensitive reads in the host's risk policy. Default remains write/execute-only for compatibility.
    pub fn check_reads(mut self) -> Self { self.check_reads = true; self }

    /// Explicit, scoped user grants remain valid after external content is read; danger/review always win.
    pub fn honor_grants_after_taint(mut self) -> Self { self.honor_grants_after_taint = true; self }

    fn danger_for(&self, mode: PermissionMode, session: &str, args: &Value, spec: &ToolSpec) -> Option<String> {
        spec.confirmation_reason.clone().or_else(|| match &self.danger {
            Some(check) if self.check_reads || spec.risk != Risk::Read => check(mode, session, &spec.name, args),
            _ => None,
        })
    }

    fn can_auto_approve(&self, session: &str, args: &Value, spec: &ToolSpec) -> bool {
        self.mode_for(session) == PermissionMode::Auto && !self.deny.contains(&spec.name)
            && !(self.needs_review(session) && spec.risk != Risk::Read)
            && (self.danger.is_some() || !spec.always_ask && !self.is_tainted(session))
            && self.danger_for(PermissionMode::Auto, session, args, spec).is_none()
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
        let session = self.root_session(session);
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

    /// Initialize trust before restoring remembered grants. Legacy tool histories are conservative.
    pub fn restore_trust(&self,dir:&std::path::Path,session:&str,legacy_tools:bool)->Result<(),String>{
        use std::io::Write;
        if session.is_empty() || !session.bytes().all(|c|c.is_ascii_alphanumeric() || matches!(c,b'-'|b'_')) {return Err("invalid trust identity".into());}
        std::fs::create_dir_all(dir).map_err(|e|e.to_string())?;
        let path=dir.join(session);let mut opts=std::fs::OpenOptions::new();opts.write(true).create_new(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;opts.mode(0o600);}
        match opts.open(path){Ok(mut f)=>{f.write_all(if legacy_tools{b"tainted"}else{b"clean"}).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{},Err(e)=>return Err(e.to_string())}
        *self.trust_dir.lock().unwrap_or_else(|e|e.into_inner())=Some(dir.into());
        if self.is_tainted(session){self.tainted.lock().unwrap_or_else(|e|e.into_inner()).insert(session.into());}Ok(())
    }
    pub fn needs_review(&self,session:&str)->bool{let session=self.root_session(session);self.trust_dir.lock().unwrap_or_else(|e|e.into_inner()).as_ref().is_some_and(|d|!matches!(std::fs::read(d.join(session)).ok().as_deref(),Some(b"clean" | b"tainted")))}
    /// Lifts a `review` mark (set by `require_review`) so remembered grants and full trust work again. A `tainted` mark,
    /// which comes from reading outside content, stays.
    pub fn clear_review(&self,session:&str)->Result<(),String>{
        use std::io::Write;
        if !self.needs_review(session){return Ok(());}
        let root=self.trust_dir.lock().unwrap_or_else(|e|e.into_inner());let dir=root.as_ref().ok_or("trust store not initialized")?;
        if !session.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_')){return Err("invalid trust identity".into());}
        let mut f=std::fs::OpenOptions::new().write(true).truncate(true).open(dir.join(session)).map_err(|e|e.to_string())?;
        f.write_all(b"clean").and_then(|_|f.sync_all()).map_err(|e|e.to_string())
    }
    pub fn require_review(&self,session:&str)->Result<(),String>{
        use std::io::Write;
        let root=self.trust_dir.lock().unwrap_or_else(|e|e.into_inner());let dir=root.as_ref().ok_or("trust store not initialized")?;
        if !session.bytes().all(|c|c.is_ascii_alphanumeric()||matches!(c,b'-'|b'_')){return Err("invalid trust identity".into());}
        let mut f=std::fs::OpenOptions::new().write(true).truncate(true).open(dir.join(session)).map_err(|e|e.to_string())?;
        f.write_all(b"review").and_then(|_|f.sync_all()).map_err(|e|e.to_string())
    }
    fn mark_tainted(&self,session:&str)->Result<(),String>{
        if self.needs_review(session){return Ok(());}
        use std::io::Write;
        if let Some(dir)=self.trust_dir.lock().unwrap_or_else(|e|e.into_inner()).as_ref(){
            let mut file=std::fs::OpenOptions::new().write(true).truncate(true).open(dir.join(session)).map_err(|e|e.to_string())?;
            file.write_all(b"tainted").and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;
        }
        self.tainted.lock().unwrap_or_else(|e|e.into_inner()).insert(session.into());Ok(())
    }
    pub fn is_tainted(&self,session:&str)->bool{
        let session=self.root_session(session);
        self.tainted.lock().unwrap_or_else(|e|e.into_inner()).contains(session) || self.trust_dir.lock().unwrap_or_else(|e|e.into_inner()).as_ref().is_some_and(|d|std::fs::read(d.join(session)).ok().as_deref()!=Some(b"clean"))
    }

}

impl Middleware for Permission {
    fn authorizes_tool(&self) -> bool { true }
    fn before_tool(&self, ctx: &ToolCtx, _call: &ToolCall, args: &Value, spec: &ToolSpec) -> ToolGate {
        let session = self.root_session(ctx.session());
        let name = spec.name.as_str();
        if self.deny.contains(name) {
            return ToolGate::Block(format!("`{name}` is disabled by policy."));
        }
        let mode = self.mode_for(session);
        if mode == PermissionMode::ReadOnly && spec.risk != Risk::Read {
            return ToolGate::Block(format!("Read-only mode: `{name}` is not allowed because it can change state."));
        }

        let tainted = self.is_tainted(session);
        let danger = self.danger_for(mode, session, args, spec);
        let exec_key = match &self.remember {
            Some(key) if spec.risk == Risk::Exec && !spec.always_ask && danger.is_none() => key(name, args),
            _ => None,
        };
        let review=self.needs_review(session)&&spec.risk!=Risk::Read;
        let sticky = review || (spec.risk == Risk::Exec && exec_key.is_none()) || spec.always_ask || danger.is_some();
        let key = Self::grant_key(session, name, exec_key.as_deref());
        let remembered = !sticky && self.always.lock().unwrap_or_else(|e| e.into_inner()).contains(&key);
        let needs_ask = if review {true} else if self.danger.is_some() && mode == PermissionMode::Auto {
            danger.is_some()
        } else if spec.always_ask {
            true
        } else if danger.is_some() {
            true
        } else if spec.risk == Risk::Read {
            false
        } else if spec.self_managed {
            // Only the host app's own state changes, so neither a flag nor external content makes this a user decision.
            danger.is_some()
        } else if tainted && !(self.honor_grants_after_taint && remembered) {
            true
        } else {
            !(mode == PermissionMode::Auto || self.allow.contains(name) || remembered)
        };

        if needs_ask {
            let request = ApprovalRequest { session, tool: name, risk: spec.risk, arguments: args, tainted, danger: danger.as_deref(), remember_as: exec_key.as_deref(), ctx, permission: self, spec };
            let decision = {
                let _waiting = ctx.control().wait_for_user();
                self.approver.approve(&request)
            };
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
            if let Err(e)=self.mark_tainted(session){return ToolGate::Block(format!("cannot persist trust boundary: {e}"));}
        }
        ToolGate::Proceed
    }
}

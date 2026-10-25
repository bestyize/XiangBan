//! Expert teams (WorkBuddy's `expertType: team`): a lead agent sets up a team, hands jobs to members, and
//! gets their work back. The lead sees four tools:
//!
//! * `TeamCreate` sets the team up for this conversation turn and lists the members;
//! * `Agent` starts a member on a job. Several `Agent` calls in one reply run at the same time;
//! * `SendMessage` goes to a member that already ran, and continues that member's own conversation;
//! * `TeamDelete` ends the team.
//!
//! Members cannot reach each other: a member's `SendMessage` can only go to the lead, so everything that flows
//! between members passes through the lead, as WorkBuddy's collaboration rules require. A member's answer
//! comes back to the lead as the result of the `Agent` / `SendMessage` call that started it: what the member
//! sent with its own `SendMessage`, or its final text when it sent nothing.
//!
//! The host builds every member agent (persona, tools, store, the same `Permission` as the lead); the kernel
//! only runs them. A member's conversation lives in its own session, `<lead session>#team-<member>-…`, and is
//! kept until the team ends, which happens on `TeamDelete` or when the lead's run ends.

use super::str_arg;
use crate::agent::Agent;
use crate::tool::{Risk, Tool, ToolCtx, ToolSpec};
use crate::types::Event;
use crate::{HarnessError, RunEnd};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_TEXT_CHARS: usize = 40_000;
const MAX_TEAM_NAME: usize = 80;
const POLL: Duration = Duration::from_millis(20);

static MEMBER_SEQ: AtomicU64 = AtomicU64::new(0);

type Observer = Box<dyn Fn(&str, &str, &Event) + Send + Sync>;
type RunObserver = Box<dyn Fn(&crate::RunEvent) -> Result<(), HarnessError> + Send + Sync>;
type FinishHook = Box<dyn Fn(&str) + Send + Sync>;
/// Builds the agent for one member. `Outbox::tool` gives the member its `SendMessage` tool.
type MemberFactory = Box<dyn Fn(&TeamMember, &Outbox) -> Result<Agent, String> + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMember {
    /// The agent id (the file name of its agent document): what the lead passes as `name` / `subagent_type`.
    pub id: String,
    /// The display name, for example 许清楚.
    pub name: String,
    pub profession: String,
    /// What the member is for, shown to the lead when the team is created.
    pub description: String,
}

impl TeamMember {
    pub fn new(id: impl Into<String>, name: impl Into<String>, profession: impl Into<String>, description: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into(), profession: profession.into(), description: description.into() }
    }

    fn label(&self) -> String {
        match (self.name.is_empty(), self.profession.is_empty()) {
            (false, false) => format!("{}（{}）", self.name, self.profession),
            (false, true) => self.name.clone(),
            (true, false) => self.profession.clone(),
            (true, true) => self.id.clone(),
        }
    }
}

/// What a member has sent to the lead during its current job.
#[derive(Clone, Default)]
pub struct Outbox {
    lead_id: Arc<String>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl Outbox {
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *lock(&self.messages))
    }

    /// The member-side `SendMessage` tool. It can only address the lead.
    pub fn tool(&self) -> Arc<dyn Tool> {
        let schema = json!({
            "type": "object",
            "properties": {
                "to": { "type": "string", "description": "Who receives it. Only the team lead can be addressed: use `lead`." },
                "message": { "type": "string", "description": "Your complete work product or question for the lead. The lead sees nothing else you did." },
                "summary": { "type": "string", "description": "A short preview of the message." }
            },
            "required": ["to", "message"]
        });
        let description = "Send your result, or a question you cannot answer yourself, to the team lead. \
            You can only message the lead; you cannot reach other members, whatever their ids. \
            Put the complete work product in the message: the lead does not see your tool calls or files.";
        Arc::new(MemberSend {
            spec: ToolSpec::new("SendMessage", description, schema).risk(Risk::Read).parallel(),
            outbox: self.clone(),
        })
    }
}

struct MemberSend {
    spec: ToolSpec,
    outbox: Outbox,
}

impl Tool for MemberSend {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, _ctx: &ToolCtx) -> Result<String, String> {
        let to = str_arg(&args, "to")?.trim().to_string();
        let message = str_arg(&args, "message")?.trim().to_string();
        let lead = self.outbox.lead_id.as_str();
        if !is_lead(&to, lead) {
            return Err(format!(
                "`{to}` cannot be messaged: members only talk to the lead (`{lead}`), and everything between members goes through the lead. \
                 Send your result to the lead instead."
            ));
        }
        if message.is_empty() || message.chars().count() > MAX_TEXT_CHARS {
            return Err(format!("`message` must be 1 to {MAX_TEXT_CHARS} characters"));
        }
        lock(&self.outbox.messages).push(message);
        Ok("Delivered to the lead.".into())
    }
}

fn is_lead(to: &str, lead: &str) -> bool {
    let to = to.trim().trim_start_matches('@');
    to.eq_ignore_ascii_case(lead) || ["lead", "team-lead", "team_lead", "teamlead"].iter().any(|a| to.eq_ignore_ascii_case(a))
}

struct Slot {
    agent: Arc<Agent>,
    session: String,
    outbox: Outbox,
    busy: AtomicBool,
}

struct Busy<'a>(&'a AtomicBool);

impl Drop for Busy<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

struct TeamRun {
    name: String,
    slots: Mutex<HashMap<String, Arc<Slot>>>,
}

/// One team for the host to hand to its lead agent. Cheap to share: tools hold an `Arc` of it.
pub struct Team {
    lead: TeamMember,
    members: Vec<TeamMember>,
    build: MemberFactory,
    runs: Mutex<HashMap<String, Arc<TeamRun>>>,
    observer: Option<Observer>,
    run_observer: Option<RunObserver>,
    on_finish: Option<FinishHook>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

impl Team {
    /// `members` are the people the lead can call on, not including the lead.
    pub fn new(
        lead: TeamMember,
        members: Vec<TeamMember>,
        build: impl Fn(&TeamMember, &Outbox) -> Result<Agent, String> + Send + Sync + 'static,
    ) -> Self {
        let members = members.into_iter().filter(|m| m.id != lead.id).collect();
        Self { lead, members, build: Box::new(build), runs: Mutex::new(HashMap::new()), observer: None, run_observer: None, on_finish: None }
    }

    /// Called with the lead's session id, the member id and every event of a member's run.
    pub fn observe(mut self, observer: impl Fn(&str, &str, &Event) + Send + Sync + 'static) -> Self {
        self.observer = Some(Box::new(observer));
        self
    }

    /// Observe full run identity of the members' runs, including `parent_run_id`. Errors fail closed.
    pub fn observe_run(mut self, observer: impl Fn(&crate::RunEvent) -> Result<(), HarnessError> + Send + Sync + 'static) -> Self {
        self.run_observer = Some(Box::new(observer));
        self
    }

    /// Called with a member's session id when its conversation ends, for cleanup such as
    /// `Permission::forget_session` or deleting stored messages.
    pub fn on_finish(mut self, hook: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_finish = Some(Box::new(hook));
        self
    }

    pub fn lead(&self) -> &TeamMember {
        &self.lead
    }

    pub fn members(&self) -> &[TeamMember] {
        &self.members
    }

    /// The lead's tools: `TeamCreate`, `TeamDelete`, `Agent` and `SendMessage`.
    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        let team = self.clone();
        vec![
            Arc::new(TeamCreate { spec: team_create_spec(), team: team.clone() }),
            Arc::new(Spawn { spec: spawn_spec(), team: team.clone() }),
            Arc::new(LeadSend { spec: lead_send_spec(), team: team.clone() }),
            Arc::new(TeamDelete { spec: team_delete_spec(), team }),
        ]
    }

    fn roster(&self) -> String {
        let mut text = format!("Team lead: {} (id `{}`) — that is you.\nMembers:\n", self.lead.label(), self.lead.id);
        for member in &self.members {
            text.push_str(&format!("- id `{}`: {}", member.id, member.label()));
            if !member.description.trim().is_empty() {
                text.push_str(&format!(" — {}", member.description.trim()));
            }
            text.push('\n');
        }
        text
    }

    fn run_of(&self, session: &str) -> Result<Arc<TeamRun>, String> {
        lock(&self.runs).get(session).cloned().ok_or_else(|| {
            "No team exists yet. Call `TeamCreate` first, then start members with `Agent`.".to_string()
        })
    }

    fn member(&self, id: &str) -> Result<&TeamMember, String> {
        let id = id.trim();
        if is_lead(id, &self.lead.id) {
            return Err(format!("`{id}` is the team lead, which is you. Do the leading, planning and compiling yourself; only members can be started."));
        }
        self.members.iter().find(|m| m.id == id).ok_or_else(|| {
            let ids: Vec<&str> = self.members.iter().map(|m| m.id.as_str()).collect();
            format!("`{id}` is not a member of this team. Use a member's agent id exactly as listed: {}", ids.join(", "))
        })
    }

    /// Ends the team of a lead session and tells the host about every member conversation that ends with it.
    fn end(&self, session: &str) -> Vec<String> {
        let Some(run) = lock(&self.runs).remove(session) else { return Vec::new() };
        let sessions: Vec<String> = lock(&run.slots).values().map(|slot| slot.session.clone()).collect();
        if let Some(hook) = &self.on_finish {
            for member_session in &sessions {
                hook(member_session);
            }
        }
        sessions
    }

    fn slot(&self, run: &TeamRun, lead_session: &str, member: &TeamMember) -> Result<Arc<Slot>, String> {
        let mut slots = lock(&run.slots);
        if let Some(slot) = slots.get(&member.id) {
            return Ok(slot.clone());
        }
        let outbox = Outbox { lead_id: Arc::new(self.lead.id.clone()), messages: Arc::default() };
        let agent = (self.build)(member, &outbox).map_err(|e| format!("could not start {}: {e}", member.label()))?;
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
        let session = format!("{lead_session}#team-{}-{millis:x}-{:x}-{}", member.id, std::process::id(), MEMBER_SEQ.fetch_add(1, Ordering::Relaxed));
        let slot = Arc::new(Slot { agent: Arc::new(agent), session, outbox, busy: AtomicBool::new(false) });
        slots.insert(member.id.clone(), slot.clone());
        Ok(slot)
    }

    /// Gives `member` a job (or a follow-up) and returns what it reports back.
    fn hand_over(&self, slot: &Slot, member: &TeamMember, prompt: &str, ctx: &ToolCtx) -> Result<String, String> {
        if slot.busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(format!("{} is still working on an earlier job. Wait for that result, or give it a different member.", member.label()));
        }
        let _busy = Busy(&slot.busy);
        slot.outbox.take();
        let cancel = Arc::new(AtomicBool::new(false));
        let finished = AtomicBool::new(false);
        let result = thread::scope(|scope| {
            scope.spawn(|| {
                while !finished.load(Ordering::Relaxed) {
                    if ctx.is_cancelled() {
                        cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                    thread::sleep(POLL);
                }
            });
            struct Finished<'a>(&'a AtomicBool);
            impl Drop for Finished<'_> {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::SeqCst);
                }
            }
            let _finished = Finished(&finished);
            let options = crate::RunOptions {
                parent_run_id: Some(ctx.run_id().into()),
                shared_budget: ctx.tool_budget().cloned(),
                control: Some(ctx.control().clone()),
                max_no_progress_rounds: Some(6),
                ..Default::default()
            };
            slot.agent.run_parts_observed(&slot.session, vec![crate::Part::Text { text: prompt.into() }], options, &mut |record| {
                if let Some(observer) = &self.run_observer {
                    observer(&record)?;
                }
                if let (Some(observer), crate::RunEventBody::Event { event }) = (&self.observer, record.body) {
                    if !matches!(event, Event::ToolDispatched { .. }) {
                        observer(ctx.session(), &member.id, &event);
                    }
                }
                Ok(())
            }, &cancel)
        });
        let sent = slot.outbox.take();
        match result {
            Ok(summary) => Ok(report(member, &sent, &summary.text)),
            // What a member already sent is not lost when its run then fails or pauses.
            Err(HarnessError::Cancelled) => Err("cancelled".into()),
            Err(error) if !sent.is_empty() => Ok(format!("{}\n\n(the member's run then stopped early: {error})", report(member, &sent, ""))),
            Err(error) => Err(format!("{} failed: {error}", member.label())),
        }
    }
}

fn report(member: &TeamMember, sent: &[String], final_text: &str) -> String {
    let body = if !sent.is_empty() {
        sent.join("\n\n")
    } else if final_text.trim().is_empty() {
        "(the member finished without a text answer)".to_string()
    } else {
        final_text.trim().to_string()
    };
    format!("[Report from {}]\n{body}", member.label())
}

fn prompt_arg(args: &Value, key: &str) -> Result<String, String> {
    let text = str_arg(args, key)?.trim();
    if text.is_empty() || text.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("`{key}` must be 1 to {MAX_TEXT_CHARS} characters"));
    }
    Ok(text.to_string())
}

// ───────────── the lead's tools ─────────────

fn team_create_spec() -> ToolSpec {
    ToolSpec::new(
        "TeamCreate",
        "Set up your team for this task. Only the team lead may call it, and it must come before you start any member. \
         It returns the members with the agent ids you pass to `Agent`. Never act out the members yourself: their work comes from them.",
        json!({
            "type": "object",
            "properties": {
                "team_name": { "type": "string", "description": "A short name for this team and task." },
                "description": { "type": "string", "description": "What the team is working on." }
            },
            "required": ["team_name"]
        }),
    )
    .risk(Risk::Read)
}

struct TeamCreate {
    spec: ToolSpec,
    team: Arc<Team>,
}

impl Tool for TeamCreate {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let name = str_arg(&args, "team_name")?.trim().to_string();
        if name.is_empty() || name.chars().count() > MAX_TEAM_NAME {
            return Err(format!("`team_name` must be 1 to {MAX_TEAM_NAME} characters"));
        }
        let mut runs = lock(&self.team.runs);
        if let Some(existing) = runs.get(ctx.session()) {
            return Err(format!("Team `{}` already exists for this task. Start members with `Agent`.", existing.name));
        }
        runs.insert(ctx.session().to_string(), Arc::new(TeamRun { name: name.clone(), slots: Mutex::new(HashMap::new()) }));
        drop(runs);
        Ok(format!("Team `{name}` created.\n{}", self.team.roster()))
    }

    /// A team lasts as long as the lead's run: members are not kept between one answer and the next.
    fn on_run_end(&self, end: &RunEnd) -> Result<(), String> {
        self.team.end(&end.session_id);
        Ok(())
    }
}

fn spawn_spec() -> ToolSpec {
    ToolSpec::new(
        "Agent",
        "Start a team member on a job and get its work back. Pass the member's agent id as both `name` and `subagent_type`. \
         The member sees only `prompt`, so include the task, all the input it needs (earlier phases' full output when it builds on them) \
         and what to report back. Start every member of a parallel phase in the same reply; they run at the same time. \
         Members cannot talk to each other: you carry results between them.",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The member's agent id, exactly as TeamCreate listed it." },
                "subagent_type": { "type": "string", "description": "The same agent id." },
                "prompt": { "type": "string", "description": "Complete instructions for this job." },
                "description": { "type": "string", "description": "A few words describing the job." }
            },
            "required": ["name", "prompt"]
        }),
    )
    .risk(Risk::Read)
    .untrusted_output()
    .parallel()
    .timeout(Duration::from_secs(1800))
}

struct Spawn {
    spec: ToolSpec,
    team: Arc<Team>,
}

impl Tool for Spawn {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let id = str_arg(&args, "name")?.trim().to_string();
        if let Some(kind) = args.get("subagent_type").and_then(Value::as_str).map(str::trim).filter(|k| !k.is_empty()) {
            if kind != id {
                return Err(format!("`subagent_type` (`{kind}`) must be the same agent id as `name` (`{id}`)."));
            }
        }
        let prompt = prompt_arg(&args, "prompt")?;
        let run = self.team.run_of(ctx.session())?;
        let member = self.team.member(&id)?.clone();
        let slot = self.team.slot(&run, ctx.session(), &member)?;
        self.team.hand_over(&slot, &member, &prompt, ctx)
    }
}

fn lead_send_spec() -> ToolSpec {
    ToolSpec::new(
        "SendMessage",
        "Send a follow-up to a member that already ran: a correction, the next phase's input, or an answer to its question. \
         It continues that member's own conversation, so it remembers its earlier work. Members you have not started yet need `Agent` first. \
         Returns the member's reply.",
        json!({
            "type": "object",
            "properties": {
                "to": { "type": "string", "description": "The member's agent id." },
                "message": { "type": "string", "description": "What to tell the member." },
                "summary": { "type": "string", "description": "A short preview of the message." }
            },
            "required": ["to", "message"]
        }),
    )
    .risk(Risk::Read)
    .untrusted_output()
    .parallel()
    .timeout(Duration::from_secs(1800))
}

struct LeadSend {
    spec: ToolSpec,
    team: Arc<Team>,
}

impl Tool for LeadSend {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let to = str_arg(&args, "to")?.trim().to_string();
        let message = prompt_arg(&args, "message")?;
        let run = self.team.run_of(ctx.session())?;
        let member = self.team.member(&to)?.clone();
        let slot = lock(&run.slots).get(&member.id).cloned().ok_or_else(|| {
            format!("{} has not been started yet. Start it with `Agent` (name `{}`) first.", member.label(), member.id)
        })?;
        self.team.hand_over(&slot, &member, &message, ctx)
    }
}

fn team_delete_spec() -> ToolSpec {
    ToolSpec::new(
        "TeamDelete",
        "End the team once its work is delivered. Members' conversations are dropped; a later phase needs a new `TeamCreate`.",
        json!({ "type": "object", "properties": {} }),
    )
    .risk(Risk::Read)
}

struct TeamDelete {
    spec: ToolSpec,
    team: Arc<Team>,
}

impl Tool for TeamDelete {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, _args: Value, ctx: &ToolCtx) -> Result<String, String> {
        self.team.run_of(ctx.session())?;
        let ended = self.team.end(ctx.session());
        Ok(format!("Team ended ({} member conversation(s) closed).", ended.len()))
    }
}

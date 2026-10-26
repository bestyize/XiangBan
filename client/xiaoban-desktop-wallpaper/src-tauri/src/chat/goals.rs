//! Desktop adapter for durable goals. All writes are serialized through ChatState.
use super::*;
use agent_kernel::{goals::{GoalContract, GoalRecord, GoalState, GoalStore}, recovery};
use agent_kernel::RunEvent;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalInput {
    pub outcome: String,
    #[serde(default)] pub verification: String,
    #[serde(default)] pub constraints: String,
    #[serde(default)] pub boundaries: String,
    #[serde(default)] pub stop_when: String,
    #[serde(default)] pub verification_command: String,
    #[serde(default)] pub auto_resume: bool,
}
impl From<GoalInput> for GoalContract {
    fn from(input: GoalInput) -> Self {
        Self { outcome: input.outcome, verification: input.verification,
            constraints: input.constraints, boundaries: input.boundaries, stop_when: input.stop_when,
            verification_command: input.verification_command, auto_resume: input.auto_resume }
    }
}

fn require_control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "control" { Ok(()) } else { Err("长期目标只能在聊天工作台操作".into()) }
}
fn millis() -> u64 { now().1.min(u128::from(u64::MAX)) as u64 }
fn store(app: &tauri::AppHandle) -> Result<GoalStore, String> {
    Ok(GoalStore::new(Dirs::of(app)?.chat.join("goals")))
}
pub(super) fn metadata_lease(app: &tauri::AppHandle, id: &str) -> Result<agent_kernel::supervisor::SessionLease, String> {
    let root = Dirs::of(app)?.chat.join("goal-leases");
    for _ in 0..50 {
        match agent_kernel::supervisor::SessionLease::acquire(&root, id) {
            Ok(lease) => return Ok(lease),
            Err(e) if e == "session is already owned by another process" => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => return Err(e),
        }
    }
    Err("目标记录正在另一个实例中更新，请稍后重试".into())
}
fn exists(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    if !sessions::valid_id(id) { return Err("无效的会话编号".into()); }
    if mutate(app, |index| index.get(id).is_some())? { Ok(()) }
    else { Err("会话不存在".into()) }
}
fn running(app: &tauri::AppHandle, id: &str) -> bool {
    lock(&app.state::<ChatState>().runs).contains_key(id)
}
fn ensure_not_owned_elsewhere(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    if !running(app, id) && agent_kernel::supervisor::SessionLease::held(Dirs::of(app)?.leases(), id)? {
        return Err("该会话正在另一个小伴实例中执行，请在那个实例里停止任务".into());
    }
    Ok(())
}
fn changed(app: &tauri::AppHandle, id: &str, record: Option<&GoalRecord>) {
    emit(app, "chat-goal-changed", json!({"session": id, "goal": record}));
}

pub(super) fn failed_before_run(app: &tauri::AppHandle, id: &str, error: &str) -> Result<(), String> {
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _lease = metadata_lease(app, id)?;
    let db = store(app)?;
    if let Some(mut goal) = db.load(id)? {
        if goal.state == GoalState::Running && goal.last_run_id.is_none() {
            goal.state = GoalState::Paused;
            goal.note = error.chars().take(2000).collect();
            goal.updated_at_ms = millis();
            db.save(&goal)?;
            changed(app, id, Some(&goal));
        }
    }
    Ok(())
}
/// Called at startup/first read to reconcile records abandoned by the previous
/// application process. An unverified dispatch never gets replayed automatically.
fn current(app: &tauri::AppHandle, id: &str) -> Result<Option<GoalRecord>, String> {
    let store = store(app)?;
    let Some(mut goal) = store.load(id)? else { return Ok(None) };
    if goal.state == GoalState::Running && !running(app, id)
        && !agent_kernel::supervisor::SessionLease::held(Dirs::of(app)?.leases(), id)?
    {
        let trace = goal.last_run_id.as_deref().and_then(|run_id| {
            let log = JsonlRunLog::new(Dirs::of(app).ok()?.runs().join(id));
            log.inspect(run_id).ok()
        });
        recovery::reconcile(&mut goal, trace.as_ref(), millis());
        store.save(&goal)?;
        changed(app, id, Some(&goal));
    }
    Ok(Some(goal))
}

/// One startup reconciliation pass. The saved contract must explicitly allow
/// resumption; a deliberate pause, uncertain dispatch or missing log never qualifies.
pub fn resume_pending(app: &tauri::AppHandle) {
    let app = app.clone();
    let _ = std::thread::Builder::new().name("goal-recovery".into()).spawn(move || {
        let attempt = || -> Result<(), String> {
            let records = store(&app)?.list()?;
            for saved in records {
                if saved.state != GoalState::Running || !saved.contract.auto_resume { continue; }
                if !mutate(&app, |index| index.get(&saved.session_id).is_some())? { continue; }
                let state = app.state::<ChatState>();
                let _write = lock(&state.goal_io);
                let _metadata_lease = metadata_lease(&app, &saved.session_id)?;
                if running(&app, &saved.session_id) || agent_kernel::supervisor::SessionLease::held(Dirs::of(&app)?.leases(), &saved.session_id)? { continue; }
                let log = JsonlRunLog::new(Dirs::of(&app)?.runs().join(&saved.session_id));
                let trace = saved.last_run_id.as_deref().and_then(|run| log.inspect(run).ok());
                let authorized = recovery::resume_authorized(&saved, trace.as_ref());
                let Some(mut goal) = current(&app, &saved.session_id)? else { continue; };
                // Terminal states and needs_review are always left for their normal workflow.
                if !authorized || goal.state != GoalState::Interrupted { continue; }
                let previous = goal.clone();
                goal.state = GoalState::Paused;
                goal.begin(millis())?;
                store(&app)?.save(&goal)?;
                if let Err(error) = super::send(app.clone(), goal.session_id.clone(), prompt(&goal), None, Some(false), None) {
                    let mut paused = previous;
                    paused.state = GoalState::Paused;
                    paused.note = format!("自动继续未启动：{error}").chars().take(2000).collect();
                    store(&app)?.save(&paused)?;
                    changed(&app, &paused.session_id, Some(&paused));
                } else {
                    changed(&app, &goal.session_id, Some(&goal));
                }
            }
            Ok(())
        };
        if let Err(error) = attempt() { eprintln!("长期目标恢复失败：{error}"); }
    });
}

#[tauri::command]
pub fn chat_goal_get(
    app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String,
) -> Result<Option<GoalRecord>, String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    current(&app, &session_id)
}

#[tauri::command]
pub fn chat_goal_set(
    app: tauri::AppHandle, window: tauri::WebviewWindow,
    session_id: String, contract: GoalInput,
) -> Result<GoalRecord, String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("运行期间不能修改目标".into()); }
    let existing = current(&app, &session_id)?;
    if existing.as_ref().is_some_and(|g| !matches!(g.state, GoalState::Draft | GoalState::Completed)) {
        return Err("当前目标尚未结束，请先继续、核验或清除它".into());
    }
    let goal = GoalRecord::draft(session_id.clone(), contract.into(), millis())?;
    state.todo.remove_session(&session_id).map_err(|e| e.to_string())?;
    store(&app)?.save(&goal)?;
    changed(&app, &session_id, Some(&goal));
    Ok(goal)
}

fn prompt(goal: &GoalRecord) -> String {
    let c = &goal.contract;
    let mut text = format!("请按照以下长期目标工作。目标：{}\n", c.outcome);
    if !c.verification.trim().is_empty() { text.push_str(&format!("验收条件：{}\n", c.verification)); }
    if !c.verification_command.trim().is_empty() {
        text.push_str(&format!("用户指定验收命令（最后一次修改后原样执行；普通检查用 shell，长检查用 process_start/process_read 等待退出码为 0）：{}\n", c.verification_command));
    }
    if !c.constraints.trim().is_empty() { text.push_str(&format!("约束：{}\n", c.constraints)); }
    if !c.boundaries.trim().is_empty() { text.push_str(&format!("范围：{}\n", c.boundaries)); }
    if !c.stop_when.trim().is_empty() { text.push_str(&format!("遇到以下情况请停止并询问：{}\n", c.stop_when)); }
    text.push_str("执行中请使用实际工具核验结果，不要仅凭口头说明宣称成功。");
    text.push_str("复杂任务使用 todo 保存里程碑、实际证据和 handoff（当前阻塞、决定和下一步），以便压缩或重启后接手。不要把所有步骤一次标记完成。");
    if goal.attempts > 1 {
        text.push_str("这是新的继续回合。请先检查当前文件、任务及外部状态，从现场继续；不要盲目重放上次可能成功的写入操作。");
    }
    text
}

/// Starts a *new* turn with the saved contract. No service runs while the
/// user is offline; no historical tool call is replayed by this API.
#[tauri::command]
pub fn chat_goal_start(
    app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String,
) -> Result<GoalRecord, String> {
    require_control(&window)?;
    start_saved(&app, session_id)
}

pub(super) fn start_saved(app: &tauri::AppHandle, session_id: String) -> Result<GoalRecord, String> {
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    // Hold the goal lock until chat_send has registered the run or refused it.
    // Otherwise a simultaneous read could mistake the brief dispatch gap
    // for a crashed process and mark an in-flight goal NeedsReview.
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("当前会话正在运行".into()); }
    let mut goal = current(&app, &session_id)?.ok_or("请先设定目标")?;
    let previous = goal.clone();
    goal.begin(millis())?;
    store(&app)?.save(&goal)?;
    match super::send(app.clone(), session_id.clone(), prompt(&goal), None, Some(false), None) {
        Ok(_) => {
            changed(&app, &session_id, Some(&goal));
            Ok(goal)
        }
        Err(error) => {
            store(&app)?.save(&previous)?;
            changed(&app, &session_id, Some(&previous));
            Err(error)
        }
    }
}

/// Only used after the host proves that the sole unresolved dispatch was ask_user.
pub(super) fn resume_answer(app: &tauri::AppHandle, session: &str, answer: String) -> Result<(), String> {
    let state=app.state::<ChatState>();
    let _write=lock(&state.goal_io);
    let _lease=metadata_lease(app,session)?;
    let Some(mut goal)=current(app,session)? else {
        super::send(app.clone(),session.into(),answer,None,None,None)?;return Ok(());
    };
    let previous=goal.clone();
    if matches!(goal.state,GoalState::Interrupted|GoalState::NeedsReview){goal.state=GoalState::Paused;}
    goal.begin(millis())?;
    store(app)?.save(&goal)?;
    let input=format!("{}\n\n{}",prompt(&goal),answer);
    if let Err(error)=super::send(app.clone(),session.into(),input,None,None,None){store(app)?.save(&previous)?;return Err(error);}
    changed(app,session,Some(&goal));Ok(())
}

#[tauri::command]
pub fn chat_goal_pause(
    app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String,
) -> Result<GoalRecord, String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    // Cancel an active run first; never wait for its blocking tool while
    // holding the goal metadata lock.
    if running(&app, &session_id) { super::cancel_run(&app, &session_id); }
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    let mut goal = current(&app, &session_id)?.ok_or("没有长期目标")?;
    goal.pause(millis())?;
    store(&app)?.save(&goal)?;
    changed(&app, &session_id, Some(&goal));
    Ok(goal)
}

/// The user inspects the uncertain state and explicitly acknowledges it.
/// Does not imply that the previous action failed, and does not replay it.
#[tauri::command]
pub fn chat_goal_acknowledge(
    app: tauri::AppHandle, window: tauri::WebviewWindow,
    session_id: String, evidence: String,
) -> Result<GoalRecord, String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("仍有运行中的任务".into()); }
    let mut goal = current(&app, &session_id)?.ok_or("没有长期目标")?;
    if !matches!(goal.state, GoalState::Interrupted | GoalState::NeedsReview) {
        return Err("当前目标没有需要人工核验的中断".into());
    }
    if evidence.trim().is_empty() || evidence.chars().count() > 2000 {
        return Err("请填写核验结论，最多 2000 字".into());
    }
    goal.note = evidence;
    goal.state = GoalState::Paused;
    goal.updated_at_ms = millis();
    store(&app)?.save(&goal)?;
    changed(&app, &session_id, Some(&goal));
    Ok(goal)
}

/// Completion is explicit: normal model finish is AwaitingVerification.
#[tauri::command]
pub fn chat_goal_finish(
    app: tauri::AppHandle, window: tauri::WebviewWindow,
    session_id: String, evidence: String,
) -> Result<GoalRecord, String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("任务仍在执行，不能验收完成".into()); }
    let mut goal = current(&app, &session_id)?.ok_or("没有长期目标")?;
    goal.finish_verified(&evidence, millis())?;
    store(&app)?.save(&goal)?;
    if let Some(run) = &goal.last_run_id {
        let outcome = task_outcome::TaskOutcome {
            run_id: run.clone(), session_id: session_id.clone(), status: task_outcome::Status::Succeeded,
            last_tool_call_id: task_outcome::inspect(&JsonlRunLog::new(Dirs::of(&app)?.runs().join(&session_id)), &session_id, run).ok().and_then(|record| record.last_tool_call_id),
            validator: "user_confirmation".into(), evidence: goal.note.clone(), verified_at_ms: Some(goal.updated_at_ms),
        };
        task_outcome::save(&Dirs::of(&app)?.outcomes(), &outcome)?;
        emit(&app, "chat-task-outcome", json!({"session":session_id,"outcome":outcome}));
    }
    changed(&app, &session_id, Some(&goal));
    Ok(goal)
}

#[tauri::command]
pub fn chat_goal_delete(
    app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String,
) -> Result<(), String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    let state = app.state::<ChatState>();
    let _write = lock(&state.goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("先暂停并停止当前任务".into()); }
    store(&app)?.delete(&session_id)?;
    changed(&app, &session_id, None);
    Ok(())
}

/// Called only after the corresponding RunEvent has been durably appended
/// to JsonlRunLog. If updating a goal fails, the observer fails closed.
pub(super) fn observe(app: &tauri::AppHandle, record: &RunEvent, tokens: u64, excluded_wait_ms: u64) -> Result<(), String> {
    if !matches!(&record.body, RunEventBody::State { .. } | RunEventBody::Event { event: agent_kernel::Event::Usage { .. } | agent_kernel::Event::ToolEnd { .. } }) { return Ok(()); }
    let chat = app.state::<ChatState>();
    let _write = lock(&chat.goal_io);
    let _metadata_lease = metadata_lease(app, &record.session_id)?;
    let db = store(app)?;
    if let Some(mut goal) = db.load(&record.session_id)? {
        if goal.state == GoalState::Running || (goal.state == GoalState::Paused && goal.last_run_id.as_deref() == Some(&record.run_id)) {
            goal.account(&record.run_id, tokens, record.timestamp_ms.saturating_sub(excluded_wait_ms))?;
            if let RunEventBody::State { state, reason } = &record.body {
                goal.observe(&record.run_id, *state, record.timestamp_ms)?;
                if let Some(reason) = reason { goal.note = reason.chars().take(2000).collect(); }
                if goal.state == GoalState::AwaitingVerification && *state == agent_kernel::RunState::Completed && !goal.contract.verification_command.trim().is_empty() {
                    let log = JsonlRunLog::new(Dirs::of(app)?.runs().join(&record.session_id));
                    let outcome = task_outcome::inspect(&log, &record.session_id, &record.run_id)?;
                    let plan = chat.todo.load_snapshot(&record.session_id).map_err(|e| e.to_string())?;
                    if outcome.status == task_outcome::Status::Unknown && plan.todos.iter().all(|item| item.status == agent_kernel::builtin::TodoStatus::Completed) {
                        if let Some(evidence) = task_runtime::trace_evidence(&log, &record.session_id, &record.run_id, &goal.contract.verification_command)? {
                            goal.finish_verified(&evidence, record.timestamp_ms)?;
                        }
                    }
                }
            }
            db.save(&goal)?;
            changed(app, &record.session_id, Some(&goal));
        }
    }
    Ok(())
}

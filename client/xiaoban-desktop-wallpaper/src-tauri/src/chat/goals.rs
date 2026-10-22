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
}
impl From<GoalInput> for GoalContract {
    fn from(input: GoalInput) -> Self {
        Self { outcome: input.outcome, verification: input.verification,
            constraints: input.constraints, boundaries: input.boundaries, stop_when: input.stop_when }
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

#[tauri::command]
pub fn chat_goal_get(
    app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String,
) -> Result<Option<GoalRecord>, String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    let _write = lock(&app.state::<ChatState>().goal_io);
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
    let _write = lock(&app.state::<ChatState>().goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("运行期间不能修改目标".into()); }
    let existing = current(&app, &session_id)?;
    if existing.as_ref().is_some_and(|g| !matches!(g.state, GoalState::Draft | GoalState::Completed)) {
        return Err("当前目标尚未结束，请先继续、核验或清除它".into());
    }
    let goal = GoalRecord::draft(session_id.clone(), contract.into(), millis())?;
    store(&app)?.save(&goal)?;
    changed(&app, &session_id, Some(&goal));
    Ok(goal)
}

fn prompt(goal: &GoalRecord) -> String {
    let c = &goal.contract;
    let mut text = format!("请按照以下长期目标工作。目标：{}\n", c.outcome);
    if !c.verification.trim().is_empty() { text.push_str(&format!("验收条件：{}\n", c.verification)); }
    if !c.constraints.trim().is_empty() { text.push_str(&format!("约束：{}\n", c.constraints)); }
    if !c.boundaries.trim().is_empty() { text.push_str(&format!("范围：{}\n", c.boundaries)); }
    if !c.stop_when.trim().is_empty() { text.push_str(&format!("遇到以下情况请停止并询问：{}\n", c.stop_when)); }
    text.push_str("执行中请使用实际工具核验结果，不要仅凭口头说明宣称成功。");
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
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    // Hold the goal lock until chat_send has registered the run or refused it.
    // Otherwise a simultaneous read could mistake the brief dispatch gap
    // for a crashed process and mark an in-flight goal NeedsReview.
    let _write = lock(&app.state::<ChatState>().goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("当前会话正在运行".into()); }
    let mut goal = current(&app, &session_id)?.ok_or("请先设定目标")?;
    let previous = goal.clone();
    goal.begin(millis())?;
    store(&app)?.save(&goal)?;
    match super::chat_send(app.clone(), session_id.clone(), prompt(&goal), None, Some(false), None) {
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

#[tauri::command]
pub fn chat_goal_pause(
    app: tauri::AppHandle, window: tauri::WebviewWindow, session_id: String,
) -> Result<GoalRecord, String> {
    require_control(&window)?;
    exists(&app, &session_id)?;
    ensure_not_owned_elsewhere(&app, &session_id)?;
    // Cancel an active run first; never wait for its blocking tool while
    // holding the goal metadata lock.
    if running(&app, &session_id) { super::chat_cancel(app.clone(), session_id.clone()); }
    let _write = lock(&app.state::<ChatState>().goal_io);
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
    let _write = lock(&app.state::<ChatState>().goal_io);
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
    let _write = lock(&app.state::<ChatState>().goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("任务仍在执行，不能验收完成".into()); }
    let mut goal = current(&app, &session_id)?.ok_or("没有长期目标")?;
    goal.finish_verified(&evidence, millis())?;
    store(&app)?.save(&goal)?;
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
    let _write = lock(&app.state::<ChatState>().goal_io);
    let _metadata_lease = metadata_lease(&app, &session_id)?;
    if running(&app, &session_id) { return Err("先暂停并停止当前任务".into()); }
    store(&app)?.delete(&session_id)?;
    changed(&app, &session_id, None);
    Ok(())
}

/// Called only after the corresponding RunEvent has been durably appended
/// to JsonlRunLog. If updating a goal fails, the observer fails closed.
pub(super) fn observe(app: &tauri::AppHandle, record: &RunEvent) -> Result<(), String> {
    let RunEventBody::State { state, .. } = &record.body else { return Ok(()) };
    let _write = lock(&app.state::<ChatState>().goal_io);
    let _metadata_lease = metadata_lease(app, &record.session_id)?;
    let db = store(app)?;
    if let Some(mut goal) = db.load(&record.session_id)? {
        if goal.state == GoalState::Running {
            goal.observe(&record.run_id, *state, record.timestamp_ms)?;
            db.save(&goal)?;
            changed(app, &record.session_id, Some(&goal));
        }
    }
    Ok(())
}

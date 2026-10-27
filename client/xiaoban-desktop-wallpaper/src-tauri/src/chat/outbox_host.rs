//! Desktop scheduling and IPC. The outbox is authoritative across both app windows.
use super::outbox::{Delivery, Outbox, Snapshot};
use super::*;

fn with_queue<T>(
    app: &tauri::AppHandle,
    f: impl FnOnce(&mut Outbox) -> Result<T, String>,
) -> Result<T, String> {
    let state = app.state::<ChatState>();
    let mut queue = lock(&state.outbox);
    if queue.is_none() {
        *queue = Some(Outbox::new(Dirs::of(app)?.chat.join("outbox")));
    }
    f(queue.as_mut().unwrap())
}
fn binding(app: &tauri::AppHandle, session: &str) -> Result<Value, String> {
    let saved = mutate(app, |index| index.get(session).cloned())?.ok_or("这个对话已经不存在")?;
    let s = serde_json::to_value(saved).map_err(|e| e.to_string())?;
    Ok(
        json!({"model":s["model"],"workspace":s["workspace"],"expert":s["expert"],"contextTokens":s["contextTokens"],"connectors":s["connectors"],"worktree":s["worktree"]}),
    )
}
pub(super) fn snapshot(app: &tauri::AppHandle, session: &str) -> Result<Snapshot, String> {
    with_queue(app, |q| q.snapshot(session))
}
fn publish(app: &tauri::AppHandle, session: &str) -> Result<Snapshot, String> {
    let snapshot = snapshot(app, session)?;
    emit(
        app,
        "chat-queue-changed",
        json!({"session":session,"queue":snapshot}),
    );
    Ok(snapshot)
}
fn validate(app: &tauri::AppHandle, session: &str, entry: &outbox::Entry) -> Result<(), String> {
    if entry.binding != binding(app, session)? {
        return Err("会话配置已变化，请检查后继续发送队列".into());
    }
    let files: Vec<Attachment> = serde_json::from_value(
        entry
            .message
            .structured_content
            .as_ref()
            .map(|v| v["attachments"].clone())
            .unwrap_or(json!([])),
    )
    .map_err(|e| e.to_string())?;
    for file in files {
        if !Path::new(&file.path).is_file() {
            return Err(format!("附件已不可用：{}，请编辑或删除这条消息", file.name));
        }
    }
    Ok(())
}

pub(super) fn open(app: &tauri::AppHandle, session: &str) -> Result<(), String> {
    with_queue(app, |q| {
        q.open(session);
        Ok(())
    })
}
pub(super) fn pause(app: &tauri::AppHandle, session: &str, reason: &str) -> Result<(), String> {
    with_queue(app, |q| q.pause(session, reason))?;
    publish(app, session)?;
    Ok(())
}
// Caller holds delivery. Starting and final cleanup use the same gate as enqueue/cancel.
pub(super) fn kick(app: &tauri::AppHandle, session: &str) {
    if lock(&app.state::<ChatState>().runs).contains_key(session) {
        return;
    }
    let result = (|| -> Result<(), String> {
        let Some(entry) = with_queue(app, |q| q.claim(session, false, false))? else {
            return Ok(());
        };
        if entry.binding != binding(app, session)? {
            return Err("会话配置已变化，请检查后继续发送队列".into());
        }
        let mentions: Vec<mentions::Mention> =
            serde_json::from_value(entry.mentions.clone()).map_err(|e| e.to_string())?;
        lock(&app.state::<ChatState>().mentions).insert(session.into(), mentions);
        let images = entry
            .message
            .parts
            .iter()
            .filter_map(|p| {
                if let Part::Image { url } = p {
                    Some(url.clone())
                } else {
                    None
                }
            })
            .collect();
        let saved: Vec<Attachment> = serde_json::from_value(
            entry
                .message
                .structured_content
                .as_ref()
                .and_then(|v| v.get("attachments"))
                .cloned()
                .unwrap_or(json!([])),
        )
        .map_err(|e| e.to_string())?;
        for file in &saved {
            if !Path::new(&file.path).is_file() {
                return Err(format!("附件已不可用：{}，请编辑或删除这条消息", file.name));
            }
        }
        send_prepared(
            app.clone(),
            session.into(),
            entry.message.text(),
            Some(images),
            None,
            None,
            Some(entry),
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        if let Ok(history) =
            JsonlStore::new(Dirs::of(app).map(|d| d.sessions()).unwrap_or_default()).load(session)
        {
            let _ = with_queue(app, |q| q.reconcile(session, &history));
        }
        let _ = pause(app, session, &error);
    }
    let _ = publish(app, session);
}
pub(super) fn finished(app: &tauri::AppHandle, session: &str, success: bool) {
    let result = (|| -> Result<(), String> {
        let history = JsonlStore::new(Dirs::of(app)?.sessions())
            .load(session)
            .map_err(|e| e.to_string())?;
        with_queue(app, |q| q.reconcile(session, &history))?;
        with_queue(app, |q| q.finish(session, success))?;
        publish(app, session)?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("无法保存消息队列状态：{error}");
        return;
    }
    kick(app, session);
}
pub(super) fn forget(app: &tauri::AppHandle, session: &str) -> Result<(), String> {
    with_queue(app, |q| q.delete(session))
}

struct Input {
    app: tauri::AppHandle,
}
impl std::fmt::Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DesktopInput")
    }
}
pub(super) fn input(app: &tauri::AppHandle) -> Arc<dyn agent_kernel::RunInput> {
    Arc::new(Input { app: app.clone() })
}
impl agent_kernel::RunInput for Input {
    fn poll(
        &self,
        session: &str,
        closing: bool,
    ) -> Result<Vec<Message>, agent_kernel::HarnessError> {
        let next = with_queue(&self.app, |q| q.claim(session, true, closing))
            .map_err(agent_kernel::HarnessError::Store)?;
        if let Some(entry) = next {
            if let Err(error) = validate(&self.app, session, &entry) {
                with_queue(&self.app, |q| q.release(session, &entry.id, &error))
                    .map_err(agent_kernel::HarnessError::Store)?;
                publish(&self.app, session).map_err(agent_kernel::HarnessError::Store)?;
                return Ok(vec![]);
            }
            publish(&self.app, session).map_err(agent_kernel::HarnessError::Store)?;
            Ok(vec![entry.message])
        } else {
            Ok(vec![])
        }
    }
    fn accepted(
        &self,
        session: &str,
        messages: &[Message],
    ) -> Result<(), agent_kernel::HarnessError> {
        let ids: Vec<_> = messages
            .iter()
            .filter_map(|m| {
                m.structured_content
                    .as_ref()?
                    .get("outboxId")?
                    .as_str()
                    .map(str::to_string)
            })
            .collect();
        with_queue(&self.app, |q| q.accepted(session, &ids))
            .map_err(agent_kernel::HarnessError::Store)?;
        publish(&self.app, session).map_err(agent_kernel::HarnessError::Store)?;
        Ok(())
    }
}

#[tauri::command]
pub fn chat_enqueue(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    session_id: String,
    text: String,
    mode: Delivery,
    images: Option<Vec<String>>,
    attachments: Option<Vec<AttachmentInput>>,
    mentions: Option<Vec<mentions::Mention>>,
    request_id: String,
    stop_now: Option<bool>,
) -> Result<Value, String> {
    chat_window(&window)?;
    checked_session_id(&session_id)?;
    let state = app.state::<ChatState>();
    let _delivery = lock(&state.delivery);
    let context = binding(&app, &session_id)?;
    if with_queue(&app, |q| q.known(&session_id, &request_id))? {
        let mut value = json!(publish(&app, &session_id)?);
        value["enqueuedId"] = json!(request_id);
        return Ok(value);
    }
    if !lock(&state.runs).contains_key(&session_id) && snapshot(&app, &session_id)?.items.is_empty()
    {
        with_queue(&app, |q| q.resume(&session_id, context.clone()))?;
    }
    let text = text.trim().to_string();
    let images = images.unwrap_or_default();
    let mut inputs = attachments.unwrap_or_default();
    if text.chars().count() > MAX_INPUT_CHARS {
        return Err("消息不能超过 30000 字".into());
    }
    if text.is_empty() && images.is_empty() && inputs.is_empty() {
        return Err("请先输入内容".into());
    }
    if images.len() + inputs.len() > MAX_IMAGES {
        return Err("一次最多发送 8 个附件".into());
    }
    let images = images
        .iter()
        .map(|s| clean_image(s))
        .collect::<Result<Vec<_>, _>>()?;
    for (n, image) in images.iter().enumerate() {
        let ext = if image.starts_with("data:image/jpeg;") {
            "jpg"
        } else if image.starts_with("data:image/webp;") {
            "webp"
        } else if image.starts_with("data:image/gif;") {
            "gif"
        } else {
            "png"
        };
        inputs.push(AttachmentInput {
            name: format!("图片-{}.{ext}", n + 1),
            data_url: image.clone(),
        });
    }
    let files = AttachmentStore::new(Dirs::of(&app)?.chat.join("attachments"));
    let saved = files.import(&session_id, &inputs)?;
    let mut message = Message::user(user_parts(&text, &images));
    message.structured_content = Some(json!({"attachments":saved}));
    let mentions = mentions
        .unwrap_or_default()
        .into_iter()
        .take(mentions::MAX_MENTIONS)
        .collect::<Vec<_>>();
    // Slash skills are bound at agent construction; keep those messages for the next run.
    let mode = if text.starts_with('/') || mentions.iter().any(|m| m.kind == "skill") {
        Delivery::Queue
    } else {
        mode
    };
    let id = match with_queue(&app, |q| {
        q.enqueue(
            &session_id,
            message,
            mode,
            context,
            json!(mentions),
            Some(request_id),
        )
    }) {
        Ok(id) => id,
        Err(e) => {
            files.rollback(&saved);
            return Err(e);
        }
    };
    if stop_now == Some(true) {
        with_queue(&app, |q| q.send_now(&session_id, &id))?;
        cancel_run(&app, &session_id);
    }
    kick(&app, &session_id);
    let mut value = json!(publish(&app, &session_id)?);
    value["enqueuedId"] = json!(id);
    Ok(value)
}

#[tauri::command]
pub fn chat_queue_update(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    session_id: String,
    action: String,
    id: Option<String>,
    text: Option<String>,
    order: Option<Vec<String>>,
    remove_images: Option<Vec<usize>>,
    remove_files: Option<Vec<String>>,
) -> Result<Snapshot, String> {
    chat_window(&window)?;
    checked_session_id(&session_id)?;
    let state = app.state::<ChatState>();
    let _delivery = lock(&state.delivery);
    match action.as_str() {
        "pause" => {
            with_queue(&app, |q| q.pause(&session_id, "队列已暂停"))?;
        }
        "resume" => {
            if !lock(&state.runs).contains_key(&session_id) {
                let history = JsonlStore::new(Dirs::of(&app)?.sessions())
                    .load(&session_id)
                    .map_err(|e| e.to_string())?;
                with_queue(&app, |q| q.reconcile(&session_id, &history))?;
            }
            let context = binding(&app, &session_id)?;
            with_queue(&app, |q| q.resume(&session_id, context))?;
            if lock(&state.runs).contains_key(&session_id) {
                open(&app, &session_id)?;
            }
        }
        "remove" | "edit" => {
            let id = id.ok_or("缺少消息编号")?;
            let old = with_queue(&app, |q| q.get(&session_id, &id))?;
            if action == "remove" {
                with_queue(&app, |q| q.remove(&session_id, &id))?;
            } else {
                with_queue(&app, |q| {
                    q.edit(
                        &session_id,
                        &id,
                        text.as_deref().unwrap_or(""),
                        &remove_images.unwrap_or_default(),
                        &remove_files.unwrap_or_default(),
                    )
                })?;
            }
            let keep = with_queue(&app, |q| q.get(&session_id, &id)).ok();
            let removed: Vec<Attachment> = serde_json::from_value::<Vec<Attachment>>(
                old.message
                    .structured_content
                    .as_ref()
                    .map(|v| v["attachments"].clone())
                    .unwrap_or(json!([])),
            )
            .unwrap_or_default()
            .into_iter()
            .filter(|a| {
                !keep.as_ref().is_some_and(|e| {
                    e.message
                        .structured_content
                        .as_ref()
                        .and_then(|v| v["attachments"].as_array())
                        .is_some_and(|files| files.iter().any(|f| f["id"] == a.id))
                })
            })
            .collect();
            AttachmentStore::new(Dirs::of(&app)?.chat.join("attachments")).rollback(&removed);
        }
        "reorder" => {
            with_queue(&app, |q| q.reorder(&session_id, &order.unwrap_or_default()))?;
        }
        "send_now" => {
            with_queue(&app, |q| {
                q.send_now(&session_id, &id.ok_or("缺少消息编号")?)
            })?;
            cancel_run(&app, &session_id);
        }
        _ => return Err("未知的队列操作".into()),
    }
    kick(&app, &session_id);
    publish(&app, &session_id)
}

#[tauri::command]
pub fn chat_queue_preview(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    session_id: String,
    id: String,
) -> Result<Option<String>, String> {
    chat_window(&window)?;
    checked_session_id(&session_id)?;
    let entry = with_queue(&app, |q| q.get(&session_id, &id))?;
    Ok(entry.message.parts.into_iter().find_map(|p| {
        if let Part::Image { url } = p {
            Some(url)
        } else {
            None
        }
    }))
}

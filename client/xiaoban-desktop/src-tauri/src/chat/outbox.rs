//! Durable, session-scoped user input. Payloads never enter history before delivery.
use agent_kernel::{Message, Part};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static IDS: AtomicU64 = AtomicU64::new(0);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Delivery {
    Queue,
    Steer,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub message: Message,
    pub mode: Delivery,
    pub binding: Value,
    pub mentions: Value,
    #[serde(default)]
    pub dispatching: bool,
}
impl Entry {
    /// Carry message-specific selections across an active run's mailbox, including older saved entries.
    pub fn into_message(mut self) -> Message {
        let metadata = self.message.structured_content.get_or_insert_with(|| json!({}));
        metadata["mentions"] = self.mentions;
        self.message
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub revision: u64,
    pub paused: bool,
    pub reason: String,
    pub items: Vec<Value>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    revision: u64,
    paused: bool,
    reason: String,
    entries: Vec<Entry>,
    #[serde(default)]
    send_next: Option<String>,
    #[serde(default)]
    delivered: Vec<String>,
}
#[derive(Debug)]
pub struct Outbox {
    root: PathBuf,
    seen: HashSet<String>,
    accepting: HashSet<String>,
}
impl Outbox {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            seen: HashSet::new(),
            accepting: HashSet::new(),
        }
    }
    fn change<T>(
        &mut self,
        session: &str,
        edit: impl FnOnce(&mut State) -> Result<T, String>,
    ) -> Result<T, String> {
        if session.is_empty()
            || session.len() > 180
            || !session
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("对话编号不对".into());
        }
        let _lease =
            agent_kernel::supervisor::SessionLease::acquire(self.root.join("locks"), session)?;
        let path = self.root.join(format!("{session}.json"));
        let mut state: State = match fs::read(&path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|e| format!("无法读取待发送队列：{e}"))?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
            Err(e) => return Err(format!("无法读取待发送队列：{e}")),
        };
        let original = serde_json::to_vec(&state).map_err(|e| e.to_string())?;
        if !self.seen.contains(session) && !state.entries.is_empty() {
            state.send_next = None;
            state.paused = true;
            state.reason = "上次队列尚未完成，请检查进度后继续发送".into();
            for entry in &mut state.entries {
                entry.dispatching = false;
            }
        }
        let result = edit(&mut state)?;
        if original == serde_json::to_vec(&state).map_err(|e| e.to_string())? {
            self.seen.insert(session.into());
            return Ok(result);
        }
        state.revision = state.revision.saturating_add(1);
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let temp = path.with_extension("tmp");
        let mut file = fs::File::create(&temp).map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec(&state).map_err(|e| e.to_string())?)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        fs::rename(&temp, &path).map_err(|e| format!("无法保存待发送队列：{e}"))?;
        self.seen.insert(session.into());
        Ok(result)
    }
    pub fn snapshot(&mut self, session: &str) -> Result<Snapshot, String> {
        self.change(session, |_| Ok(()))?;
        self.change(session, |s| Ok(Snapshot { revision: s.revision, paused: s.paused, reason: s.reason.clone(), items: s.entries.iter().map(|e| json!({
            "id": e.id, "text": e.message.text(), "mode": e.mode, "dispatching": e.dispatching,
            "images": e.message.parts.iter().filter(|p| matches!(p, Part::Image{..})).count(),
            "files": e.message.structured_content.as_ref().and_then(|v| v.get("attachments")).cloned().unwrap_or(json!([])),
        })).collect() }))
    }
    pub fn open(&mut self, session: &str) {
        self.accepting.insert(session.into());
    }
    pub fn enqueue(
        &mut self,
        session: &str,
        mut message: Message,
        mut mode: Delivery,
        binding: Value,
        mentions: Value,
        request_id: Option<String>,
    ) -> Result<String, String> {
        if mode == Delivery::Steer && !self.accepting.contains(session) {
            mode = Delivery::Queue;
        }
        let id = request_id.unwrap_or_else(|| {
            format!(
                "q-{:x}-{:x}-{:x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
                std::process::id(),
                IDS.fetch_add(1, Ordering::Relaxed)
            )
        });
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("消息编号不正确".into());
        }
        let metadata = message.structured_content.get_or_insert_with(|| json!({}));
        metadata["outboxId"] = json!(id);
        metadata["delivery"] = json!(mode);
        self.change(session, |s| {
            if s.delivered.contains(&id) || s.entries.iter().any(|e| e.id == id) {
                return Ok(id.clone());
            }
            if s.entries.len() >= 20 {
                return Err("最多排队 20 条消息，请先处理或删除部分消息".into());
            }
            let size = |m: &Message| {
                serde_json::to_vec(m)
                    .map_or(usize::MAX, |b| b.len())
                    .saturating_add(
                        m.structured_content
                            .as_ref()
                            .and_then(|v| v["attachments"].as_array())
                            .map_or(0, |a| {
                                a.iter()
                                    .map(|v| v["size"].as_u64().unwrap_or(0) as usize)
                                    .sum::<usize>()
                            }),
                    )
            };
            if s.entries
                .iter()
                .map(|e| size(&e.message))
                .sum::<usize>()
                .saturating_add(size(&message))
                > 100 * 1024 * 1024
            {
                return Err("待发送附件合计不能超过 100MB，请先处理部分消息".into());
            }
            s.entries.push(Entry {
                id: id.clone(),
                message,
                mode,
                binding,
                mentions,
                dispatching: false,
            });
            Ok(id)
        })
    }
    pub fn pause(&mut self, session: &str, reason: &str) -> Result<(), String> {
        self.accepting.remove(session);
        self.change(session, |s| {
            s.paused = true;
            s.reason = reason.into();
            s.send_next = None;
            Ok(())
        })
    }
    pub fn release(&mut self, session: &str, id: &str, reason: &str) -> Result<(), String> {
        self.accepting.remove(session);
        self.change(session, |s| {
            if let Some(e) = s.entries.iter_mut().find(|e| e.id == id) {
                e.dispatching = false;
            }
            s.paused = true;
            s.reason = reason.into();
            Ok(())
        })
    }
    pub fn finish(&mut self, session: &str, success: bool) -> Result<(), String> {
        self.accepting.remove(session);
        self.change(session, |s| {
            if !success {
                s.paused = true;
                s.reason = "任务已停止或失败，队列已暂停".into();
            }
            Ok(())
        })
    }
    pub fn resume(&mut self, session: &str, binding: Value) -> Result<(), String> {
        self.change(session, |s| {
            s.paused = false;
            s.reason.clear();
            for e in &mut s.entries {
                e.binding = binding.clone();
            }
            Ok(())
        })
    }
    pub fn claim(
        &mut self,
        session: &str,
        steer: bool,
        closing: bool,
    ) -> Result<Option<Entry>, String> {
        if steer && !self.accepting.contains(session) {
            return Ok(None);
        }
        let result = self.change(session, |s| {
            if (s.paused && (steer || s.send_next.is_none()))
                || s.entries.iter().any(|e| e.dispatching)
            {
                return Ok(None);
            }
            let next = if steer {
                s.entries.iter_mut().find(|e| e.mode == Delivery::Steer)
            } else if let Some(id) = s.send_next.take() {
                s.entries.iter_mut().find(|e| e.id == id)
            } else {
                s.entries.first_mut()
            };
            if let Some(e) = next {
                e.dispatching = true;
                return Ok(Some(e.clone()));
            }
            Ok(None)
        })?;
        if closing && result.is_none() {
            self.accepting.remove(session);
        }
        Ok(result)
    }
    pub fn accepted(&mut self, session: &str, ids: &[String]) -> Result<(), String> {
        self.change(session, |s| {
            s.entries.retain(|e| !ids.contains(&e.id));
            for id in ids {
                if !s.delivered.contains(id) {
                    s.delivered.push(id.clone());
                }
            }
            if s.delivered.len() > 256 {
                s.delivered.drain(..s.delivered.len() - 256);
            }
            Ok(())
        })
    }
    /// Reconcile append-before-ack crashes; never replay a question already saved in history.
    pub fn reconcile(&mut self, session: &str, history: &[Message]) -> Result<(), String> {
        let ids: Vec<_> = history
            .iter()
            .filter_map(|m| {
                m.structured_content
                    .as_ref()?
                    .get("outboxId")?
                    .as_str()
                    .map(str::to_string)
            })
            .collect();
        self.accepted(session, &ids)?;
        self.change(session, |s| {
            for e in &mut s.entries {
                e.dispatching = false;
            }
            Ok(())
        })
    }
    pub fn edit(
        &mut self,
        session: &str,
        id: &str,
        text: &str,
        remove_images: &[usize],
        remove_files: &[String],
    ) -> Result<(), String> {
        if text.chars().count() > 30_000 {
            return Err("消息不能超过 30000 字".into());
        }
        self.change(session, |s| {
            let e = s
                .entries
                .iter_mut()
                .find(|e| e.id == id)
                .ok_or("这条消息已被接收，请刷新队列")?;
            if e.dispatching {
                return Err("这条消息正在发送，暂时无法编辑".into());
            }
            let mut image = 0;
            e.message.parts.retain(|p| {
                if matches!(p, Part::Image { .. }) {
                    let keep = !remove_images.contains(&image);
                    image += 1;
                    keep
                } else {
                    false
                }
            });
            if !text.trim().is_empty() {
                e.message.parts.insert(
                    0,
                    Part::Text {
                        text: text.trim().into(),
                    },
                );
            }
            if let Some(files) = e
                .message
                .structured_content
                .as_mut()
                .and_then(|v| v["attachments"].as_array_mut())
            {
                let mut image = 0;
                files.retain(|v| {
                    let keep = if v["mime"]
                        .as_str()
                        .is_some_and(|mime| mime.starts_with("image/"))
                    {
                        let keep = !remove_images.contains(&image);
                        image += 1;
                        keep
                    } else {
                        true
                    };
                    keep && !remove_files.iter().any(|id| v["id"] == *id)
                });
            }
            if e.message.parts.is_empty()
                && e.message
                    .structured_content
                    .as_ref()
                    .and_then(|v| v["attachments"].as_array())
                    .is_none_or(Vec::is_empty)
            {
                return Err("消息内容不能为空".into());
            }
            Ok(())
        })
    }
    pub fn remove(&mut self, session: &str, id: &str) -> Result<(), String> {
        self.change(session, |s| {
            if s.entries.iter().any(|e| e.id == id && e.dispatching) {
                return Err("这条消息正在发送，暂时无法删除".into());
            }
            s.entries.retain(|e| e.id != id);
            Ok(())
        })
    }
    pub fn reorder(&mut self, session: &str, ids: &[String]) -> Result<(), String> {
        self.change(session, |s| {
            if s.entries.iter().any(|e| e.dispatching) {
                return Err("有消息正在发送，请稍后排序".into());
            }
            let unique: HashSet<_> = ids.iter().collect();
            if ids.len() != s.entries.len()
                || unique.len() != ids.len()
                || s.entries.iter().any(|e| !unique.contains(&e.id))
            {
                return Err("队列已变化，请刷新后重新排序".into());
            }
            s.entries
                .sort_by_key(|e| ids.iter().position(|id| id == &e.id).unwrap());
            Ok(())
        })
    }
    pub fn known(&mut self, session: &str, id: &str) -> Result<bool, String> {
        self.change(session, |s| {
            Ok(s.delivered.iter().any(|i| i == id) || s.entries.iter().any(|e| e.id == id))
        })
    }
    pub fn get(&mut self, session: &str, id: &str) -> Result<Entry, String> {
        self.change(session, |s| {
            s.entries
                .iter()
                .find(|e| e.id == id)
                .cloned()
                .ok_or("这条消息已被接收".into())
        })
    }
    pub fn send_now(&mut self, session: &str, id: &str) -> Result<(), String> {
        self.change(session, |s| {
            if !s.entries.iter().any(|e| e.id == id && !e.dispatching) {
                return Err("这条消息已被接收".into());
            }
            s.send_next = Some(id.into());
            s.paused = true;
            s.reason = "其余排队消息已暂停".into();
            Ok(())
        })
    }
    pub fn delete(&mut self, session: &str) -> Result<(), String> {
        self.change(session, |s| {
            s.entries.clear();
            s.paused = true;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delivery_adds_selections_to_legacy_entries_without_losing_message_metadata() {
        let entry: Entry = serde_json::from_value(json!({
            "id":"q1", "mode":"steer", "binding":{},
            "message": {"role":"user","parts":[{"type":"text","text":"change"}],
                "structured_content":{"outboxId":"q1","attachments":[{"id":"file"}]}},
            "mentions":[{"kind":"connector","id":"service"}]
        })).unwrap();
        let message = entry.into_message();
        let metadata = message.structured_content.unwrap();
        assert_eq!(metadata["outboxId"], "q1");
        assert_eq!(metadata["attachments"][0]["id"], "file");
        assert_eq!(metadata["mentions"][0]["id"], "service");
    }
    fn temp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "outbox-test-{}-{}",
            std::process::id(),
            IDS.fetch_add(1, Ordering::Relaxed)
        ))
    }
    #[test]
    fn fifo_reorder_isolation_and_restart_pause() {
        let root = temp();
        let mut q = Outbox::new(root.clone());
        let a = q
            .enqueue(
                "s1",
                Message::user_text("a"),
                Delivery::Queue,
                json!({}),
                json!([]),
                None,
            )
            .unwrap();
        let b = q
            .enqueue(
                "s1",
                Message::user_text("b"),
                Delivery::Queue,
                json!({}),
                json!([]),
                None,
            )
            .unwrap();
        q.enqueue(
            "s2",
            Message::user_text("other"),
            Delivery::Queue,
            json!({}),
            json!([]),
            None,
        )
        .unwrap();
        q.reorder("s1", &[b.clone(), a]).unwrap();
        assert_eq!(q.claim("s1", false, false).unwrap().unwrap().id, b);
        assert!(q.claim("s1", false, false).unwrap().is_none());
        let mut restarted = Outbox::new(root.clone());
        assert!(restarted.snapshot("s1").unwrap().paused);
        restarted
            .reconcile(
                "s1",
                &[Message {
                    structured_content: Some(json!({"outboxId":b})),
                    ..Message::user_text("b")
                }],
            )
            .unwrap();
        restarted.resume("s1", json!({})).unwrap();
        assert_eq!(
            restarted
                .claim("s1", false, false)
                .unwrap()
                .unwrap()
                .message
                .text(),
            "a"
        );
        assert_eq!(q.snapshot("s2").unwrap().items.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn stop_send_is_explicit_and_a_later_stop_cancels_it() {
        let root = temp();
        let mut q = Outbox::new(root.clone());
        let id = q
            .enqueue(
                "s",
                Message::user_text("next"),
                Delivery::Queue,
                json!({}),
                json!([]),
                Some("q-once".into()),
            )
            .unwrap();
        q.send_now("s", &id).unwrap();
        q.finish("s", false).unwrap();
        assert!(q.claim("s", false, false).unwrap().is_some());
        q.accepted("s", &[id.clone()]).unwrap();
        q.enqueue(
            "s",
            Message::user_text("next"),
            Delivery::Queue,
            json!({}),
            json!([]),
            Some(id),
        )
        .unwrap();
        assert!(q.snapshot("s").unwrap().items.is_empty());
        let next = q
            .enqueue(
                "s",
                Message::user_text("later"),
                Delivery::Queue,
                json!({}),
                json!([]),
                None,
            )
            .unwrap();
        q.send_now("s", &next).unwrap();
        q.pause("s", "stop again").unwrap();
        assert!(q.claim("s", false, false).unwrap().is_none());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn removing_an_image_also_removes_its_file_manifest() {
        let root = temp();
        let mut q = Outbox::new(root.clone());
        let mut msg = Message::user(vec![
            Part::Text {
                text: "question".into(),
            },
            Part::Image {
                url: "data:image/png;base64,YQ==".into(),
            },
        ]);
        msg.structured_content = Some(
            json!({"attachments":[{"id":"image","name":"image.png","size":1,"mime":"image/png"},{"id":"doc","name":"report.txt","size":1,"mime":"text/plain"}]}),
        );
        let id = q
            .enqueue("s", msg, Delivery::Queue, json!({}), json!([]), None)
            .unwrap();
        q.edit("s", &id, "updated", &[0], &[]).unwrap();
        let e = q.get("s", &id).unwrap();
        assert_eq!(e.message.parts.len(), 1);
        assert_eq!(
            e.message.structured_content.unwrap()["attachments"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn steering_seals_and_failed_edits_leave_original_intact() {
        let root = temp();
        let mut q = Outbox::new(root.clone());
        q.open("s");
        let id = q
            .enqueue(
                "s",
                Message::user_text("change"),
                Delivery::Steer,
                json!({}),
                json!([]),
                None,
            )
            .unwrap();
        assert!(q.edit("s", &id, "", &[], &[]).is_err());
        assert_eq!(
            q.claim("s", true, true).unwrap().unwrap().message.text(),
            "change"
        );
        assert!(q.remove("s", &id).is_err());
        q.accepted("s", &[id]).unwrap();
        assert!(q.claim("s", true, true).unwrap().is_none());
        q.enqueue(
            "s",
            Message::user_text("late"),
            Delivery::Steer,
            json!({}),
            json!([]),
            None,
        )
        .unwrap();
        assert_eq!(q.snapshot("s").unwrap().items[0]["mode"], "queue");
        q.pause("s", "stopped").unwrap();
        assert!(q.claim("s", false, false).unwrap().is_none());
        fs::remove_dir_all(root).unwrap();
    }
}

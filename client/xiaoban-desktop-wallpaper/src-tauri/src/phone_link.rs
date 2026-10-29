//! 桌面端手机互联接入层（L2）：
//! - 同一个小伴账号登录的手机和电脑自动互相发现，没有配对、没有密钥；本地只存这台电脑的设备编号（`config/phone-link-state.json`）
//! - “允许手机遥控”开关（默认关闭），开启时启动 `xiaoban-link-client`；关闭时断开并不再响应任何手机消息
//! - 会话桥：`sessions.list` / `sessions.page` / `sessions.changed`、`session.open` / `session.snapshot` / `session.close`、`message.send` / `message.accepted`
//! - 流式事件旁路：将桌面 `chat-event` / `chat-run-state` 映射为互联协议的 `event`
//! - 审批桥：`chat-approval` / `chat-question` ↔ `approval.request` / `approval.decision` / `approval.resolved`
//! - 远程来源最严格权限策略：手机发来的消息强制将对话设为 `ask` 权限并要求逐项确认，且每条远程操作写入 `chat/remote-audit.jsonl`

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager};
use crate::phone_voice;
use crate::phone_wire;
use xiaoban_link_client::{store::{FileStore, LinkStore}, Link, LinkConfig, LinkEvent, TokenFn};

const DEFAULT_RELAY_URL: &str = "https://link.xiangban.xyz/api/link";

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn default_relay_url() -> String {
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var("XIAOBAN_LINK_URL") {
        let trimmed = url.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    DEFAULT_RELAY_URL.to_string()
}

fn default_device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "小伴电脑".to_string())
}

// ───────────── 配置与运行态 ─────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PhoneLinkPrefs {
    pub enabled: bool,
    pub relay_url: String,
    pub device_name: String,
}

impl Default for PhoneLinkPrefs {
    fn default() -> Self {
        Self {
            enabled: false,
            relay_url: default_relay_url(),
            device_name: default_device_name(),
        }
    }
}

fn prefs_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config").join("phone-link.json"))
}

fn state_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("config").join("phone-link-state.json"))
}

fn audit_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("chat").join("remote-audit.jsonl"))
}

pub fn load_prefs(app: &tauri::AppHandle) -> PhoneLinkPrefs {
    let Ok(path) = prefs_path(app) else { return PhoneLinkPrefs::default() };
    let Ok(bytes) = fs::read(&path) else { return PhoneLinkPrefs::default() };
    let mut prefs: PhoneLinkPrefs = serde_json::from_slice(&bytes).unwrap_or_default();
    if prefs.relay_url.trim().is_empty() {
        prefs.relay_url = default_relay_url();
    }
    if prefs.device_name.trim().is_empty() {
        prefs.device_name = default_device_name();
    }
    prefs
}

fn save_prefs(app: &tauri::AppHandle, prefs: &PhoneLinkPrefs) -> Result<(), String> {
    let path = prefs_path(app)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

/// 记录每一条来自手机的远程操作（`message.send`、`approval.decision`），满足远程来源可审计要求。
pub fn append_remote_audit(root: &Path, entry: &Value) {
    if let Some(parent) = root.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(root) {
        if let Ok(line) = serde_json::to_string(entry) {
            let _ = writeln!(file, "{line}");
        }
    }
}

struct RuntimeInner {
    link: Option<Link>,
    connected: bool,
    last_error: Option<String>,
    /// 每个手机当前订阅的桌面会话编号集合。
    subscriptions: HashMap<String, HashSet<String>>,
    /// 当前桌面等待用户确认的审批/问题：`approval_id -> session_id`，用于发现桌面已处理后向手机广播 `approval.resolved`。
    open_approvals: HashMap<String, String>,
    /// 手机分片上传的录音，拼完交给语音识别。
    voice_in: phone_voice::Assembler,
    /// 要把回答念给手机听的对话：`会话编号 -> (手机, 这一轮最后一段回答的文字)`。
    speak: HashMap<String, (String, String)>,
    /// 合成语音的后台线程的入口；第一次有手机要听回答时才启动。
    speak_tx: Option<std::sync::mpsc::Sender<SpeakJob>>,
}

/// 一轮回答结束后要合成语音发给手机的内容。
struct SpeakJob {
    peer: String,
    session: String,
    text: String,
}

impl Default for RuntimeInner {
    fn default() -> Self {
        Self {
            link: None,
            connected: false,
            last_error: None,
            subscriptions: HashMap::new(),
            open_approvals: HashMap::new(),
            voice_in: Default::default(),
            speak: HashMap::new(),
            speak_tx: None,
        }
    }
}

pub struct PhoneLinkState {
    inner: Mutex<RuntimeInner>,
    seq: AtomicU64,
    /// 串行化 `ensure_running`：应用启动与用户操作同时触发时只起一个 `Link`。
    start_lock: tauri::async_runtime::Mutex<()>,
}

impl Default for PhoneLinkState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(RuntimeInner::default()),
            seq: AtomicU64::new(1),
            start_lock: tauri::async_runtime::Mutex::new(()),
        }
    }
}

impl PhoneLinkState {
    fn next_msg_id(&self) -> String {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        format!("d-{}-{n}", now_ms())
    }
}

fn emit_status(app: &tauri::AppHandle) {
    if let Ok(status) = status_value(app) {
        let _ = app.emit("phone-link-changed", status);
    }
}

pub fn status_value(app: &tauri::AppHandle) -> Result<Value, String> {
    let prefs = load_prefs(app);
    let logged_in = app
        .try_state::<crate::account::AccountState>()
        .is_some_and(|s| s.session.lock().unwrap_or_else(|e| e.into_inner()).is_some());
    let state = app.state::<PhoneLinkState>();
    let guard = lock(&state.inner);
    let (device_id, peers) = if let Some(link) = &guard.link {
        let list: Vec<Value> = link
            .peers()
            .into_iter()
            .map(|p| json!({ "id": p.id, "name": p.name, "online": p.online }))
            .collect();
        (link.device_id(), list)
    } else {
        // 没连接时没有手机列表（列表来自中转），只展示本机的设备编号。
        let saved = FileStore::new(state_path(app)?).load().unwrap_or_default();
        (saved.device.map(|d| d.id), Vec::new())
    };
    Ok(json!({
        "enabled": prefs.enabled,
        "connected": guard.connected,
        "deviceId": device_id,
        "deviceName": prefs.device_name,
        "relayUrl": prefs.relay_url,
        "loggedIn": logged_in,
        "peers": peers,
        "lastError": guard.last_error,
    }))
}

/// 应用启动时调用：若用户此前开启了“允许手机遥控”，自动在后台启动互联连接。
pub fn start(app: &tauri::AppHandle) {
    if load_prefs(app).enabled {
        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = ensure_running(&handle).await;
        });
    }
}

async fn ensure_running(app: &tauri::AppHandle) -> Result<Link, String> {
    let runtime = app.state::<PhoneLinkState>();
    let _starting = runtime.start_lock.lock().await;
    {
        let state = app.state::<PhoneLinkState>();
        let guard = lock(&state.inner);
        if let Some(link) = &guard.link {
            return Ok(link.clone());
        }
    }
    let prefs = load_prefs(app);
    let store = Arc::new(FileStore::new(state_path(app)?));
    let app_for_token = app.clone();
    let token_fn: TokenFn = Arc::new(move || {
        let handle = app_for_token.clone();
        Box::pin(async move { crate::account::fresh_access_token(&handle).await.ok() })
    });
    let cfg = LinkConfig::new(prefs.relay_url.clone(), prefs.device_name.clone());
    let (link, mut events) = Link::start(cfg, store, token_fn).await.map_err(|e| e.to_string())?;
    {
        let state = app.state::<PhoneLinkState>();
        let mut guard = lock(&state.inner);
        if let Some(existing) = &guard.link {
            link.shutdown();
            return Ok(existing.clone());
        }
        guard.link = Some(link.clone());
        guard.last_error = None;
    }
    emit_status(app);

    let handle = app.clone();
    let link_for_loop = link.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = events.recv().await {
            handle_link_event(&handle, &link_for_loop, event);
        }
    });
    Ok(link)
}

fn stop_running(app: &tauri::AppHandle) {
    let state = app.state::<PhoneLinkState>();
    let old = {
        let mut guard = lock(&state.inner);
        guard.connected = false;
        guard.subscriptions.clear();
        guard.link.take()
    };
    if let Some(link) = old {
        link.shutdown();
    }
    emit_status(app);
}

/// 序列化并发送；超过协议单信封上限的事件会被换成一条简短的错误事件，其他超限消息返回错误让调用方处理。
fn send_json(link: &Link, peer: &str, msg: &Value, notify: bool) -> Result<(), String> {
    let bytes = match phone_wire::fit_for_wire(msg) {
        Ok(bytes) => bytes,
        Err(Some(replacement)) => serde_json::to_vec(&replacement).map_err(|e| e.to_string())?,
        Err(None) => return Err("消息太大，无法发送".into()),
    };
    link.send(peer, &bytes, notify).map_err(|e| e.to_string())
}

fn broadcast_json(link: &Link, msg: &Value, notify: bool) {
    for peer in link.peers() {
        if peer.online || notify {
            let _ = send_json(link, &peer.id, msg, notify);
        }
    }
}

fn send_hello(app: &tauri::AppHandle, link: &Link, peer: &str) {
    let id = app.state::<PhoneLinkState>().next_msg_id();
    let _ = send_json(
        link,
        peer,
        &json!({
            "t": "hello",
            "id": id,
            "v": 1,
            "caps": ["sessions", "stream", "approvals", "snapshot_paging", "projects"],
            "app": env!("CARGO_PKG_VERSION"),
        }),
        false,
    );
}

fn handle_link_event(app: &tauri::AppHandle, link: &Link, event: LinkEvent) {
    let state = app.state::<PhoneLinkState>();
    match event {
        LinkEvent::Connected => {
            lock(&state.inner).connected = true;
            lock(&state.inner).last_error = None;
            emit_status(app);
        }
        LinkEvent::Disconnected => {
            lock(&state.inner).connected = false;
            emit_status(app);
        }
        LinkEvent::PeerOnline { peer, online } => {
            if !online {
                lock(&state.inner).subscriptions.remove(&peer);
            } else {
                send_hello(app, link, &peer);
            }
            emit_status(app);
        }
        LinkEvent::PeerAdded(peer) => {
            if peer.online {
                send_hello(app, link, &peer.id);
            }
            emit_status(app);
        }
        LinkEvent::PeerRemoved { peer } => {
            lock(&state.inner).subscriptions.remove(&peer);
            emit_status(app);
        }
        LinkEvent::Message { peer, data } => {
            if !load_prefs(app).enabled {
                return;
            }
            handle_peer_message(app, link, &peer, &data);
        }
        LinkEvent::Replaced => {
            lock(&state.inner).connected = false;
            lock(&state.inner).last_error = Some("当前设备连接已被另一处实例顶替".into());
            emit_status(app);
        }
        LinkEvent::Revoked => {
            lock(&state.inner).subscriptions.clear();
            lock(&state.inner).last_error = Some("这台电脑被从账号的设备列表里移除了，正在重新登记".into());
            emit_status(app);
        }
        LinkEvent::ServerError { code } => {
            lock(&state.inner).last_error = Some(format!("中转服务提示：{code}"));
            emit_status(app);
        }
        LinkEvent::Undelivered { .. } => {}
    }
}

/// 纯函数式协议分发（供单元测试直接验证消息路由与 `unsupported` 回包）。
pub fn classify_peer_message(raw: &[u8]) -> Result<(String, String, Value), String> {
    let value: Value = serde_json::from_slice(raw).map_err(|_| "不是有效的 JSON".to_string())?;
    let t = value.get("t").and_then(Value::as_str).unwrap_or("").to_string();
    let id = value.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    if t.is_empty() {
        return Err("缺少消息类型 t".into());
    }
    Ok((t, id, value))
}

fn handle_peer_message(app: &tauri::AppHandle, link: &Link, peer: &str, raw: &[u8]) {
    let Ok((t, req_id, msg)) = classify_peer_message(raw) else { return };
    let state = app.state::<PhoneLinkState>();
    match t.as_str() {
        "hello" => {
            let v = msg.get("v").and_then(Value::as_u64).unwrap_or(1);
            if v != 1 {
                let _ = send_json(
                    link,
                    peer,
                    &json!({ "t": "unsupported", "id": state.next_msg_id(), "ref": req_id, "reason": "version_mismatch" }),
                    false,
                );
            }
        }
        "sessions.list" => {
            let limit = msg.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
            let cursor = msg.get("cursor").and_then(Value::as_str);
            if let Ok(mut page) = crate::chat::remote_sessions_list(app, limit, cursor) {
                page["t"] = json!("sessions.page");
                page["id"] = json!(state.next_msg_id());
                page["ref"] = json!(req_id);
                let _ = send_json(link, peer, &page, false);
            }
        }
        "projects.list" => {
            if let Ok(mut page) = crate::chat::remote_projects(app) {
                page["t"] = json!("projects.page");
                page["id"] = json!(state.next_msg_id());
                page["ref"] = json!(req_id);
                let _ = send_json(link, peer, &page, false);
            }
        }
        "session.open" => {
            let Some(session_id) = msg.get("session").and_then(Value::as_str) else { return };
            let after = msg.get("after").and_then(Value::as_u64).map(|n| n as usize);
            lock(&state.inner)
                .subscriptions
                .entry(peer.to_string())
                .or_default()
                .insert(session_id.to_string());
            let failure = match crate::chat::remote_session_snapshot(app, session_id, after) {
                Ok(mut snap) => {
                    snap["t"] = json!("session.snapshot");
                    snap["id"] = json!(state.next_msg_id());
                    snap["ref"] = json!(req_id);
                    send_json(link, peer, &snap, false).err()
                }
                Err(error) => Some(error),
            };
            if let Some(message) = failure {
                // 不能让手机一直转圈：同步失败要明确告诉它。
                let _ = send_json(
                    link,
                    peer,
                    &json!({
                        "t": "event",
                        "id": state.next_msg_id(),
                        "session": session_id,
                        "kind": "error",
                        "data": { "message": format!("同步会话失败：{message}") },
                    }),
                    false,
                );
            }
        }
        "session.permission" => {
            let Some(session_id) = msg.get("session").and_then(Value::as_str) else { return };
            let mode = msg.get("mode").and_then(Value::as_str).unwrap_or("");
            if let Ok(audit) = audit_path(app) {
                append_remote_audit(
                    &audit,
                    &json!({
                        "timestampMs": now_ms(),
                        "peerId": peer,
                        "action": "session.permission",
                        "session": session_id,
                        "mode": mode,
                    }),
                );
            }
            let result = crate::chat::remote_set_permission(app, session_id, mode);
            let mut reply = json!({
                "t": "session.permission",
                "id": state.next_msg_id(),
                "ref": req_id,
                "session": session_id,
            });
            match result {
                Ok(applied) => {
                    reply["ok"] = json!(true);
                    reply["mode"] = json!(applied);
                }
                Err(error) => {
                    reply["ok"] = json!(false);
                    reply["error"] = json!(error);
                }
            }
            let _ = send_json(link, peer, &reply, false);
        }
        "session.close" => {
            if let Some(session_id) = msg.get("session").and_then(Value::as_str) {
                if let Some(set) = lock(&state.inner).subscriptions.get_mut(peer) {
                    set.remove(session_id);
                }
            }
        }
        "voice.chunk" => {
            use base64::Engine as _;
            let (Some(voice_id), Some(data)) = (msg.get("voice_id").and_then(Value::as_str), msg.get("data").and_then(Value::as_str)) else { return };
            let seq = msg.get("seq").and_then(Value::as_u64).unwrap_or(0) as usize;
            let last = msg.get("last").and_then(Value::as_bool).unwrap_or(false);
            let fail = |error: &str| {
                let _ = send_json(link, peer, &json!({ "t": "voice.text", "id": state.next_msg_id(), "voice_id": voice_id, "ok": false, "error": error }), false);
            };
            let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data) else { return fail("录音数据无法解码") };
            let key = format!("{peer}/{voice_id}");
            let pushed = lock(&state.inner).voice_in.push(&key, seq, last, &bytes);
            match pushed {
                Err(error) => fail(&error),
                Ok(None) => {}
                Ok(Some(pcm)) => {
                    if let Ok(audit) = audit_path(app) {
                        append_remote_audit(&audit, &json!({ "timestampMs": now_ms(), "peerId": peer, "action": "voice.input", "bytes": pcm.len() }));
                    }
                    let (app, link, peer, voice_id) = (app.clone(), link.clone(), peer.to_string(), voice_id.to_string());
                    tauri::async_runtime::spawn(async move {
                        let result = match phone_voice::pcm16_to_f32(&pcm) {
                            Ok(samples) => crate::asr::asr_transcribe(app.clone(), samples).await,
                            Err(error) => Err(error),
                        };
                        let state = app.state::<PhoneLinkState>();
                        let mut reply = json!({ "t": "voice.text", "id": state.next_msg_id(), "voice_id": voice_id });
                        match result {
                            Ok(text) => {
                                reply["ok"] = json!(true);
                                reply["text"] = json!(text.trim());
                            }
                            Err(error) => {
                                reply["ok"] = json!(false);
                                reply["error"] = json!(voice_error_text(&error));
                            }
                        }
                        let _ = send_json(&link, &peer, &reply, false);
                    });
                }
            }
        }
        "run.undo" => {
            let (Some(session_id), Some(run_id)) = (msg.get("session").and_then(Value::as_str), msg.get("run_id").and_then(Value::as_str)) else { return };
            if let Ok(audit) = audit_path(app) {
                append_remote_audit(&audit, &json!({ "timestampMs": now_ms(), "peerId": peer, "action": "run.undo", "session": session_id, "runId": run_id }));
            }
            let mut reply = json!({ "t": "run.undo", "id": state.next_msg_id(), "ref": req_id, "session": session_id, "run_id": run_id });
            match crate::chat::remote_undo_run(app, session_id, run_id) {
                Ok(report) => {
                    reply["ok"] = json!(true);
                    reply["restored"] = report["restored"].clone();
                    reply["removed"] = report["removed"].clone();
                    reply["failed"] = report["failed"].clone();
                }
                Err(error) => {
                    reply["ok"] = json!(false);
                    reply["error"] = json!(error);
                }
            }
            let _ = send_json(link, peer, &reply, false);
        }
        "models.list" => {
            let session_id = msg.get("session").and_then(Value::as_str);
            let mut reply = json!({ "t": "models.list", "id": state.next_msg_id(), "ref": req_id });
            match crate::chat::remote_models(app, session_id) {
                Ok(listed) => {
                    reply["ok"] = json!(true);
                    reply["models"] = listed["models"].clone();
                    reply["current"] = listed["current"].clone();
                }
                Err(error) => {
                    reply["ok"] = json!(false);
                    reply["error"] = json!(error);
                }
            }
            let _ = send_json(link, peer, &reply, false);
        }
        "session.model" => {
            let (Some(session_id), Some(provider), Some(model)) = (
                msg.get("session").and_then(Value::as_str),
                msg.get("provider").and_then(Value::as_str),
                msg.get("model").and_then(Value::as_str),
            ) else { return };
            if let Ok(audit) = audit_path(app) {
                append_remote_audit(&audit, &json!({ "timestampMs": now_ms(), "peerId": peer, "action": "session.model", "session": session_id, "provider": provider, "model": model }));
            }
            let mut reply = json!({ "t": "session.model", "id": state.next_msg_id(), "ref": req_id, "session": session_id, "provider": provider, "model": model });
            match crate::chat::remote_set_model(app, session_id, provider, model) {
                Ok(()) => reply["ok"] = json!(true),
                Err(error) => {
                    reply["ok"] = json!(false);
                    reply["error"] = json!(error);
                }
            }
            let _ = send_json(link, peer, &reply, false);
        }
        "library.save" => {
            let Some(text) = msg.get("text").and_then(Value::as_str) else { return };
            let title = msg.get("title").and_then(Value::as_str).unwrap_or("");
            let session_id = msg.get("session").and_then(Value::as_str).map(str::to_string);
            let session_title = msg.get("session_title").and_then(Value::as_str).unwrap_or("");
            if let Ok(audit) = audit_path(app) {
                append_remote_audit(&audit, &json!({ "timestampMs": now_ms(), "peerId": peer, "action": "library.save", "session": session_id }));
            }
            let mut reply = json!({ "t": "library.save", "id": state.next_msg_id(), "ref": req_id });
            match crate::data_library::remote_save(app, title, text, session_id, session_title) {
                Ok(id) => {
                    reply["ok"] = json!(true);
                    reply["item"] = json!(id);
                }
                Err(error) => {
                    reply["ok"] = json!(false);
                    reply["error"] = json!(error);
                }
            }
            let _ = send_json(link, peer, &reply, false);
        }
        "message.cancel" => {
            let Some(session_id) = msg.get("session").and_then(Value::as_str).filter(|s| !s.trim().is_empty()) else { return };
            if let Ok(audit) = audit_path(app) {
                append_remote_audit(
                    &audit,
                    &json!({
                        "timestampMs": now_ms(),
                        "peerId": peer,
                        "action": "message.cancel",
                        "session": session_id,
                    }),
                );
            }
            // 和电脑上点“停止”一样：让正在跑的这一轮尽快结束；没有在跑就什么也不发生。
            crate::chat::cancel_run(app, session_id);
        }
        "message.send" => {
            let session_id = msg.get("session").and_then(Value::as_str).filter(|s| !s.trim().is_empty());
            let text = msg.get("text").and_then(Value::as_str).unwrap_or("");
            let client_id = msg.get("client_id").and_then(Value::as_str).unwrap_or(&req_id);
            let workspace = msg.get("workspace").and_then(Value::as_str);
            let permission = msg.get("permission").and_then(Value::as_str);
            let speak = msg.get("speak").and_then(Value::as_bool).unwrap_or(false);
            if let Ok(audit) = audit_path(app) {
                append_remote_audit(
                    &audit,
                    &json!({
                        "timestampMs": now_ms(),
                        "peerId": peer,
                        "speak": speak,
                        "action": "message.send",
                        "session": session_id,
                        "clientId": client_id,
                        "chars": text.chars().count(),
                        "workspace": workspace,
                        "permission": permission,
                    }),
                );
            }
            match crate::chat::remote_send_message(app, peer, session_id, text, client_id, workspace, permission) {
                Ok(accepted) => {
                    if let Some(sid) = accepted.get("session").and_then(Value::as_str) {
                        lock(&state.inner)
                            .subscriptions
                            .entry(peer.to_string())
                            .or_default()
                            .insert(sid.to_string());
                        if speak {
                            ensure_speaker(app);
                            lock(&state.inner).speak.insert(sid.to_string(), (peer.to_string(), String::new()));
                        } else {
                            lock(&state.inner).speak.remove(sid);
                        }
                    }
                    let _ = send_json(
                        link,
                        peer,
                        &json!({
                            "t": "message.accepted",
                            "id": state.next_msg_id(),
                            "client_id": client_id,
                            "session": accepted["session"],
                            "message_id": accepted["message_id"],
                        }),
                        false,
                    );
                }
                Err(error) => {
                    let _ = send_json(
                        link,
                        peer,
                        &json!({
                            "t": "event",
                            "id": state.next_msg_id(),
                            "session": session_id.unwrap_or(""),
                            "kind": "error",
                            "data": { "message": error, "client_id": client_id },
                        }),
                        false,
                    );
                }
            }
        }
        "approval.decision" => {
            let Some(approval_id) = msg.get("approval_id").and_then(Value::as_str) else { return };
            let choice = msg.get("choice").and_then(Value::as_str).unwrap_or("deny");
            if let Ok(audit) = audit_path(app) {
                append_remote_audit(
                    &audit,
                    &json!({
                        "timestampMs": now_ms(),
                        "peerId": peer,
                        "action": "approval.decision",
                        "approvalId": approval_id,
                        "choice": choice,
                    }),
                );
            }
            match crate::chat::remote_answer_approval(app, approval_id, choice) {
                Ok(()) => {
                    lock(&state.inner).open_approvals.remove(approval_id);
                    broadcast_json(
                        link,
                        &json!({
                            "t": "approval.resolved",
                            "id": state.next_msg_id(),
                            "approval_id": approval_id,
                            "by": "phone",
                        }),
                        false,
                    );
                }
                Err(reason) => {
                    // 审批已被桌面处理、已过期，或需要在电脑上处理：告诉这台手机收起卡片，不要让它一直挂着。
                    lock(&state.inner).open_approvals.remove(approval_id);
                    let _ = send_json(
                        link,
                        peer,
                        &json!({
                            "t": "approval.resolved",
                            "id": state.next_msg_id(),
                            "approval_id": approval_id,
                            "by": "expired",
                            "reason": reason,
                        }),
                        false,
                    );
                }
            }
        }
        _ => {
            let _ = send_json(
                link,
                peer,
                &json!({
                    "t": "unsupported",
                    "id": state.next_msg_id(),
                    "ref": req_id,
                    "t_unknown": t,
                }),
                false,
            );
        }
    }
}

// ───────────── 桌面聊天与审批事件旁路 ─────────────

pub fn notify_approval_resolved(app: &tauri::AppHandle, approval_id: &str, by: &str) {
    let Some(state) = app.try_state::<PhoneLinkState>() else { return };
    let link = {
        let mut guard = lock(&state.inner);
        guard.open_approvals.remove(approval_id);
        guard.link.clone()
    };
    if let Some(link) = link {
        broadcast_json(
            &link,
            &json!({
                "t": "approval.resolved",
                "id": state.next_msg_id(),
                "approval_id": approval_id,
                "by": by,
            }),
            false,
        );
    }
}

/// 只有打开过该会话（`session.open` / `message.send`）的手机才收内容事件；`notify` 事件（完成 / 出错）
/// 对其他手机和离线手机仍会送达，但内容脱敏，只剩“有事发生”。
fn send_session_event(link: &Link, subs: &HashMap<String, HashSet<String>>, session: &str, msg: &Value, notify: bool) {
    let kind = msg.get("kind").and_then(Value::as_str).unwrap_or("");
    for peer in link.peers() {
        let subscribed = subs.get(&peer.id).is_some_and(|set| set.contains(session));
        if peer.online && subscribed {
            let _ = send_json(link, &peer.id, msg, notify);
        } else if notify {
            let mut redacted = msg.clone();
            redacted["data"] = phone_wire::redact_for_unsubscribed(kind, &msg["data"]);
            let _ = send_json(link, &peer.id, &redacted, notify);
        }
    }
}

/// 第一次有手机要听回答时，起一个后台线程专门做语音合成并分片发给手机。
fn ensure_speaker(app: &tauri::AppHandle) {
    let state = app.state::<PhoneLinkState>();
    let rx = {
        let mut inner = lock(&state.inner);
        if inner.speak_tx.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel::<SpeakJob>();
        inner.speak_tx = Some(tx);
        rx
    };
    let app = app.clone();
    std::thread::spawn(move || {
        for job in rx {
            speak_job(&app, job);
        }
    });
}

fn speak_job(app: &tauri::AppHandle, job: SpeakJob) {
    use base64::Engine as _;
    let state = app.state::<PhoneLinkState>();
    let Some(link) = lock(&state.inner).link.clone() else { return };
    let text = phone_voice::speech_text(&job.text);
    if text.is_empty() {
        return;
    }
    let audio_id = state.next_msg_id();
    match crate::tts::synthesize_raw(app, &text, None) {
        Ok(audio) => {
            let parts = phone_voice::audio_chunks(&audio);
            let total = parts.len();
            for (index, part) in parts.iter().enumerate() {
                let _ = send_json(
                    &link,
                    &job.peer,
                    &json!({
                        "t": "voice.audio",
                        "id": state.next_msg_id(),
                        "session": job.session,
                        "audio_id": audio_id,
                        "ok": true,
                        "seq": index,
                        "last": index + 1 == total,
                        "mime": "audio/mpeg",
                        "data": base64::engine::general_purpose::STANDARD.encode(part),
                    }),
                    false,
                );
            }
        }
        Err(error) => {
            let _ = send_json(
                &link,
                &job.peer,
                &json!({ "t": "voice.audio", "id": state.next_msg_id(), "session": job.session, "audio_id": audio_id, "ok": false, "error": error }),
                false,
            );
        }
    }
}

/// 语音识别没开、引擎还没加载这类错误，换成手机上看得懂的话。
fn voice_error_text(error: &str) -> String {
    if error.contains("未开启") || error.contains("请先开启") {
        "电脑上还没有开启语音识别，请先在电脑的设置里打开".to_string()
    } else {
        error.to_string()
    }
}

pub fn observe_chat_event<R: tauri::Runtime>(app: &tauri::AppHandle<R>, name: &str, payload: &Value) {
    let Some(state) = app.try_state::<PhoneLinkState>() else { return };
    let (link, subs) = {
        let guard = lock(&state.inner);
        let Some(link) = guard.link.clone() else { return };
        (link, guard.subscriptions.clone())
    };
    let send_to_session = |session: &str, msg: &Value, notify: bool| send_session_event(&link, &subs, session, msg, notify);

    match name {
        "chat-sessions-changed" => {
            if let Some(items) = payload.as_array() {
                let compact: Vec<Value> = items
                    .iter()
                    .take(50)
                    .map(|s| {
                        json!({
                            "id": s["id"],
                            "title": s["title"],
                            "updated_at": s["updatedAt"],
                            "state": "idle",
                            "workspace": s["workspace"],
                        })
                    })
                    .collect();
                broadcast_json(
                    &link,
                    &json!({
                        "t": "sessions.changed",
                        "id": state.next_msg_id(),
                        "items": compact,
                    }),
                    false,
                );
            }
        }
        "chat-event" => {
            let Some(session) = payload.get("session").and_then(Value::as_str) else { return };
            let ev = &payload["event"];
            let ev_type = ev.get("type").and_then(Value::as_str).unwrap_or("");
            // 手机要听回答的对话：攒下这一轮最后一段话，回答结束后合成语音（工具调用之前的话不念）。
            match ev_type {
                "text_delta" => {
                    if let Some(entry) = lock(&state.inner).speak.get_mut(session) {
                        entry.1.push_str(ev["text"].as_str().unwrap_or(""));
                    }
                }
                "tool_start" => {
                    if let Some(entry) = lock(&state.inner).speak.get_mut(session) {
                        entry.1.clear();
                    }
                }
                "done" => {
                    let job = {
                        let mut inner = lock(&state.inner);
                        let entry = inner.speak.remove(session);
                        entry.map(|(peer, text)| (peer, text, inner.speak_tx.clone()))
                    };
                    if let Some((peer, text, Some(tx))) = job {
                        if !text.trim().is_empty() {
                            let _ = tx.send(SpeakJob { peer, session: session.to_string(), text });
                        }
                    }
                }
                "cancelled" | "error" => {
                    lock(&state.inner).speak.remove(session);
                }
                _ => {}
            }
            let mapped = match ev_type {
                "text_delta" => Some(("text.delta", json!({ "text": ev["text"] }), false)),
                "reasoning_delta" => Some(("reasoning.delta", json!({ "text": ev["text"] }), false)),
                "tool_start" => Some((
                    "tool.start",
                    json!({ "id": ev["id"], "name": ev["name"], "arguments": phone_wire::clip_value(&ev["arguments"], phone_wire::MAX_EVENT_FIELD_CHARS) }),
                    false,
                )),
                "tool_end" => Some((
                    "tool.end",
                    json!({ "id": ev["id"], "name": ev["name"], "output": phone_wire::clip_value(&ev["output"], phone_wire::MAX_EVENT_FIELD_CHARS), "is_error": ev["is_error"] }),
                    false,
                )),
                "done" => Some(("turn.done", json!({}), true)),
                "usage" => Some(("usage", json!({ "prompt": ev["prompt_tokens"], "completion": ev["completion_tokens"] }), false)),
                "truncated" => Some(("truncated", json!({}), false)),
                "cancelled" => Some(("state", json!({ "state": "cancelled" }), false)),
                "error" => Some(("error", json!({ "message": ev["message"] }), true)),
                _ => None,
            };
            if let Some((kind, data, notify)) = mapped {
                send_to_session(
                    session,
                    &json!({
                        "t": "event",
                        "id": state.next_msg_id(),
                        "session": session,
                        "kind": kind,
                        "data": data,
                    }),
                    notify,
                );
            }
        }
        "chat-checkpoint" => {
            let Some(session) = payload.get("session").and_then(Value::as_str) else { return };
            let files: Vec<Value> = payload["files"].as_array().map(|all| all.iter().take(30).cloned().collect()).unwrap_or_default();
            let count = payload["files"].as_array().map_or(0, Vec::len);
            send_to_session(
                session,
                &json!({
                    "t": "event",
                    "id": state.next_msg_id(),
                    "session": session,
                    "kind": "checkpoint",
                    "data": { "run_id": payload["runId"], "files": files, "count": count },
                }),
                false,
            );
        }
        "chat-permission-mode-changed" => {
            let Some(session) = payload.get("sessionId").and_then(Value::as_str) else { return };
            let mode = if payload.get("mode").and_then(Value::as_str) == Some("auto") { "auto" } else { "ask" };
            send_to_session(
                session,
                &json!({
                    "t": "event",
                    "id": state.next_msg_id(),
                    "session": session,
                    "kind": "permission",
                    "data": { "mode": mode },
                }),
                false,
            );
        }
        "chat-run-state" => {
            let Some(session) = payload.get("session").and_then(Value::as_str) else { return };
            send_to_session(
                session,
                &json!({
                    "t": "event",
                    "id": state.next_msg_id(),
                    "session": session,
                    "kind": "state",
                    "data": { "state": payload["state"], "reason": payload["reason"] },
                }),
                false,
            );
        }
        "chat-approval" => {
            let Some(approval_id) = payload.get("id").and_then(Value::as_str) else { return };
            let session = payload.get("session").and_then(Value::as_str).unwrap_or("");
            let tool = payload.get("tool").and_then(Value::as_str).unwrap_or("操作");
            let danger = payload.get("danger").and_then(Value::as_str);
            let external = payload.get("externalPath").and_then(Value::as_bool).unwrap_or(false);
            let risk = if danger.is_some() || external || payload.get("risk").and_then(Value::as_str) == Some("exec") {
                "high"
            } else {
                "low"
            };
            let title = danger.map(str::to_string).unwrap_or_else(|| format!("允许执行 {tool}？"));
            let mut options = vec![
                json!({ "id": "once", "label": "允许一次" }),
                json!({ "id": "deny", "label": "拒绝" }),
            ];
            if payload.get("canRemember").and_then(Value::as_bool) == Some(true) {
                options.insert(1, json!({ "id": "always", "label": "本对话都允许" }));
            }
            lock(&state.inner).open_approvals.insert(approval_id.to_string(), session.to_string());
            broadcast_json(
                &link,
                &json!({
                    "t": "approval.request",
                    "id": state.next_msg_id(),
                    "approval_id": approval_id,
                    "session": session,
                    "title": title,
                    "detail": phone_wire::clip_detail(&payload["arguments"], phone_wire::MAX_EVENT_FIELD_CHARS),
                    "risk": risk,
                    "options": options,
                    // 手机的审批卡要显示的细节：具体是读/写/联网/执行，是不是工作区外路径，读过外部内容，高风险原因。
                    "tool": tool,
                    "risk_kind": payload.get("risk").cloned().unwrap_or(Value::Null),
                    "external_path": external,
                    "tainted": payload.get("tainted").and_then(Value::as_bool).unwrap_or(false),
                    "danger": danger,
                    "remember_as": payload.get("rememberAs").cloned().unwrap_or(Value::Null),
                }),
                true,
            );
        }
        "chat-question" => {
            let Some(question_id) = payload.get("id").and_then(Value::as_str) else { return };
            let session = payload.get("session").and_then(Value::as_str).unwrap_or("");
            let question = payload.get("question").and_then(Value::as_str).unwrap_or("请确认");
            let raw_opts = payload.get("options").and_then(Value::as_array).cloned().unwrap_or_default();
            let mut options: Vec<Value> = raw_opts
                .into_iter()
                .filter_map(|v| v.as_str().map(|s| json!({ "id": s, "label": s })))
                .collect();
            options.push(json!({ "id": "__dismiss__", "label": "跳过" }));
            lock(&state.inner).open_approvals.insert(question_id.to_string(), session.to_string());
            broadcast_json(
                &link,
                &json!({
                    "t": "approval.request",
                    "id": state.next_msg_id(),
                    "approval_id": question_id,
                    "session": session,
                    "title": question,
                    "detail": {},
                    "risk": "low",
                    "options": options,
                    // 新手机把它画成选择卡：问题、选项（不含“跳过”）、可以自己输入；老手机仍当作带按钮的确认卡。
                    "kind": "question",
                    "question": question,
                    "choices": payload.get("options").cloned().unwrap_or_else(|| json!([])),
                    "free_text": true,
                    "connectors": payload.get("connectors").map(|c| !c.is_null()).unwrap_or(false),
                }),
                true,
            );
        }
        "chat-pending-changed" => {
            let Some(session) = payload.get("session").and_then(Value::as_str) else { return };
            let pending = &payload["pending"];
            let mut active = HashSet::new();
            for key in ["approvals", "questions"] {
                if let Some(arr) = pending.get(key).and_then(Value::as_array) {
                    for item in arr {
                        if let Some(id) = item.get("id").and_then(Value::as_str) {
                            active.insert(id.to_string());
                        }
                    }
                }
            }
            let resolved: Vec<String> = {
                let mut guard = lock(&state.inner);
                let gone: Vec<String> = guard
                    .open_approvals
                    .iter()
                    .filter(|(id, sid)| sid.as_str() == session && !active.contains(*id))
                    .map(|(id, _)| id.clone())
                    .collect();
                for id in &gone {
                    guard.open_approvals.remove(id);
                }
                gone
            };
            for id in resolved {
                broadcast_json(
                    &link,
                    &json!({
                        "t": "approval.resolved",
                        "id": state.next_msg_id(),
                        "approval_id": id,
                        "by": "desktop",
                    }),
                    false,
                );
            }
        }
        _ => {}
    }
}

pub fn observe_delegation_event(app: &tauri::AppHandle, event: &agent_bridge::observer::ObservedEvent) {
    let Some(state) = app.try_state::<PhoneLinkState>() else { return };
    let (link, subs) = {
        let guard = lock(&state.inner);
        let Some(link) = guard.link.clone() else { return };
        (link, guard.subscriptions.clone())
    };
    let send_to_session = |session: &str, msg: &Value, notify: bool| send_session_event(&link, &subs, session, msg, notify);
    match event {
        agent_bridge::observer::ObservedEvent::SessionStarted { .. } => {
            if let Ok(page) = crate::chat::remote_sessions_list(app, 50, None) {
                if let Some(items) = page.get("items") {
                    broadcast_json(&link, &json!({
                        "t": "sessions.changed",
                        "id": state.next_msg_id(),
                        "items": items,
                    }), false);
                }
            }
        }
        agent_bridge::observer::ObservedEvent::ToolStarted { session_id, tool_name, summary } => {
            send_to_session(session_id, &json!({
                "t": "event",
                "id": state.next_msg_id(),
                "session": session_id,
                "kind": "tool.start",
                "data": { "name": tool_name, "arguments": phone_wire::clip_value(&json!(summary), phone_wire::MAX_EVENT_FIELD_CHARS) },
            }), false);
        }
        agent_bridge::observer::ObservedEvent::ToolFinished { session_id, tool_name, ok, preview } => {
            send_to_session(session_id, &json!({
                "t": "event",
                "id": state.next_msg_id(),
                "session": session_id,
                "kind": "tool.end",
                "data": { "name": tool_name, "is_error": !ok, "output": phone_wire::clip_value(&json!(preview), phone_wire::MAX_EVENT_FIELD_CHARS) },
            }), false);
        }
        agent_bridge::observer::ObservedEvent::PermissionRequested { session_id, request_id, tool_name, detail, risk } => {
            let title = format!("允许外部任务执行 {tool_name}？");
            lock(&state.inner).open_approvals.insert(request_id.clone(), session_id.clone());
            broadcast_json(&link, &json!({
                "t": "approval.request",
                "id": state.next_msg_id(),
                "approval_id": request_id,
                "session": session_id,
                "title": title,
                "detail": { "task_id": session_id, "detail": phone_wire::clip_value(&json!(detail), phone_wire::MAX_EVENT_FIELD_CHARS) },
                "risk": risk,
                "options": [
                    { "id": "allow", "label": "允许" },
                    { "id": "deny", "label": "拒绝" }
                ],
            }), true);
        }
        agent_bridge::observer::ObservedEvent::TaskCompleted { session_id, summary, success } => {
            send_to_session(session_id, &json!({
                "t": "event",
                "id": state.next_msg_id(),
                "session": session_id,
                "kind": if *success { "turn.done" } else { "error" },
                "data": { "message": phone_wire::clip_value(&json!(summary), phone_wire::MAX_EVENT_FIELD_CHARS) },
            }), true);
            if let Ok(page) = crate::chat::remote_sessions_list(app, 50, None) {
                if let Some(items) = page.get("items") {
                    broadcast_json(&link, &json!({
                        "t": "sessions.changed",
                        "id": state.next_msg_id(),
                        "items": items,
                    }), false);
                }
            }
        }
        _ => {}
    }
}

// ───────────── Tauri 命令 ─────────────

#[tauri::command]
pub fn phone_link_status(app: tauri::AppHandle) -> Result<Value, String> {
    status_value(&app)
}

#[tauri::command]
pub async fn phone_link_set_enabled(
    app: tauri::AppHandle,
    enabled: bool,
    device_name: Option<String>,
) -> Result<Value, String> {
    if enabled {
        crate::account::fresh_access_token(&app)
            .await
            .map_err(|_| "请先登录小伴账号，手机登录同一个账号后才能连上这台电脑".to_string())?;
    }
    let mut prefs = load_prefs(&app);
    prefs.enabled = enabled;
    if let Some(name) = device_name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        if name.chars().count() > 40 {
            return Err("设备名称最多 40 个字".into());
        }
        prefs.device_name = name;
    }
    save_prefs(&app, &prefs)?;
    if enabled {
        ensure_running(&app).await?;
    } else {
        stop_running(&app);
    }
    status_value(&app)
}

/// 把一台手机从账号的设备列表里移除（它再登录会重新出现）。
#[tauri::command]
pub async fn phone_link_remove_peer(app: tauri::AppHandle, peer_id: String) -> Result<Value, String> {
    let peer_id = peer_id.trim().to_string();
    if peer_id.is_empty() {
        return Err("缺少设备编号".into());
    }
    let link = {
        let state = app.state::<PhoneLinkState>();
        let mut guard = lock(&state.inner);
        guard.subscriptions.remove(&peer_id);
        guard.link.clone()
    };
    let link = link.ok_or_else(|| "请先开启“允许手机遥控”".to_string())?;
    link.remove_device(&peer_id).await.map_err(|e| e.to_string())?;
    emit_status(&app);
    status_value(&app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_messages_and_audit_log_are_parsed_and_recorded() {
        let (t, id, v) = classify_peer_message(r#"{"t":"message.send","id":"m1","text":"你好","client_id":"c1"}"#.as_bytes()).unwrap();
        assert_eq!(t, "message.send");
        assert_eq!(id, "m1");
        assert_eq!(v["client_id"], "c1");
        assert!(classify_peer_message(b"not-json").is_err());
        assert!(classify_peer_message(br#"{"id":"m1"}"#).is_err());

        let audit = std::env::temp_dir().join(format!("xiaoban-remote-audit-{}.jsonl", std::process::id()));
        let _ = fs::remove_file(&audit);
        append_remote_audit(&audit, &json!({ "action": "message.send", "clientId": "c1" }));
        let text = fs::read_to_string(&audit).unwrap();
        assert!(text.contains("\"clientId\":\"c1\""));
        let _ = fs::remove_file(&audit);
    }
}

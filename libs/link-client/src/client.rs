use crate::{
    api::Api,
    store::{DeviceIdentity, LinkStore, State},
    Error, TokenFn,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::{mpsc, Notify};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
};

#[derive(Clone, Debug)]
pub struct LinkConfig {
    /// 中转服务的 HTTP 地址，含路径前缀，如 `https://link.xiangban.xyz/api/link`。WebSocket 地址由它推出。
    pub base_url: String,
    /// 显示给对端看的设备名称。
    pub device_name: String,
    /// 设备类型：桌面应用是 `desktop`；手机是 `phone`（这里主要给测试用）。
    pub kind: String,
    /// 断线后第一次重连的等待时间，之后每次翻倍，直到上限。
    pub backoff_initial: Duration,
    pub backoff_max: Duration,
}

impl LinkConfig {
    pub fn new(base_url: impl Into<String>, device_name: impl Into<String>) -> Self {
        Self { base_url: base_url.into(), device_name: device_name.into(), kind: "desktop".into(), backoff_initial: Duration::from_secs(1), backoff_max: Duration::from_secs(30) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerSummary {
    pub id: String,
    pub name: String,
    pub online: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkEvent {
    Connected,
    Disconnected,
    PeerOnline { peer: String, online: bool },
    /// 同账号下多了一台对端设备（或它改了名字）。
    PeerAdded(PeerSummary),
    /// 对端设备被移除。
    PeerRemoved { peer: String },
    /// 对端发来的文本消息。
    Message { peer: String, data: Vec<u8> },
    /// 中转告知没送达（对端离线且没标通知）。
    Undelivered { peer: String },
    /// 中转报告的错误，如 rate_limited。
    ServerError { code: String },
    /// 同一设备在别处连接，本连接被顶替，不再自动重连。
    Replaced,
    /// 设备在中转被撤销，本地身份已清除，会重新登记成新设备。
    Revoked,
}

#[derive(Clone)]
struct Peer {
    name: String,
    online: bool,
}

struct Inner {
    cfg: LinkConfig,
    api: Api,
    token: TokenFn,
    store: Arc<dyn LinkStore>,
    /// 登记设备要串行：启动后马上发消息时，连接循环和调用方会同时发现“还没有身份”。
    identity_lock: tokio::sync::Mutex<()>,
    state: Mutex<State>,
    peers: Mutex<BTreeMap<String, Peer>>,
    out: Mutex<Option<mpsc::Sender<String>>>,
    events: mpsc::Sender<LinkEvent>,
    stopped: AtomicBool,
    stop: Notify,
}

#[derive(Clone)]
pub struct Link {
    inner: Arc<Inner>,
}

fn ws_url(base: &str) -> String {
    let b = base.trim_end_matches('/');
    if let Some(rest) = b.strip_prefix("https://") { format!("wss://{rest}") } else if let Some(rest) = b.strip_prefix("http://") { format!("ws://{rest}") } else { b.to_string() }
}

fn random_install_id() -> String {
    (0..32).map(|_| char::from(b"abcdefghijklmnopqrstuvwxyz0123456789"[rand::Rng::gen_range(&mut rand::thread_rng(), 0..36)])).collect()
}

enum SessionEnd {
    Retry { connected: bool },
    Stop,
}

impl Link {
    /// 启动：读状态（没有安装编号就生成并落盘），开始后台连接。返回句柄和事件流。
    pub async fn start(cfg: LinkConfig, store: Arc<dyn LinkStore>, token: TokenFn) -> Result<(Link, mpsc::Receiver<LinkEvent>), Error> {
        let mut state = store.load()?;
        if state.install_id.is_empty() {
            state.install_id = random_install_id();
            store.save(&state)?;
        }
        let (events, rx) = mpsc::channel(1024);
        let inner = Arc::new(Inner {
            api: Api::new(&cfg.base_url, token.clone()),
            cfg,
            token,
            store,
            identity_lock: tokio::sync::Mutex::new(()),
            state: Mutex::new(state),
            peers: Mutex::default(),
            out: Mutex::new(None),
            events,
            stopped: AtomicBool::new(false),
            stop: Notify::new(),
        });
        tokio::spawn(inner.clone().run());
        Ok((Link { inner }, rx))
    }

    pub fn device_id(&self) -> Option<String> {
        self.inner.state.lock().unwrap().device.as_ref().map(|d| d.id.clone())
    }

    /// 同账号下的对端设备（电脑的对端是手机），按名称排序。
    pub fn peers(&self) -> Vec<PeerSummary> {
        let mut v: Vec<_> = self.inner.peers.lock().unwrap().iter().map(|(id, p)| PeerSummary { id: id.clone(), name: p.name.clone(), online: p.online }).collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// 给对端发一条文本消息（内容是 UTF-8，通常是 JSON）。`notify` 为真时，对端离线也由中转暂存。
    /// 返回 Ok 只表示已进入发送队列；没送达会收到 [`LinkEvent::Undelivered`]。
    pub fn send(&self, peer: &str, data: &[u8], notify: bool) -> Result<(), Error> {
        let i = &self.inner;
        let text = std::str::from_utf8(data).map_err(|_| Error::BadMessage)?;
        if !i.peers.lock().unwrap().contains_key(peer) {
            return Err(Error::UnknownPeer);
        }
        let out = i.out.lock().unwrap().clone().ok_or(Error::NotConnected)?;
        let frame = json!({ "op": "send", "to": peer, "data": text, "notify": notify }).to_string();
        out.try_send(frame).map_err(|_| Error::Congested)
    }

    /// 移除一台对端设备（例如不再使用的旧手机）。它仍在运行的话，下次启动会重新登记成新设备。
    pub async fn remove_device(&self, peer: &str) -> Result<(), Error> {
        match self.inner.api.remove_device(peer).await {
            Ok(()) | Err(Error::Server { status: 404, .. }) => {}
            Err(e) => return Err(e),
        }
        self.inner.forget_peer(peer);
        Ok(())
    }

    pub fn shutdown(&self) {
        self.inner.stopped.store(true, Ordering::SeqCst);
        self.inner.stop.notify_waiters();
    }
}

impl Inner {
    async fn emit(&self, event: LinkEvent) { let _ = self.events.send(event).await; }

    async fn ensure_identity(&self) -> Result<DeviceIdentity, Error> {
        let _guard = self.identity_lock.lock().await;
        let install_id = {
            let state = self.state.lock().unwrap();
            if let Some(d) = state.device.clone() {
                return Ok(d);
            }
            state.install_id.clone()
        };
        let id = self.api.register_device(&self.cfg.kind, &self.cfg.device_name, &install_id).await?;
        let identity = DeviceIdentity { id, name: self.cfg.device_name.clone() };
        let mut state = self.state.lock().unwrap();
        state.device = Some(identity.clone());
        self.store.save(&state)?;
        Ok(identity)
    }

    fn forget_peer(&self, peer: &str) {
        self.peers.lock().unwrap().remove(peer);
    }

    /// 设备在中转被撤销：本地身份作废，下次连接会重新登记成新设备（安装编号换新，避免又拿回被撤销的）。
    fn wipe_identity(&self) {
        self.peers.lock().unwrap().clear();
        let mut state = self.state.lock().unwrap();
        state.device = None;
        state.install_id = random_install_id();
        let _ = self.store.save(&state);
    }

    async fn run(self: Arc<Self>) {
        let mut backoff = self.cfg.backoff_initial;
        loop {
            if self.stopped.load(Ordering::SeqCst) { return; }
            match self.session().await {
                SessionEnd::Stop => return,
                SessionEnd::Retry { connected } => {
                    if connected { backoff = self.cfg.backoff_initial; }
                }
            }
            if self.stopped.load(Ordering::SeqCst) { return; }
            let jitter = Duration::from_millis(rand::Rng::gen_range(&mut rand::thread_rng(), 0..=(backoff.as_millis() as u64 / 4).max(1)));
            tokio::select! {
                _ = tokio::time::sleep(backoff + jitter) => {}
                _ = self.stop.notified() => return,
            }
            backoff = (backoff * 2).min(self.cfg.backoff_max);
        }
    }

    async fn session(self: &Arc<Self>) -> SessionEnd {
        let Ok(identity) = self.ensure_identity().await else { return SessionEnd::Retry { connected: false } };
        let Some(token) = (self.token)().await else { return SessionEnd::Retry { connected: false } };
        let url = format!("{}/ws?device_id={}", ws_url(&self.cfg.base_url), identity.id);
        let Ok(mut req) = url.into_client_request() else { return SessionEnd::Retry { connected: false } };
        req.headers_mut().insert("Authorization", HeaderValue::from_str(&format!("Bearer {token}")).unwrap_or_else(|_| HeaderValue::from_static("")));
        req.headers_mut().insert("Sec-WebSocket-Protocol", HeaderValue::from_static("xb-link"));
        let ws = match connect_async(req).await {
            Ok((ws, _)) => ws,
            Err(tokio_tungstenite::tungstenite::Error::Http(resp)) if resp.status().as_u16() == 404 => {
                // 中转不认这台设备了（被撤销）。
                self.wipe_identity();
                self.emit(LinkEvent::Revoked).await;
                return SessionEnd::Retry { connected: false };
            }
            Err(_) => return SessionEnd::Retry { connected: false },
        };
        let (mut sink, mut stream) = ws.split();
        let (tx, mut rx) = mpsc::channel::<String>(1024);
        *self.out.lock().unwrap() = Some(tx.clone());
        let writer = tokio::spawn(async move {
            while let Some(frame) = rx.recv().await {
                if sink.send(Message::Text(frame.into())).await.is_err() { break; }
            }
            let _ = sink.close().await;
        });
        self.emit(LinkEvent::Connected).await;

        let mut ping = tokio::time::interval(Duration::from_secs(25));
        ping.tick().await;
        let mut end = SessionEnd::Retry { connected: true };
        loop {
            tokio::select! {
                _ = self.stop.notified() => { end = SessionEnd::Stop; break; }
                _ = ping.tick() => { let _ = tx.try_send(json!({"op":"ping"}).to_string()); }
                next = tokio::time::timeout(Duration::from_secs(90), stream.next()) => {
                    let Ok(Some(Ok(msg))) = next else { break };
                    if let Message::Text(text) = msg {
                        if let Some(stop) = self.handle_frame(text.as_str()).await {
                            if stop { end = SessionEnd::Stop; }
                            break;
                        }
                    } else if matches!(msg, Message::Close(_)) { break; }
                }
            }
        }
        *self.out.lock().unwrap() = None;
        drop(tx);
        let _ = writer.await;
        let went_offline: Vec<String> = {
            let mut peers = self.peers.lock().unwrap();
            peers.iter_mut().filter(|(_, p)| p.online).map(|(id, p)| { p.online = false; id.clone() }).collect()
        };
        for peer in went_offline { self.emit(LinkEvent::PeerOnline { peer, online: false }).await; }
        self.emit(LinkEvent::Disconnected).await;
        end
    }

    fn peer_from(v: &Value) -> Option<(String, Peer)> {
        Some((v["id"].as_str()?.to_string(), Peer { name: v["name"].as_str().unwrap_or("").to_string(), online: v["online"].as_bool().unwrap_or(false) }))
    }

    /// 返回 `Some(true)` 表示应停止重连，`Some(false)` 表示结束本次连接后重试，`None` 继续。
    async fn handle_frame(self: &Arc<Self>, text: &str) -> Option<bool> {
        let v: Value = serde_json::from_str(text).ok()?;
        match v["op"].as_str()? {
            "peers" => {
                let fresh: BTreeMap<String, Peer> = v["items"].as_array()?.iter().filter_map(Self::peer_from).collect();
                let (added, removed, changed) = {
                    let mut peers = self.peers.lock().unwrap();
                    let removed: Vec<String> = peers.keys().filter(|id| !fresh.contains_key(*id)).cloned().collect();
                    let mut added = Vec::new();
                    let mut changed = Vec::new();
                    for (id, p) in &fresh {
                        match peers.get(id) {
                            None => added.push(PeerSummary { id: id.clone(), name: p.name.clone(), online: p.online }),
                            Some(old) if old.online != p.online => changed.push((id.clone(), p.online)),
                            _ => {}
                        }
                    }
                    *peers = fresh;
                    (added, removed, changed)
                };
                for peer in removed { self.emit(LinkEvent::PeerRemoved { peer }).await; }
                for summary in added { self.emit(LinkEvent::PeerAdded(summary)).await; }
                for (peer, online) in changed { self.emit(LinkEvent::PeerOnline { peer, online }).await; }
            }
            "peer.added" => {
                let (id, mut peer) = Self::peer_from(&v["device"])?;
                let (is_new, renamed, online_changed) = {
                    let mut peers = self.peers.lock().unwrap();
                    let previous = peers.get(&id).cloned();
                    if let Some(old) = &previous { peer.online = old.online || peer.online; }
                    peers.insert(id.clone(), peer.clone());
                    match previous {
                        None => (true, false, false),
                        Some(old) => (false, old.name != peer.name, old.online != peer.online),
                    }
                };
                if is_new || renamed {
                    self.emit(LinkEvent::PeerAdded(PeerSummary { id: id.clone(), name: peer.name.clone(), online: peer.online })).await;
                } else if online_changed {
                    self.emit(LinkEvent::PeerOnline { peer: id, online: peer.online }).await;
                }
            }
            "peer.removed" => {
                let peer = v["device"].as_str()?.to_string();
                if self.peers.lock().unwrap().remove(&peer).is_some() {
                    self.emit(LinkEvent::PeerRemoved { peer }).await;
                }
            }
            "presence" => {
                let peer = v["device"].as_str()?.to_string();
                let online = v["online"].as_bool()?;
                if let Some(p) = self.peers.lock().unwrap().get_mut(&peer) { p.online = online; }
                self.emit(LinkEvent::PeerOnline { peer, online }).await;
            }
            "deliver" => {
                let peer = v["from"].as_str()?.to_string();
                let data = v["data"].as_str()?.as_bytes().to_vec();
                self.emit(LinkEvent::Message { peer, data }).await;
            }
            "undelivered" => {
                self.emit(LinkEvent::Undelivered { peer: v["to"].as_str()?.into() }).await;
            }
            "error" => {
                let code = v["code"].as_str().unwrap_or("").to_string();
                match code.as_str() {
                    "replaced" => { self.emit(LinkEvent::Replaced).await; return Some(true); }
                    "device_revoked" => { self.wipe_identity(); self.emit(LinkEvent::Revoked).await; return Some(false); }
                    _ => self.emit(LinkEvent::ServerError { code }).await,
                }
            }
            _ => {}
        }
        None
    }
}


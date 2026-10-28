use crate::{
    api::{Api, PeerInfo},
    store::{DeviceIdentity, LinkStore, PeerRecord, PendingPair, State},
    Error, TokenFn,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{mpsc, Notify};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
};
use xiaoban_link_protocol::{
    envelope::{open, Envelope, Sender},
    keys::DeviceKey,
    pairing::{derive_pairing_key, verify_proof, QrPayload},
    replay::ReplayGuard,
};

#[derive(Clone, Debug)]
pub struct LinkConfig {
    /// 中转服务的 HTTP 地址，含路径前缀，如 `https://link.xiangban.xyz/api/link`。WebSocket 地址由它推出。
    pub base_url: String,
    /// 显示给手机看的设备名称。
    pub device_name: String,
    /// 二维码有效窗口（桌面侧）。
    pub pairing_window: Duration,
    /// 断线后第一次重连的等待时间，之后每次翻倍，直到上限。
    pub backoff_initial: Duration,
    pub backoff_max: Duration,
}

impl LinkConfig {
    pub fn new(base_url: impl Into<String>, device_name: impl Into<String>) -> Self {
        Self { base_url: base_url.into(), device_name: device_name.into(), pairing_window: Duration::from_secs(300), backoff_initial: Duration::from_secs(1), backoff_max: Duration::from_secs(30) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerSummary {
    pub id: String,
    pub name: String,
    pub online: bool,
}

#[derive(Clone, Debug)]
pub struct PairingOffer {
    /// 二维码的文本内容
    pub qr_text: String,
    /// 配对码（备用，手动输入）
    pub code: String,
    pub expires_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkEvent {
    Connected,
    Disconnected,
    PeerOnline { peer: String, online: bool },
    /// 手机的配对证明通过，已建立信任。
    Paired(PeerSummary),
    /// 配对请求被拒绝（证明不对，或没有打开配对窗口）。
    PairRejected { reason: String },
    Unpaired { peer: String },
    /// 对端发来的明文（已解密、已通过重放检查）。
    Message { peer: String, data: Vec<u8> },
    /// 中转告知没送达（对端离线且没标通知）。
    Undelivered { peer: String, seq: u64 },
    /// 收到的信封被丢弃：解密失败或重放。
    Dropped { peer: String, reason: String },
    /// 中转报告的错误，如 rate_limited。
    ServerError { code: String },
    /// 同一设备在别处连接，本连接被顶替，不再自动重连。
    Replaced,
    /// 设备在中转被撤销，本地配对已清除。
    Revoked,
}

struct Inner {
    cfg: LinkConfig,
    api: Api,
    token: TokenFn,
    store: Arc<dyn LinkStore>,
    /// 登记设备要串行：启动后马上配对时，连接循环和 `begin_pairing` 会同时发现“还没有身份”。
    identity_lock: tokio::sync::Mutex<()>,
    state: Mutex<State>,
    start_counter: u64,
    senders: Mutex<HashMap<String, Sender>>,
    guards: Mutex<HashMap<String, ReplayGuard>>,
    online: Mutex<HashSet<String>>,
    handled_pairings: Mutex<HashSet<String>>,
    out: Mutex<Option<mpsc::Sender<String>>>,
    events: mpsc::Sender<LinkEvent>,
    dirty: AtomicBool,
    stopped: AtomicBool,
    stop: Notify,
}

#[derive(Clone)]
pub struct Link {
    inner: Arc<Inner>,
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn ws_url(base: &str) -> String {
    let b = base.trim_end_matches('/');
    if let Some(rest) = b.strip_prefix("https://") { format!("wss://{rest}") } else if let Some(rest) = b.strip_prefix("http://") { format!("ws://{rest}") } else { b.to_string() }
}

enum SessionEnd {
    Retry { connected: bool },
    Stop,
}

impl Link {
    /// 启动：读状态、启动计数加一并落盘、开始后台连接。返回句柄和事件流。
    pub async fn start(cfg: LinkConfig, store: Arc<dyn LinkStore>, token: TokenFn) -> Result<(Link, mpsc::Receiver<LinkEvent>), Error> {
        let mut state = store.load()?;
        state.start_counter += 1;
        store.save(&state)?;
        let (events, rx) = mpsc::channel(1024);
        let inner = Arc::new(Inner {
            api: Api::new(&cfg.base_url, token.clone()),
            cfg,
            token,
            store,
            identity_lock: tokio::sync::Mutex::new(()),
            start_counter: state.start_counter,
            state: Mutex::new(state),
            senders: Mutex::default(),
            guards: Mutex::default(),
            online: Mutex::default(),
            handled_pairings: Mutex::default(),
            out: Mutex::new(None),
            events,
            dirty: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            stop: Notify::new(),
        });
        // 恢复各对端的重放防护进度。
        {
            let state = inner.state.lock().unwrap();
            let mut guards = inner.guards.lock().unwrap();
            for (peer, rec) in &state.peers {
                guards.insert(peer.clone(), ReplayGuard::restore(rec.replay.clone()));
            }
        }
        tokio::spawn(inner.clone().run());
        tokio::spawn(inner.clone().flusher());
        Ok((Link { inner }, rx))
    }

    pub fn device_id(&self) -> Option<String> {
        self.inner.state.lock().unwrap().device.as_ref().map(|d| d.id.clone())
    }

    pub fn peers(&self) -> Vec<PeerSummary> {
        let state = self.inner.state.lock().unwrap();
        let online = self.inner.online.lock().unwrap();
        let mut v: Vec<_> = state.peers.values().map(|p| PeerSummary { id: p.peer_id.clone(), name: p.peer_name.clone(), online: online.contains(&p.peer_id) }).collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// 打开配对窗口，返回二维码内容。窗口内第一个证明通过的手机会被接受。
    pub async fn begin_pairing(&self) -> Result<PairingOffer, Error> {
        let i = &self.inner;
        let identity = i.ensure_identity().await?;
        let code = i.api.create_pair_code(&identity.id).await?;
        let mut secret = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret);
        let key = DeviceKey::from_bytes(&B64.decode(&identity.private_key).ok().and_then(|b| b.try_into().ok()).ok_or_else(|| Error::Store("私钥格式不对".into()))?)?;
        let qr = QrPayload { r: i.cfg.base_url.clone(), d: identity.id.clone(), c: code.code.clone(), k: B64.encode(key.public_bytes()), s: B64.encode(secret) };
        let expires_at_ms = now_ms() + i.cfg.pairing_window.as_millis() as i64;
        {
            let mut state = i.state.lock().unwrap();
            state.pending_pair = Some(PendingPair { code: code.code.clone(), secret: B64.encode(secret), expires_at_ms });
            i.store.save(&state)?;
        }
        Ok(PairingOffer { qr_text: qr.encode(), code: code.code, expires_at_ms })
    }

    /// 关闭配对窗口（之后到达的配对请求一律拒绝）。
    pub fn cancel_pairing(&self) {
        let mut state = self.inner.state.lock().unwrap();
        state.pending_pair = None;
        let _ = self.inner.store.save(&state);
    }

    /// 给已配对的对端发一条明文消息（由本库加密）。`notify` 为真时，对端离线也由中转暂存。
    /// 返回 Ok 只表示已进入发送队列；没送达会收到 [`LinkEvent::Undelivered`]。
    pub fn send(&self, peer: &str, data: &[u8], notify: bool) -> Result<(), Error> {
        let i = &self.inner;
        let (my_id, prk) = {
            let state = i.state.lock().unwrap();
            let me = state.device.as_ref().ok_or(Error::NotConnected)?.id.clone();
            let rec = state.peers.get(peer).ok_or(Error::UnknownPeer)?;
            (me, decode32(&rec.prk)?)
        };
        // 加密和入队放在同一把锁里，保证序号和发送顺序一致。
        let mut senders = i.senders.lock().unwrap();
        let sender = senders.entry(peer.to_string()).or_insert_with(|| Sender::new(prk, &my_id, peer, i.start_counter));
        let out = i.out.lock().unwrap().clone().ok_or(Error::NotConnected)?;
        let env = sender.seal(data)?;
        let frame = json!({ "op": "send", "env": env, "notify": notify }).to_string();
        out.try_send(frame).map_err(|_| Error::Congested)
    }

    /// 解除与某个对端的配对（同时通知中转，对端会收到 unpaired）。
    pub async fn unpair(&self, peer: &str) -> Result<(), Error> {
        let i = &self.inner;
        let rec = i.state.lock().unwrap().peers.get(peer).cloned().ok_or(Error::UnknownPeer)?;
        match i.api.remove_pairing(&rec.pairing_id).await {
            Ok(()) | Err(Error::Server { status: 404, .. }) => {}
            Err(e) => return Err(e),
        }
        i.forget_peer(peer);
        Ok(())
    }

    pub fn shutdown(&self) {
        self.inner.stopped.store(true, Ordering::SeqCst);
        self.inner.stop.notify_waiters();
        self.inner.flush();
    }
}

fn decode32(text: &str) -> Result<[u8; 32], Error> {
    B64.decode(text).ok().and_then(|b| b.try_into().ok()).ok_or_else(|| Error::Store("密钥格式不对".into()))
}

impl Inner {
    fn save(&self, state: &State) { let _ = self.store.save(state); }

    fn flush(&self) {
        if self.dirty.swap(false, Ordering::SeqCst) {
            let state = self.state.lock().unwrap();
            self.save(&state);
        }
    }

    async fn flusher(self: Arc<Self>) {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(1)) => self.flush(),
                _ = self.stop.notified() => { self.flush(); return; }
            }
            if self.stopped.load(Ordering::SeqCst) { return; }
        }
    }

    async fn emit(&self, event: LinkEvent) { let _ = self.events.send(event).await; }

    async fn ensure_identity(&self) -> Result<DeviceIdentity, Error> {
        let _guard = self.identity_lock.lock().await;
        if let Some(d) = self.state.lock().unwrap().device.clone() {
            return Ok(d);
        }
        let key = DeviceKey::generate();
        let id = self.api.register_device("desktop", &self.cfg.device_name, &B64.encode(key.public_bytes())).await?;
        let identity = DeviceIdentity { id, name: self.cfg.device_name.clone(), private_key: B64.encode(key.to_bytes()) };
        let mut state = self.state.lock().unwrap();
        state.device = Some(identity.clone());
        self.store.save(&state)?;
        Ok(identity)
    }

    fn forget_peer(&self, peer: &str) {
        self.senders.lock().unwrap().remove(peer);
        self.guards.lock().unwrap().remove(peer);
        self.online.lock().unwrap().remove(peer);
        let mut state = self.state.lock().unwrap();
        state.peers.remove(peer);
        self.save(&state);
    }

    /// 设备在中转被撤销：本地身份和所有配对作废，下次连接会重新登记成新设备。
    fn wipe_identity(&self) {
        self.senders.lock().unwrap().clear();
        self.guards.lock().unwrap().clear();
        self.online.lock().unwrap().clear();
        let mut state = self.state.lock().unwrap();
        state.device = None;
        state.peers.clear();
        state.pending_pair = None;
        self.save(&state);
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
        // 错过的配对请求（桌面离线时手机扫了码）从列表里补。
        {
            let me = self.clone();
            let id = identity.id.clone();
            tokio::spawn(async move {
                if let Ok(list) = me.api.pending(&id).await {
                    for p in list {
                        me.clone().on_pair_request(p.pairing_id, p.peer, p.proof).await;
                    }
                }
            });
        }

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
        self.flush();
        let peers: Vec<String> = self.online.lock().unwrap().drain().collect();
        for peer in peers { self.emit(LinkEvent::PeerOnline { peer, online: false }).await; }
        self.emit(LinkEvent::Disconnected).await;
        end
    }

    /// 返回 `Some(true)` 表示应停止重连，`Some(false)` 表示结束本次连接后重试，`None` 继续。
    async fn handle_frame(self: &Arc<Self>, text: &str) -> Option<bool> {
        let v: Value = serde_json::from_str(text).ok()?;
        match v["op"].as_str()? {
            "presence" => {
                let peer = v["device"].as_str()?.to_string();
                let online = v["online"].as_bool()?;
                {
                    let mut set = self.online.lock().unwrap();
                    if online { set.insert(peer.clone()); } else { set.remove(&peer); }
                }
                self.emit(LinkEvent::PeerOnline { peer, online }).await;
            }
            "pair.request" => {
                let phone: PeerInfo = serde_json::from_value(v["phone"].clone()).ok()?;
                let (id, proof) = (v["pairing_id"].as_str()?.to_string(), v["proof"].as_str()?.to_string());
                tokio::spawn(self.clone().on_pair_request(id, phone, proof));
            }
            "deliver" => {
                let env: Envelope = serde_json::from_value(v["env"].clone()).ok()?;
                self.on_deliver(env).await;
            }
            "undelivered" => {
                self.emit(LinkEvent::Undelivered { peer: v["to"].as_str()?.into(), seq: v["seq"].as_u64().unwrap_or(0) }).await;
            }
            "unpaired" => {
                let peer = v["device"].as_str()?.to_string();
                if self.state.lock().unwrap().peers.contains_key(&peer) {
                    self.forget_peer(&peer);
                    self.emit(LinkEvent::Unpaired { peer }).await;
                }
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

    async fn on_pair_request(self: Arc<Self>, pairing_id: String, phone: PeerInfo, proof_b64: String) {
        if !self.handled_pairings.lock().unwrap().insert(pairing_id.clone()) { return; }
        let reject = |reason: &'static str| {
            let me = self.clone();
            let id = pairing_id.clone();
            async move {
                let _ = me.api.reject(&id).await;
                me.emit(LinkEvent::PairRejected { reason: reason.into() }).await;
            }
        };
        let (identity, pending) = {
            let state = self.state.lock().unwrap();
            (state.device.clone(), state.pending_pair.clone())
        };
        let (Some(identity), Some(pending)) = (identity, pending) else { return reject("no_window").await };
        if pending.expires_at_ms < now_ms() { return reject("no_window").await; }
        let (Ok(secret), Ok(phone_pub), Ok(proof)) = (decode32(&pending.secret), B64.decode(&phone.public_key), B64.decode(&proof_b64)) else { return reject("bad_proof").await };
        if verify_proof(&secret, &phone.id, &identity.id, &phone_pub, &proof).is_err() { return reject("bad_proof").await; }
        let Ok(private) = decode32(&identity.private_key) else { return };
        let Ok(key) = DeviceKey::from_bytes(&private) else { return };
        let Ok(prk) = derive_pairing_key(&key, &phone_pub, &secret) else { return reject("bad_proof").await };
        if let Err(e) = self.api.confirm(&pairing_id).await {
            self.emit(LinkEvent::ServerError { code: format!("confirm_failed:{e}") }).await;
            return;
        }
        {
            let mut state = self.state.lock().unwrap();
            state.pending_pair = None;
            state.peers.insert(phone.id.clone(), PeerRecord { pairing_id: pairing_id.clone(), peer_id: phone.id.clone(), peer_name: phone.name.clone(), peer_public: phone.public_key.clone(), prk: B64.encode(prk), replay: None });
            self.save(&state);
        }
        self.senders.lock().unwrap().remove(&phone.id);
        self.guards.lock().unwrap().insert(phone.id.clone(), ReplayGuard::new());
        let online = self.online.lock().unwrap().contains(&phone.id);
        self.emit(LinkEvent::Paired(PeerSummary { id: phone.id, name: phone.name, online })).await;
    }

    async fn on_deliver(&self, env: Envelope) {
        let (my_id, rec) = {
            let state = self.state.lock().unwrap();
            (state.device.as_ref().map(|d| d.id.clone()), state.peers.get(&env.from).cloned())
        };
        let (Some(my_id), Some(rec)) = (my_id, rec) else { return };
        if env.to != my_id { return; }
        let Ok(prk) = decode32(&rec.prk) else { return };
        let plain = match open(&prk, &env.from, &env.to, &env) {
            Ok(p) => p,
            Err(e) => return self.emit(LinkEvent::Dropped { peer: env.from.clone(), reason: format!("decrypt:{e}") }).await,
        };
        let progress = {
            let mut guards = self.guards.lock().unwrap();
            let guard = guards.entry(env.from.clone()).or_default();
            guard.check(&env).map(|_| guard.state().cloned())
        };
        match progress {
            Ok(state) => {
                if let Some(rec) = self.state.lock().unwrap().peers.get_mut(&env.from) { rec.replay = state; }
                self.dirty.store(true, Ordering::SeqCst);
                self.emit(LinkEvent::Message { peer: env.from, data: plain }).await;
            }
            Err(e) => self.emit(LinkEvent::Dropped { peer: env.from, reason: format!("replay:{e}") }).await,
        }
    }
}

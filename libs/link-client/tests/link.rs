//! 端到端集成测试：真实 server-link（真实 Postgres、真实 WebSocket）+ 本库的桌面端 + 测试里手写的“手机端”。
//! 需要 TEST_DATABASE_URL；没有则跳过。
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use server_link::testing::TestServer;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
    MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;
use xiaoban_link_client::{static_token, store::MemoryStore, Link, LinkConfig, LinkEvent, TokenFn};
use xiaoban_link_protocol::{
    envelope::{open, Envelope, Sender},
    keys::DeviceKey,
    pairing::{derive_pairing_key, make_proof, QrPayload},
    replay::ReplayGuard,
};

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn server() -> Option<TestServer> {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return None;
    };
    Some(TestServer::start(&url).await)
}

async fn event(rx: &mut mpsc::Receiver<LinkEvent>, pred: impl Fn(&LinkEvent) -> bool) -> LinkEvent {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Some(e)) if pred(&e) => return e,
            Ok(Some(_)) => continue,
            _ => panic!("等事件超时"),
        }
    }
}

async fn no_event(rx: &mut mpsc::Receiver<LinkEvent>, pred: impl Fn(&LinkEvent) -> bool) {
    let until = tokio::time::Instant::now() + Duration::from_millis(600);
    while let Ok(Some(e)) = tokio::time::timeout_at(until, rx.recv()).await {
        assert!(!pred(&e), "不该出现的事件：{e:?}");
    }
}

async fn start_desktop(srv: &TestServer, uid: Uuid, store: Arc<MemoryStore>) -> (Link, mpsc::Receiver<LinkEvent>) {
    Link::start(LinkConfig::new(&srv.base, "办公室电脑"), store, static_token(srv.desktop_token(uid))).await.unwrap()
}

struct Phone {
    #[allow(dead_code)]
    key: DeviceKey,
    id: String,
    token: String,
    base: String,
    ws_base: String,
    desktop_id: String,
    prk: [u8; 32],
}

/// 手机端的配对过程：扫码 → 登记 → 兑换（带证明）。返回 pairing_id。
async fn phone_pair(srv: &TestServer, uid: Uuid, qr_text: &str, tamper_proof: bool) -> (Phone, String) {
    let qr = QrPayload::decode(qr_text).unwrap();
    let token = srv.phone_token(uid);
    let http = reqwest::Client::new();
    let key = DeviceKey::generate();
    let r: Value = http.post(format!("{}/devices", srv.base)).bearer_auth(&token).json(&json!({"kind":"phone","name":"我的手机","public_key":B64.encode(key.public_bytes())})).send().await.unwrap().json().await.unwrap();
    let id = r["device_id"].as_str().unwrap().to_string();
    let secret = qr.secret().unwrap();
    // 攻击者不知道二维码里的密钥，用别的密钥做证明。
    let proof_secret = if tamper_proof { [9u8; 32] } else { secret };
    let proof = make_proof(&proof_secret, &id, &qr.d, &key.public_bytes());
    let resp = http.post(format!("{}/pair", srv.base)).bearer_auth(&token).json(&json!({"code": qr.c, "device_id": id, "proof": B64.encode(proof)})).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 202);
    let body: Value = resp.json().await.unwrap();
    // 手机核对中转给的桌面公钥与二维码一致。
    assert_eq!(body["desktop"]["public_key"], qr.k.as_str());
    let desktop_pub = qr.desktop_public().unwrap();
    let prk = derive_pairing_key(&key, &desktop_pub, &secret).unwrap();
    (Phone { key, id, token, base: srv.base.clone(), ws_base: srv.ws_base.clone(), desktop_id: qr.d, prk }, body["pairing_id"].as_str().unwrap().to_string())
}

impl Phone {
    async fn connect(&self) -> Ws {
        let mut req = format!("{}/ws?device_id={}", self.ws_base, self.id).into_client_request().unwrap();
        req.headers_mut().insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap());
        connect_async(req).await.unwrap().0
    }
    fn sender(&self, start: u64) -> Sender { Sender::new(self.prk, &self.id, &self.desktop_id, start) }
    async fn send(&self, ws: &mut Ws, env: &Envelope, notify: bool) {
        ws.send(Message::Text(json!({"op":"send","env":env,"notify":notify}).to_string().into())).await.unwrap();
    }
    async fn wait_op(&self, ws: &mut Ws, op: &str) -> Value {
        for _ in 0..30 {
            match tokio::time::timeout(Duration::from_secs(5), ws.next()).await {
                Ok(Some(Ok(Message::Text(t)))) => {
                    let v: Value = serde_json::from_str(t.as_str()).unwrap();
                    if v["op"] == op { return v; }
                }
                Ok(Some(Ok(_))) => {}
                _ => panic!("等 {op} 超时"),
            }
        }
        panic!("没等到 {op}");
    }
    async fn recv_plain(&self, ws: &mut Ws, guard: &mut ReplayGuard) -> Vec<u8> {
        let v = self.wait_op(ws, "deliver").await;
        let env: Envelope = serde_json::from_value(v["env"].clone()).unwrap();
        let plain = open(&self.prk, &self.desktop_id, &self.id, &env).unwrap();
        guard.check(&env).unwrap();
        plain
    }
    async fn http_delete(&self, path: &str) -> u16 {
        reqwest::Client::new().delete(format!("{}{path}", self.base)).bearer_auth(&self.token).send().await.unwrap().status().as_u16()
    }
}

/// 配对并连上，返回桌面句柄、事件流、手机、手机的 WebSocket、手机的 pairing_id。
async fn paired(srv: &TestServer, uid: Uuid, store: Arc<MemoryStore>) -> (Link, mpsc::Receiver<LinkEvent>, Phone, Ws, String) {
    let (link, mut rx) = start_desktop(srv, uid, store).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Connected)).await;
    let offer = link.begin_pairing().await.unwrap();
    let (phone, pairing_id) = phone_pair(srv, uid, &offer.qr_text, false).await;
    let LinkEvent::Paired(peer) = event(&mut rx, |e| matches!(e, LinkEvent::Paired(_))).await else { unreachable!() };
    assert_eq!(peer.id, phone.id);
    let mut ws = phone.connect().await;
    phone.wait_op(&mut ws, "ready").await;
    event(&mut rx, |e| matches!(e, LinkEvent::PeerOnline { online: true, .. })).await;
    (link, rx, phone, ws, pairing_id)
}

#[tokio::test]
async fn pair_then_talk_both_ways_through_the_relay() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let (link, mut rx, phone, mut ws, _) = paired(&srv, uid, Arc::new(MemoryStore::new())).await;
    assert_eq!(link.peers().len(), 1);
    assert!(link.peers()[0].online);

    // 手机 → 桌面
    let mut p_tx = phone.sender(1);
    phone.send(&mut ws, &p_tx.seal("帮我整理下载文件夹".as_bytes()).unwrap(), false).await;
    let LinkEvent::Message { peer, data } = event(&mut rx, |e| matches!(e, LinkEvent::Message { .. })).await else { unreachable!() };
    assert_eq!((peer.as_str(), data.as_slice()), (phone.id.as_str(), "帮我整理下载文件夹".as_bytes()));

    // 桌面 → 手机（多条保持顺序）
    let mut guard = ReplayGuard::new();
    for i in 0..5 {
        link.send(&phone.id, format!("进度 {i}").as_bytes(), false).unwrap();
    }
    for i in 0..5 {
        assert_eq!(phone.recv_plain(&mut ws, &mut guard).await, format!("进度 {i}").as_bytes());
    }

    // 不认识的对端、发给自己都不行。
    assert!(link.send("phn_unknown", b"x", false).is_err());
    link.shutdown();
}

#[tokio::test]
async fn wrong_proof_or_closed_window_is_rejected_and_nothing_is_stored() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let (link, mut rx) = start_desktop(&srv, uid, Arc::new(MemoryStore::new())).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Connected)).await;

    // 拿到配对码但不知道二维码里的一次性密钥：证明对不上。
    let offer = link.begin_pairing().await.unwrap();
    let (evil, _) = phone_pair(&srv, uid, &offer.qr_text, true).await;
    let LinkEvent::PairRejected { reason } = event(&mut rx, |e| matches!(e, LinkEvent::PairRejected { .. })).await else { unreachable!() };
    assert_eq!(reason, "bad_proof");
    assert!(link.peers().is_empty());
    // 中转那边也没有激活。
    let mut ws = evil.connect().await;
    evil.wait_op(&mut ws, "ready").await;
    let stray = evil.sender(1).seal(b"x").unwrap();
    evil.send(&mut ws, &stray, false).await;
    assert_eq!(evil.wait_op(&mut ws, "error").await["code"], "not_paired");

    // 真正的手机随后还能用同一个二维码配对成功（假请求没有烧掉窗口）。
    let (good, _) = phone_pair(&srv, uid, &offer.qr_text, false).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Paired(_))).await;
    assert_eq!(link.peers()[0].id, good.id);

    // 窗口关闭后的请求被拒绝。
    let offer2 = link.begin_pairing().await.unwrap();
    link.cancel_pairing();
    let (_late, _) = phone_pair(&srv, uid, &offer2.qr_text, false).await;
    let LinkEvent::PairRejected { reason } = event(&mut rx, |e| matches!(e, LinkEvent::PairRejected { .. })).await else { unreachable!() };
    assert_eq!(reason, "no_window");
    assert_eq!(link.peers().len(), 1);
    link.shutdown();
}

#[tokio::test]
async fn state_survives_restart_and_old_envelopes_cannot_be_replayed() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let store = Arc::new(MemoryStore::new());
    let (link, mut rx, phone, mut ws, _) = paired(&srv, uid, store.clone()).await;

    let mut p_tx = phone.sender(1);
    let first = p_tx.seal(b"one").unwrap();
    phone.send(&mut ws, &first, false).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Message { .. })).await;
    // 桌面发一条，记下它的会话，之后重启要换新会话。
    let mut guard = ReplayGuard::new();
    link.send(&phone.id, b"before restart", false).unwrap();
    assert_eq!(phone.recv_plain(&mut ws, &mut guard).await, b"before restart");
    let old_state = guard.state().cloned();

    link.shutdown();
    tokio::time::sleep(Duration::from_millis(300)).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Disconnected)).await;

    // 重启：配对关系还在，启动计数加一。
    let (link2, mut rx2) = start_desktop(&srv, uid, store.clone()).await;
    event(&mut rx2, |e| matches!(e, LinkEvent::Connected)).await;
    assert_eq!(link2.peers().len(), 1, "配对关系从存储恢复");
    assert_eq!(store_start_counter(&store), 2);
    let mut ws = phone.connect().await;
    phone.wait_op(&mut ws, "ready").await;
    event(&mut rx2, |e| matches!(e, LinkEvent::PeerOnline { online: true, .. })).await;

    // 把重启前截获的信封重放：被拒绝，不会当成新消息。
    phone.send(&mut ws, &first, false).await;
    let LinkEvent::Dropped { reason, .. } = event(&mut rx2, |e| matches!(e, LinkEvent::Dropped { .. } | LinkEvent::Message { .. })).await else { panic!("重放不应被当作消息") };
    assert!(reason.starts_with("replay:"), "{reason}");
    // 手机继续用同一个会话发新序号，仍然可以。
    phone.send(&mut ws, &p_tx.seal(b"two").unwrap(), false).await;
    let LinkEvent::Message { data, .. } = event(&mut rx2, |e| matches!(e, LinkEvent::Message { .. })).await else { unreachable!() };
    assert_eq!(data, b"two");

    // 桌面重启后用新会话发消息，手机侧按启动计数接受，并拒绝旧会话的重放。
    link2.send(&phone.id, b"after restart", false).unwrap();
    let v = phone.wait_op(&mut ws, "deliver").await;
    let env: Envelope = serde_json::from_value(v["env"].clone()).unwrap();
    let mut resumed = ReplayGuard::restore(old_state);
    resumed.check(&env).unwrap();
    assert_eq!(open(&phone.prk, &phone.desktop_id, &phone.id, &env).unwrap(), b"after restart");
    link2.shutdown();
}

fn store_start_counter(store: &MemoryStore) -> u64 {
    use xiaoban_link_client::store::LinkStore;
    store.load().unwrap().start_counter
}

#[tokio::test]
async fn notifications_sent_while_desktop_is_offline_arrive_after_it_starts() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let store = Arc::new(MemoryStore::new());
    let (link, mut rx, phone, mut ws, _) = paired(&srv, uid, store.clone()).await;
    link.shutdown();
    event(&mut rx, |e| matches!(e, LinkEvent::Disconnected)).await;
    phone.wait_op(&mut ws, "presence").await;

    let mut tx = phone.sender(1);
    phone.send(&mut ws, &tx.seal(b"command").unwrap(), false).await;
    assert_eq!(phone.wait_op(&mut ws, "undelivered").await["reason"], "offline", "指令不暂存");
    phone.send(&mut ws, &tx.seal(b"remember me").unwrap(), true).await;
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (link2, mut rx2) = start_desktop(&srv, uid, store).await;
    let LinkEvent::Message { data, .. } = event(&mut rx2, |e| matches!(e, LinkEvent::Message { .. })).await else { unreachable!() };
    assert_eq!(data, b"remember me");
    link2.shutdown();
}

#[tokio::test]
async fn unpairing_from_either_side_clears_both() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    // 手机先解除
    let (link, mut rx, phone, mut ws, pairing_id) = paired(&srv, uid, Arc::new(MemoryStore::new())).await;
    assert_eq!(phone.http_delete(&format!("/pairings/{pairing_id}")).await, 204);
    event(&mut rx, |e| matches!(e, LinkEvent::Unpaired { .. })).await;
    assert!(link.peers().is_empty());
    assert!(link.send(&phone.id, b"x", false).is_err());
    link.shutdown();

    // 桌面先解除
    let uid2 = Uuid::new_v4();
    let (link, mut rx, phone, mut ws2, _) = paired(&srv, uid2, Arc::new(MemoryStore::new())).await;
    link.unpair(&phone.id).await.unwrap();
    assert!(link.peers().is_empty());
    assert_eq!(phone.wait_op(&mut ws2, "unpaired").await["device"], phone.desktop_id.as_str());
    no_event(&mut rx, |e| matches!(e, LinkEvent::Message { .. })).await;
    link.shutdown();
    let _ = &mut ws;
}

#[tokio::test]
async fn keeps_retrying_until_a_token_is_available() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let real = srv.desktop_token(uid);
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    // 前几次取不到令牌（比如还没登录/正在刷新），之后才有。
    let token: TokenFn = Arc::new(move || {
        let n = c.fetch_add(1, Ordering::SeqCst);
        let t = real.clone();
        Box::pin(async move { if n < 3 { None } else { Some(t) } })
    });
    let mut cfg = LinkConfig::new(&srv.base, "电脑");
    cfg.backoff_initial = Duration::from_millis(50);
    cfg.backoff_max = Duration::from_millis(200);
    let (link, mut rx) = Link::start(cfg, Arc::new(MemoryStore::new()), token).await.unwrap();
    event(&mut rx, |e| matches!(e, LinkEvent::Connected)).await;
    assert!(calls.load(Ordering::SeqCst) >= 4);
    link.shutdown();
}

#[tokio::test]
async fn pairing_right_after_start_registers_only_one_device() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    // 不等连接建立就配对：连接循环和 begin_pairing 同时需要设备身份，只能登记一次，且二维码里的设备就是连上去的那台。
    let (link, mut rx) = start_desktop(&srv, uid, Arc::new(MemoryStore::new())).await;
    let offer = link.begin_pairing().await.unwrap();
    let (phone, _) = phone_pair(&srv, uid, &offer.qr_text, false).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Paired(_))).await;
    let devices: Value = reqwest::Client::new().get(format!("{}/devices", srv.base)).bearer_auth(srv.desktop_token(uid)).send().await.unwrap().json().await.unwrap();
    let desktops = devices["items"].as_array().unwrap().iter().filter(|d| d["kind"] == "desktop").count();
    assert_eq!(desktops, 1, "只应登记一台桌面设备");
    let mut ws = phone.connect().await;
    phone.wait_op(&mut ws, "ready").await;
    event(&mut rx, |e| matches!(e, LinkEvent::PeerOnline { online: true, .. })).await;
    let mut tx = phone.sender(1);
    phone.send(&mut ws, &tx.seal(b"hi").unwrap(), false).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Message { .. })).await;
    link.shutdown();
}

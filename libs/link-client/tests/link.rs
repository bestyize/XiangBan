//! 端到端集成测试：真实 server-link（真实 Postgres、真实 WebSocket）+ 两个本库的客户端（一个当电脑，一个当手机）。
//! 需要 TEST_DATABASE_URL；没有则跳过。
use server_link::testing::TestServer;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::mpsc;
use uuid::Uuid;
use xiaoban_link_client::{static_token, store::{FileStore, LinkStore, MemoryStore}, Link, LinkConfig, LinkEvent, TokenFn};

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

/// 对端在线状态可能随 `peers` 帧或 `presence` 帧到达，用状态而不是事件来等。
async fn wait_peer_online(link: &Link, online: bool) {
    for _ in 0..160 {
        if link.peers().first().is_some_and(|p| p.online == online) { return; }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("对端没有变成 online={online}：{:?}", link.peers());
}

async fn start(srv: &TestServer, token: String, kind: &str, name: &str, store: Arc<dyn LinkStore>) -> (Link, mpsc::Receiver<LinkEvent>) {
    let mut cfg = LinkConfig::new(&srv.base, name);
    cfg.kind = kind.into();
    cfg.backoff_initial = Duration::from_millis(50);
    cfg.backoff_max = Duration::from_millis(200);
    Link::start(cfg, store, static_token(token)).await.unwrap()
}

async fn desktop(srv: &TestServer, uid: Uuid) -> (Link, mpsc::Receiver<LinkEvent>) {
    start(srv, srv.desktop_token(uid), "desktop", "办公室电脑", Arc::new(MemoryStore::new())).await
}
async fn phone(srv: &TestServer, uid: Uuid) -> (Link, mpsc::Receiver<LinkEvent>) {
    start(srv, srv.phone_token(uid), "phone", "我的手机", Arc::new(MemoryStore::new())).await
}

fn text(e: &LinkEvent) -> String {
    match e {
        LinkEvent::Message { data, .. } => String::from_utf8_lossy(data).to_string(),
        other => panic!("不是消息：{other:?}"),
    }
}

#[tokio::test]
async fn same_account_devices_find_each_other_and_exchange_messages_in_order() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let (desk, mut desk_rx) = desktop(&srv, uid).await;
    let (phn, mut phone_rx) = phone(&srv, uid).await;

    // 不需要配对：双方都直接看到对端，且知道它在线。
    let seen = event(&mut phone_rx, |e| matches!(e, LinkEvent::PeerAdded(_))).await;
    let LinkEvent::PeerAdded(desktop_peer) = seen else { unreachable!() };
    assert_eq!(desktop_peer.name, "办公室电脑");
    event(&mut desk_rx, |e| matches!(e, LinkEvent::PeerAdded(p) if p.name == "我的手机")).await;
    wait_peer_online(&phn, true).await;
    assert_eq!(phn.peers().len(), 1);
    assert_eq!(desk.peers()[0].name, "我的手机");

    // 手机 → 电脑：中文、较大的消息、连发 50 条保持顺序。
    let desk_id = desktop_peer.id.clone();
    let phone_id = phn.device_id().unwrap();
    phn.send(&desk_id, "你好，小伴".as_bytes(), false).unwrap();
    assert_eq!(text(&event(&mut desk_rx, |e| matches!(e, LinkEvent::Message { .. })).await), "你好，小伴");
    let big = "字".repeat(60_000);
    phn.send(&desk_id, big.as_bytes(), false).unwrap();
    assert_eq!(text(&event(&mut desk_rx, |e| matches!(e, LinkEvent::Message { .. })).await).chars().count(), 60_000);
    for i in 0..50 {
        desk.send(&phone_id, format!("m{i}").as_bytes(), false).unwrap();
    }
    for i in 0..50 {
        assert_eq!(text(&event(&mut phone_rx, |e| matches!(e, LinkEvent::Message { .. })).await), format!("m{i}"));
    }
    // 来源是中转按认证填的设备编号。
    desk.send(&phone_id, b"who", false).unwrap();
    let LinkEvent::Message { peer, .. } = event(&mut phone_rx, |e| matches!(e, LinkEvent::Message { .. })).await else { unreachable!() };
    assert_eq!(peer, desk.device_id().unwrap());

    // 不是文本、不认识的对端：发送前就报错。
    assert!(phn.send(&desk_id, &[0xff, 0xfe], false).is_err());
    assert!(phn.send("dsk_nobody", b"x", false).is_err());
    desk.shutdown();
    phn.shutdown();
}

#[tokio::test]
async fn other_accounts_never_see_each_other() {
    let Some(srv) = server().await else { return };
    let (alice, bob) = (Uuid::new_v4(), Uuid::new_v4());
    let (desk, mut desk_rx) = desktop(&srv, alice).await;
    let (_bob_phone, _bob_rx) = phone(&srv, bob).await;
    event(&mut desk_rx, |e| matches!(e, LinkEvent::Connected)).await;
    no_event(&mut desk_rx, |e| matches!(e, LinkEvent::PeerAdded(_))).await;
    assert!(desk.peers().is_empty());
    desk.shutdown();
}

#[tokio::test]
async fn offline_notifications_are_delivered_on_reconnect_but_commands_are_not() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let (desk, mut desk_rx) = desktop(&srv, uid).await;
    let store: Arc<dyn LinkStore> = Arc::new(MemoryStore::new());
    let (phn, _phone_rx) = start(&srv, srv.phone_token(uid), "phone", "我的手机", store.clone()).await;
    wait_peer_online(&desk, true).await;
    let phone_id = phn.device_id().unwrap();
    phn.shutdown();
    wait_peer_online(&desk, false).await;

    desk.send(&phone_id, "指令".as_bytes(), false).unwrap();
    event(&mut desk_rx, |e| matches!(e, LinkEvent::Undelivered { .. })).await;
    desk.send(&phone_id, "任务做完了".as_bytes(), true).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 手机用同一份存储重启：还是同一台设备，补收到通知，没有收到指令。
    let (phn2, mut phone_rx2) = start(&srv, srv.phone_token(uid), "phone", "我的手机", store).await;
    assert_eq!(text(&event(&mut phone_rx2, |e| matches!(e, LinkEvent::Message { .. })).await), "任务做完了");
    assert_eq!(phn2.device_id().unwrap(), phone_id);
    no_event(&mut phone_rx2, |e| matches!(e, LinkEvent::Message { .. })).await;
    desk.shutdown();
    phn2.shutdown();
}

#[tokio::test]
async fn removing_a_device_notifies_it_and_it_registers_again_as_a_new_one() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let (desk, mut desk_rx) = desktop(&srv, uid).await;
    let (phn, mut phone_rx) = phone(&srv, uid).await;
    let LinkEvent::PeerAdded(p) = event(&mut desk_rx, |e| matches!(e, LinkEvent::PeerAdded(_))).await else { unreachable!() };
    wait_peer_online(&phn, true).await;
    let old_id = phn.device_id().unwrap();
    assert_eq!(p.id, old_id);

    desk.remove_device(&old_id).await.unwrap();
    assert!(desk.peers().is_empty());
    event(&mut phone_rx, |e| matches!(e, LinkEvent::Revoked)).await;
    // 手机会自动重新登记成一台新设备，电脑重新看到它。
    let LinkEvent::PeerAdded(again) = event(&mut desk_rx, |e| matches!(e, LinkEvent::PeerAdded(p) if p.id != old_id)).await else { unreachable!() };
    assert_ne!(again.id, old_id);
    desk.shutdown();
    phn.shutdown();
}

#[tokio::test]
async fn keeps_retrying_until_a_token_is_available_and_registers_only_one_device() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let calls = Arc::new(AtomicUsize::new(0));
    let (c, real) = (calls.clone(), srv.desktop_token(uid));
    let token: TokenFn = Arc::new(move || {
        let (c, real) = (c.clone(), real.clone());
        Box::pin(async move { if c.fetch_add(1, Ordering::SeqCst) < 3 { None } else { Some(real) } })
    });
    let mut cfg = LinkConfig::new(&srv.base, "电脑");
    cfg.backoff_initial = Duration::from_millis(30);
    cfg.backoff_max = Duration::from_millis(60);
    let (link, mut rx) = Link::start(cfg, Arc::new(MemoryStore::new()), token).await.unwrap();
    event(&mut rx, |e| matches!(e, LinkEvent::Connected)).await;
    assert!(calls.load(Ordering::SeqCst) >= 4);
    let devices: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM devices WHERE uid = $1").bind(uid).fetch_one(&srv.pool).await.unwrap();
    assert_eq!(devices, 1);
    link.shutdown();
}

#[tokio::test]
async fn state_file_survives_restart_and_ignores_fields_from_the_old_pairing_version() {
    let Some(srv) = server().await else { return };
    let uid = Uuid::new_v4();
    let dir = std::env::temp_dir().join(format!("xiaoban-link-test-{}", Uuid::new_v4().simple()));
    let path = dir.join("state.json");
    let store: Arc<dyn LinkStore> = Arc::new(FileStore::new(&path));
    let (link, mut rx) = start(&srv, srv.desktop_token(uid), "desktop", "电脑", store.clone()).await;
    event(&mut rx, |e| matches!(e, LinkEvent::Connected)).await;
    let id = link.device_id().unwrap();
    link.shutdown();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let (again, mut rx2) = start(&srv, srv.desktop_token(uid), "desktop", "电脑", store).await;
    event(&mut rx2, |e| matches!(e, LinkEvent::Connected)).await;
    assert_eq!(again.device_id().unwrap(), id, "重启后仍是同一台设备");
    again.shutdown();

    // 旧版本的状态文件带着私钥、配对密钥等字段：读得进来，多余字段忽略。
    std::fs::write(&path, r#"{"device":{"id":"dsk_old","name":"旧电脑","private_key":"x"},"start_counter":9,"peers":{},"pending_pair":null}"#).unwrap();
    let state = FileStore::new(&path).load().unwrap();
    assert_eq!(state.device.unwrap().id, "dsk_old");
    assert!(state.install_id.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

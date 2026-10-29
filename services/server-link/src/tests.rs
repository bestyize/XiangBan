//! 集成测试：真实 Postgres（TEST_DATABASE_URL，每个测试独立 schema）、真实 HTTP 与 WebSocket 连接。
use crate::testing::TestServer;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
    MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Env {
    srv: TestServer,
    base: String,
    ws_base: String,
    http: reqwest::Client,
}

impl Env {
    fn token_aud(&self, uid: Uuid, aud: &str, ttl: i64) -> String { self.srv.token_aud(uid, aud, ttl) }
    fn phone_token(&self, uid: Uuid) -> String { self.srv.phone_token(uid) }
    fn desktop_token(&self, uid: Uuid) -> String { self.srv.desktop_token(uid) }

    async fn call(&self, method: &str, path: &str, token: Option<&str>, body: Value) -> (u16, Value) {
        let m = reqwest::Method::from_bytes(method.as_bytes()).unwrap();
        let mut req = self.http.request(m, format!("{}{path}", self.base));
        if let Some(t) = token { req = req.bearer_auth(t); }
        if !body.is_null() { req = req.json(&body); }
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap();
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    async fn ws(&self, token: &str, device: &str) -> Ws {
        let mut req = format!("{}/ws?device_id={device}", self.ws_base).into_client_request().unwrap();
        req.headers_mut().insert("Authorization", HeaderValue::from_str(&format!("Bearer {token}")).unwrap());
        connect_async(req).await.expect("ws connect").0
    }

    async fn register(&self, token: &str, kind: &str, name: &str) -> String {
        let (s, b) = self.call("POST", "/devices", Some(token), json!({"kind": kind, "name": name, "install_id": format!("test-install-{}", Uuid::new_v4().simple())})).await;
        assert!(s == 201 || s == 200, "{s} {b}");
        b["device_id"].as_str().unwrap().to_string()
    }
}

async fn next_json(ws: &mut Ws) -> Option<Value> {
    loop {
        match tokio::time::timeout(Duration::from_secs(3), ws.next()).await {
            Ok(Some(Ok(Message::Text(t)))) => return serde_json::from_str(t.as_str()).ok(),
            Ok(Some(Ok(_))) => continue,
            _ => return None,
        }
    }
}

/// 读到指定 op 的帧为止，途中的其他帧丢弃。
async fn expect_op(ws: &mut Ws, op: &str) -> Value {
    for _ in 0..20 {
        match next_json(ws).await {
            Some(v) if v["op"] == op => return v,
            Some(_) => continue,
            None => panic!("等 {op} 超时或连接已关闭"),
        }
    }
    panic!("没等到 {op}");
}

async fn send_text(ws: &mut Ws, v: Value) { ws.send(Message::Text(v.to_string().into())).await.unwrap(); }

async fn setup() -> Option<Env> {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return None;
    };
    let srv = TestServer::start_with(&url, |c| { c.notify_queue_cap = 2; }).await;
    Some(Env { base: srv.base.clone(), ws_base: srv.ws_base.clone(), http: reqwest::Client::new(), srv })
}

#[tokio::test]
async fn devices_api_validation_and_ownership() {
    let Some(env) = setup().await else { return };
    let (alice, bob) = (Uuid::new_v4(), Uuid::new_v4());
    let (ta, tb) = (env.phone_token(alice), env.phone_token(bob));

    assert_eq!(env.call("POST", "/devices", None, json!({})).await.0, 401);
    // 令牌的受众不在配置的客户端里，或已过期。
    assert_eq!(env.call("GET", "/devices", Some(&env.token_aud(alice, "forum-web", 600)), Value::Null).await.0, 401);
    assert_eq!(env.call("GET", "/devices", Some(&env.token_aud(alice, "xiaoban-android", -600)), Value::Null).await.0, 401, "过期");

    let id = "install-aaaaaaaaaaaaaaaa";
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "tablet", "name": "x", "install_id": id})).await.0, 400);
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "  ", "install_id": id})).await.0, 400);
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "x", "install_id": "short"})).await.0, 400);
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "x", "install_id": "has space has space"})).await.0, 400);

    let (s, first) = env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "我的手机", "install_id": id})).await;
    assert_eq!(s, 201);
    let (s, again) = env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "改个名", "install_id": id})).await;
    assert_eq!(s, 200, "同一安装编号幂等");
    assert_eq!(first["device_id"], again["device_id"]);
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "desktop", "name": "x", "install_id": id})).await.0, 400, "安装编号不能换类型");
    let (_, list) = env.call("GET", "/devices", Some(&ta), Value::Null).await;
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["name"], "改个名");
    assert_eq!(list["items"][0]["online"], false);
    assert!(list["items"][0].get("public_key").is_none());

    // 别的账号看不到、删不了；同一个安装编号在别的账号下是另一台设备。
    let (_, other) = env.call("GET", "/devices", Some(&tb), Value::Null).await;
    assert!(other["items"].as_array().unwrap().is_empty());
    let device = first["device_id"].as_str().unwrap();
    assert_eq!(env.call("POST", "/devices", Some(&tb), json!({"kind": "phone", "name": "别人", "install_id": id})).await.1["created"], true);
    assert_eq!(env.call("DELETE", &format!("/devices/{device}"), Some(&tb), Value::Null).await.0, 404);
    assert_eq!(env.call("DELETE", &format!("/devices/{device}"), Some(&ta), Value::Null).await.0, 204);
    assert_eq!(env.call("DELETE", &format!("/devices/{device}"), Some(&ta), Value::Null).await.0, 404);
}

#[tokio::test]
async fn same_account_devices_find_each_other_and_exchange_messages() {
    let Some(env) = setup().await else { return };
    let (alice, mallory) = (Uuid::new_v4(), Uuid::new_v4());
    let (t_desk, t_phone, t_evil) = (env.desktop_token(alice), env.phone_token(alice), env.phone_token(mallory));
    let desktop = env.register(&t_desk, "desktop", "办公室电脑").await;
    let phone = env.register(&t_phone, "phone", "我的手机").await;
    let evil_phone = env.register(&t_evil, "phone", "别人的手机").await;

    // 电脑只看到同账号的手机，手机只看到同账号的电脑。
    let (_, seen_by_phone) = env.call("GET", &format!("/peers?device_id={phone}"), Some(&t_phone), Value::Null).await;
    assert_eq!(seen_by_phone["items"][0]["id"], desktop.as_str());
    assert_eq!(seen_by_phone["items"][0]["name"], "办公室电脑");
    assert_eq!(seen_by_phone["items"].as_array().unwrap().len(), 1);
    let (_, seen_by_desktop) = env.call("GET", &format!("/peers?device_id={desktop}"), Some(&t_desk), Value::Null).await;
    assert_eq!(seen_by_desktop["items"].as_array().unwrap().len(), 1, "别的账号的手机不可见");
    assert_eq!(env.call("GET", &format!("/peers?device_id={phone}"), Some(&t_evil), Value::Null).await.0, 404, "别人的设备编号当作不存在");

    let mut phone_ws = env.ws(&t_phone, &phone).await;
    expect_op(&mut phone_ws, "ready").await;
    let peers = expect_op(&mut phone_ws, "peers").await;
    assert_eq!(peers["items"][0]["id"], desktop.as_str());
    assert_eq!(peers["items"][0]["online"], false);

    // 电脑上线：手机收到 presence。
    let mut desk_ws = env.ws(&t_desk, &desktop).await;
    expect_op(&mut desk_ws, "ready").await;
    assert_eq!(expect_op(&mut desk_ws, "peers").await["items"][0]["online"], true);
    let presence = expect_op(&mut phone_ws, "presence").await;
    assert_eq!((presence["device"].as_str().unwrap(), presence["online"].as_bool().unwrap()), (desktop.as_str(), true));

    // 手机 → 电脑：内容原样转交，来源由中转填写。
    send_text(&mut phone_ws, json!({"op": "send", "to": desktop, "data": "{\"t\":\"hello\"}"})).await;
    let got = expect_op(&mut desk_ws, "deliver").await;
    assert_eq!(got["from"], phone.as_str());
    assert_eq!(got["data"], "{\"t\":\"hello\"}");
    // 电脑 → 手机，带中文。
    send_text(&mut desk_ws, json!({"op": "send", "to": phone, "data": "你好，小伴"})).await;
    assert_eq!(expect_op(&mut phone_ws, "deliver").await["data"], "你好，小伴");

    // 发给别的账号的设备、不存在的设备、同类型设备：一律 unknown_peer。
    let other_desktop = env.register(&t_desk, "desktop", "另一台电脑").await;
    for target in [evil_phone.as_str(), "dsk_nonexistent", other_desktop.as_str()] {
        send_text(&mut desk_ws, json!({"op": "send", "to": target, "data": "x"})).await;
        assert_eq!(expect_op(&mut desk_ws, "error").await["code"], "unknown_peer", "{target}");
    }
    // 别人的手机连上来，也收不到任何东西、发不到我的电脑。
    let mut evil_ws = env.ws(&t_evil, &evil_phone).await;
    assert_eq!(expect_op(&mut evil_ws, "peers").await["items"].as_array().unwrap().len(), 0);
    send_text(&mut evil_ws, json!({"op": "send", "to": desktop, "data": "x"})).await;
    assert_eq!(expect_op(&mut evil_ws, "error").await["code"], "unknown_peer");

    // 超过上限的正文被拒绝。
    send_text(&mut phone_ws, json!({"op": "send", "to": desktop, "data": "x".repeat(crate::ws::MAX_DATA + 1)})).await;
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "payload_too_large");
}

#[tokio::test]
async fn new_devices_appear_and_removed_devices_disappear_without_reconnecting() {
    let Some(env) = setup().await else { return };
    let uid = Uuid::new_v4();
    let (t_desk, t_phone) = (env.desktop_token(uid), env.phone_token(uid));
    let desktop = env.register(&t_desk, "desktop", "电脑").await;
    let mut desk_ws = env.ws(&t_desk, &desktop).await;
    expect_op(&mut desk_ws, "ready").await;
    assert_eq!(expect_op(&mut desk_ws, "peers").await["items"].as_array().unwrap().len(), 0);

    // 手机后来才登记：已在线的电脑立刻收到，并且可以直接互发。
    let phone = env.register(&t_phone, "phone", "新手机").await;
    let added = expect_op(&mut desk_ws, "peer.added").await;
    assert_eq!(added["device"]["id"], phone.as_str());
    assert_eq!(added["device"]["name"], "新手机");
    let mut phone_ws = env.ws(&t_phone, &phone).await;
    expect_op(&mut phone_ws, "ready").await;
    expect_op(&mut desk_ws, "presence").await;
    send_text(&mut desk_ws, json!({"op": "send", "to": phone, "data": "hi"})).await;
    assert_eq!(expect_op(&mut phone_ws, "deliver").await["data"], "hi");

    // 撤销手机：电脑收到 peer.removed，之后发不过去；被撤销的连接被断开，也连不上了。
    assert_eq!(env.call("DELETE", &format!("/devices/{phone}"), Some(&t_phone), Value::Null).await.0, 204);
    assert_eq!(expect_op(&mut desk_ws, "peer.removed").await["device"], phone.as_str());
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "device_revoked");
    assert!(next_json(&mut phone_ws).await.is_none());
    send_text(&mut desk_ws, json!({"op": "send", "to": phone, "data": "x"})).await;
    assert_eq!(expect_op(&mut desk_ws, "error").await["code"], "unknown_peer");
    let mut r = format!("{}/ws?device_id={phone}", env.ws_base).into_client_request().unwrap();
    r.headers_mut().insert("Authorization", HeaderValue::from_str(&format!("Bearer {t_phone}")).unwrap());
    assert!(connect_async(r).await.is_err(), "已撤销的设备不能连接");
}

#[tokio::test]
async fn offline_delivery_presence_and_replacement() {
    let Some(env) = setup().await else { return };
    let uid = Uuid::new_v4();
    let (t_desk, t_phone) = (env.desktop_token(uid), env.phone_token(uid));
    let desktop = env.register(&t_desk, "desktop", "电脑").await;
    let phone = env.register(&t_phone, "phone", "手机").await;
    let mut phone_ws = env.ws(&t_phone, &phone).await;
    expect_op(&mut phone_ws, "ready").await;

    // 电脑离线：普通消息直接告知没送达；只有标了 notify 的才暂存（每个设备最多 2 条）。
    send_text(&mut phone_ws, json!({"op": "send", "to": desktop, "data": "指令，不暂存"})).await;
    let undelivered = expect_op(&mut phone_ws, "undelivered").await;
    assert_eq!((undelivered["to"].as_str().unwrap(), undelivered["reason"].as_str().unwrap()), (desktop.as_str(), "offline"));
    for text in ["n1", "n2", "n3"] {
        send_text(&mut phone_ws, json!({"op": "send", "to": desktop, "data": text, "notify": true})).await;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    let mut desk_ws = env.ws(&t_desk, &desktop).await;
    expect_op(&mut desk_ws, "ready").await;
    assert_eq!(expect_op(&mut desk_ws, "deliver").await["data"], "n2");
    assert_eq!(expect_op(&mut desk_ws, "deliver").await["data"], "n3", "只保留最近 2 条");
    assert_eq!(expect_op(&mut phone_ws, "presence").await["online"], true);

    // 同一设备再连一次：旧连接收到 replaced 并被关闭。
    let mut desk_ws2 = env.ws(&t_desk, &desktop).await;
    expect_op(&mut desk_ws2, "ready").await;
    assert_eq!(expect_op(&mut desk_ws, "error").await["code"], "replaced");
    assert!(next_json(&mut desk_ws).await.is_none(), "旧连接应已关闭");

    // 断开：手机收到下线通知。
    desk_ws2.close(None).await.unwrap();
    loop {
        let p = expect_op(&mut phone_ws, "presence").await;
        if p["online"] == false { break; }
    }
}

#[tokio::test]
async fn websocket_token_can_come_from_subprotocol_and_rate_limits_apply() {
    let Some(env) = setup().await else { return };
    let uid = Uuid::new_v4();
    let t_p = env.phone_token(uid);
    let phone = env.register(&t_p, "phone", "手机").await;

    // 没有令牌连不上；子协议里带令牌可以，且服务端回应 xb-link。
    let bare = format!("{}/ws?device_id={phone}", env.ws_base).into_client_request().unwrap();
    assert!(connect_async(bare).await.is_err());
    let mut req = format!("{}/ws?device_id={phone}", env.ws_base).into_client_request().unwrap();
    req.headers_mut().insert("Sec-WebSocket-Protocol", HeaderValue::from_str(&format!("xb-link, xiaoban-token.{t_p}")).unwrap());
    let (mut ws, resp) = connect_async(req).await.expect("子协议认证");
    assert_eq!(resp.headers().get("Sec-WebSocket-Protocol").unwrap(), "xb-link");
    expect_op(&mut ws, "ready").await;

    // 别人的设备编号连不上。
    let other = env.phone_token(Uuid::new_v4());
    let mut r = format!("{}/ws?device_id={phone}", env.ws_base).into_client_request().unwrap();
    r.headers_mut().insert("Authorization", HeaderValue::from_str(&format!("Bearer {other}")).unwrap());
    assert!(connect_async(r).await.is_err());

    // 连发大量消息会被限流（不断开连接）。
    for _ in 0..200 {
        send_text(&mut ws, json!({"op": "ping"})).await;
    }
    let mut limited = false;
    for _ in 0..250 {
        match next_json(&mut ws).await {
            Some(v) if v["op"] == "error" && v["code"] == "rate_limited" => { limited = true; break; }
            Some(_) => {}
            None => break,
        }
    }
    assert!(limited, "应触发 rate_limited");
}

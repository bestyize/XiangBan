//! 集成测试：真实 Postgres（TEST_DATABASE_URL，每个测试独立 schema）、真实 HTTP 与 WebSocket 连接。
use crate::{auth::JwksVerifier, build_router, config::Config, db, hub::Hub, ratelimit::WindowLimiter, state::AppState};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
    MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;
use xiaoban_link_protocol::{envelope::{open, Sender}, keys::DeviceKey, pairing};

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Env {
    base: String,
    ws_base: String,
    http: reqwest::Client,
    enc: EncodingKey,
}

impl Env {
    fn token_aud(&self, uid: Uuid, aud: &str, ttl: i64) -> String {
        #[derive(serde::Serialize)]
        struct C { iss: String, sub: String, aud: String, exp: i64 }
        let c = C { iss: "iss".into(), sub: uid.to_string(), aud: aud.into(), exp: chrono::Utc::now().timestamp() + ttl };
        encode(&Header::new(Algorithm::EdDSA), &c, &self.enc).unwrap()
    }
    fn token(&self, uid: Uuid) -> String { self.token_aud(uid, "xiaoban-android", 600) }

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

async fn setup() -> Option<(Env, sqlx::PgPool)> {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return None;
    };
    let schema = format!("link_test_{}", Uuid::new_v4().simple());
    let admin = db::connect(&url, None).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await.unwrap();
    let pool = db::connect(&url, Some(&schema)).await.unwrap();
    db::migrate(&pool).await.unwrap();
    db::migrate(&pool).await.unwrap();

    let mut config = Config::from_env();
    config.client_ids = vec!["xiaoban-desktop-wallpaper".into(), "xiaoban-android".into()];
    config.pair_code_max_redeems = 3;
    config.notify_queue_cap = 2;
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
    let x = B64.encode(pair.public_key().as_ref());
    let verifier = JwksVerifier::with_static_key("iss".into(), config.client_ids.clone(), DecodingKey::from_ed_components(&x).unwrap());
    // 签发者要和验签一致。
    let state = AppState {
        pool: pool.clone(),
        hub: Arc::new(Hub::new(config.notify_queue_cap, config.notify_ttl)),
        limiter: Arc::new(WindowLimiter::default()),
        verifier: Arc::new(verifier),
        config: Arc::new(config),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, build_router(state)).await.unwrap() });
    let env = Env {
        base: format!("http://{addr}/api/link"),
        ws_base: format!("ws://{addr}/api/link"),
        http: reqwest::Client::new(),
        enc: EncodingKey::from_ed_der(pkcs8.as_ref()),
    };
    Some((env, pool))
}

struct Party {
    key: DeviceKey,
    id: String,
}

async fn register(env: &Env, token: &str, kind: &str, name: &str) -> Party {
    let key = DeviceKey::generate();
    let (s, b) = env.call("POST", "/devices", Some(token), json!({"kind": kind, "name": name, "public_key": B64.encode(key.public_bytes())})).await;
    assert!(s == 201 || s == 200, "{s} {b}");
    Party { key, id: b["device_id"].as_str().unwrap().to_string() }
}

#[tokio::test]
async fn devices_api_validation_and_ownership() {
    let Some((env, _pool)) = setup().await else { return };
    let (alice, bob) = (Uuid::new_v4(), Uuid::new_v4());
    let (ta, tb) = (env.token(alice), env.token(bob));

    assert_eq!(env.call("POST", "/devices", None, json!({})).await.0, 401);
    // 令牌的受众不在配置的客户端里。
    assert_eq!(env.call("GET", "/devices", Some(&env.token_aud(alice, "forum-web", 600)), Value::Null).await.0, 401);
    assert_eq!(env.call("GET", "/devices", Some(&env.token_aud(alice, "xiaoban-android", -600)), Value::Null).await.0, 401, "过期");

    let key = DeviceKey::generate();
    let good = json!({"kind": "phone", "name": "我的手机", "public_key": B64.encode(key.public_bytes())});
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "tablet", "name": "x", "public_key": B64.encode(key.public_bytes())})).await.0, 400);
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "  ", "public_key": B64.encode(key.public_bytes())})).await.0, 400);
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "x", "public_key": "AAAA"})).await.0, 400);
    let mut bad = key.public_bytes().to_vec();
    bad[0] = 2;
    assert_eq!(env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "x", "public_key": B64.encode(bad)})).await.0, 400, "必须是未压缩点");

    let (s, first) = env.call("POST", "/devices", Some(&ta), good.clone()).await;
    assert_eq!(s, 201);
    let (s, again) = env.call("POST", "/devices", Some(&ta), json!({"kind": "phone", "name": "改个名", "public_key": B64.encode(key.public_bytes())})).await;
    assert_eq!(s, 200, "同一公钥幂等");
    assert_eq!(first["device_id"], again["device_id"]);
    let (_, list) = env.call("GET", "/devices", Some(&ta), Value::Null).await;
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["name"], "改个名");
    assert_eq!(list["items"][0]["online"], false);

    // 别的账号看不到、删不了。
    let (_, other) = env.call("GET", "/devices", Some(&tb), Value::Null).await;
    assert!(other["items"].as_array().unwrap().is_empty());
    let id = first["device_id"].as_str().unwrap();
    assert_eq!(env.call("DELETE", &format!("/devices/{id}"), Some(&tb), Value::Null).await.0, 404);
    assert_eq!(env.call("DELETE", &format!("/devices/{id}"), Some(&ta), Value::Null).await.0, 204);
    assert_eq!(env.call("DELETE", &format!("/devices/{id}"), Some(&ta), Value::Null).await.0, 404);
}

#[tokio::test]
async fn pairing_flow_security_and_end_to_end_encrypted_relay() {
    let Some((env, _pool)) = setup().await else { return };
    let (alice, mallory) = (Uuid::new_v4(), Uuid::new_v4());
    let (t_desk, t_phone, t_evil) = (env.token_aud(alice, "xiaoban-desktop-wallpaper", 600), env.token(alice), env.token(mallory));
    let desktop = register(&env, &t_desk, "desktop", "办公室电脑").await;
    let phone = register(&env, &t_phone, "phone", "我的手机").await;
    let evil_phone = register(&env, &t_evil, "phone", "别人的手机").await;

    // 桌面申请配对码，手机端不能申请，别人的设备不能用。
    assert_eq!(env.call("POST", "/pair-codes", Some(&t_desk), json!({"device_id": phone.id})).await.0, 400, "手机不能申请配对码");
    assert_eq!(env.call("POST", "/pair-codes", Some(&t_evil), json!({"device_id": desktop.id})).await.0, 404);
    let (s, code) = env.call("POST", "/pair-codes", Some(&t_desk), json!({"device_id": desktop.id})).await;
    assert_eq!(s, 200);
    let code_text = code["code"].as_str().unwrap().to_string();
    assert_eq!(code_text.len(), 8);

    // 二维码里的一次性密钥只有桌面和扫码的手机知道。
    let secret = [7u8; 32];
    let proof = pairing::make_proof(&secret, &phone.id, &desktop.id, &phone.key.public_bytes());
    let proof_b64 = B64.encode(proof);

    // 别的账号拿到码也兑换不了（和过期同一个说法）；错误的码一样。
    let (s, b) = env.call("POST", "/pair", Some(&t_evil), json!({"code": code_text, "device_id": evil_phone.id, "proof": proof_b64})).await;
    assert_eq!((s, b["code"].as_str()), (410, Some("code_expired")));
    let (s, b) = env.call("POST", "/pair", Some(&t_phone), json!({"code": "ZZZZZZZZ", "device_id": phone.id, "proof": proof_b64})).await;
    assert_eq!((s, b["code"].as_str()), (410, Some("code_expired")));
    assert_eq!(env.call("POST", "/pair", Some(&t_phone), json!({"code": code_text, "device_id": phone.id, "proof": "短"})).await.0, 400);

    // 桌面在线，手机兑换：桌面收到 pair.request，里面带着手机公钥和证明。
    let mut desk_ws = env.ws(&t_desk, &desktop.id).await;
    expect_op(&mut desk_ws, "ready").await;
    let (s, redeemed) = env.call("POST", "/pair", Some(&t_phone), json!({"code": code_text.to_lowercase(), "device_id": phone.id, "proof": proof_b64})).await;
    assert_eq!(s, 202, "{redeemed}");
    assert_eq!(redeemed["desktop"]["public_key"], B64.encode(desktop.key.public_bytes()), "手机据此核对二维码里的桌面公钥");
    let req = expect_op(&mut desk_ws, "pair.request").await;
    assert_eq!(req["phone"]["id"], phone.id.as_str());
    assert_eq!(req["phone"]["public_key"], B64.encode(phone.key.public_bytes()));
    assert_eq!(req["proof"], proof_b64.as_str());
    pairing::verify_proof(&secret, &phone.id, &desktop.id, &phone.key.public_bytes(), &B64.decode(req["proof"].as_str().unwrap()).unwrap()).unwrap();
    let pairing_id = req["pairing_id"].as_str().unwrap().to_string();

    // 在确认之前，手机不能发消息；非桌面端（同账号的手机）也不该确认成功后还能冒充——这里只验证确认后才通。
    let mut phone_ws = env.ws(&t_phone, &phone.id).await;
    expect_op(&mut phone_ws, "ready").await;
    send_text(&mut phone_ws, json!({"op": "send", "env": {"v":1,"from":phone.id,"to":desktop.id,"ep":"AAAA","seq":0,"ct":"AAAA"}})).await;
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "not_paired");

    // 别的账号不能确认这条请求。
    assert_eq!(env.call("POST", &format!("/pairings/{pairing_id}/confirm"), Some(&t_evil), Value::Null).await.0, 404);
    let (s, conf) = env.call("POST", &format!("/pairings/{pairing_id}/confirm"), Some(&t_desk), Value::Null).await;
    assert_eq!(s, 200, "{conf}");
    assert_eq!(env.call("POST", &format!("/pairings/{pairing_id}/confirm"), Some(&t_desk), Value::Null).await.0, 409, "不能重复确认");
    let confirmed = expect_op(&mut phone_ws, "pair.confirmed").await;
    assert_eq!(confirmed["desktop"]["id"], desktop.id.as_str());
    // 码已用掉。
    let (s, b) = env.call("POST", "/pair", Some(&t_phone), json!({"code": code_text, "device_id": phone.id, "proof": proof_b64})).await;
    assert_eq!((s, b["code"].as_str()), (410, Some("code_used")));

    // 两端各自派生配对密钥，经中转端到端通信；中转转发的信封与发出的完全一致。
    let prk_d = pairing::derive_pairing_key(&desktop.key, &phone.key.public_bytes(), &secret).unwrap();
    let prk_p = pairing::derive_pairing_key(&phone.key, &desktop.key.public_bytes(), &secret).unwrap();
    assert_eq!(prk_d, prk_p);
    let mut p_tx = Sender::new(prk_p, &phone.id, &desktop.id, 1);
    let env1 = p_tx.seal("你好，电脑上的小伴".as_bytes()).unwrap();
    send_text(&mut phone_ws, json!({"op": "send", "env": env1})).await;
    let got = expect_op(&mut desk_ws, "deliver").await;
    assert_eq!(got["env"], serde_json::to_value(&env1).unwrap());
    let env_back: xiaoban_link_protocol::envelope::Envelope = serde_json::from_value(got["env"].clone()).unwrap();
    assert_eq!(open(&prk_d, &phone.id, &desktop.id, &env_back).unwrap(), "你好，电脑上的小伴".as_bytes());
    let mut d_tx = Sender::new(prk_d, &desktop.id, &phone.id, 1);
    send_text(&mut desk_ws, json!({"op": "send", "env": d_tx.seal(b"pong").unwrap()})).await;
    let back = expect_op(&mut phone_ws, "deliver").await;
    let env_back: xiaoban_link_protocol::envelope::Envelope = serde_json::from_value(back["env"].clone()).unwrap();
    assert_eq!(open(&prk_p, &desktop.id, &phone.id, &env_back).unwrap(), b"pong");

    // 伪造发送方、发给没配对的设备、超大信封：都被拒绝。
    let forged = Sender::new(prk_p, &desktop.id, &phone.id, 2).seal(b"x").unwrap();
    send_text(&mut phone_ws, json!({"op": "send", "env": forged})).await;
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "bad_from");
    let stray = Sender::new(prk_p, &phone.id, &evil_phone.id, 2).seal(b"x").unwrap();
    send_text(&mut phone_ws, json!({"op": "send", "env": stray})).await;
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "not_paired");
    let mut huge = serde_json::to_value(&env1).unwrap();
    huge["ct"] = json!("A".repeat(400_000));
    send_text(&mut phone_ws, json!({"op": "send", "env": huge})).await;
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "payload_too_large");
    send_text(&mut phone_ws, json!({"op": "nonsense"})).await;
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "bad_frame");

    // 配对列表：双方各看到对方。
    let (_, l) = env.call("GET", &format!("/pairings?device_id={}", phone.id), Some(&t_phone), Value::Null).await;
    assert_eq!(l["items"][0]["peer"]["id"], desktop.id.as_str());
    assert_eq!(l["items"][0]["online"], true);
    // 别人查不到。
    assert_eq!(env.call("GET", &format!("/pairings?device_id={}", phone.id), Some(&t_evil), Value::Null).await.0, 404);

    // 解除配对：两端都收到 unpaired，之后发不出去。
    assert_eq!(env.call("DELETE", &format!("/pairings/{pairing_id}"), Some(&t_phone), Value::Null).await.0, 204);
    assert_eq!(expect_op(&mut desk_ws, "unpaired").await["device"], phone.id.as_str());
    assert_eq!(expect_op(&mut phone_ws, "unpaired").await["device"], desktop.id.as_str());
    send_text(&mut phone_ws, json!({"op": "send", "env": Sender::new(prk_p, &phone.id, &desktop.id, 3).seal(b"x").unwrap()})).await;
    assert_eq!(expect_op(&mut phone_ws, "error").await["code"], "not_paired");
}

#[tokio::test]
async fn code_expiry_attempt_limits_reject_and_offline_desktop() {
    let Some((env, pool)) = setup().await else { return };
    let uid = Uuid::new_v4();
    let t = env.token(uid);
    let desktop = register(&env, &t, "desktop", "电脑").await;
    let phone = register(&env, &t, "phone", "手机").await;
    let proof = B64.encode([1u8; 32]);

    // 过期的码。
    let (_, c) = env.call("POST", "/pair-codes", Some(&t), json!({"device_id": desktop.id})).await;
    sqlx::query("UPDATE pair_codes SET expires_at = now() - interval '1 minute'").execute(&pool).await.unwrap();
    let (s, b) = env.call("POST", "/pair", Some(&t), json!({"code": c["code"], "device_id": phone.id, "proof": proof})).await;
    assert_eq!((s, b["code"].as_str()), (410, Some("code_expired")));

    // 新码顶掉旧码；兑换次数用完（被拒绝的也算）后作废，即使是正确的手机。
    let (_, c1) = env.call("POST", "/pair-codes", Some(&t), json!({"device_id": desktop.id})).await;
    let (_, c2) = env.call("POST", "/pair-codes", Some(&t), json!({"device_id": desktop.id})).await;
    assert_ne!(c1["code"], c2["code"]);
    let (s, _) = env.call("POST", "/pair", Some(&t), json!({"code": c1["code"], "device_id": phone.id, "proof": proof})).await;
    assert_eq!(s, 410, "旧码已被顶掉");
    for i in 0..3 {
        let (s, b) = env.call("POST", "/pair", Some(&t), json!({"code": c2["code"], "device_id": phone.id, "proof": proof})).await;
        assert_eq!(s, 202, "第 {i} 次 {b}");
        let pid = b["pairing_id"].as_str().unwrap().to_string();
        // 桌面不在线也能拒绝；手机在线时收到 pair.rejected。
        assert_eq!(env.call("POST", &format!("/pairings/{pid}/reject"), Some(&t), Value::Null).await.0, 204);
        assert_eq!(env.call("POST", &format!("/pairings/{pid}/reject"), Some(&t), Value::Null).await.0, 409);
    }
    let (s, b) = env.call("POST", "/pair", Some(&t), json!({"code": c2["code"], "device_id": phone.id, "proof": proof})).await;
    assert_eq!((s, b["code"].as_str()), (429, Some("too_many_attempts")));

    // 待确认请求在桌面离线时留在中转，桌面上线后补发，也能通过列表取到（带证明）。
    let (_, c3) = env.call("POST", "/pair-codes", Some(&t), json!({"device_id": desktop.id})).await;
    let (s, r) = env.call("POST", "/pair", Some(&t), json!({"code": c3["code"], "device_id": phone.id, "proof": proof})).await;
    assert_eq!(s, 202);
    let (_, pending) = env.call("GET", &format!("/pairings?device_id={}&status=pending", desktop.id), Some(&t), Value::Null).await;
    assert_eq!(pending["items"][0]["proof"], proof.as_str());
    let mut ws = env.ws(&t, &desktop.id).await;
    let req = expect_op(&mut ws, "pair.request").await;
    assert_eq!(req["pairing_id"], r["pairing_id"]);

    // 请求过期后不能再确认。
    sqlx::query("UPDATE pairings SET created_at = now() - interval '1 hour'").execute(&pool).await.unwrap();
    let pid = r["pairing_id"].as_str().unwrap();
    let (s, b) = env.call("POST", &format!("/pairings/{pid}/confirm"), Some(&t), Value::Null).await;
    assert_eq!((s, b["code"].as_str()), (410, Some("pairing_expired")));
}

async fn paired_pair(env: &Env, uid: Uuid) -> (Party, Party, String, String) {
    let (t_d, t_p) = (env.token_aud(uid, "xiaoban-desktop-wallpaper", 600), env.token(uid));
    let desktop = register(env, &t_d, "desktop", "电脑").await;
    let phone = register(env, &t_p, "phone", "手机").await;
    let (_, c) = env.call("POST", "/pair-codes", Some(&t_d), json!({"device_id": desktop.id})).await;
    let proof = B64.encode([2u8; 32]);
    let (_, r) = env.call("POST", "/pair", Some(&t_p), json!({"code": c["code"], "device_id": phone.id, "proof": proof})).await;
    let pid = r["pairing_id"].as_str().unwrap();
    assert_eq!(env.call("POST", &format!("/pairings/{pid}/confirm"), Some(&t_d), Value::Null).await.0, 200);
    (desktop, phone, t_d, t_p)
}

#[tokio::test]
async fn offline_delivery_presence_replacement_and_revocation() {
    let Some((env, _pool)) = setup().await else { return };
    let uid = Uuid::new_v4();
    let (desktop, phone, t_d, t_p) = paired_pair(&env, uid).await;
    let prk = pairing::derive_pairing_key(&phone.key, &desktop.key.public_bytes(), &[2u8; 32]).unwrap();
    let mut tx = Sender::new(prk, &phone.id, &desktop.id, 1);

    let mut phone_ws = env.ws(&t_p, &phone.id).await;
    let ready = expect_op(&mut phone_ws, "ready").await;
    assert_eq!(ready["device_id"], phone.id.as_str());
    assert_eq!(expect_op(&mut phone_ws, "presence").await, json!({"op":"presence","device":desktop.id,"online":false}));

    // 电脑离线：普通消息回 undelivered，通知消息暂存（上限 2，挤掉最旧的），电脑上线后补发。
    let cmd = tx.seal(b"do it").unwrap();
    send_text(&mut phone_ws, json!({"op": "send", "env": cmd})).await;
    let und = expect_op(&mut phone_ws, "undelivered").await;
    assert_eq!((und["to"].as_str(), und["reason"].as_str()), (Some(desktop.id.as_str()), Some("offline")));
    let mut sent = Vec::new();
    for i in 0..3 {
        let e = tx.seal(format!("n{i}").as_bytes()).unwrap();
        send_text(&mut phone_ws, json!({"op": "send", "env": e, "notify": true})).await;
        sent.push(e);
    }
    send_text(&mut phone_ws, json!({"op": "ping"})).await;
    expect_op(&mut phone_ws, "pong").await;

    let mut desk_ws = env.ws(&t_d, &desktop.id).await;
    expect_op(&mut desk_ws, "ready").await;
    let first = expect_op(&mut desk_ws, "deliver").await;
    let second = expect_op(&mut desk_ws, "deliver").await;
    assert_eq!(first["env"], serde_json::to_value(&sent[1]).unwrap());
    assert_eq!(second["env"], serde_json::to_value(&sent[2]).unwrap(), "只保留最近 2 条");
    // 电脑上线时手机收到 presence。
    assert_eq!(expect_op(&mut phone_ws, "presence").await["online"], true);

    // 同一设备再连一次：旧连接收到 replaced 并被关闭。
    let mut desk_ws2 = env.ws(&t_d, &desktop.id).await;
    expect_op(&mut desk_ws2, "ready").await;
    assert_eq!(expect_op(&mut desk_ws, "error").await["code"], "replaced");
    assert!(next_json(&mut desk_ws).await.is_none(), "旧连接应已关闭");
    assert_eq!(expect_op(&mut phone_ws, "presence").await["online"], true, "新连接上线");

    // 断开：手机收到下线通知。
    desk_ws2.close(None).await.unwrap();
    assert_eq!(expect_op(&mut phone_ws, "presence").await["online"], false);

    // 撤销桌面设备：手机收到 unpaired，被撤销设备再也连不上。
    let mut desk_ws3 = env.ws(&t_d, &desktop.id).await;
    expect_op(&mut desk_ws3, "ready").await;
    assert_eq!(env.call("DELETE", &format!("/devices/{}", desktop.id), Some(&t_d), Value::Null).await.0, 204);
    assert_eq!(expect_op(&mut desk_ws3, "error").await["code"], "device_revoked");
    assert!(next_json(&mut desk_ws3).await.is_none());
    assert_eq!(expect_op(&mut phone_ws, "unpaired").await["device"], desktop.id.as_str());
    let err = connect_async({
        let mut r = format!("{}/ws?device_id={}", env.ws_base, desktop.id).into_client_request().unwrap();
        r.headers_mut().insert("Authorization", HeaderValue::from_str(&format!("Bearer {t_d}")).unwrap());
        r
    })
    .await;
    assert!(err.is_err(), "已撤销的设备不能连接");
}

#[tokio::test]
async fn websocket_token_can_come_from_subprotocol_and_rate_limits_apply() {
    let Some((env, _pool)) = setup().await else { return };
    let uid = Uuid::new_v4();
    let (_desktop, phone, _t_d, t_p) = paired_pair(&env, uid).await;

    // 没有令牌连不上；子协议里带令牌可以，且服务端回应 xb-link。
    let bare = format!("{}/ws?device_id={}", env.ws_base, phone.id).into_client_request().unwrap();
    assert!(connect_async(bare).await.is_err());
    let mut req = format!("{}/ws?device_id={}", env.ws_base, phone.id).into_client_request().unwrap();
    req.headers_mut().insert("Sec-WebSocket-Protocol", HeaderValue::from_str(&format!("xb-link, xiaoban-token.{t_p}")).unwrap());
    let (mut ws, resp) = connect_async(req).await.expect("子协议认证");
    assert_eq!(resp.headers().get("Sec-WebSocket-Protocol").unwrap(), "xb-link");
    expect_op(&mut ws, "ready").await;

    // 别人的设备编号连不上。
    let other = env.token(Uuid::new_v4());
    let mut r = format!("{}/ws?device_id={}", env.ws_base, phone.id).into_client_request().unwrap();
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

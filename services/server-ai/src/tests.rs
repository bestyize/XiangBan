//! 集成测试：需要 TEST_DATABASE_URL 指向可用的 PostgreSQL，每个测试使用独立 schema，结束后删除；未设置时直接跳过。
//! New API 用一个本地假网关代替，它会校验请求格式，所以客户端写错参数会直接让测试失败。

use crate::{
    admin_auth, binding, build_router, config::AppConfig, db, jwks::JwksVerifier, keys::Keys, newapi::NewApi, password,
    products, quota, state::AppState, sync,
};
use axum::{
    body::{to_bytes, Body},
    extract::{Path, Query, State},
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};
use tower::ServiceExt;
use uuid::Uuid;

const ISSUER: &str = "https://account.test";
const CLIENT: &str = "xiangban-wallpaper-web";
const ADMIN_PASSWORD: &str = "admin-pass-123";
const ADMIN_TOKEN: &str = "test-admin-token";
const TRIAL: i64 = 250_000;
const REAL_STANDARD: &str = "deepseek-v4.1-flash";
const REAL_PREMIUM: &str = "claude-sonnet-5.5";

// ---- 假的 New API ----

#[derive(Clone, Copy, PartialEq)]
enum ChatMode {
    Json,
    Sse,
    Fail,
}

struct MockUser {
    username: String,
    quota: i64,
    key: String,
}

struct MockInner {
    users: BTreeMap<i64, MockUser>,
    next_id: i64,
    signup_bonus: i64,
    fail_manage: bool,
    chat_mode: ChatMode,
    chat_cost: i64,
    chat_requests: Vec<(String, Value)>,
}

struct Mock(Mutex<MockInner>);

impl Mock {
    fn quota_of(&self, id: i64) -> i64 {
        self.0.lock().unwrap().users[&id].quota
    }
    fn only_user(&self) -> (i64, i64) {
        let inner = self.0.lock().unwrap();
        assert_eq!(inner.users.len(), 1, "expected exactly one New API user");
        let (id, user) = inner.users.iter().next().unwrap();
        (*id, user.quota)
    }
    fn user_count(&self) -> usize {
        self.0.lock().unwrap().users.len()
    }
    fn consume(&self, id: i64, amount: i64) {
        self.0.lock().unwrap().users.get_mut(&id).unwrap().quota -= amount;
    }
    fn credit(&self, id: i64, amount: i64) {
        self.0.lock().unwrap().users.get_mut(&id).unwrap().quota += amount;
    }
}

fn authorized(headers: &HeaderMap) -> bool {
    headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) == Some(&format!("Bearer {ADMIN_TOKEN}"))
}

fn ok(data: Value) -> Response {
    Json(json!({ "success": true, "message": "", "data": data })).into_response()
}

fn refused(message: &str) -> Response {
    Json(json!({ "success": false, "message": message })).into_response()
}

fn user_json(id: i64, user: &MockUser) -> Value {
    json!({ "id": id, "username": user.username, "quota": user.quota, "used_quota": 0, "status": 1 })
}

async fn mock_create(State(m): State<Arc<Mock>>, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    if !authorized(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let username = body["username"].as_str().unwrap_or_default().to_string();
    if username.is_empty() || body["password"].as_str().unwrap_or_default().len() < 8 || body["role"] != json!(1) {
        return refused("Invalid parameters");
    }
    let mut inner = m.0.lock().unwrap();
    if inner.users.values().any(|u| u.username == username) {
        return refused("username already exists");
    }
    inner.next_id += 1;
    let id = inner.next_id;
    let quota = inner.signup_bonus;
    inner.users.insert(id, MockUser { username, quota, key: format!("mockkey{id:042}") });
    ok(Value::Null)
}

async fn mock_search(State(m): State<Arc<Mock>>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    if !authorized(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let keyword = q.get("keyword").cloned().unwrap_or_default();
    let inner = m.0.lock().unwrap();
    let items: Vec<Value> = inner.users.iter().filter(|(_, u)| u.username.contains(&keyword)).map(|(id, u)| user_json(*id, u)).collect();
    ok(json!({ "items": items, "total": items.len() }))
}

async fn mock_get_user(State(m): State<Arc<Mock>>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    if !authorized(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match m.0.lock().unwrap().users.get(&id) {
        Some(user) => ok(user_json(id, user)),
        None => refused("record not found"),
    }
}

async fn mock_manage(State(m): State<Arc<Mock>>, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    if !authorized(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut inner = m.0.lock().unwrap();
    if inner.fail_manage {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let (id, value) = (body["id"].as_i64().unwrap_or(0), body["value"].as_i64().unwrap_or(0));
    if body["action"] != json!("add_quota") || value <= 0 {
        return refused("Invalid parameters");
    }
    let Some(user) = inner.users.get_mut(&id) else { return refused("User does not exist") };
    match body["mode"].as_str() {
        Some("add") => user.quota += value,
        Some("subtract") if user.quota >= value => user.quota -= value,
        _ => return refused("Invalid parameters"),
    }
    ok(Value::Null)
}

async fn mock_token(State(m): State<Arc<Mock>>, headers: HeaderMap, Path(id): Path<i64>, Json(body): Json<Value>) -> Response {
    if !authorized(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if body["name"] != json!("server-ai") {
        return refused("Invalid parameters");
    }
    match m.0.lock().unwrap().users.get(&id) {
        Some(user) => ok(json!({ "id": id, "name": "server-ai", "key": user.key, "created": true })),
        None => refused("record not found"),
    }
}

async fn mock_pricing() -> Response {
    Json(json!({ "success": true, "data": [
        { "model_name": REAL_STANDARD, "model_ratio": 0.07, "completion_ratio": 1.0, "model_price": 0 },
        { "model_name": REAL_PREMIUM, "model_ratio": 1.5, "completion_ratio": 5.0, "model_price": 0 },
        { "model_name": "unpriced-model", "model_ratio": 37.5, "completion_ratio": 1.0, "model_price": 0 },
    ]}))
    .into_response()
}

async fn mock_chat(State(m): State<Arc<Mock>>, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    let auth = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
    let mut inner = m.0.lock().unwrap();
    inner.chat_requests.push((auth.clone(), body.clone()));
    let Some(key) = auth.strip_prefix("Bearer sk-") else { return StatusCode::UNAUTHORIZED.into_response() };
    let Some(id) = inner.users.iter().find(|(_, u)| u.key == key).map(|(id, _)| *id) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let cost = inner.chat_cost;
    inner.users.get_mut(&id).unwrap().quota -= cost;
    let model = body["model"].as_str().unwrap_or_default().to_string();
    match inner.chat_mode {
        ChatMode::Json => Json(json!({ "id": "chatcmpl-1", "object": "chat.completion", "model": model,
            "choices": [{ "index": 0, "message": { "role": "assistant", "content": "你好" }, "finish_reason": "stop" }] }))
        .into_response(),
        ChatMode::Sse => {
            let chunk = |text: &str| format!("data: {}\n\n", json!({ "id": "1", "model": model, "choices": [{ "delta": { "content": text } }] }));
            let payload = format!("{}{}data: [DONE]\n\n", chunk("你"), chunk("好"));
            Response::builder().header(header::CONTENT_TYPE, "text/event-stream").body(Body::from(payload)).unwrap()
        }
        ChatMode::Fail => (StatusCode::INTERNAL_SERVER_ERROR, format!("model {model} failed on channel 7")).into_response(),
    }
}

async fn start_mock() -> (Arc<Mock>, String) {
    let mock = Arc::new(Mock(Mutex::new(MockInner {
        users: BTreeMap::new(),
        next_id: 100,
        signup_bonus: 0,
        fail_manage: false,
        chat_mode: ChatMode::Json,
        chat_cost: 0,
        chat_requests: Vec::new(),
    })));
    let app = Router::new()
        .route("/api/user/", post(mock_create))
        .route("/api/user/search", get(mock_search))
        .route("/api/user/manage", post(mock_manage))
        .route("/api/user/{id}", get(mock_get_user))
        .route("/api/user/{id}/token", post(mock_token))
        .route("/api/pricing", get(mock_pricing))
        .route("/v1/chat/completions", post(mock_chat))
        .with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (mock, base)
}

// ---- 测试夹具 ----

struct Harness {
    app: Router,
    state: AppState,
    mock: Arc<Mock>,
    signer: EncodingKey,
    admin: PgPool,
    schema: String,
}

impl Harness {
    async fn new() -> Option<Self> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let schema = format!("ai_test_{}", Uuid::new_v4().simple());
        let admin = db::connect(&url, None).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await.unwrap();
        let pool = db::connect(&url, Some(&schema)).await.unwrap();
        db::migrate(&pool).await.unwrap();

        let (mock, base) = start_mock().await;
        let config = AppConfig::from_lookup(|key| match key {
            "AI_NEWAPI_BASE_URL" => Some(base.clone()),
            "AI_NEWAPI_ADMIN_TOKEN" => Some(ADMIN_TOKEN.to_string()),
            "AI_ADMIN_USER" => Some("Root".to_string()),
            "AI_ADMIN_PASSWORD" => Some(ADMIN_PASSWORD.to_string()),
            _ => None,
        });
        admin_auth::seed(&pool, &config).await.unwrap();

        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let x = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
        let verifier = JwksVerifier::with_static_key(ISSUER.into(), DecodingKey::from_ed_components(&x).unwrap());

        let state = AppState {
            pool,
            newapi: Arc::new(NewApi::new(base, Some(ADMIN_TOKEN.to_string()))),
            config: Arc::new(config),
            keys: Arc::new(Keys::ephemeral()),
            verifier: Arc::new(verifier),
            dummy_password_hash: Arc::new(password::hash("dummy".into()).await.unwrap()),
        };
        Some(Self { app: build_router(state.clone()), state, mock, signer: EncodingKey::from_ed_der(pkcs8.as_ref()), admin, schema })
    }

    async fn cleanup(self) {
        self.state.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema)).execute(&self.admin).await.unwrap();
    }

    fn token(&self, uid: Uuid, client: &str) -> String {
        let claims = json!({ "iss": ISSUER, "sub": uid.to_string(), "aud": client, "exp": Utc::now().timestamp() + 600 });
        encode(&Header::new(Algorithm::EdDSA), &claims, &self.signer).unwrap()
    }

    async fn raw(&self, method: Method, path: &str, token: Option<&str>, body: Value) -> (StatusCode, String) {
        let mut req = Request::builder().method(method).uri(path).header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let response = self.app.clone().oneshot(req.body(Body::from(body.to_string())).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    async fn call(&self, method: Method, path: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
        let (status, text) = self.raw(method, path, token, body).await;
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    async fn admin_token(&self) -> String {
        let (status, body) = self
            .call(Method::POST, "/api/admin/login", None, json!({ "username": "root", "password": ADMIN_PASSWORD }))
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body["token"].as_str().unwrap().to_string()
    }

    /// 通过管理接口建好小伴壁纸产品和两个档位，顺带覆盖这些接口。
    async fn seed_wallpaper(&self) {
        let admin = self.admin_token().await;
        let (status, body) = self
            .call(
                Method::PUT,
                "/api/admin/products/wallpaper",
                Some(&admin),
                json!({ "name": "小伴壁纸", "client_ids": [CLIENT, "xiaoban-mobile-wallpaper"], "trial_quota": TRIAL, "trial_days": 7, "enabled": true }),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        for (tier, name, model, paid, default, sort) in
            [("standard", "标准", REAL_STANDARD, false, true, 1), ("premium", "高级", REAL_PREMIUM, true, false, 2)]
        {
            let (status, body) = self
                .call(
                    Method::PUT,
                    &format!("/api/admin/products/wallpaper/tiers/{tier}"),
                    Some(&admin),
                    json!({ "display_name": name, "model": model, "requires_paid": paid, "is_default": default, "sort_order": sort, "enabled": true }),
                )
                .await;
            assert_eq!(status, StatusCode::OK, "{body}");
        }
    }

    async fn chat(&self, uid: Uuid, tier: &str) -> (StatusCode, Value) {
        let token = self.token(uid, CLIENT);
        self.call(Method::POST, "/v1/chat/completions", Some(&token), json!({ "model": tier, "messages": [{ "role": "user", "content": "hi" }] })).await
    }

    async fn only_binding(&self) -> i64 {
        let (id,): (i64,) = sqlx::query_as("SELECT id FROM bindings").fetch_one(&self.state.pool).await.unwrap();
        id
    }

    async fn balance(&self, binding_id: i64) -> i64 {
        let mut conn = self.state.pool.acquire().await.unwrap();
        quota::live_total(&mut conn, binding_id).await.unwrap()
    }

    async fn bucket(&self, kind: &str) -> (i64, i64, Option<String>) {
        sqlx::query_as("SELECT id, remaining, closed_reason FROM quota_buckets WHERE kind = $1 ORDER BY id LIMIT 1")
            .bind(kind)
            .fetch_one(&self.state.pool)
            .await
            .unwrap()
    }

    async fn grant(&self, binding_id: i64, quota: i64) -> (StatusCode, Value) {
        let admin = self.admin_token().await;
        self.call(Method::POST, &format!("/api/admin/bindings/{binding_id}/grant"), Some(&admin), json!({ "quota": quota, "note": "测试" })).await
    }
}

macro_rules! harness {
    () => {
        match Harness::new().await {
            Some(h) => h,
            None => return,
        }
    };
}

// ---- 首次使用与开通 ----

#[tokio::test]
async fn first_request_provisions_user_token_and_trial_then_reuses_them() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();

    let (status, body) = h.chat(uid, "standard").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["model"], "standard", "response must carry the tier, not the real model");

    let (user_id, wallet) = h.mock.only_user();
    assert_eq!(wallet, TRIAL, "trial quota is pushed into the New API wallet");
    let binding_id = h.only_binding().await;
    assert_eq!(h.mock.0.lock().unwrap().users[&user_id].username, binding::newapi_username(binding_id));
    assert_eq!(h.balance(binding_id).await, TRIAL);
    let (_, remaining, _) = h.bucket("trial").await;
    assert_eq!(remaining, TRIAL);

    let (auth, forwarded) = h.mock.0.lock().unwrap().chat_requests[0].clone();
    assert_eq!(forwarded["model"], REAL_STANDARD);
    assert!(auth.starts_with("Bearer sk-mockkey"), "{auth}");

    h.chat(uid, "standard").await;
    assert_eq!(h.mock.user_count(), 1, "a second request must reuse the binding");
    let (grants,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota_buckets WHERE kind = 'trial'").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(grants, 1, "the trial is granted once per account and product");

    h.cleanup().await;
}

#[tokio::test]
async fn simultaneous_first_requests_create_one_user() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();

    let (a, b, c) = tokio::join!(h.chat(uid, "standard"), h.chat(uid, "standard"), h.chat(uid, "standard"));
    for (status, body) in [a, b, c] {
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    assert_eq!(h.mock.user_count(), 1);
    let (trials,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota_buckets WHERE kind = 'trial'").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(trials, 1);
    assert_eq!(h.mock.only_user().1, TRIAL);

    h.cleanup().await;
}

#[tokio::test]
async fn products_are_isolated_per_account_and_product() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    let (status, _) = h
        .call(Method::PUT, "/api/admin/products/forum", Some(&admin), json!({ "name": "论坛", "client_ids": ["forum-web"], "trial_quota": 100000, "trial_days": 3, "enabled": true }))
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = h
        .call(Method::PUT, "/api/admin/products/forum/tiers/standard", Some(&admin), json!({ "display_name": "标准", "model": REAL_STANDARD, "requires_paid": false, "is_default": true, "sort_order": 1, "enabled": true }))
        .await;
    assert_eq!(status, StatusCode::OK);

    let uid = Uuid::new_v4();
    h.chat(uid, "standard").await;
    let token = h.token(uid, "forum-web");
    let (status, body) = h.call(Method::POST, "/v1/chat/completions", Some(&token), json!({ "messages": [] })).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    assert_eq!(h.mock.user_count(), 2, "the same account gets a separate New API user per product");
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT b.product_id, q.total FROM bindings b JOIN quota_buckets q ON q.binding_id = b.id AND q.kind = 'trial' ORDER BY b.product_id",
    )
    .fetch_all(&h.state.pool)
    .await
    .unwrap();
    assert_eq!(rows, vec![("forum".to_string(), 100_000), ("wallpaper".to_string(), TRIAL)]);

    h.cleanup().await;
}

// ---- 对账 ----

#[tokio::test]
async fn consumption_is_settled_oldest_expiry_first_and_balance_matches_the_wallet() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();

    let (status, _) = h.grant(binding_id, 1_000_000).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(h.mock.quota_of(user_id), TRIAL + 1_000_000, "a grant reaches the wallet immediately");

    h.mock.consume(user_id, 300_000);
    let outcome = sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(outcome, sync::SyncOutcome::Synced { consumed: 300_000, pushed: 0 });

    let (_, trial_left, trial_closed) = h.bucket("trial").await;
    let (_, topup_left, _) = h.bucket("topup").await;
    assert_eq!((trial_left, trial_closed.as_deref()), (0, Some("spent")), "the 7-day trial expires first, so it is spent first");
    assert_eq!(topup_left, 950_000);
    assert_eq!(h.balance(binding_id).await, h.mock.quota_of(user_id), "buckets and wallet agree after settlement");

    let (consumed,): (i64,) =
        sqlx::query_as("SELECT -SUM(delta)::BIGINT FROM quota_ledger WHERE kind = 'consume'").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(consumed, 300_000);

    h.cleanup().await;
}

#[tokio::test]
async fn repeated_sync_without_activity_changes_nothing() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;

    let (before,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota_ledger").fetch_one(&h.state.pool).await.unwrap();
    for _ in 0..3 {
        assert_eq!(sync::sync_binding(&h.state, binding_id).await.unwrap(), sync::SyncOutcome::Synced { consumed: 0, pushed: 0 });
    }
    let (after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota_ledger").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(before, after, "idle syncs must not write ledger rows");

    h.cleanup().await;
}

#[tokio::test]
async fn expired_bucket_is_closed_and_removed_from_the_wallet() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();
    h.grant(binding_id, 400_000).await;

    sqlx::query("UPDATE quota_buckets SET expires_at = now() - interval '1 minute' WHERE kind = 'trial'").execute(&h.state.pool).await.unwrap();
    let due = sync::due_bindings(&mut h.state.pool.acquire().await.unwrap(), 10).await.unwrap();
    assert_eq!(due, vec![binding_id], "an expired bucket makes the binding due even when idle");

    let outcome = sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(outcome, sync::SyncOutcome::Synced { consumed: 0, pushed: -TRIAL });
    let (_, remaining, reason) = h.bucket("trial").await;
    assert_eq!((remaining, reason.as_deref()), (0, Some("expired")));
    assert_eq!(h.mock.quota_of(user_id), 400_000);
    let (expired,): (i64,) = sqlx::query_as("SELECT -delta FROM quota_ledger WHERE kind = 'expire'").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(expired, TRIAL);

    h.cleanup().await;
}

#[tokio::test]
async fn manual_credit_in_new_api_is_flagged_and_realigned() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();

    h.mock.credit(user_id, 90_000);
    let outcome = sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(outcome, sync::SyncOutcome::Synced { consumed: 0, pushed: -90_000 });
    assert_eq!(h.mock.quota_of(user_id), TRIAL, "the wallet is pulled back to what the buckets say");
    let (open, detail): (i64, String) =
        sqlx::query_as("SELECT COUNT(*), MAX(detail) FROM sync_anomalies WHERE resolved_at IS NULL AND kind = 'external_credit'")
            .fetch_one(&h.state.pool)
            .await
            .unwrap();
    assert_eq!(open, 1);
    assert!(detail.contains("90000"), "{detail}");

    h.cleanup().await;
}

#[tokio::test]
async fn overspend_is_recorded_and_the_balance_never_goes_negative() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();

    h.mock.consume(user_id, TRIAL + 40_000);
    sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(h.balance(binding_id).await, 0);
    let (overdraft,): (i64,) =
        sqlx::query_as("SELECT -SUM(delta)::BIGINT FROM quota_ledger WHERE kind = 'consume_overdraft'").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(overdraft, 40_000);

    let (status, body) = h.chat(Uuid::nil(), "standard").await;
    assert_eq!(status, StatusCode::OK, "other accounts are unaffected: {body}");
    h.cleanup().await;
}

#[tokio::test]
async fn a_failed_push_is_retried_and_consumption_is_not_counted_twice() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();
    h.mock.consume(user_id, 100_000);

    h.mock.0.lock().unwrap().fail_manage = true;
    let (status, _) = h.grant(binding_id, 500_000).await;
    assert_eq!(status, StatusCode::OK, "the grant itself is recorded even if the push fails");
    assert!(sync::sync_binding(&h.state, binding_id).await.is_err());
    assert_eq!(h.mock.quota_of(user_id), TRIAL - 100_000, "nothing was pushed");
    let (trial_left, ..) = (h.bucket("trial").await.1,);
    assert_eq!(trial_left, TRIAL, "the failed sync rolled back, so the 100000 is not yet settled");

    let due = sync::due_bindings(&mut h.state.pool.acquire().await.unwrap(), 10).await.unwrap();
    assert_eq!(due, vec![binding_id], "an unpushed change keeps the binding due");

    h.mock.0.lock().unwrap().fail_manage = false;
    sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(h.mock.quota_of(user_id), TRIAL - 100_000 + 500_000);
    assert_eq!(h.balance(binding_id).await, h.mock.quota_of(user_id));
    assert_eq!(h.bucket("trial").await.1, TRIAL - 100_000, "the consumption is settled exactly once");
    // 刚用过的绑定本来就会在 15 分钟内持续对账；清掉使用时间后，只剩「未推送的变化」这一个条件，它应已被清除。
    sqlx::query("UPDATE bindings SET last_used_at = NULL").execute(&h.state.pool).await.unwrap();
    assert!(sync::due_bindings(&mut h.state.pool.acquire().await.unwrap(), 10).await.unwrap().is_empty());

    h.cleanup().await;
}

#[tokio::test]
async fn a_signup_bonus_in_new_api_is_flagged_not_trusted() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.mock.0.lock().unwrap().signup_bonus = 777_000;
    h.chat(Uuid::new_v4(), "standard").await;

    assert_eq!(h.mock.only_user().1, TRIAL, "a leftover New API sign-up bonus must not leak extra quota");
    let (open,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sync_anomalies WHERE resolved_at IS NULL").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(open, 1);

    h.cleanup().await;
}

// ---- 档位与门槛 ----

#[tokio::test]
async fn premium_tier_needs_a_paid_bucket_and_uses_the_configured_model() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();
    h.chat(uid, "standard").await;

    let (status, body) = h.chat(uid, "premium").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "paid_required");

    let token = h.token(uid, CLIENT);
    let (_, tiers) = h.call(Method::GET, "/api/ai/tiers", Some(&token), Value::Null).await;
    let by_name: HashMap<String, bool> =
        tiers["tiers"].as_array().unwrap().iter().map(|t| (t["tier"].as_str().unwrap().to_string(), t["available"].as_bool().unwrap())).collect();
    assert_eq!(by_name, HashMap::from([("standard".to_string(), true), ("premium".to_string(), false)]));
    assert!(!tiers.to_string().contains(REAL_PREMIUM) && !tiers.to_string().contains(REAL_STANDARD), "real models must never be listed");

    h.grant(h.only_binding().await, 500_000).await;
    let (status, body) = h.chat(uid, "premium").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["model"], "premium");
    assert_eq!(h.mock.0.lock().unwrap().chat_requests.last().unwrap().1["model"], REAL_PREMIUM);

    h.cleanup().await;
}

#[tokio::test]
async fn real_model_names_and_unknown_tiers_are_rejected_or_hidden() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();

    for requested in [REAL_STANDARD, "gpt-4", "STANDARD"] {
        let (status, body) = h.chat(uid, requested).await;
        assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::BAD_REQUEST, Some("model_not_found")), "{requested}");
    }
    assert_eq!(h.mock.user_count(), 0, "a rejected request must not provision anything");

    let token = h.token(uid, CLIENT);
    let (status, models) = h.call(Method::GET, "/v1/models", Some(&token), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<&str> = models["data"].as_array().unwrap().iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec!["standard", "premium"]);

    h.cleanup().await;
}

#[tokio::test]
async fn missing_model_uses_the_default_tier() {
    let h = harness!();
    h.seed_wallpaper().await;
    let token = h.token(Uuid::new_v4(), CLIENT);
    let (status, _) = h.call(Method::POST, "/v1/chat/completions", Some(&token), json!({ "messages": [] })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(h.mock.0.lock().unwrap().chat_requests[0].1["model"], REAL_STANDARD);
    h.cleanup().await;
}

#[tokio::test]
async fn exhausted_balance_is_refused_with_payment_required() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();
    h.chat(uid, "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();
    h.mock.consume(user_id, TRIAL);
    sync::sync_binding(&h.state, binding_id).await.unwrap();

    let before = h.mock.0.lock().unwrap().chat_requests.len();
    let (status, body) = h.chat(uid, "standard").await;
    assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::PAYMENT_REQUIRED, Some("insufficient_quota")));
    assert_eq!(h.mock.0.lock().unwrap().chat_requests.len(), before, "nothing is forwarded without balance");

    h.cleanup().await;
}

// ---- 鉴权 ----

#[tokio::test]
async fn only_known_clients_with_valid_tokens_can_call() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();

    let (status, _) = h.call(Method::POST, "/v1/chat/completions", None, json!({})).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = h.call(Method::POST, "/v1/chat/completions", Some("garbage"), json!({})).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let stranger = h.token(uid, "unknown-app");
    let (status, body) = h.call(Method::POST, "/v1/chat/completions", Some(&stranger), json!({})).await;
    assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::FORBIDDEN, Some("forbidden")));

    let other_client = h.token(uid, "xiaoban-mobile-wallpaper");
    let (status, _) = h.call(Method::POST, "/v1/chat/completions", Some(&other_client), json!({ "model": "standard", "messages": [] })).await;
    assert_eq!(status, StatusCode::OK);
    h.chat(uid, "standard").await;
    assert_eq!(h.mock.user_count(), 1, "every client of one product shares one account balance");

    let user_token = h.token(uid, CLIENT);
    let (status, _) = h.call(Method::GET, "/api/admin/overview", Some(&user_token), Value::Null).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "a user token must not open the admin API");
    h.cleanup().await;
}

// ---- 响应改写与错误 ----

#[tokio::test]
async fn streamed_responses_carry_the_tier_instead_of_the_real_model() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.mock.0.lock().unwrap().chat_mode = ChatMode::Sse;
    let token = h.token(Uuid::new_v4(), CLIENT);
    let (status, text) = h
        .raw(Method::POST, "/v1/chat/completions", Some(&token), json!({ "model": "standard", "stream": true, "messages": [] }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(text.contains(r#""model":"standard""#) && !text.contains(REAL_STANDARD), "{text}");
    assert!(text.trim_end().ends_with("data: [DONE]"), "{text}");
    h.cleanup().await;
}

#[tokio::test]
async fn upstream_failures_do_not_leak_model_or_channel_details() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.mock.0.lock().unwrap().chat_mode = ChatMode::Fail;
    let (status, text) = {
        let token = h.token(Uuid::new_v4(), CLIENT);
        h.raw(Method::POST, "/v1/chat/completions", Some(&token), json!({ "model": "standard", "messages": [] })).await
    };
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert!(!text.contains(REAL_STANDARD) && !text.contains("channel"), "{text}");
    let body: Value = serde_json::from_str(&text).unwrap();
    assert!(body["error"]["message"].is_string(), "errors use the OpenAI shape so SDKs can read them: {text}");
    h.cleanup().await;
}

// ---- 余额接口 ----

#[tokio::test]
async fn balance_does_not_provision_and_lists_live_buckets_afterwards() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();
    let token = h.token(uid, CLIENT);

    let (status, body) = h.call(Method::GET, "/api/ai/balance", Some(&token), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["total"].as_i64(), body["trial"]["available"].as_bool(), body["trial"]["quota"].as_i64()), (Some(0), Some(true), Some(TRIAL)));
    assert_eq!(h.mock.user_count(), 0, "reading the balance must not create a New API user");

    h.chat(uid, "standard").await;
    let (_, body) = h.call(Method::GET, "/api/ai/balance", Some(&token), Value::Null).await;
    assert_eq!((body["total"].as_i64(), body["paid"].as_i64(), body["trial"]["available"].as_bool()), (Some(TRIAL), Some(0), Some(false)));
    assert_eq!(body["buckets"].as_array().unwrap().len(), 1);
    assert!(body["buckets"][0]["expires_at"].is_string());

    let (_, ledger) = h.call(Method::GET, "/api/ai/ledger", Some(&token), Value::Null).await;
    assert_eq!(ledger["items"][0]["kind"], "grant");
    h.cleanup().await;
}

// ---- 管理接口 ----

#[tokio::test]
async fn tier_saves_are_validated_against_the_gateway() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    let put = |tier: &str, model: &str, default: bool, paid: bool, enabled: bool| {
        let admin = admin.clone();
        let path = format!("/api/admin/products/wallpaper/tiers/{tier}");
        let body = json!({ "display_name": "测试", "model": model, "requires_paid": paid, "is_default": default, "sort_order": 9, "enabled": enabled });
        let h = &h;
        async move { h.call(Method::PUT, &path, Some(&admin), body).await }
    };

    let (status, body) = put("lab", "no-such-model", false, false, true).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, body) = put("lab", "unpriced-model", false, false, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["warnings"].as_array().unwrap().len(), 1, "a model still at the default ratio is flagged: {body}");
    let (_, body) = put("lab", REAL_STANDARD, false, false, true).await;
    assert!(body["warnings"].as_array().unwrap().is_empty());

    for (default, paid, enabled) in [(true, true, true), (true, false, false)] {
        let (status, _) = put("lab", REAL_STANDARD, default, paid, enabled).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "a default tier must be enabled and free");
    }
    let (status, _) = put("BadSlug", REAL_STANDARD, false, false, true).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    put("lab", REAL_STANDARD, true, false, true).await;
    let tiers = products::tiers(&h.state.pool, "wallpaper", false).await.unwrap();
    let defaults: Vec<&str> = tiers.iter().filter(|t| t.is_default).map(|t| t.tier.as_str()).collect();
    assert_eq!(defaults, vec!["lab"], "making a tier the default demotes the previous one");

    let (_, listing) = h.call(Method::GET, "/api/admin/models", Some(&admin), Value::Null).await;
    let flagged: Vec<&str> =
        listing["items"].as_array().unwrap().iter().filter(|m| m["ratio_warning"] == true).map(|m| m["model_name"].as_str().unwrap()).collect();
    assert_eq!(flagged, vec!["unpriced-model"]);
    h.cleanup().await;
}

#[tokio::test]
async fn a_client_can_belong_to_only_one_product() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    let (status, body) = h
        .call(Method::PUT, "/api/admin/products/other", Some(&admin), json!({ "name": "别的", "client_ids": [CLIENT], "trial_quota": 0, "trial_days": 7, "enabled": true }))
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    h.cleanup().await;
}

#[tokio::test]
async fn admin_can_grant_and_claw_back_and_everything_is_audited() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();
    let admin = h.admin_token().await;

    for bad in [0, -5, 2_000_000_000] {
        let (status, _) = h.grant(binding_id, bad).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
    }
    let (status, body) = h.grant(binding_id, 600_000).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let bucket_id = body["bucket_id"].as_i64().unwrap();
    assert_eq!(h.mock.quota_of(user_id), TRIAL + 600_000);

    let (status, body) = h.call(Method::POST, &format!("/api/admin/buckets/{bucket_id}/void"), Some(&admin), json!({ "note": "退款" })).await;
    assert_eq!((status, body["voided"].as_i64()), (StatusCode::OK, Some(600_000)));
    assert_eq!(h.mock.quota_of(user_id), TRIAL, "clawing back removes the quota from the wallet");
    let (status, _) = h.call(Method::POST, &format!("/api/admin/buckets/{bucket_id}/void"), Some(&admin), json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "a bucket can only be voided once");

    let (_, detail) = h.call(Method::GET, &format!("/api/admin/bindings/{binding_id}"), Some(&admin), Value::Null).await;
    assert_eq!(detail["binding"]["balance"], TRIAL);
    assert!(detail["ledger"].as_array().unwrap().iter().any(|r| r["kind"] == "void"));

    let (_, audit) = h.call(Method::GET, "/api/admin/audit", Some(&admin), Value::Null).await;
    let actions: Vec<&str> = audit["items"].as_array().unwrap().iter().map(|r| r["action"].as_str().unwrap()).collect();
    assert!(actions.contains(&"binding.grant") && actions.contains(&"bucket.void"), "{actions:?}");

    let (_, overview) = h.call(Method::GET, "/api/admin/overview", Some(&admin), Value::Null).await;
    assert_eq!((overview["bindings"].as_i64(), overview["balance"].as_i64(), overview["newapi_configured"].as_bool()), (Some(1), Some(TRIAL), Some(true)));
    let (_, listing) = h.call(Method::GET, "/api/admin/bindings?q=nobody", Some(&admin), Value::Null).await;
    assert_eq!(listing["total"], 0);
    h.cleanup().await;
}

#[tokio::test]
async fn admin_login_locks_after_repeated_failures_and_password_change_revokes_tokens() {
    let h = harness!();
    let old = h.admin_token().await;
    for _ in 0..5 {
        let (status, _) = h.call(Method::POST, "/api/admin/login", None, json!({ "username": "root", "password": "wrong-password" })).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (status, _) = h.call(Method::POST, "/api/admin/login", None, json!({ "username": "root", "password": ADMIN_PASSWORD })).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "even the right password is refused while locked");
    let (status, _) = h.call(Method::POST, "/api/admin/login", None, json!({ "username": "ghost", "password": "whatever-123" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "unknown users look the same as wrong passwords");

    let (status, _) = h
        .call(Method::POST, "/api/admin/password", Some(&old), json!({ "current_password": ADMIN_PASSWORD, "new_password": "brand-new-pass-1" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = h.call(Method::GET, "/api/admin/me", Some(&old), Value::Null).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "changing the password invalidates earlier admin tokens");
    h.cleanup().await;
}

#[tokio::test]
async fn trial_is_not_granted_when_the_product_has_no_trial_quota() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    h.call(Method::PUT, "/api/admin/products/wallpaper", Some(&admin), json!({ "name": "小伴壁纸", "client_ids": [CLIENT], "trial_quota": 0, "trial_days": 7, "enabled": true })).await;

    let (status, body) = h.chat(Uuid::new_v4(), "standard").await;
    assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::PAYMENT_REQUIRED, Some("insufficient_quota")));
    assert_eq!(h.mock.user_count(), 1, "the account is still provisioned so a later top-up works");
    h.cleanup().await;
}

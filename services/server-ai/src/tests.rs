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
    admin_token: String,
    /// 每次转发到 /v1/* 收到的 (路径, 全部请求头名, Authorization, 请求体)。
    relayed: Vec<Relayed>,
    /// New API has no channel for the search models.
    tools_down: bool,
}

struct Relayed {
    path: String,
    header_names: Vec<String>,
    anthropic_version: Option<String>,
    authorization: String,
    body: Value,
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
    fn accept_admin_token(&self, token: &str) {
        self.0.lock().unwrap().admin_token = token.to_string();
    }
    fn relayed(&self, path: &str) -> Vec<Relayed> {
        self.0.lock().unwrap().relayed.iter().filter(|r| r.path == path).map(|r| Relayed {
            path: r.path.clone(),
            header_names: r.header_names.clone(),
            anthropic_version: r.anthropic_version.clone(),
            authorization: r.authorization.clone(),
            body: r.body.clone(),
        }).collect()
    }
    fn consume(&self, id: i64, amount: i64) {
        self.0.lock().unwrap().users.get_mut(&id).unwrap().quota -= amount;
    }
    fn credit(&self, id: i64, amount: i64) {
        self.0.lock().unwrap().users.get_mut(&id).unwrap().quota += amount;
    }
}

fn authorized(m: &Mock, headers: &HeaderMap) -> bool {
    let expected = m.0.lock().unwrap().admin_token.clone();
    headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) == Some(&format!("Bearer {expected}"))
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
    if !authorized(&m, &headers) {
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
    if !authorized(&m, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let keyword = q.get("keyword").cloned().unwrap_or_default();
    let inner = m.0.lock().unwrap();
    let items: Vec<Value> = inner.users.iter().filter(|(_, u)| u.username.contains(&keyword)).map(|(id, u)| user_json(*id, u)).collect();
    ok(json!({ "items": items, "total": items.len() }))
}

async fn mock_get_user(State(m): State<Arc<Mock>>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    if !authorized(&m, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match m.0.lock().unwrap().users.get(&id) {
        Some(user) => ok(user_json(id, user)),
        None => refused("record not found"),
    }
}

async fn mock_manage(State(m): State<Arc<Mock>>, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    if !authorized(&m, &headers) {
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
    if !authorized(&m, &headers) {
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

/// 假的 /v1/responses 与 /v1/messages：和对话补全一样按 Key 扣钱，响应里把真实模型放在协议规定的位置，
/// 并故意在用户数据（工具参数）里也放一个叫 model 的字段，用来确认代理不会误改。
fn mock_caller(m: &Mock, headers: &HeaderMap) -> Option<i64> {
    let authorization = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or_default();
    let key = authorization.strip_prefix("Bearer sk-")?;
    m.0.lock().unwrap().users.iter().find(|(_, u)| u.key == key).map(|(id, _)| *id)
}

fn record(m: &Mock, path: &str, headers: &HeaderMap, body: Value) {
    m.0.lock().unwrap().relayed.push(Relayed {
        path: path.into(),
        header_names: headers.keys().map(|k| k.as_str().to_string()).collect(),
        anthropic_version: None,
        authorization: headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or_default().into(),
        body,
    });
}

async fn mock_images(State(m): State<Arc<Mock>>, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    let Some(id) = mock_caller(&m, &headers) else { return StatusCode::UNAUTHORIZED.into_response() };
    record(&m, "/v1/images/generations", &headers, body.clone());
    m.consume(id, 2_000);
    Json(json!({ "created": 1, "model": body["model"], "data": [{ "b64_json": "iVBORw0K", "revised_prompt": "一只猫" }], "usage": { "total_tokens": 10 } })).into_response()
}

async fn mock_transcribe(State(m): State<Arc<Mock>>, headers: HeaderMap, mut form: axum::extract::Multipart) -> Response {
    let Some(id) = mock_caller(&m, &headers) else { return StatusCode::UNAUTHORIZED.into_response() };
    let mut seen = serde_json::Map::new();
    while let Some(field) = form.next_field().await.unwrap() {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            seen.insert("file_name".into(), json!(field.file_name()));
            seen.insert("content_type".into(), json!(field.content_type()));
            seen.insert("file_bytes".into(), json!(field.bytes().await.unwrap().len()));
        } else {
            seen.insert(name, json!(field.text().await.unwrap()));
        }
    }
    record(&m, "/v1/audio/transcriptions", &headers, Value::Object(seen));
    m.consume(id, 1_000);
    Json(json!({ "text": " 你好，小伴 " })).into_response()
}

/// New API's relay of DeepSeek Search / Fetch: bills the caller's wallet itself. Answers 503 (no channel for
/// `deepseek-search`) while `tools_down` is set.
async fn mock_tools(State(m): State<Arc<Mock>>, uri: axum::http::Uri, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    let Some(id) = mock_caller(&m, &headers) else { return StatusCode::UNAUTHORIZED.into_response() };
    if m.0.lock().unwrap().tools_down { return (StatusCode::SERVICE_UNAVAILABLE, "no available channel for model deepseek-search").into_response(); }
    record(&m, uri.path(), &headers, body.clone());
    m.consume(id, 375);
    if uri.path() == "/v1/tools/fetch" { return Json(json!({ "url": body["url"], "title": "经网关的页面", "content": "# 正文" })).into_response(); }
    Json(json!({ "results": [{ "title": "经 New API", "url": "https://example.com/n", "snippet": "s", "date": null }] })).into_response()
}

/// Says which model it was opened for, then echoes every event back.
async fn mock_realtime(State(m): State<Arc<Mock>>, headers: HeaderMap, Query(query): Query<HashMap<String, String>>, ws: axum::extract::ws::WebSocketUpgrade) -> Response {
    use axum::extract::ws::Message;
    if mock_caller(&m, &headers).is_none() { return StatusCode::UNAUTHORIZED.into_response() }
    record(&m, "/v1/realtime", &headers, json!({ "model": query.get("model") }));
    let model = query.get("model").cloned().unwrap_or_default();
    ws.on_upgrade(move |mut socket| async move {
        let created = json!({ "type": "session.created", "session": { "id": "sess_1", "model": model } });
        if socket.send(Message::Text(created.to_string().into())).await.is_err() { return; }
        while let Some(Ok(message)) = socket.recv().await {
            let Message::Text(text) = message else { continue };
            let got: Value = serde_json::from_str(text.as_str()).unwrap_or(Value::Null);
            if socket.send(Message::Text(json!({ "type": "echo", "got": got }).to_string().into())).await.is_err() { break; }
        }
    })
}

async fn mock_relay(State(m): State<Arc<Mock>>, uri: axum::http::Uri, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    let path = uri.path().to_string();
    let authorization = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
    let mut inner = m.0.lock().unwrap();
    inner.relayed.push(Relayed {
        path: path.clone(),
        header_names: headers.keys().map(|k| k.as_str().to_string()).collect(),
        anthropic_version: headers.get("anthropic-version").and_then(|v| v.to_str().ok()).map(str::to_string),
        authorization: authorization.clone(),
        body: body.clone(),
    });
    let Some(key) = authorization.strip_prefix("Bearer sk-") else { return StatusCode::UNAUTHORIZED.into_response() };
    let Some(id) = inner.users.iter().find(|(_, u)| u.key == key).map(|(id, _)| *id) else { return StatusCode::UNAUTHORIZED.into_response() };
    let cost = inner.chat_cost;
    inner.users.get_mut(&id).unwrap().quota -= cost;
    let model = body["model"].as_str().unwrap_or_default().to_string();
    let anthropic = path == "/v1/messages";
    let streaming = body["stream"] == json!(true);
    match (inner.chat_mode, anthropic, streaming) {
        (ChatMode::Fail, _, _) => (StatusCode::INTERNAL_SERVER_ERROR, format!("model {model} failed on channel 7")).into_response(),
        (_, true, false) => Json(json!({ "id": "msg_1", "type": "message", "role": "assistant", "model": format!("{model}-20260101"),
            "content": [{ "type": "tool_use", "id": "t1", "name": "pick", "input": { "model": "keep-me" } }], "stop_reason": "tool_use" })).into_response(),
        (_, false, false) => Json(json!({ "id": "resp_1", "object": "response", "model": model, "status": "completed",
            "output": [{ "type": "function_call", "name": "pick", "arguments": "{\"model\":\"keep-me\"}" }] })).into_response(),
        (_, true, true) => {
            let events = [
                ("message_start", json!({ "type": "message_start", "message": { "id": "msg_1", "model": format!("{model}-20260101"), "role": "assistant" } })),
                ("content_block_start", json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "tool_use", "input": { "model": "keep-me" } } })),
                ("message_stop", json!({ "type": "message_stop" })),
            ];
            let text: String = events.iter().map(|(name, data)| format!("event: {name}\ndata: {data}\n\n")).collect();
            Response::builder().header(header::CONTENT_TYPE, "text/event-stream").body(Body::from(text)).unwrap()
        }
        (_, false, true) => {
            let events = [
                ("response.created", json!({ "type": "response.created", "response": { "id": "resp_1", "model": model, "status": "in_progress" } })),
                ("response.output_item.done", json!({ "type": "response.output_item.done", "item": { "type": "function_call", "model": "keep-me" } })),
                ("response.completed", json!({ "type": "response.completed", "response": { "id": "resp_1", "model": model, "status": "completed" } })),
            ];
            let text: String = events.iter().map(|(name, data)| format!("event: {name}\ndata: {data}\n\n")).collect();
            Response::builder().header(header::CONTENT_TYPE, "text/event-stream").body(Body::from(text)).unwrap()
        }
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
        admin_token: ADMIN_TOKEN.to_string(),
        relayed: Vec::new(),
        tools_down: false,
    })));
    let app = Router::new()
        .route("/api/user/", post(mock_create))
        .route("/api/user/search", get(mock_search))
        .route("/api/user/manage", post(mock_manage))
        .route("/api/user/{id}", get(mock_get_user))
        .route("/api/user/{id}/token", post(mock_token))
        .route("/api/pricing", get(mock_pricing))
        .route("/v1/chat/completions", post(mock_chat))
        .route("/v1/responses", post(mock_relay))
        .route("/v1/messages", post(mock_relay))
        .route("/v1/images/generations", post(mock_images))
        .route("/v1/audio/transcriptions", post(mock_transcribe).layer(axum::extract::DefaultBodyLimit::disable()))
        .route("/v1/realtime", get(mock_realtime))
        .route("/v1/tools/search", post(mock_tools))
        .route("/v1/tools/fetch", post(mock_tools))
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

        let (redeem_user_limiter, redeem_ip_limiter) = crate::state::redeem_limiters();
        let state = AppState {
            pool,
            redeem_user_limiter,
            redeem_ip_limiter,
            newapi: Arc::new(NewApi::new(base, Some(ADMIN_TOKEN.to_string()))),
            search: Arc::new(crate::search::Search::from_config(&config)),
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
    assert_eq!(
        (overview["granted_total"].as_i64(), overview["consumed_total"].as_i64(), overview["lapsed_total"].as_i64()),
        (Some(TRIAL + 600_000), Some(0), Some(600_000)),
        "every granted point is accounted for: the claw-back counts as lapsed, not consumed"
    );
    let series = overview["series"].as_array().unwrap();
    assert_eq!(series.len(), 14, "two weeks, oldest first, one entry per day even when nothing happened");
    let days: Vec<&str> = series.iter().map(|d| d["day"].as_str().unwrap()).collect();
    assert!(days.windows(2).all(|w| w[0] < w[1]), "days ascend: {days:?}");
    assert_eq!(series.iter().map(|d| d["granted"].as_i64().unwrap()).sum::<i64>(), TRIAL + 600_000, "the daily grants add up to the total");
    assert_eq!(series[13]["granted"], TRIAL + 600_000, "today is the last entry");
    assert!(series[..13].iter().all(|d| d["granted"] == 0 && d["consumed"] == 0));
    let product = &overview["products"][0];
    assert_eq!((product["product_id"].as_str(), product["bindings"].as_i64(), product["balance"].as_i64(), product["consumed_7d"].as_i64()), (Some("wallpaper"), Some(1), Some(TRIAL), Some(0)));
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

// ---- 兑换码 ----

const TOPUP: i64 = 1_000_000;

impl Harness {
    async fn make_batch(&self, kind: &str, quota: i64, days: i64, count: i64) -> Vec<String> {
        let admin = self.admin_token().await;
        let (status, body) = self
            .call(
                Method::POST,
                "/api/admin/batches",
                Some(&admin),
                json!({ "product_id": "wallpaper", "kind": kind, "quota": quota, "valid_days": days, "count": count, "note": "淘宝 5 美元包" }),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body["codes"].as_array().unwrap().iter().map(|c| c.as_str().unwrap().to_string()).collect()
    }

    async fn redeem(&self, uid: Uuid, client: &str, code: &str) -> (StatusCode, Value) {
        let token = self.token(uid, client);
        self.call(Method::POST, "/api/ai/redeem", Some(&token), json!({ "code": code })).await
    }

    async fn code_status(&self, code: &str) -> String {
        let canonical = crate::codes::canonical(code).unwrap();
        let (status,): (String,) = sqlx::query_as("SELECT status FROM redeem_codes WHERE code_hash = $1")
            .bind(self.state.keys.code_hash(&canonical))
            .fetch_one(&self.state.pool)
            .await
            .unwrap();
        status
    }
}

#[tokio::test]
async fn a_batch_returns_valid_unique_codes_and_stores_only_hashes() {
    let h = harness!();
    h.seed_wallpaper().await;
    let codes = h.make_batch("topup", TOPUP, 365, 50).await;

    assert_eq!(codes.len(), 50);
    assert_eq!(codes.iter().collect::<std::collections::HashSet<_>>().len(), 50);
    for code in &codes {
        assert_eq!(code.len(), 19, "four groups of four joined by dashes: {code}");
        assert!(crate::codes::canonical(code).is_some(), "{code}");
    }

    let stored: Vec<(Vec<u8>,)> = sqlx::query_as("SELECT code_hash FROM redeem_codes").fetch_all(&h.state.pool).await.unwrap();
    assert_eq!(stored.len(), 50);
    let canonical = crate::codes::canonical(&codes[0]).unwrap();
    assert!(stored.iter().all(|(hash,)| hash.len() == 32 && hash.as_slice() != canonical.as_bytes()), "only keyed hashes are stored");
    let (leaks,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admin_audit WHERE detail LIKE '%' || $1 || '%'").bind(&canonical[..8]).fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(leaks, 0, "codes must not appear in the audit log");

    let admin = h.admin_token().await;
    for (body, why) in [
        (json!({ "product_id": "wallpaper", "kind": "topup", "quota": TOPUP, "valid_days": 30, "count": 0 }), "no codes"),
        (json!({ "product_id": "wallpaper", "kind": "topup", "quota": TOPUP, "valid_days": 30, "count": 5000 }), "too many"),
        (json!({ "product_id": "wallpaper", "kind": "trial", "quota": TOPUP, "valid_days": 30, "count": 1 }), "bad kind"),
        (json!({ "product_id": "wallpaper", "kind": "topup", "quota": 0, "valid_days": 30, "count": 1 }), "zero face value"),
        (json!({ "product_id": "wallpaper", "kind": "topup", "quota": TOPUP, "valid_days": 0, "count": 1 }), "zero days"),
        (json!({ "product_id": "ghost", "kind": "topup", "quota": TOPUP, "valid_days": 30, "count": 1 }), "unknown product"),
    ] {
        let (status, _) = h.call(Method::POST, "/api/admin/batches", Some(&admin), body).await;
        assert!(status.is_client_error(), "{why}");
    }
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM redeem_codes").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(total, 50, "rejected requests must not create codes");
    h.cleanup().await;
}

#[tokio::test]
async fn redeeming_adds_a_dated_bucket_and_pushes_it_to_the_wallet_once() {
    let h = harness!();
    h.seed_wallpaper().await;
    let codes = h.make_batch("monthly", TOPUP, 30, 2).await;
    let uid = Uuid::new_v4();
    h.chat(uid, "standard").await;
    let (user_id, _) = h.mock.only_user();

    let (status, body) = h.redeem(uid, CLIENT, &codes[0]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!((body["kind"].as_str(), body["quota"].as_i64(), body["balance"].as_i64()), (Some("monthly"), Some(TOPUP), Some(TRIAL + TOPUP)));
    assert_eq!(h.mock.quota_of(user_id), TRIAL + TOPUP, "the wallet already holds the new quota when the response returns");

    let (kind, total, days_left): (String, i64, f64) = sqlx::query_as(
        "SELECT kind, total, (EXTRACT(EPOCH FROM expires_at - now()) / 86400)::FLOAT8 FROM quota_buckets WHERE kind = 'monthly'",
    )
    .fetch_one(&h.state.pool)
    .await
    .unwrap();
    assert_eq!((kind.as_str(), total), ("monthly", TOPUP));
    assert!((29.9..=30.0).contains(&days_left), "validity counts from the day of redemption: {days_left}");
    assert_eq!(h.code_status(&codes[0]).await, "redeemed");
    assert_eq!(h.code_status(&codes[1]).await, "unused", "the other code in the batch is untouched");

    let (status, body) = h.redeem(uid, CLIENT, &codes[0]).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::CONFLICT, Some("code_used")));
    assert_eq!(h.mock.quota_of(user_id), TRIAL + TOPUP, "a second attempt adds nothing");

    let (status, body) = h.redeem(Uuid::new_v4(), CLIENT, &codes[0]).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::CONFLICT, Some("code_used")), "another account cannot reuse it either");
    h.cleanup().await;
}

#[tokio::test]
async fn redeeming_before_any_chat_opens_the_account_with_trial_plus_the_code() {
    let h = harness!();
    h.seed_wallpaper().await;
    let codes = h.make_batch("topup", TOPUP, 365, 1).await;

    let (status, body) = h.redeem(Uuid::new_v4(), "xiaoban-mobile-wallpaper", &codes[0].to_lowercase().replace('-', " ")).await;
    assert_eq!(status, StatusCode::OK, "lowercase and spaces are accepted: {body}");
    assert_eq!(h.mock.user_count(), 1);
    assert_eq!(h.mock.only_user().1, TRIAL + TOPUP);
    let (premium_status, _) = h.chat(Uuid::new_v4(), "premium").await;
    assert_eq!(premium_status, StatusCode::FORBIDDEN, "other accounts are unaffected");
    h.cleanup().await;
}

#[tokio::test]
async fn a_paid_code_unlocks_the_premium_tier() {
    let h = harness!();
    h.seed_wallpaper().await;
    let codes = h.make_batch("topup", TOPUP, 365, 1).await;
    let uid = Uuid::new_v4();

    let (status, _) = h.chat(uid, "premium").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(h.redeem(uid, CLIENT, &codes[0]).await.0, StatusCode::OK);
    let (status, body) = h.chat(uid, "premium").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    h.cleanup().await;
}

#[tokio::test]
async fn simultaneous_redemptions_of_one_code_succeed_exactly_once() {
    let h = harness!();
    h.seed_wallpaper().await;
    let codes = h.make_batch("topup", TOPUP, 365, 1).await;
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    h.chat(a, "standard").await;
    h.chat(b, "standard").await;

    let (r1, r2, r3, r4) = tokio::join!(h.redeem(a, CLIENT, &codes[0]), h.redeem(b, CLIENT, &codes[0]), h.redeem(a, CLIENT, &codes[0]), h.redeem(b, CLIENT, &codes[0]));
    let outcomes = [r1, r2, r3, r4];
    assert_eq!(outcomes.iter().filter(|(s, _)| *s == StatusCode::OK).count(), 1, "{outcomes:?}");
    assert!(outcomes.iter().filter(|(s, _)| *s != StatusCode::OK).all(|(s, v)| *s == StatusCode::CONFLICT && v["code"] == "code_used"), "{outcomes:?}");
    let (buckets,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota_buckets WHERE kind = 'topup'").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(buckets, 1);
    let (granted,): (i64,) = sqlx::query_as("SELECT COALESCE(SUM(delta), 0)::BIGINT FROM quota_ledger WHERE kind = 'grant' AND bucket_id IN (SELECT id FROM quota_buckets WHERE kind = 'topup')").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(granted, TOPUP, "the quota is credited once, not once per request");
    h.cleanup().await;
}

#[tokio::test]
async fn a_code_only_works_for_its_own_product() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    h.call(Method::PUT, "/api/admin/products/forum", Some(&admin), json!({ "name": "论坛", "client_ids": ["forum-web"], "trial_quota": 0, "trial_days": 7, "enabled": true })).await;
    let codes = h.make_batch("topup", TOPUP, 365, 1).await;
    let uid = Uuid::new_v4();

    let (status, body) = h.redeem(uid, "forum-web", &codes[0]).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::BAD_REQUEST, Some("code_wrong_product")));
    assert_eq!(h.code_status(&codes[0]).await, "unused", "a wrong-product attempt must not burn the code");
    assert_eq!(h.mock.user_count(), 0, "and must not provision anything");
    assert_eq!(h.redeem(uid, CLIENT, &codes[0]).await.0, StatusCode::OK);
    h.cleanup().await;
}

#[tokio::test]
async fn unknown_malformed_and_unauthenticated_attempts_are_refused_without_side_effects() {
    let h = harness!();
    h.seed_wallpaper().await;
    h.make_batch("topup", TOPUP, 365, 1).await;
    let uid = Uuid::new_v4();
    let unknown = crate::codes::display(&crate::codes::generate());

    for (code, label) in [(unknown.as_str(), "well-formed but not ours"), ("ABCD-EFGH", "too short"), ("", "empty"), ("我是兑换码", "not a code")] {
        let (status, body) = h.redeem(uid, CLIENT, code).await;
        assert_eq!((status, body["code"].as_str()), (StatusCode::BAD_REQUEST, Some("code_invalid")), "{label}");
    }
    assert_eq!(h.mock.user_count(), 0, "bad codes never create a gateway user");

    let (status, _) = h.call(Method::POST, "/api/ai/redeem", None, json!({ "code": unknown })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let stranger = h.token(uid, "unknown-app");
    let (status, _) = h.call(Method::POST, "/api/ai/redeem", Some(&stranger), json!({ "code": unknown })).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    h.cleanup().await;
}

#[tokio::test]
async fn repeated_wrong_guesses_lock_the_account_but_successes_do_not_count() {
    let h = harness!();
    h.seed_wallpaper().await;
    let good = h.make_batch("topup", 100_000, 365, 12).await;
    let uid = Uuid::new_v4();

    for code in &good[..10] {
        assert_eq!(h.redeem(uid, CLIENT, code).await.0, StatusCode::OK, "ten genuine redemptions in a row are fine");
    }
    let wrong = crate::codes::display(&crate::codes::generate());
    for _ in 0..8 {
        assert_eq!(h.redeem(uid, CLIENT, &wrong).await.0, StatusCode::BAD_REQUEST);
    }
    let (status, body) = h.redeem(uid, CLIENT, &good[10]).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::TOO_MANY_REQUESTS, Some("too_many_attempts")), "{body}");
    assert_eq!(h.code_status(&good[10]).await, "unused", "even a genuine code is refused while locked");

    let (status, _) = h.redeem(Uuid::new_v4(), CLIENT, &good[10]).await;
    assert_eq!(status, StatusCode::OK, "one account's lock does not lock out a different account behind the same address");
    h.cleanup().await;
}

#[tokio::test]
async fn many_accounts_guessing_from_one_address_trip_the_address_lock() {
    let h = harness!();
    h.seed_wallpaper().await;
    let good = h.make_batch("topup", 100_000, 365, 1).await;
    let wrong = crate::codes::display(&crate::codes::generate());

    // 每个账号只错一次，永远碰不到单账号的上限；只有按来源地址累计才拦得住这种分散的猜测。
    for _ in 0..60 {
        assert_eq!(h.redeem(Uuid::new_v4(), CLIENT, &wrong).await.0, StatusCode::BAD_REQUEST);
    }
    let (status, body) = h.redeem(Uuid::new_v4(), CLIENT, &good[0]).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::TOO_MANY_REQUESTS, Some("too_many_attempts")), "{body}");
    assert_eq!(h.code_status(&good[0]).await, "unused");
    h.cleanup().await;
}

#[tokio::test]
async fn a_failed_gateway_push_after_redeeming_is_retried_by_the_sync() {
    let h = harness!();
    h.seed_wallpaper().await;
    let codes = h.make_batch("topup", TOPUP, 365, 1).await;
    let uid = Uuid::new_v4();
    h.chat(uid, "standard").await;
    let (user_id, _) = h.mock.only_user();

    h.mock.0.lock().unwrap().fail_manage = true;
    let (status, body) = h.redeem(uid, CLIENT, &codes[0]).await;
    assert_eq!(status, StatusCode::OK, "the redemption itself is recorded even if the push fails: {body}");
    assert_eq!(h.mock.quota_of(user_id), TRIAL, "nothing reached the wallet yet");
    assert_eq!(h.code_status(&codes[0]).await, "redeemed");

    h.mock.0.lock().unwrap().fail_manage = false;
    sync::sync_binding(&h.state, h.only_binding().await).await.unwrap();
    assert_eq!(h.mock.quota_of(user_id), TRIAL + TOPUP);
    h.cleanup().await;
}

#[tokio::test]
async fn refunds_void_unused_codes_and_claw_back_redeemed_ones() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    let codes = h.make_batch("topup", TOPUP, 365, 4).await;
    let uid = Uuid::new_v4();
    h.chat(uid, "standard").await;
    let (user_id, _) = h.mock.only_user();
    assert_eq!(h.redeem(uid, CLIENT, &codes[0]).await.0, StatusCode::OK);

    let lookup = |code: String| {
        let admin = admin.clone();
        let h = &h;
        async move { h.call(Method::POST, "/api/admin/codes/lookup", Some(&admin), json!({ "code": code })).await }
    };
    let (status, found) = lookup(codes[1].to_lowercase()).await;
    assert_eq!((status, found["code"]["status"].as_str(), found["code"]["quota"].as_i64()), (StatusCode::OK, Some("unused"), Some(TOPUP)));
    assert_eq!(found["code"]["note"], "淘宝 5 美元包");

    let unused_id = found["code"]["id"].as_i64().unwrap();
    let (status, _) = h.call(Method::POST, &format!("/api/admin/codes/{unused_id}/void"), Some(&admin), json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(h.code_status(&codes[1]).await, "void");
    let (status, body) = h.redeem(Uuid::new_v4(), CLIENT, &codes[1]).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::CONFLICT, Some("code_void")));
    let (status, _) = h.call(Method::POST, &format!("/api/admin/codes/{unused_id}/void"), Some(&admin), json!({})).await;
    assert_eq!(status, StatusCode::CONFLICT, "voiding twice is refused");

    let (_, redeemed) = lookup(codes[0].clone()).await;
    assert_eq!((redeemed["code"]["status"].as_str(), redeemed["code"]["bucket_remaining"].as_i64()), (Some("redeemed"), Some(TOPUP)));
    assert_eq!(redeemed["code"]["redeemed_by"].as_str(), Some(uid.to_string().as_str()));
    let redeemed_id = redeemed["code"]["id"].as_i64().unwrap();
    let (status, body) = h.call(Method::POST, &format!("/api/admin/codes/{redeemed_id}/void"), Some(&admin), json!({})).await;
    assert_eq!(status, StatusCode::CONFLICT, "a redeemed code is clawed back through its bucket instead: {body}");

    let bucket_id = redeemed["code"]["bucket_id"].as_i64().unwrap();
    let (status, _) = h.call(Method::POST, &format!("/api/admin/buckets/{bucket_id}/void"), Some(&admin), json!({ "note": "淘宝退款" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(h.mock.quota_of(user_id), TRIAL, "the refunded quota leaves the wallet");

    let (_, batches) = h.call(Method::GET, "/api/admin/batches", Some(&admin), Value::Null).await;
    let batch_id = batches["items"][0]["id"].as_i64().unwrap();
    let counts = |b: &Value| (b["unused"].as_i64().unwrap(), b["redeemed"].as_i64().unwrap(), b["void"].as_i64().unwrap());
    assert_eq!(counts(&batches["items"][0]), (2, 1, 1));
    let (status, body) = h.call(Method::POST, &format!("/api/admin/batches/{batch_id}/void"), Some(&admin), json!({})).await;
    assert_eq!((status, body["voided"].as_i64()), (StatusCode::OK, Some(2)));
    let (_, batches) = h.call(Method::GET, "/api/admin/batches", Some(&admin), Value::Null).await;
    assert_eq!(counts(&batches["items"][0]), (0, 1, 3), "voiding a batch leaves redeemed codes alone");

    let (_, audit) = h.call(Method::GET, "/api/admin/audit", Some(&admin), Value::Null).await;
    let actions: Vec<&str> = audit["items"].as_array().unwrap().iter().map(|r| r["action"].as_str().unwrap()).collect();
    for expected in ["batch.create", "batch.void", "code.void", "bucket.void"] {
        assert!(actions.contains(&expected), "{expected} missing from {actions:?}");
    }

    let (status, _) = h.call(Method::POST, "/api/admin/codes/lookup", Some(&admin), json!({ "code": "ABCD-EFGH-JKLM-NPQR" })).await;
    assert!(status.is_client_error(), "a code with a bad check character is refused");
    let (status, _) = h.call(Method::POST, "/api/admin/codes/lookup", Some(&admin), json!({ "code": crate::codes::generate() })).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "a well-formed code that we never issued is reported as unknown");
    let user_token = h.token(uid, CLIENT);
    let (status, _) = h.call(Method::POST, "/api/admin/codes/lookup", Some(&user_token), json!({ "code": codes[0] })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "end users cannot look codes up");
    h.cleanup().await;
}

#[tokio::test]
async fn users_never_see_admin_notes_or_operator_names() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();
    h.chat(uid, "standard").await;
    let binding_id = h.only_binding().await;
    let admin = h.admin_token().await;
    h.call(Method::POST, &format!("/api/admin/bindings/{binding_id}/grant"), Some(&admin), json!({ "quota": 300_000, "note": "淘宝订单 7654321" })).await;
    let codes = h.make_batch("topup", TOPUP, 365, 1).await;
    h.redeem(uid, CLIENT, &codes[0]).await;

    let token = h.token(uid, CLIENT);
    let (_, balance) = h.call(Method::GET, "/api/ai/balance", Some(&token), Value::Null).await;
    let (_, ledger) = h.call(Method::GET, "/api/ai/ledger", Some(&token), Value::Null).await;
    for (name, body) in [("balance", &balance), ("ledger", &ledger)] {
        let text = body.to_string();
        for secret in ["root", "7654321", "淘宝", "admin:", "code:", "source", "note"] {
            assert!(!text.contains(secret), "{name} leaks {secret:?}: {text}");
        }
    }
    let titles: Vec<&str> = ledger["items"].as_array().unwrap().iter().map(|r| r["title"].as_str().unwrap()).collect();
    assert!(titles.contains(&"新用户试用额度") && titles.contains(&"兑换码入账") && titles.contains(&"额度补发"), "{titles:?}");
    assert_eq!(balance["buckets"].as_array().unwrap().len(), 3);
    h.cleanup().await;
}

// ---- /v1/responses 与 /v1/messages ----

impl Harness {
    async fn api_call(&self, path: &str, headers: &[(&str, String)], body: Value) -> (StatusCode, String) {
        let mut req = Request::builder().method(Method::POST).uri(path).header(header::CONTENT_TYPE, "application/json");
        for (name, value) in headers {
            req = req.header(*name, value);
        }
        let response = self.app.clone().oneshot(req.body(Body::from(body.to_string())).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    fn bearer(&self, uid: Uuid) -> (&'static str, String) {
        ("authorization", format!("Bearer {}", self.token(uid, CLIENT)))
    }

    /// Anthropic 的 SDK 把密钥放在 x-api-key 里，而不是 Authorization。
    fn api_key(&self, uid: Uuid) -> (&'static str, String) {
        ("x-api-key", self.token(uid, CLIENT))
    }
}

#[tokio::test]
async fn responses_requests_are_translated_and_the_real_model_never_comes_back() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();

    let (status, text) = h.api_call("/v1/responses", &[h.bearer(uid)], json!({ "model": "standard", "input": "hi" })).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(body["model"], "standard");
    assert!(!text.contains(REAL_STANDARD), "{text}");
    assert!(text.contains("keep-me"), "user data inside tool arguments must survive untouched: {text}");

    let seen = h.mock.relayed("/v1/responses");
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].body["model"], REAL_STANDARD, "the tier is swapped for the real model on the way in");
    assert!(seen[0].authorization.starts_with("Bearer sk-mockkey"), "the gateway sees the binding's key, not the user's login token");
    assert!(!seen[0].authorization.contains(&h.token(uid, CLIENT)));
    assert_eq!(h.mock.user_count(), 1, "the first call provisions the account just like chat completions");

    let (status, text) = h.api_call("/v1/responses", &[h.bearer(uid)], json!({ "input": "hi" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(serde_json::from_str::<Value>(&text).unwrap()["model"], "standard", "a missing model falls back to the default tier");
    h.cleanup().await;
}

#[tokio::test]
async fn responses_streams_relabel_the_nested_model_and_keep_user_data() {
    let h = harness!();
    h.seed_wallpaper().await;
    let (status, text) = h.api_call("/v1/responses", &[h.bearer(Uuid::new_v4())], json!({ "model": "standard", "stream": true, "input": "hi" })).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    assert!(!text.contains(REAL_STANDARD), "{text}");
    assert_eq!(text.matches(r#""model":"standard""#).count(), 2, "response.created and response.completed both carry the tier: {text}");
    assert!(text.contains(r#""model":"keep-me""#), "an item's own model field is user data: {text}");
    assert!(text.contains("event: response.completed"), "event lines pass through: {text}");
    h.cleanup().await;
}

#[tokio::test]
async fn messages_accept_x_api_key_forward_only_protocol_headers_and_relabel() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();

    let (status, text) = h
        .api_call(
            "/v1/messages",
            &[h.api_key(uid), ("anthropic-version", "2023-06-01".into()), ("x-forwarded-for", "203.0.113.9".into())],
            json!({ "model": "standard", "max_tokens": 16, "messages": [{ "role": "user", "content": "hi" }] }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{text}");
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(body["model"], "standard", "a dated upstream id like model-20260101 is replaced as a whole");
    assert!(!text.contains(REAL_STANDARD), "{text}");
    assert_eq!(body["content"][0]["input"]["model"], "keep-me");

    let seen = h.mock.relayed("/v1/messages");
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].anthropic_version.as_deref(), Some("2023-06-01"));
    assert_eq!(seen[0].body["model"], REAL_STANDARD);
    for leaked in ["x-api-key", "x-forwarded-for", "cookie"] {
        assert!(!seen[0].header_names.iter().any(|n| n == leaked), "{leaked} must not reach the gateway: {:?}", seen[0].header_names);
    }
    assert!(!seen[0].authorization.contains(&h.token(uid, CLIENT)), "the user's login token must not reach the gateway");

    let (status, text) = h.api_call("/v1/messages", &[h.bearer(uid)], json!({ "model": "standard", "max_tokens": 16, "messages": [] })).await;
    assert_eq!(status, StatusCode::OK, "Authorization: Bearer works for messages too: {text}");
    h.cleanup().await;
}

#[tokio::test]
async fn message_streams_relabel_message_start_and_leave_content_blocks_alone() {
    let h = harness!();
    h.seed_wallpaper().await;
    let (status, text) = h
        .api_call("/v1/messages", &[h.api_key(Uuid::new_v4())], json!({ "model": "standard", "max_tokens": 16, "stream": true, "messages": [] }))
        .await;
    assert_eq!(status, StatusCode::OK, "{text}");
    assert!(!text.contains(REAL_STANDARD), "{text}");
    assert!(text.contains(r#""model":"standard""#), "message_start carries the tier: {text}");
    assert!(text.contains(r#""model":"keep-me""#), "tool input is user data: {text}");
    assert!(text.contains("event: message_start") && text.contains("event: message_stop"), "{text}");
    h.cleanup().await;
}

#[tokio::test]
async fn errors_follow_each_protocol_and_the_paid_tier_gate_applies_everywhere() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();

    let (status, text) = h.api_call("/v1/messages", &[h.api_key(uid)], json!({ "model": "premium", "max_tokens": 16, "messages": [] })).await;
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!((body["type"].as_str(), body["error"]["type"].as_str(), body["error"]["code"].as_str()), (Some("error"), Some("permission_error"), Some("paid_required")), "{text}");

    let (status, text) = h.api_call("/v1/responses", &[h.bearer(uid)], json!({ "model": "premium", "input": "hi" })).await;
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::FORBIDDEN, Some("paid_required")), "{text}");

    let (status, text) = h.api_call("/v1/messages", &[h.api_key(uid)], json!({ "model": REAL_STANDARD, "max_tokens": 16, "messages": [] })).await;
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!((status, body["error"]["type"].as_str(), body["error"]["code"].as_str()), (StatusCode::BAD_REQUEST, Some("invalid_request_error"), Some("model_not_found")), "a real model name is not a tier: {text}");

    let (status, text) = h.api_call("/v1/messages", &[], json!({ "model": "standard", "max_tokens": 16, "messages": [] })).await;
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!((status, body["error"]["type"].as_str()), (StatusCode::UNAUTHORIZED, Some("authentication_error")), "{text}");
    let (status, text) = h.api_call("/v1/messages", &[("x-api-key", "garbage".into())], json!({ "model": "standard", "max_tokens": 16, "messages": [] })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{text}");

    h.mock.0.lock().unwrap().chat_mode = ChatMode::Fail;
    for (path, headers) in [("/v1/messages", vec![h.api_key(uid)]), ("/v1/responses", vec![h.bearer(uid)])] {
        let (status, text) = h.api_call(path, &headers, json!({ "model": "standard", "max_tokens": 16, "messages": [], "input": "hi" })).await;
        assert_eq!(status, StatusCode::BAD_GATEWAY, "{path}");
        assert!(!text.contains(REAL_STANDARD) && !text.contains("channel"), "{path} leaks upstream details: {text}");
    }
    h.cleanup().await;
}

#[tokio::test]
async fn all_three_protocols_draw_on_the_same_balance() {
    let h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();
    h.mock.0.lock().unwrap().chat_cost = 40_000;

    h.chat(uid, "standard").await;
    h.api_call("/v1/responses", &[h.bearer(uid)], json!({ "model": "standard", "input": "hi" })).await;
    h.api_call("/v1/messages", &[h.api_key(uid)], json!({ "model": "standard", "max_tokens": 16, "messages": [] })).await;
    let (user_id, wallet) = h.mock.only_user();
    assert_eq!(wallet, TRIAL - 3 * 40_000);

    let binding_id = h.only_binding().await;
    sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(h.balance(binding_id).await, h.mock.quota_of(user_id), "consumption through any protocol is settled into the buckets");
    h.cleanup().await;
}

// ---- 后台填写的网关令牌 ----

const NEW_TOKEN: &str = "nap_rotated_token_0123456789abcdef";

#[tokio::test]
async fn the_gateway_token_can_be_entered_in_the_console_verified_and_swapped_live() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;

    let (_, view) = h.call(Method::GET, "/api/admin/settings", Some(&admin), Value::Null).await;
    assert_eq!((view["newapi"]["source"].as_str(), view["newapi"]["configured"].as_bool()), (Some("env"), Some(true)));
    assert_eq!(view["newapi"]["hint"], "…oken", "only the last four characters are ever shown");

    // 填错的令牌被网关拒绝：什么都不保存，正在用的令牌继续工作。
    let (status, body) = h.call(Method::PUT, "/api/admin/settings/newapi", Some(&admin), json!({ "admin_token": NEW_TOKEN })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(body["message"].as_str().unwrap().contains("不接受"), "{body}");
    let (stored,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_settings").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(stored, 0, "a rejected token is not saved");
    assert_eq!(h.chat(Uuid::new_v4(), "standard").await.0, StatusCode::OK, "the previous token still works");

    for (bad, why) in [("", "empty"), ("   ", "blank"), ("nap_short", "too short"), ("nap_has space_0123456789", "whitespace inside"), ("令牌令牌令牌令牌令牌令牌令牌令牌", "non-ascii")] {
        let (status, _) = h.call(Method::PUT, "/api/admin/settings/newapi", Some(&admin), json!({ "admin_token": bad })).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{why}");
    }

    // 网关换了令牌后填入新的：保存成功，立刻生效，旧的不再被使用。
    h.mock.accept_admin_token(NEW_TOKEN);
    let (status, body) = h.call(Method::PUT, "/api/admin/settings/newapi", Some(&admin), json!({ "admin_token": format!("  {NEW_TOKEN}\n") })).await;
    assert_eq!((status, body["hint"].as_str()), (StatusCode::OK, Some("…cdef")), "{body}");
    let (status, body) = h.chat(Uuid::new_v4(), "standard").await;
    assert_eq!(status, StatusCode::OK, "provisioning now uses the console token, which is the only one the gateway accepts: {body}");

    let (_, view) = h.call(Method::GET, "/api/admin/settings", Some(&admin), Value::Null).await;
    assert_eq!((view["newapi"]["source"].as_str(), view["newapi"]["hint"].as_str()), (Some("console"), Some("…cdef")));
    assert!(view["newapi"]["updated_at"].is_string());
    assert!(!view.to_string().contains(NEW_TOKEN), "the token itself is never returned");

    let (_, test) = h.call(Method::POST, "/api/admin/settings/newapi/test", Some(&admin), json!({})).await;
    assert_eq!(test["ok"], true, "{test}");
    h.mock.accept_admin_token("rotated-away-again-0123456789");
    let (status, test) = h.call(Method::POST, "/api/admin/settings/newapi/test", Some(&admin), json!({})).await;
    assert_eq!((status, test["ok"].as_bool()), (StatusCode::OK, Some(false)), "a revoked token is reported, not hidden: {test}");
    assert!(test["message"].as_str().unwrap().contains("不接受"));
    h.cleanup().await;
}

#[tokio::test]
async fn the_saved_gateway_token_is_encrypted_audited_and_survives_a_restart() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    h.mock.accept_admin_token(NEW_TOKEN);
    let (status, _) = h.call(Method::PUT, "/api/admin/settings/newapi", Some(&admin), json!({ "admin_token": NEW_TOKEN })).await;
    assert_eq!(status, StatusCode::OK);

    let (raw,): (Vec<u8>,) = sqlx::query_as("SELECT value_enc FROM system_settings WHERE key = 'newapi_admin_token'").fetch_one(&h.state.pool).await.unwrap();
    assert!(!raw.windows(NEW_TOKEN.len()).any(|w| w == NEW_TOKEN.as_bytes()), "the database must not hold the plaintext token");
    assert!(h.state.keys.decrypt("some-other-purpose", &raw).is_err(), "the ciphertext is bound to its purpose");

    // 重启：新进程只有环境变量里的旧令牌，启动时应当用数据库里的覆盖它。
    let restarted = NewApi::new(h.state.newapi.base_url().to_string(), Some(ADMIN_TOKEN.to_string()));
    assert!(restarted.check_current().await.is_err(), "the env token alone is no longer accepted by the gateway");
    let stored = crate::settings::load_newapi_token(&h.state.pool, &h.state.keys).await.unwrap().expect("a saved token");
    restarted.set_admin_token(Some(stored.token));
    restarted.check_current().await.expect("the stored token works after a restart");

    let (_, audit) = h.call(Method::GET, "/api/admin/audit", Some(&admin), Value::Null).await;
    let entry = audit["items"].as_array().unwrap().iter().find(|r| r["action"] == "settings.newapi_token").expect("audit entry");
    assert_eq!(entry["detail"], "令牌尾号 …cdef");
    assert!(!audit.to_string().contains(NEW_TOKEN), "the token must not appear in the audit log");
    h.cleanup().await;
}

#[tokio::test]
async fn only_administrators_can_read_or_change_the_gateway_settings() {
    let h = harness!();
    h.seed_wallpaper().await;
    let user = h.token(Uuid::new_v4(), CLIENT);
    for (method, path) in [(Method::GET, "/api/admin/settings"), (Method::PUT, "/api/admin/settings/newapi"), (Method::POST, "/api/admin/settings/newapi/test")] {
        let (status, _) = h.call(method.clone(), path, None, json!({ "admin_token": NEW_TOKEN })).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path} without credentials");
        let (status, _) = h.call(method.clone(), path, Some(&user), json!({ "admin_token": NEW_TOKEN })).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path} with a user token");
    }
    h.cleanup().await;
}

#[tokio::test]
async fn consumption_shows_up_in_the_overview_series_and_per_product_figures() {
    let h = harness!();
    h.seed_wallpaper().await;
    let admin = h.admin_token().await;
    h.chat(Uuid::new_v4(), "standard").await;
    let binding_id = h.only_binding().await;
    let (user_id, _) = h.mock.only_user();
    h.mock.consume(user_id, 90_000);
    sync::sync_binding(&h.state, binding_id).await.unwrap();

    let (_, overview) = h.call(Method::GET, "/api/admin/overview", Some(&admin), Value::Null).await;
    assert_eq!((overview["consumed_24h"].as_i64(), overview["consumed_total"].as_i64(), overview["active_24h"].as_i64()), (Some(90_000), Some(90_000), Some(1)));
    assert_eq!(overview["balance"], TRIAL - 90_000);
    assert_eq!(overview["series"][13]["consumed"], 90_000, "today's bar carries today's consumption");
    let product = &overview["products"][0];
    assert_eq!((product["consumed_7d"].as_i64(), product["balance"].as_i64()), (Some(90_000), Some(TRIAL - 90_000)));

    // 发放 = 用掉 + 作废/过期 + 还剩：这条恒等式成立，概览页的「去向」条才可信。
    let conserved = overview["consumed_total"].as_i64().unwrap() + overview["lapsed_total"].as_i64().unwrap() + overview["balance"].as_i64().unwrap();
    assert_eq!(conserved, overview["granted_total"].as_i64().unwrap());
    h.cleanup().await;
}

// ---- 联网搜索 ----

/// A stand-in for DeepSeek's Search and Fetch: replies with `results` for any query except "空" (none found).
async fn fake_search() -> String {
    use axum::routing::post;
    let search = |axum::Json(body): axum::Json<Value>| async move {
        if body["query"] == "空" { return axum::Json(json!({ "results": [] })); }
        axum::Json(json!({ "results": [{ "title": "结果", "url": "https://example.com/r", "snippet": "摘要", "date": "2026-10-20", "rerank_score": 2.0, "authority": 3 }] }))
    };
    let fetch = |axum::Json(body): axum::Json<Value>| async move { axum::Json(json!({ "url": body["url"], "title": "页面", "content": "# 正文" })) };
    let app = Router::new().route("/v1/tools/search", post(search)).route("/v1/tools/fetch", post(fetch));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    base
}

#[tokio::test]
async fn web_search_goes_through_new_api_first_and_the_gateway_charges_only_what_it_paid_for() {
    let mut h = harness!();
    h.seed_wallpaper().await;
    let uid = Uuid::new_v4();
    let token = h.token(uid, CLIENT);

    // Default order: New API first, which bills the caller's wallet itself.
    let (status, _) = h.call(Method::POST, "/v1/search", None, json!({ "query": "天气" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, body) = h.call(Method::POST, "/v1/search", Some(&token), json!({ "query": "", "count": 5 })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, body) = h.call(Method::POST, "/v1/search", Some(&token), json!({ "query": "天气", "count": 3 })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({ "results": [{ "title": "经 New API", "url": "https://example.com/n", "snippet": "s", "date": null }] }));
    let (user_id, wallet) = h.mock.only_user();
    let sent = &h.mock.relayed("/v1/tools/search")[0];
    assert_eq!(sent.body, json!({ "query": "天气", "count": 3, "need_content": false }), "forwarded in DeepSeek's format");
    assert!(sent.authorization.starts_with("Bearer sk-"), "with the caller's own New API key");
    assert_eq!(wallet, TRIAL - 375, "New API's charge only; the gateway adds nothing");
    let (status, body) = h.call(Method::POST, "/v1/fetch", Some(&token), json!({ "url": "https://example.com/a" })).await;
    assert_eq!((status, body["title"].as_str()), (StatusCode::OK, Some("经网关的页面")), "{body}");
    assert_eq!(h.mock.quota_of(user_id), TRIAL - 750);

    // New API without a search channel: DeepSeek direct answers, and the gateway charges for it.
    h.mock.0.lock().unwrap().tools_down = true;
    let base = fake_search().await;
    h.state.search = Arc::new(crate::search::Search::grouped(vec![
        vec![crate::search::Provider::NewApi { base: h.state.config.newapi_base_url.clone() }],
        vec![crate::search::Provider::DeepSeek { base, key: "sk-test".into() }],
    ]));
    h.app = build_router(h.state.clone());
    let (status, body) = h.call(Method::POST, "/v1/search", Some(&token), json!({ "query": "天气" })).await;
    assert_eq!((status, body["results"][0]["title"].as_str()), (StatusCode::OK, Some("结果")), "{body}");
    assert!(!body.to_string().contains("deepseek"), "the provider is not named");
    assert_eq!(h.mock.quota_of(user_id), TRIAL - 750 - 500, "a direct search costs AI_SEARCH_COST");
    let (status, body) = h.call(Method::POST, "/v1/search", Some(&token), json!({ "query": "空" })).await;
    assert_eq!((status, body["results"].as_array().map(Vec::len)), (StatusCode::OK, Some(0)));
    assert_eq!(h.mock.quota_of(user_id), TRIAL - 750 - 500, "nothing found, nothing charged");
    let (status, _) = h.call(Method::POST, "/v1/fetch", Some(&token), json!({ "url": "file:///etc/passwd" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let binding_id = h.only_binding().await;
    sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(h.balance(binding_id).await, h.mock.quota_of(user_id), "search use is settled into the buckets like any other use");

    h.state.search = Arc::new(crate::search::Search::grouped(vec![]));
    h.app = build_router(h.state.clone());
    let (status, body) = h.call(Method::POST, "/v1/search", Some(&token), json!({ "query": "天气" })).await;
    assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::SERVICE_UNAVAILABLE, Some("search_unavailable")), "no provider configured: {body}");
    h.cleanup().await;
}

// ---- 图片、语音识别、实时语音 ----

const REAL_IMAGE: &str = "gpt-image-2";
const REAL_TRANSCRIBE: &str = "whisper-3";
const REAL_REALTIME: &str = "qwen-omni-realtime";

impl Harness {
    /// Turns on images, recognition and realtime voice with these upstream models.
    fn with_media(&mut self) {
        let mut config = (*self.state.config).clone();
        config.image_model = Some(REAL_IMAGE.into());
        config.image_sizes = vec!["1024x1024".into(), "1536x1024".into()];
        config.transcribe_model = Some(REAL_TRANSCRIBE.into());
        config.realtime_model = Some(REAL_REALTIME.into());
        self.state.config = Arc::new(config);
        self.app = build_router(self.state.clone());
    }

    async fn upload(&self, token: &str, parts: &[(&str, Option<&str>, &str, Vec<u8>)]) -> (StatusCode, Value) {
        let boundary = "xbboundary";
        let mut body = Vec::new();
        for (name, file, content_type, data) in parts {
            body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"").as_bytes());
            if let Some(file) = file { body.extend_from_slice(format!("; filename=\"{file}\"\r\nContent-Type: {content_type}").as_bytes()); }
            body.extend_from_slice(b"\r\n\r\n");
            body.extend_from_slice(data);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        let request = Request::builder().method(Method::POST).uri("/v1/audio/transcriptions")
            .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::from(body)).unwrap();
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }
}

#[tokio::test]
async fn images_use_the_configured_model_and_return_images_only() {
    let mut h = harness!();
    h.seed_wallpaper().await;
    let token = h.token(Uuid::new_v4(), CLIENT);
    let (status, body) = h.call(Method::POST, "/v1/images/generations", Some(&token), json!({ "prompt": "一只猫" })).await;
    assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::SERVICE_UNAVAILABLE, Some("image_unavailable")), "{body}");

    h.with_media();
    let (status, body) = h.call(Method::POST, "/v1/images/generations", Some(&token), json!({ "prompt": "一只猫", "model": "something-else", "size": "1536x1024" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({ "created": 1, "data": [{ "b64_json": "iVBORw0K", "revised_prompt": "一只猫" }] }));
    let sent = h.mock.relayed("/v1/images/generations");
    assert_eq!(sent[0].body, json!({ "model": REAL_IMAGE, "prompt": "一只猫", "n": 1, "size": "1536x1024" }));
    let (status, _) = h.call(Method::POST, "/v1/images/generations", Some(&token), json!({ "prompt": "x", "size": "8x8" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (user_id, wallet) = h.mock.only_user();
    assert_eq!(wallet, TRIAL - 2_000, "New API charged the wallet");
    let binding_id = h.only_binding().await;
    sync::sync_binding(&h.state, binding_id).await.unwrap();
    assert_eq!(h.balance(binding_id).await, h.mock.quota_of(user_id));
    h.cleanup().await;
}

#[tokio::test]
async fn recordings_are_recognised_by_the_configured_model_even_above_the_usual_body_limit() {
    let mut h = harness!();
    h.seed_wallpaper().await;
    let token = h.token(Uuid::new_v4(), CLIENT);
    let audio = |size: usize| vec![7u8; size];
    let (status, body) = h.upload(&token, &[("file", Some("a.wav"), "audio/wav", audio(10))]).await;
    assert_eq!((status, body["error"]["code"].as_str()), (StatusCode::SERVICE_UNAVAILABLE, Some("transcribe_unavailable")));

    h.with_media();
    let big = 9 * 1024 * 1024;
    let (status, body) = h.upload(&token, &[
        ("model", None, "", b"evil".to_vec()),
        ("language", None, "", b"zh".to_vec()),
        ("file", Some("会议 录音.m4a"), "audio/mp4", audio(big)),
    ]).await;
    assert_eq!((status, &body), (StatusCode::OK, &json!({ "text": "你好，小伴" })));
    let sent = &h.mock.relayed("/v1/audio/transcriptions")[0].body;
    assert_eq!((sent["model"].as_str(), sent["language"].as_str(), sent["file_bytes"].as_u64(), sent["content_type"].as_str()), (Some(REAL_TRANSCRIBE), Some("zh"), Some(big as u64), Some("audio/mp4")));
    assert_eq!(sent["file_name"], "会议录音.m4a", "the file name is cleaned");

    let (status, _) = h.upload(&token, &[("file", Some("x.exe"), "application/x-msdownload", audio(10))]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "only audio");
    let (status, _) = h.upload(&token, &[("language", None, "", b"zh".to_vec())]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "a file is required");
    let (status, _) = h.upload(&token, &[("file", Some("a.wav"), "audio/wav", audio(crate::media::TRANSCRIBE_MAX_FILE + 1))]).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    h.cleanup().await;
}

#[tokio::test]
async fn realtime_relays_events_both_ways_with_the_model_kept_out_of_sight() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
    let mut h = harness!();
    h.seed_wallpaper().await;
    h.with_media();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_router(h.state.clone());
    tokio::spawn(async move { axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap() });
    let token = h.token(Uuid::new_v4(), CLIENT);

    // A browser: the token rides in the subprotocol list.
    let mut request = format!("ws://{addr}/v1/realtime").into_client_request().unwrap();
    request.headers_mut().insert("sec-websocket-protocol", format!("realtime, xiaoban-token.{token}").parse().unwrap());
    let (mut socket, response) = tokio_tungstenite::connect_async(request).await.unwrap();
    assert_eq!(response.headers().get("sec-websocket-protocol").and_then(|v| v.to_str().ok()), Some("realtime"));
    let first = socket.next().await.unwrap().unwrap();
    let created: Value = serde_json::from_str(first.to_text().unwrap()).unwrap();
    assert_eq!((created["type"].as_str(), created["session"]["model"].as_str()), (Some("session.created"), Some("realtime")), "the real model is not shown");
    assert!(!first.to_text().unwrap().contains(REAL_REALTIME));

    socket.send(Message::Text(json!({ "type": "session.update", "session": { "model": "gpt-other", "voice": "alloy" } }).to_string().into())).await.unwrap();
    let echo: Value = serde_json::from_str(socket.next().await.unwrap().unwrap().to_text().unwrap()).unwrap();
    assert_eq!(echo["got"], json!({ "type": "session.update", "session": { "voice": "alloy" } }), "the client cannot pick the model");
    assert_eq!(h.mock.relayed("/v1/realtime")[0].body["model"], REAL_REALTIME);
    socket.close(None).await.unwrap();

    let (status, _) = h.call(Method::GET, "/v1/realtime", Some(&token), json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "a plain GET is not a WebSocket");
    let anonymous = format!("ws://{addr}/v1/realtime").into_client_request().unwrap();
    match tokio_tungstenite::connect_async(anonymous).await {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => assert_eq!(response.status(), StatusCode::UNAUTHORIZED),
        other => panic!("an anonymous socket must be refused: {:?}", other.map(|_| ())),
    }
    h.cleanup().await;
}

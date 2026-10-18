//! 集成测试：需要 TEST_DATABASE_URL 指向可用的 PostgreSQL，每个测试使用独立 schema，结束后删除。
//! 未设置时测试直接跳过。

use crate::{
    admin_auth, build_router, config::AppConfig, db, keys::Keys, password, sender::Sender, state::AppState,
};
use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tower::ServiceExt;
use uuid::Uuid;

const CLIENT: &str = "xiaban-web";
const PASSWORD: &str = "pass-word-123";
static NEXT_USER: AtomicUsize = AtomicUsize::new(1);
const ADMIN_PASSWORD: &str = "admin-pass-123";

struct Harness {
    app: Router,
    state: AppState,
    admin: PgPool,
    schema: String,
}

impl Harness {
    async fn new(overrides: &[(&str, &str)]) -> Option<Self> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let schema = format!("acct_test_{}", Uuid::new_v4().simple());
        let single = std::env::var("TEST_SINGLE_CONNECTION").as_deref() == Ok("1");
        let admin = if single {sqlx::postgres::PgPoolOptions::new().max_connections(1).connect(&url).await.unwrap()} else {db::connect(&url, None).await.unwrap()};
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await.unwrap();
        let pool = if single {admin.clone()} else {db::connect(&url, Some(&schema)).await.unwrap()};
        if single {sqlx::raw_sql(&format!("SET search_path TO {schema}")).execute(&pool).await.unwrap();}

        let overrides: Vec<(String, String)> = overrides.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        let config = AppConfig::from_lookup(|key| {
            if let Some((_, v)) = overrides.iter().find(|(k, _)| k == key) {
                return Some(v.clone());
            }
            match key {
                "ACCOUNT_RESEND_COOLDOWN_SECS" => Some("0".to_string()),
                "ACCOUNT_ADMIN_USER" => Some("Root".to_string()),
                "ACCOUNT_ADMIN_PASSWORD" => Some(ADMIN_PASSWORD.to_string()),
                _ => None,
            }
        });
        db::migrate(&pool).await.unwrap();
        db::seed_clients(&pool, &config.clients).await.unwrap();
        admin_auth::seed(&pool, &config).await.unwrap();

        let state = AppState {
            pool,
            config: Arc::new(config),
            keys: Arc::new(Keys::ephemeral()),
            sender: Arc::new(Sender::capture()),
            dummy_password_hash: Arc::new(password::hash("dummy".into()).await.unwrap()),
            username_limiter: crate::state::username_limiter(),
        };
        // 生产默认只开放邮箱；多数用例要用手机号，所以测试环境先全部开放，专门的用例再恢复默认。
        crate::signup_methods::save(&state, crate::signup_methods::SignupMethods { email: true, phone: true }).await.unwrap();
        Some(Self { app: build_router(state.clone()), state, admin, schema })
    }

    async fn cleanup(self) {
        if std::env::var("TEST_SINGLE_CONNECTION").as_deref() == Ok("1") {
            sqlx::raw_sql(&format!("SET search_path TO public; DROP SCHEMA {} CASCADE; DEALLOCATE ALL",self.schema)).execute(&self.state.pool).await.unwrap();
            self.state.pool.close().await;
            return;
        }
        self.state.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema)).execute(&self.admin).await.unwrap();
    }

    async fn call(&self, method: Method, path: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
        let mut req = Request::builder().method(method).uri(path).header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let response = self.app.clone().oneshot(req.body(Body::from(body.to_string())).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    async fn post(&self, path: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
        self.call(Method::POST, path, token, body).await
    }

    fn last_code(&self) -> String {
        self.state.sender.last_code().expect("no code sent")
    }

    fn sent_count(&self) -> usize {
        self.state.sender.sent_count()
    }

    async fn admin_token(&self) -> String {
        let (status, body) = self
            .post("/api/admin/login", None, json!({ "username": "root", "password": ADMIN_PASSWORD }))
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body["token"].as_str().unwrap().to_string()
    }

    async fn code_login(&self, identifier: &str, client: &str) -> (StatusCode, Value) {
        let (status, _) = self.post("/api/auth/code/send", None, json!({ "identifier": identifier })).await;
        assert_eq!(status, StatusCode::OK);
        self.post("/api/auth/code/login", None, json!({ "identifier": identifier, "code": self.last_code(), "client_id": client }))
            .await
    }

    async fn register(&self, token: &str, username: &str, client: &str) -> (StatusCode, Value) {
        self.post(
            "/api/auth/register",
            None,
            json!({ "registration_token": token, "username": username, "password": PASSWORD, "client_id": client }),
        )
        .await
    }

    /// 验证码登录；新的邮箱 / 手机号会接着完成注册（不指定用户名时自动生成一个）。
    async fn login_as(&self, identifier: &str, client: &str, username: Option<&str>) -> Value {
        let (status, body) = self.code_login(identifier, client).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        if body["registration_required"] != true {
            return body;
        }
        let name = username.map(str::to_string).unwrap_or_else(|| format!("user{:04}", NEXT_USER.fetch_add(1, Ordering::Relaxed)));
        let (status, session) = self.register(body["registration_token"].as_str().unwrap(), &name, client).await;
        assert_eq!(status, StatusCode::OK, "{session}");
        session
    }

    async fn login(&self, identifier: &str) -> Value {
        self.login_as(identifier, CLIENT, None).await
    }
}

macro_rules! harness {
    ($overrides:expr) => {
        match Harness::new($overrides).await {
            Some(h) => h,
            None => {
                eprintln!("TEST_DATABASE_URL 未设置，跳过");
                return;
            }
        }
    };
}

#[tokio::test]
async fn code_login_registers_then_reuses_the_same_user() {
    let h = harness!(&[]);
    let first = h.login("Alice@Example.com").await;
    assert_eq!(first["is_new_user"], true);
    assert_eq!(first["token_type"], "Bearer");
    assert_eq!(first["user"]["identities"][0]["masked"], "a***@example.com");
    assert_eq!(first["user"]["has_password"], true, "every account is created with a password");

    let second = h.login("alice@example.com").await;
    assert_eq!(second["is_new_user"], false);
    assert_eq!(second["user"]["uid"], first["user"]["uid"]);

    let (status, me) = h.call(Method::GET, "/api/me", first["access_token"].as_str(), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["uid"], first["user"]["uid"]);

    let (stored,): (Vec<u8>,) = sqlx::query_as("SELECT identifier_enc FROM identities LIMIT 1")
        .fetch_one(&h.state.pool)
        .await
        .unwrap();
    assert!(!String::from_utf8_lossy(&stored).contains("alice@example.com"), "identifier must not be stored in plaintext");
    h.cleanup().await;
}

#[tokio::test]
async fn me_reveals_verified_email_only_to_allowlisted_clients() {
    let h = harness!(&[]);
    let first = h.login("Alice@Example.com").await;
    let (status, me) = h.call(Method::GET, "/api/me", first["access_token"].as_str(), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["email"], "alice@example.com");

    let (status, _) = h.post("/api/auth/code/send", None, json!({ "identifier": "alice@example.com" })).await;
    assert_eq!(status, StatusCode::OK);
    let (status, other) = h
        .post(
            "/api/auth/code/login",
            None,
            json!({ "identifier": "alice@example.com", "code": h.last_code(), "client_id": "xiangban-wallpaper-web" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{other}");
    let (status, me) = h.call(Method::GET, "/api/me", other["access_token"].as_str(), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert!(me.get("email").is_none(), "{me}");
    h.cleanup().await;
}

#[tokio::test]
async fn wrong_codes_exhaust_attempts_and_lock_out_the_real_code() {
    let h = harness!(&[]);
    h.post("/api/auth/code/send", None, json!({ "identifier": "13800138000" })).await;
    let real = h.last_code();
    let wrong = if real == "000000" { "111111" } else { "000000" };
    for _ in 0..5 {
        let (status, _) = h
            .post("/api/auth/code/login", None, json!({ "identifier": "13800138000", "code": wrong, "client_id": CLIENT }))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    let (status, _) = h
        .post("/api/auth/code/login", None, json!({ "identifier": "13800138000", "code": real, "client_id": CLIENT }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "code must be dead after max attempts");
    h.cleanup().await;
}

#[tokio::test]
async fn code_is_single_use_and_unknown_client_is_rejected() {
    let h = harness!(&[]);
    h.post("/api/auth/code/send", None, json!({ "identifier": "bob@example.com" })).await;
    let code = h.last_code();
    let (status, _) = h
        .post("/api/auth/code/login", None, json!({ "identifier": "bob@example.com", "code": code, "client_id": "nope" }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let body = json!({ "identifier": "bob@example.com", "code": code, "client_id": CLIENT });
    assert_eq!(h.post("/api/auth/code/login", None, body.clone()).await.0, StatusCode::OK);
    assert_eq!(h.post("/api/auth/code/login", None, body).await.0, StatusCode::BAD_REQUEST);
    h.cleanup().await;
}

#[tokio::test]
async fn resend_cooldown_and_daily_limits_apply() {
    let h = harness!(&[("ACCOUNT_RESEND_COOLDOWN_SECS", "60"), ("ACCOUNT_DAILY_PER_IDENTIFIER", "2")]);
    let body = json!({ "identifier": "carol@example.com" });
    assert_eq!(h.post("/api/auth/code/send", None, body.clone()).await.0, StatusCode::OK);
    let (status, err) = h.post("/api/auth/code/send", None, body).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(err["retry_after"].as_i64().unwrap() > 0);
    assert_eq!(h.sent_count(), 1);
    h.cleanup().await;

    let h = harness!(&[("ACCOUNT_DAILY_PER_IDENTIFIER", "2")]);
    let body = json!({ "identifier": "carol@example.com" });
    assert_eq!(h.post("/api/auth/code/send", None, body.clone()).await.0, StatusCode::OK);
    assert_eq!(h.post("/api/auth/code/send", None, body.clone()).await.0, StatusCode::OK);
    assert_eq!(h.post("/api/auth/code/send", None, body).await.0, StatusCode::TOO_MANY_REQUESTS);
    h.cleanup().await;
}

#[tokio::test]
async fn refresh_rotates_and_reuse_revokes_the_whole_family() {
    let h = harness!(&[]);
    let login = h.login("dave@example.com").await;
    let r1 = login["refresh_token"].as_str().unwrap().to_string();

    let (status, second) = h.post("/api/auth/refresh", None, json!({ "refresh_token": r1, "client_id": CLIENT })).await;
    assert_eq!(status, StatusCode::OK);
    let r2 = second["refresh_token"].as_str().unwrap().to_string();
    assert_ne!(r1, r2);

    let (status, _) = h.post("/api/auth/refresh", None, json!({ "refresh_token": r1, "client_id": CLIENT })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "reusing a rotated token must fail");
    let (status, _) = h.post("/api/auth/refresh", None, json!({ "refresh_token": r2, "client_id": CLIENT })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "reuse must revoke the descendant too");

    let fresh = h.login("dave@example.com").await;
    let (status, _) = h
        .post("/api/auth/refresh", None, json!({ "refresh_token": fresh["refresh_token"], "client_id": "xiaban-api" }))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "refresh token is bound to its client");
    h.cleanup().await;
}

#[tokio::test]
async fn logout_revokes_refresh_token() {
    let h = harness!(&[]);
    let login = h.login("erin@example.com").await;
    let refresh = login["refresh_token"].clone();
    assert_eq!(h.post("/api/auth/logout", None, json!({ "refresh_token": refresh })).await.0, StatusCode::OK);
    let (status, _) = h.post("/api/auth/refresh", None, json!({ "refresh_token": refresh, "client_id": CLIENT })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    h.cleanup().await;
}

#[tokio::test]
async fn password_can_be_set_used_and_locks_after_failures() {
    let h = harness!(&[]);
    let login = h.login("frank@example.com").await;
    let token = login["access_token"].as_str().unwrap();

    let (status, _) = h.post("/api/me/password", Some(token), json!({ "current_password": PASSWORD, "new_password": "short" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = h
        .post("/api/me/password", Some(token), json!({ "new_password": "correct horse battery" }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "changing the password requires the current one");
    let (status, _) = h
        .post("/api/me/password", Some(token), json!({ "current_password": PASSWORD, "new_password": "correct horse battery" }))
        .await;
    assert_eq!(status, StatusCode::OK);

    let ok = json!({ "identifier": "frank@example.com", "password": "correct horse battery", "client_id": CLIENT });
    let (status, body) = h.post("/api/auth/password/login", None, ok.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["has_password"], true);

    let bad = json!({ "identifier": "frank@example.com", "password": "wrong password!", "client_id": CLIENT });
    for _ in 0..5 {
        assert_eq!(h.post("/api/auth/password/login", None, bad.clone()).await.0, StatusCode::UNAUTHORIZED);
    }
    assert_eq!(h.post("/api/auth/password/login", None, ok).await.0, StatusCode::TOO_MANY_REQUESTS);

    let unknown = json!({ "identifier": "nobody@example.com", "password": "whatever123", "client_id": CLIENT });
    let (status, body) = h.post("/api/auth/password/login", None, unknown).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["message"], "账号或密码错误");
    h.cleanup().await;
}

#[tokio::test]
async fn reset_password_revokes_sessions_and_does_not_leak_existence() {
    let h = harness!(&[]);
    let login = h.login("gina@example.com").await;

    let before = h.sent_count();
    let (status, _) = h
        .post("/api/auth/code/send", None, json!({ "identifier": "ghost@example.com", "purpose": "reset_password" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(h.sent_count(), before, "no code may be sent for unknown accounts");

    h.post("/api/auth/code/send", None, json!({ "identifier": "gina@example.com", "purpose": "reset_password" })).await;
    let (status, _) = h
        .post(
            "/api/auth/password/reset",
            None,
            json!({ "identifier": "gina@example.com", "code": h.last_code(), "new_password": "brand new password" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = h
        .post("/api/auth/refresh", None, json!({ "refresh_token": login["refresh_token"], "client_id": CLIENT }))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = h
        .post(
            "/api/auth/password/login",
            None,
            json!({ "identifier": "gina@example.com", "password": "brand new password", "client_id": CLIENT }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // 登录用途的验证码不能拿来重置密码
    h.post("/api/auth/code/send", None, json!({ "identifier": "gina@example.com" })).await;
    let (status, _) = h
        .post(
            "/api/auth/password/reset",
            None,
            json!({ "identifier": "gina@example.com", "code": h.last_code(), "new_password": "yet another password" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    h.cleanup().await;
}

#[tokio::test]
async fn bind_phone_to_email_account_and_reject_conflicts() {
    let h = harness!(&[]);
    let hank = h.login("hank@example.com").await;
    let token = hank["access_token"].as_str().unwrap();

    let (status, _) = h.post("/api/me/identities/code", Some(token), json!({ "identifier": "13900139000" })).await;
    assert_eq!(status, StatusCode::OK);
    let (status, me) = h
        .post("/api/me/identities", Some(token), json!({ "identifier": "13900139000", "code": h.last_code() }))
        .await;
    assert_eq!(status, StatusCode::OK, "{me}");
    assert_eq!(me["identities"].as_array().unwrap().len(), 2);

    // 之后用手机号验证码登录得到同一个用户
    let by_phone = h.login("13900139000").await;
    assert_eq!(by_phone["is_new_user"], false);
    assert_eq!(by_phone["user"]["uid"], hank["user"]["uid"]);

    // 另一个用户不能绑定已被占用的手机号；同类型只能绑定一个
    let ivy = h.login("ivy@example.com").await;
    let (status, _) = h
        .post("/api/me/identities/code", ivy["access_token"].as_str(), json!({ "identifier": "13900139000" }))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = h.post("/api/me/identities/code", Some(token), json!({ "identifier": "13700137000" })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    h.cleanup().await;
}

#[tokio::test]
async fn profile_update_validates_input() {
    let h = harness!(&[]);
    let login = h.login("jack@example.com").await;
    let token = login["access_token"].as_str();

    let before = login["user"]["username"].clone();
    let (status, me) = h
        .call(Method::PATCH, "/api/me", token, json!({ "username": "someone_else", "avatar_url": "https://img.example.com/a.png" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["username"], before, "the username can never be changed by the user");
    assert_eq!(me["avatar_url"], "https://img.example.com/a.png");
    assert_eq!(
        h.call(Method::PATCH, "/api/me", token, json!({ "avatar_url": "http://insecure.example/a.png" })).await.0,
        StatusCode::BAD_REQUEST
    );
    let (_, me) = h.call(Method::PATCH, "/api/me", token, json!({ "avatar_url": "" })).await;
    assert_eq!(me["avatar_url"], Value::Null);

    assert_eq!(h.call(Method::GET, "/api/me", None, Value::Null).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(h.call(Method::GET, "/api/me", Some("garbage"), Value::Null).await.0, StatusCode::UNAUTHORIZED);
    h.cleanup().await;
}

#[tokio::test]
async fn disabled_user_cannot_log_in_or_refresh() {
    let h = harness!(&[]);
    let login = h.login("kate@example.com").await;
    sqlx::query("UPDATE users SET status = 'disabled'").execute(&h.state.pool).await.unwrap();

    let (status, _) = h
        .post("/api/auth/refresh", None, json!({ "refresh_token": login["refresh_token"], "client_id": CLIENT }))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    h.post("/api/auth/code/send", None, json!({ "identifier": "kate@example.com" })).await;
    let (status, _) = h
        .post("/api/auth/code/login", None, json!({ "identifier": "kate@example.com", "code": h.last_code(), "client_id": CLIENT }))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    h.cleanup().await;
}

#[tokio::test]
async fn downstream_services_can_verify_tokens_with_published_jwks() {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let h = harness!(&[]);
    let login = h.login("leo@example.com").await;
    let (status, jwks) = h.call(Method::GET, "/.well-known/jwks.json", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let jwk = &jwks["keys"][0];
    assert_eq!(jwk["kty"], "OKP");

    let key = DecodingKey::from_ed_components(jwk["x"].as_str().unwrap()).unwrap();
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.set_audience(&[CLIENT]);
    validation.set_issuer(&[h.state.config.issuer.as_str()]);
    let data = decode::<Value>(login["access_token"].as_str().unwrap(), &key, &validation).unwrap();
    assert_eq!(data.claims["sub"], login["user"]["uid"]);
    assert_eq!(data.header.kid.as_deref(), jwk["kid"].as_str());
    h.cleanup().await;
}

#[tokio::test]
async fn admin_endpoints_are_isolated_from_user_tokens_and_lock_on_bad_passwords() {
    let h = harness!(&[]);
    let user = h.login("mia@example.com").await;

    assert_eq!(h.call(Method::GET, "/api/admin/stats", None, Value::Null).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        h.call(Method::GET, "/api/admin/stats", user["access_token"].as_str(), Value::Null).await.0,
        StatusCode::UNAUTHORIZED,
        "user access tokens must not open the admin API"
    );

    let token = h.admin_token().await;
    assert_eq!(h.call(Method::GET, "/api/me", Some(&token), Value::Null).await.0, StatusCode::UNAUTHORIZED, "admin tokens must not act as user tokens");
    let (status, me) = h.call(Method::GET, "/api/admin/me", Some(&token), Value::Null).await;
    assert_eq!((status, me["username"].as_str()), (StatusCode::OK, Some("root")));

    let bad = json!({ "username": "root", "password": "not the password" });
    for _ in 0..5 {
        assert_eq!(h.post("/api/admin/login", None, bad.clone()).await.0, StatusCode::UNAUTHORIZED);
    }
    let good = json!({ "username": "root", "password": ADMIN_PASSWORD });
    assert_eq!(h.post("/api/admin/login", None, good).await.0, StatusCode::TOO_MANY_REQUESTS);
    let unknown = json!({ "username": "nobody", "password": "whatever-123" });
    assert_eq!(h.post("/api/admin/login", None, unknown).await.0, StatusCode::UNAUTHORIZED);
    h.cleanup().await;
}

#[tokio::test]
async fn admin_password_change_invalidates_existing_admin_tokens() {
    let h = harness!(&[]);
    let old = h.admin_token().await;

    let (status, _) = h
        .post("/api/admin/password", Some(&old), json!({ "current_password": "wrong", "new_password": "a-new-admin-pass" }))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = h
        .post("/api/admin/password", Some(&old), json!({ "current_password": ADMIN_PASSWORD, "new_password": "a-new-admin-pass" }))
        .await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(h.call(Method::GET, "/api/admin/me", Some(&old), Value::Null).await.0, StatusCode::UNAUTHORIZED);
    let (status, _) = h
        .post("/api/admin/login", None, json!({ "username": "root", "password": "a-new-admin-pass" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    h.cleanup().await;
}

#[tokio::test]
async fn admin_stats_and_user_search() {
    let h = harness!(&[]);
    let admin = h.admin_token().await;
    let admin = Some(admin.as_str());
    let nina = h.login_as("nina@example.com", CLIENT, Some("Nina_Red")).await;
    let oscar = h.login_as("13800138000", CLIENT, Some("oscar99")).await;

    let (status, stats) = h.call(Method::GET, "/api/admin/stats", admin, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stats["users"]["total"], 2);
    assert_eq!(stats["users"]["active"], 2);
    assert_eq!(stats["users"]["new_24h"], 2);
    assert_eq!(stats["active_sessions"], 2);
    assert_eq!(stats["codes_24h"]["total"], 2);
    assert_eq!(stats["codes_24h"]["top_ips"][0]["count"], 2);

    let list = |q: &str| {
        let path = format!("/api/admin/users{q}");
        let h = &h;
        async move { h.call(Method::GET, &path, admin, Value::Null).await }
    };
    let (_, all) = list("").await;
    assert_eq!((all["total"].as_i64(), all["items"].as_array().unwrap().len()), (Some(2), 2));

    let (_, by_email) = list("?q=Nina@Example.com").await;
    assert_eq!(by_email["items"][0]["uid"], nina["user"]["uid"]);
    assert_eq!(by_email["items"][0]["identities"][0]["masked"], "n***@example.com");
    let (_, by_phone) = list("?q=13800138000").await;
    assert_eq!((by_phone["total"].as_i64(), by_phone["items"][0]["uid"].clone()), (Some(1), oscar["user"]["uid"].clone()));
    let uid = oscar["user"]["uid"].as_str().unwrap();
    assert_eq!(list(&format!("?q={uid}")).await.1["total"], 1);
    assert_eq!(list("?q=nina_r").await.1["total"], 1, "username substring, case-insensitive");
    assert_eq!(list("?q=_").await.1["total"], 1, "underscore must be escaped, not a wildcard for every row");
    assert_eq!(list("?q=%25").await.1["total"], 0, "percent must be escaped, not a wildcard for every row");
    assert_eq!(list("?q=nobody@example.com").await.1["total"], 0);
    assert_eq!(list("?status=disabled").await.1["total"], 0);
    assert_eq!(list("?status=bogus").await.0, StatusCode::BAD_REQUEST);

    let (_, page) = list("?page=2&page_size=1").await;
    assert_eq!((page["total"].as_i64(), page["items"].as_array().unwrap().len()), (Some(2), 1));
    h.cleanup().await;
}

#[tokio::test]
async fn disabling_a_user_revokes_sessions_and_is_audited() {
    let h = harness!(&[]);
    let admin = h.admin_token().await;
    let admin = Some(admin.as_str());
    let pat = h.login("pat@example.com").await;
    let uid = pat["user"]["uid"].as_str().unwrap();

    let (status, detail) = h.call(Method::GET, &format!("/api/admin/users/{uid}"), admin, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["status"], "active");
    assert_eq!(detail["sessions"][0]["client_id"], CLIENT);

    let (status, _) = h.post(&format!("/api/admin/users/{uid}/status"), admin, json!({ "status": "disabled" })).await;
    assert_eq!(status, StatusCode::OK);
    let refresh = json!({ "refresh_token": pat["refresh_token"], "client_id": CLIENT });
    assert_ne!(h.post("/api/auth/refresh", None, refresh).await.0, StatusCode::OK);
    h.post("/api/auth/code/send", None, json!({ "identifier": "pat@example.com" })).await;
    let login = json!({ "identifier": "pat@example.com", "code": h.last_code(), "client_id": CLIENT });
    assert_eq!(h.post("/api/auth/code/login", None, login).await.0, StatusCode::FORBIDDEN);

    h.post(&format!("/api/admin/users/{uid}/status"), admin, json!({ "status": "active" })).await;
    assert_eq!(h.login("pat@example.com").await["is_new_user"], false);
    let (status, _) = h.post(&format!("/api/admin/users/{uid}/revoke-sessions"), admin, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let (_, detail) = h.call(Method::GET, &format!("/api/admin/users/{uid}"), admin, Value::Null).await;
    assert_eq!(detail["sessions"].as_array().unwrap().len(), 0);

    let missing = Uuid::new_v4();
    assert_eq!(h.post(&format!("/api/admin/users/{missing}/status"), admin, json!({ "status": "disabled" })).await.0, StatusCode::NOT_FOUND);
    assert_eq!(h.post(&format!("/api/admin/users/{uid}/status"), admin, json!({ "status": "banned" })).await.0, StatusCode::BAD_REQUEST);

    let (_, log) = h.call(Method::GET, "/api/admin/audit", admin, Value::Null).await;
    let actions: Vec<&str> = log["items"].as_array().unwrap().iter().map(|i| i["action"].as_str().unwrap()).collect();
    assert_eq!(actions, ["user.revoke_sessions", "user.active", "user.disabled"]);
    assert_eq!(log["items"][0]["admin"], "root");
    h.cleanup().await;
}

#[tokio::test]
async fn admin_can_manage_clients() {
    let h = harness!(&[]);
    let admin = h.admin_token().await;
    let admin = Some(admin.as_str());

    for bad in ["A", "Has Space", "-lead", "x"] {
        assert_eq!(h.post("/api/admin/clients", admin, json!({ "client_id": bad, "name": "n" })).await.0, StatusCode::BAD_REQUEST, "{bad}");
    }
    assert_eq!(h.post("/api/admin/clients", admin, json!({ "client_id": "ok-app", "name": "  " })).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(h.post("/api/admin/clients", admin, json!({ "client_id": "ok-app", "name": "新应用" })).await.0, StatusCode::OK);
    assert_eq!(h.post("/api/admin/clients", admin, json!({ "client_id": "ok-app", "name": "again" })).await.0, StatusCode::CONFLICT);

    let session = h.login_as("quinn@example.com", "ok-app", None).await;

    let (_, clients) = h.call(Method::GET, "/api/admin/clients", admin, Value::Null).await;
    let row = clients["items"].as_array().unwrap().iter().find(|c| c["client_id"] == "ok-app").unwrap();
    assert_eq!((row["active_sessions"].as_i64(), row["enabled"].as_bool()), (Some(1), Some(true)));

    let patch = |body: Value| h.call(Method::PATCH, "/api/admin/clients/ok-app", admin, body);
    assert_eq!(patch(json!({ "enabled": false })).await.0, StatusCode::OK);
    let refresh = json!({ "refresh_token": session["refresh_token"], "client_id": "ok-app" });
    assert_eq!(h.post("/api/auth/refresh", None, refresh).await.0, StatusCode::UNAUTHORIZED, "disabled client cannot refresh");
    h.post("/api/auth/code/send", None, json!({ "identifier": "quinn@example.com" })).await;
    let login = json!({ "identifier": "quinn@example.com", "code": h.last_code(), "client_id": "ok-app" });
    assert_eq!(h.post("/api/auth/code/login", None, login).await.0, StatusCode::BAD_REQUEST, "disabled client cannot log in");

    assert_eq!(patch(json!({ "enabled": true, "name": "改名后" })).await.0, StatusCode::OK);
    let (name,): (String,) = sqlx::query_as("SELECT name FROM clients WHERE client_id = 'ok-app'").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(name, "改名后");
    let missing = h.call(Method::PATCH, "/api/admin/clients/ghost", admin, json!({ "enabled": false })).await;
    assert_eq!(missing.0, StatusCode::NOT_FOUND);
    h.cleanup().await;
}

#[tokio::test]
async fn admin_can_configure_and_read_sender_settings() {
    let h = harness!(&[]);
    let admin = h.admin_token().await;
    let admin = Some(admin.as_str());

    // 1. 读取初始状态（capture）
    let (status, cfg) = h.call(Method::GET, "/api/admin/sender", admin, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cfg["provider"], "capture");

    // 2. 更新为 SMTP
    let update_body = json!({
        "provider": "smtp",
        "smtp": {
            "host": "smtp.example.com",
            "port": 465,
            "secure": true,
            "username": "mailer@example.com",
            "password": "secret-auth-code",
            "from_name": "相伴测试",
            "from_email": "mailer@example.com"
        }
    });
    let (status, resp) = h.call(Method::PUT, "/api/admin/sender", admin, update_body).await;
    assert_eq!(status, StatusCode::OK, "{resp}");
    assert_eq!(resp["ok"], true);

    // 3. 读取更新后的状态，密码应被脱敏（has_password = true）
    let (status, cfg) = h.call(Method::GET, "/api/admin/sender", admin, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cfg["provider"], "smtp");
    assert_eq!(cfg["smtp"]["host"], "smtp.example.com");
    assert_eq!(cfg["smtp"]["has_password"], true);
    assert!(cfg["smtp"].get("password").is_none());

    // 4. 验证审计日志中记录了 sender.update
    let (_, audit) = h.call(Method::GET, "/api/admin/audit?limit=1", admin, Value::Null).await;
    assert_eq!(audit["items"][0]["action"], "sender.update");

    // 5. 测试邮件接口校验
    let (status, _) = h.post("/api/admin/sender/test", admin, json!({ "recipient": "invalid-email" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 6. 切换回 console
    let (status, _) = h.call(Method::PUT, "/api/admin/sender", admin, json!({ "provider": "console" })).await;
    assert_eq!(status, StatusCode::OK);
    let (_, cfg) = h.call(Method::GET, "/api/admin/sender", admin, Value::Null).await;
    assert_eq!(cfg["provider"], "console");

    h.cleanup().await;
}

#[tokio::test]
async fn a_new_identifier_gets_a_registration_token_and_no_account_until_it_registers() {
    let h = harness!(&[]);
    let (status, body) = h.code_login("newbie@example.com", CLIENT).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["registration_required"], true);
    assert!(body.get("access_token").is_none(), "no session before the account exists");
    let token = body["registration_token"].as_str().unwrap();
    let (users,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(users, 0, "verifying the code must not create an account");

    let (status, session) = h.register(token, "Newbie_One", CLIENT).await;
    assert_eq!(status, StatusCode::OK, "{session}");
    assert_eq!(session["is_new_user"], true);
    assert_eq!(session["user"]["username"], "Newbie_One");
    assert_eq!(session["user"]["has_password"], true);
    assert!(session["access_token"].as_str().is_some());

    let again = h.login("newbie@example.com").await;
    assert_eq!(again["is_new_user"], false);
    assert_eq!(again["user"]["uid"], session["user"]["uid"]);
    h.cleanup().await;
}

#[tokio::test]
async fn usernames_are_unique_ignoring_case_and_a_taken_name_keeps_the_token_usable() {
    let h = harness!(&[]);
    h.login_as("first@example.com", CLIENT, Some("Alice_Wonder")).await;

    let (_, body) = h.code_login("second@example.com", CLIENT).await;
    let token = body["registration_token"].as_str().unwrap();
    let (status, err) = h.register(token, "alice_wonder", CLIENT).await;
    assert_eq!((status, err["code"].as_str()), (StatusCode::CONFLICT, Some("username_taken")));
    let (users,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(users, 1, "a failed registration leaves nothing behind");

    let (status, _) = h.register(token, "bob_builder", CLIENT).await;
    assert_eq!(status, StatusCode::OK, "the same token can retry with another name");
    h.cleanup().await;
}

#[tokio::test]
async fn registration_rejects_bad_input_and_replayed_or_forged_tokens() {
    let h = harness!(&[]);
    let (_, body) = h.code_login("rules@example.com", CLIENT).await;
    let token = body["registration_token"].as_str().unwrap();

    for bad in ["ab", "1abcdef", "has space", "a@b.com", "admin_x", "xiaoban2026", "用", "ab1", "用户名用户名用户名用户名"] {
        assert_eq!(h.register(token, bad, CLIENT).await.0, StatusCode::BAD_REQUEST, "{bad}");
    }
    let weak = |password: &str, username: &str| {
        h.post(
            "/api/auth/register",
            None,
            json!({ "registration_token": token, "username": username, "password": password, "client_id": CLIENT }),
        )
    };
    assert_eq!(weak("short", "rules_one").await.0, StatusCode::BAD_REQUEST);
    assert_eq!(weak("Rules_One", "rules_one").await.0, StatusCode::BAD_REQUEST, "password must differ from the username");
    assert_eq!(h.register(token, "rules_one", "nope").await.0, StatusCode::BAD_REQUEST, "unknown client");

    let (status, forged) = h.register("aaa.bbb.ccc", "rules_one", CLIENT).await;
    assert_eq!((status, forged["code"].as_str()), (StatusCode::UNAUTHORIZED, Some("registration_expired")));
    let admin_token = crate::admin_auth::issue(&h.state.keys.sub.admin_jwt_secret, uuid::Uuid::new_v4(), 0, 600).unwrap();
    assert_eq!(h.register(&admin_token, "rules_one", CLIENT).await.0, StatusCode::UNAUTHORIZED, "an admin token is not a registration token");

    assert_eq!(h.register(token, "rules_one", CLIENT).await.0, StatusCode::OK);
    let (status, replay) = h.register(token, "rules_two", CLIENT).await;
    assert_eq!((status, replay["code"].as_str()), (StatusCode::CONFLICT, Some("identity_exists")));
    h.cleanup().await;
}

#[tokio::test]
async fn password_login_accepts_username_email_or_phone_and_hides_which_part_was_wrong() {
    let h = harness!(&[]);
    h.login_as("dora@example.com", CLIENT, Some("Dora_Explorer")).await;
    let phone = h.login_as("13700137000", CLIENT, Some("phone_user")).await;
    assert_eq!(phone["user"]["identities"][0]["kind"], "phone");

    let login = |identifier: &str, password: &str| {
        h.post("/api/auth/password/login", None, json!({ "identifier": identifier, "password": password, "client_id": CLIENT }))
    };
    for identifier in ["Dora_Explorer", "dora_explorer", "dora@example.com", "DORA@example.com"] {
        let (status, body) = login(identifier, PASSWORD).await;
        assert_eq!(status, StatusCode::OK, "{identifier}: {body}");
        assert_eq!(body["user"]["username"], "Dora_Explorer");
        assert_eq!(body["is_new_user"], false);
    }
    assert_eq!(login("phone_user", PASSWORD).await.0, StatusCode::OK);
    assert_eq!(login("13700137000", PASSWORD).await.0, StatusCode::OK);

    let wrong = login("dora_explorer", "wrong-password").await;
    let unknown = login("nobody_here", PASSWORD).await;
    let junk = login("???", PASSWORD).await;
    for (status, body) in [&wrong, &unknown, &junk] {
        assert_eq!(*status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["message"], wrong.1["message"], "same message whatever was wrong");
    }
    h.cleanup().await;
}

#[tokio::test]
async fn a_username_never_receives_codes_and_password_reset_still_works_by_email() {
    let h = harness!(&[]);
    h.login_as("eve@example.com", CLIENT, Some("Eve_Online")).await;
    let (status, _) = h.post("/api/auth/code/send", None, json!({ "identifier": "Eve_Online" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "a username cannot be used to request a code");

    h.post("/api/auth/code/send", None, json!({ "identifier": "eve@example.com", "purpose": "reset_password" })).await;
    let reset = json!({ "identifier": "eve@example.com", "code": h.last_code(), "new_password": "brand-new-pass-9" });
    assert_eq!(h.post("/api/auth/password/reset", None, reset).await.0, StatusCode::OK);
    let login = json!({ "identifier": "Eve_Online", "password": "brand-new-pass-9", "client_id": CLIENT });
    assert_eq!(h.post("/api/auth/password/login", None, login).await.0, StatusCode::OK);
    h.cleanup().await;
}

#[tokio::test]
async fn username_availability_reports_format_taken_and_free_and_is_rate_limited() {
    let h = harness!(&[]);
    h.login_as("fred@example.com", CLIENT, Some("Fred_Flint")).await;
    let check = |name: &str| {
        let path = format!("/api/auth/username/available?username={name}");
        let h = &h;
        async move { h.call(Method::GET, &path, None, Value::Null).await }
    };
    let (status, taken) = check("fred_flint").await;
    assert_eq!((status, taken["available"].as_bool()), (StatusCode::OK, Some(false)));
    let (_, free) = check("barney_rubble").await;
    assert_eq!(free["available"], true);
    let (_, bad) = check("a").await;
    assert_eq!(bad["available"], false);
    assert!(bad["message"].as_str().unwrap().contains("2 个汉字或 3 个英文字母"));
    let (_, reserved) = check("administrator").await;
    assert_eq!(reserved["available"], false);

    let mut limited = None;
    for _ in 0..70 {
        let (status, body) = check("barney_rubble").await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            limited = Some(body);
            break;
        }
    }
    assert!(limited.expect("probing usernames must hit the rate limit")["retry_after"].as_i64().unwrap() > 0);
    h.cleanup().await;
}

#[tokio::test]
async fn me_shows_the_username_and_changing_the_password_rejects_the_username() {
    let h = harness!(&[]);
    let login = h.login_as("gus@example.com", CLIENT, Some("Gus_Gus")).await;
    let token = login["access_token"].as_str();
    let (_, me) = h.call(Method::GET, "/api/me", token, Value::Null).await;
    assert_eq!(me["username"], "Gus_Gus");
    assert!(me.get("nickname").is_none());

    let same = h.post("/api/me/password", token, json!({ "current_password": PASSWORD, "new_password": "gus_gus" })).await;
    assert_eq!(same.0, StatusCode::BAD_REQUEST);
    let wrong_current = h.post("/api/me/password", token, json!({ "current_password": "nope-nope-nope", "new_password": "a-new-password-1" })).await;
    assert_eq!(wrong_current.0, StatusCode::UNAUTHORIZED);
    let ok = h.post("/api/me/password", token, json!({ "current_password": PASSWORD, "new_password": "a-new-password-1" })).await;
    assert_eq!(ok.0, StatusCode::OK);
    h.cleanup().await;
}

#[tokio::test]
async fn only_an_admin_can_rename_a_user_and_it_is_audited() {
    let h = harness!(&[]);
    let admin = h.admin_token().await;
    let admin = Some(admin.as_str());
    let hal = h.login_as("hal@example.com", CLIENT, Some("Hal_9000")).await;
    h.login_as("ian@example.com", CLIENT, Some("Ian_Fleming")).await;
    let uid = hal["user"]["uid"].as_str().unwrap();
    let path = format!("/api/admin/users/{uid}/username");

    assert_eq!(h.post(&path, hal["access_token"].as_str(), json!({ "username": "Hal_Renamed" })).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(h.post(&path, admin, json!({ "username": "ian_fleming" })).await.0, StatusCode::CONFLICT);
    assert_eq!(h.post(&path, admin, json!({ "username": "admin_hal" })).await.0, StatusCode::BAD_REQUEST);
    let (status, body) = h.post(&path, admin, json!({ "username": "Hal_Renamed" })).await;
    assert_eq!((status, body["username"].as_str()), (StatusCode::OK, Some("Hal_Renamed")));
    let login = json!({ "identifier": "hal_renamed", "password": PASSWORD, "client_id": CLIENT });
    assert_eq!(h.post("/api/auth/password/login", None, login).await.0, StatusCode::OK);
    let old = json!({ "identifier": "Hal_9000", "password": PASSWORD, "client_id": CLIENT });
    assert_eq!(h.post("/api/auth/password/login", None, old).await.0, StatusCode::UNAUTHORIZED, "the old name is released");

    let missing = format!("/api/admin/users/{}/username", uuid::Uuid::new_v4());
    assert_eq!(h.post(&missing, admin, json!({ "username": "Ghost_User" })).await.0, StatusCode::NOT_FOUND);
    let (_, audit) = h.call(Method::GET, "/api/admin/audit", admin, Value::Null).await;
    let entry = audit["items"].as_array().unwrap().iter().find(|e| e["action"] == "user.rename").expect("rename is audited");
    assert_eq!(entry["detail"], "Hal_9000 -> Hal_Renamed");
    h.cleanup().await;
}


#[tokio::test]
async fn admin_and_scoped_service_create_real_users_without_fake_verified_identities() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::ERROR)
        .try_init();
    const KEY: &str = "forum-provision-test-key-at-least-32-bytes";
    let h = harness!(&[("ACCOUNT_FORUM_PROVISION_KEY", KEY)]);
    let admin = h.admin_token().await;
    let req = json!({"request_id":Uuid::new_v4(),"username":"EditorAlice","password":PASSWORD,"avatar_url":null});
    let (status, _) = h.post("/api/admin/users", None, req.clone()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, created) = h.post("/api/admin/users", Some(&admin), req.clone()).await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let (_, retry) = h.post("/api/admin/users", Some(&admin), req.clone()).await;
    assert_eq!(created["uid"], retry["uid"]);
    let mut same_name = req.clone();
    same_name["request_id"] = json!(Uuid::new_v4());
    let (status, _) = h.post("/api/admin/users", Some(&admin), same_name).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, login) = h
        .post(
            "/api/auth/password/login",
            None,
            json!({"identifier":"editoralice","password":PASSWORD,"client_id":CLIENT}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{login}");
    let (_, me) = h
        .call(
            Method::GET,
            "/api/me",
            login["access_token"].as_str(),
            Value::Null,
        )
        .await;
    assert_eq!(me["identities"], json!([]));
    let service_req = json!({"request_id":Uuid::new_v4(),"username":"EditorBob","password":PASSWORD,"avatar_url":null});
    let (status, _) = h
        .post(
            "/api/internal/forum/users",
            Some("invalid"),
            service_req.clone(),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, created) = h
        .post("/api/internal/forum/users", Some(KEY), service_req.clone())
        .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let (_, retry) = h
        .post("/api/internal/forum/users", Some(KEY), service_req)
        .await;
    assert_eq!(created["uid"], retry["uid"]);
    h.cleanup().await;
}

#[tokio::test]
async fn account_ids_start_at_13586_count_up_as_strings_and_survive_a_rename() {
    let h = harness!(&[]);
    let admin = h.admin_token().await;
    let admin = Some(admin.as_str());
    let first = h.login_as("ids1@example.com", CLIENT, Some("张三")).await;
    let second = h.login_as("ids2@example.com", CLIENT, Some("Second_One")).await;
    assert_eq!(first["user"]["account_id"], "13586");
    assert_eq!(second["user"]["account_id"], "13587");

    let (_, me) = h.call(Method::GET, "/api/me", first["access_token"].as_str(), Value::Null).await;
    assert_eq!((me["account_id"].clone(), me["username"].clone()), (json!("13586"), json!("张三")));

    let (_, listed) = h.call(Method::GET, "/api/admin/users?q=13587", admin, Value::Null).await;
    assert_eq!((listed["total"].as_i64(), listed["items"][0]["username"].clone()), (Some(1), json!("Second_One")));
    assert_eq!(listed["items"][0]["account_id"], "13587");

    let created = h
        .post(
            "/api/admin/users",
            admin,
            json!({ "request_id": uuid::Uuid::new_v4(), "username": "Made_By_Admin", "password": PASSWORD }),
        )
        .await;
    assert_eq!((created.0, created.1["account_id"].clone()), (StatusCode::OK, json!("13588")));

    let uid = first["user"]["uid"].as_str().unwrap();
    let renamed = h.post(&format!("/api/admin/users/{uid}/username"), admin, json!({ "username": "李四" })).await;
    assert_eq!(renamed.0, StatusCode::OK);
    let (_, detail) = h.call(Method::GET, &format!("/api/admin/users/{uid}"), admin, Value::Null).await;
    assert_eq!(detail["user"]["account_id"], "13586");
    h.cleanup().await;
}

#[tokio::test]
async fn signup_methods_default_to_email_only_and_gate_new_phone_accounts_but_not_existing_ones() {
    let h = harness!(&[]);
    let admin = h.admin_token().await;
    let admin = Some(admin.as_str());
    let existing = h.login_as("13600136000", CLIENT, Some("phone_old")).await;
    let emailer = h.login_as("emailer@example.com", CLIENT, Some("email_only")).await;
    let emailer_token = emailer["access_token"].as_str();

    sqlx::query("DELETE FROM system_settings WHERE key = 'signup_methods'").execute(&h.state.pool).await.unwrap();
    let (status, methods) = h.call(Method::GET, "/api/auth/methods", None, Value::Null).await;
    assert_eq!((status, methods), (StatusCode::OK, json!({ "email": true, "phone": false })));

    let (_, body) = h.post("/api/auth/code/send", None, json!({ "identifier": "13500135000" })).await;
    assert_eq!(body["ok"], true);
    let (status, err) = h
        .post("/api/auth/code/login", None, json!({ "identifier": "13500135000", "code": h.last_code(), "client_id": CLIENT }))
        .await;
    assert_eq!((status, err["code"].as_str()), (StatusCode::FORBIDDEN, Some("signup_method_disabled")));
    let (users,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(&h.state.pool).await.unwrap();
    assert_eq!(users, 2);

    let again = h.login("13600136000").await;
    assert_eq!(again["user"]["uid"], existing["user"]["uid"], "an existing phone account can still sign in by code");
    let (status, _) = h.post("/api/auth/password/login", None, json!({ "identifier": "13600136000", "password": PASSWORD, "client_id": CLIENT })).await;
    assert_eq!(status, StatusCode::OK);

    let (_, email_login) = h.code_login("fresh@example.com", CLIENT).await;
    assert_eq!(email_login["registration_required"], true, "email stays open");

    let (status, err) = h.post("/api/me/identities/code", emailer_token, json!({ "identifier": "13300133000" })).await;
    assert_eq!((status, err["code"].as_str()), (StatusCode::FORBIDDEN, Some("signup_method_disabled")), "binding a closed kind is refused");
    let (status, saved) = h.call(Method::PUT, "/api/admin/signup-methods", admin, json!({ "email": true, "phone": true })).await;
    assert_eq!((status, saved), (StatusCode::OK, json!({ "email": true, "phone": true })));
    assert_eq!(h.call(Method::GET, "/api/auth/methods", None, Value::Null).await.1, json!({ "email": true, "phone": true }));
    let (_, audit) = h.call(Method::GET, "/api/admin/audit", admin, Value::Null).await;
    assert!(audit.to_string().contains("signup_methods.update"));

    assert_eq!(h.call(Method::PUT, "/api/admin/signup-methods", admin, json!({ "email": false, "phone": false })).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(h.call(Method::PUT, "/api/admin/signup-methods", emailer_token, json!({ "email": true, "phone": true })).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(h.call(Method::PUT, "/api/admin/signup-methods", None, json!({ "email": true, "phone": true })).await.0, StatusCode::UNAUTHORIZED);

    h.call(Method::PUT, "/api/admin/signup-methods", admin, json!({ "email": true, "phone": false })).await;
    let (status, body) = h.code_login("13400134000", CLIENT).await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::FORBIDDEN, Some("signup_method_disabled")));
    h.cleanup().await;
}

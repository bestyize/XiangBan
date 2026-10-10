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
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const CLIENT: &str = "xiaban-web";
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
        let admin = db::connect(&url, None).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await.unwrap();
        let pool = db::connect(&url, Some(&schema)).await.unwrap();

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
        };
        Some(Self { app: build_router(state.clone()), state, admin, schema })
    }

    async fn cleanup(self) {
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

    async fn login(&self, identifier: &str) -> Value {
        let (status, _) = self.post("/api/auth/code/send", None, json!({ "identifier": identifier })).await;
        assert_eq!(status, StatusCode::OK);
        let (status, body) = self
            .post("/api/auth/code/login", None, json!({ "identifier": identifier, "code": self.last_code(), "client_id": CLIENT }))
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body
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
    assert_eq!(first["user"]["has_password"], false);

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

    let (status, _) = h.post("/api/me/password", Some(token), json!({ "new_password": "short" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = h.post("/api/me/password", Some(token), json!({ "new_password": "correct horse battery" })).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = h
        .post("/api/me/password", Some(token), json!({ "new_password": "another long password" }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "changing an existing password requires the current one");

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

    let (status, me) = h
        .call(Method::PATCH, "/api/me", token, json!({ "nickname": " 小明 ", "avatar_url": "https://img.example.com/a.png" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["nickname"], "小明");
    assert_eq!(h.call(Method::PATCH, "/api/me", token, json!({ "nickname": "" })).await.0, StatusCode::BAD_REQUEST);
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
    let nina = h.login("nina@example.com").await;
    let oscar = h.login("13800138000").await;
    h.call(Method::PATCH, "/api/me", nina["access_token"].as_str(), json!({ "nickname": "小红_100%" })).await;

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
    assert_eq!(list("?q=%E5%B0%8F%E7%BA%A2").await.1["total"], 1, "nickname substring");
    assert_eq!(list("?q=100%25").await.1["total"], 1, "literal percent sign");
    assert_eq!(list("?q=%25").await.1["total"], 1, "percent must be escaped, not a wildcard for every row");
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

    h.post("/api/auth/code/send", None, json!({ "identifier": "quinn@example.com" })).await;
    let login = json!({ "identifier": "quinn@example.com", "code": h.last_code(), "client_id": "ok-app" });
    let (status, session) = h.post("/api/auth/code/login", None, login).await;
    assert_eq!(status, StatusCode::OK);

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

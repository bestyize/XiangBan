//! Tests against a real PostgreSQL server: the search cache, the admin tables and the admin API.
//!
//! They run only when `TEST_DATABASE_URL` is set, e.g.
//! `TEST_DATABASE_URL=postgres://postgres:password123@localhost:5432/postgres cargo test --test postgres_tests`.
//! Each test works in its own throwaway schema, so existing data is never touched.

use axum::{
    body::Body,
    http::{header, HeaderMap, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{sync::Arc, time::Duration};
use tower::ServiceExt;
use wallpaper_collect::{
    admin::AdminService,
    api::{create_api_router, AppState},
    config::AppConfig,
    db::Database,
    models::{MediaType, SearchQuery},
    providers::ProviderRegistry,
    runtime::{self, CacheSettings, Runtime},
    services::{CachedSearch, SearchCache, WallpaperAggregator, WallpaperCollector},
    store::Store,
};

struct TestDb {
    admin_pool: PgPool,
    pool: PgPool,
    schema: String,
}

impl TestDb {
    async fn connect() -> Option<Self> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("skipped: set TEST_DATABASE_URL to run the PostgreSQL tests");
            return None;
        };
        let schema = format!("wc_test_{}", uuid::Uuid::new_v4().simple());
        let admin_pool = PgPool::connect(&url).await.expect("connect to TEST_DATABASE_URL");
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin_pool)
            .await
            .expect("create test schema");

        let search_path = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(6)
            .after_connect(move |conn, _| {
                let search_path = search_path.clone();
                Box::pin(async move {
                    sqlx::query(&format!("SET search_path TO {search_path}"))
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&url)
            .await
            .expect("connect with test search_path");

        Some(Self { admin_pool, pool, schema })
    }

    async fn store(&self) -> Store {
        Store::init(self.pool.clone()).await.expect("create admin tables")
    }

    async fn cleanup(self) {
        self.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.admin_pool)
            .await
            .expect("drop test schema");
    }
}

fn config() -> AppConfig {
    AppConfig {
        server_host: "127.0.0.1".into(),
        server_port: 0,
        database_url: None,
        unsplash_access_key: None,
        pixabay_api_key: None,
        pexels_api_key: None,
        enable_demo_data: true,
        cache_ttl_secs: 3600,
        auto_sync_interval_mins: 0,
    }
}

async fn configure_cache(store: &Store, runtime: &Runtime, enabled: bool, ttl_secs: u64, max_entries: u64) {
    runtime::save_cache_settings(
        store,
        &CacheSettings {
            enabled,
            ttl_secs,
            max_entries,
        },
    )
    .await
    .unwrap();
    runtime.reload(store).await.unwrap();
}

fn entry(tag: &str) -> CachedSearch {
    CachedSearch {
        items: Vec::new(),
        providers: vec![tag.to_string()],
        has_more: false,
    }
}

fn demo_query(q: &str, page: u32) -> SearchQuery {
    SearchQuery {
        q: Some(q.to_string()),
        category: None,
        media_type: Some(MediaType::Image),
        orientation: None,
        provider: None,
        page: Some(page),
        per_page: Some(10),
        sort: None,
    }
}

/* ---------------------------------- search cache ---------------------------------- */

#[tokio::test]
async fn cache_serves_identical_requests_and_counts_them() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let runtime = Runtime::empty();
    let cache = SearchCache::new(Some(store), runtime);
    assert!(cache.is_active());

    let key = SearchCache::key(&demo_query("space", 1));
    assert!(cache.lookup(&key).await.is_none());
    cache.put(&key, &entry("first")).await;

    let hit = cache.lookup(&key).await.expect("stored entry is served");
    assert_eq!(hit.providers, vec!["first"]);

    // Any difference in the request is a different entry.
    for other in [demo_query("space", 2), demo_query("Space", 1), demo_query("space ", 1)] {
        let other_key = SearchCache::key(&other);
        if other_key != key {
            assert!(cache.lookup(&other_key).await.is_none(), "{other_key}");
        }
    }

    let stats = cache.stats().await.unwrap();
    assert!(stats.available);
    assert_eq!(stats.entries, 1);
    assert!(stats.bytes > 0);
    assert_eq!(stats.hits, 1);

    assert_eq!(cache.clear().await.unwrap(), 1);
    assert!(cache.lookup(&key).await.is_none());
    assert_eq!(cache.stats().await.unwrap().entries, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn cache_evicts_the_least_recently_used_entry() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let runtime = Runtime::empty();
    configure_cache(&store, &runtime, true, 3600, 3).await;
    let cache = SearchCache::new(Some(store.clone()), runtime.clone());

    for key in ["k1", "k2", "k3"] {
        cache.put(key, &entry(key)).await;
    }
    // Using k1 makes k2 the entry that has gone unused the longest.
    assert!(cache.lookup("k1").await.is_some());
    cache.put("k4", &entry("k4")).await;

    assert!(cache.lookup("k2").await.is_none(), "least recently used entry is evicted");
    for key in ["k1", "k3", "k4"] {
        assert!(cache.lookup(key).await.is_some(), "{key} should survive");
    }
    assert_eq!(cache.stats().await.unwrap().entries, 3);

    // Lowering the limit in the console trims at once, again from the least recently used end.
    assert!(cache.lookup("k3").await.is_some());
    configure_cache(&store, &runtime, true, 3600, 1).await;
    cache.enforce_limits().await;
    assert_eq!(cache.stats().await.unwrap().entries, 1);
    assert!(cache.lookup("k3").await.is_some(), "the most recently used entry is the one kept");
    assert!(cache.lookup("k1").await.is_none());
    assert!(cache.lookup("k4").await.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn cache_entries_expire_after_the_configured_lifetime() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let runtime = Runtime::empty();
    configure_cache(&store, &runtime, true, 1, 100).await;
    let cache = SearchCache::new(Some(store.clone()), runtime.clone());

    cache.put("short-lived", &entry("x")).await;
    assert!(cache.lookup("short-lived").await.is_some());

    tokio::time::sleep(Duration::from_millis(1300)).await;
    assert!(cache.lookup("short-lived").await.is_none(), "older than the 1 second lifetime");

    // The lifetime is judged when an entry is read, so a longer setting applies to existing entries too.
    configure_cache(&store, &runtime, true, 3600, 100).await;
    assert!(cache.lookup("short-lived").await.is_some());

    // Expired rows are removed on the next write.
    configure_cache(&store, &runtime, true, 1, 100).await;
    tokio::time::sleep(Duration::from_millis(1300)).await;
    cache.put("fresh", &entry("y")).await;
    assert_eq!(cache.stats().await.unwrap().entries, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn disabled_cache_stores_and_serves_nothing() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let runtime = Runtime::empty();
    configure_cache(&store, &runtime, false, 3600, 100).await;
    let cache = SearchCache::new(Some(store), runtime);

    assert!(!cache.is_active());
    cache.put("k", &entry("x")).await;
    assert!(cache.lookup("k").await.is_none());
    assert_eq!(cache.stats().await.unwrap().entries, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn aggregator_answers_repeat_searches_from_the_cache() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let runtime = Runtime::empty();
    let cache = SearchCache::new(Some(store), runtime.clone());
    let aggregator = Arc::new(WallpaperAggregator::new(
        ProviderRegistry::new(&runtime, true),
        Database::init(None).await,
        cache.clone(),
    ));

    let query = demo_query("space", 1);
    let first = aggregator.search(&query).await.unwrap();
    assert!(!first.items.is_empty());
    assert_ne!(first.providers, vec!["cache"]);

    let second = aggregator.search(&query).await.unwrap();
    assert_eq!(second.providers, vec!["cache"]);
    assert_eq!(second.items.len(), first.items.len());

    let other_page = aggregator.search(&demo_query("space", 2)).await.unwrap();
    assert_ne!(other_page.providers, vec!["cache"]);
    db.cleanup().await;
}

#[tokio::test]
async fn concurrent_identical_searches_reach_the_providers_once() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let runtime = Runtime::empty();
    let cache = SearchCache::new(Some(store), runtime.clone());
    let aggregator = Arc::new(WallpaperAggregator::new(
        ProviderRegistry::new(&runtime, true),
        Database::init(None).await,
        cache.clone(),
    ));

    let query = demo_query("cyberpunk", 1);
    let searches = (0..8).map(|_| {
        let aggregator = aggregator.clone();
        let query = query.clone();
        async move { aggregator.search(&query).await.unwrap() }
    });
    let results = futures::future::join_all(searches).await;
    assert!(results.iter().all(|r| !r.items.is_empty()));

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.misses, 1, "only one request should have gone upstream");
    assert_eq!(stats.hits, 7);
    db.cleanup().await;
}

/* ---------------------------------- stored configuration ---------------------------------- */

#[tokio::test]
async fn env_keys_are_imported_once_and_then_managed_in_the_database() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let mut cfg = config();
    cfg.pexels_api_key = Some("env-pexels-key-1234".into());

    let runtime = Runtime::load(&store, &cfg).await.unwrap();
    assert!(runtime.has_keys("pexels"));
    let keys = store.list_keys().await.unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].provider, "pexels");

    // Deleting it in the console must stick across restarts.
    assert!(store.delete_key("pexels", keys[0].id).await.unwrap());
    let runtime = Runtime::load(&store, &cfg).await.unwrap();
    assert!(!runtime.has_keys("pexels"));
    assert!(store.list_keys().await.unwrap().is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn keys_rotate_and_disabled_keys_are_left_out() {
    let Some(db) = TestDb::connect().await else { return };
    let store = db.store().await;
    let runtime = Runtime::empty();

    let a = store.insert_key("unsplash", "a", "aaaaaaaa-key").await.unwrap();
    let b = store.insert_key("unsplash", "b", "bbbbbbbb-key").await.unwrap();
    store.insert_key("unsplash", "c", "cccccccc-key").await.unwrap();
    assert!(store.update_key("unsplash", b, None, Some(false)).await.unwrap());
    assert!(store.update_key("unsplash", a, Some("renamed"), None).await.unwrap());
    assert!(!store.update_key("pexels", a, Some("wrong provider"), None).await.unwrap());
    runtime.reload(&store).await.unwrap();

    let rows = store.list_keys().await.unwrap();
    assert_eq!(rows.iter().find(|k| k.id == a).unwrap().label, "renamed");
    assert!(rows.iter().find(|k| k.id == a).unwrap().enabled, "label change keeps the enabled flag");
    assert!(!rows.iter().find(|k| k.id == b).unwrap().enabled);
    assert_eq!(store.count_keys("unsplash").await.unwrap(), 3);
    assert!(runtime.has_keys("unsplash"));
    db.cleanup().await;
}

/* ---------------------------------- admin API ---------------------------------- */

async fn build_app(db: &TestDb) -> (Router, AppState) {
    let store = db.store().await;
    let cfg = config();
    let runtime = Runtime::load(&store, &cfg).await.unwrap();
    let registry = ProviderRegistry::new(&runtime, true);
    let database = Database::init(None).await;
    let cache = SearchCache::new(Some(store.clone()), runtime.clone());
    let state = AppState {
        aggregator: Arc::new(WallpaperAggregator::new(registry.clone(), database.clone(), cache.clone())),
        collector: Arc::new(WallpaperCollector::new(registry.clone(), database.clone(), 0)),
        registry,
        db: database,
        runtime,
        cache,
        admin: AdminService::bootstrap(Some(store)).await.unwrap(),
        config: cfg,
    };
    (create_api_router(state.clone()), state)
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Value,
}

async fn call(app: &Router, method: &str, path: &str, cookie: Option<&str>, body: Option<Value>) -> Reply {
    call_from(app, method, path, cookie, None, body).await
}

async fn call_from(app: &Router, method: &str, path: &str, cookie: Option<&str>, origin: Option<&str>, body: Option<Value>) -> Reply {
    let mut request = Request::builder()
        .method(method)
        .uri(format!("/api/v1/admin{path}"))
        .header(header::HOST, "localhost:8081");
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    if let Some(origin) = origin {
        request = request.header(header::ORIGIN, origin);
    }
    let request = match body {
        Some(json) => request
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string()))
            .unwrap(),
        None => request.body(Body::empty()).unwrap(),
    };
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Reply { status, headers, body }
}

async fn sign_in(app: &Router, username: &str, password: &str) -> Reply {
    call(app, "POST", "/auth/login", None, Some(json!({ "username": username, "password": password }))).await
}

fn session_cookie(reply: &Reply) -> String {
    let set_cookie = reply.headers.get(header::SET_COOKIE).expect("login sets a cookie").to_str().unwrap();
    set_cookie.split(';').next().unwrap().to_string()
}

#[tokio::test]
async fn admin_routes_require_a_session_and_login_uses_the_default_account() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, _state) = build_app(&db).await;

    for (method, path) in [("GET", "/providers"), ("GET", "/proxies"), ("GET", "/cache"), ("DELETE", "/cache"), ("GET", "/auth/me")] {
        let reply = call(&app, method, path, None, None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path} must need a session");
    }
    let forged = call(&app, "GET", "/providers", Some("admin_session=not-a-real-token"), None).await;
    assert_eq!(forged.status, StatusCode::UNAUTHORIZED);

    assert_eq!(sign_in(&app, "admin", "wrong").await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(sign_in(&app, "root", "123456").await.status, StatusCode::UNAUTHORIZED);

    let reply = sign_in(&app, "admin", "123456").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["username"], "admin");
    assert_eq!(reply.body["is_default_password"], true);

    let set_cookie = reply.headers.get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(set_cookie.contains("HttpOnly"), "{set_cookie}");
    assert!(set_cookie.contains("SameSite=Strict"), "{set_cookie}");
    assert!(set_cookie.contains("Path=/api/v1/admin"), "{set_cookie}");

    let cookie = session_cookie(&reply);
    assert_eq!(call(&app, "GET", "/auth/me", Some(&cookie), None).await.status, StatusCode::OK);

    let out = call(&app, "POST", "/auth/logout", Some(&cookie), None).await;
    assert_eq!(out.status, StatusCode::OK);
    assert_eq!(call(&app, "GET", "/auth/me", Some(&cookie), None).await.status, StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}

#[tokio::test]
async fn changing_the_password_keeps_this_session_and_ends_the_others() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, _state) = build_app(&db).await;

    let here = session_cookie(&sign_in(&app, "admin", "123456").await);
    let elsewhere = session_cookie(&sign_in(&app, "admin", "123456").await);

    let wrong_current = call(&app, "PUT", "/account", Some(&here), Some(json!({ "current_password": "nope", "new_password": "a-better-password" }))).await;
    assert_eq!(wrong_current.status, StatusCode::BAD_REQUEST);

    for weak in ["123456", "short"] {
        let reply = call(&app, "PUT", "/account", Some(&here), Some(json!({ "current_password": "123456", "new_password": weak }))).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{weak}");
    }
    let bad_name = call(&app, "PUT", "/account", Some(&here), Some(json!({ "current_password": "123456", "new_username": "x" }))).await;
    assert_eq!(bad_name.status, StatusCode::BAD_REQUEST);

    let changed = call(
        &app,
        "PUT",
        "/account",
        Some(&here),
        Some(json!({ "current_password": "123456", "new_username": "boss", "new_password": "a-better-password" })),
    )
    .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.body);
    assert_eq!(changed.body["username"], "boss");
    assert_eq!(changed.body["is_default_password"], false);

    assert_eq!(call(&app, "GET", "/auth/me", Some(&here), None).await.body["username"], "boss");
    assert_eq!(call(&app, "GET", "/auth/me", Some(&elsewhere), None).await.status, StatusCode::UNAUTHORIZED);

    assert_eq!(sign_in(&app, "admin", "123456").await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(sign_in(&app, "boss", "123456").await.status, StatusCode::UNAUTHORIZED);
    let again = sign_in(&app, "boss", "a-better-password").await;
    assert_eq!(again.status, StatusCode::OK);
    assert_eq!(again.body["is_default_password"], false);
    db.cleanup().await;
}

#[tokio::test]
async fn repeated_wrong_passwords_lock_the_login() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, _state) = build_app(&db).await;

    for _ in 0..5 {
        assert_eq!(sign_in(&app, "admin", "guess").await.status, StatusCode::UNAUTHORIZED);
    }
    let locked = sign_in(&app, "admin", "123456").await;
    assert_eq!(locked.status, StatusCode::TOO_MANY_REQUESTS, "even the right password waits out the lock");
    db.cleanup().await;
}

#[tokio::test]
async fn requests_from_another_origin_are_refused() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, _state) = build_app(&db).await;
    let cookie = session_cookie(&sign_in(&app, "admin", "123456").await);

    let cross_site = call_from(&app, "DELETE", "/cache", Some(&cookie), Some("https://evil.example"), None).await;
    assert_eq!(cross_site.status, StatusCode::FORBIDDEN);

    let same_site = call_from(&app, "DELETE", "/cache", Some(&cookie), Some("http://localhost:8081"), None).await;
    assert_eq!(same_site.status, StatusCode::OK);
    db.cleanup().await;
}

#[tokio::test]
async fn provider_keys_are_managed_and_never_returned_in_full() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, state) = build_app(&db).await;
    let cookie = session_cookie(&sign_in(&app, "admin", "123456").await);
    let secret = "pexels-SECRET-0123456789";

    let listed = call(&app, "GET", "/providers", Some(&cookie), None).await;
    let ids: Vec<&str> = listed.body["providers"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec!["unsplash", "pixabay", "pexels"]);

    let added = call(&app, "POST", "/providers/pexels/keys", Some(&cookie), Some(json!({ "label": "main", "key": secret }))).await;
    assert_eq!(added.status, StatusCode::OK, "{}", added.body);
    let second = call(&app, "POST", "/providers/pexels/keys", Some(&cookie), Some(json!({ "label": "spare", "key": "pexels-other-9876543210" }))).await;
    assert_eq!(second.status, StatusCode::OK);
    assert!(state.runtime.has_keys("pexels"), "a new key is live without a restart");

    let listed = call(&app, "GET", "/providers", Some(&cookie), None).await;
    let text = listed.body.to_string();
    assert!(!text.contains(secret), "the full key must never be returned: {text}");
    let pexels = listed.body["providers"][2]["keys"].as_array().unwrap();
    assert_eq!(pexels.len(), 2);
    assert_eq!(pexels[0]["preview"], "pexe…6789");

    // Bad input and unknown providers are rejected.
    for body in [json!({ "label": "x", "key": "short" }), json!({ "label": "  ", "key": secret }), json!({ "label": "x", "key": "has space in it 1234" })] {
        let reply = call(&app, "POST", "/providers/pexels/keys", Some(&cookie), Some(body)).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    }
    let unknown = call(&app, "POST", "/providers/nope/keys", Some(&cookie), Some(json!({ "label": "x", "key": secret }))).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);

    // Disable one, rename the other, then remove both.
    let first_id = pexels[0]["id"].as_i64().unwrap();
    let second_id = pexels[1]["id"].as_i64().unwrap();
    let disable = call(&app, "PATCH", &format!("/providers/pexels/keys/{first_id}"), Some(&cookie), Some(json!({ "enabled": false }))).await;
    assert_eq!(disable.status, StatusCode::OK);
    let rename = call(&app, "PATCH", &format!("/providers/pexels/keys/{second_id}"), Some(&cookie), Some(json!({ "label": "backup" }))).await;
    assert_eq!(rename.status, StatusCode::OK);
    let listed = call(&app, "GET", "/providers", Some(&cookie), None).await;
    assert_eq!(listed.body["providers"][2]["keys"][0]["enabled"], false);
    assert_eq!(listed.body["providers"][2]["keys"][1]["label"], "backup");

    for id in [first_id, second_id] {
        let removed = call(&app, "DELETE", &format!("/providers/pexels/keys/{id}"), Some(&cookie), None).await;
        assert_eq!(removed.status, StatusCode::OK);
    }
    assert!(!state.runtime.has_keys("pexels"));
    let gone = call(&app, "DELETE", &format!("/providers/pexels/keys/{first_id}"), Some(&cookie), None).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);

    let proxy_flag = call(&app, "PUT", "/providers/unsplash/settings", Some(&cookie), Some(json!({ "use_proxy": true }))).await;
    assert_eq!(proxy_flag.status, StatusCode::OK);
    assert!(state.runtime.uses_proxy("unsplash"));
    db.cleanup().await;
}

#[tokio::test]
async fn changing_keys_drops_cached_search_results() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, state) = build_app(&db).await;
    let cookie = session_cookie(&sign_in(&app, "admin", "123456").await);

    state.cache.put("some-search", &entry("old")).await;
    assert_eq!(state.cache.stats().await.unwrap().entries, 1);

    let added = call(&app, "POST", "/providers/pexels/keys", Some(&cookie), Some(json!({ "label": "k", "key": "pexels-key-0123456789" }))).await;
    assert_eq!(added.status, StatusCode::OK);
    assert_eq!(state.cache.stats().await.unwrap().entries, 0, "results built without the new key are stale");
    db.cleanup().await;
}

#[tokio::test]
async fn proxy_pool_is_managed_and_credentials_stay_hidden() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, _state) = build_app(&db).await;
    let cookie = session_cookie(&sign_in(&app, "admin", "123456").await);

    let added = call(&app, "POST", "/proxies", Some(&cookie), Some(json!({ "name": "hk-1", "url": "socks5://alice:hunter2@10.0.0.5:1080" }))).await;
    assert_eq!(added.status, StatusCode::OK, "{}", added.body);
    let id = added.body["id"].as_i64().unwrap();

    for url in ["ftp://host:21", "not a url", "10.0.0.5:1080"] {
        let reply = call(&app, "POST", "/proxies", Some(&cookie), Some(json!({ "name": "bad", "url": url }))).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{url}");
    }

    let listed = call(&app, "GET", "/proxies", Some(&cookie), None).await;
    assert!(!listed.body.to_string().contains("hunter2"), "proxy passwords must not be returned");
    assert_eq!(listed.body["proxies"][0]["url"], "socks5://10.0.0.5:1080");
    assert_eq!(listed.body["proxies"][0]["has_auth"], true);
    assert_eq!(listed.body["strategy"], "round_robin");

    let strategy = call(&app, "PUT", "/proxies/settings", Some(&cookie), Some(json!({ "strategy": "random" }))).await;
    assert_eq!(strategy.status, StatusCode::OK);
    let invalid = call(&app, "PUT", "/proxies/settings", Some(&cookie), Some(json!({ "strategy": "fastest" }))).await;
    assert!(invalid.status.is_client_error());
    assert_eq!(call(&app, "GET", "/proxies", Some(&cookie), None).await.body["strategy"], "random");

    let off = call(&app, "PATCH", &format!("/proxies/{id}"), Some(&cookie), Some(json!({ "enabled": false }))).await;
    assert_eq!(off.status, StatusCode::OK);
    assert_eq!(call(&app, "GET", "/proxies", Some(&cookie), None).await.body["proxies"][0]["enabled"], false);

    assert_eq!(call(&app, "DELETE", &format!("/proxies/{id}"), Some(&cookie), None).await.status, StatusCode::OK);
    assert_eq!(call(&app, "DELETE", &format!("/proxies/{id}"), Some(&cookie), None).await.status, StatusCode::NOT_FOUND);
    assert_eq!(call(&app, "POST", &format!("/proxies/{id}/test"), Some(&cookie), None).await.status, StatusCode::NOT_FOUND);
    db.cleanup().await;
}

#[tokio::test]
async fn cache_settings_are_validated_saved_and_applied() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, state) = build_app(&db).await;
    let cookie = session_cookie(&sign_in(&app, "admin", "123456").await);

    let overview = call(&app, "GET", "/cache", Some(&cookie), None).await;
    assert_eq!(overview.body["settings"]["enabled"], true);
    assert_eq!(overview.body["settings"]["ttl_secs"], 3600);
    assert_eq!(overview.body["settings"]["max_entries"], 1000);
    assert_eq!(overview.body["stats"]["available"], true);

    for bad in [
        json!({ "enabled": true, "ttl_secs": 0, "max_entries": 10 }),
        json!({ "enabled": true, "ttl_secs": 999999999, "max_entries": 10 }),
        json!({ "enabled": true, "ttl_secs": 60, "max_entries": 0 }),
    ] {
        let reply = call(&app, "PUT", "/cache/settings", Some(&cookie), Some(bad.clone())).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{bad}");
    }

    let saved = call(&app, "PUT", "/cache/settings", Some(&cookie), Some(json!({ "enabled": true, "ttl_secs": 120, "max_entries": 2 }))).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    assert_eq!(saved.body["settings"]["ttl_secs"], 120);
    assert_eq!(state.runtime.cache_settings().max_entries, 2);

    // The new limit applies at once to what is already stored.
    for key in ["a", "b", "c"] {
        state.cache.put(key, &entry(key)).await;
    }
    assert_eq!(state.cache.stats().await.unwrap().entries, 2);

    let cleared = call(&app, "DELETE", "/cache", Some(&cookie), None).await;
    assert_eq!(cleared.body["cleared"], 2);
    db.cleanup().await;
}

/// A restart must find the same settings, because they live in the database rather than in memory.
#[tokio::test]
async fn settings_survive_a_restart() {
    let Some(db) = TestDb::connect().await else { return };
    let (app, _state) = build_app(&db).await;
    let cookie = session_cookie(&sign_in(&app, "admin", "123456").await);

    call(&app, "PUT", "/cache/settings", Some(&cookie), Some(json!({ "enabled": false, "ttl_secs": 777, "max_entries": 55 }))).await;
    call(&app, "PUT", "/providers/pixabay/settings", Some(&cookie), Some(json!({ "use_proxy": true }))).await;
    call(&app, "POST", "/providers/pixabay/keys", Some(&cookie), Some(json!({ "label": "kept", "key": "pixabay-key-0123456789" }))).await;
    call(&app, "PUT", "/account", Some(&cookie), Some(json!({ "current_password": "123456", "new_username": "owner", "new_password": "stays-after-restart" }))).await;

    // A second service instance over the same database, as after a restart.
    let (restarted, state) = build_app(&db).await;
    let settings = state.runtime.cache_settings();
    assert_eq!((settings.enabled, settings.ttl_secs, settings.max_entries), (false, 777, 55));
    assert!(state.runtime.uses_proxy("pixabay"));
    assert!(state.runtime.has_keys("pixabay"));

    assert_eq!(sign_in(&restarted, "admin", "123456").await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(sign_in(&restarted, "owner", "stays-after-restart").await.status, StatusCode::OK);
    db.cleanup().await;
}

use axum::{
    body::Body,
    http::{header, HeaderMap, Request, StatusCode},
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Executor,
};
use std::{path::PathBuf, str::FromStr};
use tower::ServiceExt;
use xiaoban_wallpaper_server::{api, auth, probe, protocol, AppState, Config};

struct Fixture {
    state: AppState,
    app: Router,
    admin: sqlx::PgPool,
    schema: String,
}
impl Fixture {
    async fn new() -> Self {
        let url = std::env::var("XIAOBAN_TEST_DATABASE_URL")
            .expect("Integration tests require a dedicated XIAOBAN_TEST_DATABASE_URL");
        let parsed = url::Url::parse(&url).unwrap();
        assert!(
            parsed.path().starts_with("/xiaoban_wallpaper_test"),
            "Only a named xiaoban_wallpaper_test database may run these tests"
        );
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap();
        let schema = format!("xiaoban_test_{}", uuid::Uuid::new_v4().simple());
        admin
            .execute(format!("CREATE SCHEMA {schema}").as_str())
            .await
            .unwrap();
        let search = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(6)
            .after_connect(move |conn, _| {
                let search = search.clone();
                Box::pin(async move {
                    conn.execute(format!("SET search_path TO {search}").as_str())
                        .await?;
                    Ok(())
                })
            })
            .connect_with(PgConnectOptions::from_str(&url).unwrap())
            .await
            .unwrap();
        let state = AppState::new(
            pool,
            Config {
                public_origin: "http://127.0.0.1:55006".into(),
                secure_cookie: false,
                allow_loopback_resources: true,
                frontend_dist: PathBuf::from("missing-test-dist"),
            },
        );
        auth::initialize(&state).await.unwrap();
        Self {
            app: api::router(state.clone()),
            state,
            admin,
            schema,
        }
    }
    async fn close(self) {
        self.state.pool.close().await;
        assert!(self.schema.starts_with("xiaoban_test_"));
        self.admin
            .execute(format!("DROP SCHEMA {} CASCADE", self.schema).as_str())
            .await
            .unwrap();
        self.admin.close().await;
    }
    async fn login(&self) -> (String, String) {
        let (status, data, headers) = call(
            &self.app,
            "POST",
            "/api/admin/login",
            json!({"username":"admin","password":"123456"}),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        (
            headers[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .into(),
            data["csrfToken"].as_str().unwrap().into(),
        )
    }
}
async fn call(
    app: &Router,
    method: &str,
    path: &str,
    body: Value,
    auth: Option<&(String, String)>,
) -> (StatusCode, Value, HeaderMap) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some((cookie, csrf)) = auth {
        request = request
            .header(header::COOKIE, cookie)
            .header("x-csrf-token", csrf);
    }
    let response = app
        .clone()
        .oneshot(
            request
                .body(if method == "GET" {
                    Body::empty()
                } else {
                    Body::from(serde_json::to_vec(&body).unwrap())
                })
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        headers,
    )
}
fn item(id: &str) -> Value {
    json!({"id":id,"version":"1.0.0","title":"测试风景","description":"仅测试元数据","author":"测试","kind":"image","resource":{"url":"https://cdn.example.test/image.png","sha256":"a".repeat(64),"bytes":3,"mime":"image/png"},"assets":{},"controls":[]})
}

#[test]
fn draft_import_and_desktop_protocol_constraints() {
    let items = protocol::parse_import(
        json!({"title":"午后的海","kind":"video","downloadUrl":"https://cdn.example.test/a.mp4"}),
    )
    .unwrap();
    assert_eq!(items.len(), 1);
    assert!(protocol::validate_wallpaper(&items[0], false, false).is_ok());
    assert!(protocol::validate_wallpaper(&items[0], true, false).is_err());
    for id in ["companion", "builtin-x", "local-x", "../secret", ".", ".."] {
        let mut v = item(id);
        let w: protocol::Wallpaper = serde_json::from_value(v.take()).unwrap();
        assert!(protocol::validate_wallpaper(&w, true, false).is_err());
    }
    let mut w: protocol::Wallpaper = serde_json::from_value(item("valid")).unwrap();
    assert!(protocol::validate_wallpaper(&w, true, false).is_ok());
    w.resource.url = "file:///private".into();
    assert!(protocol::validate_wallpaper(&w, false, false).is_err());
    assert!(protocol::parse_import(json!([item("dupe"), item("dupe")])).is_err());
    let mut c = item("animation");
    c["kind"] = json!("canvas");
    c["resource"]["mime"] = json!("application/javascript");
    c["controls"] = json!([{"key":"speed","label":"速度","type":"range","default":4,"min":0,"max":1,"step":0.1}]);
    let w = serde_json::from_value(c).unwrap();
    assert!(protocol::validate_wallpaper(&w, true, false).is_err());
}
#[test]
fn probe_rejects_private_and_transition_networks() {
    for ip in [
        "127.0.0.1",
        "10.0.0.1",
        "169.254.169.254",
        "192.168.1.1",
        "100.64.0.1",
        "0.0.0.0",
        "::1",
        "fc00::1",
        "fe80::1",
        "::ffff:127.0.0.1",
        "2002:7f00:1::1",
    ] {
        assert!(!probe::public_ip(ip.parse().unwrap()), "{ip}");
    }
    for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
        assert!(probe::public_ip(ip.parse().unwrap()));
    }
}
#[tokio::test]
async fn auth_csrf_account_rotation_and_restart() {
    let f = Fixture::new().await;
    assert_eq!(
        call(&f.app, "GET", "/api/admin/wallpapers", Value::Null, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &f.app,
            "POST",
            "/api/admin/login",
            json!({"username":"admin","password":"wrong"}),
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let auth = f.login().await;
    let mut invalid = auth.clone();
    invalid.1 = "invalid".into();
    assert_eq!(
        call(
            &f.app,
            "POST",
            "/api/admin/wallpapers",
            json!({"definition":item("bad")}),
            Some(&invalid)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let req = Request::builder()
        .method("POST")
        .uri("/api/admin/logout")
        .header(header::COOKIE, &auth.0)
        .header("x-csrf-token", &auth.1)
        .header(header::ORIGIN, "https://attacker.invalid")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        f.app.clone().oneshot(req).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let result = call(
        &f.app,
        "PUT",
        "/api/admin/account",
        json!({"username":"owner","currentPassword":"123456","newPassword":""}),
        Some(&auth),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(
        call(&f.app, "GET", "/api/admin/me", Value::Null, Some(&auth))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let (status, data, headers) = call(
        &f.app,
        "POST",
        "/api/admin/login",
        json!({"username":"owner","password":"123456"}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let auth = (
        headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .into(),
        data["csrfToken"].as_str().unwrap().into(),
    );
    let result = call(
        &f.app,
        "PUT",
        "/api/admin/account",
        json!({"username":"owner","currentPassword":"123456","newPassword":"changed-password"}),
        Some(&auth),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(
        call(&f.app, "GET", "/api/admin/me", Value::Null, Some(&auth))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    auth::initialize(&f.state).await.unwrap();
    assert_eq!(
        call(
            &f.app,
            "POST",
            "/api/admin/login",
            json!({"username":"admin","password":"123456"}),
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let result = call(
        &f.app,
        "POST",
        "/api/admin/login",
        json!({"username":"owner","password":"changed-password"}),
        None,
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert!(result.2[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .contains("HttpOnly"));
    let stored: String = sqlx::query_scalar("SELECT password_hash FROM administrators WHERE id=1")
        .fetch_one(&f.state.pool)
        .await
        .unwrap();
    assert!(stored.starts_with("$argon2id$"));
    assert!(!stored.contains("changed-password"));
    f.close().await;
}
#[tokio::test]
async fn import_atomicity_publication_export_and_optimistic_updates() {
    let f = Fixture::new().await;
    let auth = f.login().await;
    let result = call(
        &f.app,
        "POST",
        "/api/admin/wallpapers/import",
        json!({"data":[item("one"),item("two")]}),
        Some(&auth),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    let result = call(
        &f.app,
        "POST",
        "/api/admin/wallpapers/import",
        json!({"data":[item("three"),item("one")]}),
        Some(&auth),
    )
    .await;
    assert_eq!(result.0, StatusCode::CONFLICT);
    let list = call(
        &f.app,
        "GET",
        "/api/admin/wallpapers",
        Value::Null,
        Some(&auth),
    )
    .await
    .1;
    assert_eq!(list["items"].as_array().unwrap().len(), 2);
    assert_eq!(
        call(&f.app, "GET", "/api/catalog", Value::Null, None)
            .await
            .1["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        call(
            &f.app,
            "POST",
            "/api/admin/wallpapers/one/publish",
            json!({"revision":1,"published":true}),
            Some(&auth)
        )
        .await
        .0,
        StatusCode::OK
    );
    let catalog = call(&f.app, "GET", "/api/catalog", Value::Null, None)
        .await
        .1;
    assert_eq!(catalog["schemaVersion"], 1);
    assert_eq!(catalog["items"].as_array().unwrap().len(), 1);
    let imported = protocol::parse_import(catalog).unwrap();
    assert!(protocol::validate_wallpaper(&imported[0], true, false).is_ok());
    assert_eq!(
        call(
            &f.app,
            "PUT",
            "/api/admin/wallpapers/one",
            json!({"definition":item("one"),"revision":1}),
            Some(&auth)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &f.app,
            "DELETE",
            "/api/admin/wallpapers/one",
            json!({"revision":2}),
            Some(&auth)
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&f.app, "GET", "/api/catalog", Value::Null, None)
            .await
            .1["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        call(
            &f.app,
            "POST",
            "/api/admin/uploads",
            json!({"file":"not supported"}),
            Some(&auth)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    f.close().await;
}
#[tokio::test]
async fn publish_limit_is_transactional_under_concurrent_requests() {
    let f = Fixture::new().await;
    let auth = f.login().await;
    for i in 0..99 {
        let id = format!("p-{i}");
        sqlx::query("INSERT INTO wallpapers(id,definition,published) VALUES($1,$2,TRUE)")
            .bind(&id)
            .bind(item(&id))
            .execute(&f.state.pool)
            .await
            .unwrap();
    }
    for id in ["a", "b"] {
        assert_eq!(
            call(
                &f.app,
                "POST",
                "/api/admin/wallpapers",
                json!({"definition":item(id)}),
                Some(&auth)
            )
            .await
            .0,
            StatusCode::OK
        );
    }
    let (a, b) = tokio::join!(
        call(
            &f.app,
            "POST",
            "/api/admin/wallpapers/a/publish",
            json!({"revision":1,"published":true}),
            Some(&auth)
        ),
        call(
            &f.app,
            "POST",
            "/api/admin/wallpapers/b/publish",
            json!({"revision":1,"published":true}),
            Some(&auth)
        )
    );
    assert!(
        a.0 == StatusCode::OK && b.0 == StatusCode::BAD_REQUEST
            || a.0 == StatusCode::BAD_REQUEST && b.0 == StatusCode::OK
    );
    assert_eq!(
        call(&f.app, "GET", "/api/catalog", Value::Null, None)
            .await
            .1["items"]
            .as_array()
            .unwrap()
            .len(),
        100
    );
    f.close().await;
}
#[tokio::test]
async fn streamed_probe_sizes_hashes_limits_and_private_network_guard() {
    let fixture = Router::new().route(
        "/bytes",
        get(|| async { ([(header::CONTENT_TYPE, "image/png")], "streamed-test-bytes") }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });
    let resource = protocol::Resource {
        url: format!("http://{address}/bytes"),
        sha256: String::new(),
        bytes: 0,
        mime: "image/png".into(),
    };
    assert!(probe::inspect(&resource, 1024, false).await.is_err());
    assert!(probe::inspect(&resource, 2, true).await.is_err());
    let observed = probe::inspect(&resource, 1024, true).await.unwrap();
    assert_eq!(observed.bytes, 19);
    assert_eq!(observed.sha256, auth::digest("streamed-test-bytes"));
    let f = Fixture::new().await;
    let auth = f.login().await;
    let mut definition = item("probe");
    definition["resource"] = serde_json::to_value(resource).unwrap();
    assert_eq!(
        call(
            &f.app,
            "POST",
            "/api/admin/wallpapers",
            json!({"definition":definition}),
            Some(&auth)
        )
        .await
        .0,
        StatusCode::OK
    );
    let result = call(
        &f.app,
        "POST",
        "/api/admin/wallpapers/probe/verify",
        json!({"revision":1}),
        Some(&auth),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(
        result.1["definition"]["resource"]["sha256"],
        observed.sha256
    );
    assert!(result.1["verifiedAt"].is_string());
    f.close().await;
    server.abort();
}
#[tokio::test]
async fn release_publication_and_semver_update_responses() {
    let f = Fixture::new().await;
    let auth = f.login().await;
    assert_eq!(
        call(
            &f.app,
            "GET",
            "/api/updates?currentVersion=0.1.0&target=windows-x86_64",
            Value::Null,
            None
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    let definition = json!({"schemaVersion":1,"version":"0.2.0","notes":"新版本","platforms":{"windows-x86_64":{"url":"https://cdn.example.test/app.zip","sha256":"a".repeat(64),"bytes":100,"mime":"application/zip"}}});
    assert_eq!(
        call(
            &f.app,
            "PUT",
            "/api/admin/release",
            json!({"definition":definition,"revision":1}),
            Some(&auth)
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &f.app,
            "POST",
            "/api/admin/release/publish",
            json!({"revision":2}),
            Some(&auth)
        )
        .await
        .0,
        StatusCode::OK
    );
    let result = call(
        &f.app,
        "GET",
        "/api/updates?currentVersion=0.1.0&target=windows-x86_64",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["version"], "0.2.0");
    assert_eq!(
        call(
            &f.app,
            "GET",
            "/api/updates?currentVersion=0.2.0&target=windows-x86_64",
            Value::Null,
            None
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        call(&f.app, "GET", "/api/site", Value::Null, None).await.1["release"]["version"],
        "0.2.0"
    );
    f.close().await;
}

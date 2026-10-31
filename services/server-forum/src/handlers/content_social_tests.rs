use super::*;
use crate::{auth::JwksVerifier, config::AppConfig};
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
struct Harness {
    pool: PgPool,
    root: PgPool,
    schema: String,
    app: Router,
    admin: String,
    key: EncodingKey,
    state: AppState,
}
async fn call(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
impl Harness {
    async fn new() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("skipped: TEST_DATABASE_URL is not set");
            return Ok(None);
        };
        let root = crate::db::connect(&url, None).await?;
        let schema = format!("content_social_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?;
        social::schema::init(&pool).await?;
        social::schema::init(&pool).await?;
        let mut config = AppConfig::from_env();
        config.admin_user = Some("editor".into());
        config.admin_password = Some("test-community-editorial".into());
        crate::handlers::admin::seed_admin(&pool, &config).await?;
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let public = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
        let verifier = JwksVerifier::with_static_key(
            "test-editorial".into(),
            "test-editorial".into(),
            DecodingKey::from_ed_components(&public).unwrap(),
        );
        let key = EncodingKey::from_ed_der(pkcs8.as_ref());
        let state = AppState {
            pool: pool.clone(),
            config: Arc::new(config),
            verifier: Arc::new(verifier),
            http: reqwest::Client::new(),
        };
        let saved_state = state.clone();
        let app = crate::build_router(state.clone()).merge(
            Router::new()
                .nest(
                    "/api",
                    routes().nest("/admin/content-social", admin_routes()),
                )
                .with_state(state),
        );
        let (status, login) = call(
            &app,
            "POST",
            "/api/admin/login",
            None,
            json!({"username":"editor","password":"test-community-editorial"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{login}");
        Ok(Some(Self {
            pool,
            root,
            schema,
            app,
            admin: login["token"].as_str().unwrap().into(),
            key,
            state: saved_state,
        }))
    }
    async fn user(&self) -> (Uuid, String) {
        let uid = Uuid::new_v4();
        sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'作者',1)")
            .bind(uid)
            .execute(&self.pool)
            .await
            .unwrap();
        let token=encode(&Header::new(Algorithm::EdDSA),&json!({"iss":"test-editorial","aud":"test-editorial","sub":uid.to_string(),"exp":chrono::Utc::now().timestamp()+600}),&self.key).unwrap();
        (uid, token)
    }
    async fn pack(&self, uid: Uuid, title: &str, status: &str) -> i64 {
        let (id,):(i64,)=sqlx::query_as("INSERT INTO packs(kind,slug,title,author_uid,status) VALUES('effect',$1,$2,$3,$4) RETURNING id").bind(Uuid::new_v4().simple().to_string()).bind(title).bind(uid).bind(status).fetch_one(&self.pool).await.unwrap();
        id
    }
    async fn close(self) -> Result<(), sqlx::Error> {
        self.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.root)
            .await?;
        self.root.close().await;
        Ok(())
    }
}

#[tokio::test]
async fn social_bookmarks_are_private_idempotent_and_hide_unreadable_targets(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (author, _) = h.user().await;
    let (_, user) = h.user().await;
    let (_, other) = h.user().await;
    let pack = h.pack(author, "作品", "published").await;
    let (scheme,): (i64,) = sqlx::query_as(
        "INSERT INTO schemes(author_uid,title,items) VALUES($1,'公开方案',$2) RETURNING id",
    )
    .bind(author)
    .bind(json!([{"pack_id":pack,"version":1}]))
    .fetch_one(&h.pool)
    .await?;
    sqlx::query("INSERT INTO pack_versions(pack_id,version,label,sha256,size,manifest,file_key) VALUES($1,1,'v1',$2,1,'{}','not-returned')").bind(pack).bind("a".repeat(64)).execute(&h.pool).await?;
    let safe = crate::handlers::content::public_content_card(&h.state, "pack", pack, Some(1))
        .await?
        .unwrap();
    assert!(safe.get("versions").is_none());
    assert!(safe.get("manifest").is_none());
    assert!(
        crate::handlers::content::public_content_card(&h.state, "pack", pack, Some(99))
            .await?
            .is_none()
    );
    for (kind, id) in [("pack", pack), ("scheme", scheme)] {
        let path = format!("/api/content-social/{kind}/{id}/bookmark");
        assert_eq!(
            call(&h.app, "PUT", &path, None, Value::Null).await.0,
            StatusCode::UNAUTHORIZED
        );
        for _ in 0..2 {
            assert_eq!(
                call(&h.app, "PUT", &path, Some(&user), Value::Null).await.0,
                StatusCode::OK
            );
        }
    }
    let (_, list) = call(
        &h.app,
        "GET",
        "/api/me/content-bookmarks",
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(list["total"], 2);
    assert_eq!(list["items"].as_array().unwrap().len(), 2);
    let (_, empty) = call(
        &h.app,
        "GET",
        "/api/me/content-bookmarks",
        Some(&other),
        Value::Null,
    )
    .await;
    assert_eq!(empty["total"], 0);
    // A different account's DELETE cannot remove this user's bookmark.
    call(
        &h.app,
        "DELETE",
        &format!("/api/content-social/pack/{pack}/bookmark"),
        Some(&other),
        Value::Null,
    )
    .await;
    let (_, state) = call(
        &h.app,
        "GET",
        &format!("/api/content-social/pack/{pack}"),
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(state, json!({"bookmarked":true}));
    sqlx::query("UPDATE packs SET status='removed' WHERE id=$1")
        .bind(pack)
        .execute(&h.pool)
        .await?;
    let (_, list) = call(
        &h.app,
        "GET",
        "/api/me/content-bookmarks",
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(list["total"], 1);
    assert_eq!(list["items"][0]["content"]["items"][0]["available"], false);
    assert!(list["items"][0]["content"]["items"][0]
        .get("title")
        .is_none());
    sqlx::query("UPDATE packs SET status='published' WHERE id=$1")
        .bind(pack)
        .execute(&h.pool)
        .await?;
    let (category,): (i32,) =
        sqlx::query_as("INSERT INTO categories(slug,name) VALUES('social','讨论') RETURNING id")
            .fetch_one(&h.pool)
            .await?;
    let (topic,):(i64,)=sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html,min_read_level) VALUES($1,$2,'私有讨论','正文','正文',1) RETURNING id").bind(category).bind(author).fetch_one(&h.pool).await?;
    sqlx::query("UPDATE packs SET topic_id=$2 WHERE id=$1")
        .bind(pack)
        .bind(topic)
        .execute(&h.pool)
        .await?;
    let (_, list) = call(
        &h.app,
        "GET",
        "/api/me/content-bookmarks?kind=pack",
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(list["total"], 0);
    assert_eq!(
        call(
            &h.app,
            "PUT",
            &format!("/api/content-social/pack/{pack}/bookmark"),
            Some(&other),
            Value::Null
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    sqlx::query("UPDATE users SET status='banned' WHERE uid=$1")
        .bind(author)
        .execute(&h.pool)
        .await?;
    let (_, list) = call(
        &h.app,
        "GET",
        "/api/me/content-bookmarks",
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(list["total"], 0);
    assert_eq!(
        call(
            &h.app,
            "DELETE",
            &format!("/api/content-social/pack/{pack}/bookmark"),
            Some(&user),
            Value::Null
        )
        .await
        .0,
        StatusCode::OK
    );
    h.close().await?;
    Ok(())
}
#[tokio::test]
async fn social_reports_admin_closure_revision_and_private_reasons(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (author, author_token) = h.user().await;
    let (_, user) = h.user().await;
    let (_, other) = h.user().await;
    let pack = h.pack(author, "举报作品", "published").await;
    let path = format!("/api/content-social/pack/{pack}/report");
    let report = json!({"reason":"copyright","detail":"举报人的私人证据"});
    for _ in 0..2 {
        assert_eq!(
            call(&h.app, "POST", &path, Some(&user), report.clone())
                .await
                .0,
            StatusCode::OK
        );
    }
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/admin/content-social/reports",
            Some(&user),
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/admin/content-social/reports",
            None,
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&h.app, "GET", &path, Some(&author_token), Value::Null)
            .await
            .0,
        StatusCode::METHOD_NOT_ALLOWED
    );
    let (_, queue) = call(
        &h.app,
        "GET",
        "/api/admin/content-social/reports",
        Some(&h.admin),
        Value::Null,
    )
    .await;
    assert_eq!(queue["total"], 1);
    let item = &queue["items"][0];
    let id = item["id"].as_i64().unwrap();
    let old = item["revision"].as_i64().unwrap();
    call(&h.app, "POST", &path, Some(&user), report.clone()).await;
    let handle = format!("/api/admin/content-social/reports/{id}/handle");
    let body = json!({"expected_revision":old,"action":"remove","note":"管理员私密判断"});
    assert_eq!(
        call(&h.app, "POST", &handle, Some(&user), body.clone())
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&h.app, "POST", &handle, Some(&h.admin), body).await.0,
        StatusCode::CONFLICT
    );
    call(&h.app, "POST", &path, Some(&other), report).await;
    let (_, queue) = call(
        &h.app,
        "GET",
        "/api/admin/content-social/reports",
        Some(&h.admin),
        Value::Null,
    )
    .await;
    assert_eq!(queue["total"], 2);
    let item = queue["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap();
    let body =
        json!({"expected_revision":item["revision"],"action":"remove","note":"管理员私密判断"});
    let (status, result) = call(&h.app, "POST", &handle, Some(&h.admin), body.clone()).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["reports_handled"], 2);
    assert_eq!(result["target_removed"], true);
    assert_eq!(
        call(&h.app, "POST", &handle, Some(&h.admin), body).await.0,
        StatusCode::CONFLICT
    );
    let (status,): (String,) = sqlx::query_as("SELECT status FROM packs WHERE id=$1")
        .bind(pack)
        .fetch_one(&h.pool)
        .await?;
    assert_eq!(status, "removed");
    let (_, notifications) = call(
        &h.app,
        "GET",
        "/api/me/notifications",
        Some(&author_token),
        Value::Null,
    )
    .await;
    let serialized = notifications.to_string();
    assert!(!serialized.contains("私人证据"));
    assert!(!serialized.contains("私密判断"));
    assert!(serialized.contains("content_removed"), "{notifications}");
    let (logs,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM admin_audit WHERE action='content.report.remove'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(logs, 1);
    assert_eq!(
        call(
            &h.app,
            "POST",
            &path,
            Some(&user),
            json!({"reason":"other"})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    // Scheme reports are dismissed without changing publication; subsequent fresh reports are permitted.
    let (scheme,): (i64,) =
        sqlx::query_as("INSERT INTO schemes(author_uid,title) VALUES($1,'方案') RETURNING id")
            .bind(author)
            .fetch_one(&h.pool)
            .await?;
    let path = format!("/api/content-social/scheme/{scheme}/report");
    call(&h.app, "POST", &path, Some(&user), json!({"reason":"spam"})).await;
    let (_, queue) = call(
        &h.app,
        "GET",
        "/api/admin/content-social/reports",
        Some(&h.admin),
        Value::Null,
    )
    .await;
    let item = &queue["items"][0];
    let path = format!("/api/admin/content-social/reports/{}/handle", item["id"]);
    let (status, _) = call(
        &h.app,
        "POST",
        &path,
        Some(&h.admin),
        json!({"expected_revision":item["revision"],"action":"dismiss","note":"没有发现违规"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status,): (String,) = sqlx::query_as("SELECT status FROM schemes WHERE id=$1")
        .bind(scheme)
        .fetch_one(&h.pool)
        .await?;
    assert_eq!(status, "published");
    sqlx::query("UPDATE users SET status='banned' WHERE uid=$1")
        .bind(author)
        .execute(&h.pool)
        .await?;
    assert_eq!(
        call(
            &h.app,
            "POST",
            &format!("/api/content-social/scheme/{scheme}/report"),
            Some(&user),
            json!({"reason":"other"})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/me/content-bookmarks",
            Some(&author_token),
            Value::Null
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    h.close().await?;
    Ok(())
}

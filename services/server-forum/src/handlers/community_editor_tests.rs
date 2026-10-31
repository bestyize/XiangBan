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
        let schema = format!("editorial_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?;
        editorial::schema::init(&pool).await?;
        editorial::schema::init(&pool).await?;
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
        let app = crate::build_router(state);
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
async fn editorial_featured_requires_admin_and_hides_unavailable_packs_immediately(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (uid, user) = h.user().await;
    let first = h.pack(uid, "First", "published").await;
    let second = h.pack(uid, "Second", "published").await;
    let removed = h.pack(uid, "Removed", "removed").await;
    for token in [None, Some(user.as_str())] {
        let (status, _) = call(
            &h.app,
            "GET",
            "/api/admin/community-editorial/featured",
            token,
            Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (status, data) = call(
        &h.app,
        "GET",
        "/api/admin/community-editorial/featured",
        Some(&h.admin),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(data["items"], json!([]));
    let invalid = json!({"expected_revision":1,"items":[{"pack_id":removed,"sort_order":0}]});
    let (status, _) = call(
        &h.app,
        "PUT",
        "/api/admin/community-editorial/featured",
        Some(&h.admin),
        invalid,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let input = json!({"expected_revision":1,"items":[{"pack_id":first,"sort_order":1},{"pack_id":second,"sort_order":0}]});
    let (status, data) = call(
        &h.app,
        "PUT",
        "/api/admin/community-editorial/featured",
        Some(&h.admin),
        input.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{data}");
    assert_eq!(
        editorial::featured_ids(&h.pool, 8).await?,
        vec![second, first]
    );
    let cards = editorial::featured(&h.pool, 8).await?;
    assert_eq!(cards[0]["title"], "Second");
    assert_eq!(cards[0]["one_click"], true);
    let (status, _) = call(
        &h.app,
        "PUT",
        "/api/admin/community-editorial/featured",
        Some(&h.admin),
        input,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    sqlx::query("UPDATE packs SET status='removed' WHERE id=$1")
        .bind(first)
        .execute(&h.pool)
        .await?;
    assert_eq!(editorial::featured_ids(&h.pool, 8).await?, vec![second]);
    sqlx::query("UPDATE users SET status='banned' WHERE uid=$1")
        .bind(uid)
        .execute(&h.pool)
        .await?;
    assert!(editorial::featured(&h.pool, 8).await?.is_empty());
    let admin = editorial::featured_admin(&h.pool).await?;
    assert!(admin["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|i| i["visible"] == false));
    let (logs,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM admin_audit WHERE action='community.featured.replace'",
    )
    .fetch_one(&h.pool)
    .await?;
    assert_eq!(logs, 1);
    h.close().await?;
    Ok(())
}

fn challenge_input() -> Value {
    json!({"title":"桌面小工具挑战","description":"分享你自己创作的公开作品。投票使用作品讨论帖点赞，不发积分。","starts_at":chrono::Utc::now()-chrono::Duration::hours(1),"ends_at":chrono::Utc::now()+chrono::Duration::days(7),"submission_tag":"desktop-challenge","topic_id":null,"expected_revision":0})
}
async fn create_challenge(h: &Harness, key: &str) -> Value {
    let (status, value) = call(
        &h.app,
        "PUT",
        &format!("/api/admin/community-editorial/challenges/{key}"),
        Some(&h.admin),
        challenge_input(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}
async fn publish(h: &Harness, key: &str, revision: i64) -> Value {
    let (status, value) = call(
        &h.app,
        "POST",
        &format!("/api/admin/community-editorial/challenges/{key}/publish"),
        Some(&h.admin),
        json!({"expected_revision":revision}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}
#[tokio::test]
async fn editorial_challenge_manual_publication_ownership_idempotency_and_deadline_are_enforced(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (uid, user) = h.user().await;
    let (other_uid, other_user) = h.user().await;
    let own = h.pack(uid, "我的作品", "published").await;
    let other = h.pack(other_uid, "其他人的作品", "published").await;
    let removed = h.pack(uid, "下架作品", "removed").await;
    let data = create_challenge(&h, "desktop-tools").await;
    let revision = data["challenge"]["revision"].as_i64().unwrap();
    let (status, _) = call(
        &h.app,
        "GET",
        "/api/challenges/desktop-tools",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, list) = call(&h.app, "GET", "/api/challenges", None, Value::Null).await;
    assert_eq!(list["total"], 0);
    let (status, _) = call(
        &h.app,
        "POST",
        "/api/admin/community-editorial/challenges/desktop-tools/publish",
        Some(&user),
        json!({"expected_revision":revision}),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let published = publish(&h, "desktop-tools", revision).await;
    assert_eq!(published["challenge"]["status"], "open");
    let (status, _) = call(
        &h.app,
        "PUT",
        "/api/admin/community-editorial/challenges/desktop-tools",
        Some(&h.admin),
        challenge_input(),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    for _ in 0..2 {
        let (status, value) = call(
            &h.app,
            "POST",
            "/api/challenges/desktop-tools/entries",
            Some(&user),
            json!({"pack_id":own}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
    }
    for (token, pack_id, status) in [
        (user.as_str(), other, StatusCode::FORBIDDEN),
        (other_user.as_str(), own, StatusCode::FORBIDDEN),
        (user.as_str(), removed, StatusCode::BAD_REQUEST),
    ] {
        let (actual, value) = call(
            &h.app,
            "POST",
            "/api/challenges/desktop-tools/entries",
            Some(token),
            json!({"pack_id":pack_id}),
        )
        .await;
        assert_eq!(actual, status, "{value}");
    }
    let (_, detail) = call(
        &h.app,
        "GET",
        "/api/challenges/desktop-tools",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(detail["entries"]["total"], 1);
    assert_eq!(detail["challenge"]["entry_count"], 1);
    assert!(detail["entries"]["items"][0]["vote_topic_id"].is_null());
    assert!(detail["entries"]["items"][0]["likes"].is_null());
    let (points,): (i32,) = sqlx::query_as("SELECT points FROM users WHERE uid=$1")
        .bind(uid)
        .fetch_one(&h.pool)
        .await?;
    assert_eq!(points, 0, "challenge submission must not award points");
    sqlx::query("UPDATE community_challenges SET starts_at=clock_timestamp()-interval '1 day',ends_at=clock_timestamp() WHERE key='desktop-tools'").execute(&h.pool).await?;
    let (status, _) = call(
        &h.app,
        "POST",
        "/api/challenges/desktop-tools/entries",
        Some(&user),
        json!({"pack_id":own}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "even retries after the exact deadline are rejected"
    );
    let (_, detail) = call(
        &h.app,
        "GET",
        "/api/challenges/desktop-tools",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(detail["challenge"]["status"], "ended");
    h.close().await?;
    Ok(())
}
#[tokio::test]
async fn editorial_challenge_feature_audit_and_dynamic_visibility_follow_pack_and_author(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (uid, user) = h.user().await;
    let pack = h.pack(uid, "参赛作品", "published").await;
    let (category,): (i32,) =
        sqlx::query_as("INSERT INTO categories(slug,name) VALUES('votes','讨论') RETURNING id")
            .fetch_one(&h.pool)
            .await?;
    let (topic,):(i64,)=sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html,like_count) VALUES($1,$2,'作品讨论','介绍','介绍',5) RETURNING id").bind(category).bind(uid).fetch_one(&h.pool).await?;
    sqlx::query("UPDATE packs SET topic_id=$2 WHERE id=$1")
        .bind(pack)
        .bind(topic)
        .execute(&h.pool)
        .await?;
    let draft = create_challenge(&h, "creative-day").await;
    let published = publish(
        &h,
        "creative-day",
        draft["challenge"]["revision"].as_i64().unwrap(),
    )
    .await;
    let (status, value) = call(
        &h.app,
        "POST",
        "/api/challenges/creative-day/entries",
        Some(&user),
        json!({"pack_id":pack}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let (status, admin) = call(
        &h.app,
        "GET",
        "/api/admin/community-editorial/challenges/creative-day",
        Some(&h.admin),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{admin}");
    let revision = admin["entries"]["items"][0]["revision"].as_i64().unwrap();
    let input = json!({"expected_revision":revision,"featured":true,"sort_order":1});
    let path = format!("/api/admin/community-editorial/challenges/creative-day/entries/{pack}");
    let (status, _) = call(&h.app, "PATCH", &path, Some(&user), input.clone()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, value) = call(&h.app, "PATCH", &path, Some(&h.admin), input.clone()).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let (status, _) = call(&h.app, "PATCH", &path, Some(&h.admin), input).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (_, detail) = call(
        &h.app,
        "GET",
        "/api/challenges/creative-day",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(detail["entries"]["items"][0]["featured"], true);
    assert_eq!(detail["entries"]["items"][0]["vote_topic_id"], topic);
    assert_eq!(
        detail["entries"]["items"][0]["likes"], 5,
        "entry votes reuse the existing topic counter"
    );
    sqlx::query("UPDATE topics SET min_read_level=1 WHERE id=$1")
        .bind(topic)
        .execute(&h.pool)
        .await?;
    let (_, hidden) = call(
        &h.app,
        "GET",
        "/api/challenges/creative-day",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(
        hidden["entries"]["total"], 0,
        "private linked discussion hides the work"
    );
    sqlx::query("UPDATE topics SET min_read_level=0 WHERE id=$1")
        .bind(topic)
        .execute(&h.pool)
        .await?;

    sqlx::query("UPDATE packs SET status='removed' WHERE id=$1")
        .bind(pack)
        .execute(&h.pool)
        .await?;
    let (_, hidden) = call(
        &h.app,
        "GET",
        "/api/challenges/creative-day",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(hidden["entries"]["total"], 0);
    assert_eq!(hidden["challenge"]["entry_count"], 0);
    sqlx::query("UPDATE packs SET status='published' WHERE id=$1")
        .bind(pack)
        .execute(&h.pool)
        .await?;
    sqlx::query("UPDATE users SET status='banned' WHERE uid=$1")
        .bind(uid)
        .execute(&h.pool)
        .await?;
    let (_, hidden) = call(
        &h.app,
        "GET",
        "/api/challenges/creative-day",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(hidden["entries"]["total"], 0);
    let close = json!({"expected_revision":published["challenge"]["revision"]});
    let (status, value) = call(
        &h.app,
        "POST",
        "/api/admin/community-editorial/challenges/creative-day/close",
        Some(&h.admin),
        close,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["challenge"]["status"], "closed");
    let (logs,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM admin_audit WHERE action='community.challenge.entry.feature'",
    )
    .fetch_one(&h.pool)
    .await?;
    assert_eq!(logs, 1);
    h.close().await?;
    Ok(())
}
#[tokio::test]
async fn editorial_challenge_upcoming_withdrawal_and_public_discussion_permissions(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (uid, user) = h.user().await;
    let pack = h.pack(uid, "即将参赛", "published").await;
    let (category,): (i32,) =
        sqlx::query_as("INSERT INTO categories(slug,name) VALUES('challenge','挑战') RETURNING id")
            .fetch_one(&h.pool)
            .await?;
    let (topic,):(i64,)=sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html) VALUES($1,$2,'挑战讨论','讨论','讨论') RETURNING id").bind(category).bind(uid).fetch_one(&h.pool).await?;
    let mut input = challenge_input();
    input["topic_id"] = json!(topic);
    input["starts_at"] = json!(chrono::Utc::now() + chrono::Duration::days(1));
    input["key"] = json!("tomorrow");
    let (status, draft) = call(
        &h.app,
        "POST",
        "/api/admin/community-editorial/challenges",
        Some(&h.admin),
        input.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let published = publish(
        &h,
        "tomorrow",
        draft["challenge"]["revision"].as_i64().unwrap(),
    )
    .await;
    let (status, _) = call(
        &h.app,
        "POST",
        "/api/challenges/tomorrow/entries",
        Some(&user),
        json!({"pack_id":pack}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (_, detail) = call(&h.app, "GET", "/api/challenges/tomorrow", None, Value::Null).await;
    assert_eq!(detail["challenge"]["status"], "upcoming");
    assert_eq!(detail["challenge"]["topic_id"], topic);
    sqlx::query("UPDATE topics SET min_read_level=1 WHERE id=$1")
        .bind(topic)
        .execute(&h.pool)
        .await?;
    let (_, detail) = call(&h.app, "GET", "/api/challenges/tomorrow", None, Value::Null).await;
    assert!(detail["challenge"]["topic_id"].is_null());
    let (status, draft) = call(
        &h.app,
        "POST",
        "/api/admin/community-editorial/challenges/tomorrow/unpublish",
        Some(&h.admin),
        json!({"expected_revision":published["challenge"]["revision"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{draft}");
    let (status, _) = call(&h.app, "GET", "/api/challenges/tomorrow", None, Value::Null).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &h.app,
        "POST",
        "/api/admin/community-editorial/challenges/tomorrow/publish",
        Some(&h.admin),
        json!({"expected_revision":draft["challenge"]["revision"]}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "publication rechecks discussion visibility"
    );
    h.close().await?;
    Ok(())
}

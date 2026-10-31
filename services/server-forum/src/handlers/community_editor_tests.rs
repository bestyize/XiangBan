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

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
        let schema = format!("information_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?;
        super::init(&pool).await?;
        super::init(&pool).await?;
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
        let app = Router::new().nest("/api", routes()).with_state(state);
        Ok(Some(Self {
            pool,
            root,
            schema,
            app,
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
    async fn close(self) -> Result<(), sqlx::Error> {
        self.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.root)
            .await?;
        self.root.close().await;
        Ok(())
    }
}

fn input(id: Uuid) -> Value {
    json!({"id":id,"revision":0,"title":"城市骑行","interests":"杭州骑行与周末户外活动","recipient":"reader@example.test","schedule":{"kind":"daily","time":"08:00"},"timezone":"Asia/Shanghai","active":true,"confirmed":true,"budget_per_run":150,"budget_per_month":300})
}
#[tokio::test]
async fn ownership_verification_edits_and_history() {
    let Some(h) = Harness::new().await.unwrap() else {
        return;
    };
    let (uid, token) = h.user().await;
    let (_, other) = h.user().await;
    let id = Uuid::new_v4();
    let path = format!("/api/me/information-subscriptions/{id}");
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/me/information-subscriptions",
            None,
            input(id)
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (status, row) = call(
        &h.app,
        "POST",
        "/api/me/information-subscriptions",
        Some(&token),
        input(id),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(row["status"], "unverified");
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/me/information-subscriptions",
            Some(&token),
            input(id)
        )
        .await
        .1["revision"],
        1
    );
    assert_eq!(
        call(&h.app, "PUT", &path, Some(&other), input(id)).await.0,
        StatusCode::NOT_FOUND
    );
    let code = "test-verification-code";
    sqlx::query("INSERT INTO information_verifications(subscription_id,digest,expires_at) VALUES($1,$2,now()+interval '30 minutes')").bind(id).bind(digest(id,code)).execute(&h.pool).await.unwrap();
    assert_eq!(
        call(
            &h.app,
            "POST",
            &format!("{path}/verify"),
            Some(&token),
            json!({"code":"wrong"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            &format!("{path}/verify"),
            Some(&token),
            json!({"code":code})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            &format!("{path}/verify"),
            Some(&token),
            json!({"code":code})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    // Both ticks race over the same due occurrence. Only one edition survives.
    sqlx::query(
        "UPDATE information_subscriptions SET next_run=now()-interval '1 minute' WHERE id=$1",
    )
    .bind(id)
    .execute(&h.pool)
    .await
    .unwrap();
    let now = Utc::now();
    let (a, b) = tokio::join!(
        super::worker::enqueue(&h.pool, now),
        super::worker::enqueue(&h.pool, now)
    );
    a.unwrap();
    b.unwrap();
    let editions: i64 = sqlx::query_scalar("SELECT count(*) FROM information_editions")
        .fetch_one(&h.pool)
        .await
        .unwrap();
    assert_eq!(editions, 1);
    let (a, b) = tokio::join!(super::worker::claim(&h.pool), super::worker::claim(&h.pool));
    assert_eq!(
        usize::from(a.unwrap().is_some()) + usize::from(b.unwrap().is_some()),
        1
    );
    let history = call(
        &h.app,
        "GET",
        "/api/me/information-editions",
        Some(&token),
        Value::Null,
    )
    .await
    .1;
    let edition = history["items"][0]["id"].as_str().unwrap();
    assert_eq!(
        call(
            &h.app,
            "GET",
            &format!("/api/me/information-editions/{edition}"),
            Some(&other),
            Value::Null
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let mut update = input(id);
    update["revision"] = json!(2);
    update["active"] = json!(false);
    assert_eq!(
        call(&h.app, "PUT", &path, Some(&token), update.clone())
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&h.app, "PUT", &path, Some(&token), update).await.0,
        StatusCode::CONFLICT
    );
    let status: String = sqlx::query_scalar("SELECT status FROM information_editions")
        .fetch_one(&h.pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
    let mut update = input(id);
    update["revision"] = json!(3);
    update["recipient"] = json!("new@example.test");
    let row = call(&h.app, "PUT", &path, Some(&token), update).await.1;
    assert_eq!(row["status"], "unverified");
    assert!(row["verified_at"].is_null());
    assert_eq!(
        call(&h.app, "DELETE", &path, Some(&token), json!({"revision":4}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/me/information-subscriptions",
            Some(&token),
            Value::Null
        )
        .await
        .1["items"],
        json!([])
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/me/information-editions",
            Some(&token),
            Value::Null
        )
        .await
        .1["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let owners: i64 =
        sqlx::query_scalar("SELECT count(*) FROM information_editions WHERE owner=$1")
            .bind(uid)
            .fetch_one(&h.pool)
            .await
            .unwrap();
    assert_eq!(owners, 1);
    h.close().await.unwrap();
}
#[test]
fn rejects_unconfirmed_and_credentials() {
    let mut v = input(Uuid::new_v4());
    v["confirmed"] = json!(false);
    assert!(serde_json::from_value::<Input>(v)
        .unwrap()
        .validate()
        .is_err());
    let mut v = input(Uuid::new_v4());
    v["api_key"] = json!("never-accept");
    assert!(serde_json::from_value::<Input>(v).is_err());
    let mut v = input(Uuid::new_v4());
    v["recipient"] = json!("a@b.test\r\nBcc:attacker@evil.test");
    assert!(serde_json::from_value::<Input>(v)
        .unwrap()
        .validate()
        .is_err());
}

#[tokio::test]
async fn archive_cursor_keeps_every_row_with_identical_timestamp() {
    let Some(h) = Harness::new().await.unwrap() else {
        return;
    };
    let (owner, token) = h.user().await;
    let id = Uuid::new_v4();
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/me/information-subscriptions",
            Some(&token),
            input(id)
        )
        .await
        .0,
        StatusCode::OK
    );
    let at = Utc::now();
    for i in 0..65 {
        sqlx::query("INSERT INTO information_editions(id,subscription_id,owner,revision,due_at,snapshot,created_at) VALUES($1,$2,$3,1,$4,$5,$6)")
            .bind(Uuid::new_v4()).bind(id).bind(owner).bind(at+Duration::seconds(i)).bind(json!({"title":"同批期刊"})).bind(at).execute(&h.pool).await.unwrap();
    }
    let mut path = "/api/me/information-editions".to_string();
    let mut seen = std::collections::HashSet::new();
    let mut sizes = Vec::new();
    loop {
        let (status, page) = call(&h.app, "GET", &path, Some(&token), Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        let rows = page["items"].as_array().unwrap();
        sizes.push(rows.len());
        for row in rows {
            assert!(
                seen.insert(row["id"].as_str().unwrap().to_string()),
                "duplicate page row"
            );
        }
        if page["next_cursor"].is_null() {
            break;
        }
        let cursor = &page["next_cursor"];
        path = format!(
            "/api/me/information-editions?before={}&before_id={}",
            cursor["created_at"].as_str().unwrap().replace('+', "%2B"),
            cursor["id"].as_str().unwrap()
        );
    }
    assert_eq!(sizes, vec![30, 30, 5]);
    assert_eq!(seen.len(), 65);
    h.close().await.unwrap();
}

#[test]
fn validates_explicit_subscription_budgets_without_price_defaults() {
    let mut v = input(Uuid::new_v4());
    v["budget_per_run"] = json!(0);
    assert!(serde_json::from_value::<Input>(v)
        .unwrap()
        .validate()
        .is_err());
    let mut v = input(Uuid::new_v4());
    v["budget_per_month"] = json!(100);
    assert!(serde_json::from_value::<Input>(v)
        .unwrap()
        .validate()
        .is_err());
    let mut v = input(Uuid::new_v4());
    v["budget_per_run"] = Value::Null;
    v["budget_per_month"] = Value::Null;
    v["revision"] = json!(2);
    assert!(serde_json::from_value::<Input>(v.clone())
        .unwrap()
        .validate()
        .is_err());
    v["active"] = json!(false);
    assert!(serde_json::from_value::<Input>(v)
        .unwrap()
        .validate()
        .is_ok());
}

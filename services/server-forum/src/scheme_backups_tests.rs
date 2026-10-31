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
        let schema = format!("scheme_backups_test_{}", Uuid::new_v4().simple());
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

fn payload(pack: i64, revision: i64, title: &str) -> Value {
    json!({"schema_version":1,"title":title,"items":[{"pack_id":pack,"version":1}],"expected_revision":revision})
}
async fn published_pack(h: &Harness, uid: Uuid) -> i64 {
    let id = h.pack(uid, "作品", "published").await;
    sqlx::query("INSERT INTO pack_versions(pack_id,version,label,sha256,size,manifest,file_key) VALUES($1,1,'v1',$2,1,'{}','private-file')").bind(id).bind("a".repeat(64)).execute(&h.pool).await.unwrap();
    id
}
#[tokio::test]
async fn scheme_backup_rejects_nonexistent_versions_and_bounds_pagination(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (uid, user) = h.user().await;
    let pack = published_pack(&h, uid).await;
    let id = Uuid::new_v4();
    let path = format!("/api/me/scheme-backups/{id}");
    let valid = payload(pack, 0, "备份");
    let mut invalid = valid.clone();
    invalid["items"][0]["version"] = json!(99);
    assert_eq!(
        call(&h.app, "PUT", &path, Some(&user), invalid).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&h.app, "GET", &path, Some(&user), Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&h.app, "PUT", &path, Some(&user), valid).await.0,
        StatusCode::OK
    );
    let mut invalid = payload(pack, 1, "不得改写");
    invalid["items"][0]["version"] = json!(99);
    assert_eq!(
        call(&h.app, "PUT", &path, Some(&user), invalid).await.0,
        StatusCode::BAD_REQUEST
    );
    let (_, detail) = call(&h.app, "GET", &path, Some(&user), Value::Null).await;
    assert_eq!(detail["revision"], 1);
    assert_eq!(detail["title"], "备份");
    let (status, list) = call(
        &h.app,
        "GET",
        "/api/me/scheme-backups?page=9223372036854775807&page_size=50",
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["page"], 100000);
    assert_eq!(list["items"], json!([]));
    assert_eq!(list["total"], 1);
    h.close().await?;
    Ok(())
}
#[tokio::test]
async fn scheme_backup_concurrent_updates_and_remove_compare_exact_revision(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (uid, user) = h.user().await;
    let pack = published_pack(&h, uid).await;
    let id = Uuid::new_v4();
    let path = format!("/api/me/scheme-backups/{id}");
    assert_eq!(
        call(&h.app, "PUT", &path, Some(&user), payload(pack, 0, "初版"))
            .await
            .0,
        StatusCode::OK
    );
    let (first, second) = tokio::join!(
        call(
            &h.app,
            "PUT",
            &path,
            Some(&user),
            payload(pack, 1, "甲编辑")
        ),
        call(
            &h.app,
            "PUT",
            &path,
            Some(&user),
            payload(pack, 1, "乙编辑")
        )
    );
    let mut statuses = [first.0.as_u16(), second.0.as_u16()];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    let (_, detail) = call(&h.app, "GET", &path, Some(&user), Value::Null).await;
    assert_eq!(detail["revision"], 2);
    assert!(["甲编辑", "乙编辑"].contains(&detail["title"].as_str().unwrap()));
    let remove = format!("{path}/remove");
    let (update, removed) = tokio::join!(
        call(
            &h.app,
            "PUT",
            &path,
            Some(&user),
            payload(pack, 2, "再次更新")
        ),
        call(
            &h.app,
            "POST",
            &remove,
            Some(&user),
            json!({"expected_revision":2})
        )
    );
    let mut statuses = [update.0.as_u16(), removed.0.as_u16()];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    let (_, detail) = call(&h.app, "GET", &path, Some(&user), Value::Null).await;
    assert_eq!(detail["revision"], 3);
    if update.0 == StatusCode::OK {
        assert_eq!(detail["status"], "active");
        assert_eq!(detail["title"], "再次更新");
    } else {
        assert_eq!(detail["status"], "removed");
    }
    h.close().await?;
    Ok(())
}

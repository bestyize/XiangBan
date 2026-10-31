use crate::{auth::JwksVerifier, config::AppConfig};
use crate::{reply_content, state::AppState};
use axum::Router;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{json, Value};
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
        let schema = format!("reply_content_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?;
        reply_content::schema::init(&pool).await?;
        reply_content::schema::init(&pool).await?;
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
        config.upload_dir = std::env::temp_dir().join(&schema);
        tokio::fs::create_dir_all(&config.upload_dir).await?;
        let state = AppState {
            pool: pool.clone(),
            config: Arc::new(config),
            verifier: Arc::new(verifier),
            http: reqwest::Client::new(),
        };
        let saved_state = state.clone();
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
        let _ = tokio::fs::remove_dir_all(&self.state.config.upload_dir).await;
        self.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.root)
            .await?;
        self.root.close().await;
        Ok(())
    }
}

impl Harness {
    async fn topic(&self, uid: Uuid) -> i64 {
        let (cat,): (i32,) =
            sqlx::query_as("INSERT INTO categories(slug,name) VALUES($1,'讨论') RETURNING id")
                .bind(Uuid::new_v4().to_string())
                .fetch_one(&self.pool)
                .await
                .unwrap();
        let (id,):(i64,)=sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html) VALUES($1,$2,'回复测试','正文','正文') RETURNING id").bind(cat).bind(uid).fetch_one(&self.pool).await.unwrap();
        id
    }
    async fn attachment(&self, uid: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO attachments(id,uploader_uid,filename,size) VALUES($1,$2,'hello.txt',5)",
        )
        .bind(id)
        .bind(uid)
        .execute(&self.pool)
        .await
        .unwrap();
        tokio::fs::write(self.state.config.upload_dir.join(id.to_string()), b"hello")
            .await
            .unwrap();
        id
    }
    async fn version(&self, id: i64) {
        sqlx::query("INSERT INTO pack_versions(pack_id,version,label,sha256,size,manifest,file_key) VALUES($1,1,'v1',$2,1,'{}','private-file')").bind(id).bind("a".repeat(64)).execute(&self.pool).await.unwrap();
    }
}
#[tokio::test]
async fn reply_content_ownership_atomicity_edit_compatibility_and_topic_isolation(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (uid, user) = h.user().await;
    let (other_uid, _) = h.user().await;
    let topic = h.topic(uid).await;
    let pack = h.pack(uid, "引用作品", "published").await;
    h.version(pack).await;
    let own = h.attachment(uid).await;
    let other = h.attachment(other_uid).await;
    let create = format!("/api/topics/{topic}/posts");
    let (status,_)=call(&h.app,"POST",&create,Some(&user),json!({"body":"作品介绍","work_refs":[{"kind":"pack","id":pack,"version":1}],"attachment_ids":[own,other]})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM posts WHERE topic_id=$1")
        .bind(topic)
        .fetch_one(&h.pool)
        .await?;
    assert_eq!(n, 0);
    let (unused,): (bool,) = sqlx::query_as("SELECT topic_id IS NULL FROM attachments WHERE id=$1")
        .bind(own)
        .fetch_one(&h.pool)
        .await?;
    assert!(unused);
    for refs in [
        json!([{"kind":"pack","id":pack,"version":99}]),
        json!([{"kind":"pack","id":pack},{"kind":"pack","id":pack}]),
        json!([{"kind":"topic","id":topic}]),
    ] {
        assert_eq!(
            call(
                &h.app,
                "POST",
                &create,
                Some(&user),
                json!({"body":"非法引用","work_refs":refs})
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
    }
    let (status,data)=call(&h.app,"POST",&create,Some(&user),json!({"body":"分享作品","work_refs":[{"kind":"pack","id":pack}],"attachment_ids":[own],"attachment_prices":[{"id":own,"price":3}]})).await;
    assert_eq!(status, StatusCode::OK, "{data}");
    let post = data["id"].as_i64().unwrap();
    let (_, list) = call(&h.app, "GET", &create, Some(&user), Value::Null).await;
    assert_eq!(list["items"][0]["work_refs"][0]["version"], 1);
    assert_eq!(list["items"][0]["attachments"][0]["price"], 3);
    assert!(list["items"][0]["work_refs"][0]["content"]
        .get("versions")
        .is_none());
    let (status, _) = call(
        &h.app,
        "PATCH",
        &format!("/api/posts/{post}"),
        Some(&user),
        json!({"body":"旧客户端编辑"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, source) = call(
        &h.app,
        "GET",
        &format!("/api/posts/{post}/source"),
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(source["work_refs"].as_array().unwrap().len(), 1);
    assert_eq!(source["attachments"].as_array().unwrap().len(), 1);
    let header = h.attachment(uid).await;
    let mut tx = h.pool.begin().await?;
    crate::handlers::attachments::set_for_topic(&mut tx, topic, uid, &[header]).await?;
    tx.commit().await?;
    let header_list =
        crate::handlers::attachments::list_for_topic(&h.pool, topic, Some(uid)).await?;
    assert_eq!(header_list.len(), 1);
    assert_eq!(header_list[0].id, header);
    let (_, list) = call(&h.app, "GET", &create, Some(&user), Value::Null).await;
    assert_eq!(list["items"][0]["attachments"].as_array().unwrap().len(), 1);
    // A topic edit cannot steal a reply attachment or delete it as an omitted topic attachment.
    let mut tx = h.pool.begin().await?;
    assert!(
        crate::handlers::attachments::set_for_topic(&mut tx, topic, uid, &[own])
            .await
            .is_err()
    );
    tx.rollback().await?;
    let (status, _) = call(
        &h.app,
        "POST",
        &create,
        Some(&user),
        json!({"body":"不能重复领取","attachment_ids":[own]}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    sqlx::query("UPDATE packs SET status='removed' WHERE id=$1")
        .bind(pack)
        .execute(&h.pool)
        .await?;
    let (_, list) = call(&h.app, "GET", &create, Some(&user), Value::Null).await;
    let unavailable = &list["items"][0]["work_refs"][0];
    assert_eq!(unavailable["available"], false);
    assert!(unavailable.get("content").is_none());
    let (status, _) = call(
        &h.app,
        "PATCH",
        &format!("/api/posts/{post}"),
        Some(&user),
        json!({"body":"保留失效引用编辑正文","work_refs":[{"kind":"pack","id":pack,"version":1}]}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "unchanged unavailable references remain editable"
    );
    let (status, _) = call(
        &h.app,
        "PATCH",
        &format!("/api/posts/{post}"),
        Some(&user),
        json!({"body":"明确清空附件","work_refs":[],"attachment_ids":[]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, source) = call(
        &h.app,
        "GET",
        &format!("/api/posts/{post}/source"),
        Some(&user),
        Value::Null,
    )
    .await;
    assert_eq!(source["work_refs"], json!([]));
    assert_eq!(source["attachments"], json!([]));
    assert!(!h.state.config.upload_dir.join(own.to_string()).exists());
    h.close().await?;
    Ok(())
}
#[tokio::test]
async fn reply_content_hidden_blocked_banned_and_restricted_files_cannot_be_bypassed(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (author, user) = h.user().await;
    let (viewer, reader) = h.user().await;
    let topic = h.topic(author).await;
    let pack = h.pack(author, "公开引用", "published").await;
    h.version(pack).await;
    let attachment = h.attachment(author).await;
    let create = format!("/api/topics/{topic}/posts");
    let (_,data)=call(&h.app,"POST",&create,Some(&user),json!({"body":"作品附件","work_refs":[{"kind":"pack","id":pack}],"attachment_ids":[attachment]})).await;
    let post = data["id"].as_i64().unwrap();
    let file = format!("/api/attachments/{attachment}");
    assert_eq!(
        call(&h.app, "GET", &file, None, Value::Null).await.0,
        StatusCode::OK
    );
    sqlx::query("UPDATE posts SET status='hidden' WHERE id=$1")
        .bind(post)
        .execute(&h.pool)
        .await?;
    assert_eq!(
        call(&h.app, "GET", &file, None, Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            &format!("{file}/purchase"),
            Some(&reader),
            json!({"price":0})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (_, list) = call(&h.app, "GET", &create, Some(&reader), Value::Null).await;
    assert_eq!(list["items"][0]["work_refs"], json!([]));
    assert_eq!(list["items"][0]["attachments"], json!([]));
    sqlx::query("UPDATE posts SET status='normal' WHERE id=$1")
        .bind(post)
        .execute(&h.pool)
        .await?;
    sqlx::query("INSERT INTO user_blocks(blocker,blocked) VALUES($1,$2)")
        .bind(viewer)
        .bind(author)
        .execute(&h.pool)
        .await?;
    let (_, list) = call(&h.app, "GET", &create, Some(&reader), Value::Null).await;
    assert_eq!(list["items"][0]["attachments"], json!([]));
    assert_eq!(list["items"][0]["work_refs"], json!([]));
    assert_eq!(
        call(&h.app, "GET", &file, Some(&reader), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    sqlx::query("DELETE FROM user_blocks")
        .execute(&h.pool)
        .await?;
    sqlx::query("UPDATE topics SET min_read_level=2 WHERE id=$1")
        .bind(topic)
        .execute(&h.pool)
        .await?;
    assert_eq!(
        call(&h.app, "GET", &file, Some(&reader), Value::Null)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("UPDATE topics SET min_read_level=0 WHERE id=$1")
        .bind(topic)
        .execute(&h.pool)
        .await?;
    sqlx::query("UPDATE users SET status='banned' WHERE uid=$1")
        .bind(author)
        .execute(&h.pool)
        .await?;
    let (_, list) = call(&h.app, "GET", &create, Some(&reader), Value::Null).await;
    assert_eq!(list["items"][0]["work_refs"], json!([]));
    assert_eq!(list["items"][0]["status"], "hidden");
    assert_eq!(
        call(&h.app, "GET", &file, None, Value::Null).await.0,
        StatusCode::NOT_FOUND
    );
    h.close().await?;
    Ok(())
}

use super::*;
use crate::{auth::JwksVerifier, config::AppConfig};
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

async fn call(
    app: axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 32 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn scoped_workflow_is_private_idempotent_and_revocable(
) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return Ok(());
    };
    let schema = format!("editor_test_{}", Uuid::new_v4().simple());
    let single = std::env::var("TEST_SINGLE_CONNECTION").as_deref() == Ok("1");
    let admin = if single {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await?
    } else {
        crate::db::connect(&url, None).await?
    };
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await?;
    let pool = if single {
        admin.clone()
    } else {
        crate::db::connect(&url, Some(&schema)).await?
    };
    if single {
        sqlx::raw_sql(&format!("SET search_path TO {schema}"))
            .execute(&pool)
            .await?;
    }
    crate::db::migrate(&pool).await?;
    crate::db::migrate(&pool).await?;
    crate::db::seed(&pool).await?;
    let mut config = AppConfig::from_env();
    config.upload_dir = std::env::temp_dir().join(&schema);
    let state = AppState {
        pool: pool.clone(),
        config: Arc::new(config),
        verifier: Arc::new(JwksVerifier::new(
            "test".into(),
            "test".into(),
            "test".into(),
        )),
        http: reqwest::Client::new(),
    };
    let app = crate::build_router(state.clone());
    let uid = Uuid::new_v4();
    let other = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    for author in [uid, other] {
        sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES ($1,'editor',3)")
            .bind(author)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO editor_authors(uid) VALUES ($1)")
            .bind(author)
            .execute(&pool)
            .await?;
    }
    sqlx::query("INSERT INTO editor_tokens(id,name,author_uid,can_publish,expires_at) VALUES ($1,'test',$2,false,now()+interval '1 day')").bind(token_id).bind(uid).execute(&pool).await?;
    let token = encode(
        &Header::new(Algorithm::HS256),
        &Claims {
            sub: uid.to_string(),
            jti: token_id,
            aud: "forum-editor".into(),
            exp: (Utc::now() + Duration::days(1)).timestamp(),
        },
        &EncodingKey::from_secret(&state.config.admin_jwt_secret),
    )?;
    let (category,):(String,)=sqlx::query_as("SELECT c.slug FROM categories c WHERE c.kind='normal' AND NOT EXISTS(SELECT 1 FROM categories ch WHERE ch.parent_id=c.id) LIMIT 1").fetch_one(&pool).await?;
    let input = json!({"author_uid":uid,"external_id":"test-001","title":"具体可用的图文教程","category":category,"body":"这篇文章包含明确的操作步骤。\n\n![图](images/a.png)","tags":["测试"],"images":[{"path":"images/a.png","data_base64":STANDARD.encode(b"\x89PNG\r\n\x1a\nexample")}]});
    let (status, _) = call(
        app.clone(),
        "POST",
        "/api/editor/drafts",
        None,
        input.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let mut invalid = input.clone();
    invalid["author_uid"] = json!(other);
    let (status, _) = call(
        app.clone(),
        "POST",
        "/api/editor/drafts",
        Some(&token),
        invalid,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, result) = call(
        app.clone(),
        "POST",
        "/api/editor/drafts",
        Some(&token),
        input.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let id = result["id"].as_str().unwrap();
    let (status, _) = call(
        app.clone(),
        "POST",
        "/api/editor/drafts",
        Some(&token),
        input.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (_, d) = call(
        app.clone(),
        "GET",
        &format!("/api/editor/drafts/{id}"),
        Some(&token),
        Value::Null,
    )
    .await;
    assert!(d["html"]
        .as_str()
        .unwrap()
        .contains("data:image/png;base64,"));
    let (image,): (Uuid,) =
        sqlx::query_as("SELECT image_id FROM editor_draft_assets WHERE draft_id=$1")
            .bind(Uuid::parse_str(id)?)
            .fetch_one(&pool)
            .await?;
    let (status, _) = call(
        app.clone(),
        "GET",
        &format!("/api/images/{image}"),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    sqlx::query("UPDATE images SET created_at=now()-interval '2 days'")
        .execute(&pool)
        .await?;
    assert_eq!(
        images::purge_unused(&pool, &state.config.upload_dir).await?,
        0
    );
    let publish_path = format!("/api/editor/drafts/{id}/publish");
    let (status, _) = call(
        app.clone(),
        "POST",
        &publish_path,
        Some(&token),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    sqlx::query("UPDATE editor_tokens SET can_publish=true WHERE id=$1")
        .bind(token_id)
        .execute(&pool)
        .await?;
    let (status, published) = call(
        app.clone(),
        "POST",
        &publish_path,
        Some(&token),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{published}");
    let (_, again) = call(
        app.clone(),
        "POST",
        &publish_path,
        Some(&token),
        Value::Null,
    )
    .await;
    assert_eq!(published["id"], again["id"]);
    assert_eq!(again["already_published"], true);
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM topics")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 1);
    let (status, _) = call(
        app.clone(),
        "GET",
        &format!("/api/images/{image}"),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    sqlx::query("UPDATE editor_tokens SET revoked_at=now() WHERE id=$1")
        .bind(token_id)
        .execute(&pool)
        .await?;
    let (status, _) = call(app, "GET", "/api/editor/context", Some(&token), Value::Null).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    drop(state);
    if single {
        sqlx::raw_sql(&format!(
            "SET search_path TO public; DROP SCHEMA {schema} CASCADE; DEALLOCATE ALL"
        ))
        .execute(&pool)
        .await?;
        pool.close().await;
    } else {
        pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&admin)
            .await?;
    }
    tokio::fs::remove_dir_all(std::env::temp_dir().join(schema)).await?;
    admin.close().await;
    Ok(())
}

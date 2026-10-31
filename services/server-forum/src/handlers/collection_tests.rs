use super::*;
use crate::{auth::JwksVerifier, collectors::CollectedItem, config::AppConfig};
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
struct Harness {
    pool: PgPool,
    root: PgPool,
    schema: String,
    app: axum::Router,
    token: String,
    state: AppState,
}
async fn request(
    app: &axum::Router,
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
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
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
        let schema = format!("collection_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?;
        collectors::schema::init(&pool).await?;
        collectors::schema::init(&pool).await?;
        let mut config = AppConfig::from_env();
        config.admin_user = Some("test".into());
        config.admin_password = Some("test-password-for-collections".into());
        super::super::admin::seed_admin(&pool, &config).await?;
        let state = AppState {
            pool: pool.clone(),
            config: Arc::new(config),
            verifier: Arc::new(JwksVerifier::new(
                "unused".into(),
                "unused".into(),
                "unused".into(),
            )),
            http: reqwest::Client::new(),
        };
        let app = crate::build_router(state.clone());
        let (status, login) = request(
            &app,
            "POST",
            "/api/admin/login",
            None,
            json!({"username":"test","password":"test-password-for-collections"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{login}");
        Ok(Some(Self {
            pool,
            root,
            schema,
            app,
            token: login["token"].as_str().unwrap().into(),
            state,
        }))
    }
    async fn close(self) -> Result<(), sqlx::Error> {
        self.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&self.root)
            .await?;
        self.root.close().await;
        Ok(())
    }
    async fn run(&self) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO collection_runs(id,source,status) VALUES($1,'openrouter','running')",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .unwrap();
        sqlx::query("UPDATE collection_sources SET active_run=$1,lease_until=now()+interval '10 minutes' WHERE id='openrouter'").bind(id).execute(&self.pool).await.unwrap();
        id
    }
    async fn succeeded(&self, id: Uuid) {
        sqlx::query("UPDATE collection_runs SET status='succeeded',finished_at=now() WHERE id=$1")
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}
fn usage(date: &str, tokens: &str) -> CollectedItem {
    CollectedItem {
        kind: "model_usage".into(),
        key: date.into(),
        sort_order: 0,
        published_at: None,
        source_name: "OpenRouter".into(),
        source_url: "https://openrouter.ai/rankings".into(),
        data: json!({"source":"OpenRouter","source_url":"https://openrouter.ai/rankings","date":date,"as_of":"2026-10-30T12:00:00Z","version":"1","period":"day","timezone":"UTC","token_unit":"prompt_tokens + completion_tokens","total_tokens":tokens,"other_tokens":"0","models":[{"id":"example/model","name":"Example","tokens":tokens,"rank":1,"share_pct":100.0}]}),
    }
}
async fn revisions(pool: &PgPool, run: Uuid) -> Vec<discover_editor::BatchReviewItem> {
    let values: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT kind,key,revision FROM collection_run_items WHERE run_id=$1 ORDER BY kind,key",
    )
    .bind(run)
    .fetch_all(pool)
    .await
    .unwrap();
    values
        .into_iter()
        .map(
            |(kind, key, expected_revision)| discover_editor::BatchReviewItem {
                kind,
                key,
                expected_revision,
            },
        )
        .collect()
}

#[tokio::test]
async fn collection_batch_is_atomic_idempotent_and_review_requires_exact_complete_manifest(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let run = h.run().await;
    let items = vec![
        usage("2026-10-28", "9007199254740993"),
        usage("2026-10-29", "9"),
    ];
    discover_editor::write_collected_batch(&h.pool, "openrouter", run, items.clone()).await?;
    let (live,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM discover_items WHERE kind='model_usage'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(live, 0, "unreviewed collections must remain private");
    let manifest = revisions(&h.pool, run).await;
    let run2 = h.run().await;
    let mut same = items.clone();
    same[0].data["as_of"] = json!("2026-10-30T13:00:00Z");
    discover_editor::write_collected_batch(&h.pool, "openrouter", run2, same).await?;
    let manifest2 = revisions(&h.pool, run2).await;
    assert_eq!(
        manifest[0].expected_revision, manifest2[0].expected_revision,
        "metadata timestamps cannot create review churn"
    );
    h.succeeded(run).await;
    assert!(
        discover_editor::review_collected_batch(&h.pool, run, "test", vec![])
            .await
            .is_err()
    );
    let mut wrong = revisions(&h.pool, run).await;
    wrong[0].expected_revision += 1;
    assert!(
        discover_editor::review_collected_batch(&h.pool, run, "test", wrong)
            .await
            .is_err()
    );
    let draft = discover_editor::collected_batch_preview(&h.pool, run).await?;
    assert_eq!(draft["items"].as_array().unwrap().len(), 2);
    assert!(draft["items"][0]["live"].is_null());
    assert_eq!(draft["items"][0]["conflict"], false);
    let n = discover_editor::review_collected_batch(
        &h.pool,
        run,
        "test",
        revisions(&h.pool, run).await,
    )
    .await?;
    assert_eq!(n, 2);
    let (live,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM discover_items WHERE kind='model_usage' AND status='published'",
    )
    .fetch_one(&h.pool)
    .await?;
    assert_eq!(live, 2);
    assert!(discover_editor::review_collected_batch(
        &h.pool,
        run,
        "test",
        revisions(&h.pool, run).await
    )
    .await
    .is_err());
    let run3 = h.run().await;
    let mut changed = items.clone();
    changed[0] = usage("2026-10-28", "9007199254740994");
    discover_editor::write_collected_batch(&h.pool, "openrouter", run3, changed).await?;
    let (tokens,):(String,)=sqlx::query_as("SELECT data->>'total_tokens' FROM discover_items WHERE kind='model_usage' AND key='2026-10-28'").fetch_one(&h.pool).await?;
    assert_eq!(
        tokens, "9007199254740993",
        "source revisions cannot publish automatically"
    );
    // A manual edit to the later item must roll back the earlier changed draft too.
    sqlx::query("UPDATE discover_drafts SET updated_by='manual-editor',revision=nextval('discover_draft_revision_seq') WHERE kind='model_usage' AND key='2026-10-29'").execute(&h.pool).await?;
    let (before,): (i64,) = sqlx::query_as(
        "SELECT revision FROM discover_drafts WHERE kind='model_usage' AND key='2026-10-28'",
    )
    .fetch_one(&h.pool)
    .await?;
    let run4 = h.run().await;
    assert!(discover_editor::write_collected_batch(
        &h.pool,
        "openrouter",
        run4,
        vec![usage("2026-10-28", "100"), usage("2026-10-29", "11")]
    )
    .await
    .is_err());
    let (after,): (i64,) = sqlx::query_as(
        "SELECT revision FROM discover_drafts WHERE kind='model_usage' AND key='2026-10-28'",
    )
    .fetch_one(&h.pool)
    .await?;
    assert_eq!(before, after, "a conflict must roll back all batch writes");
    let (links,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM collection_run_items WHERE run_id=$1")
            .bind(run4)
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(links, 0);
    h.succeeded(run3).await;
    assert!(
        discover_editor::review_collected_batch(
            &h.pool,
            run3,
            "test",
            revisions(&h.pool, run3).await
        )
        .await
        .is_err(),
        "manual revisions must block the whole review"
    );
    // Legacy live administration cannot silently reset the collector's base version.
    sqlx::query("UPDATE discover_items SET updated_at=clock_timestamp() WHERE kind='model_usage' AND key='2026-10-28'").execute(&h.pool).await?;
    let edited_live_run = h.run().await;
    assert!(discover_editor::write_collected_batch(
        &h.pool,
        "openrouter",
        edited_live_run,
        vec![usage("2026-10-28", "100")]
    )
    .await
    .is_err());
    // Source boundaries and expired leases are enforced before any writes.
    let expired = Uuid::new_v4();
    assert!(
        discover_editor::write_collected_batch(&h.pool, "openrouter", expired, items.clone())
            .await
            .is_err()
    );
    let run5 = h.run().await;
    assert!(
        discover_editor::write_collected_batch(&h.pool, "artificial_analysis", run5, items)
            .await
            .is_err()
    );
    h.close().await?;
    Ok(())
}

#[tokio::test]
async fn collection_admin_auth_configuration_and_exclusion_are_enforced(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let (status, _) = request(
        &h.app,
        "GET",
        "/api/admin/collection/sources",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, body) = request(
        &h.app,
        "GET",
        "/api/admin/collection/sources",
        Some(&h.token),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|i| i["configured"] == false && i["enabled"] == false));
    for input in [
        json!({"enabled":true}),
        json!({"interval_minutes":59}),
        json!({"interval_minutes":10081}),
        json!({}),
    ] {
        let (status, body) = request(
            &h.app,
            "PATCH",
            "/api/admin/collection/sources/openrouter",
            Some(&h.token),
            input,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }
    let (status, body) = request(
        &h.app,
        "PATCH",
        "/api/admin/collection/sources/openrouter",
        Some(&h.token),
        json!({"interval_minutes":60,"enabled":false}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let run = h.run().await;
    assert!(matches!(
        collectors::start(h.state.clone(), "openrouter".into(), false).await,
        Err(AppError::Conflict(_))
    ));
    let (status, body) = request(
        &h.app,
        "GET",
        "/api/admin/collection/runs?source=openrouter",
        Some(&h.token),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["id"], run.to_string());
    let (status, _) = request(
        &h.app,
        "POST",
        &format!("/api/admin/collection/runs/{run}/review"),
        None,
        json!({"items":[]}),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    h.close().await?;
    Ok(())
}

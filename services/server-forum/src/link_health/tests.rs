use super::*;
#[test]
fn unsafe_urls_are_redacted_but_refs_remain_actionable() {
    let mut map = BTreeMap::new();
    insert_target(
        &mut map,
        "https://user:secret@openai.com/news?token=secret",
        Reference {
            kind: "nav".into(),
            key: "test".into(),
            field: "url".into(),
            scope: "published".into(),
        },
    );
    let row = map.into_values().next().unwrap();
    assert!(!row.url.contains("secret"));
    assert_eq!(row.refs[0].key, "test");
}
async fn settled(pool: &PgPool, id: Uuid) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let (status,): (String,) =
                sqlx::query_as("SELECT status FROM link_health_runs WHERE id=$1")
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .unwrap();
            if status != "running" {
                assert_eq!(status, "succeeded");
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}
async fn claim(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO link_health_runs(id,status,total_candidates,selected_count)VALUES($1,'running',1,1)").bind(id).execute(pool).await.unwrap();
    sqlx::query(
        "UPDATE link_health_settings SET active_run=$1,lease_until=now()+interval '5 minutes'",
    )
    .bind(id)
    .execute(pool)
    .await
    .unwrap();
    id
}
async fn exercise(pool: PgPool) {
    // Real migration, including status/publication-time and draft review constraints.
    crate::db::migrate(&pool).await.unwrap();
    collectors::schema::init(&pool).await.unwrap();
    schema::init(&pool).await.unwrap();
    schema::init(&pool).await.unwrap();
    let (enabled,): (bool,) = sqlx::query_as("SELECT enabled FROM link_health_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!enabled);
    assert!(matches!(
        start(&pool, true).await,
        Err(AppError::Conflict(_))
    ));
    for (kind, key, status, delta, data) in [
        (
            "nav",
            "live",
            "published",
            -1,
            json!({"url":"https://example.com/live"}),
        ),
        (
            "doc",
            "future",
            "published",
            1,
            json!({"url":"https://openai.com/future"}),
        ),
        (
            "doc",
            "removed",
            "draft",
            -1,
            json!({"url":"https://openai.com/removed"}),
        ),
        (
            "tool",
            "sample",
            "published",
            -1,
            json!({"homepage_url":"https://example.com/live","sample":true}),
        ),
    ] {
        sqlx::query("INSERT INTO discover_items(kind,key,status,published_at,data)VALUES($1,$2,$3,now()+make_interval(days=>$4),$5)").bind(kind).bind(key).bind(status).bind(delta).bind(data).execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO discover_drafts(kind,key,data,source_url,updated_by)VALUES('news','pending','{}','https://example.com/source','test')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO collection_owned_items(source,kind,key,revision)SELECT 'official_news',kind,key,revision FROM discover_drafts WHERE key='pending'").execute(&pool).await.unwrap();
    sqlx::raw_sql("UPDATE collection_sources SET enabled=true,last_success_at=now()-interval '3 days' WHERE id='artificial_analysis';UPDATE collection_sources SET enabled=true,last_success_at=now() WHERE id='github_tools';UPDATE collection_sources SET enabled=true WHERE id='official_news';INSERT INTO collection_runs(id,source,status)VALUES(gen_random_uuid(),'official_news','failed'),(gen_random_uuid(),'github_tools','succeeded');").execute(&pool).await.unwrap();
    let current = targets(&pool).await.unwrap();
    assert_eq!(
        current.len(),
        2,
        "duplicate homepage/source merged; removed/future excluded"
    );
    assert_eq!(
        current
            .iter()
            .find(|t| t.url.ends_with("/live"))
            .unwrap()
            .refs
            .len(),
        2
    );
    assert!(current.iter().any(|t| t.refs[0].scope == "editor_source"));
    assert!(!current.iter().any(|t| t.raw.contains("openai.com")));
    let summary = summary(&pool).await.unwrap();
    let columns = summary["columns"].as_array().unwrap();
    let doc = columns.iter().find(|c| c["kind"] == "doc").unwrap();
    assert_eq!(doc["published"], 0);
    assert_eq!(doc["future"], 1);
    assert_eq!(doc["unlisted"], 1);
    assert_eq!(
        columns.iter().find(|c| c["kind"] == "tool").unwrap()["sample"],
        1
    );
    let sources = summary["sources"].as_array().unwrap();
    for (id, state) in [
        ("artificial_analysis", "stale"),
        ("github_tools", "fresh"),
        ("official_news", "never"),
        ("openrouter", "disabled"),
    ] {
        assert_eq!(
            sources.iter().find(|s| s["id"] == id).unwrap()["freshness"],
            state
        );
    }
    let news = sources.iter().find(|s| s["id"] == "official_news").unwrap();
    assert_eq!(news["pending"], 1);
    assert_eq!(news["failed_24h"], 1);
    assert!(summary.get("users").is_none());
    // Neither caller URLs nor unknown fields can enter run jobs. Auth remains mandatory.
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use std::sync::Arc;
    use tower::ServiceExt;
    let mut config = crate::config::AppConfig::from_env();
    config.admin_user = Some("health-test".into());
    config.admin_password = Some("test-password-123".into());
    config.admin_jwt_secret = vec![42; 32];
    crate::handlers::admin::seed_admin(&pool, &config)
        .await
        .unwrap();
    let state = AppState {
        pool: pool.clone(),
        config: Arc::new(config),
        http: reqwest::Client::new(),
        verifier: Arc::new(crate::auth::JwksVerifier::new(
            "unused".into(),
            "unused".into(),
            "unused".into(),
        )),
    };
    let app = Router::new()
        .nest(
            "/api/admin",
            crate::handlers::admin::routes().merge(admin_routes()),
        )
        .with_state(state);
    let request = |method: &str, path: &str, token: &str, body: Value| {
        let mut req = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json");
        if !token.is_empty() {
            req = req.header("authorization", format!("Bearer {token}"))
        }
        req.body(Body::from(body.to_string())).unwrap()
    };
    assert_eq!(
        app.clone()
            .oneshot(request("GET", "/api/admin/link-health", "", json!({})))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let login = app
        .clone()
        .oneshot(request(
            "POST",
            "/api/admin/login",
            "",
            json!({"username":"health-test","password":"test-password-123"}),
        ))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let login: Value =
        serde_json::from_slice(&to_bytes(login.into_body(), 32768).await.unwrap()).unwrap();
    let token = login["token"].as_str().unwrap();
    assert_eq!(
        app.clone()
            .oneshot(request(
                "POST",
                "/api/admin/link-health/run",
                token,
                json!({"url":"http://127.0.0.1"})
            ))
            .await
            .unwrap()
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        app.clone()
            .oneshot(request(
                "PATCH",
                "/api/admin/link-health/settings",
                token,
                json!({"enabled":true,"interval_hours":1})
            ))
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_REQUEST
    );
    for path in ["/api/admin/link-health", "/api/admin/community-operations"] {
        assert_eq!(
            app.clone()
                .oneshot(request("GET", path, token, json!({})))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
    }
    // A valid active lease rejects duplicate tasks. Expired ownership is recovered explicitly.
    let old = claim(&pool).await;
    assert!(matches!(
        start(&pool, false).await,
        Err(AppError::Conflict(_))
    ));
    sqlx::query("UPDATE link_health_settings SET lease_until=now()-interval '1 minute'")
        .execute(&pool)
        .await
        .unwrap();
    for n in 0..63 {
        sqlx::query("INSERT INTO discover_items(kind,key,data)VALUES('nav',$1,$2)")
            .bind(format!("bounded-{n:02}"))
            .bind(json!({"url":format!("https://example.com/{n}")}))
            .execute(&pool)
            .await
            .unwrap();
    }
    let (id, total, selected) = start(&pool, false).await.unwrap();
    assert_eq!(total, 65);
    assert_eq!(selected, 60);
    settled(&pool, id).await;
    let (old_status,): (String,) =
        sqlx::query_as("SELECT status FROM link_health_runs WHERE id=$1")
            .bind(old)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(old_status, "failed");
    let (checked, blocked): (i32, i32) =
        sqlx::query_as("SELECT checked_count,blocked_count FROM link_health_runs WHERE id=$1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((checked, blocked), (60, 60));
    let (id, _, _) = start(&pool, false).await.unwrap();
    settled(&pool, id).await;
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM link_health_results")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count, 65,
        "unchecked links have priority on the next bounded run"
    );
    // Failure never erases the previous success, and stale jobs cannot overwrite a newer lease.
    let target = targets(&pool).await.unwrap().remove(0);
    let id = claim(&pool).await;
    let success = network::Outcome {
        status: "healthy".into(),
        http_status: Some(200),
        final_url: Some(target.url.clone()),
        error: None,
        duration_ms: 1,
    };
    finish(&pool, id, vec![(target.clone(), success)], false)
        .await
        .unwrap();
    let (last,): (DateTime<Utc>,) =
        sqlx::query_as("SELECT last_success_at FROM link_health_results WHERE id=$1")
            .bind(&target.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let id = claim(&pool).await;
    finish(
        &pool,
        id,
        vec![(
            target.clone(),
            network::Outcome::failure("broken", "不存在"),
        )],
        true,
    )
    .await
    .unwrap();
    let (preserved, failures): (DateTime<Utc>, i32) =
        sqlx::query_as("SELECT last_success_at,failures FROM link_health_results WHERE id=$1")
            .bind(&target.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(preserved, last);
    assert_eq!(failures, 1);
    let _new = claim(&pool).await;
    assert!(matches!(
        finish(&pool, id, vec![], false).await,
        Err(AppError::Conflict(_))
    ));
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_items")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 67);
    let (status,): (String,) =
        sqlx::query_as("SELECT status FROM discover_items WHERE key='removed'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "draft");
}
#[tokio::test]
async fn real_database_statuses_aggregates_auth_leases_bounded_runs_and_failure_preservation() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("SKIPPED link-health DB test: TEST_DATABASE_URL not set");
        return;
    };
    let root = crate::db::connect(&url, None).await.unwrap();
    let name = format!("link_health_test_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {name}"))
        .execute(&root)
        .await
        .unwrap();
    let pool = crate::db::connect(&url, Some(&name)).await.unwrap();
    let task = tokio::spawn(exercise(pool.clone())).await;
    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {name} CASCADE"))
        .execute(&root)
        .await
        .unwrap();
    root.close().await;
    task.unwrap();
}

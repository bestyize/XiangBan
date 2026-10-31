use super::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};
struct Harness {
    pool: PgPool,
    root: PgPool,
    schema: String,
}
impl Harness {
    async fn new() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("skipped: TEST_DATABASE_URL is not set");
            return Ok(None);
        };
        let root = crate::db::connect(&url, None).await?;
        let schema = format!("github_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?;
        super::super::schema::init(&pool).await?;
        Ok(Some(Self { pool, root, schema }))
    }
    async fn run(&self, config: &Value) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("UPDATE collection_sources SET config=$1,active_run=$2,lease_until=now()+interval '10 minutes' WHERE id='github_repos'").bind(config).bind(id).execute(&self.pool).await.unwrap();
        sqlx::query("INSERT INTO collection_runs(id,source,status,config) VALUES($1,'github_repos','running',$2)").bind(id).bind(config).execute(&self.pool).await.unwrap();
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
async fn mock(
    responses: Vec<(&'static str, u16, Option<String>, Value)>,
) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let mock_origin = origin.clone();
    let handle = tokio::spawn(async move {
        for (path, status, location, value) in responses {
            let (mut stream, _) =
                tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
                    .await
                    .unwrap()
                    .unwrap();
            let mut buffer = vec![0u8; 16384];
            let n = stream.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..n]);
            assert_eq!(
                request.lines().next().unwrap().split_whitespace().nth(1),
                Some(path)
            );
            assert!(request
                .to_ascii_lowercase()
                .contains("authorization: bearer test-token"));
            let body = value.to_string();
            let location = location
                .map(|v| format!("Location: {}\r\n", v.replace("{origin}", &mock_origin)))
                .unwrap_or_default();
            let response=format!("HTTP/1.1 {status} Response\r\n{location}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    });
    (origin, handle)
}
fn repo(id: i64, name: &str, stars: i64) -> Repository {
    Repository {
        id,
        full_name: name.into(),
        description: "A public project".into(),
        language: "Rust".into(),
        stars,
    }
}
fn wire(id: i64, name: &str, stars: i64) -> Value {
    json!({"id":id,"full_name":name,"stargazers_count":stars,"private":false,"visibility":"public","description":"A public project","language":"Rust"})
}
fn now() -> DateTime<Utc> {
    Utc::now() - Duration::minutes(1)
}

#[test]
fn github_config_enforces_small_unique_public_name_allowlist() {
    assert_eq!(
        sanitize_config(json!({"repositories":["Zed/Example","abc/repo.name"]})).unwrap(),
        json!({"repositories":["abc/repo.name","zed/example"]})
    );
    assert!(sanitize_config(json!({"repositories":["a/b","A/B"]})).is_err());
    for name in [
        "https://github.com/a/b",
        "a/b/c",
        "../repo",
        "a/..",
        "a/b?token=test-token",
        "a/%2e",
        "a/",
        "/b",
        "-owner/repo",
    ] {
        assert!(
            sanitize_config(json!({"repositories":[name]})).is_err(),
            "{name}"
        );
    }
    assert!(sanitize_config(json!({"repositories":[],"token":"test-token"})).is_err());
    assert!(configured_names(&json!({"repositories":[]})).is_err());
    assert!(sanitize_config(
        json!({"repositories":(0..31).map(|i|format!("owner/repo{i}")).collect::<Vec<_>>()})
    )
    .is_err());
}

#[tokio::test]
async fn github_network_auth_failure_private_repo_and_untrusted_redirect_are_sanitized() {
    let http = super::super::credentials::http_client().unwrap();
    let (api, server) = mock(vec![(
        "/rate_limit",
        200,
        None,
        json!({"resources":{"core":{"limit":5000,"remaining":4999}}}),
    )])
    .await;
    verify_at(&api, "test-token", &http).await.unwrap();
    server.await.unwrap();
    for (status, value) in [
        (401, json!({"message":"test-token"})),
        (
            200,
            json!({"resources":{"core":{"limit":60,"remaining":59}}}),
        ),
    ] {
        let (api, server) = mock(vec![("/rate_limit", status, None, value)]).await;
        let error = verify_at(&api, "test-token", &http).await.err().unwrap();
        assert!(!error.contains("test-token"));
        server.await.unwrap();
    }
    let (api, server) = mock(vec![(
        "/repos/owner/repo",
        301,
        Some("https://attacker.invalid/repositories/1".into()),
        Value::Null,
    )])
    .await;
    assert!(fetch(&api, "owner/repo", "test-token", &http)
        .await
        .is_err());
    server.await.unwrap();
    let (api, server) = mock(vec![
        (
            "/repos/owner/repo",
            301,
            Some("{origin}/repositories/42".into()),
            Value::Null,
        ),
        (
            "/repositories/42",
            200,
            None,
            wire(42, "renamed/project", 100),
        ),
    ])
    .await;
    let relocated = fetch(&api, "owner/repo", "test-token", &http)
        .await
        .unwrap();
    assert_eq!(relocated.id, 42);
    assert_eq!(relocated.full_name, "renamed/project");
    server.await.unwrap();
    let mut private = wire(1, "owner/repo", 1);
    private["private"] = json!(true);
    let (api, server) = mock(vec![("/repos/owner/repo", 200, None, private)]).await;
    assert!(fetch(&api, "owner/repo", "test-token", &http)
        .await
        .is_err());
    server.await.unwrap();
}

#[tokio::test]
async fn github_observation_needs_eight_boundaries_for_seven_days_and_never_fills_gaps(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let now = now();
    let mut repos = BTreeMap::new();
    repos.insert(1, repo(1, "owner/project", 100));
    save_snapshot(&h.pool, &repos[&1], now).await?;
    assert!(
        build_issue(&h.pool, &repos, now).await?.is_empty(),
        "first successful observation is only a baseline"
    );
    for ago in 1..=6 {
        save_snapshot(
            &h.pool,
            &repo(1, "owner/project", 100 - ago * 5),
            now - Duration::days(ago),
        )
        .await?;
    }
    let seven = build_issue(&h.pool, &repos, now).await?;
    assert_eq!(seven[0].data["observed_days"], 6);
    assert_eq!(seven[0].data["complete"], false);
    assert_eq!(seven[0].data["daily_gain"], json!([]));
    save_snapshot(
        &h.pool,
        &repo(1, "owner/project", 65),
        now - Duration::days(7),
    )
    .await?;
    let eight = build_issue(&h.pool, &repos, now).await?;
    assert_eq!(eight[0].data["observed_days"], 7);
    assert_eq!(eight[0].data["complete"], true);
    assert_eq!(eight[0].data["total_gain"], 35);
    assert_eq!(eight[0].data["daily_gain"], json!([5, 5, 5, 5, 5, 5, 5]));
    assert!(eight[0].data["wow_pct"].is_null());
    assert_eq!(
        eight[0].data["ranks"][0]["snapshot_times"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    // Same-day re-execution keeps the exact first observed time and star count.
    save_snapshot(
        &h.pool,
        &repo(1, "owner/project", 999),
        now + Duration::seconds(1),
    )
    .await?;
    let retry = build_issue(&h.pool, &repos, now + Duration::seconds(1)).await?;
    assert_eq!(retry[0].data["ranks"], eight[0].data["ranks"]);
    sqlx::query("DELETE FROM collection_github_snapshots WHERE repo_id=1 AND day=$1")
        .bind((now - Duration::days(3)).date_naive())
        .execute(&h.pool)
        .await?;
    let gap = build_issue(&h.pool, &repos, now).await?;
    assert_eq!(gap[0].data["observed_days"], 7);
    assert_eq!(gap[0].data["complete"], false);
    assert_eq!(gap[0].data["daily_gain"], json!([]));
    assert_eq!(
        gap[0].data["ranks"][0]["spark"].as_array().unwrap().len(),
        7
    );
    let mut forged = gap[0].data.clone();
    forged["complete"] = json!(true);
    assert!(validate_issue(forged).is_err());
    h.close().await?;
    Ok(())
}

#[tokio::test]
async fn github_rename_keeps_identity_and_partial_failures_preserve_only_real_progress(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let http = super::super::credentials::http_client()?;
    let at = now();
    let config = json!({"repositories":["owner/original"]});
    let run = h.run(&config).await;
    let (api, server) = mock(vec![(
        "/repos/owner/original",
        200,
        None,
        wire(42, "owner/original", 100),
    )])
    .await;
    assert!(collect_at(
        &h.pool,
        "test-token",
        &http,
        &config,
        run,
        &api,
        at - Duration::days(1)
    )
    .await
    .unwrap()
    .is_empty());
    server.await?;
    // A new canonical name for the same numeric ID retains the old baseline.
    let (api, server) = mock(vec![(
        "/repos/owner/original",
        200,
        None,
        wire(42, "new-owner/renamed", 110),
    )])
    .await;
    let output = collect_at(&h.pool, "test-token", &http, &config, run, &api, at)
        .await
        .unwrap();
    server.await?;
    assert_eq!(output[0].data["ranks"][0]["repo_id"], "42");
    assert_eq!(output[0].data["ranks"][0]["star_gain"], 10);
    assert_eq!(output[0].data["ranks"][0]["full_name"], "new-owner/renamed");
    crate::handlers::discover_editor::write_collected_batch(
        &h.pool,
        "github_repos",
        run,
        output.clone(),
    )
    .await?;
    let (visible,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM discover_items WHERE kind='repo_issue'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(
        visible, 0,
        "GitHub snapshots cannot publish without a human review"
    );
    sqlx::query("UPDATE collection_runs SET status='succeeded' WHERE id=$1")
        .bind(run)
        .execute(&h.pool)
        .await?;
    let manifest: Vec<(String, String, i64)> =
        sqlx::query_as("SELECT kind,key,revision FROM collection_run_items WHERE run_id=$1")
            .bind(run)
            .fetch_all(&h.pool)
            .await?;
    let review = manifest
        .into_iter()
        .map(
            |(kind, key, expected_revision)| crate::handlers::discover_editor::BatchReviewItem {
                kind,
                key,
                expected_revision,
            },
        )
        .collect();
    assert_eq!(
        crate::handlers::discover_editor::review_collected_batch(&h.pool, run, "test", review)
            .await?,
        1
    );
    let (visible,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM discover_items WHERE kind='repo_issue' AND status='published'",
    )
    .fetch_one(&h.pool)
    .await?;
    assert_eq!(visible, 1);
    let partial = json!({"repositories":["a/valid","b/deleted"]});
    let run = h.run(&partial).await;
    let (api, server) = mock(vec![
        ("/repos/a/valid", 200, None, wire(99, "a/valid", 5)),
        (
            "/repos/b/deleted",
            404,
            None,
            json!({"message":"test-token"}),
        ),
    ])
    .await;
    let error = collect_at(&h.pool, "test-token", &http, &partial, run, &api, at)
        .await
        .err()
        .unwrap();
    assert!(!error.contains("test-token"));
    server.await?;
    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM collection_github_snapshots WHERE repo_id=99")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(
        count, 1,
        "successful partial progress is kept without producing a partial candidate batch"
    );
    sqlx::query("UPDATE collection_sources SET config=$1 WHERE id='github_repos'")
        .bind(json!({"repositories":[]}))
        .execute(&h.pool)
        .await?;
    let (api, server) = mock(vec![("/repos/a/valid", 200, None, wire(99, "a/valid", 8))]).await;
    assert!(collect_at(
        &h.pool,
        "test-token",
        &http,
        &partial,
        run,
        &api,
        at + Duration::days(1)
    )
    .await
    .is_err());
    server.await?;
    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM collection_github_snapshots WHERE repo_id=99")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(
        count, 1,
        "changed configuration invalidates the old run snapshot"
    );
    h.close().await?;
    Ok(())
}

#[tokio::test]
async fn github_flags_negative_and_unusual_growth_without_manufacturing_daily_history(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = Harness::new().await? else {
        return Ok(());
    };
    let at = now();
    let mut repos = BTreeMap::new();
    for (id, start, end) in [(1, 100, 80), (2, 10000, 25000)] {
        let repository = repo(id, &format!("owner/repo{id}"), end);
        save_snapshot(
            &h.pool,
            &repo(id, &repository.full_name, start),
            at - Duration::days(2),
        )
        .await?;
        save_snapshot(&h.pool, &repository, at).await?;
        repos.insert(id, repository);
    }
    let output = build_issue(&h.pool, &repos, at).await?;
    let data = &output[0].data;
    assert_eq!(data["observed_days"], 2);
    assert_eq!(data["complete"], false);
    assert_eq!(data["total_gain"], 14980);
    assert_eq!(data["daily_gain"], json!([]));
    assert!(data["ranks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|rank| rank["suspected_growth"] == true));
    assert_eq!(data["ranks"][1]["star_gain"], -20);
    h.close().await?;
    Ok(())
}

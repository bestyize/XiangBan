//! 专家包：没有安装包的专家会被补上包，桌面端才能按技能的方式安装。
//! 需要 `TEST_DATABASE_URL`（有 CREATEDB 权限的账号）；未设置时跳过。
use server_xiaoban_market::{build_state, config::Config, importer, package, storage};

#[tokio::test]
async fn experts_without_a_package_get_one_and_stay_stable() {
    let Ok(admin_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("TEST_DATABASE_URL 未设置，跳过");
        return;
    };
    let db_name = format!("mkt_{}", uuid::Uuid::new_v4().simple());
    let admin = sqlx::PgPool::connect(&admin_url).await.unwrap();
    sqlx::query(&format!("CREATE DATABASE \"{db_name}\"")).execute(&admin).await.unwrap();
    let (prefix, _) = admin_url.rsplit_once('/').unwrap();
    let data = std::env::temp_dir().join(&db_name);
    let state = build_state(Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: format!("{prefix}/{db_name}"),
        jwt_secret: "test-secret-test-secret-test-secret".into(),
        admin_user: "admin".into(),
        admin_pass: Some("test-pass-123".into()),
        data_dir: data.clone(),
        web_dir: "does-not-exist".into(),
        allow_private_upstream: true,
        account_issuer: None,
        account_jwks_url: String::new(),
        account_url: String::new(),
        account_client_id: "xiaoban-market".into(),
    })
    .await
    .unwrap();
    let pool = &state.pool;

    let content = "---\nname: tech-test-automation\ndescription: 自动化测试\nversion: 1.0.0\n---\n# 流程\n";
    let expert: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO items (kind, namespace, slug, name, description, status, published_at) VALUES ('expert', 'skillhub', 'tech-test-automation', '自动化测试', $1, 'published', now()) RETURNING id",
    )
    .bind(content)
    .fetch_one(pool)
    .await
    .unwrap();
    // 专家团没有成员以外的正文，不该被补包。
    sqlx::query("INSERT INTO items (kind, namespace, slug, name, description, variant) VALUES ('expert', 'skillhub', 'a-team', '专家团', '说明', 'team')").execute(pool).await.unwrap();

    assert_eq!(importer::backfill_expert_packages(pool, &data).await.unwrap(), 1);
    assert_eq!(importer::backfill_expert_packages(pool, &data).await.unwrap(), 0, "幂等");

    let (version, package_sha, manifest_sha): (String, String, String) =
        sqlx::query_as("SELECT version, package_sha256, manifest_sha256 FROM versions WHERE item_id = $1").bind(expert).fetch_one(pool).await.unwrap();
    assert_eq!(version, "1.0.0");
    let latest: Option<String> = sqlx::query_scalar("SELECT latest_version FROM items WHERE id = $1").bind(expert).fetch_one(pool).await.unwrap();
    assert_eq!(latest.as_deref(), Some("1.0.0"));
    let bytes = std::fs::read(storage::package_path(&data, &package_sha)).unwrap();
    let info = package::inspect_zip(&bytes).unwrap();
    assert_eq!(info.manifest_sha256, manifest_sha);
    assert_eq!(info.skill_md, content);
    let teams: i64 = sqlx::query_scalar("SELECT count(*) FROM versions v JOIN items i ON i.id = v.item_id WHERE i.variant = 'team'").fetch_one(pool).await.unwrap();
    assert_eq!(teams, 0);

    pool.close().await;
    sqlx::query(&format!("DROP DATABASE \"{db_name}\" WITH (FORCE)")).execute(&admin).await.ok();
    std::fs::remove_dir_all(&data).ok();
}

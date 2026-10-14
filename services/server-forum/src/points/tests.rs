use super::*;
use crate::{auth::{AuthUser, JwksVerifier, MaybeUser}, config::AppConfig, handlers::{attachments, help}, users};
use axum::extract::Path;
use std::sync::Arc;

struct Fixture { state: AppState, url: String, schema: String }

impl Fixture {
    async fn new() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else { return Ok(None) };
        let admin = crate::db::connect(&url, None).await?;
        let schema = format!("points_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await?;
        sqlx::raw_sql("DEALLOCATE ALL").execute(&admin).await?;
        admin.close().await;
        use std::str::FromStr;
        let single = std::env::var("TEST_SINGLE_CONNECTION").as_deref() == Ok("1");
        let options = sqlx::postgres::PgConnectOptions::from_str(&url)?.options([("search_path", &schema)]);
        let pool = sqlx::postgres::PgPoolOptions::new().max_connections(if single { 1 } else { 10 }).connect_with(options).await?;
        sqlx::raw_sql(&format!("SET search_path TO {schema}")).execute(&pool).await?;
        crate::db::migrate(&pool).await?;
        crate::db::migrate(&pool).await?;
        crate::db::seed(&pool).await?;
        let mut config = AppConfig::from_env();
        config.upload_dir = std::env::temp_dir().join(&schema);
        tokio::fs::create_dir_all(&config.upload_dir).await?;
        let state = AppState { pool, config: Arc::new(config),
            verifier: Arc::new(JwksVerifier::new("test".into(), "test".into(), "test".into())),
            http: reqwest::Client::new() };
        Ok(Some(Self { state, url, schema }))
    }
    async fn user(&self) -> AppResult<Uuid> {
        let uid = Uuid::new_v4();
        sqlx::query("INSERT INTO users (uid, nickname, trust_level) VALUES ($1, 'test', 1)")
            .bind(uid).execute(&self.state.pool).await?;
        on_visit(&self.state.pool, uid, None).await?;
        Ok(uid)
    }
    async fn auth(&self, uid: Uuid) -> AppResult<AuthUser> {
        Ok(AuthUser { user: users::find(&self.state.pool, uid).await?.unwrap(), token: "test".into() })
    }
    async fn balance(&self, uid: Uuid) -> AppResult<i32> {
        Ok(users::find(&self.state.pool, uid).await?.unwrap().points)
    }
    async fn topic(&self, author: Uuid, slug: &str) -> AppResult<i64> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO topics (author_uid, category_id, title, body_md, body_html)
             VALUES ($1, (SELECT id FROM categories WHERE slug = $2), 'test topic', 'body', 'body') RETURNING id",
        ).bind(author).bind(slug).fetch_one(&self.state.pool).await?;
        Ok(id)
    }
    async fn finish(self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::fs::remove_dir_all(&self.state.config.upload_dir).await?;
        sqlx::raw_sql("DEALLOCATE ALL").execute(&self.state.pool).await?;
        self.state.pool.close().await;
        let admin = crate::db::connect(&self.url, None).await?;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema)).execute(&admin).await?;
        sqlx::raw_sql("DEALLOCATE ALL").execute(&admin).await?;
        admin.close().await;
        Ok(())
    }
}

#[tokio::test]
async fn rewards_and_invites_are_once_only_and_daily_capped() -> Result<(), Box<dyn std::error::Error>> {
    let Some(f) = Fixture::new().await? else { return Ok(()) };
    let inviter = f.user().await?;
    assert_eq!(f.balance(inviter).await?, 51);
    on_visit(&f.state.pool, inviter, None).await?;
    assert_eq!(f.balance(inviter).await?, 51);
    for n in 0..4 {
        let uid = Uuid::new_v4();
        sqlx::query("INSERT INTO users (uid, nickname) VALUES ($1, 'new user')").bind(uid).execute(&f.state.pool).await?;
        on_visit(&f.state.pool, uid, Some(inviter.simple().to_string())).await?;
        let first = check_in(State(f.state.clone()), f.auth(uid).await?).await?.0;
        assert_eq!(first["earned"], 1);
        assert_eq!(f.balance(uid).await?, 52);
        assert_eq!(f.balance(inviter).await?, 51 + 30 * (n + 1).min(3));
        let second = check_in(State(f.state.clone()), f.auth(uid).await?).await?.0;
        assert_eq!(second["earned"], 0);
        let (status,): (String,) = sqlx::query_as("SELECT invite_status FROM forum_registrations WHERE uid = $1")
            .bind(uid).fetch_one(&f.state.pool).await?;
        assert_eq!(status, if n < 3 { "rewarded" } else { "capped" });
    }
    let (sum,): (i64,) = sqlx::query_as("SELECT SUM(delta) FROM points_ledger WHERE uid = $1")
        .bind(inviter).fetch_one(&f.state.pool).await?;
    assert_eq!(sum, f.balance(inviter).await? as i64);
    f.finish().await
}

#[tokio::test]
async fn purchases_are_atomic_idempotent_and_keep_read_permissions() -> Result<(), Box<dyn std::error::Error>> {
    let Some(f) = Fixture::new().await? else { return Ok(()) };
    let buyer = f.user().await?;
    let seller = f.user().await?;
    let topic = f.topic(seller, "share-learn").await?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO attachments (id, topic_id, uploader_uid, filename, size, price) VALUES ($1, $2, $3, 'test.txt', 4, 10)")
        .bind(id).bind(topic).bind(seller).execute(&f.state.pool).await?;
    tokio::fs::write(f.state.config.upload_dir.join(id.to_string()), b"test").await?;
    assert!(attachments::download(State(f.state.clone()), MaybeUser(None), Path(id)).await.is_err());
    assert!(attachments::purchase(State(f.state.clone()), f.auth(buyer).await?, Path(id), Json(attachments::PurchaseInput { price: 9 })).await.is_err());
    assert_eq!(f.balance(buyer).await?, 51);
    let one = f.auth(buyer).await?;
    let two = f.auth(buyer).await?;
    let (a,b) = tokio::join!(
        attachments::purchase(State(f.state.clone()), one, Path(id), Json(attachments::PurchaseInput { price: 10 })),
        attachments::purchase(State(f.state.clone()), two, Path(id), Json(attachments::PurchaseInput { price: 10 })),
    );
    assert_eq!(a?.0["charged"].as_i64().unwrap() + b?.0["charged"].as_i64().unwrap(), 10);
    assert_eq!(f.balance(buyer).await?, 41);
    assert_eq!(f.balance(seller).await?, 61);
    let reader = users::find(&f.state.pool, buyer).await?;
    assert!(attachments::download(State(f.state.clone()), MaybeUser(reader), Path(id)).await.is_ok());
    sqlx::query("UPDATE topics SET min_read_level = 3 WHERE id = $1").bind(topic).execute(&f.state.pool).await?;
    let reader = users::find(&f.state.pool, buyer).await?;
    assert!(attachments::download(State(f.state.clone()), MaybeUser(reader), Path(id)).await.is_err());
    sqlx::query("UPDATE topics SET min_read_level = 0 WHERE id = $1").bind(topic).execute(&f.state.pool).await?;
    sqlx::query("UPDATE attachments SET price = 5 WHERE id = $1").bind(id).execute(&f.state.pool).await?;
    assert_eq!(attachments::purchase(State(f.state.clone()), f.auth(buyer).await?, Path(id), Json(attachments::PurchaseInput { price: 10 })).await?.0["charged"], 0);
    assert_eq!(attachments::purchase(State(f.state.clone()), f.auth(seller).await?, Path(id), Json(attachments::PurchaseInput { price: 5 })).await?.0["charged"], 0);
    let broke = f.user().await?;
    sqlx::query("UPDATE users SET points = 0 WHERE uid = $1").bind(broke).execute(&f.state.pool).await?;
    assert!(attachments::purchase(State(f.state.clone()), f.auth(broke).await?, Path(id), Json(attachments::PurchaseInput { price: 5 })).await.is_err());
    assert_eq!(f.balance(seller).await?, 61);
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM attachment_purchases WHERE attachment_id = $1").bind(id).fetch_one(&f.state.pool).await?;
    assert_eq!(count, 1);
    f.finish().await
}

#[tokio::test]
async fn bounties_settle_once_or_refund_and_never_both() -> Result<(), Box<dyn std::error::Error>> {
    let Some(f) = Fixture::new().await? else { return Ok(()) };
    let owner = f.user().await?;
    let answerer = f.user().await?;
    let topic = f.topic(owner, "share-help").await?;
    let (category,): (i32,) = sqlx::query_as("SELECT category_id FROM topics WHERE id = $1").bind(topic).fetch_one(&f.state.pool).await?;
    let mut tx = f.state.pool.begin().await?;
    help::create_for_topic(&mut tx, topic, owner, category, Some(&help::HelpInput { bounty: 20, days: 7 })).await?;
    tx.commit().await?;
    assert_eq!(f.balance(owner).await?, 31);
    let listing = crate::models::list_topics(&f.state.pool, &crate::models::ListFilter {
        category: Some("share-help".into()), help_status: Some("bounty".into()), ..Default::default()
    }).await?;
    assert_eq!(listing.total, 1);
    assert_eq!(listing.items[0].help.as_ref().unwrap().bounty, 20);
    let (post,): (i64,) = sqlx::query_as("INSERT INTO posts (topic_id, author_uid, floor, body_md, body_html) VALUES ($1,$2,1,'answer','answer') RETURNING id")
        .bind(topic).bind(answerer).fetch_one(&f.state.pool).await?;
    assert!(help::accept(State(f.state.clone()), f.auth(answerer).await?, Path(topic), Json(help::AcceptInput { post_id: post })).await.is_err());
    let a = f.auth(owner).await?;
    let b = f.auth(owner).await?;
    let (accept, close) = tokio::join!(
        help::accept(State(f.state.clone()), a, Path(topic), Json(help::AcceptInput { post_id: post })),
        help::close(State(f.state.clone()), b, Path(topic)),
    );
    let (status,): (String,) = sqlx::query_as("SELECT status FROM help_requests WHERE topic_id=$1").bind(topic).fetch_one(&f.state.pool).await?;
    if status == "solved" {
        assert!(accept.is_ok()); assert!(close.is_err());
        assert_eq!(f.balance(owner).await?, 31); assert_eq!(f.balance(answerer).await?, 71);
    } else {
        assert!(accept.is_err()); assert!(close.is_ok());
        assert_eq!(status, "closed"); assert_eq!(f.balance(owner).await?, 51); assert_eq!(f.balance(answerer).await?, 51);
    }
    assert_eq!(f.balance(owner).await? + f.balance(answerer).await?, 102);
    let expired = f.topic(owner, "share-help").await?;
    let mut tx = f.state.pool.begin().await?;
    help::create_for_topic(&mut tx, expired, owner, category, Some(&help::HelpInput { bounty: 10, days: 7 })).await?;
    tx.commit().await?;
    let after_escrow = f.balance(owner).await?;
    sqlx::query("UPDATE help_requests SET expires_at = now() - interval '1 second' WHERE topic_id=$1").bind(expired).execute(&f.state.pool).await?;
    help::expire(&f.state.pool).await?;
    assert_eq!(f.balance(owner).await?, after_escrow + 10);
    help::expire(&f.state.pool).await?;
    assert_eq!(f.balance(owner).await?, after_escrow + 10);
    f.finish().await
}

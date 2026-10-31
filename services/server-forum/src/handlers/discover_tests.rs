use crate::{auth::JwksVerifier, config::AppConfig, state::AppState};
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn call(app: &axum::Router, method: &str, path: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(path).header("content-type", "application/json");
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    let response = app.clone().oneshot(req.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

struct Harness {
    app: axum::Router,
    pool: sqlx::PgPool,
    admin_pool: sqlx::PgPool,
    schema: String,
    admin_t: String,
    make_token: Box<dyn Fn(Uuid) -> String>,
}

async fn harness() -> Result<Option<Harness>, Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return Ok(None);
    };
    let schema = format!("discover_test_{}", Uuid::new_v4().simple());
    let admin_pool = crate::db::connect(&url, None).await?;
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin_pool).await?;
    let pool = crate::db::connect(&url, Some(&schema)).await?;
    crate::db::migrate(&pool).await?;
    crate::db::migrate(&pool).await?;
    crate::db::seed(&pool).await?;
    crate::discover::seed(&pool).await?;
    crate::discover::seed(&pool).await?;

    let mut config = AppConfig::from_env();
    config.admin_user = Some("root".into());
    config.admin_password = Some("correct-horse-battery".into());
    crate::handlers::admin::seed_admin(&pool, &config).await?;

    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
    let x = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
    let verifier = JwksVerifier::with_static_key("iss".into(), "aud".into(), DecodingKey::from_ed_components(&x).unwrap());
    let enc = EncodingKey::from_ed_der(pkcs8.as_ref());
    let make_token = Box::new(move |uid: Uuid| {
        #[derive(serde::Serialize)]
        struct C {
            iss: String,
            sub: String,
            aud: String,
            exp: i64,
        }
        let c = C { iss: "iss".into(), sub: uid.to_string(), aud: "aud".into(), exp: chrono::Utc::now().timestamp() + 600 };
        encode(&Header::new(Algorithm::EdDSA), &c, &enc).unwrap()
    });

    let state = AppState { pool: pool.clone(), config: Arc::new(config), verifier: Arc::new(verifier), http: reqwest::Client::new() };
    let app = crate::build_router(state);
    let (s, login) = call(&app, "POST", "/api/admin/login", None, json!({"username":"root","password":"correct-horse-battery"})).await;
    assert_eq!(s, StatusCode::OK, "{login}");
    let admin_t = login["token"].as_str().unwrap().to_string();
    Ok(Some(Harness { app, pool, admin_pool, schema, admin_t, make_token }))
}

impl Harness {
    async fn finish(self) -> Result<(), Box<dyn std::error::Error>> {
        drop(self.pool);
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema)).execute(&self.admin_pool).await?;
        Ok(())
    }

    async fn member(&self) -> Result<String, Box<dyn std::error::Error>> {
        let uid = Uuid::new_v4();
        sqlx::query("INSERT INTO users (uid, nickname, trust_level) VALUES ($1, '测试用户', 1)").bind(uid).execute(&self.pool).await?;
        Ok((self.make_token)(uid))
    }
}

#[tokio::test]
async fn discover_pages_read_seeded_and_admin_edited_items() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let app = &h.app;

    // 种子只写一次，重复启动不会翻倍。
    let (s, home) = call(app, "GET", "/api/discover/home", None, Value::Null).await;
    assert_eq!(s, StatusCode::OK, "{home}");
    assert_eq!(home["banners"].as_array().unwrap().len(), 1);
    assert_eq!(home["news"]["items"].as_array().unwrap().len(), 3);
    assert_eq!(home["news"]["total"], 3);
    assert_eq!(home["repos"]["ranks"][0]["rank"], 1);
    assert_eq!(home["sample"], true);

    // 导航按分组返回，且是真实内容（不带示例标记）。
    let (_, nav) = call(app, "GET", "/api/discover/nav", None, Value::Null).await;
    assert_eq!(nav["groups"][0]["name"], "海外");
    assert_eq!(nav["groups"][1]["name"], "国产");
    assert_eq!(nav["groups"][0]["items"][0]["sample"], false);

    // 模型榜：按指标分档由后端给出。
    let (_, board) = call(app, "GET", "/api/discover/models", None, Value::Null).await;
    assert_eq!(board["models"][0]["tier"], "S");
    assert_eq!(board["models"][3]["tier"], "C");
    assert_eq!(board["tiers"][0]["id"], "S");
    assert_eq!(board["tiers"][3]["min"], Value::Null);
    assert_eq!(board["sample"], true);

    // 追踪：额度重置、上新、限时免费；进度由服务端算。
    let (_, tr) = call(app, "GET", "/api/discover/tracker", None, Value::Null).await;
    assert_eq!(tr["quota"]["progress_pct"], 68);
    assert_eq!(tr["launches"].as_array().unwrap().len(), 2);
    let pct = tr["promos"][0]["pct"].as_u64().unwrap();
    assert!((50..=65).contains(&pct), "{pct}");
    assert_eq!(tr["promos"][0]["days_left"], 3);

    // 工具：列表有最新版本，详情带安装包和讨论数。
    let (_, tools) = call(app, "GET", "/api/tools", None, Value::Null).await;
    assert_eq!(tools["items"][0]["key"], "cc-switch");
    assert_eq!(tools["items"][0]["latest_version"], "v0.0.0");
    let (s, tool) = call(app, "GET", "/api/tools/cc-switch", None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(tool["artifacts"].as_array().unwrap().len(), 3);
    assert_eq!(tool["discussion_count"], 0);
    let (s, _) = call(app, "GET", "/api/tools/nope", None, Value::Null).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // 后台写入：没登录和普通用户都不行。
    let put = json!({ "data": { "title": "新日报", "source": "来源", "url": "https://example.com/a" }, "sample": false });
    let (s, _) = call(app, "PUT", "/api/admin/discover/news/today-1", None, put.clone()).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let member = h.member().await?;
    let (s, _) = call(app, "PUT", "/api/admin/discover/news/today-1", Some(&member), put.clone()).await;
    assert!(s == StatusCode::UNAUTHORIZED || s == StatusCode::FORBIDDEN, "{s}");

    // 非法内容、非法 key、未知类型。
    let bad_url = json!({ "data": { "title": "x", "source": "s", "url": "javascript:alert(1)" } });
    let (s, _) = call(app, "PUT", "/api/admin/discover/news/bad", Some(&h.admin_t), bad_url).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = call(app, "PUT", "/api/admin/discover/news/Bad%20Key", Some(&h.admin_t), put.clone()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = call(app, "PUT", "/api/admin/discover/nope/a", Some(&h.admin_t), put.clone()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // 新增后出现在日报最前面；草稿不公开；删除后消失。
    let (s, _) = call(app, "PUT", "/api/admin/discover/news/today-1", Some(&h.admin_t), put).await;
    assert_eq!(s, StatusCode::OK);
    let (_, daily) = call(app, "GET", "/api/discover/daily?page_size=2", None, Value::Null).await;
    assert_eq!(daily["items"][0]["key"], "today-1");
    assert_eq!(daily["total"], 4);
    assert_eq!(daily["items"].as_array().unwrap().len(), 2);
    let draft = json!({ "data": { "title": "草稿", "source": "s", "url": "https://example.com/d" }, "status": "draft" });
    call(app, "PUT", "/api/admin/discover/news/draft-1", Some(&h.admin_t), draft).await;
    let (_, daily) = call(app, "GET", "/api/discover/daily", None, Value::Null).await;
    assert_eq!(daily["total"], 4);
    let (_, listed) = call(app, "GET", "/api/admin/discover/news", Some(&h.admin_t), Value::Null).await;
    assert_eq!(listed["items"].as_array().unwrap().len(), 5);
    let (s, _) = call(app, "DELETE", "/api/admin/discover/news/today-1", Some(&h.admin_t), Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = call(app, "DELETE", "/api/admin/discover/news/today-1", Some(&h.admin_t), Value::Null).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // 开源热榜：多期时默认最新一期，可按 key 取往期。
    let issue2 = json!({ "data": { "issue_no": 2, "week_start": "10.30", "week_end": "11.05", "total_gain": 5,
        "ranks": [{ "full_name": "x/low", "star_total": 1, "star_gain": 1 }, { "full_name": "y/high", "star_total": 2, "star_gain": 9 }] } });
    call(app, "PUT", "/api/admin/discover/repo_issue/2", Some(&h.admin_t), issue2).await;
    let (_, latest) = call(app, "GET", "/api/discover/repos", None, Value::Null).await;
    assert_eq!(latest["issue_no"], 2);
    assert_eq!(latest["ranks"][0]["full_name"], "y/high");
    let (_, old) = call(app, "GET", "/api/discover/repos?issue=1", None, Value::Null).await;
    assert_eq!(old["issue_no"], 1);
    let (_, issues) = call(app, "GET", "/api/discover/repos/issues", None, Value::Null).await;
    assert_eq!(issues["items"][0]["issue_no"], 2);
    let (s, _) = call(app, "GET", "/api/discover/repos?issue=99", None, Value::Null).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    h.finish().await
}

#[tokio::test]
async fn topics_carry_images_polls_and_help_stats() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let app = &h.app;
    let alice = h.member().await?;
    let bob = h.member().await?;
    let cat = |slug: &'static str| {
        let pool = h.pool.clone();
        async move { sqlx::query_as::<_, (i32,)>("SELECT id FROM categories WHERE slug = $1").bind(slug).fetch_one(&pool).await.map(|r| r.0) }
    };
    let (fun, work, help) = (cat("casual-fun").await?, cat("casual-work").await?, cat("share-help").await?);

    // 带三宫格图片和投票的休闲帖。
    let imgs: Vec<String> = (0..4).map(|_| format!("/api/images/{}", Uuid::new_v4())).collect();
    let body = format!("晚霞很好看，大家怎么看这件事呢\n\n{}", imgs.iter().map(|i| format!("![]({i})")).collect::<Vec<_>>().join("\n"));
    let topic = json!({ "category_id": fun, "title": "今天几点下班呀", "body": body,
        "poll": { "options": ["学一门新手艺", "多陪家人", "继续工作"], "days": 3 } });
    let (s, created) = call(app, "POST", "/api/topics", Some(&alice), topic).await;
    assert_eq!(s, StatusCode::OK, "{created}");
    let id = created["id"].as_i64().unwrap();

    let (s, _) = call(app, "POST", "/api/topics", Some(&alice), json!({ "category_id": work, "title": "单选项投票", "body": "只有一个选项的投票应该被拒绝",
        "poll": { "options": ["只有一个"] } })).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = call(app, "POST", "/api/topics", Some(&alice), json!({ "category_id": work, "title": "重复选项投票", "body": "重复的选项应该被拒绝的",
        "poll": { "options": ["同一个", "同一个"] } })).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = call(app, "POST", "/api/topics", Some(&alice), json!({ "category_id": work, "title": "普通职场帖", "body": "一个没有投票也没有图片的帖子" })).await;
    assert_eq!(s, StatusCode::OK);

    // 列表：配图最多 3 张，投票带票数；没登录时没有 my_vote。
    let (_, list) = call(app, "GET", "/api/topics?category=casual", None, Value::Null).await;
    assert_eq!(list["total"], 2);
    let item = list["items"].as_array().unwrap().iter().find(|t| t["id"] == id).unwrap().clone();
    assert_eq!(item["images"].as_array().unwrap().len(), 3);
    assert_eq!(item["images"][0], json!(imgs[0]));
    assert_eq!(item["poll"]["total"], 0);
    assert_eq!(item["poll"]["options"][1]["text"], "多陪家人");
    assert_eq!(item["poll"]["my_vote"], Value::Null);

    // 多个分类用逗号分隔。
    let (_, both) = call(app, "GET", "/api/topics?category=casual-fun,casual-work", None, Value::Null).await;
    assert_eq!(both["total"], 2);
    let (_, one) = call(app, "GET", "/api/topics?category=casual-fun", None, Value::Null).await;
    assert_eq!(one["total"], 1);

    // 投票：要登录、选项要存在、每人一次。
    let vote = |option: i64| json!({ "option": option });
    let path = format!("/api/topics/{id}/poll/vote");
    let (s, _) = call(app, "PUT", &path, None, vote(0)).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, _) = call(app, "PUT", &path, Some(&bob), vote(7)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, voted) = call(app, "PUT", &path, Some(&bob), vote(1)).await;
    assert_eq!(s, StatusCode::OK, "{voted}");
    assert_eq!(voted["poll"]["options"][1]["votes"], 1);
    assert_eq!(voted["poll"]["my_vote"], 1);
    let (s, _) = call(app, "PUT", &path, Some(&bob), vote(0)).await;
    assert_eq!(s, StatusCode::CONFLICT);
    let (s, _) = call(app, "PUT", &path, Some(&alice), vote(1)).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = call(app, "PUT", "/api/topics/999999/poll/vote", Some(&bob), vote(0)).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // 详情和列表都带当前读者的投票。
    let (_, detail) = call(app, "GET", &format!("/api/topics/{id}"), Some(&bob), Value::Null).await;
    assert_eq!(detail["poll"]["total"], 2);
    assert_eq!(detail["poll"]["my_vote"], 1);
    let (_, list) = call(app, "GET", "/api/topics?category=casual-fun", Some(&alice), Value::Null).await;
    assert_eq!(list["items"][0]["poll"]["my_vote"], 1);

    // 投票结束后不能再投。
    sqlx::query("UPDATE polls SET closes_at = now() - interval '1 hour' WHERE topic_id = $1").bind(id).execute(&h.pool).await?;
    let carol = h.member().await?;
    let (s, _) = call(app, "PUT", &path, Some(&carol), vote(0)).await;
    assert_eq!(s, StatusCode::CONFLICT);

    // 求助统计：进行中的求助、悬赏合计，以及本周已解决。
    let (s, _) = call(app, "POST", "/api/topics", Some(&alice), json!({ "category_id": help, "title": "想要一个番茄钟组件", "body": "左边番茄钟，右边今天的待办。" })).await;
    assert_eq!(s, StatusCode::OK);
    let (_, stats) = call(app, "GET", "/api/help/stats", None, Value::Null).await;
    assert_eq!(stats["open"], 1);
    assert_eq!(stats["solved_this_week"], 0);
    assert_eq!(stats["bounty_points"], 0);

    h.finish().await
}

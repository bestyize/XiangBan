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

#[path = "discover_editor_tests.rs"]
mod workflow;

#[path = "model_rankings_tests.rs"]
mod model_rankings;

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
async fn news_article_and_comments_are_published_scoped_and_moderated() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let payload = json!({"data":{"title":"站内文章","source":"官方","url":"https://example.com/news",
        "summary":"摘要","body":"## 小标题\n\n正文 ![配图](https://example.com/image.png)\n\n<script>alert(1)</script>"}});
    assert_eq!(call(&h.app, "PUT", "/api/admin/discover/news/article-test", Some(&h.admin_t), payload).await.0, StatusCode::OK);
    let (status, detail) = call(&h.app, "GET", "/api/discover/daily/article-test", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let html = detail["body_html"].as_str().unwrap();
    assert!(html.contains("<h2>") && html.contains("<img") && !html.contains("<script"), "{html}");
    let (_, listing) = call(&h.app, "GET", "/api/discover/daily", None, Value::Null).await;
    assert!(listing["items"].as_array().unwrap().iter().all(|v| v.get("body").is_none()));

    let path = "/api/discover/daily/article-test/comments";
    assert_eq!(call(&h.app, "POST", path, None, json!({"body":"留言"})).await.0, StatusCode::UNAUTHORIZED);
    let author = h.member().await?;
    let other = h.member().await?;
    let (status, created) = call(&h.app, "POST", path, Some(&author), json!({"body":"第一条留言"})).await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let id = created["id"].as_i64().unwrap();
    assert_eq!(call(&h.app, "POST", path, Some(&author), json!({"body":"太快了"})).await.0, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(call(&h.app, "POST", path, Some(&other), json!({"body":"回复留言","reply_to":id})).await.0, StatusCode::OK);
    assert_eq!(call(&h.app, "PUT", &format!("{path}/{id}/like"), Some(&other), json!({})).await.0, StatusCode::OK);
    let (_, comments) = call(&h.app, "GET", path, Some(&other), Value::Null).await;
    assert_eq!(comments["total"], 2);
    assert_eq!(comments["items"][1]["like_count"], 1);
    assert_eq!(comments["items"][1]["liked"], true);
    assert_eq!(comments["items"][0]["reply_to"], id);
    assert_eq!(call(&h.app, "DELETE", &format!("{path}/{id}"), Some(&other), json!({})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app, "DELETE", &format!("{path}/{id}"), Some(&author), json!({})).await.0, StatusCode::OK);
    sqlx::query("UPDATE discover_items SET status='draft' WHERE kind='news' AND key='article-test'").execute(&h.pool).await?;
    assert_eq!(call(&h.app, "GET", path, None, Value::Null).await.0, StatusCode::NOT_FOUND);
    h.finish().await
}

#[tokio::test]
async fn discover_pages_read_seeded_and_admin_edited_items() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let app = &h.app;

    // 种子只写一次，重复启动不会翻倍。
    let (s, home) = call(app, "GET", "/api/discover/home", None, Value::Null).await;
    assert_eq!(s, StatusCode::OK, "{home}");
    assert_eq!(home["banners"].as_array().unwrap().len(), 1);
    assert_eq!(home["news"]["items"].as_array().unwrap().len(), 5);
    let day_total = home["news"]["total"].as_i64().unwrap();
    assert!((1..=5).contains(&day_total), "{day_total}");
    assert_eq!(home["repos"]["issue_no"], 42);
    assert_eq!(home["repos"]["ranks"].as_array().unwrap().len(), 3, "首页只放热榜前三");
    assert_eq!(home["repos"]["ranks"][0]["rank"], 1);
    assert_eq!(home["repos"]["ranks"][0]["icon"], "term");
    assert_eq!(home["sample"], true);

    // 导航按分组返回，且是真实内容（不带示例标记）。
    let (_, nav) = call(app, "GET", "/api/discover/nav", None, Value::Null).await;
    assert_eq!(nav["groups"][0]["name"], "海外");
    assert_eq!(nav["groups"][1]["name"], "国产");
    assert_eq!(nav["groups"][0]["items"][0]["sample"], false);

    // 模型榜：按指标分档由后端给出。
    let (_, board) = call(app, "GET", "/api/discover/models", None, Value::Null).await;
    assert_eq!(board["vendors"].as_array().unwrap().len(), 6);
    assert_eq!(board["models"].as_array().unwrap().len(), 12);
    assert_eq!(board["models"][0]["name"], "模型 A");
    assert_eq!(board["models"][0]["tier"], "S");
    assert_eq!(board["models"][2]["tier"], "S");
    assert_eq!(board["models"][3]["tier"], "A");
    assert_eq!(board["models"][11]["tier"], "C");
    assert_eq!(board["tiers"][0]["id"], "S");
    assert_eq!(board["tiers"][3]["min"], Value::Null);
    assert_eq!(board["sample"], true);

    // 追踪：额度重置、上新、限时免费；进度由服务端算。
    let (_, tr) = call(app, "GET", "/api/discover/tracker", None, Value::Null).await;
    assert_eq!(tr["quota"]["progress_pct"], 68);
    assert_eq!(tr["launches"].as_array().unwrap().len(), 3);
    assert_eq!(tr["promos"].as_array().unwrap().len(), 2);
    let pct = tr["promos"][0]["pct"].as_u64().unwrap();
    assert!((50..=65).contains(&pct), "{pct}");
    assert_eq!(tr["promos"][0]["days_left"], 3);

    // 工具：列表有最新版本，详情带安装包和讨论数。
    let (_, tools) = call(app, "GET", "/api/tools", None, Value::Null).await;
    assert_eq!(tools["items"].as_array().unwrap().len(), 5);
    let cc = tools["items"].as_array().unwrap().iter().find(|t| t["key"] == "cc-switch").unwrap();
    assert_eq!(cc["latest_version"], "v3.5.0");
    assert_eq!(cc["license"], "open");
    let unknown = tools["items"].as_array().unwrap().iter().find(|t| t["key"] == "deepseek-harness").unwrap();
    assert_eq!(unknown["license"], "unknown");
    assert_eq!(unknown["latest_version"], Value::Null);
    let (s, tool) = call(app, "GET", "/api/tools/cc-switch", None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(tool["artifacts"].as_array().unwrap().len(), 3);
    assert_eq!(tool["releases"].as_array().unwrap().len(), 3);
    assert_eq!(tool["releases"][0]["items"][1]["kind"], "improve");
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

    // 热门搜索来自后台词表。
    let (_, hot) = call(app, "GET", "/api/discover/hot-search", None, Value::Null).await;
    assert_eq!(hot["items"].as_array().unwrap().len(), 8);
    assert_eq!(hot["items"][0]["text"], "桌面特效");
    assert_eq!(hot["items"][0]["tag"], "热");

    // 新增后出现在日报最前面；草稿不公开；删除后消失。
    let (s, _) = call(app, "PUT", "/api/admin/discover/news/today-1", Some(&h.admin_t), put).await;
    assert_eq!(s, StatusCode::OK);
    let (_, daily) = call(app, "GET", "/api/discover/daily?page_size=2", None, Value::Null).await;
    assert_eq!(daily["items"][0]["key"], "today-1");
    assert_eq!(daily["total"], 6);
    assert_eq!(daily["items"].as_array().unwrap().len(), 2);
    let draft = json!({ "data": { "title": "草稿", "source": "s", "url": "https://example.com/d" }, "status": "draft" });
    call(app, "PUT", "/api/admin/discover/news/draft-1", Some(&h.admin_t), draft).await;
    let (_, daily) = call(app, "GET", "/api/discover/daily", None, Value::Null).await;
    assert_eq!(daily["total"], 6);
    let (_, listed) = call(app, "GET", "/api/admin/discover/news", Some(&h.admin_t), Value::Null).await;
    assert_eq!(listed["items"].as_array().unwrap().len(), 7);
    let (s, _) = call(app, "DELETE", "/api/admin/discover/news/today-1", Some(&h.admin_t), Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = call(app, "DELETE", "/api/admin/discover/news/today-1", Some(&h.admin_t), Value::Null).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // 开源热榜：多期时默认最新一期，可按 key 取往期。
    let issue2 = json!({ "data": { "issue_no": 43, "week_start": "10.30", "week_end": "11.05", "total_gain": 5,
        "ranks": [{ "full_name": "x/low", "star_total": 1, "star_gain": 1 }, { "full_name": "y/high", "star_total": 2, "star_gain": 9 }] } });
    call(app, "PUT", "/api/admin/discover/repo_issue/43", Some(&h.admin_t), issue2).await;
    let (_, latest) = call(app, "GET", "/api/discover/repos", None, Value::Null).await;
    assert_eq!(latest["issue_no"], 43);
    assert_eq!(latest["ranks"][0]["full_name"], "y/high");
    let (_, old) = call(app, "GET", "/api/discover/repos?issue=42", None, Value::Null).await;
    assert_eq!(old["issue_no"], 42);
    let (_, issues) = call(app, "GET", "/api/discover/repos/issues", None, Value::Null).await;
    assert_eq!(issues["items"][0]["issue_no"], 43);
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

    // 休闲板块有设计稿里的全部筛选项。
    let (_, cats) = call(app, "GET", "/api/categories", None, Value::Null).await;
    for slug in ["casual-work", "casual-offwork", "casual-indie", "casual-gear", "casual-loss", "casual-fun", "casual-media", "casual-think", "casual-help", "casual-chat"] {
        assert!(cats.as_array().unwrap().iter().any(|c| c["slug"] == slug), "缺少 {slug}");
    }

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

/// 线上从上一版种子升级：旧示例被清掉换成新的，后台录入的真实内容和已有的导航保留。
#[tokio::test]
async fn seed_upgrade_replaces_old_samples_but_keeps_real_items() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    sqlx::query("DELETE FROM data_migrations WHERE name = 'discover-seed-v2'").execute(&h.pool).await?;
    sqlx::query("INSERT INTO discover_items (kind, key, data) VALUES ('model', 'old-sample', '{\"name\":\"旧示例\",\"vendor\":\"x\",\"iq\":50,\"price\":1,\"speed\":1,\"sample\":true}')")
        .execute(&h.pool).await?;
    sqlx::query("INSERT INTO discover_items (kind, key, data) VALUES ('model', 'real-one', '{\"name\":\"真实模型\",\"vendor\":\"claude\",\"iq\":80,\"price\":2,\"speed\":10,\"sample\":false}')")
        .execute(&h.pool).await?;
    sqlx::query("UPDATE discover_items SET data = jsonb_set(data, '{vendor}', '\"claude\"') WHERE kind = 'model' AND key = 'a'").execute(&h.pool).await?;
    sqlx::query("DELETE FROM discover_items WHERE kind = 'doc' AND key = 'claude-skills'").execute(&h.pool).await?;
    sqlx::query("UPDATE discover_items SET data = jsonb_set(data, '{title}', '\"Claude Code 文档\"') WHERE kind = 'doc' AND key = 'claude-code'").execute(&h.pool).await?;

    crate::discover::seed(&h.pool).await?;

    let keys: Vec<(String,)> = sqlx::query_as("SELECT key FROM discover_items WHERE kind = 'model' ORDER BY key").fetch_all(&h.pool).await?;
    let keys: Vec<String> = keys.into_iter().map(|k| k.0).collect();
    assert!(!keys.contains(&"old-sample".to_string()), "{keys:?}");
    assert!(keys.contains(&"real-one".to_string()), "{keys:?}");
    assert_eq!(keys.len(), 13, "12 个新示例 + 1 个真实条目");
    let (title,): (String,) = sqlx::query_as("SELECT data->>'title' FROM discover_items WHERE kind = 'doc' AND key = 'claude-code'").fetch_one(&h.pool).await?;
    assert_eq!(title, "Claude Code 上手导读");
    let (docs,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_items WHERE kind = 'doc'").fetch_one(&h.pool).await?;
    assert_eq!(docs, 5);
    h.finish().await
}

#[tokio::test]
async fn tracker_promos_keep_ongoing_offers_without_countdown_and_filter_before_limit() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h)=harness().await? else {return Ok(());};
    sqlx::query("DELETE FROM discover_items WHERE kind IN ('promo','launch')").execute(&h.pool).await?;
    let now=chrono::Utc::now();
    let ongoing=crate::discover::validate("promo",json!({"text":"验证学生资格后申请权益","started_at":null,"ends_at":null,"source_url":"https://education.github.com/pack","eligibility":"仅已验证学生；以官方地区与身份校验为准"}))?;
    let timed=crate::discover::validate("promo",json!({"text":"有真实起止日期","started_at":now-chrono::Duration::days(1),"ends_at":now+chrono::Duration::days(2)}))?;
    let future=crate::discover::validate("promo",json!({"text":"未来活动","started_at":now+chrono::Duration::days(1),"ends_at":now+chrono::Duration::days(3)}))?;
    for (key,data) in [("ongoing",ongoing),("timed",timed),("future",future)] {
        sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) VALUES('promo',$1,$2,'published',now()-interval '1 minute')").bind(key).bind(data).execute(&h.pool).await?;
    }
    let expired=json!({"text":"已过期","started_at":now-chrono::Duration::days(3),"ends_at":now-chrono::Duration::days(1)});
    sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) SELECT 'promo','expired-'||n,$1,'published',now()-interval '1 minute' FROM generate_series(1,60) n").bind(expired).execute(&h.pool).await?;
    let launch=crate::discover::validate("launch",json!({"title":"历史回顾｜2025模型发布","summary":"记录历史事件，并非今日上新","source_url":"https://qwenlm.github.io/blog/qwen3/"}))?;
    sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) VALUES('launch','historical',$1,'published','2025-04-29T00:00:00Z')").bind(launch).execute(&h.pool).await?;
    let (status,body)=call(&h.app,"GET","/api/discover/tracker/v2",None,Value::Null).await;
    assert_eq!(status,StatusCode::OK,"{body}");
    let items=body["promos"].as_array().unwrap();assert_eq!(items.len(),2,"{body}");
    let ongoing=items.iter().find(|p|p["key"]=="ongoing").unwrap();
    assert!(ongoing["ends_at"].is_null() && ongoing["days_left"].is_null() && ongoing["pct"].is_null());
    assert_eq!(ongoing["source_url"],"https://education.github.com/pack");
    let timed=items.iter().find(|p|p["key"]=="timed").unwrap();
    assert_eq!(timed["days_left"],2);assert!((30..=35).contains(&timed["pct"].as_i64().unwrap()));
    assert!(body["launches"][0]["published_at"].as_str().unwrap().starts_with("2025-04-29"));
    assert_eq!(body["launches"][0]["source_url"],"https://qwenlm.github.io/blog/qwen3/");
    h.finish().await?;Ok(())
}

#[tokio::test]
async fn discover_v2_preserves_nullable_truth_while_legacy_apps_decode_only_representable_data() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(serde::Deserialize)]
    struct OldRank { star_gain:i64, star_total:i64 }
    #[derive(serde::Deserialize)]
    struct OldIssue { total_gain:i64, wow_pct:f64, ranks:Vec<OldRank> }
    #[derive(serde::Deserialize)]
    struct OldHome { repos:Option<OldIssue> }
    #[derive(serde::Deserialize)]
    struct OldPromo { ends_at:String, pct:u8, days_left:i64 }
    #[derive(serde::Deserialize)]
    struct OldTracker { promos:Vec<OldPromo> }
    let Some(h)=harness().await? else {return Ok(());};
    sqlx::query("DELETE FROM discover_items WHERE kind IN ('repo_issue','promo')").execute(&h.pool).await?;
    let baseline:Value=serde_json::from_str(include_str!("../../content/community-launch-20261031/research/repo-baseline-2026-10-31.json"))?;
    let baseline=crate::discover::validate("repo_issue",baseline)?;
    sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) VALUES('repo_issue','baseline',$1,'published',now()-interval '1 minute')").bind(baseline).execute(&h.pool).await?;
    let (_,home)=call(&h.app,"GET","/api/discover/home",None,Value::Null).await;
    assert!(serde_json::from_value::<OldHome>(home)?.repos.is_none());
    assert_eq!(call(&h.app,"GET","/api/discover/repos",None,Value::Null).await.0,StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app,"GET","/api/discover/repos?issue=baseline",None,Value::Null).await.0,StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app,"GET","/api/discover/repos/issues",None,Value::Null).await.1["items"].as_array().unwrap().len(),0);
    let (status,issue)=call(&h.app,"GET","/api/discover/repos/v2",None,Value::Null).await;
    assert_eq!(status,StatusCode::OK);assert_eq!(issue["ranks"].as_array().unwrap().len(),8);
    assert!(issue["total_gain"].is_null() && issue["ranks"][0]["star_gain"].is_null());
    assert_eq!(call(&h.app,"GET","/api/discover/home/v2",None,Value::Null).await.1["repos"]["metric"],"star total baseline");
    assert_eq!(call(&h.app,"GET","/api/discover/repos/issues/v2",None,Value::Null).await.1["items"].as_array().unwrap().len(),1);
    let measured=crate::discover::validate("repo_issue",json!({"issue_no":1,"week_start":"2025-01-01","week_end":"2025-01-07","total_gain":12,"wow_pct":1.5,"daily_gain":[],"ranks":[{"full_name":"owner/repo","star_total":200,"star_gain":12}]}))?;
    sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) VALUES('repo_issue','numeric-old',$1,'published',now()-interval '1 minute')").bind(measured).execute(&h.pool).await?;
    let (_,compatible)=call(&h.app,"GET","/api/discover/repos",None,Value::Null).await;
    let old:OldIssue=serde_json::from_value(compatible)?;
    assert_eq!(old.total_gain,12);assert_eq!(old.wow_pct,1.5);assert_eq!(old.ranks[0].star_gain,12);assert_eq!(old.ranks[0].star_total,200);
    let (_,home)=call(&h.app,"GET","/api/discover/home",None,Value::Null).await;
    assert!(serde_json::from_value::<OldHome>(home)?.repos.is_none(),"home must not disguise an older compatible issue as latest");
    let (_,old_issues)=call(&h.app,"GET","/api/discover/repos/issues",None,Value::Null).await;
    assert_eq!(old_issues["items"].as_array().unwrap().len(),1);assert_eq!(old_issues["items"][0]["key"],"numeric-old");
    let reviewed:Value=serde_json::from_str(include_str!("../../content/community-launch-20261031/research/tracker-reviewed-items.json"))?;
    for row in reviewed["items"].as_array().unwrap().iter().filter(|r|r["kind"]=="promo") {
        let data=crate::discover::validate("promo",row["data"].clone())?;
        sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) VALUES('promo',$1,$2,'published',now()-interval '1 minute')").bind(row["key"].as_str().unwrap()).bind(data).execute(&h.pool).await?;
    }
    let now=chrono::Utc::now();
    let timed=crate::discover::validate("promo",json!({"text":"真实限时活动","started_at":now-chrono::Duration::days(1),"ends_at":now+chrono::Duration::days(2)}))?;
    sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) VALUES('promo','timed',$1,'published',now()-interval '1 minute')").bind(timed).execute(&h.pool).await?;
    let (_,legacy)=call(&h.app,"GET","/api/discover/tracker",None,Value::Null).await;
    let legacy:OldTracker=serde_json::from_value(legacy)?;
    assert_eq!(legacy.promos.len(),1);assert!(!legacy.promos[0].ends_at.is_empty());assert_eq!(legacy.promos[0].days_left,2);assert!(legacy.promos[0].pct<=100);
    let (_,v2)=call(&h.app,"GET","/api/discover/tracker/v2",None,Value::Null).await;
    assert_eq!(v2["promos"].as_array().unwrap().len(),6);
    assert_eq!(v2["promos"].as_array().unwrap().iter().filter(|p|p["ends_at"].is_null() && p["pct"].is_null() && p["days_left"].is_null()).count(),5);
    h.finish().await?;Ok(())
}

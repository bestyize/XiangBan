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

#[path = "discover_repo_tests.rs"]
mod repo_detail;
#[path = "open_source_tests.rs"]
mod open_source_system;

#[tokio::test]
async fn navigation_catalog_additions_preserve_edits_and_do_not_restore_removals() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    sqlx::query("DELETE FROM data_migrations WHERE name='navigation-catalog-20261103-v2'").execute(&h.pool).await?;
    sqlx::query("UPDATE discover_items SET data=jsonb_set(jsonb_set(data - 'category' - 'summary' - 'logo','{name}','\"Claude 已编辑\"'),'{url}','\"https://example.com/custom\"'),status='draft' WHERE kind='nav' AND key='claude'").execute(&h.pool).await?;
    sqlx::query("DELETE FROM discover_items WHERE kind='nav' AND key='grok'").execute(&h.pool).await?;
    crate::navigation_catalog::seed(&h.pool).await?;
    let (name,status):(String,String)=sqlx::query_as("SELECT data->>'name',status FROM discover_items WHERE kind='nav' AND key='claude'").fetch_one(&h.pool).await?;
    assert_eq!((name,status),("Claude 已编辑".into(),"draft".into()));
    let (url,category,group):(String,String,String)=sqlx::query_as("SELECT data->>'url',data->>'category',data->>'group' FROM discover_items WHERE kind='nav' AND key='claude'").fetch_one(&h.pool).await?;
    assert_eq!((url,category,group),("https://example.com/custom".into(),"harness".into(),"Harness".into()));
    let (_,nav)=call(&h.app,"GET","/api/discover/nav",None,Value::Null).await;
    assert_eq!(nav["catalog_version"],2);
    let entries:Vec<_>=nav["groups"].as_array().unwrap().iter().flat_map(|g|g["items"].as_array().unwrap()).collect();
    assert!(entries.iter().any(|item|item["key"]=="grok"));
    assert!(!entries.iter().any(|item|item["key"]=="claude"));
    sqlx::query("DELETE FROM discover_items WHERE kind='nav' AND key='grok'").execute(&h.pool).await?;
    crate::discover::seed(&h.pool).await?;
    let (count,):(i64,)=sqlx::query_as("SELECT count(*) FROM discover_items WHERE kind='nav' AND key='grok'").fetch_one(&h.pool).await?;
    assert_eq!(count,0,"a later startup must not resurrect an editorial removal");
    h.finish().await
}

#[tokio::test]
async fn navigation_catalog_v1_deletions_survive_v2_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    sqlx::query("DELETE FROM data_migrations WHERE name='navigation-catalog-20261103-v2'").execute(&h.pool).await?;
    sqlx::query("INSERT INTO data_migrations(name) VALUES('navigation-catalog-20261103-v1') ON CONFLICT DO NOTHING").execute(&h.pool).await?;
    sqlx::query("DELETE FROM discover_items WHERE kind='nav' AND key IN ('grok','hermes-agent')").execute(&h.pool).await?;
    crate::discover::seed(&h.pool).await?;
    let keys:Vec<(String,)>=sqlx::query_as("SELECT key FROM discover_items WHERE kind='nav' AND key IN ('grok','hermes-agent')").fetch_all(&h.pool).await?;
    assert_eq!(keys,vec![("hermes-agent".to_string(),)],"add new v2 entries without restoring a removed v1 product");
    h.finish().await
}

#[test]
fn navigation_catalog_validates_category_metadata() {
    let mut item=crate::navigation_catalog::catalog()["items"].as_array().unwrap().iter().find(|i|i["key"]=="openrouter-api").unwrap().clone();
    let normalized=crate::discover::validate("nav",item.clone()).unwrap();
    assert_eq!(normalized["category"],"api");assert_eq!(normalized["subtype"],"router");assert!(!normalized["summary"].as_str().unwrap().is_empty());
    item["category"]=json!("models");assert!(crate::discover::validate("nav",item.clone()).is_err());
    item["category"]=json!("api");item["region"]=json!("invalid");assert!(crate::discover::validate("nav",item).is_err());
}

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
    state: AppState,
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
    let app = crate::build_router(state.clone());
    let (s, login) = call(&app, "POST", "/api/admin/login", None, json!({"username":"root","password":"correct-horse-battery"})).await;
    assert_eq!(s, StatusCode::OK, "{login}");
    let admin_t = login["token"].as_str().unwrap().to_string();
    Ok(Some(Harness { app, state, pool, admin_pool, schema, admin_t, make_token }))
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
async fn uncategorized_topics_stay_in_their_own_boards() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    crate::taxonomy::migrate(&h.pool).await?;
    crate::taxonomy::migrate_sections(&h.pool).await?;
    tokio::try_join!(
        crate::taxonomy::add_uncategorized_boards(&h.pool),
        crate::taxonomy::add_uncategorized_boards(&h.pool),
    )?;

    let (status, categories) = call(&h.app, "GET", "/api/categories", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let defaults: Vec<_> = categories.as_array().unwrap().iter()
        .filter(|board| board["kind"] == crate::taxonomy::UNCATEGORIZED_KIND)
        .collect();
    assert_eq!(defaults.len(), 3);
    let member = h.member().await?;
    for (section, specific) in [("ideas", "create-wallpaper"), ("discuss", "learn"), ("resources", "resource-help")] {
        let board = defaults.iter().find(|board| board["section"] == section).unwrap();
        let slug = format!("{section}-uncategorized");
        assert_eq!(board["slug"], slug.as_str());
        assert_eq!(board["name"], "默认");
        let (status, created) = call(&h.app, "POST", "/api/topics", Some(&member), json!({
            "category_id": board["id"], "title": "Uncategorized topic", "body": "This post belongs to the default board."
        })).await;
        assert_eq!(status, StatusCode::OK, "{created}");

        let (_, list) = call(&h.app, "GET", &format!("/api/topics?category={slug}&active_only=true"), None, Value::Null).await;
        assert_eq!(list["total"], 1);
        assert_eq!(list["items"][0]["id"], created["id"]);
        assert_eq!(list["items"][0]["category"]["id"], board["id"]);
        assert!(list["items"][0]["help"].is_null());
        let (_, other) = call(&h.app, "GET", &format!("/api/topics?category={specific}"), None, Value::Null).await;
        assert_eq!(other["total"], 0);
    }

    let board_id = defaults[0]["id"].as_i64().unwrap();
    let uri = format!("/api/admin/categories/{board_id}");
    let (status, _) = call(&h.app, "PATCH", &uri, Some(&h.admin_t), json!({"section": "site"})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = call(&h.app, "PATCH", &uri, Some(&h.admin_t), json!({"posting_enabled": false})).await;
    assert_eq!(status, StatusCode::OK);
    crate::taxonomy::add_uncategorized_boards(&h.pool).await?;
    let (status, _) = call(&h.app, "POST", "/api/topics", Some(&member), json!({
        "category_id": board_id, "title": "Closed uncategorized board", "body": "This post must not be published."
    })).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    h.finish().await
}

#[tokio::test]
async fn deals_board_accepts_regular_discussion_posts() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    crate::taxonomy::migrate(&h.pool).await?;
    crate::taxonomy::migrate_sections(&h.pool).await?;
    crate::taxonomy::add_app_channel(&h.pool).await?;
    crate::taxonomy::retire_open_source_board(&h.pool).await?;
    crate::taxonomy::add_uncategorized_boards(&h.pool).await?;
    crate::taxonomy::flatten_community_boards(&h.pool).await?;
    tokio::try_join!(crate::taxonomy::add_deals_board(&h.pool), crate::taxonomy::add_deals_board(&h.pool))?;

    let (status, categories) = call(&h.app, "GET", "/api/categories", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let boards = categories.as_array().unwrap();
    let deals = boards.iter().find(|board| board["slug"] == "casual-deals").unwrap();
    assert_eq!(deals["name"], "羊毛");
    assert_eq!(deals["section"], "discuss");
    assert_eq!(deals["kind"], "normal");
    assert!(deals["parent_id"].is_null());

    let member = h.member().await?;
    let (status, topic) = call(&h.app, "POST", "/api/topics", Some(&member), json!({
        "category_id": deals["id"], "title": "分享一个软件的免费试用活动", "body": "这里介绍活动规则和官方入口。"
    })).await;
    assert_eq!(status, StatusCode::OK, "{topic}");
    let (status, list) = call(&h.app, "GET", "/api/topics?category=casual-deals&active_only=true", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 1);
    assert_eq!(list["items"][0]["id"], topic["id"]);

    sqlx::query("UPDATE categories SET status='archived' WHERE slug='casual-deals'").execute(&h.pool).await?;
    crate::taxonomy::add_deals_board(&h.pool).await?;
    let board_status: String = sqlx::query_scalar("SELECT status FROM categories WHERE slug='casual-deals'").fetch_one(&h.pool).await?;
    assert_eq!(board_status, "archived");
    h.finish().await
}

#[tokio::test]
async fn community_board_flattening_preserves_content_and_prevents_new_subcategories() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    crate::taxonomy::migrate(&h.pool).await?;
    crate::taxonomy::migrate_sections(&h.pool).await?;
    crate::taxonomy::add_app_channel(&h.pool).await?;
    crate::taxonomy::retire_open_source_board(&h.pool).await?;
    crate::taxonomy::add_uncategorized_boards(&h.pool).await?;

    let ids: std::collections::HashMap<String, i32> = sqlx::query_as("SELECT slug, id FROM categories")
        .fetch_all(&h.pool).await?.into_iter().collect();
    let author = Uuid::new_v4();
    sqlx::query("INSERT INTO users (uid,nickname,trust_level) VALUES ($1,'Old board author',3)")
        .bind(author).execute(&h.pool).await?;
    sqlx::query("UPDATE categories SET on_home=false WHERE slug='learn-codex'").execute(&h.pool).await?;
    sqlx::query("UPDATE categories SET min_post_level=1 WHERE slug='site-feedback'").execute(&h.pool).await?;
    let archived: i32 = sqlx::query_scalar(
        "INSERT INTO categories(slug,name,section,parent_id,status)
         VALUES('old-tutorials','Archived tutorials','discuss',$1,'archived') RETURNING id",
    ).bind(ids["learn"]).fetch_one(&h.pool).await?;
    let casual: i32 = sqlx::query_scalar(
        "INSERT INTO categories(slug,name,section,parent_id) VALUES('local-chat','Local chat','discuss',$1) RETURNING id",
    ).bind(ids["casual"]).fetch_one(&h.pool).await?;
    let mut topics = std::collections::HashMap::new();
    for (name, category, pin) in [
        ("Tutorial", ids["learn-codex"], "parent"),
        ("Archived", archived, "none"),
        ("Feedback", ids["site-feedback"], "site"),
        ("Wallpaper", ids["create-wallpaper"], "category"),
        ("General idea", ids["create"], "none"),
    ] {
        let topic: i64 = sqlx::query_scalar(
            "INSERT INTO topics(category_id,author_uid,title,body_md,body_html,pin_scope,pinned)
             VALUES($1,$2,$3,'Public body','Public body',$4,$4 <> 'none') RETURNING id",
        ).bind(category).bind(author).bind(name).bind(pin).fetch_one(&h.pool).await?;
        topics.insert(name, topic);
    }
    let draft = Uuid::new_v4();
    sqlx::query("INSERT INTO editor_drafts(id,author_uid,external_id,category_id,title,body_md) VALUES ($1,$2,'old-draft',$3,'Tutorial draft','draft')")
        .bind(draft).bind(author).bind(ids["learn-codex"]).execute(&h.pool).await?;

    tokio::try_join!(
        crate::taxonomy::flatten_community_boards(&h.pool),
        crate::taxonomy::flatten_community_boards(&h.pool),
    )?;
    crate::db::seed(&h.pool).await?;
    crate::taxonomy::migrate(&h.pool).await?;
    let (status, categories) = call(&h.app, "GET", "/api/categories", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    let boards = categories.as_array().unwrap();
    assert!(boards.iter().filter(|row| row["section"] != "resources").all(|row| row["parent_id"].is_null()));
    assert!(boards.iter().all(|row| !["create","casual","learn-codex","site-feedback","old-tutorials"]
        .contains(&row["slug"].as_str().unwrap())));
    for slug in ["learn", "site", "create-wallpaper", "casual-games", "resource-help"] {
        let board = boards.iter().find(|row| row["slug"] == slug).unwrap();
        assert_eq!(board["id"], ids[slug]);
    }
    assert_eq!(boards.iter().find(|row| row["id"] == casual).unwrap()["allow_short_title"], true);
    for (name, board, pin) in [
        ("Tutorial", "learn", "category"),
        ("Archived", "learn", "none"),
        ("Feedback", "site", "site"),
        ("Wallpaper", "create-wallpaper", "category"),
        ("General idea", "ideas-uncategorized", "none"),
    ] {
        let (_, topic) = call(&h.app, "GET", &format!("/api/topics/{}", topics[name]), None, Value::Null).await;
        assert_eq!(topic["category"]["slug"], board, "{topic}");
        assert_eq!(topic["pin_scope"], pin);
    }
    let draft_board: i32 = sqlx::query_scalar("SELECT category_id FROM editor_drafts WHERE id=$1")
        .bind(draft).fetch_one(&h.pool).await?;
    assert_eq!(draft_board, ids["learn"]);
    for (query, count) in [("learn-codex&active_only=true", 1), ("learn&active_only=true&home=true", 0), ("old-tutorials", 2)] {
        let (_, list) = call(&h.app, "GET", &format!("/api/topics?category={query}"), None, Value::Null).await;
        assert_eq!(list["total"], count, "{query}: {list}");
    }
    for (old, destination) in [("learn-codex","/c/learn"),("old-tutorials","/c/learn"),("site-feedback","/c/site"),("create","/ideas")] {
        let request = Request::builder().uri(format!("/c/{old}")).body(Body::empty())?;
        let response = h.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(response.headers()["location"], destination);
    }
    let member = h.member().await?;
    for (category, title) in [(ids["learn"], "New tutorial"), (ids["site"], "New feedback"), (casual, ""), (ids["create-wallpaper"], "New wallpaper")] {
        let (status, post) = call(&h.app, "POST", "/api/topics", Some(&member), json!({
            "category_id": category, "title": title, "body": "This is a post in a flat board."
        })).await;
        assert_eq!(status, StatusCode::OK, "{post}");
    }
    for parent in [ids["learn"], ids["site"], ids["create-wallpaper"]] {
        let (status, _) = call(&h.app, "POST", "/api/admin/categories", Some(&h.admin_t),
            json!({"name":"Invalid child","kind":"normal","slug":"invalid-child","parent_id":parent})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    let attempted_child = sqlx::query("INSERT INTO categories(slug,name,section,parent_id) VALUES('invalid','Invalid','resources',$1)")
        .bind(ids["learn"]).execute(&h.pool).await;
    assert!(attempted_child.is_err());
    h.finish().await
}

#[tokio::test]
async fn newcomers_can_upload_and_publish_discover_comment_images() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let upload_dir = std::env::temp_dir().join(&h.schema);
    let mut state = h.state.clone();
    Arc::make_mut(&mut state.config).upload_dir = upload_dir.clone();
    let app = crate::build_router(state.clone());

    let uid = Uuid::new_v4();
    sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'新人',0)").bind(uid).execute(&h.pool).await?;
    let author = (h.make_token)(uid);
    let other = h.member().await?;
    let (_, session) = call(&app, "POST", "/api/session", Some(&author), json!({})).await;
    assert_eq!(session["user"]["trust_level"], 0, "{session}");
    assert_eq!(session["limits"]["image_max_bytes"], state.config.upload_limit_image);
    assert_eq!(session["limits"]["comment_images"], true);

    let mut upload = b"--check\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.gif\"\r\nContent-Type: image/gif\r\n\r\n".to_vec();
    upload.extend(b"GIF89a\x01\x00\x01\x00");
    upload.extend(b"\r\n--check--\r\n");
    let response = app.clone().oneshot(Request::post("/api/images")
        .header("authorization", format!("Bearer {author}"))
        .header("content-type", "multipart/form-data; boundary=check")
        .body(Body::from(upload))?).await?;
    let status = response.status();
    let image: Value = serde_json::from_slice(&to_bytes(response.into_body(), 65536).await?)?;
    assert_eq!(status, StatusCode::OK, "{image}");
    let id = Uuid::parse_str(image["id"].as_str().unwrap())?;
    let image_url = image["url"].as_str().unwrap();

    for (kind, key) in [("news", "comment-img"), ("frontier", "comment-img"), ("repo_issue", "comment-img")] {
        let data = match kind {
            "repo_issue" => json!({"ranks":[{"full_name":"demo/image-comments"}]}),
            _ => json!({"title":"配图评论"}),
        };
        sqlx::query("INSERT INTO discover_items(kind,key,data,status,published_at) VALUES($1,$2,$3,'published',now())")
            .bind(kind).bind(key).bind(data).execute(&h.pool).await?;
    }
    let paths = ["/api/discover/daily/comment-img/comments", "/api/discover/frontier/comment-img/comments",
        "/api/discover/repo/demo/image-comments/comments"];
    let first = paths[0];
    let (status, _) = call(&app, "POST", first, Some(&other), json!({"body":"别人上传的图","image_ids":[id]})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let mut posted = Vec::new();
    for path in paths {
        let (status, result) = call(&app, "POST", path, Some(&author), json!({"body":"","image_ids":[id]})).await;
        assert_eq!(status, StatusCode::OK, "{path}: {result}");
        posted.push((path, result["id"].as_i64().unwrap()));
        let (status, listing) = call(&app, "GET", path, None, Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(listing["total"], 1);
        assert_eq!(listing["items"][0]["images"], json!([image_url]));
    }
    sqlx::query("UPDATE images SET created_at=now()-interval '2 days' WHERE id=$1").bind(id).execute(&h.pool).await?;
    assert_eq!(crate::handlers::images::purge_unused(&h.pool, &upload_dir).await?, 0);
    for (path, id) in posted {
        assert_eq!(call(&app, "DELETE", &format!("{path}/{id}"), Some(&author), Value::Null).await.0, StatusCode::OK);
    }
    assert_eq!(crate::handlers::images::purge_unused(&h.pool, &upload_dir).await?, 1);
    tokio::fs::remove_dir(upload_dir).await?;
    h.finish().await
}

#[tokio::test]
async fn avatar_images_survive_unused_image_purge() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let upload_dir = std::env::temp_dir().join(format!("{}-avatar", h.schema));
    tokio::fs::create_dir_all(&upload_dir).await?;
    let (uid, avatar, pending, unused) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    sqlx::query("INSERT INTO users(uid,nickname,trust_level,avatar_url) VALUES($1,'换头像',0,$2)")
        .bind(uid).bind(format!("https://chat.xiangban.xyz/api/images/{avatar}")).execute(&h.pool).await?;
    for id in [avatar, pending, unused] {
        sqlx::query("INSERT INTO images(id,uploader_uid,mime,size,created_at) VALUES($1,$2,'image/jpeg',4,now()-interval '2 days')")
            .bind(id).bind(uid).execute(&h.pool).await?;
    }
    let hold = format!("/api/images/{pending}/avatar-hold");
    let other = h.member().await?;
    assert_eq!(call(&h.app, "POST", &hold, Some(&other), Value::Null).await.0, StatusCode::NOT_FOUND);
    let owner = (h.make_token)(uid);
    for _ in 0..2 {
        assert_eq!(call(&h.app, "POST", &hold, Some(&owner), Value::Null).await.0, StatusCode::OK);
    }
    let left = |pool: sqlx::PgPool| async move {
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM images WHERE id = ANY($1)").bind(vec![avatar, pending, unused]).fetch_all(&pool).await
    };
    crate::handlers::images::purge_unused(&h.pool, &upload_dir).await?;
    let retained = left(h.pool.clone()).await?;
    assert_eq!(retained.len(), 2);
    assert!(retained.contains(&avatar) && retained.contains(&pending));
    let next = format!("https://res.xiangban.xyz/forum-content/images/{pending}");
    crate::users::store_synced_profile(&h.pool, uid, "换头像", Some(next)).await?;
    assert!(!sqlx::query_scalar::<_, bool>("SELECT avatar_pending FROM images WHERE id=$1").bind(pending).fetch_one(&h.pool).await?);
    crate::handlers::images::purge_unused(&h.pool, &upload_dir).await?;
    assert_eq!(left(h.pool.clone()).await?, vec![pending]);
    crate::users::store_synced_profile(&h.pool, uid, "换头像", None).await?;
    crate::handlers::images::purge_unused(&h.pool, &upload_dir).await?;
    assert!(left(h.pool.clone()).await?.is_empty(), "换掉头像后旧图按普通未引用图片清理");
    tokio::fs::remove_dir_all(upload_dir).await?;
    h.finish().await
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
    assert_eq!(call(&h.app, "POST", path, Some(&author), json!({"body":" \n "})).await.0, StatusCode::BAD_REQUEST);
    let (status, created) = call(&h.app, "POST", path, Some(&author), json!({"body":"好"})).await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let id = created["id"].as_i64().unwrap();
    assert_eq!(call(&h.app, "POST", path, Some(&author), json!({"body":"太快了"})).await.0, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(call(&h.app, "POST", path, Some(&other), json!({"body":"👍","reply_to":id})).await.0, StatusCode::OK);
    assert_eq!(call(&h.app, "PUT", &format!("{path}/{id}/like"), Some(&other), json!({})).await.0, StatusCode::OK);
    let (_, comments) = call(&h.app, "GET", path, Some(&other), Value::Null).await;
    assert_eq!(comments["total"], 2);
    assert_eq!(comments["items"][1]["like_count"], 1);
    assert_eq!(comments["items"][1]["liked"], true);
    assert_eq!(comments["items"][0]["reply_to"], id);
    assert_eq!(comments["items"][0]["body"], "👍");
    assert_eq!(comments["items"][1]["body"], "好");
    assert_eq!(call(&h.app, "DELETE", &format!("{path}/{id}"), Some(&other), json!({})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app, "DELETE", &format!("{path}/{id}"), Some(&author), json!({})).await.0, StatusCode::OK);
    sqlx::query("UPDATE discover_items SET status='draft' WHERE kind='news' AND key='article-test'").execute(&h.pool).await?;
    assert_eq!(call(&h.app, "GET", path, None, Value::Null).await.0, StatusCode::NOT_FOUND);
    h.finish().await
}

#[tokio::test]
async fn idea_comments_share_the_comment_contract_without_mixing_with_daily() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let mut state = h.state.clone();
    Arc::make_mut(&mut state.config).web_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web/public");
    let app = crate::build_router(state);
    sqlx::query("INSERT INTO discover_items(kind,key,status,published_at,data) VALUES('news','plush-rabbit','published',now(),'{}')")
        .execute(&h.pool).await?;

    let work = "/api/ideas/plush-rabbit/comments";
    let daily = "/api/discover/daily/plush-rabbit/comments";
    let other_work = "/api/ideas/saltline/comments";
    for key in ["micro-gallery", "not-a-work"] {
        let path = format!("/api/ideas/{key}/comments");
        assert_eq!(call(&app, "GET", &path, None, Value::Null).await.0, StatusCode::NOT_FOUND);
    }
    let (status, empty) = call(&app, "GET", work, None, Value::Null).await;
    assert_eq!(status, StatusCode::OK, "{empty}");
    assert_eq!(empty["total"], 0);
    assert_eq!(call(&app, "POST", work, None, json!({"body":"留言"})).await.0, StatusCode::UNAUTHORIZED);

    let author = h.member().await?;
    let reader = h.member().await?;
    let (status, comment) = call(&app, "POST", work, Some(&author), json!({"body":"作品评论"})).await;
    assert_eq!(status, StatusCode::OK, "{comment}");
    let id = comment["id"].as_i64().unwrap();
    assert_eq!(call(&app, "POST", work, Some(&author), json!({"body":"太快了"})).await.0, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(call(&app, "POST", daily, Some(&author), json!({"body":"日报评论"})).await.0, StatusCode::OK);
    assert_eq!(call(&app, "POST", other_work, Some(&reader), json!({"body":"回复", "reply_to":id})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(call(&app, "POST", work, Some(&reader), json!({"body":"回复", "reply_to":id})).await.0, StatusCode::OK);
    assert_eq!(call(&app, "PUT", &format!("{daily}/{id}/like"), Some(&reader), Value::Null).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&app, "PUT", &format!("{work}/{id}/like"), Some(&reader), Value::Null).await.0, StatusCode::OK);

    let (_, comments) = call(&app, "GET", work, Some(&reader), Value::Null).await;
    assert_eq!(comments["total"], 2);
    assert_eq!(comments["items"][0]["reply_to"], id);
    assert_eq!(comments["items"][1]["liked"], true);
    let (_, comments) = call(&app, "GET", daily, None, Value::Null).await;
    assert_eq!(comments["total"], 1);
    assert_eq!(comments["items"][0]["body"], "日报评论");
    assert_eq!(call(&app, "DELETE", &format!("{work}/{id}"), Some(&reader), Value::Null).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&app, "DELETE", &format!("{work}/{id}"), Some(&author), Value::Null).await.0, StatusCode::OK);
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
    assert_eq!(nav["groups"].as_array().unwrap().iter().map(|g|g["name"].as_str().unwrap()).collect::<Vec<_>>(), vec!["大模型", "Harness", "Skill", "API"]);
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
async fn topic_tags_commit_with_the_topic_and_scores_do_not_gate_creation() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let user = h.member().await?;
    let category: i32 = sqlx::query_scalar("SELECT id FROM categories WHERE slug = 'casual-fun'").fetch_one(&h.pool).await?;
    sqlx::raw_sql(
        "CREATE FUNCTION reject_test_score() RETURNS trigger LANGUAGE plpgsql AS $$
             BEGIN RAISE EXCEPTION 'test score update failure'; END; $$;
         CREATE TRIGGER reject_test_score BEFORE UPDATE OF hot_score ON topics
             FOR EACH ROW EXECUTE FUNCTION reject_test_score();",
    ).execute(&h.pool).await?;
    let draft = |title, tag| json!({"category_id": category, "title": title, "body": "用于本地测试的话题正文。", "tags": [tag]});
    let (status, topic) = call(&h.app, "POST", "/api/topics", Some(&user), draft("初始帖子", "原标签")).await;
    assert_eq!(status, StatusCode::OK, "{topic}");
    let id = topic["id"].as_i64().unwrap();

    sqlx::query("ALTER TABLE topic_tags ADD CONSTRAINT reject_test_tag CHECK (false) NOT VALID").execute(&h.pool).await?;
    let (status, _) = call(&h.app, "POST", "/api/topics", Some(&user), draft("失败的发布", "新标签")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM topics WHERE title = '失败的发布'").fetch_one(&h.pool).await?;
    assert_eq!(count, 0);

    let path = format!("/api/topics/{id}");
    let (status, _) = call(&h.app, "PATCH", &path, Some(&user), draft("失败的编辑", "新标签")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let (_, stored) = call(&h.app, "GET", &path, Some(&user), Value::Null).await;
    assert_eq!(stored["title"], "初始帖子");
    assert_eq!(stored["tags"], json!(["原标签"]));
    h.finish().await
}

#[tokio::test]
async fn topic_publish_retry_returns_the_same_id_even_after_quota_is_used() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let (author, other) = (h.member().await?, h.member().await?);
    let mut state = h.state.clone();
    Arc::make_mut(&mut state.config).trust.daily_topics[1] = 1;
    let app = crate::build_router(state);
    let category: i32 = sqlx::query_scalar("SELECT id FROM categories WHERE slug='casual-fun'").fetch_one(&h.pool).await?;
    let attempt = Uuid::new_v4();
    let input = json!({"category_id":category,"title":"断网重试","body":"这是一篇用来校验发帖回执恢复的测试帖子。","request_id":attempt});
    let ((first_status, first), (retry_status, retry)) = tokio::join!(
        call(&app, "POST", "/api/topics", Some(&author), input.clone()),
        call(&app, "POST", "/api/topics", Some(&author), input.clone()),
    );
    assert_eq!(first_status, StatusCode::OK, "{first}");
    assert_eq!(retry_status, StatusCode::OK, "{retry}");
    assert_eq!(first["id"], retry["id"]);
    let (status, after_quota) = call(&app, "POST", "/api/topics", Some(&author), input.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(after_quota["id"], first["id"]);
    let mut new_input = input.clone();
    new_input["request_id"] = json!(Uuid::new_v4());
    assert_eq!(call(&app, "POST", "/api/topics", Some(&author), new_input).await.0, StatusCode::TOO_MANY_REQUESTS);

    let (status, other_post) = call(&app, "POST", "/api/topics", Some(&other), input).await;
    assert_eq!(status, StatusCode::OK, "{other_post}");
    assert_ne!(other_post["id"], first["id"]);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM topics WHERE publish_request_id=$1")
        .bind(attempt).fetch_one(&h.pool).await?;
    assert_eq!(count, 2);
    h.finish().await
}

#[tokio::test]
async fn restricted_discussion_images_follow_the_readers_permissions() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let author = Uuid::new_v4();
    let reader = Uuid::new_v4();
    for uid in [author, reader] {
        sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'测试用户',1)")
            .bind(uid).execute(&h.pool).await?;
    }
    let author_token = (h.make_token)(author);
    let reader_token = (h.make_token)(reader);
    let cover = Uuid::new_v4();
    let reply = Uuid::new_v4();
    let public = Uuid::new_v4();
    for (image, owner) in [(cover, author), (reply, reader), (public, author)] {
        sqlx::query("INSERT INTO images(id,uploader_uid,mime,size) VALUES($1,$2,'image/png',4)")
            .bind(image).bind(owner).execute(&h.pool).await?;
    }
    let category: i32 = sqlx::query_scalar("SELECT id FROM categories WHERE slug='casual-fun'").fetch_one(&h.pool).await?;
    let draft = |title, id, level| json!({"category_id":category,"title":title,
        "body":format!("测试图片正文\n\n![图](/api/images/{id})"),"min_read_level":level});
    let (status, created) = call(&h.app, "POST", "/api/topics", Some(&author_token), draft("受限图片测试", cover, 1)).await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let topic = created["id"].as_i64().unwrap();
    let url = format!("/api/topics/{topic}");
    let posts = format!("{url}/posts");
    let (status, result) = call(&h.app, "POST", &posts, Some(&reader_token), json!({"body":format!("![回帖图](/api/images/{reply})")})).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let protected: i64 = sqlx::query_scalar("SELECT count(*) FROM images WHERE id=ANY($1) AND discussion_private")
        .bind(vec![cover,reply]).fetch_one(&h.pool).await?;
    assert_eq!(protected, 2);

    // Simulate existing rows from before the migration.
    sqlx::query("UPDATE images SET discussion_private=false WHERE id=ANY($1)").bind(vec![cover,reply]).execute(&h.pool).await?;
    sqlx::query("DELETE FROM data_migrations WHERE name='restricted-discussion-images-v1'").execute(&h.pool).await?;
    crate::discussion_images::init(&h.pool).await?;
    assert_eq!(call(&h.app, "GET", &url, None, Value::Null).await.0, StatusCode::FORBIDDEN);
    for id in [cover, reply] {
        for method in ["GET", "HEAD"] {
            assert_eq!(call(&h.app, method, &format!("/api/images/{id}"), None, Value::Null).await.0, StatusCode::NOT_FOUND);
        }
    }
    assert_eq!(call(&h.app, "HEAD", &format!("/api/images/{public}"), None, Value::Null).await.0, StatusCode::OK);
    let (_, detail) = call(&h.app, "GET", &url, Some(&reader_token), Value::Null).await;
    let cover_view = detail["images"][0].as_str().unwrap();
    assert!(detail["body_html"].as_str().unwrap().contains(cover_view));
    assert!(!detail["body_md"].as_str().unwrap().contains("/view?"));
    let mut local_state = h.state.clone();
    let dir = std::env::temp_dir().join(format!("{}-restricted-image", h.schema));
    tokio::fs::create_dir_all(&dir).await?;
    tokio::fs::write(dir.join(format!("img-{cover}")), b"test").await?;
    Arc::make_mut(&mut local_state.config).upload_dir = dir.clone();
    let local = crate::build_router(local_state);
    let image = local.oneshot(Request::builder().uri(cover_view).body(Body::empty())?).await?;
    assert_eq!(image.status(), StatusCode::OK);
    assert_eq!(image.headers()["cache-control"], "no-store");
    assert_eq!(to_bytes(image.into_body(), 4).await?.as_ref(), b"test");
    tokio::fs::remove_dir_all(dir).await?;
    let (_, preview) = call(&h.app, "POST", "/api/markdown/preview", Some(&author_token),
        json!({"body":format!("![受限](/api/images/{cover})\n\n![公开](/api/images/{public})")})).await;
    let preview = preview["html"].as_str().unwrap();
    assert!(preview.contains(&format!("/api/images/{cover}/view?key=")));
    assert!(preview.contains(&format!("src=\"/api/images/{public}\"")));
    let (_, list) = call(&h.app, "GET", "/api/topics", Some(&reader_token), Value::Null).await;
    let item = list["items"].as_array().unwrap().iter().find(|v| v["id"] == topic).unwrap();
    assert!(item["images"][0].as_str().unwrap().contains("/view?key="));
    assert_eq!(call(&h.app, "HEAD", cover_view, None, Value::Null).await.0, StatusCode::OK);
    let (_, discussion) = call(&h.app, "GET", &posts, Some(&author_token), Value::Null).await;
    let html = discussion["items"][0]["body_html"].as_str().unwrap();
    let reply_view = html.split("src=\"").nth(1).unwrap().split('"').next().unwrap();
    assert!(reply_view.contains("/view?key="));
    assert_eq!(call(&h.app, "HEAD", reply_view, None, Value::Null).await.0, StatusCode::OK);

    sqlx::query("UPDATE users SET trust_level=0 WHERE uid=$1").bind(reader).execute(&h.pool).await?;
    assert_eq!(call(&h.app, "HEAD", cover_view, None, Value::Null).await.0, StatusCode::NOT_FOUND);
    let (status, _) = call(&h.app, "PATCH", &url, Some(&author_token), draft("受限图片改公开", cover, 0)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(call(&h.app, "HEAD", &format!("/api/images/{cover}"), None, Value::Null).await.0, StatusCode::OK);
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
async fn retired_tracker_is_removed_on_database_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    sqlx::raw_sql(
        "DELETE FROM data_migrations WHERE name = 'remove-discover-tracker-v1';
         INSERT INTO discover_items (kind, key, data)
             SELECT kind, 'retirement-test', '{}' FROM unnest(ARRAY['quota', 'launch', 'promo', 'nav']) AS kind;
         INSERT INTO discover_drafts (kind, key, data, updated_by)
             SELECT kind, 'retirement-test', '{}' , 'test' FROM unnest(ARRAY['quota', 'launch', 'promo', 'nav']) AS kind;
         UPDATE discover_drafts SET status = 'scheduled', publish_after = now() + interval '1 day',
             reviewed_by = 'test', reviewed_at = now() WHERE kind = 'promo' AND key = 'retirement-test';
         INSERT INTO discover_tokens (id, name, kinds, publish_kinds, expires_at) VALUES
             (gen_random_uuid(), 'retired', ARRAY['quota', 'promo'], '{}', now() + interval '1 day'),
             (gen_random_uuid(), 'mixed', ARRAY['launch', 'news', 'frontier'], ARRAY['news'], now() + interval '1 day');",
    )
    .execute(&h.pool)
    .await?;

    crate::db::migrate(&h.pool).await?;
    crate::discover::seed(&h.pool).await?;

    for table in ["discover_items", "discover_drafts"] {
        let kinds: Vec<(String,)> = sqlx::query_as(&format!(
            "SELECT kind FROM {table} WHERE key = 'retirement-test' OR kind IN ('quota', 'launch', 'promo')",
        ))
        .fetch_all(&h.pool)
        .await?;
        assert_eq!(kinds, vec![("nav".into(),)], "{table}");
    }
    let tokens: Vec<(String, Vec<String>, Vec<String>)> =
        sqlx::query_as("SELECT name, kinds, publish_kinds FROM discover_tokens").fetch_all(&h.pool).await?;
    assert_eq!(
        tokens,
        vec![("mixed".into(), vec!["news".into(), "frontier".into()], vec!["news".into()])],
    );
    for path in ["/api/discover/tracker", "/api/discover/tracker/v2"] {
        assert_eq!(call(&h.app, "GET", path, None, Value::Null).await.0, StatusCode::NOT_FOUND);
    }
    for kind in ["quota", "launch", "promo"] {
        let path = format!("/api/admin/discover/{kind}");
        assert_eq!(call(&h.app, "GET", &path, Some(&h.admin_t), Value::Null).await.0, StatusCode::BAD_REQUEST);
        let path = format!("/api/admin/discover-workflow/preview/{kind}");
        assert_eq!(
            call(&h.app, "PUT", &path, Some(&h.admin_t), json!({"data": {}})).await.0,
            StatusCode::BAD_REQUEST,
        );
    }
    h.finish().await
}

#[tokio::test]
async fn discover_v2_preserves_nullable_truth_while_legacy_apps_decode_only_representable_data() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(serde::Deserialize)]
    struct OldRank { star_gain:i64, star_total:i64 }
    #[derive(serde::Deserialize)]
    struct OldIssue { total_gain:i64, wow_pct:f64, ranks:Vec<OldRank> }
    #[derive(serde::Deserialize)]
    struct OldHome { repos:Option<OldIssue> }
    let Some(h)=harness().await? else {return Ok(());};
    sqlx::query("DELETE FROM discover_items WHERE kind = 'repo_issue'").execute(&h.pool).await?;
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
    h.finish().await?;Ok(())
}

#[tokio::test]
async fn resource_head_checks_visibility_without_reading_image_bodies() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let owner = Uuid::new_v4();
    sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'test',1)").bind(owner).execute(&h.pool).await?;
    let image = Uuid::new_v4();
    sqlx::query("INSERT INTO images(id,uploader_uid,mime,size,editor_pending) VALUES($1,$2,'image/png',1,true)")
        .bind(image).bind(owner).execute(&h.pool).await?;
    let path = format!("/api/images/{image}");
    assert_eq!(call(&h.app,"HEAD",&path,None,Value::Null).await.0,StatusCode::NOT_FOUND);
    sqlx::query("UPDATE images SET editor_pending=false WHERE id=$1").bind(image).execute(&h.pool).await?;
    assert_eq!(call(&h.app,"HEAD",&path,None,Value::Null).await.0,StatusCode::OK);
    sqlx::query("DELETE FROM images WHERE id=$1").bind(image).execute(&h.pool).await?;
    assert_eq!(call(&h.app,"HEAD",&path,None,Value::Null).await.0,StatusCode::NOT_FOUND);

    let asset = Uuid::new_v4();
    sqlx::query("INSERT INTO frontier_assets(id,article_key,mime,data,kind) VALUES($1,'resource-head','image/png',$2,'frontier')")
        .bind(asset).bind(vec![1u8]).execute(&h.pool).await?;
    let data = json!({"cover_url":format!("/api/discover/frontier-assets/{asset}")});
    sqlx::query("INSERT INTO discover_items(kind,key,status,data) VALUES('frontier','resource-head','draft',$1)")
        .bind(&data).execute(&h.pool).await?;
    let path = format!("/api/discover/frontier-assets/{asset}");
    assert_eq!(call(&h.app,"HEAD",&path,None,Value::Null).await.0,StatusCode::NOT_FOUND);
    sqlx::query("UPDATE discover_items SET status='published' WHERE kind='frontier' AND key='resource-head'").execute(&h.pool).await?;
    assert_eq!(call(&h.app,"HEAD",&path,None,Value::Null).await.0,StatusCode::OK);
    sqlx::query("UPDATE discover_items SET data='{}'::jsonb WHERE kind='frontier' AND key='resource-head'").execute(&h.pool).await?;
    assert_eq!(call(&h.app,"HEAD",&path,None,Value::Null).await.0,StatusCode::NOT_FOUND);
    h.finish().await
}

#[tokio::test]
async fn direct_resource_uploads_require_bound_signed_receipts() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let token = h.member().await?;
    let other = h.member().await?;
    let mut state = h.state.clone();
    let config = Arc::make_mut(&mut state.config);
    config.resource_base_url = Some("https://res.xiangban.xyz".into());
    config.resource_secret = b"0123456789abcdef0123456789abcdef".to_vec();
    let secret = config.resource_secret.clone();
    let app = crate::build_router(state.clone());
    for (kind, resource_kind, mime, ext) in [("image","images","image/png","png"),("attachment","attachments","application/octet-stream","zip")] {
        let hash = "a".repeat(64);
        let (status, ticket) = call(&app,"POST","/api/resource-uploads/ticket",Some(&token),json!({
            "kind":kind,"name":format!("asset.{ext}"),"mime":mime,"size":8,"sha256":hash,
        })).await;
        assert_eq!(status,StatusCode::OK,"{ticket}");
        let id = ticket["id"].as_str().unwrap();
        let headers = &ticket["headers"];
        let expiry = headers["X-Expiration"].as_str().unwrap();
        let mut upload_message = format!("POST\n{resource_kind}/{id}\n{hash}\n8\n{expiry}");
        if kind == "attachment" { upload_message.push_str("\nzip"); }
        assert!(crate::content::verify_callback(upload_message.as_bytes(),headers["X-Signature"].as_str().unwrap(),&secret));
        let object = format!("forum-content/objects/{}.{ext}","b".repeat(32));
        let receipt_message = format!("RECEIPT\n{resource_kind}/{id}\n{hash}\n8\n{object}");
        let signature = crate::content::sign_callback(receipt_message.as_bytes(),&secret);
        let mut receipt = json!({"object_key":object,"sha256":hash,"size":8,"signature":signature});
        let complete = format!("/api/resource-uploads/{id}/complete");
        assert_eq!(call(&app,"POST",&complete,Some(&other),receipt.clone()).await.0,StatusCode::NOT_FOUND);
        receipt["signature"] = json!("0".repeat(64));
        assert_eq!(call(&app,"POST",&complete,Some(&token),receipt.clone()).await.0,StatusCode::FORBIDDEN);
        receipt["signature"] = json!(signature);
        let (status, result) = call(&app,"POST",&complete,Some(&token),receipt.clone()).await;
        assert_eq!(status,StatusCode::OK,"{result}");
        assert_eq!(result["id"],id);
        assert_eq!(call(&app,"POST",&complete,Some(&token),receipt).await.0,StatusCode::OK);
        if kind == "image" { assert_eq!(call(&app,"HEAD",&format!("/api/images/{id}"),None,Value::Null).await.0,StatusCode::OK); }
    }
    h.finish().await
}

#[tokio::test]
async fn contact_qr_head_identifies_current_public_bytes_and_respects_visibility() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    let uri = "/api/contact/qr/qq-group";
    let response = h.app.clone().oneshot(Request::builder().uri(uri).body(Body::empty())?).await?;
    assert_eq!(response.status(),StatusCode::OK);
    let expected = response.headers()["x-resource-sha256"].to_str()?.to_owned();
    let bytes = to_bytes(response.into_body(),6 * 1024 * 1024).await?;
    let actual = ring::digest::digest(&ring::digest::SHA256,&bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect::<String>();
    assert_eq!(expected,actual);
    let head = h.app.clone().oneshot(Request::builder().method("HEAD").uri(uri).body(Body::empty())?).await?;
    assert_eq!(head.headers()["x-resource-sha256"],expected.as_str());
    assert!(to_bytes(head.into_body(),1).await?.is_empty());
    let (_,mut config) = call(&h.app,"GET","/api/contact",None,Value::Null).await;
    config["qqGroupQr"] = json!(false);
    sqlx::query("INSERT INTO contact_settings (id,config) VALUES (true,$1) ON CONFLICT (id) DO UPDATE SET config=EXCLUDED.config")
        .bind(sqlx::types::Json(config)).execute(&h.pool).await?;
    let head = h.app.clone().oneshot(Request::builder().method("HEAD").uri(uri).body(Body::empty())?).await?;
    assert_eq!(head.status(),StatusCode::NOT_FOUND);
    h.finish().await
}

#[tokio::test]
async fn direct_uploads_reach_real_resource_service_without_forum_file_copy() -> Result<(), Box<dyn std::error::Error>> {
    use axum::{extract::{Path, State}, Json};
    let Ok(base) = std::env::var("TEST_DIRECT_RESOURCE_BASE") else { return Ok(()) };
    let Some(h) = harness().await? else { return Ok(()) };
    let token = h.member().await?;
    let mut state = h.state.clone();
    let config = Arc::make_mut(&mut state.config);
    config.resource_base_url = Some(base);
    config.resource_secret = b"0123456789abcdef0123456789abcdef".to_vec();
    let app = crate::build_router(state.clone());
    for (kind, name, mime, bytes) in [
        ("image","picture.png","image/png",b"\x89PNG\r\n\x1a\n".as_slice()),
        ("attachment","private.zip","application/octet-stream",b"private-test-file".as_slice()),
    ] {
        let hash = ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().iter().map(|b|format!("{b:02x}")).collect::<String>();
        let (status,ticket) = call(&app,"POST","/api/resource-uploads/ticket",Some(&token),json!({"kind":kind,"name":name,"mime":mime,"size":bytes.len(),"sha256":hash})).await;
        assert_eq!(status,StatusCode::OK,"{ticket}");
        let id = ticket["id"].as_str().unwrap();
        let mut upload = reqwest::Client::new().post(ticket["upload_url"].as_str().unwrap());
        for (key,value) in ticket["headers"].as_object().unwrap() { upload=upload.header(key,value.as_str().unwrap()); }
        let response = upload.body(bytes.to_vec()).send().await?;
        assert_eq!(response.status(),StatusCode::CREATED);
        let receipt: Value = response.json().await?;
        let (status,stored) = call(&app,"POST",&format!("/api/resource-uploads/{id}/complete"),Some(&token),receipt).await;
        assert_eq!(status,StatusCode::OK,"{stored}");
        assert!(!h.state.config.upload_dir.join(if kind=="image" {format!("img-{id}")} else {id.to_string()}).exists());
        if kind == "image" {
            assert_eq!(call(&app,"HEAD",&format!("/api/images/{id}"),None,Value::Null).await.0,StatusCode::OK);
        } else {
            assert_ne!(call(&app,"GET",&format!("/api/attachments/{id}"),None,Value::Null).await.0,StatusCode::OK);
            let req = Request::builder().uri(format!("/api/attachments/{id}")).header("authorization",format!("Bearer {token}")).body(Body::empty())?;
            let response = app.clone().oneshot(req).await?;
            assert_eq!(response.status(),StatusCode::TEMPORARY_REDIRECT);
            let url = response.headers()["location"].to_str()?;
            let download = reqwest::get(url).await?;
            assert_eq!(download.status(),StatusCode::OK);
            assert_eq!(download.bytes().await?.as_ref(),bytes);
            // Paid attachments stored only on the resource host still check
            // access and existence before charging the buyer.
            let attachment_id = Uuid::parse_str(id)?;
            let (owner,): (Uuid,) = sqlx::query_as("SELECT uid FROM forum_direct_uploads WHERE id=$1").bind(attachment_id).fetch_one(&h.pool).await?;
            let buyer = Uuid::new_v4();
            sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'buyer',1)").bind(buyer).execute(&h.pool).await?;
            crate::points::on_visit(&h.pool,buyer,None).await?;
            let (topic,): (i64,) = sqlx::query_as("INSERT INTO topics(author_uid,category_id,title,body_md,body_html) VALUES($1,(SELECT id FROM categories LIMIT 1),'test','body','body') RETURNING id")
                .bind(owner).fetch_one(&h.pool).await?;
            sqlx::query("UPDATE attachments SET topic_id=$2,price=3 WHERE id=$1").bind(attachment_id).bind(topic).execute(&h.pool).await?;
            let buyer_user = crate::users::find(&h.pool,buyer).await?.unwrap();
            assert!(crate::handlers::attachments::download(State(state.clone()),crate::auth::MaybeUser(Some(buyer_user.clone())),Path(attachment_id)).await.is_err());
            let auth = crate::auth::AuthUser { user:buyer_user, token:"test".into() };
            let purchase = crate::handlers::attachments::purchase(State(state.clone()),auth,Path(attachment_id),Json(crate::handlers::attachments::PurchaseInput {price:3})).await?;
            assert_eq!(purchase.0["charged"],3);
            let buyer_user = crate::users::find(&h.pool,buyer).await?.unwrap();
            let bought = crate::handlers::attachments::download(State(state.clone()),crate::auth::MaybeUser(Some(buyer_user)),Path(attachment_id)).await?;
            assert_eq!(bought.status(),StatusCode::TEMPORARY_REDIRECT);
        }
    }
    h.finish().await
}

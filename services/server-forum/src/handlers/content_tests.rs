use crate::{auth::JwksVerifier, config::AppConfig, content, state::AppState};
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

async fn callback(app: &axum::Router, secret: &[u8], body: Value, sign: bool) -> StatusCode {
    let raw = body.to_string();
    let sig = if sign { content::sign_callback(raw.as_bytes(), secret) } else { "00".repeat(32) };
    let req = Request::builder()
        .method("POST")
        .uri("/api/internal/pack-uploaded")
        .header("content-type", "application/json")
        .header("x-signature", sig)
        .body(Body::from(raw))
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

fn manifest(kind: &str, id: &str, version: &str) -> Value {
    json!({ "schemaVersion": 1, "kind": kind, "id": id, "name": "测试包", "version": version })
}

/// 需要 TEST_DATABASE_URL；每次使用独立 schema，和其他集成测试一致。
#[tokio::test]
async fn pack_lifecycle_upload_review_download() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return Ok(());
    };
    let schema = format!("content_test_{}", Uuid::new_v4().simple());
    let admin_pool = crate::db::connect(&url, None).await?;
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin_pool).await?;
    let pool = crate::db::connect(&url, Some(&schema)).await?;
    crate::db::migrate(&pool).await?;
    crate::db::migrate(&pool).await?;
    crate::db::seed(&pool).await?;

    let mut config = AppConfig::from_env();
    config.resource_base_url = Some("https://res.test".into());
    config.resource_secret = b"0123456789abcdef0123456789abcdef".to_vec();
    config.admin_user = Some("root".into());
    config.admin_password = Some("correct-horse-battery".into());
    config.daily_pack_publishes = 3;
    crate::handlers::admin::seed_admin(&pool, &config).await?;
    let secret = config.resource_secret.clone();

    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
    let x = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
    let verifier = JwksVerifier::with_static_key("iss".into(), "aud".into(), DecodingKey::from_ed_components(&x).unwrap());
    let enc = EncodingKey::from_ed_der(pkcs8.as_ref());
    let user_token = |uid: Uuid| {
        #[derive(serde::Serialize)]
        struct C {
            iss: String,
            sub: String,
            aud: String,
            exp: i64,
        }
        let c = C { iss: "iss".into(), sub: uid.to_string(), aud: "aud".into(), exp: chrono::Utc::now().timestamp() + 600 };
        encode(&Header::new(Algorithm::EdDSA), &c, &enc).unwrap()
    };

    let state = AppState { pool: pool.clone(), config: Arc::new(config), verifier: Arc::new(verifier), http: reqwest::Client::new() };
    let app = crate::build_router(state);
    let author = Uuid::new_v4();
    let other = Uuid::new_v4();
    let (author_t, other_t) = (user_token(author), user_token(other));
    let sha = "a".repeat(64);

    // 凭证：类型、大小、哈希都要合法。
    let (s, _) = call(&app, "POST", "/api/packs/upload-ticket", None, json!({})).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, _) = call(&app, "POST", "/api/packs/upload-ticket", Some(&author_t), json!({"kind":"effect","size":2*1024*1024,"sha256":sha})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "effect 超过 1MB 应拒绝");
    let (s, _) = call(&app, "POST", "/api/packs/upload-ticket", Some(&author_t), json!({"kind":"nope","size":10,"sha256":sha})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, t) = call(&app, "POST", "/api/packs/upload-ticket", Some(&author_t), json!({"kind":"effect","size":1000,"sha256":sha})).await;
    assert_eq!(s, StatusCode::OK, "{t}");
    let file_key = t["file_key"].as_str().unwrap().to_string();
    let claims = content::verify_upload_ticket(t["ticket"].as_str().unwrap(), chrono::Utc::now().timestamp(), &secret)?;
    assert_eq!(claims.file_key, file_key);
    assert_eq!(claims.uid, author.to_string());

    // 资源机还没确认前不能发布。
    let publish = json!({"kind":"effect","title":"夕阳特效","summary":"桌面夕阳","tags":["特效"," 特效 ","桌面"],
        "file_key":file_key});
    let (s, _) = call(&app, "POST", "/api/packs", Some(&author_t), publish.clone()).await;
    assert_eq!(s, StatusCode::CONFLICT);

    // 回调：签名不对拒绝；哈希或大小对不上拒绝；正确才通过。
    assert_eq!(callback(&app, &secret, json!({"file_key":file_key,"size":1000,"sha256":sha}), false).await, StatusCode::UNAUTHORIZED);
    assert_eq!(callback(&app, &secret, json!({"file_key":file_key,"size":999,"sha256":sha,"manifest":manifest("effect","sunset","1.0.0")}), true).await, StatusCode::CONFLICT);
    assert_eq!(callback(&app, &secret, json!({"file_key":file_key,"size":1000,"sha256":sha}), true).await, StatusCode::BAD_REQUEST, "内容包回调必须带 manifest");
    assert_eq!(callback(&app, &secret, json!({"file_key":file_key,"size":1000,"sha256":sha,"manifest":manifest("reminder","sunset","1.0.0")}), true).await, StatusCode::BAD_REQUEST, "manifest 类型要与上传类型一致");
    assert_eq!(callback(&app, &secret, json!({"file_key":file_key,"size":1000,"sha256":sha,"manifest":manifest("effect","sunset","1.0.0")}), true).await, StatusCode::OK);

    // 别人不能拿走这个文件；作者可以发布，并且是 T2、一键可用（effect）。
    let (s, _) = call(&app, "POST", "/api/packs", Some(&other_t), publish.clone()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let mut wrong = publish.clone();
    wrong["kind"] = json!("reminder");
    let (s, _) = call(&app, "POST", "/api/packs", Some(&author_t), wrong).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "上传时的类型与发布类型不一致应拒绝");
    let (s, p) = call(&app, "POST", "/api/packs", Some(&author_t), publish.clone()).await;
    assert_eq!(s, StatusCode::OK, "{p}");
    let id = p["id"].as_i64().unwrap();
    assert_eq!(p["trust_level"], 2);
    let (s, _) = call(&app, "POST", "/api/packs", Some(&author_t), publish).await;
    assert_eq!(s, StatusCode::CONFLICT, "文件不能重复使用");

    // 封面：只能用自己上传并确认过的图片，不能随便填键。
    let (s, _) = call(&app, "POST", "/api/packs/upload-ticket", Some(&author_t), json!({"kind":"cover","size":3*1024*1024,"sha256":sha})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "封面最大 2MB");
    let (s, ct) = call(&app, "POST", "/api/packs/upload-ticket", Some(&author_t), json!({"kind":"cover","size":5000,"sha256":"e".repeat(64)})).await;
    assert_eq!(s, StatusCode::OK, "{ct}");
    let cover_key = ct["file_key"].as_str().unwrap().to_string();
    assert!(cover_key.starts_with("covers/"));
    // 第二个内容包带着还没确认的封面发布：被封面检查拦下，且不消耗文件。
    let sha2 = "f".repeat(64);
    let (_, t3) = call(&app, "POST", "/api/packs/upload-ticket", Some(&author_t), json!({"kind":"effect","size":500,"sha256":sha2})).await;
    let key3 = t3["file_key"].as_str().unwrap().to_string();
    assert_eq!(callback(&app, &secret, json!({"file_key":key3,"size":500,"sha256":sha2,"manifest":manifest("effect","dawn","1.0.0")}), true).await, StatusCode::OK);
    let with_cover = json!({"kind":"effect","title":"晨光","file_key":key3,"cover_key":cover_key});
    let (s, _) = call(&app, "POST", "/api/packs", Some(&author_t), with_cover.clone()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "封面未确认");
    let (s, _) = call(&app, "POST", "/api/packs", Some(&other_t), with_cover.clone()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "别人的封面和文件不能用");
    assert_eq!(callback(&app, &secret, json!({"file_key":cover_key,"size":5000,"sha256":"e".repeat(64)}), true).await, StatusCode::OK);
    let (s, dawn) = call(&app, "POST", "/api/packs", Some(&author_t), with_cover).await;
    assert_eq!(s, StatusCode::OK, "{dawn}");
    let dawn_id = dawn["id"].as_i64().unwrap();
    let (_, dd) = call(&app, "GET", &format!("/api/packs/{dawn_id}"), None, Value::Null).await;
    assert_eq!(dd["cover_url"], format!("https://res.test/{cover_key}"));

    // 列表与详情。
    let (s, list) = call(&app, "GET", "/api/packs?kind=effect&q=%E5%A4%95", None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(list["total"], 1);
    assert_eq!(list["items"][0]["one_click"], true);
    assert_eq!(list["items"][0]["tags"], json!(["特效", "桌面"]));
    let (_, none) = call(&app, "GET", "/api/packs?q=%25", None, Value::Null).await;
    assert_eq!(none["total"], 0, "通配符按字面匹配");
    let (s, detail) = call(&app, "GET", &format!("/api/packs/{id}"), None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(detail["versions"].as_array().unwrap().len(), 1);

    // 下载地址：签名可被资源机一侧校验。
    let (s, d) = call(&app, "GET", &format!("/api/packs/{id}/download"), None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(d["sha256"], sha);
    let url = d["url"].as_str().unwrap();
    let (path, query) = url.strip_prefix("https://res.test/").unwrap().split_once('?').unwrap();
    let exp: i64 = query.split('&').find_map(|kv| kv.strip_prefix("exp=")).unwrap().parse()?;
    let sig = query.split('&').find_map(|kv| kv.strip_prefix("sig=")).unwrap();
    assert!(content::verify_download(path, exp, sig, chrono::Utc::now().timestamp(), &secret));
    assert!(!content::verify_download("packs/other.zip", exp, sig, chrono::Utc::now().timestamp(), &secret));

    // 应用计数。
    let (s, _) = call(&app, "POST", &format!("/api/packs/{id}/applied"), None, json!({"version":1})).await;
    assert_eq!(s, StatusCode::OK);
    let (_, hot) = call(&app, "GET", "/api/packs?sort=hot", None, Value::Null).await;
    assert_eq!(hot["items"][0]["apply_count"], 1);

    // 新版本：只有作者；id 必须一致；版本号递增。
    let (_, t2) = call(&app, "POST", "/api/packs/upload-ticket", Some(&author_t), json!({"kind":"effect","size":2000,"sha256":"b".repeat(64)})).await;
    let key2 = t2["file_key"].as_str().unwrap().to_string();
    assert_eq!(callback(&app, &secret, json!({"file_key":key2,"size":2000,"sha256":"b".repeat(64),"manifest":manifest("effect","sunset","1.1.0")}), true).await, StatusCode::OK);
    let v2 = json!({"file_key":key2,"changelog":"更亮"});
    let (s, _) = call(&app, "POST", &format!("/api/packs/{id}/versions"), Some(&other_t), v2.clone()).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, v) = call(&app, "POST", &format!("/api/packs/{id}/versions"), Some(&author_t), v2).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["version"], 2);
    let (_, d2) = call(&app, "GET", &format!("/api/packs/{id}/download"), None, Value::Null).await;
    assert_eq!(d2["version"], 2);
    assert_eq!(d2["sha256"], "b".repeat(64));

    // 方案只引用已上架的内容。
    let (s, _) = call(&app, "POST", "/api/schemes", Some(&other_t), json!({"title":"我的桌面","items":[{"pack_id":99999}]})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, sc) = call(&app, "POST", "/api/schemes", Some(&other_t), json!({"title":"我的桌面","items":[{"pack_id":id}]})).await;
    assert_eq!(s, StatusCode::OK, "{sc}");
    let (_, schemes) = call(&app, "GET", "/api/schemes", None, Value::Null).await;
    assert_eq!(schemes["items"][0]["items"][0]["slug"], "sunset");

    // 后台：登录、审核队列、通过审核后升为 T1，下架后前台不可见。
    let (s, _) = call(&app, "GET", "/api/admin/moderation", Some(&author_t), Value::Null).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, login) = call(&app, "POST", "/api/admin/login", None, json!({"username":"root","password":"correct-horse-battery"})).await;
    assert_eq!(s, StatusCode::OK, "{login}");
    let admin_t = login["token"].as_str().unwrap().to_string();
    let (_, queue) = call(&app, "GET", "/api/admin/moderation", Some(&admin_t), Value::Null).await;
    assert_eq!(queue["items"].as_array().unwrap().len(), 2, "两个内容包各一条，新版本不重复入队");
    let (s, r) = call(&app, "POST", &format!("/api/admin/packs/{id}/review"), Some(&admin_t), json!({"action":"approve","note":"ok"})).await;
    assert_eq!(s, StatusCode::OK, "{r}");
    assert_eq!(r["trust_level"], 1);
    let (_, queue) = call(&app, "GET", "/api/admin/moderation", Some(&admin_t), Value::Null).await;
    assert_eq!(queue["items"].as_array().unwrap().len(), 1);
    let (s, _) = call(&app, "POST", &format!("/api/admin/packs/{id}/review"), Some(&admin_t), json!({"action":"bogus"})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    call(&app, "POST", &format!("/api/admin/packs/{id}/review"), Some(&admin_t), json!({"action":"remove"})).await;
    let (s, _) = call(&app, "GET", &format!("/api/packs/{id}"), None, Value::Null).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = call(&app, "GET", &format!("/api/packs/{id}"), Some(&author_t), Value::Null).await;
    assert_eq!(s, StatusCode::OK, "作者仍能看到自己下架的内容");

    // 官方登记（T0）+ 镜像。
    let official = json!({"kind":"wallpaper","title":"官方壁纸","manifest":manifest("wallpaper","aurora","2.0.0"),
        "file_key":"official/aurora-2.0.0.zip","sha256":"c".repeat(64),"size":20*1024*1024,
        "mirrors":[{"name":"modelscope","url":"https://modelscope.cn/x/aurora.zip"}]});
    let (s, o) = call(&app, "POST", "/api/admin/packs", Some(&admin_t), official.clone()).await;
    assert_eq!(s, StatusCode::OK, "{o}");
    let oid = o["id"].as_i64().unwrap();
    let (s, _) = call(&app, "POST", "/api/admin/packs", Some(&admin_t), official).await;
    assert_eq!(s, StatusCode::CONFLICT, "id 重复");
    let (_, od) = call(&app, "GET", &format!("/api/packs/{oid}/download"), None, Value::Null).await;
    assert_eq!(od["mirrors"][0]["name"], "modelscope");
    assert_eq!(od["trust_level"], 0);
    let mut big = json!({"kind":"effect","title":"x","manifest":manifest("effect","big","1"),"file_key":"a/b.zip","sha256":sha,"size":5*1024*1024});
    let (s, _) = call(&app, "POST", "/api/admin/packs", Some(&admin_t), big.clone()).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    big["size"] = json!(100);
    big["mirrors"] = json!([{"name":"x","url":"http://insecure"}]);
    let (s, _) = call(&app, "POST", "/api/admin/packs", Some(&admin_t), big).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "镜像必须是 https");

    // 项目与周刊：草稿不可见，发布后条目按引用补全，下架的条目自动隐去。
    let (s, _) = call(&app, "PUT", "/api/admin/projects/xiaoban", Some(&admin_t), json!({"name":"小伴","summary":"桌面助手","body_md":"**hi**","links":[{"title":"官网","url":"https://x"}]})).await;
    assert_eq!(s, StatusCode::OK);
    let (_, projects) = call(&app, "GET", "/api/projects", None, Value::Null).await;
    assert_eq!(projects["items"].as_array().unwrap().len(), 1);
    let (s, _) = call(&app, "GET", "/api/admin/projects", Some(&author_t), Value::Null).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (_, admin_projects) = call(&app, "GET", "/api/admin/projects", Some(&admin_t), Value::Null).await;
    assert_eq!(admin_projects["items"][0]["body_md"], "**hi**");
    let (s, pr) = call(&app, "GET", "/api/projects/xiaoban", None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert!(pr["body_html"].as_str().unwrap().contains("<strong>"));

    let week = "2026-W44";
    let issue = json!({"title":"第 44 期","summary":"本周精选","items":[{"type":"pack","ref":oid,"note":"推荐"},{"type":"pack","ref":id,"note":"已下架"},{"type":"text","title":"说明"}]});
    let (s, _) = call(&app, "PUT", &format!("/api/admin/weekly/{week}"), Some(&admin_t), issue.clone()).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = call(&app, "GET", &format!("/api/weekly/{week}"), None, Value::Null).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "草稿不对外");
    let (_, drafts) = call(&app, "GET", "/api/admin/weekly", Some(&admin_t), Value::Null).await;
    assert_eq!(drafts["items"][0]["status"], "draft", "后台能看到草稿");
    let mut published = issue;
    published["status"] = json!("published");
    call(&app, "PUT", &format!("/api/admin/weekly/{week}"), Some(&admin_t), published).await;
    let (s, w) = call(&app, "GET", &format!("/api/weekly/{week}"), None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(w["items"].as_array().unwrap().len(), 2, "下架的内容包不出现在周刊里");
    assert_eq!(w["items"][0]["title"], "官方壁纸");
    let (s, _) = call(&app, "PUT", "/api/admin/weekly/2026-44", Some(&admin_t), json!({"title":"x"})).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // 分享信息。
    let (s, sh) = call(&app, "GET", &format!("/api/share/pack/{oid}"), None, Value::Null).await;
    assert_eq!(s, StatusCode::OK);
    assert!(sh["app_link"].as_str().unwrap().starts_with("xiaoban://community/apply/wallpaper/"));
    let (s, _) = call(&app, "GET", &format!("/api/share/pack/{id}"), None, Value::Null).await;
    assert_eq!(s, StatusCode::NOT_FOUND);

    // 每日发布配额（测试里为 3）：未使用的凭证也占名额，防止反复申请凭证。
    for i in 0..3 {
        let (s, _) = call(&app, "POST", "/api/packs/upload-ticket", Some(&other_t), json!({"kind":"effect","size":10,"sha256":format!("{:0>64}", i)})).await;
        assert_eq!(s, StatusCode::OK);
    }
    let (s, _) = call(&app, "POST", "/api/packs/upload-ticket", Some(&other_t), json!({"kind":"effect","size":10,"sha256":"d".repeat(64)})).await;
    assert_eq!(s, StatusCode::TOO_MANY_REQUESTS);

    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&admin_pool).await?;
    Ok(())
}

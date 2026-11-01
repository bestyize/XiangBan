use super::*;
use crate::{
    collectors::credentials,
    error::AppError,
    handlers::{admin::AdminIdentity, translation_admin},
    repo_details,
    translation::test_support::{chat_ok, mock},
};
use std::time::Duration;

fn ranks_json() -> Value {
    json!([
        {"full_name":"Owner/Repo","url":"https://github.com/Owner/Repo","description":"A trending project","language":"Rust","language_color":"#dea584","star_total":5000,"star_gain":120,"spark":[]},
        {"full_name":"Other/Thing","url":"https://github.com/Other/Thing","description":"Second","language":"Go","language_color":"","star_total":900,"star_gain":80,"spark":[]},
    ])
}

async fn put_issue(h: &Harness, key: &str, issue_no: i64, ranks: Value, status: &str) -> Result<(), sqlx::Error> {
    let data = json!({"source":"GitHub","metric":"github trending weekly stars","issue_no":issue_no,"week_start":"2026-10-25","week_end":"2026-11-01","as_of":"2026-11-01T10:00:00Z","ranks":ranks});
    sqlx::query("INSERT INTO discover_items(kind,key,status,data) VALUES('repo_issue',$1,$2,$3)").bind(key).bind(status).bind(data).execute(&h.pool).await?;
    Ok(())
}

fn admin_input(origin: &str, key: Option<&str>) -> translation_admin::Input {
    serde_json::from_value(json!({"base_url":format!("{origin}/v1"),"model":"standard","key":key})).unwrap()
}

async fn admin_identity(h: &Harness) -> Result<AdminIdentity, sqlx::Error> {
    let (id,): (Uuid,) = sqlx::query_as("SELECT id FROM admins WHERE username='root'").fetch_one(&h.pool).await?;
    Ok(AdminIdentity { id, username: "root".into() })
}

async fn wait_for(pool: &sqlx::PgPool, sql: &str, what: &str) {
    for _ in 0..60 {
        let (done,): (bool,) = sqlx::query_as(sql).fetch_one(pool).await.unwrap();
        if done {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("等待超时：{what}");
}

#[tokio::test]
async fn repo_detail_serves_card_readme_blocks_and_hides_unpublished_projects() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    put_issue(&h, "github-rolling-2026-10-25", 20261025, json!([{"full_name":"Old/Gone","description":"Dropped","language":"","language_color":"","star_total":10,"star_gain":1,"spark":[]}]), "published").await?;
    put_issue(&h, "github-rolling-2026-11-01", 20261101, ranks_json(), "published").await?;
    put_issue(&h, "github-rolling-2026-11-08", 20261108, json!([{"full_name":"Draft/Only","description":"x","language":"","language_color":"","star_total":1,"star_gain":1,"spark":[]}]), "draft").await?;
    sqlx::query("INSERT INTO repo_details(key,full_name,desc_en,desc_zh,readme_status,readme_md,readme_hash,readme_zh,readme_zh_hash,readme_zh_truncated)
        VALUES('owner/repo','Owner/Repo','A trending project','一个热门项目','ok','# Title\n\nbody **text**\n\n<script>alert(1)</script>\n','h1','# 标题\n\n正文\n','h1',true)")
        .execute(&h.pool).await?;

    let (status, d) = call(&h.app, "GET", "/api/discover/repo/owner/repo", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!((d["full_name"].as_str(), d["owner"].as_str(), d["repo"].as_str()), (Some("Owner/Repo"), Some("Owner"), Some("Repo")));
    assert_eq!((d["rank"].as_i64(), d["is_latest"].as_bool(), d["star_gain"].as_i64(), d["star_total"].as_i64()), (Some(1), Some(true), Some(120), Some(5000)));
    assert_eq!((d["description"].as_str(), d["description_zh"].as_str(), d["url"].as_str()), (Some("A trending project"), Some("一个热门项目"), Some("https://github.com/Owner/Repo")));
    assert_eq!(d["readme"]["status"], "ok");
    assert_eq!(d["readme"]["blocks"].as_array().unwrap().len(), 1, "开头的一级标题已去掉：{d}");
    assert_eq!(d["readme"]["blocks"][0]["x"], "body text");
    assert_eq!(d["readme"]["zh_blocks"][0]["x"], "正文");
    assert_eq!(d["readme"]["zh_truncated"], true);
    assert!(!d.to_string().contains("alert(1)"), "README 里的脚本不能原样返回");
    assert_eq!(call(&h.app, "GET", "/api/discover/repo/OWNER/repo", None, Value::Null).await.0, StatusCode::OK, "大小写不敏感");

    // 没有抓过 README 的项目：卡片照常，README 标为不可用。
    let (_, other) = call(&h.app, "GET", "/api/discover/repo/other/thing", None, Value::Null).await;
    assert_eq!((other["rank"].as_i64(), other["readme"]["status"].as_str(), other["description_zh"].as_str()), (Some(2), Some("unavailable"), Some("")));
    assert!(other["readme"]["zh_blocks"].is_null());

    // 掉出最新榜单的项目仍可打开，并标明不在本周榜。
    let (status, gone) = call(&h.app, "GET", "/api/discover/repo/old/gone", None, Value::Null).await;
    assert_eq!(status, StatusCode::OK, "{gone}");
    assert_eq!(gone["is_latest"], false);

    for path in ["/api/discover/repo/draft/only", "/api/discover/repo/no/such", "/api/discover/repo/a/..", "/api/discover/repo/-bad/repo"] {
        assert_eq!(call(&h.app, "GET", path, None, Value::Null).await.0, StatusCode::NOT_FOUND, "{path}");
    }
    h.finish().await
}

#[tokio::test]
async fn repo_comments_are_scoped_moderated_and_owned_by_their_authors() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    put_issue(&h, "github-rolling-2026-11-01", 20261101, ranks_json(), "published").await?;
    let path = "/api/discover/repo/owner/repo/comments";
    assert_eq!(call(&h.app, "POST", path, None, json!({"body":"留言"})).await.0, StatusCode::UNAUTHORIZED);
    let author = h.member().await?;
    let other = h.member().await?;
    assert_eq!(call(&h.app, "POST", path, Some(&author), json!({"body":"x"})).await.0, StatusCode::BAD_REQUEST, "太短");
    let (status, created) = call(&h.app, "POST", path, Some(&author), json!({"body":"第一条评论"})).await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let id = created["id"].as_i64().unwrap();
    assert_eq!(call(&h.app, "POST", path, Some(&author), json!({"body":"发得太快"})).await.0, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(call(&h.app, "POST", path, Some(&other), json!({"body":"回复一下","reply_to":id})).await.0, StatusCode::OK);
    let third = h.member().await?;
    assert_eq!(call(&h.app, "POST", path, Some(&third), json!({"body":"回复不存在的","reply_to":999999})).await.0, StatusCode::BAD_REQUEST);

    // 评论按项目隔离：同一个 id 在别的项目下不存在。
    let other_path = "/api/discover/repo/other/thing/comments";
    assert_eq!(call(&h.app, "PUT", &format!("{other_path}/{id}/like"), Some(&other), json!({})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app, "PUT", &format!("{path}/{id}/like"), Some(&other), json!({})).await.0, StatusCode::OK);
    assert_eq!(call(&h.app, "PUT", &format!("{path}/{id}/like"), Some(&other), json!({})).await.0, StatusCode::OK, "重复点赞是幂等的");
    let (_, list) = call(&h.app, "GET", path, Some(&other), Value::Null).await;
    assert_eq!(list["total"], 2);
    assert_eq!((list["items"][1]["like_count"].as_i64(), list["items"][1]["liked"].as_bool(), list["items"][1]["mine"].as_bool()), (Some(1), Some(true), Some(false)));
    assert_eq!((list["items"][0]["reply_to"].as_i64(), list["items"][0]["mine"].as_bool()), (Some(id), Some(true)));
    let (_, guest) = call(&h.app, "GET", path, None, Value::Null).await;
    assert_eq!((guest["total"].as_i64(), guest["items"][1]["liked"].as_bool()), (Some(2), Some(false)));
    assert_eq!(call(&h.app, "DELETE", &format!("{path}/{id}/like"), Some(&other), json!({})).await.0, StatusCode::OK);

    // 屏蔽的作者不出现在屏蔽者的列表里；禁言用户不能发。
    let (author_uid,): (Uuid,) = sqlx::query_as("SELECT author_uid FROM repo_comments WHERE id=$1").bind(id).fetch_one(&h.pool).await?;
    let (other_uid,): (Uuid,) = sqlx::query_as("SELECT author_uid FROM repo_comments WHERE reply_to=$1").bind(id).fetch_one(&h.pool).await?;
    sqlx::query("INSERT INTO user_blocks(blocker,blocked) VALUES($1,$2)").bind(other_uid).bind(author_uid).execute(&h.pool).await?;
    let (_, filtered) = call(&h.app, "GET", path, Some(&other), Value::Null).await;
    assert_eq!(filtered["total"], 1);
    sqlx::query("DELETE FROM user_blocks").execute(&h.pool).await?;
    sqlx::query("UPDATE users SET status='muted' WHERE uid=$1").bind(other_uid).execute(&h.pool).await?;
    assert_eq!(call(&h.app, "POST", path, Some(&other), json!({"body":"禁言后发言"})).await.0, StatusCode::FORBIDDEN);
    sqlx::query("UPDATE users SET status='active' WHERE uid=$1").bind(other_uid).execute(&h.pool).await?;

    assert_eq!(call(&h.app, "DELETE", &format!("{path}/{id}"), Some(&other), json!({})).await.0, StatusCode::NOT_FOUND, "不能删别人的评论");
    assert_eq!(call(&h.app, "DELETE", &format!("{path}/{id}"), Some(&author), json!({})).await.0, StatusCode::OK);
    let (_, after) = call(&h.app, "GET", path, None, Value::Null).await;
    assert_eq!(after["total"], 1);

    // 项目不在任何已发布榜单里时，评论接口一律 404。
    assert_eq!(call(&h.app, "GET", "/api/discover/repo/no/such/comments", None, Value::Null).await.0, StatusCode::NOT_FOUND);
    assert_eq!(call(&h.app, "POST", "/api/discover/repo/no/such/comments", Some(&author), json!({"body":"没有这个项目"})).await.0, StatusCode::NOT_FOUND);
    sqlx::query("UPDATE discover_items SET status='draft' WHERE kind='repo_issue'").execute(&h.pool).await?;
    assert_eq!(call(&h.app, "GET", path, None, Value::Null).await.0, StatusCode::NOT_FOUND);
    h.finish().await
}

#[tokio::test]
async fn translation_settings_are_validated_encrypted_and_never_returned() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    credentials::TEST_MASTER_KEY.get_or_init(|| [9u8; 32]);
    let admin = admin_identity(&h).await?;
    let secret = "sk-test-secret-123456";

    assert_eq!(call(&h.app, "GET", "/api/admin/translation", None, Value::Null).await.0, StatusCode::UNAUTHORIZED);
    let (_, empty) = call(&h.app, "GET", "/api/admin/translation", Some(&h.admin_t), Value::Null).await;
    assert_eq!((empty["configured"].as_bool(), empty["options"]["readme_max_chars"].as_u64()), (Some(false), Some(8000)));
    assert_eq!(call(&h.app, "POST", "/api/admin/translation/test", Some(&h.admin_t), json!({})).await.0, StatusCode::BAD_REQUEST, "未配置时不能测试");

    // 接口路由只收公网 https：本机地址和 http 一律拒绝。
    let (status, _) = call(&h.app, "PUT", "/api/admin/translation", Some(&h.admin_t), json!({"base_url":"http://127.0.0.1:9/v1","model":"standard","key":secret})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 上游验证失败：不入库，错误里没有密钥和上游正文。
    let bad = mock(|_| (401, r#"{"error":"upstream says sk-test-secret-123456"}"#.to_string())).await;
    let err = translation_admin::apply(&h.state, &admin, admin_input(&bad.origin, Some(secret)), true).await.err().unwrap();
    let text = match err {
        AppError::BadRequest(m) => m,
        other => panic!("{other:?}"),
    };
    assert!(text.contains("401") && !text.contains(secret) && !text.contains("upstream says"), "{text}");
    assert!(crate::translation::stored(&h.pool).await?.is_none());

    // 验证通过才加密保存，只回尾号。
    let good = mock(|_| (200, chat_ok("面向 Claude Code 的通用 SEO 技能。"))).await;
    let saved = translation_admin::apply(&h.state, &admin, admin_input(&good.origin, Some(secret)), true).await.map_err(|e| format!("{e:?}"))?;
    assert_eq!(saved.0["hint"], "••••3456");
    let (sealed,): (Vec<u8>,) = sqlx::query_as("SELECT credential FROM translation_settings").fetch_one(&h.pool).await?;
    assert!(!String::from_utf8_lossy(&sealed).contains(secret) && sealed.len() > 30);
    let (_, shown) = call(&h.app, "GET", "/api/admin/translation", Some(&h.admin_t), Value::Null).await;
    assert_eq!((shown["configured"].as_bool(), shown["model"].as_str(), shown["hint"].as_str()), (Some(true), Some("standard"), Some("••••3456")));
    assert!(!shown.to_string().contains(secret) && shown.get("key").is_none() && shown.get("credential").is_none());
    let verify = good.requests.lock().unwrap()[0].to_lowercase();
    assert!(verify.contains(&format!("authorization: bearer {secret}")) && verify.starts_with("post /v1/chat/completions "));

    // 不带密钥再保存：沿用旧密钥，改模型和选项；选项越界被拒。
    let input: translation_admin::Input = serde_json::from_value(json!({"base_url":format!("{}/v1", good.origin),"model":"premium","options":{"readme_max_chars":16000,"translate_readme":false}}))?;
    let _ = translation_admin::apply(&h.state, &admin, input, true).await.map_err(|e| format!("{e:?}"))?;
    let stored = crate::translation::stored(&h.pool).await?.unwrap();
    assert_eq!((stored.model.as_str(), stored.options.readme_max_chars, stored.options.translate_readme, stored.hint.as_str()), ("premium", 16000, false, "••••3456"));
    let last = good.requests.lock().unwrap().last().unwrap().to_lowercase();
    assert!(last.contains(&format!("authorization: bearer {secret}")), "沿用旧密钥验证");
    let bad_options: translation_admin::Input = serde_json::from_value(json!({"base_url":format!("{}/v1", good.origin),"model":"premium","options":{"max_new":99}}))?;
    assert!(translation_admin::apply(&h.state, &admin, bad_options, true).await.is_err());

    let (status, tested) = call(&h.app, "POST", "/api/admin/translation/test", Some(&h.admin_t), json!({})).await;
    assert_eq!(status, StatusCode::OK, "{tested}");
    assert_eq!((tested["translated"].as_str(), tested["source"].as_str()), (Some("面向 Claude Code 的通用 SEO 技能。"), Some(crate::translation::SAMPLE)));
    let (_, again) = call(&h.app, "GET", "/api/admin/translation", Some(&h.admin_t), Value::Null).await;
    assert_eq!(again["recent"][0]["kind"], "test");
    let (audits,): (i64,) = sqlx::query_as("SELECT count(*) FROM admin_audit WHERE action='translation.configure'").fetch_one(&h.pool).await?;
    assert_eq!(audits, 2);
    h.finish().await
}

#[tokio::test]
async fn enrich_fetches_readmes_translates_new_text_once_and_degrades_without_a_translator() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    credentials::TEST_MASTER_KEY.get_or_init(|| [9u8; 32]);
    let github = mock(|req| {
        let line = req.lines().next().unwrap_or("").to_lowercase();
        if line.starts_with("get /repos/owner/has/readme ") {
            (200, "# Has\n\nAnti-AI-slop design skill.\n\n## Install\n\n```bash\nrun it\n```\n".to_string())
        } else if line.starts_with("get /repos/owner/none/readme ") {
            (404, "{}".to_string())
        } else {
            (500, "{}".to_string())
        }
    })
    .await;
    let translator = mock(|req| {
        let body = req.split("\r\n\r\n").nth(1).unwrap_or("");
        if body.contains("Markdown 文档") {
            (200, chat_ok("# 有\n\n去 AI 味的设计技能。\n\n## 安装\n\n```bash\nrun it\n```"))
        } else if body.contains("Anti-AI-slop design skill") {
            (200, chat_ok("“去 AI 味的设计技能”"))
        } else {
            (200, chat_ok("占位译文"))
        }
    })
    .await;
    let http = credentials::http_client()?;
    let new_ranks = || vec![
        json!({"full_name":"Owner/Has","description":"Anti-AI-slop design skill"}),
        json!({"full_name":"Owner/None","description":"Nothing to read here"}),
        json!({"full_name":"Owner/Zh","description":"面向开发者的中文简介"}),
        json!({"full_name":"Owner/Empty","description":""}),
    ];

    // 没配翻译服务：README 照常保存，榜单项没有中文简介，也不报错。
    let mut ranks = new_ranks();
    repo_details::enrich_with(&h.pool, &http, &mut ranks, &github.origin).await;
    assert!(ranks.iter().all(|r| r.get("description_zh").is_none()));
    let (has, none): (String, String) = sqlx::query_as("SELECT (SELECT readme_status FROM repo_details WHERE key='owner/has'),(SELECT readme_status FROM repo_details WHERE key='owner/none')").fetch_one(&h.pool).await?;
    assert_eq!((has.as_str(), none.as_str()), ("ok", "none"));
    let (unavailable,): (String,) = sqlx::query_as("SELECT readme_status FROM repo_details WHERE key='owner/zh'").fetch_one(&h.pool).await?;
    assert_eq!(unavailable, "unavailable", "抓取失败的项目不伪造成没有 README");

    // 配好翻译服务后：中文简介写回榜单项，README 在后台翻译。
    let admin = admin_identity(&h).await?;
    let _ = translation_admin::apply(&h.state, &admin, admin_input(&translator.origin, Some("sk-test-secret-123456")), true).await.map_err(|e| format!("{e:?}"))?;
    let setup_calls = translator.requests.lock().unwrap().len();
    let mut ranks = new_ranks();
    repo_details::enrich_with(&h.pool, &http, &mut ranks, &github.origin).await;
    assert_eq!(ranks[0]["description_zh"], "去 AI 味的设计技能", "引号被去掉");
    assert_eq!(ranks[1]["description_zh"], "占位译文");
    assert_eq!(ranks[2]["description_zh"], "面向开发者的中文简介", "本来就是中文的直接沿用，不调用翻译");
    assert!(ranks[3].get("description_zh").is_none(), "空简介不翻译");
    wait_for(&h.pool, "SELECT readme_zh_hash=readme_hash AND readme_zh<>'' FROM repo_details WHERE key='owner/has'", "README 译文写入").await;
    let (zh_md, truncated): (String, bool) = sqlx::query_as("SELECT readme_zh,readme_zh_truncated FROM repo_details WHERE key='owner/has'").fetch_one(&h.pool).await?;
    assert!(zh_md.contains("去 AI 味") && !truncated);
    let (none_zh,): (String,) = sqlx::query_as("SELECT readme_zh FROM repo_details WHERE key='owner/none'").fetch_one(&h.pool).await?;
    assert_eq!(none_zh, "", "没有 README 的不翻译");

    // 内容没变就不再调用翻译（不重复计费）。
    let calls_after_first = translator.requests.lock().unwrap().len();
    assert_eq!(calls_after_first - setup_calls, 3, "两条简介 + 一份 README");
    let mut again = new_ranks();
    repo_details::enrich_with(&h.pool, &http, &mut again, &github.origin).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(translator.requests.lock().unwrap().len(), calls_after_first);
    assert_eq!(again[0]["description_zh"], "去 AI 味的设计技能");

    // 简介变了才重新翻译。
    let mut changed = vec![json!({"full_name":"Owner/Has","description":"Anti-AI-slop design skill, v2"})];
    repo_details::enrich_with(&h.pool, &http, &mut changed, &github.origin).await;
    assert_eq!(changed[0]["description_zh"], "去 AI 味的设计技能");
    assert_eq!(translator.requests.lock().unwrap().len(), calls_after_first + 1);

    // 译文只留文本块，详情接口能直接读出。
    put_issue(&h, "github-rolling-2026-11-01", 20261101, json!([{"full_name":"Owner/Has","description":"Anti-AI-slop design skill","language":"","language_color":"","star_total":5,"star_gain":3,"spark":[]}]), "published").await?;
    let (_, detail) = call(&h.app, "GET", "/api/discover/repo/owner/has", None, Value::Null).await;
    assert_eq!(detail["readme"]["zh_blocks"][0]["x"], "去 AI 味的设计技能。");
    assert_eq!(detail["readme"]["blocks"][0]["x"], "Anti-AI-slop design skill.");
    assert_eq!(detail["readme"]["blocks"][2]["t"], "code");
    let (_, logs) = call(&h.app, "GET", "/api/admin/translation", Some(&h.admin_t), Value::Null).await;
    assert!(logs["recent"].as_array().unwrap().iter().any(|l| l["kind"] == "readme" && l["ok"] == true));
    h.finish().await
}

#[tokio::test]
async fn translation_failures_never_block_enrichment_and_are_logged() -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else { return Ok(()) };
    credentials::TEST_MASTER_KEY.get_or_init(|| [9u8; 32]);
    let github = mock(|_| (200, "# T\n\ntext\n".to_string())).await;
    let good = mock(|_| (200, chat_ok("好"))).await;
    let admin = admin_identity(&h).await?;
    let _ = translation_admin::apply(&h.state, &admin, admin_input(&good.origin, Some("sk-test-secret-123456")), true).await.map_err(|e| format!("{e:?}"))?;
    // 保存后翻译服务坏了：把地址改到一个没人监听的端口。
    sqlx::query("UPDATE translation_settings SET base_url='http://127.0.0.1:1/v1'").execute(&h.pool).await?;
    let http = credentials::http_client()?;
    let mut ranks = vec![json!({"full_name":"Owner/Has","description":"Some English words"})];
    repo_details::enrich_with(&h.pool, &http, &mut ranks, &github.origin).await;
    assert!(ranks[0].get("description_zh").is_none());
    let (status,): (String,) = sqlx::query_as("SELECT readme_status FROM repo_details WHERE key='owner/has'").fetch_one(&h.pool).await?;
    assert_eq!(status, "ok");
    wait_for(&h.pool, "SELECT EXISTS(SELECT 1 FROM translation_log WHERE kind='readme' AND NOT ok) AND NOT EXISTS(SELECT 1 FROM repo_details WHERE key='owner/has' AND translating_until IS NOT NULL)", "README 失败记录且释放认领").await;
    let (logs,): (i64,) = sqlx::query_as("SELECT count(*) FROM translation_log WHERE NOT ok").fetch_one(&h.pool).await?;
    assert!(logs >= 2);
    let (note,): (String,) = sqlx::query_as("SELECT note FROM translation_log WHERE NOT ok LIMIT 1").fetch_one(&h.pool).await?;
    assert!(!note.contains("127.0.0.1") && !note.contains("sk-test"), "{note}");
    h.finish().await
}

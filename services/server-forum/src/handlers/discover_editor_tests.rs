use super::*;
use crate::handlers::discover_editor::publish_due;
use chrono::{Duration, Utc};

fn news(title: &str, revision: i64) -> Value {
    json!({"data":{"title":title,"source":"官方","url":"https://example.com/announcement"},
        "expected_revision":revision,"source_name":"官方公告","source_url":"https://example.com/announcement"})
}

async fn draft(h: &Harness, key: &str, input: Value) -> Value {
    let (s, d) = call(
        &h.app,
        "PUT",
        &format!("/api/admin/discover-workflow/drafts/news/{key}"),
        Some(&h.admin_t),
        input,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{d}");
    d
}

async fn review(
    h: &Harness,
    key: &str,
    revision: i64,
    after: Option<chrono::DateTime<Utc>>,
) -> (StatusCode, Value) {
    call(
        &h.app,
        "POST",
        &format!("/api/admin/discover-workflow/drafts/news/{key}/review"),
        Some(&h.admin_t),
        json!({"expected_revision":revision,"publish_after":after}),
    )
    .await
}

#[tokio::test]
async fn discover_workflow_isolates_live_content_and_reviews_exact_revision(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else {
        return Ok(());
    };
    let mut live = news("原版", 0);
    live.as_object_mut().unwrap().remove("expected_revision");
    let (s, v) = call(
        &h.app,
        "PUT",
        "/api/admin/discover/news/editorial",
        Some(&h.admin_t),
        live,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    let d = draft(&h, "editorial", news("待审版本", 0)).await;
    assert_eq!(d["status"], "draft");
    assert_eq!(d["data"]["sample"], false);
    let revision = d["revision"].as_i64().unwrap();
    let (_, daily) = call(&h.app, "GET", "/api/discover/daily", None, Value::Null).await;
    assert_eq!(
        daily["items"][0]["title"], "原版",
        "保存草稿不能改变公开内容"
    );
    let (s, detail) = call(
        &h.app,
        "GET",
        "/api/admin/discover-workflow/drafts/news/editorial",
        Some(&h.admin_t),
        Value::Null,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{detail}");
    assert_eq!(detail["live"]["data"]["title"], "原版");
    assert_eq!(detail["draft"]["source_name"], "官方公告");

    let (s, p) = call(&h.app, "PUT", "/api/admin/discover-workflow/preview/news", Some(&h.admin_t),
        json!({"data":{"title":"待审版本","source":"官方","url":"https://example.com/announcement","unknown":true}})).await;
    assert_eq!(s, StatusCode::OK, "{p}");
    assert!(p["data"].get("unknown").is_none());
    assert_eq!(p["data"]["sample"], false);
    let (s, _) = call(
        &h.app,
        "PUT",
        "/api/admin/discover-workflow/preview/news",
        Some(&h.admin_t),
        json!({"data":{"title":"x","source":"x","url":"javascript:alert(1)"}}),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let member = h.member().await?;
    let (s, _) = call(
        &h.app,
        "POST",
        "/api/admin/discover-workflow/drafts/news/editorial/review",
        Some(&member),
        json!({"expected_revision":revision}),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);

    let changed = draft(&h, "editorial", news("重新编辑", revision)).await;
    let new_revision = changed["revision"].as_i64().unwrap();
    assert_ne!(revision, new_revision);
    assert_eq!(
        review(&h, "editorial", revision, None).await.0,
        StatusCode::CONFLICT
    );
    let (s, v) = review(&h, "editorial", new_revision, None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["status"], "published");
    assert_eq!(
        review(&h, "editorial", new_revision, None).await.0,
        StatusCode::CONFLICT
    );
    let (_, daily) = call(&h.app, "GET", "/api/discover/daily", None, Value::Null).await;
    assert_eq!(daily["items"][0]["title"], "重新编辑");
    let (actions,): (i64,) = sqlx::query_as("SELECT count(*) FROM admin_audit WHERE action='discover.publish' AND target_id='news/editorial'").fetch_one(&h.pool).await?;
    assert_eq!(actions, 1);
    h.finish().await
}

#[tokio::test]
async fn discover_workflow_scopes_tokens_and_imports_atomically(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else {
        return Ok(());
    };
    let (s, created) = call(
        &h.app,
        "POST",
        "/api/admin/discover-workflow/tokens",
        Some(&h.admin_t),
        json!({"name":"日报采集","kinds":["news","news"],"days":30}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{created}");
    let token = created["token"].as_str().unwrap();
    let id = created["id"].as_str().unwrap();
    let (s, listed) = call(
        &h.app,
        "GET",
        "/api/admin/discover-workflow/tokens",
        Some(&h.admin_t),
        Value::Null,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{listed}");
    assert_eq!(listed["items"][0]["kinds"], json!(["news"]));
    assert!(!listed.to_string().contains(token));
    let mut a = news("一", 0);
    a["kind"] = json!("news");
    a["key"] = json!("import-a");
    let mut b = news("二", 0);
    b["kind"] = json!("news");
    b["key"] = json!("import-b");
    let batch = json!({"items":[a.clone(),b.clone()]});
    let (s, v) = call(
        &h.app,
        "POST",
        "/api/discover-editor/drafts/import",
        Some(token),
        batch.clone(),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["items"].as_array().unwrap().len(), 2);
    assert_eq!(v["items"][0]["status"], "draft");
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/discover-editor/drafts/import",
            Some(token),
            batch
        )
        .await
        .0,
        StatusCode::CONFLICT
    );

    let mut new = a.clone();
    new["key"] = json!("aaa-rollback");
    // 先写入新稿、随后遇到已有稿件的冲突，也必须回滚整批及审计。
    let (s, _) = call(
        &h.app,
        "POST",
        "/api/discover-editor/drafts/import",
        Some(token),
        json!({"items":[new.clone(),a.clone()]}),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
    let (n,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM discover_drafts WHERE key='aaa-rollback'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(n, 0);
    let (n,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM admin_audit WHERE target_id='news/aaa-rollback'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(n, 0);
    let (s, _) = call(
        &h.app,
        "POST",
        "/api/discover-editor/drafts/import",
        Some(token),
        json!({"items":[new.clone(),new]}),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let mut invalid = b.clone();
    invalid["data"]["url"] = json!("javascript:alert(1)");
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/discover-editor/drafts/import",
            Some(token),
            json!({"items":[a,invalid]})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &h.app,
            "PUT",
            "/api/discover-editor/drafts/hotword/nope",
            Some(token),
            json!({"expected_revision":0,"data":{"text":"x","tag":"热"}})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/discover-editor/drafts?kind=tool",
            Some(token),
            Value::Null
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/discover-workflow/drafts/news/import-a/review",
            Some(token),
            json!({"expected_revision":1})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(
            &h.app,
            "PUT",
            "/api/admin/discover/news/import-a",
            Some(token),
            news("禁止直接发布", 0)
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let mut sample = news("示例", 0);
    sample["sample"] = json!(true);
    assert_eq!(
        call(
            &h.app,
            "PUT",
            "/api/discover-editor/drafts/news/sample",
            Some(token),
            sample
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut smuggle = news("试图发布", 0);
    smuggle["status"] = json!("published");
    assert_eq!(
        call(
            &h.app,
            "PUT",
            "/api/discover-editor/drafts/news/smuggle",
            Some(token),
            smuggle
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/discover-editor/drafts",
            None,
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let audit_rows: Vec<(String,)> = sqlx::query_as("SELECT detail FROM admin_audit")
        .fetch_all(&h.pool)
        .await?;
    assert!(audit_rows.iter().all(|r| !r.0.contains(token)));

    let (s, _) = call(
        &h.app,
        "DELETE",
        &format!("/api/admin/discover-workflow/tokens/{id}"),
        Some(&h.admin_t),
        Value::Null,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/discover-editor/drafts",
            Some(token),
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    sqlx::query("UPDATE discover_tokens SET revoked_at=NULL,expires_at=now()-interval '1 minute'")
        .execute(&h.pool)
        .await?;
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/discover-editor/drafts",
            Some(token),
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (_, daily) = call(&h.app, "GET", "/api/discover/daily", None, Value::Null).await;
    assert_eq!(daily["total"], 5, "采集与导入不会发布");
    h.finish().await
}

#[tokio::test]
async fn discover_workflow_scheduling_survives_retries_and_requires_new_review_after_edits(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else {
        return Ok(());
    };
    let d = draft(&h, "scheduled", news("已终审", 0)).await;
    let revision = d["revision"].as_i64().unwrap();
    assert_eq!(
        review(
            &h,
            "scheduled",
            revision,
            Some(Utc::now() - Duration::minutes(1))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (s, v) = review(
        &h,
        "scheduled",
        revision,
        Some(Utc::now() + Duration::hours(1)),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(publish_due(&h.pool).await?, 0);
    sqlx::query(
        "UPDATE discover_drafts SET publish_after=now()-interval '1 second' WHERE key='scheduled'",
    )
    .execute(&h.pool)
    .await?;
    let (a, b) = tokio::join!(publish_due(&h.pool), publish_due(&h.pool));
    assert_eq!(a? + b?, 1, "多实例只能发布一次");
    assert_eq!(publish_due(&h.pool).await?, 0);
    let (status, reviewer): (String, String) =
        sqlx::query_as("SELECT status,reviewed_by FROM discover_drafts WHERE key='scheduled'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(status, "published");
    assert_eq!(reviewer, "root");
    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM admin_audit WHERE action='discover.publish' AND target_id='news/scheduled'").fetch_one(&h.pool).await?;
    assert_eq!(n, 1);

    let d = draft(&h, "changed", news("待修改", 0)).await;
    assert_eq!(
        review(
            &h,
            "changed",
            d["revision"].as_i64().unwrap(),
            Some(Utc::now() + Duration::hours(1))
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, detail) = call(
        &h.app,
        "GET",
        "/api/admin/discover-workflow/drafts/news/changed",
        Some(&h.admin_t),
        Value::Null,
    )
    .await;
    let edited = draft(
        &h,
        "changed",
        news("修改后", detail["draft"]["revision"].as_i64().unwrap()),
    )
    .await;
    assert_eq!(edited["status"], "draft");
    assert!(edited["publish_after"].is_null());
    assert!(edited["reviewed_by"].is_null());
    assert_eq!(publish_due(&h.pool).await?, 0);
    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_items WHERE key='changed'")
        .fetch_one(&h.pool)
        .await?;
    assert_eq!(n, 0);

    let d = draft(&h, "canceled", news("取消", 0)).await;
    assert_eq!(
        review(
            &h,
            "canceled",
            d["revision"].as_i64().unwrap(),
            Some(Utc::now() + Duration::hours(1))
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, detail) = call(
        &h.app,
        "GET",
        "/api/admin/discover-workflow/drafts/news/canceled",
        Some(&h.admin_t),
        Value::Null,
    )
    .await;
    let (s, v) = call(
        &h.app,
        "POST",
        "/api/admin/discover-workflow/drafts/news/canceled/cancel",
        Some(&h.admin_t),
        json!({"expected_revision":detail["draft"]["revision"]}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(publish_due(&h.pool).await?, 0);
    h.finish().await
}

#[tokio::test]
async fn discover_workflow_detects_live_conflicts_and_discarded_revision_cannot_review_recreated_draft(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else {
        return Ok(());
    };
    let d = draft(&h, "conflict", news("旧稿", 0)).await;
    let (s, _) = call(
        &h.app,
        "PUT",
        "/api/admin/discover/news/conflict",
        Some(&h.admin_t),
        json!({"data":news("线上新稿",0)["data"]}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        review(&h, "conflict", d["revision"].as_i64().unwrap(), None)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let d = draft(
        &h,
        "conflict",
        news("合并后的内容", d["revision"].as_i64().unwrap()),
    )
    .await;
    assert_eq!(
        review(
            &h,
            "conflict",
            d["revision"].as_i64().unwrap(),
            Some(Utc::now() + Duration::hours(1))
        )
        .await
        .0,
        StatusCode::OK
    );
    let (s, _) = call(
        &h.app,
        "PUT",
        "/api/admin/discover/news/conflict",
        Some(&h.admin_t),
        json!({"data":news("线上又改了",0)["data"]}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (_, detail) = call(
        &h.app,
        "GET",
        "/api/admin/discover-workflow/drafts/news/conflict",
        Some(&h.admin_t),
        Value::Null,
    )
    .await;
    assert_eq!(detail["draft"]["status"], "draft");
    assert!(!detail["draft"]["last_error"].as_str().unwrap().is_empty());
    assert_eq!(publish_due(&h.pool).await?, 0);

    // 运维绕过 HTTP 改数据，调度器仍按基线检测，冲突退回草稿并记审计。
    let d = draft(
        &h,
        "conflict",
        news("运维冲突", detail["draft"]["revision"].as_i64().unwrap()),
    )
    .await;
    assert_eq!(
        review(
            &h,
            "conflict",
            d["revision"].as_i64().unwrap(),
            Some(Utc::now() + Duration::hours(1))
        )
        .await
        .0,
        StatusCode::OK
    );
    sqlx::query("UPDATE discover_items SET updated_at=clock_timestamp() WHERE key='conflict'")
        .execute(&h.pool)
        .await?;
    sqlx::query(
        "UPDATE discover_drafts SET publish_after=now()-interval '1 second' WHERE key='conflict'",
    )
    .execute(&h.pool)
    .await?;
    assert_eq!(publish_due(&h.pool).await?, 0);
    let (status, error): (String, String) =
        sqlx::query_as("SELECT status,last_error FROM discover_drafts WHERE key='conflict'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(status, "draft");
    assert!(error.contains("线上内容"));
    let (n,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM admin_audit WHERE action='discover.publish.failed'")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(n, 1);

    let old = draft(&h, "recreated", news("丢弃前", 0)).await;
    let old_revision = old["revision"].as_i64().unwrap();
    let (s, _) = call(
        &h.app,
        "POST",
        "/api/admin/discover-workflow/drafts/news/recreated/discard",
        Some(&h.admin_t),
        json!({"expected_revision":old_revision}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let new = draft(&h, "recreated", news("重新创建", 0)).await;
    assert_ne!(new["revision"].as_i64().unwrap(), old_revision);
    assert_eq!(
        review(&h, "recreated", old_revision, None).await.0,
        StatusCode::CONFLICT
    );
    let update = news("并发修改", new["revision"].as_i64().unwrap());
    let (a, b) = tokio::join!(
        call(
            &h.app,
            "PUT",
            "/api/admin/discover-workflow/drafts/news/recreated",
            Some(&h.admin_t),
            update.clone()
        ),
        call(
            &h.app,
            "PUT",
            "/api/admin/discover-workflow/drafts/news/recreated",
            Some(&h.admin_t),
            update
        )
    );
    assert!(
        (a.0 == StatusCode::OK && b.0 == StatusCode::CONFLICT)
            || (b.0 == StatusCode::OK && a.0 == StatusCode::CONFLICT)
    );
    h.finish().await
}

#[tokio::test]
async fn discover_workflow_future_content_dates_stay_private_in_lists_counts_and_details(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else {
        return Ok(());
    };
    // 所有已发布条目在内容日期前均不可读，包含未登录详情与汇总计数。
    sqlx::query("UPDATE discover_items SET published_at=now()+interval '1 day'")
        .execute(&h.pool)
        .await?;
    let (_, home) = call(&h.app, "GET", "/api/discover/home", None, Value::Null).await;
    assert_eq!(home["banners"], json!([]));
    assert_eq!(home["news"]["total"], 0);
    assert!(home["repos"].is_null());
    let (_, daily) = call(&h.app, "GET", "/api/discover/daily", None, Value::Null).await;
    assert_eq!(daily["total"], 0);
    assert_eq!(daily["items"], json!([]));
    assert_eq!(
        call(&h.app, "GET", "/api/discover/repos", None, Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let (_, issues) = call(
        &h.app,
        "GET",
        "/api/discover/repos/issues",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(issues["items"], json!([]));
    let (_, models) = call(&h.app, "GET", "/api/discover/models", None, Value::Null).await;
    assert_eq!(models["models"], json!([]));
    let (tool,): (String,) =
        sqlx::query_as("SELECT key FROM discover_items WHERE kind='tool' LIMIT 1")
            .fetch_one(&h.pool)
            .await?;
    assert_eq!(
        call(
            &h.app,
            "GET",
            &format!("/api/tools/{tool}"),
            None,
            Value::Null
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (_, hot) = call(&h.app, "GET", "/api/discover/hot-search", None, Value::Null).await;
    assert_eq!(hot["items"], json!([]));
    h.finish().await
}

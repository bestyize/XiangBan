use super::*;
use crate::handlers::{open_source_import, open_source_rules as rules};

fn one_project() -> Value {
    let mut data = crate::handlers::open_source::catalog().clone();
    let mut project = data["projects"][0].clone();
    project["full_name"] = json!("Example/Fresh");
    project["name"] = json!("Fresh");
    project["cover_template"] = json!("auto");
    data["projects"] = json!([project]);
    data["collections"] = json!([]);
    data
}
#[tokio::test]
async fn curated_draft_publish_revision_and_scope_are_isolated(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else {
        return Ok(());
    };
    assert_eq!(
        call(&h.app, "GET", "/api/admin/open-source", None, Value::Null)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let member = h.member().await?;
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/admin/open-source",
            Some(&member),
            Value::Null
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (_, initial) = call(&h.app, "GET", "/api/open-source", None, Value::Null).await;
    let (original_count,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_items")
        .fetch_one(&h.pool)
        .await?;
    let (status, saved) = call(
        &h.app,
        "PUT",
        "/api/admin/open-source",
        Some(&h.admin_t),
        json!({"expected_revision":0,"data":one_project()}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["revision"], 1);
    assert_eq!(
        call(&h.app, "GET", "/api/open-source", None, Value::Null)
            .await
            .1,
        initial,
        "draft cannot change public data"
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/discover/repo/Example/Fresh/comments",
            None,
            Value::Null
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &h.app,
            "PUT",
            "/api/admin/open-source",
            Some(&h.admin_t),
            json!({"expected_revision":0,"data":one_project()})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/open-source/publish",
            Some(&h.admin_t),
            json!({"expected_revision":0})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let (status, published) = call(
        &h.app,
        "POST",
        "/api/admin/open-source/publish",
        Some(&h.admin_t),
        json!({"expected_revision":1}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{published}");
    assert_eq!(published["published_revision"], 1);
    let (_, live) = call(&h.app, "GET", "/api/open-source", None, Value::Null).await;
    assert_eq!(live["projects"][0]["full_name"], "Example/Fresh");
    assert_eq!(live["revision"], 1);
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/discover/repo/Example/Fresh/comments",
            None,
            Value::Null
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut empty = one_project();
    empty["projects"] = json!([]);
    let (status, saved) = call(
        &h.app,
        "PUT",
        "/api/admin/open-source",
        Some(&h.admin_t),
        json!({"expected_revision":1,"data":empty}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/open-source/publish",
            Some(&h.admin_t),
            json!({"expected_revision":2})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&h.app, "GET", "/api/open-source", None, Value::Null)
            .await
            .1["projects"]
            .as_array()
            .unwrap()
            .len(),
        0,
        "an intentionally empty publication must not fall back to seed"
    );
    assert_eq!(
        call(
            &h.app,
            "GET",
            "/api/discover/repo/Example/Fresh/comments",
            None,
            Value::Null
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_items")
        .fetch_one(&h.pool)
        .await?;
    assert_eq!(
        count, original_count,
        "other discovery boards are unchanged"
    );
    let mut incomplete = one_project();
    incomplete["projects"][0]["source_url"] = json!("");
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/open-source/preview",
            Some(&h.admin_t),
            incomplete
        )
        .await
        .0,
        StatusCode::BAD_REQUEST,
        "preview uses the same required fields as publishing"
    );
    let mut wrong = one_project();
    wrong["scope"] = json!("news");
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/open-source/preview",
            Some(&h.admin_t),
            wrong
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    h.finish().await?;
    Ok(())
}
async fn asset_status(
    app: &axum::Router,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, Vec<u8>) {
    let mut request = Request::builder().uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    (status, bytes.to_vec())
}
#[tokio::test]
async fn curated_assets_are_private_until_referenced_by_the_exact_published_project(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(h) = harness().await? else {
        return Ok(());
    };
    let (status,asset)=call(&h.app,"POST","/api/admin/open-source/assets",Some(&h.admin_t),json!({"project":"Example/Fresh","source_url":"https://example.com/logo.png","data_base64":"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jvZkAAAAASUVORK5CYII="})).await;
    assert_eq!(status, StatusCode::OK, "{asset}");
    let url = asset["url"].as_str().unwrap();
    assert_eq!(
        asset_status(&h.app, url, None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        asset_status(
            &h.app,
            &format!(
                "/api/admin/open-source/assets/{}",
                asset["id"].as_str().unwrap()
            ),
            Some(&h.admin_t)
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut data = one_project();
    data["projects"][0]["icon_url"] = json!(url);
    assert_eq!(
        call(
            &h.app,
            "PUT",
            "/api/admin/open-source",
            Some(&h.admin_t),
            json!({"expected_revision":0,"data":data})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        asset_status(&h.app, url, None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/open-source/publish",
            Some(&h.admin_t),
            json!({"expected_revision":1})
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status, bytes) = asset_status(&h.app, url, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(bytes.starts_with(b"\x89PNG"));
    let mut wrong = one_project();
    wrong["projects"][0]["full_name"] = json!("Other/Project");
    wrong["projects"][0]["icon_url"] = json!(url);
    assert_eq!(
        call(
            &h.app,
            "PUT",
            "/api/admin/open-source",
            Some(&h.admin_t),
            json!({"expected_revision":1,"data":wrong})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/open-source/publish",
            Some(&h.admin_t),
            json!({"expected_revision":2})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&h.app, "GET", "/api/open-source", None, Value::Null)
            .await
            .1["revision"],
        1
    );
    assert_eq!(
        call(
            &h.app,
            "POST",
            "/api/admin/open-source/assets/fetch",
            Some(&h.admin_t),
            json!({"project":"Example/Fresh","url":"https://127.0.0.1/metadata"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    h.finish().await?;
    Ok(())
}
#[test]
fn github_import_only_accepts_repository_identities_and_extracts_evidenced_images() {
    assert_eq!(
        open_source_import::repository("https://github.com/Owner/Repo.git#readme").unwrap(),
        "Owner/Repo"
    );
    for invalid in [
        "https://evil.test/o/r",
        "https://user:pass@github.com/o/r",
        "https://github.com/o/r/issues",
        "o/r/other",
        "../private",
    ] {
        assert!(
            open_source_import::repository(invalid).is_err(),
            "{invalid}"
        );
    }
    let found=open_source_import::candidates("![badge](https://img.shields.io/badge/ok)\n![Project logo](./logo.png)\n<img src=\"./shot.png\" alt=\"Screenshot\">\n![bad](javascript:alert)\n", "https://raw.githubusercontent.com/o/r/main/README.md");
    assert_eq!(found.len(), 2);
    assert_eq!(found[0]["role"], "logo");
    assert_eq!(
        found[0]["url"],
        "https://raw.githubusercontent.com/o/r/main/logo.png"
    );
    assert_eq!(found[1]["role"], "screenshot");
}
#[test]
fn curated_rules_reject_other_boards_and_unsafe_media_without_affecting_seed() {
    let mut wrong = one_project();
    wrong["scope"] = json!("frontier");
    assert!(rules::normalize(wrong, false).is_err());
    let mut wrong = one_project();
    wrong["projects"][0]["cover_template"] = json!("news-banner");
    assert!(rules::normalize(wrong, false).is_err());
    for url in [
        "javascript:alert(1)",
        "data:image/svg+xml,evil",
        "//other.test/x",
        "/open-source-media/../secret",
    ] {
        let mut wrong = one_project();
        wrong["projects"][0]["cover_url"] = json!(url);
        assert!(rules::normalize(wrong, false).is_err());
    }
    let mut repeated = one_project();
    repeated["collections"] =
        json!([{"key":"same-notes","title":"专题","projects":["Example/Fresh"],"notes":[""]}]);
    assert_eq!(
        rules::normalize(repeated, false).unwrap()["collections"][0]["notes"],
        json!([""])
    );
}

#[test]
fn curated_cover_protocol_matches_browser_cases() {
    let cases: Value =
        serde_json::from_str(include_str!("../../editorial/open-source-cover-cases.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let mut project = case["input"].clone();
        rules::resolve(&mut project).unwrap();
        for (key, value) in case["expected"].as_object().unwrap() {
            assert_eq!(&project[key], value, "{key}: {project}");
        }
    }
    assert_eq!(rules::hash("Example/Tool"), rules::hash("EXAMPLE/TOOL"));
}

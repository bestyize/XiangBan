use super::*;
use axum::{
    body::Bytes,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
const SECRET: &[u8] = b"0123456789abcdef0123456789abcdef";
fn claims() -> Claims {
    Claims {
        nonce: uuid::Uuid::new_v4().to_string(),
        repository: "owner/tool".into(),
        version: "v1".into(),
        filename: "tool-windows-x64.exe".into(),
        repo_id: 1,
        asset_id: 2,
        source_url: "https://github.com/owner/tool/releases/download/v1/tool-windows-x64.exe"
            .into(),
        size: 4,
        sha256: "a".repeat(64),
        exp: Utc::now().timestamp() + 240,
    }
}
fn receipt(c: &Claims) -> Receipt {
    Receipt {
        nonce: c.nonce.clone(),
        repo_id: c.repo_id,
        asset_id: c.asset_id,
        filename: c.filename.clone(),
        sha256: c.sha256.clone(),
        size: c.size,
        file_key: format!("tools/{}.bin", c.sha256),
        readback_verified: true,
        verified_at: Utc::now().timestamp(),
    }
}
#[test]
fn receipts_bind_nonce_asset_hash_size_path_and_expiry() {
    let c = claims();
    let good = receipt(&c);
    assert!(receipt_valid(&good, &c, Utc::now().timestamp()));
    for field in [
        "nonce", "hash", "size", "asset", "path", "verified", "stale",
    ] {
        let mut r = receipt(&c);
        match field {
            "nonce" => r.nonce = uuid::Uuid::new_v4().to_string(),
            "hash" => r.sha256 = "b".repeat(64),
            "size" => r.size = 3,
            "asset" => r.asset_id = 3,
            "path" => r.file_key = "packs/evil".into(),
            "verified" => r.readback_verified = false,
            _ => r.verified_at -= 1000,
        };
        assert!(!receipt_valid(&r, &c, Utc::now().timestamp()), "{field}")
    }
}
#[test]
fn hmac_domains_and_resource_base_are_not_interchangeable() {
    let c = claims();
    let raw = serde_json::to_vec(&c).unwrap();
    let sig = signature(REQUEST_DOMAIN, &raw, SECRET);
    assert!(verify(REQUEST_DOMAIN, &raw, &sig, SECRET));
    assert!(!verify(RECEIPT_DOMAIN, &raw, &sig, SECRET));
    assert!(!verify(REQUEST_DOMAIN, b"changed", &sig, SECRET));
    assert!(!verify(REQUEST_DOMAIN, &raw, &"中".repeat(32), SECRET));
    for raw in [
        "https://user:pass@res.example",
        "https://res.example?token=secret",
        "https://res.example/#private",
        "http://169.254.169.254/",
    ] {
        assert!(base_url(raw).is_err())
    }
    assert_eq!(
        endpoint("https://res.example", "internal/tool-mirrors")
            .unwrap()
            .as_str(),
        "https://res.example/internal/tool-mirrors"
    )
}
async fn server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, task)
}
#[tokio::test]
async fn only_signed_verified_resource_receipt_can_return_success() {
    async fn mirror(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
        let sig = headers.get("x-tool-signature").unwrap().to_str().unwrap();
        assert!(verify(REQUEST_DOMAIN, &body, sig, SECRET));
        assert!(headers.get("authorization").is_none());
        let c: Claims = serde_json::from_slice(&body).unwrap();
        let raw = serde_json::to_vec(&receipt(&c)).unwrap();
        let sig = signature(RECEIPT_DOMAIN, &raw, SECRET);
        (
            [
                (
                    axum::http::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                ),
                (
                    axum::http::HeaderName::from_static("x-tool-receipt-signature"),
                    HeaderValue::from_str(&sig).unwrap(),
                ),
            ],
            raw,
        )
    }
    let (base, task) = server(Router::new().route("/internal/tool-mirrors", post(mirror))).await;
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let c = claims();
    assert!(request_mirror(&http, &base, &c, SECRET).await.is_ok());
    assert!(request_mirror(&http, &base, &c, b"wrong key")
        .await
        .is_err());
    task.abort();
}
#[tokio::test]
async fn unsigned_or_replayed_receipt_does_not_register() {
    for signed in [false, true] {
        let c = claims();
        let mut wrong = receipt(&c);
        wrong.nonce = uuid::Uuid::new_v4().to_string();
        let raw = serde_json::to_vec(&wrong).unwrap();
        let response = raw.clone();
        let sig = if signed {
            signature(RECEIPT_DOMAIN, &raw, SECRET)
        } else {
            "0".repeat(64)
        };
        let app = Router::new().route(
            "/internal/tool-mirrors",
            post(move || {
                let raw = response.clone();
                let sig = sig.clone();
                async move {
                    (
                        [(
                            axum::http::HeaderName::from_static("x-tool-receipt-signature"),
                            sig,
                        )],
                        raw,
                    )
                }
            }),
        );
        let (base, task) = server(app).await;
        let http = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        assert!(request_mirror(&http, &base, &c, SECRET).await.is_err());
        task.abort();
    }
}

#[test]
fn protocol_vector_and_encoded_download_filename_match_resource() {
    assert_eq!(
        signature(REQUEST_DOMAIN, br#"{"nonce":"fixed"}"#, SECRET),
        "10ee1b286d23e43aa69bdaaf37faa6b416f4891b2375c7fdfcd66c5b12c47860"
    );
    let hash = "a".repeat(64);
    let name = "tool windows #x64.exe";
    let url =
        resource_download_url("https://res.example", &hash, name, 1900000000, SECRET).unwrap();
    assert!(url.as_str().contains("%23"));
    assert!(url.fragment().is_none());
    let sig = url
        .query_pairs()
        .find(|(k, _)| k == "sig")
        .unwrap()
        .1
        .to_string();
    assert!(crate::content::verify_download(
        &format!("tool-mirrors/{hash}/{name}"),
        1900000000,
        &sig,
        1,
        SECRET
    ));
}
fn snapshot() -> Value {
    json!({"source":"GitHub Releases","as_of":"2026-10-31T04:00:00Z","github_repo_id":1,"github_full_name":"owner/tool","release_id":3,"release_url":"https://github.com/owner/tool/releases/tag/v1","source_published_at":"2026-10-30T00:00:00Z","name":"tool","developer":"owner","type_label":"桌面 / 命令行工具","license":"unknown","license_spdx":null,"license_source_url":"https://github.com/owner/tool","platforms":["Windows"],"description":"test","homepage_url":"https://github.com/owner/tool","icon_url":"","discussion_tag":"","releases":[{"version":"v1","date":"2026-10-30","url":"https://github.com/owner/tool/releases/tag/v1","items":[{"kind":"new","text":"Official release"}]}],"artifacts":[{"os":"windows","arch":"x86_64","desc":"tool-windows-x64.exe","filename":"tool-windows-x64.exe","size":4,"sha256":"a".repeat(64),"hash_source":"github_digest","asset_id":2,"source_updated_at":"2026-10-30T00:00:00Z","mirror_url":"","github_url":"https://github.com/owner/tool/releases/download/v1/tool-windows-x64.exe"}],"collection_notes":[]})
}
#[tokio::test]
async fn staged_mirrors_stay_private_until_same_asset_is_published_and_revision_conflicts_do_not_register(
) {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::sync::Arc;
    use tower::ServiceExt;
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("SKIPPED database mirror test: TEST_DATABASE_URL not set");
        return;
    };
    let root = crate::db::connect(&url, None).await.unwrap();
    let schema = format!("tool_mirror_test_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&root)
        .await
        .unwrap();
    let pool = crate::db::connect(&url, Some(&schema)).await.unwrap();
    sqlx::raw_sql("CREATE TABLE discover_drafts(kind TEXT,key TEXT,revision BIGINT,status TEXT DEFAULT 'draft',data JSONB);CREATE TABLE discover_items(kind TEXT,key TEXT,data JSONB);").execute(&pool).await.unwrap();
    init(&pool).await.unwrap();
    let work_pool = pool.clone();
    let task = tokio::spawn(async move {
        let data = snapshot();
        tool_sources::validate_tool(data.clone()).unwrap();
        sqlx::query("INSERT INTO discover_drafts(kind,key,revision,data)VALUES('tool','github-tool-1',1,$1)").bind(&data).execute(&work_pool).await.unwrap();
        let c = candidate(data.clone(), 1, 2).unwrap();
        let r = receipt(&c.claims);
        register(&work_pool, "github-tool-1", &c, &r, None)
            .await
            .unwrap();
        let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_items")
            .fetch_one(&work_pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let mut unpublished = data.clone();
        decorate_tool(
            &work_pool,
            "https://chat.example",
            "github-tool-1",
            &mut unpublished,
        )
        .await
        .unwrap();
        assert_eq!(unpublished["artifacts"][0]["mirror_url"], "");
        let mut config = crate::config::AppConfig::from_env();
        config.resource_base_url = None;
        let state = AppState {
            pool: work_pool.clone(),
            config: Arc::new(config),
            verifier: Arc::new(crate::auth::JwksVerifier::new(
                "unused".into(),
                "unused".into(),
                "unused".into(),
            )),
            http: Client::new(),
        };
        let app = public_routes().with_state(state.clone());
        let path = format!("/discover/tool-mirrors/github-tool-1/2/{}", "a".repeat(64));
        let get = || Request::builder().uri(&path).body(Body::empty()).unwrap();
        assert_eq!(
            app.clone().oneshot(get()).await.unwrap().status(),
            StatusCode::NOT_FOUND
        );
        sqlx::query("INSERT INTO discover_items(kind,key,data)VALUES('tool','github-tool-1',$1)")
            .bind(&data)
            .execute(&work_pool)
            .await
            .unwrap();
        let mut published = data.clone();
        decorate_tool(
            &work_pool,
            "https://chat.example",
            "github-tool-1",
            &mut published,
        )
        .await
        .unwrap();
        assert!(published["artifacts"][0]["mirror_url"]
            .as_str()
            .unwrap()
            .contains("/api/discover/tool-mirrors/github-tool-1/2/"));
        let response = app.clone().oneshot(get()).await.unwrap();
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response.headers().get("location").unwrap(),
            c.claims.source_url.as_str()
        );
        let (base, resource_task) = server(Router::new().route(
            "/tool-mirrors/{sha}/{filename}",
            axum::routing::get(|| async { StatusCode::OK }),
        ))
        .await;
        let mut online = (*state.config).clone();
        online.resource_base_url = Some(base.clone());
        online.resource_secret = SECRET.to_vec();
        let online_app = public_routes().with_state(AppState {
            config: Arc::new(online),
            ..state.clone()
        });
        let response = online_app.clone().oneshot(get()).await.unwrap();
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        let target = Url::parse(
            response
                .headers()
                .get("location")
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
        assert!(target.as_str().starts_with(&base));
        let query = target
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();
        let exp = query["exp"].parse().unwrap();
        assert!(crate::content::verify_download(
            &format!("tool-mirrors/{}/{}", c.claims.sha256, c.claims.filename),
            exp,
            &query["sig"],
            Utc::now().timestamp(),
            SECRET
        ));
        resource_task.abort();
        let response = online_app.oneshot(get()).await.unwrap();
        assert_eq!(
            response.headers().get("location").unwrap(),
            c.claims.source_url.as_str()
        );
        sqlx::query("UPDATE discover_drafts SET revision=2 WHERE key='github-tool-1'")
            .execute(&work_pool)
            .await
            .unwrap();
        assert!(matches!(
            register(&work_pool, "github-tool-1", &c, &r, None).await,
            Err(AppError::Conflict(_))
        ));
        let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM tool_mirror_registry")
            .fetch_one(&work_pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
        sqlx::query("DELETE FROM discover_items")
            .execute(&work_pool)
            .await
            .unwrap();
        assert_eq!(
            app.oneshot(get()).await.unwrap().status(),
            StatusCode::NOT_FOUND
        );
        let admin_app = admin_routes().with_state(state);
        assert_eq!(
            admin_app
                .oneshot(
                    Request::builder()
                        .uri("/tool-mirrors")
                        .body(Body::empty())
                        .unwrap()
                )
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    });
    let result = task.await;
    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&root)
        .await
        .unwrap();
    root.close().await;
    result.unwrap();
}

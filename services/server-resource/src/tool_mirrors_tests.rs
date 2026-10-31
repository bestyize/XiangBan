use super::*;
use axum::{body::to_bytes, routing::get};
use std::sync::Arc;
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
        sha256: crate::hex(digest::digest(&digest::SHA256, b"demo").as_ref()),
        exp: crate::now() + 240,
    }
}
fn state(dir: &FsPath) -> AppState {
    AppState {
        data_dir: Arc::new(dir.to_path_buf()),
        secret: Arc::new(SECRET.to_vec()),
        callback_url: Arc::new(String::new()),
        http: reqwest::Client::new(),
        uploads: Arc::new(tokio::sync::Semaphore::new(1)),
    }
}
fn local(url: &reqwest::Url) -> bool {
    url.scheme() == "http"
        && url.host_str() == Some("127.0.0.1")
        && url.username().is_empty()
        && url.password().is_none()
}
fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .no_gzip()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}
async fn server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    (base, task)
}
#[test]
fn signature_expiry_source_and_path_are_bound() {
    let c = claims();
    assert!(claims_valid(&c, crate::now()));
    let raw = serde_json::to_vec(&c).unwrap();
    let sig = signature(REQUEST_DOMAIN, &raw, SECRET);
    assert!(verify(REQUEST_DOMAIN, &raw, &sig, SECRET));
    assert!(!verify(RECEIPT_DOMAIN, &raw, &sig, SECRET));
    for bad in [
        "http://127.0.0.1/a",
        "https://github.com/evil/repo/releases/download/v1/a",
        "https://github.com@evil.test/a",
    ] {
        let mut changed = c.clone();
        changed.source_url = bad.into();
        assert!(!claims_valid(&changed, crate::now()));
    }
    let mut changed = c.clone();
    changed.exp = crate::now() + 301;
    assert!(!claims_valid(&changed, crate::now()));
    changed = c;
    changed.filename = "../x.exe".into();
    assert!(!claims_valid(&changed, crate::now()));
    assert!(!verify(REQUEST_DOMAIN, b"changed", &sig, SECRET));
}
#[tokio::test]
async fn stream_write_disk_readback_signed_range_and_idempotent_reuse() {
    let dir = std::env::temp_dir().join(format!("tool-mirror-{}", uuid::Uuid::new_v4()));
    let (base, task) =
        server(Router::new().route("/asset", get(|| async { b"demo".to_vec() }))).await;
    let mut c = claims();
    c.source_url = format!("{base}/asset");
    let receipt = store(&dir, &http(), &c, local, MAX_STORAGE, MAX_OBJECTS)
        .await
        .unwrap();
    assert!(receipt.readback_verified);
    assert_eq!(receipt.sha256, c.sha256);
    assert_eq!(
        fs::read(dir.join(&receipt.file_key)).await.unwrap(),
        b"demo"
    );
    let reused = store(&dir, &http(), &c, local, 4, 1).await.unwrap();
    assert_eq!(reused.file_key, receipt.file_key);
    let key = format!("tool-mirrors/{}/{}", c.sha256, c.filename);
    let exp = crate::now() + 60;
    let sig = crate::sign::sign_callback(format!("GET\n{key}\n{exp}").as_bytes(), SECRET);
    let app = routes().with_state(state(&dir));
    let r = app
        .oneshot(
            Request::builder()
                .uri(format!("/{key}?exp={exp}&sig={sig}"))
                .header("range", "bytes=1-2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(to_bytes(r.into_body(), 10).await.unwrap().as_ref(), b"em");
    task.abort();
    fs::remove_dir_all(dir).await.unwrap();
}
#[tokio::test]
async fn wrong_hash_quota_and_redirect_escape_never_leave_registered_files() {
    let dir = std::env::temp_dir().join(format!("tool-mirror-{}", uuid::Uuid::new_v4()));
    let app = Router::new()
        .route("/asset", get(|| async { b"demo".to_vec() }))
        .route(
            "/escape",
            get(|| async {
                (
                    StatusCode::FOUND,
                    [("location", "http://169.254.169.254/private")],
                )
            }),
        );
    let (base, task) = server(app).await;
    let mut c = claims();
    c.source_url = format!("{base}/asset");
    c.sha256 = "0".repeat(64);
    assert!(store(&dir, &http(), &c, local, MAX_STORAGE, MAX_OBJECTS)
        .await
        .is_err());
    assert!(!fs::try_exists(dir.join(format!("tools/{}.bin", c.sha256)))
        .await
        .unwrap());
    c = claims();
    c.source_url = format!("{base}/asset");
    assert!(store(&dir, &http(), &c, local, 3, 128).await.is_err());
    c.source_url = format!("{base}/escape");
    assert!(store(&dir, &http(), &c, local, MAX_STORAGE, MAX_OBJECTS)
        .await
        .is_err());
    task.abort();
    fs::remove_dir_all(dir).await.unwrap();
}
#[tokio::test]
async fn quota_lock_rejects_concurrent_writers_and_tampering_is_not_acknowledged() {
    let dir = std::env::temp_dir().join(format!("tool-mirror-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(dir.join("tools")).await.unwrap();
    let lock = std::fs::File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join("tools/.quota.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let c = claims();
    assert_eq!(
        store(&dir, &http(), &c, allowed, MAX_STORAGE, MAX_OBJECTS)
            .await
            .unwrap_err()
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    drop(lock);
    fs::write(dir.join(format!("tools/{}.bin", c.sha256)), b"evil")
        .await
        .unwrap();
    assert!(store(&dir, &http(), &c, allowed, MAX_STORAGE, MAX_OBJECTS)
        .await
        .is_err());
    assert!(!fs::try_exists(dir.join(format!("tools/{}.bin", c.sha256)))
        .await
        .unwrap());
    fs::remove_dir_all(dir).await.unwrap();
}
#[tokio::test]
async fn forged_request_never_starts_upstream_download() {
    let dir = std::env::temp_dir().join(format!("tool-mirror-{}", uuid::Uuid::new_v4()));
    let app = routes().with_state(state(&dir));
    let r = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/tool-mirrors")
                .header("content-type", "application/json")
                .header("x-tool-signature", "bad")
                .body(Body::from(serde_json::to_vec(&claims()).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::FORBIDDEN);
    assert!(!fs::try_exists(dir).await.unwrap());
}

#[test]
fn protocol_signature_vector_matches_forum() {
    assert_eq!(
        signature(REQUEST_DOMAIN, br#"{"nonce":"fixed"}"#, SECRET),
        "10ee1b286d23e43aa69bdaaf37faa6b416f4891b2375c7fdfcd66c5b12c47860"
    );
}

#[tokio::test]
async fn signed_cached_request_returns_nonce_bound_readback_receipt_without_network() {
    let dir = std::env::temp_dir().join(format!("tool-mirror-{}", uuid::Uuid::new_v4()));
    let c = claims();
    fs::create_dir_all(dir.join("tools")).await.unwrap();
    fs::write(dir.join(format!("tools/{}.bin", c.sha256)), b"demo")
        .await
        .unwrap();
    let raw = serde_json::to_vec(&c).unwrap();
    let sig = signature(REQUEST_DOMAIN, &raw, SECRET);
    let app = routes().with_state(state(&dir));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/tool-mirrors")
                .header("content-type", "application/json")
                .header("x-tool-signature", sig)
                .body(Body::from(raw))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let sig = response
        .headers()
        .get("x-tool-receipt-signature")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = to_bytes(response.into_body(), 8192).await.unwrap();
    assert!(verify(RECEIPT_DOMAIN, &body, &sig, SECRET));
    let receipt: Receipt = serde_json::from_slice(&body).unwrap();
    assert_eq!(receipt.nonce, c.nonce);
    assert_eq!(receipt.asset_id, c.asset_id);
    assert_eq!(receipt.sha256, c.sha256);
    assert_eq!(receipt.size, c.size);
    assert!(receipt.readback_verified);
    assert!(!verify(
        REQUEST_DOMAIN,
        &body,
        &format!("{}aaaa", "中".repeat(20)),
        SECRET
    ));
    fs::remove_dir_all(dir).await.unwrap();
}

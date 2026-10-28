use super::*;
use axum::body::to_bytes;
use std::io::Write;
use std::sync::Mutex;

const SECRET: &[u8] = b"0123456789abcdef0123456789abcdef";

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(&mut buf);
    for (name, data) in entries {
        writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap();
    buf.into_inner()
}

fn sha(data: &[u8]) -> String {
    hex(digest::digest(&digest::SHA256, data).as_ref())
}

/// 起一个假的论坛：记录回调，并按设定返回状态码。
async fn fake_forum(status: StatusCode) -> (String, Arc<Mutex<Vec<(String, Vec<u8>)>>>) {
    let log: Arc<Mutex<Vec<(String, Vec<u8>)>>> = Arc::default();
    let recorded = log.clone();
    let app = Router::new().route(
        "/cb",
        axum::routing::post(move |headers: HeaderMap, body: axum::body::Bytes| {
            let recorded = recorded.clone();
            async move {
                let sig = headers.get("x-signature").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
                recorded.lock().unwrap().push((sig, body.to_vec()));
                (status, Json(json!({ "message": "拒绝" })))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}/cb"), log)
}

fn state(dir: &FsPath, callback: String) -> AppState {
    AppState {
        data_dir: Arc::new(dir.to_path_buf()),
        secret: Arc::new(SECRET.to_vec()),
        callback_url: Arc::new(callback),
        http: reqwest::Client::new(),
        uploads: Arc::new(Semaphore::new(2)),
    }
}

fn ticket(kind: &str, data: &[u8], key: &str) -> String {
    sign::issue_upload_ticket(
        &sign::UploadClaims { uid: "u".into(), kind: kind.into(), size: data.len() as u64, sha256: sha(data), file_key: key.into(), exp: now() + 600 },
        SECRET,
    )
}

async fn put(app: &Router, ticket: &str, data: Vec<u8>) -> StatusCode {
    let req = Request::builder().method("PUT").uri("/upload").header("x-upload-ticket", ticket).body(Body::from(data)).unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

fn signed_url(key: &str, exp: i64) -> String {
    let sig = sign::sign_callback(format!("GET\n{key}\n{exp}").as_bytes(), SECRET);
    format!("/{key}?exp={exp}&sig={sig}")
}

#[tokio::test]
async fn upload_verify_callback_and_signed_download() {
    let dir = std::env::temp_dir().join(format!("res-{}", uuid::Uuid::new_v4()));
    let (cb, log) = fake_forum(StatusCode::OK).await;
    let app = build_router(state(&dir, cb), &[]);
    let pack = zip_bytes(&[("manifest.json", br#"{"schemaVersion":1,"kind":"effect","id":"a","name":"A","version":"1"}"#), ("x.txt", b"hello")]);
    let key = "packs/effect/aaa.zip";

    // 坏凭证、不匹配的大小与哈希都拒绝，且不留下文件。
    assert_eq!(put(&app, "garbage", pack.clone()).await, StatusCode::FORBIDDEN);
    assert_eq!(put(&app, &ticket("effect", &pack, key), pack[..pack.len() - 1].to_vec()).await, StatusCode::BAD_REQUEST);
    let mut tampered = pack.clone();
    *tampered.last_mut().unwrap() ^= 1;
    assert_eq!(put(&app, &ticket("effect", &pack, key), tampered).await, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(!dir.join(key).exists());
    assert!(log.lock().unwrap().is_empty(), "校验失败不应通知论坛");

    // 内容不是合法内容包。
    let junk = b"not a zip".to_vec();
    assert_eq!(put(&app, &ticket("effect", &junk, "packs/effect/junk.zip"), junk.clone()).await, StatusCode::UNPROCESSABLE_ENTITY);

    // 正常上传：落盘并回调，签名可验证，回调带清单。
    assert_eq!(put(&app, &ticket("effect", &pack, key), pack.clone()).await, StatusCode::OK);
    assert!(dir.join(key).exists());
    {
        let calls = log.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let (sig, body) = &calls[0];
        assert_eq!(*sig, sign::sign_callback(body, SECRET));
        let v: Value = serde_json::from_slice(body).unwrap();
        assert_eq!(v["file_key"], key);
        assert_eq!(v["sha256"], sha(&pack));
        assert_eq!(v["manifest"]["id"], "a");
    }
    // 同一个键不能覆盖。
    assert_eq!(put(&app, &ticket("effect", &pack, key), pack.clone()).await, StatusCode::CONFLICT);

    // 下载：无签名 / 过期 / 错键拒绝，有效则返回内容并支持 Range。
    let get = |uri: String, range: Option<&str>| {
        let app = app.clone();
        let range = range.map(str::to_string);
        async move {
            let mut req = Request::builder().uri(uri);
            if let Some(r) = range {
                req = req.header(header::RANGE, r);
            }
            app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap()
        }
    };
    assert_eq!(get(format!("/{key}"), None).await.status(), StatusCode::FORBIDDEN);
    assert_eq!(get(signed_url(key, now() - 10), None).await.status(), StatusCode::FORBIDDEN);
    let other_sig = signed_url("packs/effect/other.zip", now() + 60);
    let borrowed = format!("/{key}?{}", other_sig.split_once('?').unwrap().1);
    assert_eq!(get(borrowed, None).await.status(), StatusCode::FORBIDDEN, "别的文件的签名不能拿来下这个");
    assert_eq!(get(other_sig, None).await.status(), StatusCode::NOT_FOUND);
    let ok = get(signed_url(key, now() + 60), None).await;
    assert_eq!(ok.status(), StatusCode::OK);
    assert_eq!(ok.headers()[header::CONTENT_TYPE], "application/zip");
    assert_eq!(to_bytes(ok.into_body(), 1 << 20).await.unwrap().to_vec(), pack);
    let part = get(signed_url(key, now() + 60), Some("bytes=0-3")).await;
    assert_eq!(part.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(to_bytes(part.into_body(), 1 << 20).await.unwrap().to_vec(), pack[..4].to_vec());
    assert_eq!(get("/tmp/anything".into(), None).await.status(), StatusCode::NOT_FOUND);
    assert_eq!(get("/packs/..%2F..%2Fetc%2Fpasswd".into(), None).await.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn covers_are_checked_and_public() {
    let dir = std::env::temp_dir().join(format!("res-{}", uuid::Uuid::new_v4()));
    let (cb, log) = fake_forum(StatusCode::OK).await;
    let app = build_router(state(&dir, cb), &[]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(&[0; 64]);
    let svg = b"<svg xmlns='http://www.w3.org/2000/svg'><script>alert(1)</script></svg>".to_vec();

    assert_eq!(put(&app, &ticket("cover", &svg, "covers/bad"), svg.clone()).await, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(put(&app, &ticket("cover", &png, "covers/good"), png.clone()).await, StatusCode::OK);
    let body: Value = serde_json::from_slice(&log.lock().unwrap()[0].1).unwrap();
    assert!(body["manifest"].is_null());

    let res = app.clone().oneshot(Request::builder().uri("/covers/good").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "封面公开，不需要签名");
    assert_eq!(res.headers()[header::CONTENT_TYPE], "image/png");
    assert_eq!(res.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    let missing = app.clone().oneshot(Request::builder().uri("/covers/bad").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn forum_rejection_removes_the_file() {
    let dir = std::env::temp_dir().join(format!("res-{}", uuid::Uuid::new_v4()));
    let (cb, _) = fake_forum(StatusCode::CONFLICT).await;
    let app = build_router(state(&dir, cb), &[]);
    let pack = zip_bytes(&[("manifest.json", b"{}")]);
    assert_eq!(put(&app, &ticket("effect", &pack, "packs/effect/z.zip"), pack.clone()).await, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(!dir.join("packs/effect/z.zip").exists(), "论坛不接受时撤销文件");

    let (cb, _) = fake_forum(StatusCode::INTERNAL_SERVER_ERROR).await;
    let app = build_router(state(&dir, cb), &[]);
    assert_eq!(put(&app, &ticket("effect", &pack, "packs/effect/y.zip"), pack.clone()).await, StatusCode::BAD_GATEWAY);
    assert!(!dir.join("packs/effect/y.zip").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

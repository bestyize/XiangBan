use super::*;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
    Router,
};
use std::sync::{Arc, Mutex};

fn hash(bytes: &[u8]) -> String {
    digest::digest(&digest::SHA256, bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn repo() -> Value {
    json!({"id":12345,"private":false,"full_name":"owner/Tool","name":"Tool","owner":{"login":"owner"},"description":"Official project description","license":{"spdx_id":"MIT"}})
}
fn asset(name: &str) -> Value {
    json!({"id":42,"name":name,"state":"uploaded","size":4,"digest":format!("sha256:{}",hash(b"demo")),"updated_at":"2026-10-30T12:00:00Z","browser_download_url":format!("https://github.com/owner/Tool/releases/download/v1.0/{name}")})
}
fn release() -> Value {
    json!({"id":678,"tag_name":"v1.0","name":"Release 1.0","body":"IGNORED_FULL_BODY","draft":false,"prerelease":false,"html_url":"https://github.com/owner/Tool/releases/tag/v1.0","published_at":"2026-10-30T12:00:00Z","assets":[asset("tool-windows-x64.exe")]})
}
fn client() -> Client {
    Client::builder()
        .no_gzip()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}
fn budget() -> Budget {
    Budget {
        remaining: TOTAL_DOWNLOAD,
        deadline: Instant::now() + Duration::from_secs(10),
    }
}
#[test]
fn config_has_ten_repo_limit_and_no_url_or_duplicate_escape_hatches() {
    assert_eq!(
        sanitize_config(json!({"repositories":[" Owner/Tool "]})).unwrap(),
        json!({"repositories":["owner/tool"]})
    );
    for bad in [
        json!({"repositories":["owner/tool","OWNER/TOOL"]}),
        json!({"repositories":["https://github.com/owner/tool"]}),
        json!({"repositories":["owner/.."]}),
        json!({"repositories":["-owner/tool"]}),
        json!({"repositories":["owner/tool?x"]}),
        json!({"repositories":[],"api":"http://localhost"}),
        json!({"repositories":(0..11).map(|i|format!("o/r{i}")).collect::<Vec<_>>()}),
    ] {
        assert!(sanitize_config(bad).is_err());
    }
}
#[test]
fn platforms_and_architectures_are_explicit_never_assumed() {
    for (name, expected) in [
        ("tool-win-x64.zip", Some(("windows", "x86_64"))),
        (
            "tool_aarch64-apple-darwin.tar.gz",
            Some(("macos", "aarch64")),
        ),
        ("tool-linux-armv7.deb", Some(("linux", "armv7"))),
        ("tool-universal.dmg", Some(("macos", "universal"))),
        ("tool.dmg", Some(("macos", "unknown"))),
        (
            "tool-x86_64-unknown-linux-gnu.tar.gz",
            Some(("linux", "x86_64")),
        ),
        ("tool-source-windows-x64.zip", None),
        ("tool-linux-x64-debug.tar.gz", None),
        ("tool.zip", None),
        ("tool-linux.zip", None),
        ("tool-x64.tar.gz", None),
        ("tool-windows-x64.exe.sig", None),
        ("tool-macos-windows-x64.zip", None),
    ] {
        assert_eq!(classify(name), expected, "{name}");
    }
}
#[tokio::test]
async fn official_digest_preserved_rename_stable_id_and_unknown_license_not_open() {
    let first = build_tool(repo(), release(), &client(), &mut budget())
        .await
        .unwrap();
    assert_eq!(first.key, "github-tool-12345");
    assert_eq!(first.data["artifacts"][0]["sha256"], hash(b"demo"));
    assert_eq!(first.data["artifacts"][0]["hash_source"], "github_digest");
    assert_eq!(first.data["artifacts"][0]["mirror_url"], "");
    assert!(!first.data.to_string().contains("IGNORED_FULL_BODY"));
    let mut renamed = repo();
    renamed["name"] = json!("NewTool");
    renamed["full_name"] = json!("owner/NewTool");
    renamed["license"] = Value::Null;
    let mut newer = release();
    newer["html_url"] = json!("https://github.com/owner/NewTool/releases/tag/v1.0");
    newer["assets"][0]["browser_download_url"] =
        json!("https://github.com/owner/NewTool/releases/download/v1.0/NewTool-windows-x64.exe");
    newer["assets"][0]["name"] = json!("NewTool-windows-x64.exe");
    let second = build_tool(renamed, newer, &client(), &mut budget())
        .await
        .unwrap();
    assert_eq!(first.key, second.key);
    assert_eq!(second.data["license"], "unknown");
    let mut unknown = repo();
    unknown["license"] = json!({"spdx_id":"NOASSERTION"});
    assert_eq!(
        build_tool(unknown, release(), &client(), &mut budget())
            .await
            .unwrap()
            .data["license"],
        "unknown"
    );
}
#[tokio::test]
async fn draft_prerelease_empty_unknown_and_unsafe_assets_do_not_replace_versions() {
    for flag in ["draft", "prerelease"] {
        let mut value = release();
        value[flag] = json!(true);
        assert!(build_tool(repo(), value, &client(), &mut budget())
            .await
            .is_err());
    }
    for name in ["source.zip", "tool.zip", "sha256sum.txt"] {
        let mut value = release();
        value["assets"] = json!([asset(name)]);
        assert!(build_tool(repo(), value, &client(), &mut budget())
            .await
            .is_err());
    }
    for (field, bad) in [
        ("digest", json!("sha256:bad")),
        ("browser_download_url", json!("http://127.0.0.1/private")),
        (
            "browser_download_url",
            json!("https://github.com/other/repo/releases/download/v1/a.exe"),
        ),
    ] {
        let mut value = release();
        value["assets"][0][field] = bad;
        assert!(build_tool(repo(), value, &client(), &mut budget())
            .await
            .is_err());
    }
}
#[tokio::test]
async fn snapshot_validator_rejects_forged_links_hash_arch_size_and_strips_unknown_fields() {
    let good = build_tool(repo(), release(), &client(), &mut budget())
        .await
        .unwrap()
        .data;
    for (path, value) in [
        ("sha256", json!("fake")),
        ("arch", json!("aarch64")),
        ("size", json!(0)),
        ("mirror_url", json!("https://evil.example/file")),
        (
            "github_url",
            json!("https://github.com.evil.example/release"),
        ),
        ("source_updated_at", json!("yesterday")),
    ] {
        let mut data = good.clone();
        data["artifacts"][0][path] = value;
        assert!(validate_tool(data).is_err(), "{path}");
    }
    let mut extra = good.clone();
    extra["untrusted_html"] = json!("<script>bad()</script>");
    assert!(validate_tool(extra)
        .unwrap()
        .get("untrusted_html")
        .is_none());
    let mut fake = good.clone();
    fake["platforms"] = json!(["Linux"]);
    assert!(validate_tool(fake).is_err());
}
async fn serve(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (base, task)
}
fn local(url: &Url) -> bool {
    url.scheme() == "http"
        && url.host_str() == Some("127.0.0.1")
        && url.username().is_empty()
        && url.password().is_none()
}
#[tokio::test]
async fn metadata_is_bounded_and_never_echoes_secret_or_upstream_body() {
    let app = Router::new()
        .route("/repo", get(|| async { axum::Json(repo()) }))
        .route(
            "/missing",
            get(|| async { (StatusCode::NOT_FOUND, "SECRET_UPSTREAM_BODY") }),
        )
        .route("/large", get(|| async { "x".repeat(MAX_METADATA + 1) }));
    let (base, task) = serve(app).await;
    assert_eq!(
        metadata(&client(), &format!("{base}/repo"), "secret")
            .await
            .unwrap()["id"],
        12345
    );
    for endpoint in ["missing", "large"] {
        let err = metadata(&client(), &format!("{base}/{endpoint}"), "secret")
            .await
            .unwrap_err();
        assert!(!err.contains("SECRET"));
        assert!(!err.contains("secret"));
    }
    task.abort();
}
#[tokio::test]
async fn missing_digest_is_stream_hashed_size_checked_and_budgeted() {
    let app = Router::new().route("/asset", get(|| async { b"demo".to_vec() }));
    let (base, task) = serve(app).await;
    let mut usage = budget();
    let actual = hash_download(
        &client(),
        &format!("{base}/asset"),
        4,
        None,
        &mut usage,
        local,
    )
    .await
    .unwrap();
    assert_eq!(actual, hash(b"demo"));
    assert_eq!(usage.remaining, TOTAL_DOWNLOAD - 4);
    assert!(hash_download(
        &client(),
        &format!("{base}/asset"),
        5,
        None,
        &mut budget(),
        local
    )
    .await
    .is_err());
    assert!(hash_download(
        &client(),
        &format!("{base}/asset"),
        4,
        Some(&"0".repeat(64)),
        &mut budget(),
        local
    )
    .await
    .unwrap_err()
    .contains("SHA-256"));
    let mut small = budget();
    small.remaining = 3;
    assert!(hash_download(
        &client(),
        &format!("{base}/asset"),
        4,
        None,
        &mut small,
        local
    )
    .await
    .is_err());
    task.abort();
}
#[tokio::test]
async fn redirect_is_bounded_host_checked_and_never_carries_metadata_token() {
    let seen = Arc::new(Mutex::new(Vec::<bool>::new()));
    async fn binary(State(seen): State<Arc<Mutex<Vec<bool>>>>, headers: HeaderMap) -> Vec<u8> {
        seen.lock()
            .unwrap()
            .push(headers.contains_key(AUTHORIZATION));
        b"demo".to_vec()
    }
    let app = Router::new()
        .route(
            "/redirect",
            get(|| async { (StatusCode::FOUND, [("location", "/asset")]) }),
        )
        .route("/asset", get(binary))
        .route(
            "/escape",
            get(|| async {
                (
                    StatusCode::FOUND,
                    [("location", "http://169.254.169.254/latest/meta-data")],
                )
            }),
        )
        .route(
            "/loop",
            get(|| async { (StatusCode::FOUND, [("location", "/loop")]) }),
        )
        .with_state(seen.clone());
    let (base, task) = serve(app).await;
    let downloads = client();
    assert_eq!(
        hash_download(
            &downloads,
            &format!("{base}/redirect"),
            4,
            None,
            &mut budget(),
            local
        )
        .await
        .unwrap(),
        hash(b"demo")
    );
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    for p in ["escape", "loop"] {
        assert!(hash_download(
            &downloads,
            &format!("{base}/{p}"),
            4,
            None,
            &mut budget(),
            local
        )
        .await
        .is_err());
    }
    for url in [
        "https://github.com@evil.example/x",
        "https://127.0.0.1/x",
        "https://github.com.evil.example/x",
        "http://github.com/o/r/releases/download/v/a",
        "https://user:pass@github.com/o/r/releases/download/v/a",
    ] {
        assert!(!download_url(&Url::parse(url).unwrap()));
    }
    task.abort();
}

#[test]
fn helper_programs_and_source_archives_are_never_primary_installers() {
    assert!(main_asset("codex-aarch64-apple-darwin.tar.gz", "codex"));
    assert!(main_asset("tool-v1.2.3-linux-x64.tar.gz", "tool"));
    assert!(!main_asset(
        "codex-app-server-aarch64-apple-darwin.tar.gz",
        "codex"
    ));
    assert!(!main_asset(
        "argument-comment-lint-x86_64-pc-windows-msvc.zip",
        "codex"
    ));
}
#[tokio::test]
async fn no_installers_under_size_limit_fails_and_cannot_publish_empty_replacement() {
    let mut r = release();
    r["assets"][0]["size"] = json!(MAX_ASSET + 1);
    assert!(build_tool(repo(), r, &client(), &mut budget())
        .await
        .err()
        .unwrap()
        .contains("128 MiB"));
}
#[tokio::test]
#[ignore = "official GitHub metadata only; never download a binary in this smoke test"]
async fn live_codex_metadata_produces_official_digest_snapshot_without_asset_downloads() {
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("XiaobanCommunityCollector/1.0")
        .build()
        .unwrap();
    let repo = metadata(&http, &format!("{API}/repos/openai/codex"), "")
        .await
        .unwrap();
    let release = metadata(
        &http,
        &format!("{API}/repos/openai/codex/releases/latest"),
        "",
    )
    .await
    .unwrap();
    let name = repo["name"].as_str().unwrap();
    for a in release["assets"].as_array().unwrap() {
        if main_asset(a["name"].as_str().unwrap(), name)
            && classify(a["name"].as_str().unwrap()).is_some()
            && a["size"].as_u64().unwrap() <= MAX_ASSET
        {
            assert!(
                a["digest"]
                    .as_str()
                    .is_some_and(|v| v.starts_with("sha256:") && sha(&v[7..])),
                "Missing official digest: stop before any binary download"
            );
        }
    }
    let item = build_tool(repo, release, &client(), &mut budget())
        .await
        .unwrap();
    assert_eq!(item.data["source"], "GitHub Releases");
    assert!(item.data["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .all(|a| a["hash_source"] == "github_digest"));
}

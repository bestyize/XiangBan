//! 端到端测试：真实 PostgreSQL + 本机模拟的上游目录。
//! 需要 `TEST_DATABASE_URL`（有 CREATEDB 权限的账号，例如 postgres://market:market@127.0.0.1/postgres）；未设置时跳过。
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use server_xiaoban_market::{build_state, config::Config, router};
use std::{
    collections::HashMap,
    io::Write,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in entries {
        w.start_file(*name, opts).unwrap();
        w.write_all(data).unwrap();
    }
    w.finish().unwrap().into_inner()
}

// ───────────── 模拟上游 ─────────────

#[derive(Clone)]
struct Mock {
    summary: Arc<Mutex<String>>,
    total: Arc<AtomicU64>,
    skill_requests: Arc<AtomicU64>,
}

async fn mock_skills(State(m): State<Mock>) -> Json<Value> {
    m.skill_requests.fetch_add(1, Ordering::SeqCst);
    let summary = m.summary.lock().unwrap().clone();
    Json(json!({"data": {"total": m.total.load(Ordering::SeqCst), "skills": [
        {"slug": "pdf-toolkit", "name": "PDF 文档处理", "description_zh": summary, "namespace": {"handle": "user_1"},
         "category": "文档处理", "source": "official", "labels": ["PDF", "表格"], "version": "1.0.0", "downloads": 10},
        {"slug": "web-to-md", "name": "网页转 Markdown", "description_zh": "抓取网页正文", "namespace": {"handle": "user_1"},
         "category": "Web 工具", "source": "community", "version": "1.0.0"},
        {"slug": "Bad Slug", "name": "非法标识"},
        {"slug": "banned-one", "name": "已下架", "banned": true},
    ]}}))
}

async fn mock_skill_detail(Path(slug): Path<String>) -> Json<Value> {
    Json(json!({"skill": {"slug": slug, "banned": false}, "namespace": {"handle": "user_1"},
        "latestVersion": {"version": "1.0.0", "changelog": "首发"}}))
}

async fn mock_download(Query(q): Query<HashMap<String, String>>) -> Vec<u8> {
    let slug = q.get("slug").cloned().unwrap_or_default();
    zip_bytes(&[
        ("SKILL.md", format!("---\nname: {slug}\ndescription: 示例技能\n---\n# {slug}\n").as_bytes()),
        ("scripts/run.py", b"print('hi')"),
    ])
}

async fn mock_generic_list(Json(body): Json<Value>) -> Result<Json<Value>, StatusCode> {
    // 模板里的 {page} / {page_size} 必须被替换成数字。
    let (page, size) = (body["page"].as_u64().ok_or(StatusCode::BAD_REQUEST)?, body["page_size"].as_u64().ok_or(StatusCode::BAD_REQUEST)?);
    assert_eq!(size, 2);
    let all = [
        json!({"id": "m-1", "name": "File Server", "desc": "读写本地文件", "tags": ["文件", "本地"], "cat": "工具", "logo": "https://example.com/a.png",
               "cfg": {"mcpServers": {"fs": {"command": "npx", "args": ["-y", "fs"]}}}}),
        json!({"id": "m-2", "name": "天气", "desc": "查询天气", "tags": ["天气"], "cat": "工具"}),
        json!({"id": "m-3", "name": "数据库", "desc": "只读查询", "tags": "数据库,SQL", "cat": "数据"}),
    ];
    let items: Vec<Value> = all.iter().skip(((page - 1) * size) as usize).take(size as usize).cloned().collect();
    Ok(Json(json!({"data": {"items": items, "total": 3}})))
}

async fn spawn_mock() -> (String, Mock) {
    let mock = Mock { summary: Arc::new(Mutex::new("读取、拆分、合并 PDF".into())), total: Arc::new(AtomicU64::new(2)), skill_requests: Arc::new(AtomicU64::new(0)) };
    let app = Router::new()
        .route("/api/skills", get(mock_skills))
        .route("/api/v1/skills/{slug}", get(mock_skill_detail))
        .route("/api/v1/download", get(mock_download))
        .route("/list", post(mock_generic_list))
        .with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, mock)
}

// ───────────── 测试应用 ─────────────

struct Account {
    key: jsonwebtoken::EncodingKey,
}

struct App {
    account: Account,
    base: String,
    upstream: String,
    mock: Mock,
    http: reqwest::Client,
    token: String,
    admin_url: String,
    db_name: String,
    _data: std::path::PathBuf,
}

async fn setup() -> Option<App> {
    setup_with(false).await
}

/// 在本机起一个只提供 JWKS 的「账号中心」，并返回它的地址和签名密钥。
async fn spawn_account() -> (String, Account) {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use ring::signature::{Ed25519KeyPair, KeyPair};
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
    let x = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
    let jwks = json!({"keys": [{"kty": "OKP", "crv": "Ed25519", "alg": "EdDSA", "use": "sig", "kid": "test-key", "x": x}]});
    let app = Router::new().route("/.well-known/jwks.json", get(move || { let j = jwks.clone(); async move { Json(j) } }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, Account { key: jsonwebtoken::EncodingKey::from_ed_der(pkcs8.as_ref()) })
}

async fn setup_with(account_enabled: bool) -> Option<App> {
    let admin_url = std::env::var("TEST_DATABASE_URL").ok()?;
    let db_name = format!("mkt_{}", uuid::Uuid::new_v4().simple());
    let admin = sqlx::PgPool::connect(&admin_url).await.unwrap();
    sqlx::query(&format!("CREATE DATABASE \"{db_name}\"")).execute(&admin).await.unwrap();
    let (prefix, _) = admin_url.rsplit_once('/').unwrap();
    let data = std::env::temp_dir().join(&db_name);
    let (account_url, account) = spawn_account().await;
    let config = Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: format!("{prefix}/{db_name}"),
        jwt_secret: "test-secret-test-secret-test-secret".into(),
        admin_user: "admin".into(),
        admin_pass: Some("test-pass-123".into()),
        data_dir: data.clone(),
        web_dir: "does-not-exist".into(),
        allow_private_upstream: true,
        account_issuer: account_enabled.then(|| "https://account.test".to_string()),
        account_jwks_url: format!("{account_url}/.well-known/jwks.json"),
        account_url: account_url.clone(),
        account_client_id: "xiaoban-market".into(),
    };
    let state = build_state(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router(state)).await.unwrap() });
    let (upstream, mock) = spawn_mock().await;
    let http = reqwest::Client::new();
    let login: Value = http.post(format!("{base}/api/v1/admin/login")).json(&json!({"username": "admin", "password": "test-pass-123"})).send().await.unwrap().json().await.unwrap();
    let token = login["token"].as_str().unwrap().to_string();
    Some(App { account, base, upstream, mock, http, token, admin_url, db_name, _data: data })
}

impl App {
    /// 以账号中心的身份签发一个访问令牌（可指定受众、发行方和过期时间）。
    fn account_token(&self, sub: uuid::Uuid, iss: &str, aud: &str, ttl: i64) -> String {
        let exp = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 + ttl) as usize;
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::EdDSA);
        header.kid = Some("test-key".into());
        jsonwebtoken::encode(&header, &json!({"sub": sub, "iss": iss, "aud": aud, "exp": exp}), &self.account.key).unwrap()
    }

    async fn call(&self, method: reqwest::Method, path: &str, body: Option<Value>, auth: bool) -> (u16, Value) {
        let mut req = self.http.request(method, format!("{}{}", self.base, path));
        if auth {
            req = req.bearer_auth(&self.token);
        }
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }
    async fn get(&self, path: &str) -> Value {
        let (s, v) = self.call(reqwest::Method::GET, path, None, true).await;
        assert_eq!(s, 200, "GET {path}: {v}");
        v
    }
    async fn post(&self, path: &str, body: Value) -> Value {
        let (s, v) = self.call(reqwest::Method::POST, path, Some(body), true).await;
        assert_eq!(s, 200, "POST {path}: {v}");
        v
    }
    async fn put(&self, path: &str, body: Value) -> Value {
        let (s, v) = self.call(reqwest::Method::PUT, path, Some(body), true).await;
        assert_eq!(s, 200, "PUT {path}: {v}");
        v
    }
    async fn source_id(&self, key: &str) -> i64 {
        let v = self.get("/api/v1/admin/sources").await;
        v["results"].as_array().unwrap().iter().find(|s| s["key"] == key).unwrap()["id"].as_i64().unwrap()
    }
    async fn run_job(&self, source: i64, params: Value) -> Value {
        let job = self.post(&format!("/api/v1/admin/sources/{source}/jobs"), params).await;
        self.wait_job(job["id"].as_str().unwrap()).await
    }
    async fn wait_job(&self, id: &str) -> Value {
        for _ in 0..150 {
            let v = self.get(&format!("/api/v1/admin/jobs/{id}")).await;
            let status = v["job"]["status"].as_str().unwrap().to_string();
            if status != "queued" && status != "running" {
                return v;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("job {id} did not finish");
    }
    async fn staged(&self, query: &str) -> Vec<Value> {
        self.get(&format!("/api/v1/admin/staged?{query}")).await["results"].as_array().unwrap().clone()
    }
    async fn configure_skillhub(&self) -> i64 {
        let id = self.source_id("skillhub-skills").await;
        self.put(&format!("/api/v1/admin/sources/{id}"), json!({"config": {"base_url": self.upstream, "origin": "skillhub", "request_interval_ms": 0}})).await;
        id
    }
    async fn cleanup(self) {
        let admin = sqlx::PgPool::connect(&self.admin_url).await.unwrap();
        sqlx::query(&format!("DROP DATABASE \"{}\" WITH (FORCE)", self.db_name)).execute(&admin).await.ok();
        std::fs::remove_dir_all(&self._data).ok();
    }
}

macro_rules! app {
    () => {
        match setup().await {
            Some(app) => app,
            None => {
                eprintln!("TEST_DATABASE_URL 未设置，跳过");
                return;
            }
        }
    };
}

macro_rules! app_with_account {
    () => {
        match setup_with(true).await {
            Some(app) => app,
            None => {
                eprintln!("TEST_DATABASE_URL 未设置，跳过");
                return;
            }
        }
    };
}

// ───────────── 用例 ─────────────

#[tokio::test]
async fn admin_endpoints_require_a_valid_token() {
    let app = app!();
    let (status, _) = app.call(reqwest::Method::GET, "/api/v1/admin/stats", None, false).await;
    assert_eq!(status, 401);
    let (status, _) = app.call(reqwest::Method::POST, "/api/v1/admin/login", Some(json!({"username": "admin", "password": "wrong"})), false).await;
    assert_eq!(status, 401);
    // 登录接口、公开接口不需要令牌。
    let (status, v) = app.call(reqwest::Method::GET, "/api/v1/market/skills", None, false).await;
    assert_eq!((status, v["total"].as_i64()), (200, Some(0)));
    let (status, _) = app.call(reqwest::Method::GET, "/api/v1/market/apps", None, false).await;
    assert_eq!(status, 404);
    app.cleanup().await;
}

#[tokio::test]
async fn skillhub_crawl_stage_import_search_download_and_update_flow() {
    let app = app!();
    let source = app.configure_skillhub().await;

    // 1. 首次采集：两条有效，非法标识与已下架被跳过。
    let done = app.run_job(source, json!({})).await;
    assert_eq!(done["job"]["status"], "succeeded", "{done}");
    let stats = &done["job"]["stats"];
    assert_eq!((stats["new"].as_i64(), stats["unchanged"].as_i64(), stats["skipped"].as_i64()), (Some(2), Some(0), Some(2)));
    let staged = app.staged("status=pending").await;
    assert_eq!(staged.len(), 2);
    assert!(staged.iter().all(|s| s["change"] == "new"));

    // 2. 内容不变时再次采集：全部无变化。
    let again = app.run_job(source, json!({})).await;
    assert_eq!(again["job"]["stats"]["unchanged"], 2);
    assert_eq!(again["job"]["stats"]["new"], 0);

    // 3. 入库并上架：二次抓取详情并下载校验安装包；上游分类「文档处理」命中种子分类。
    let ids: Vec<Value> = staged.iter().map(|s| s["id"].clone()).collect();
    let imported = app.post("/api/v1/admin/staged/import", json!({"ids": ids, "status": "published"})).await;
    assert!(imported["results"].as_array().unwrap().iter().all(|r| r["ok"] == true), "{imported}");
    assert_eq!(app.staged("status=imported").await.len(), 2);

    // 4. 公开搜索：子串匹配、分类过滤、拼写容错。
    let (_, list) = app.call(reqwest::Method::GET, "/api/v1/market/skills?keyword=pdf", None, false).await;
    assert_eq!(list["total"], 1);
    assert_eq!(list["results"][0]["slug"], "pdf-toolkit");
    assert_eq!(list["results"][0]["category"]["slug"], "docs");
    let (_, cn) = app.call(reqwest::Method::GET, "/api/v1/market/skills?keyword=%E6%96%87%E6%A1%A3", None, false).await;
    assert_eq!(cn["total"], 1, "中文子串应能命中：{cn}");
    let (_, typo) = app.call(reqwest::Method::GET, "/api/v1/market/skills?keyword=pdf-tolkit", None, false).await;
    assert_eq!(typo["total"], 0);
    assert_eq!(typo["near"][0]["slug"], "pdf-toolkit", "拼写错误应在 near 里给出相近匹配：{typo}");
    let (_, by_cat) = app.call(reqwest::Method::GET, "/api/v1/market/skills?category=docs", None, false).await;
    assert_eq!(by_cat["total"], 1);
    let (_, cats) = app.call(reqwest::Method::GET, "/api/v1/market/categories?kind=skills", None, false).await;
    assert_eq!(cats["results"].as_array().unwrap().iter().find(|c| c["slug"] == "docs").unwrap()["count"], 1);
    let (_, hint) = app.call(reqwest::Method::GET, "/api/v1/market/suggest?kind=skills&q=PD", None, false).await;
    assert_eq!(hint["results"][0]["slug"], "pdf-toolkit");
    // 通配符被当作普通字符，不会匹配全部。
    let (_, wild) = app.call(reqwest::Method::GET, "/api/v1/market/skills?keyword=%25", None, false).await;
    assert_eq!(wild["total"], 0);

    // 5. 详情：版本、文件清单。
    let (_, detail) = app.call(reqwest::Method::GET, "/api/v1/market/skills/user_1/pdf-toolkit", None, false).await;
    assert_eq!(detail["item"]["name"], "PDF 文档处理");
    assert_eq!(detail["item"]["description"].as_str().unwrap().contains("# pdf-toolkit"), true);
    assert_eq!(detail["versions"][0]["version"], "1.0.0");
    let files: Vec<&str> = detail["files"].as_array().unwrap().iter().map(|f| f["path"].as_str().unwrap()).collect();
    assert_eq!(files, vec!["SKILL.md", "scripts/run.py"]);
    let sha = detail["versions"][0]["packageSha256"].as_str().unwrap().to_string();

    // 6. 下载：校验头、去重计数、Range。
    let url = format!("{}{}", app.base, detail["versions"][0]["downloadUrl"].as_str().unwrap());
    let resp = app.http.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["x-content-sha256"].to_str().unwrap(), sha);
    let body = resp.bytes().await.unwrap();
    assert!(body.starts_with(b"PK"));
    assert_eq!(server_xiaoban_market::package::sha256_hex(&body), sha);
    app.http.get(&url).send().await.unwrap();
    let ranged = app.http.get(&url).header("range", "bytes=0-3").send().await.unwrap();
    assert_eq!(ranged.status(), 206);
    assert_eq!(ranged.bytes().await.unwrap().len(), 4);
    let (_, after) = app.call(reqwest::Method::GET, "/api/v1/market/skills/user_1/pdf-toolkit", None, false).await;
    assert_eq!(after["item"]["downloads"], 1, "同一来源同一天只计一次");

    // 7. 客户端更新检测。
    let (_, upd) = app
        .call(
            reqwest::Method::POST,
            "/api/v1/market/updates",
            Some(json!({"items": [
                {"kind": "skill", "namespace": "user_1", "slug": "pdf-toolkit", "version": "0.9.0"},
                {"kind": "skill", "namespace": "user_1", "slug": "pdf-toolkit", "version": "1.0.0"},
                {"kind": "skill", "namespace": "user_1", "slug": "nope", "version": "1"},
            ]})),
            false,
        )
        .await;
    let r = upd["results"].as_array().unwrap();
    assert_eq!((r[0]["hasUpdate"].as_bool(), r[1]["hasUpdate"].as_bool(), r[2]["available"].as_bool()), (Some(true), Some(false), Some(false)));
    assert_eq!(r[0]["latest"], "1.0.0");

    // 8. 上游内容变化：回到待处理（有更新），重新入库保持已上架状态。
    *app.mock.summary.lock().unwrap() = "全新的摘要".into();
    let changed = app.run_job(source, json!({})).await;
    assert_eq!((changed["job"]["stats"]["changed"].as_i64(), changed["job"]["stats"]["unchanged"].as_i64()), (Some(1), Some(1)));
    let pending = app.staged("status=pending&change=changed").await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["slug"], "pdf-toolkit");
    app.post("/api/v1/admin/staged/import", json!({"ids": [pending[0]["id"]]})).await;
    let items = app.get("/api/v1/admin/items?kind=skill&keyword=pdf").await;
    assert_eq!(items["results"][0]["summary"], "全新的摘要");
    assert_eq!(items["results"][0]["status"], "published");

    // 9. 撤回版本：不可下载，列表里不再提供下载地址。
    let item_id = items["results"][0]["id"].as_str().unwrap();
    let admin_item = app.get(&format!("/api/v1/admin/items/{item_id}")).await;
    let version_id = admin_item["versions"][0]["id"].as_str().unwrap();
    app.put(&format!("/api/v1/admin/versions/{version_id}"), json!({"yanked": true})).await;
    assert_eq!(app.http.get(&url).send().await.unwrap().status(), 404);
    let (_, yanked) = app.call(reqwest::Method::GET, "/api/v1/market/skills/user_1/pdf-toolkit", None, false).await;
    assert_eq!(yanked["versions"][0]["yanked"], true);
    assert!(yanked["versions"][0]["downloadUrl"].is_null());

    // 10. 下架后公开接口看不到。
    app.post(&format!("/api/v1/admin/items/{item_id}/status"), json!({"status": "offline"})).await;
    let (status, _) = app.call(reqwest::Method::GET, "/api/v1/market/skills/user_1/pdf-toolkit", None, false).await;
    assert_eq!(status, 404);
    app.cleanup().await;
}

#[tokio::test]
async fn ignored_items_stay_ignored_and_can_be_restored() {
    let app = app!();
    let source = app.configure_skillhub().await;
    app.run_job(source, json!({})).await;
    let staged = app.staged("status=pending").await;
    let id = staged[0]["id"].clone();
    app.post("/api/v1/admin/staged/ignore", json!({"ids": [id]})).await;
    assert_eq!(app.staged("status=ignored").await.len(), 1);
    // 上游变化后，已忽略的保持忽略。
    *app.mock.summary.lock().unwrap() = "变了".into();
    app.run_job(source, json!({})).await;
    assert_eq!(app.staged("status=ignored").await.len(), 1);
    app.post("/api/v1/admin/staged/restore", json!({"ids": [id]})).await;
    assert_eq!(app.staged("status=pending").await.len(), 2);
    app.cleanup().await;
}

#[tokio::test]
async fn auto_import_publishes_new_items_but_never_changes_existing_status() {
    let app = app!();
    let source = app.configure_skillhub().await;
    app.put(&format!("/api/v1/admin/sources/{source}"), json!({"autoImport": "published"})).await;
    let done = app.run_job(source, json!({})).await;
    assert_eq!(done["job"]["stats"]["imported"], 2, "{done}");
    let (_, list) = app.call(reqwest::Method::GET, "/api/v1/market/skills", None, false).await;
    assert_eq!(list["total"], 2);
    // 管理员下架其中一个，上游变化后自动入库不会把它重新上架。
    let pdf = list["results"].as_array().unwrap().iter().find(|i| i["slug"] == "pdf-toolkit").unwrap()["id"].as_str().unwrap().to_string();
    app.post(&format!("/api/v1/admin/items/{pdf}/status"), json!({"status": "offline"})).await;
    *app.mock.summary.lock().unwrap() = "上游改了".into();
    app.run_job(source, json!({})).await;
    let (_, list) = app.call(reqwest::Method::GET, "/api/v1/market/skills", None, false).await;
    assert_eq!(list["total"], 1);
    let admin = app.get(&format!("/api/v1/admin/items/{pdf}")).await;
    assert_eq!((admin["item"]["status"].as_str(), admin["item"]["summary"].as_str()), (Some("offline"), Some("上游改了")));
    app.cleanup().await;
}

#[tokio::test]
async fn one_active_job_per_source_and_jobs_can_be_cancelled() {
    let app = app!();
    let source = app.configure_skillhub().await;
    // 让上游声称有很多页，并在页间等待，这样任务会持续一段时间。
    app.mock.total.store(1000, Ordering::SeqCst);
    app.put(&format!("/api/v1/admin/sources/{source}"), json!({"config": {"base_url": app.upstream, "origin": "skillhub", "request_interval_ms": 700}})).await;
    let job = app.post(&format!("/api/v1/admin/sources/{source}/jobs"), json!({"max_pages": 20})).await;
    let id = job["id"].as_str().unwrap().to_string();
    let (status, body) = app.call(reqwest::Method::POST, &format!("/api/v1/admin/sources/{source}/jobs"), Some(json!({})), true).await;
    assert_eq!(status, 409, "{body}");
    tokio::time::sleep(Duration::from_millis(300)).await;
    app.post(&format!("/api/v1/admin/jobs/{id}/cancel"), json!({})).await;
    let done = app.wait_job(&id).await;
    assert_eq!(done["job"]["status"], "cancelled", "{done}");
    assert!(done["logs"].as_array().unwrap().iter().any(|l| l["message"].as_str().unwrap().contains("任务已取消")));
    // 取消后可以再次启动。
    app.mock.total.store(2, Ordering::SeqCst);
    assert_eq!(app.run_job(source, json!({})).await["job"]["status"], "succeeded");
    app.cleanup().await;
}

#[tokio::test]
async fn generic_json_source_probe_crawl_and_import_connectors() {
    let app = app!();
    let source = app.source_id("modelscope-mcp").await;

    // 未配置接口时，任务失败并给出可操作的提示；试采也一样。
    let probe = app.post(&format!("/api/v1/admin/sources/{source}/probe"), json!({})).await;
    assert_eq!(probe["ok"], false);
    assert!(probe["error"].as_str().unwrap().contains("接口地址尚未配置"));
    let failed = app.run_job(source, json!({})).await;
    assert_eq!(failed["job"]["status"], "failed");

    // 配置：POST + 请求体模板 + 字段映射。
    let config = json!({
        "origin": "modelscope", "url": format!("{}/list", app.upstream), "method": "POST",
        "body": {"page": "{page}", "page_size": "{page_size}", "keyword": "{keyword}"},
        "items_path": "/data/items", "total_path": "/data/total", "page_size": 2, "request_interval_ms": 0,
        "mapping": {"upstream_id": "/id", "name": "/name", "summary": "/desc", "tags": "/tags", "category": "/cat", "icon_url": "/logo", "version": "/missing"},
        "config_path": "/cfg", "origin_url_template": "https://example.com/mcp/{upstream_id}",
        "headers": {"Authorization": "Bearer secret-token"}
    });
    app.put(&format!("/api/v1/admin/sources/{source}"), json!({"config": config})).await;

    // 配置里的密钥不会回显。
    let sources = app.get("/api/v1/admin/sources").await;
    let shown = sources["results"].as_array().unwrap().iter().find(|s| s["id"] == source).unwrap();
    assert_eq!(shown["config"]["headers"]["Authorization"], "***");
    assert_eq!(shown["configured"], true);

    // 试采：展示原文、规范化结果，以及每个字段是否命中。
    let probe = app.post(&format!("/api/v1/admin/sources/{source}/probe"), json!({})).await;
    assert_eq!(probe["ok"], true, "{probe}");
    assert_eq!((probe["count"].as_i64(), probe["total"].as_i64(), probe["hasMore"].as_bool()), (Some(2), Some(3), Some(true)));
    assert_eq!(probe["entries"][0]["normalized"]["name"], "File Server");
    let hit = |field: &str| probe["mapping"].as_array().unwrap().iter().find(|m| m["field"] == field).unwrap()["hit"].as_bool().unwrap();
    assert!(hit("name") && hit("tags") && hit("icon_url"));
    assert!(!hit("version"), "不存在的字段应报告未命中");

    // 保存时回传 *** 不会覆盖真实密钥（再次试采仍然成功即可说明未破坏配置）。
    app.put(&format!("/api/v1/admin/sources/{source}"), json!({"config": shown["config"]})).await;

    // 采集两页，共三条。
    let done = app.run_job(source, json!({})).await;
    assert_eq!(done["job"]["status"], "succeeded", "{done}");
    assert_eq!((done["job"]["stats"]["pages"].as_i64(), done["job"]["stats"]["new"].as_i64()), (Some(2), Some(3)));

    // 入库：配置模板解析进 metadata，分类「工具」自动新建，随后记住映射。
    let staged = app.staged("status=pending").await;
    let fs = staged.iter().find(|s| s["upstreamId"] == "m-1").unwrap();
    let imported = app.post("/api/v1/admin/staged/import", json!({"ids": [fs["id"]], "status": "published"})).await;
    assert_eq!(imported["results"][0]["ok"], true, "{imported}");
    let (_, list) = app.call(reqwest::Method::GET, "/api/v1/market/connectors?keyword=%E6%96%87%E4%BB%B6", None, false).await;
    assert_eq!(list["total"], 1, "{list}");
    let ns = list["results"][0]["namespace"].as_str().unwrap();
    let (_, detail) = app.call(reqwest::Method::GET, &format!("/api/v1/market/connectors/{ns}/{}", list["results"][0]["slug"].as_str().unwrap()), None, false).await;
    assert_eq!(detail["item"]["metadata"]["configs"][0]["name"], "fs");
    assert_eq!(detail["item"]["originUrl"], "https://example.com/mcp/m-1");
    assert_eq!(detail["item"]["origin"], "modelscope");
    assert_eq!(detail["item"]["category"]["name"], "工具");

    // 第二条同分类：直接命中刚创建的分类，不会重复创建。
    let weather = staged.iter().find(|s| s["upstreamId"] == "m-2").unwrap();
    app.post("/api/v1/admin/staged/import", json!({"ids": [weather["id"]], "status": "draft"})).await;
    let cats = app.get("/api/v1/admin/categories?kind=connector").await;
    assert_eq!(cats["results"].as_array().unwrap().iter().filter(|c| c["name"] == "工具").count(), 1);
    app.cleanup().await;
}

#[tokio::test]
async fn manual_items_and_package_uploads_follow_client_rules() {
    let app = app!();
    // 手动新建技能草稿。
    let created = app
        .post("/api/v1/admin/items", json!({"kind": "skills", "slug": "my-skill", "name": "我的技能", "summary": "手工上架", "tags": ["a", "a", "b"], "prompts": ["试试这个"]}))
        .await;
    let id = created["id"].as_str().unwrap().to_string();
    let (status, _) = app.call(reqwest::Method::POST, "/api/v1/admin/items", Some(json!({"kind": "skill", "slug": "my-skill", "name": "重复"})), true).await;
    assert_eq!(status, 409);
    let (status, _) = app.call(reqwest::Method::POST, "/api/v1/admin/items", Some(json!({"kind": "skill", "slug": "Bad Slug", "name": "x"})), true).await;
    assert_eq!(status, 400);
    // 没有可用版本不能上架。
    let (status, body) = app.call(reqwest::Method::POST, &format!("/api/v1/admin/items/{id}/status"), Some(json!({"status": "published"})), true).await;
    assert_eq!(status, 400, "{body}");

    let upload = |version: &str, bytes: Vec<u8>| {
        let form = reqwest::multipart::Form::new()
            .text("version", version.to_string())
            .text("changelog", "测试")
            .part("file", reqwest::multipart::Part::bytes(bytes).file_name("pkg.zip"));
        app.http.post(format!("{}/api/v1/admin/items/{id}/versions", app.base)).bearer_auth(&app.token).multipart(form).send()
    };
    // 不合规的包（缺 SKILL.md / 路径穿越）被拒绝，且不会留下版本。
    assert_eq!(upload("1.0.0", zip_bytes(&[("README.md", b"x")])).await.unwrap().status(), 400);
    assert_eq!(upload("1.0.0", zip_bytes(&[("SKILL.md", b"x"), ("../evil.txt", b"x")])).await.unwrap().status(), 400);
    assert_eq!(upload("1.0.0", b"not a zip".to_vec()).await.unwrap().status(), 400);
    // 合规的包成功，重复版本号冲突。
    let ok = upload("1.0.0", zip_bytes(&[("pkg/SKILL.md", b"# hi"), ("pkg/scripts/a.py", b"1")])).await.unwrap();
    assert_eq!(ok.status(), 200);
    let ok: Value = ok.json().await.unwrap();
    assert_eq!(ok["files"], 2);
    assert_eq!(upload("1.0.0", zip_bytes(&[("SKILL.md", b"# hi")])).await.unwrap().status(), 409);
    app.post(&format!("/api/v1/admin/items/{id}/status"), json!({"status": "published"})).await;
    let (_, detail) = app.call(reqwest::Method::GET, "/api/v1/market/skills/xiaoban/my-skill", None, false).await;
    assert_eq!(detail["item"]["tags"], json!(["a", "b"]));
    assert_eq!(detail["item"]["prompts"], json!(["试试这个"]));
    assert_eq!(detail["item"]["version"], "1.0.0");

    // 编辑后搜索文本同步更新。
    app.put(&format!("/api/v1/admin/items/{id}"), json!({"name": "全新名字", "tags": ["独特标签"]})).await;
    let (_, found) = app.call(reqwest::Method::GET, "/api/v1/market/skills?keyword=%E7%8B%AC%E7%89%B9", None, false).await;
    assert_eq!(found["total"], 1);
    // 改密码需要当前密码。
    let (status, _) = app.call(reqwest::Method::PUT, "/api/v1/admin/account", Some(json!({"currentPassword": "bad", "newPassword": "new-password-1"})), true).await;
    assert_eq!(status, 401);
    app.put("/api/v1/admin/account", json!({"currentPassword": "test-pass-123", "newPassword": "new-password-1"})).await;
    let (status, _) = app.call(reqwest::Method::POST, "/api/v1/admin/login", Some(json!({"username": "admin", "password": "new-password-1"})), false).await;
    assert_eq!(status, 200);
    app.cleanup().await;
}

#[tokio::test]
async fn upstream_urls_must_be_public_https_unless_tests_allow_private() {
    use server_xiaoban_market::upstream::validate_url;
    assert!(validate_url("https://www.example.com/a", false).is_ok());
    for bad in ["http://example.com", "https://localhost/a", "https://127.0.0.1/a", "https://10.0.0.5/a", "https://192.168.1.1/a", "https://[::1]/a", "https://user:pw@example.com", "ftp://x.com", "nonsense"] {
        assert!(validate_url(bad, false).is_err(), "{bad} 应被拒绝");
    }
    assert!(validate_url("http://127.0.0.1:8080/a", true).is_ok());
}

#[tokio::test]
async fn account_login_requires_binding_and_unbinding_revokes_sessions() {
    let app = app_with_account!();
    let iss = "https://account.test";
    let me = uuid::Uuid::new_v4();

    // 登录页配置。
    let (_, cfg) = app.call(reqwest::Method::GET, "/api/v1/admin/config", None, false).await;
    assert_eq!((cfg["account"]["enabled"].as_bool(), cfg["account"]["clientId"].as_str()), (Some(true), Some("xiaoban-market")));

    // 1. 账号有效但没有被绑定：不能登录。
    let good = app.account_token(me, iss, "xiaoban-market", 600);
    let login = |token: String| app.call(reqwest::Method::POST, "/api/v1/admin/login/account", Some(json!({"accessToken": token})), false);
    let (status, body) = login(good.clone()).await;
    assert_eq!(status, 401, "{body}");
    assert!(body["message"].as_str().unwrap().contains("绑定"));

    // 2. 本地管理员绑定。必须是本应用的令牌，别的应用的令牌不能被绑定。
    let (status, _) = app.call(reqwest::Method::POST, "/api/v1/admin/account/bindings", Some(json!({"accessToken": app.account_token(me, iss, "other-app", 600), "label": "x"})), true).await;
    assert_eq!(status, 401);
    let (status, _) = app.call(reqwest::Method::POST, "/api/v1/admin/account/bindings", Some(json!({"accessToken": good, "label": "   "})), true).await;
    assert_eq!(status, 400);
    app.post("/api/v1/admin/account/bindings", json!({"accessToken": good, "label": "张三"})).await;
    let bindings = app.get("/api/v1/admin/account/bindings").await;
    assert_eq!(bindings["results"].as_array().unwrap().len(), 1);
    assert_eq!(bindings["results"][0]["boundBy"], "admin");

    // 3. 绑定后可以登录，并能访问后台接口。
    let (status, session) = login(good.clone()).await;
    assert_eq!(status, 200, "{session}");
    assert_eq!(session["username"], "张三");
    let acct_token = session["token"].as_str().unwrap().to_string();
    let with_acct = |path: &'static str| {
        let (http, url, t) = (app.http.clone(), format!("{}{}", app.base, path), acct_token.clone());
        async move { http.get(url).bearer_auth(t).send().await.unwrap().status().as_u16() }
    };
    assert_eq!(with_acct("/api/v1/admin/stats").await, 200);

    // 4. 受众、发行方、过期、伪造都不行。
    for (name, token) in [
        ("错误受众", app.account_token(me, iss, "other-app", 600)),
        ("错误发行方", app.account_token(me, "https://evil.example", "xiaoban-market", 600)),
        ("已过期", app.account_token(me, iss, "xiaoban-market", -600)),
        ("乱码", "not-a-token".to_string()),
    ] {
        let (status, _) = login(token).await;
        assert_eq!(status, 401, "{name} 应被拒绝");
    }
    // 用服务自己的密钥伪造（HS256）的令牌，也不能当作账号令牌。
    let forged = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &json!({"sub": me, "iss": iss, "aud": "xiaoban-market", "exp": 4102444800u64}), &jsonwebtoken::EncodingKey::from_secret(b"test-secret-test-secret-test-secret")).unwrap();
    assert_eq!(login(forged).await.0, 401);

    // 5. 解绑后，已经签发的后台令牌在下一次请求就失效；重新登录也不行。本地管理员不受影响。
    app.call(reqwest::Method::DELETE, &format!("/api/v1/admin/account/bindings/{}", bindings["results"][0]["id"].as_str().unwrap()), None, true).await;
    assert_eq!(with_acct("/api/v1/admin/stats").await, 401);
    assert_eq!(login(good).await.0, 401);
    assert_eq!(app.call(reqwest::Method::GET, "/api/v1/admin/stats", None, true).await.0, 200);
    app.cleanup().await;
}

#[tokio::test]
async fn account_login_is_off_unless_configured() {
    let app = app!();
    let (_, cfg) = app.call(reqwest::Method::GET, "/api/v1/admin/config", None, false).await;
    assert_eq!(cfg["account"]["enabled"], false);
    let (status, _) = app.call(reqwest::Method::POST, "/api/v1/admin/login/account", Some(json!({"accessToken": "x"})), false).await;
    assert_eq!(status, 400);
    app.cleanup().await;
}

#[tokio::test]
async fn expert_teams_and_skill_bundles_reference_published_members() {
    let app = app!();
    let make = |kind: &str, slug: &str, extra: Value| {
        let mut body = json!({"kind": kind, "slug": slug, "name": format!("名称-{slug}"), "summary": "示例", "status": "published"});
        body.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        app.call(reqwest::Method::POST, "/api/v1/admin/items", Some(body), true)
    };
    // 两个专家直接上架；一个草稿专家。
    assert_eq!(make("expert", "pm", json!({})).await.0, 200);
    assert_eq!(make("expert", "dev", json!({})).await.0, 200);
    let (_, draft) = make("expert", "wip", json!({"status": "draft"})).await;
    let wip_id = draft["id"].as_str().unwrap().to_string();

    // 校验：成员必须存在、类型匹配、不重复、数量合法；团队只能是专家，套件只能是技能。
    let team = |members: Value| make("expert", "dream-team", json!({"variant": "team", "members": members}));
    assert_eq!(team(json!([])).await.0, 400);
    assert_eq!(team(json!([{"namespace": "xiaoban", "slug": "ghost"}])).await.0, 400);
    assert_eq!(team(json!([{"namespace": "xiaoban", "slug": "pm"}, {"namespace": "xiaoban", "slug": "pm"}])).await.0, 400);
    let (status, body) = team(json!([{"namespace": "xiaoban", "slug": "wip"}])).await;
    assert_eq!(status, 400, "未上架的成员不能让团队上架：{body}");
    assert!(body["message"].as_str().unwrap().contains("xiaoban/wip"));
    assert_eq!(make("skill", "bad-team", json!({"variant": "team", "members": [{"namespace": "xiaoban", "slug": "pm"}]})).await.0, 400);
    assert_eq!(make("expert", "single-with-members", json!({"members": [{"namespace": "xiaoban", "slug": "pm"}]})).await.0, 400);

    // 成功创建，顺序按登记的来。
    let (status, _) = team(json!([{"namespace": "xiaoban", "slug": "dev"}, {"namespace": "xiaoban", "slug": "pm"}])).await;
    assert_eq!(status, 200);
    // 团队不能再当成员（不嵌套）。
    assert_eq!(make("expert", "super-team", json!({"variant": "team", "members": [{"namespace": "xiaoban", "slug": "dream-team"}]})).await.0, 400);

    // 公开列表：variant 过滤，团队带成员数。
    let (_, all) = app.call(reqwest::Method::GET, "/api/v1/market/experts", None, false).await;
    assert_eq!(all["total"], 3);
    let (_, teams) = app.call(reqwest::Method::GET, "/api/v1/market/experts?variant=team", None, false).await;
    assert_eq!((teams["total"].as_i64(), teams["results"][0]["slug"].as_str(), teams["results"][0]["memberCount"].as_i64()), (Some(1), Some("dream-team"), Some(2)));
    let (_, singles) = app.call(reqwest::Method::GET, "/api/v1/market/experts?variant=single", None, false).await;
    assert_eq!(singles["total"], 2);
    assert!(singles["results"].as_array().unwrap().iter().all(|i| i["variant"] == "single"));

    // 详情：成员解析成最新信息，顺序保持。
    let (_, detail) = app.call(reqwest::Method::GET, "/api/v1/market/experts/xiaoban/dream-team", None, false).await;
    assert_eq!(detail["item"]["variant"], "team");
    let slugs: Vec<&str> = detail["members"].as_array().unwrap().iter().map(|m| m["slug"].as_str().unwrap()).collect();
    assert_eq!(slugs, vec!["dev", "pm"]);
    assert_eq!(detail["members"][0]["name"], "名称-dev");

    // 成员下架后，前台详情不再展示它（不会出现打不开的成员）；已上架的团队改成员也要求成员已上架。
    let admin_items = app.get("/api/v1/admin/items?kind=expert&keyword=dev").await;
    let dev_id = admin_items["results"].as_array().unwrap().iter().find(|i| i["slug"] == "dev").unwrap()["id"].as_str().unwrap().to_string();
    app.post(&format!("/api/v1/admin/items/{dev_id}/status"), json!({"status": "offline"})).await;
    let (_, detail) = app.call(reqwest::Method::GET, "/api/v1/market/experts/xiaoban/dream-team", None, false).await;
    assert_eq!(detail["members"].as_array().unwrap().len(), 1);
    let team_id = app.get("/api/v1/admin/items?kind=expert&variant=team").await["results"][0]["id"].as_str().unwrap().to_string();
    let (status, _) = app.call(reqwest::Method::PUT, &format!("/api/v1/admin/items/{team_id}"), Some(json!({"members": [{"namespace": "xiaoban", "slug": "wip"}]})), true).await;
    assert_eq!(status, 400);
    let (status, _) = app.call(reqwest::Method::PUT, &format!("/api/v1/admin/items/{team_id}"), Some(json!({"members": [{"namespace": "xiaoban", "slug": "pm"}]})), true).await;
    assert_eq!(status, 200);
    let admin = app.get(&format!("/api/v1/admin/items/{team_id}")).await;
    assert_eq!(admin["members"].as_array().unwrap().len(), 1);
    assert_eq!(admin["members"][0]["status"], "published");
    // 成员上架后再重新上架 wip 与团队。
    app.post(&format!("/api/v1/admin/items/{wip_id}/status"), json!({"status": "published"})).await;

    // 套件：成员是技能，套件本身没有安装包也能上架；按来源过滤。
    assert_eq!(make("skill", "s1", json!({"status": "draft"})).await.0, 200);
    let (status, body) = make("skill", "kit", json!({"variant": "bundle", "members": [{"namespace": "xiaoban", "slug": "s1"}]})).await;
    assert_eq!(status, 400, "{body}");
    let (_, s1) = app.call(reqwest::Method::GET, "/api/v1/admin/items?kind=skill&keyword=s1", None, true).await;
    let s1_id = s1["results"][0]["id"].as_str().unwrap().to_string();
    // 单个技能没有版本不能上架……
    let (status, _) = app.call(reqwest::Method::POST, &format!("/api/v1/admin/items/{s1_id}/status"), Some(json!({"status": "published"})), true).await;
    assert_eq!(status, 400);
    // ……上传安装包后可以。
    let form = reqwest::multipart::Form::new().text("version", "1.0.0").part("file", reqwest::multipart::Part::bytes(zip_bytes(&[("SKILL.md", b"# s1")])).file_name("s.zip"));
    assert_eq!(app.http.post(format!("{}/api/v1/admin/items/{s1_id}/versions", app.base)).bearer_auth(&app.token).multipart(form).send().await.unwrap().status(), 200);
    app.post(&format!("/api/v1/admin/items/{s1_id}/status"), json!({"status": "published"})).await;
    let (status, _) = make("skill", "kit", json!({"variant": "bundle", "members": [{"namespace": "xiaoban", "slug": "s1"}]})).await;
    assert_eq!(status, 200);
    let (_, bundles) = app.call(reqwest::Method::GET, "/api/v1/market/skills?variant=bundle", None, false).await;
    assert_eq!((bundles["total"].as_i64(), bundles["results"][0]["slug"].as_str()), (Some(1), Some("kit")));
    let (_, kit) = app.call(reqwest::Method::GET, "/api/v1/market/skills/xiaoban/kit", None, false).await;
    assert_eq!(kit["members"][0]["slug"], "s1");
    assert!(kit["versions"].as_array().unwrap().is_empty(), "套件本身没有安装包");
    let (_, manual) = app.call(reqwest::Method::GET, "/api/v1/market/skills?origin=manual&variant=single", None, false).await;
    assert_eq!(manual["total"], 1);
    let (_, none) = app.call(reqwest::Method::GET, "/api/v1/market/skills?origin=skillhub", None, false).await;
    assert_eq!(none["total"], 0);
    app.cleanup().await;
}

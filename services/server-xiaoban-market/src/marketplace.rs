//! Read-only marketplace traversal, deterministic packaging and draft imports.
use crate::{connectors, package, plugin};
use serde_json::{json, Value};
use std::{collections::BTreeMap, io::{Cursor, Read, Write}, path::{Component, Path, PathBuf}, time::Duration};

pub const USAGE: &str = "import-marketplace --dir <目录> --server <https://…> --token <管理员令牌> [--dry-run] [--only <name>,<name>]";

pub struct Options {
    pub dir: PathBuf,
    pub server: String,
    // Never derive Debug or serialize options: this contains an administrator credential.
    token: String,
    pub dry_run: bool,
    pub only: Vec<String>,
}

impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let (mut dir, mut server, mut token, mut only) = (None, None, None, Vec::new());
        let mut dry_run = false;
        while let Some(arg) = args.next() {
            if arg == "--dry-run" { dry_run = true; continue; }
            if !matches!(arg.as_str(), "--dir" | "--server" | "--token" | "--only") { return Err(USAGE.into()); }
            let value = args.next().filter(|s| !s.is_empty() && !s.starts_with("--")).ok_or(USAGE)?;
            match arg.as_str() {
                "--dir" => dir = Some(PathBuf::from(value)),
                "--server" => server = Some(value),
                "--token" => token = Some(value),
                _ => only = value.split(',').filter(|s| !s.trim().is_empty()).map(|s| s.trim().into()).collect(),
            }
        }
        Ok(Self { dir: dir.ok_or(USAGE)?, server: server.ok_or(USAGE)?, token: token.ok_or(USAGE)?, dry_run, only })
    }

    pub fn redact(&self, message: &str) -> String { message.replace(&self.token, "[REDACTED]") }
}

#[derive(Clone)]
pub struct Entry { pub name: String, source: String, metadata: Value }

pub fn discover(root: &Path) -> Result<Vec<Entry>, String> {
    let index = [root.join(".codebuddy-plugin/marketplace.json"), root.join("marketplace.json")].into_iter().find(|p| p.exists());
    let mut entries = if let Some(index) = index {
        let bytes = std::fs::read(index).map_err(|_| "无法读取 marketplace.json")?;
        let v: Value = serde_json::from_slice(&bytes).map_err(|_| "marketplace.json 无效")?;
        v["plugins"].as_array().ok_or("marketplace.json 缺少 plugins 数组")?.iter().enumerate().map(|(i, p)| Entry {
            name: p["name"].as_str().unwrap_or("").to_string(),
            source: p["source"].as_str().unwrap_or("").to_string(),
            metadata: if p.is_object() { p.clone() } else { json!({"invalidEntry":i}) },
        }).collect::<Vec<_>>()
    } else {
        let base = if root.join("connectors").is_dir() { root.join("connectors") } else { root.to_path_buf() };
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&base).map_err(|_| "无法读取市场目录")? {
            let entry = entry.map_err(|_| "无法读取市场目录条目")?;
            if entry.path().is_dir() {
                let name = entry.file_name().to_str().ok_or("目录名不是 UTF-8")?.to_string();
                let source = entry.path().strip_prefix(root).map_err(|_| "目录不在市场内")?.to_str().ok_or("目录名不是 UTF-8")?.replace('\\', "/");
                out.push(Entry { name, source, metadata: json!({}) });
            }
        }
        out
    };
    entries.sort_by(|a,b| (&a.name, &a.source).cmp(&(&b.name, &b.source)));
    Ok(entries)
}

fn is_link(meta: &std::fs::Metadata) -> bool {
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 { return true; }
    }
    meta.file_type().is_symlink()
}

fn collect(dir: &Path, root: &Path, files: &mut BTreeMap<String, Vec<u8>>, total: &mut u64, depth: usize) -> Result<(), String> {
    if depth > 12 { return Err("插件目录过深".into()); }
    for entry in std::fs::read_dir(dir).map_err(|_| "读取插件目录失败")? {
        let path = entry.map_err(|_| "读取插件文件失败")?.path();
        // WorkBuddy adds this local download timestamp outside the published plugin.
        // It is neither package content nor stable across installations.
        if dir == root && path.file_name().is_some_and(|name| name == ".downloaded_at") { continue; }
        let meta = std::fs::symlink_metadata(&path).map_err(|_| "读取文件属性失败")?;
        if is_link(&meta) { return Err("插件不能包含符号链接或重解析点".into()); }
        if meta.is_dir() { collect(&path, root, files, total, depth + 1)?; }
        else if meta.is_file() {
            if files.len() >= package::MAX_PLUGIN_FILES { return Err("插件文件过多".into()); }
            let mut bytes = Vec::new();
            std::fs::File::open(&path).map_err(|_| "读取插件文件失败")?.take(package::MAX_FILE_BYTES + 1).read_to_end(&mut bytes).map_err(|_| "读取插件文件失败")?;
            *total += bytes.len() as u64;
            if bytes.len() as u64 > package::MAX_FILE_BYTES || *total > package::MAX_TOTAL_BYTES { return Err("插件文件超过大小限制".into()); }
            let name = path.strip_prefix(root).map_err(|_| "非法插件路径")?.to_str().ok_or("文件名不是 UTF-8")?.replace('\\', "/");
            files.insert(name, bytes);
        } else { return Err("插件包含非常规文件".into()); }
    }
    Ok(())
}

pub fn provenance(m: &plugin::Manifest, fallback: &Value, license: Option<&str>) -> (String, Value) {
    let license = m.license.as_deref().filter(|s| !s.trim().is_empty()).or(license.filter(|s| !s.trim().is_empty())).unwrap_or("unknown").to_string();
    let mut attribution = json!({});
    for (key, own) in [("author", &m.author), ("homepage", &m.homepage), ("repository", &m.repository)] {
        attribution[key] = own.as_ref().filter(|v| !v.is_null()).cloned().or_else(|| fallback.get(key).cloned()).unwrap_or(Value::Null);
    }
    (license, attribution)
}

pub struct Prepared { pub bytes: Vec<u8>, pub license: String, pub attribution: Value }

/// None denotes an unsupported skill-only plugin, not a broken connector.
pub fn prepare(root: &Path, entry: &Entry) -> Result<Option<Prepared>, String> {
    if entry.source.is_empty() || entry.source.contains(['\\', ':']) || Path::new(&entry.source).components().any(|c| !matches!(c, Component::Normal(_) | Component::CurDir)) {
        return Err("source 必须是市场内的相对路径".into());
    }
    let mut dir = root.to_path_buf();
    for c in Path::new(&entry.source).components() {
        dir.push(c);
        if is_link(&std::fs::symlink_metadata(&dir).map_err(|_| "source 目录不存在")?) { return Err("source 不允许符号链接或重解析点".into()); }
    }
    let root_path = root.canonicalize().map_err(|_| "市场目录不存在")?;
    if !dir.canonicalize().map_err(|_| "source 目录不存在")?.starts_with(root_path) { return Err("source 超出市场目录".into()); }
    let mut files = BTreeMap::new();
    collect(&dir, &dir, &mut files, &mut 0, 0)?;
    let manifest = package::MANIFEST_DIRS.iter().find_map(|d| files.get(&format!("{d}/plugin.json"))).ok_or("缺少 plugin.json")?;
    let m = plugin::parse_manifest(manifest)?;
    if !entry.name.is_empty() && entry.name != m.name { return Err("市场名称与 plugin.json 的 name 不一致".into()); }
    let kind = match connectors::kind(&m, &files) {
        Ok(kind) => kind,
        Err(_) if !files.contains_key("mcp.json") && !files.contains_key(".mcp.json") && files.keys().any(|p| p.starts_with("skills/")) && m.expert_type.is_empty() => return Ok(None),
        Err(e) => return Err(e),
    };
    if !crate::upstream::valid_slug(&m.name) || m.version.is_empty() || m.version.len() > 44 || !m.version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b,b'.'|b'-'|b'+')) { return Err("插件名称或版本无效".into()); }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).last_modified_time(zip::DateTime::default()).unix_permissions(0o644);
    for (path, bytes) in &files {
        writer.start_file(path, options).map_err(|_| "创建 ZIP 失败")?;
        writer.write_all(bytes).map_err(|_| "写入 ZIP 失败")?;
    }
    let bytes = writer.finish().map_err(|_| "完成 ZIP 失败")?.into_inner();
    let pkg = plugin::inspect_plugin_zip(&bytes)?;
    let issues = if kind == "connector" { connectors::validate(&m, &pkg.unpacked.contents) } else { plugin::validate_expert(&m, &pkg.unpacked.contents, true) };
    let errors: Vec<_> = issues.iter().filter(|i| i.error).map(|i| i.message.as_str()).collect();
    if !errors.is_empty() { return Err(errors.join("；")); }
    let (license, attribution) = provenance(&m, &entry.metadata, entry.metadata["license"].as_str());
    Ok(Some(Prepared { bytes, license, attribution }))
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Summary { pub created: usize, pub updated: usize, pub skipped: usize, pub skills: usize, pub failed: usize, pub checked: usize }

async fn upload(client: &reqwest::Client, endpoint: &str, token: &str, p: Prepared) -> Result<&'static str, String> {
    let form = reqwest::multipart::Form::new().text("status", "draft").text("origin", "workbuddy-official").text("source", "official")
        .text("lenient", "true").text("license", p.license).text("attribution", p.attribution.to_string())
        .part("file", reqwest::multipart::Part::bytes(p.bytes).file_name("plugin.zip"));
    let response = client.post(endpoint).bearer_auth(token).multipart(form).send().await.map_err(|_| "HTTP 请求失败（连接或超时）")?;
    if !response.status().is_success() { return Err(format!("HTTP {}", response.status().as_u16())); }
    let v: Value = response.json().await.map_err(|_| "服务端响应不是 JSON")?;
    match (v["skipped"].as_bool(), v["created"].as_bool()) {
        (Some(true), _) => Ok("skipped"),
        (_, Some(true)) => Ok("created"),
        (_, Some(false)) => Ok("updated"),
        _ => Err("服务端响应缺少导入结果".into()),
    }
}

pub async fn run(opts: &Options, mut output: impl FnMut(String)) -> Result<Summary, String> {
    let mut emit = |s: String| output(opts.redact(&s));
    let endpoint = format!("{}/api/v1/admin/plugins/import", opts.server.trim_end_matches('/'));
    let url = reqwest::Url::parse(&endpoint).map_err(|_| "server 地址无效")?;
    if !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some()
        || !(url.scheme() == "https" || (url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")))) { return Err("server 必须是 HTTPS 地址（本机测试允许 HTTP）且不含凭据、查询或片段".into()); }
    let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(60)).build().map_err(|_| "HTTP 客户端初始化失败")?;
    let mut summary = Summary::default();
    let entries = discover(&opts.dir)?;
    for entry in entries.into_iter().filter(|e| opts.only.is_empty() || opts.only.contains(&e.name)) {
        let result = match prepare(&opts.dir, &entry) {
            Ok(None) => { summary.skills += 1; summary.skipped += 1; Ok("跳过（仅技能插件）") },
            Ok(Some(_)) if opts.dry_run => { summary.checked += 1; Ok("校验打包通过（未上传）") },
            Ok(Some(p)) => match upload(&client, url.as_str(), &opts.token, p).await {
                Ok("created") => { summary.created += 1; Ok("新增（草稿）") },
                Ok("updated") => { summary.updated += 1; Ok("更新（草稿）") },
                Ok(_) => { summary.skipped += 1; Ok("跳过（内容相同）") },
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };
        match result { Ok(label) => emit(format!("{}：{label}",entry.name)), Err(e) => { summary.failed += 1; emit(format!("{}：失败：{e}", entry.name)); } }
    }
    emit(format!("汇总：新增 {} / 更新 {} / 跳过 {}（仅技能 {}）/ 失败 {} / 校验 {}", summary.created, summary.updated, summary.skipped, summary.skills, summary.failed, summary.checked));
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{extract::{Multipart, State}, http::{HeaderMap, StatusCode}, routing::post, Json, Router};
    use std::sync::{Arc, Mutex};

    struct Market(PathBuf);
    impl Market {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("marketplace-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn write(&self, path: &str, bytes: &[u8]) {
            let path = self.0.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
        }
        fn connector(&self, name: &str, extra: Value) {
            let mut manifest = json!({"name":name, "version":"1.0.0"});
            for (k,v) in extra.as_object().unwrap() { manifest[k] = v.clone(); }
            self.write(&format!("connectors/{name}/.codebuddy-plugin/plugin.json"), manifest.to_string().as_bytes());
            self.write(&format!("connectors/{name}/mcp.json"), br#"{"mcpServers":{"test":{"type":"sse","url":"https://example.com/mcp"}}}"#);
        }
        fn index(&self, plugins: Value) { self.write(".codebuddy-plugin/marketplace.json", json!({"name":"fixture", "plugins":plugins}).to_string().as_bytes()); }
        fn options(&self, server: &str, dry_run: bool) -> Options {
            Options { dir: self.0.clone(), server: server.into(), token: "secret-admin-token".into(), dry_run, only: vec![] }
        }
    }
    impl Drop for Market { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

    #[test]
    fn deterministic_zip_sorted_paths_fixed_time_and_manifest_precedence() {
        let m = Market::new();
        m.connector("alpha", json!({"license":"MIT", "author":{"name":"Plugin author"}, "repository":"https://example.com/repo"}));
        m.write("connectors/alpha/z.txt", b"last");
        m.write("connectors/alpha/a.txt", b"first");
        m.write("connectors/alpha/.downloaded_at", b"first local timestamp");
        m.index(json!([{"name":"alpha", "source":"./connectors/alpha", "author":"Market author", "license":"Apache-2.0", "homepage":"https://example.com"}]));
        let entries = discover(&m.0).unwrap();
        let a = prepare(&m.0, &entries[0]).unwrap().unwrap();
        // Recreate a file, changing directory enumeration order and timestamps.
        std::fs::remove_file(m.0.join("connectors/alpha/a.txt")).unwrap();
        m.write("connectors/alpha/a.txt", b"first");
        m.write("connectors/alpha/.downloaded_at", b"another local timestamp");
        let b = prepare(&m.0, &entries[0]).unwrap().unwrap();
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.license, "MIT");
        assert_eq!(a.attribution["author"]["name"], "Plugin author");
        assert_eq!(a.attribution["homepage"], "https://example.com");
        assert_eq!(a.attribution["repository"], "https://example.com/repo");
        let mut zip = zip::ZipArchive::new(Cursor::new(a.bytes)).unwrap();
        let mut names = Vec::new();
        for i in 0..zip.len() {
            let f = zip.by_index(i).unwrap();
            names.push(f.name().to_string());
            let time = f.last_modified().unwrap();
            assert_eq!((time.year(), time.month(), time.day(), time.hour()), (1980,1,1,0));
        }
        let mut sorted = names.clone(); sorted.sort(); assert_eq!(names, sorted);
        assert!(!names.iter().any(|p| p == ".downloaded_at"));
        let (license, attribution) = provenance(&plugin::Manifest::default(), &json!({}), None);
        assert_eq!(license, "unknown");
        assert_eq!(attribution, json!({"author":null,"homepage":null,"repository":null}));
        let (license, attribution) = provenance(&plugin::Manifest::default(), &json!({"author":"Market author"}), Some("BSD-3-Clause"));
        assert_eq!(license,"BSD-3-Clause"); assert_eq!(attribution["author"], "Market author");
    }

    #[test]
    fn discovers_flat_connector_layout_and_validates_experts() {
        let m = Market::new();
        m.connector("alpha", json!({}));
        assert_eq!(discover(&m.0).unwrap().len(), 1);
        let entries = discover(&m.0.join("connectors")).unwrap();
        assert!(prepare(&m.0.join("connectors"), &entries[0]).unwrap().is_some());
        let manifest = json!({"name":"expert-one","version":"1.0.0","description":"Example expert","plugin":"expert-one",
            "expertType":"agent","agentName":"expert-one","agents":["agents/expert-one.md"],"skills":[],
            "avatar":"avatars/expert.png","categoryId":"08-FinanceInvestment",
            "displayName":"Example expert","profession":"Advisor","displayDescription":"An example expert."});
        m.write("plugins/expert-one/.codebuddy-plugin/plugin.json", manifest.to_string().as_bytes());
        m.write("plugins/expert-one/avatars/expert.png", &[0x89,b'P',b'N',b'G',13,10,26,10]);
        m.write("plugins/expert-one/agents/expert-one.md", b"---\nname: expert-one\ndescription: Example expert\n---\nYou are an expert.");
        m.index(json!([{"name":"expert-one","source":"./plugins/expert-one"}]));
        assert!(prepare(&m.0, &discover(&m.0).unwrap()[0]).unwrap().is_some());
    }

    #[test]
    fn refuses_escape_missing_sources_and_invalid_packages() {
        let m = Market::new();
        m.connector("alpha", json!({}));
        for source in ["../outside", "/absolute", "C:/absolute", "./connectors/../alpha", "connectors\\alpha", "missing"] {
            let e = Entry { name:"alpha".into(), source:source.into(), metadata:json!({}) };
            assert!(prepare(&m.0, &e).is_err(),"{source}");
        }
        let e = discover(&m.0).unwrap().remove(0);
        m.write("connectors/alpha/mcp.json", b"not json");
        assert!(prepare(&m.0, &e).is_err());
        m.connector("alpha", json!({}));
        m.write("connectors/alpha/.hidden", b"hidden file");
        assert!(prepare(&m.0, &e).is_err());
    }

    #[tokio::test]
    async fn dry_run_skips_skills_counts_failures_continues_and_only_selects_names() {
        let m = Market::new();
        m.connector("alpha", json!({}));
        m.write("plugins/skills/.codebuddy-plugin/plugin.json", br#"{"name":"skills","version":"1.0.0"}"#);
        m.write("plugins/skills/skills/helper/SKILL.md", b"# Helper");
        m.index(json!([
            {"name":"broken","source":"./missing"},
            {"name":"skills","source":"./plugins/skills"},
            {"name":"alpha","source":"./connectors/alpha"},
            {"name":"secret-admin-token","source":"../escape"}
        ]));
        let mut logs = Vec::new();
        let mut opts = m.options("http://127.0.0.1:1", true);
        let s = run(&opts, |line| logs.push(line)).await.unwrap();
        assert_eq!(s, Summary { checked:1, skipped:1, skills:1, failed:2, ..Default::default() });
        assert!(logs.join("\n").contains("跳过（仅技能插件）"));
        assert!(!logs.join("\n").contains("secret-admin-token"));
        opts.only = vec!["alpha".into()];
        assert_eq!(run(&opts, |_|{}).await.unwrap().checked,1);
        opts.only = vec!["nonexistent".into()];
        assert_eq!(run(&opts, |_|{}).await.unwrap(), Summary::default());
    }

    #[derive(Clone, Default)]
    struct Mock { requests: Arc<Mutex<Vec<Value>>> }
    async fn import(State(mock): State<Mock>, headers: HeaderMap, mut body: Multipart) -> (StatusCode, Json<Value>) {
        assert_eq!(headers["authorization"],"Bearer secret-admin-token");
        let mut fields = json!({});
        let mut name = String::new();
        while let Some(field) = body.next_field().await.unwrap() {
            let key = field.name().unwrap().to_string();
            if key == "file" {
                let bytes = field.bytes().await.unwrap();
                name = plugin::inspect_plugin_zip(&bytes).unwrap().manifest.name;
            } else { fields[key] = Value::String(field.text().await.unwrap()); }
        }
        mock.requests.lock().unwrap().push(fields);
        match name.as_str() {
            "alpha" => (StatusCode::OK, Json(json!({"created":true,"skipped":false}))),
            "beta" => (StatusCode::OK, Json(json!({"created":false,"skipped":false}))),
            "delta" => (StatusCode::OK, Json(json!({"created":false,"skipped":true}))),
            // A hostile server must never echo the administrator token into output.
            "epsilon" => (StatusCode::CONFLICT, Json(json!({"error":"secret-admin-token"}))),
            "gamma" => (StatusCode::OK, Json(json!({"unexpected":"secret-admin-token"}))),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({}))),
        }
    }

    #[tokio::test]
    async fn http_uploads_drafts_provenance_and_summarizes_new_updated_skipped_failed() {
        let m = Market::new();
        for name in ["alpha","beta","delta","epsilon","gamma"] { m.connector(name, json!({})); }
        let mock = Mock::default();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}",listener.local_addr().unwrap());
        let app = Router::new().route("/api/v1/admin/plugins/import", post(import)).with_state(mock.clone());
        let task = tokio::spawn(async move { axum::serve(listener,app).await.unwrap(); });
        let mut logs = Vec::new();
        let s = run(&m.options(&base, false), |line| logs.push(line)).await.unwrap();
        task.abort();
        assert_eq!(s, Summary { created:1, updated:1, skipped:1, failed:2, ..Default::default() });
        assert!(!logs.join("\n").contains("secret-admin-token"));
        let requests = mock.requests.lock().unwrap();
        assert_eq!(requests.len(),5);
        for req in requests.iter() {
            assert_eq!(req["status"],"draft"); assert_eq!(req["origin"],"workbuddy-official");
            assert_eq!(req["source"],"official"); assert_eq!(req["license"],"unknown");
            let attribution: Value = serde_json::from_str(req["attribution"].as_str().unwrap()).unwrap();
            assert_eq!(attribution["author"],Value::Null);
        }
    }

    #[tokio::test]
    async fn cli_errors_and_untrusted_urls_never_expose_tokens() {
        let args = ["--token", "secret-admin-token", "--bad", "secret-admin-token"].map(str::to_string);
        let err = Options::parse(args).err().unwrap();
        assert!(!err.contains("secret-admin-token"));
        let m = Market::new();
        for server in ["https://secret-admin-token@example.com", "http://example.com", "https://example.com?secret-admin-token", "invalid-secret-admin-token"] {
            let err = run(&m.options(server,true), |_|{}).await.unwrap_err();
            assert!(!err.contains("secret-admin-token"));
        }
    }
}

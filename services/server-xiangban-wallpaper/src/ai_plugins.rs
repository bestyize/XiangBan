//! Small, read-only SkillHub adapter. Only metadata is cached; archives and signed URLs are not.
use crate::{api::AppState, error::AppError};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use reqwest::{redirect::Policy, Client, Url};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::{Arc, Mutex}, time::{Duration, Instant}};
use tokio::sync::Semaphore;

const ORIGIN: &str = "https://api.skillhub.cn";
const PAGE_SIZE: u64 = 24;
const JSON_LIMIT: usize = 2 * 1024 * 1024;
const CACHE_BYTES: usize = 16 * 1024 * 1024;
const CACHE_TTL: Duration = Duration::from_secs(180);
type Result<T> = std::result::Result<T, AppError>;

struct Cached { expires: Instant, value: Value, size: usize }
pub struct Catalog {
    origin: String,
    api: Client,
    objects: Client,
    api_key: Option<String>,
    cache: Mutex<BTreeMap<String, Cached>>,
    requests: Semaphore,
    cooldown: Mutex<Option<Instant>>,
}

impl Catalog {
    fn new(api_key: Option<String>) -> Self {
        let api = Client::builder().timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5)).redirect(Policy::none())
            .user_agent("Xiaoban-Plugin-Market/1.0").build().expect("HTTP client configuration");
        // Never forward the team API key to object storage, including across redirects.
        let objects = Client::builder().timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .redirect(Policy::custom(|attempt| {
                if attempt.previous().len() < 3 && allowed_object(attempt.url()) {
                    attempt.follow()
                } else { attempt.stop() }
            })).build().expect("HTTP client configuration");
        Self { origin: ORIGIN.into(), api, objects, api_key, cache: Mutex::new(BTreeMap::new()),
            requests: Semaphore::new(8), cooldown: Mutex::new(None) }
    }

    async fn request(&self, route: &str, pairs: &[(&str, String)]) -> Result<reqwest::Response> {
        if self.cooldown.lock().unwrap().is_some_and(|until| Instant::now() < until) {
            return Err(AppError::TooManyRequests("插件数据源繁忙，请稍后重试".into()));
        }
        let mut request = self.api.get(format!("{}{route}", self.origin)).query(pairs);
        if let Some(key) = &self.api_key { request = request.header("X-API-Key", key); }
        let response = request.send().await.map_err(|_| upstream("暂时无法连接插件数据源"))?;
        if response.status().as_u16() == 429 {
            let seconds = response.headers().get(header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(30).clamp(1, 300);
            *self.cooldown.lock().unwrap() = Some(Instant::now() + Duration::from_secs(seconds));
        }
        Ok(response)
    }

    async fn json(&self, route: &str, pairs: &[(&str, String)]) -> Result<Value> {
        let key = Url::parse_with_params(&format!("{}{route}", self.origin), pairs)
            .map_err(|_| AppError::BadRequest("无效的插件查询".into()))?.to_string();
        if let Some(hit) = self.cache.lock().unwrap().get(&key).filter(|v| v.expires > Instant::now()) {
            return Ok(hit.value.clone());
        }
        let _permit = self.requests.try_acquire().map_err(|_| AppError::TooManyRequests("插件请求较多，请稍后重试".into()))?;
        let response = self.request(route, pairs).await?;
        check_status(response.status())?;
        let body = limited_body(response, JSON_LIMIT).await?;
        let value: Value = serde_json::from_slice(&body).map_err(|_| upstream("插件数据格式暂不可用"))?;
        if value.get("error").is_some() || value.get("code").is_some_and(|v| v != 0) {
            return Err(upstream("插件数据源返回错误，请稍后重试"));
        }
        let mut cache = self.cache.lock().unwrap();
        cache.retain(|_, v| v.expires > Instant::now());
        if cache.len() >= 128 || cache.values().map(|v| v.size).sum::<usize>() + body.len() > CACHE_BYTES {
            // Bounded process memory; no database, full-catalog sync or disk cache is required.
            cache.clear();
        }
        cache.insert(key, Cached { expires: Instant::now() + CACHE_TTL, value: value.clone(), size: body.len() });
        Ok(value)
    }

    async fn object(&self, route: &str, pairs: &[(&str, String)], limit: usize) -> Result<Vec<u8>> {
        let _permit = self.requests.try_acquire().map_err(|_| AppError::TooManyRequests("插件请求较多，请稍后重试".into()))?;
        let response = self.request(route, pairs).await?;
        let response = if response.status().is_redirection() {
            let target = response.headers().get(header::LOCATION).and_then(|v| v.to_str().ok())
                .and_then(|v| Url::parse(v).ok()).filter(allowed_object)
                .ok_or_else(|| upstream("插件下载地址暂不受支持，请到来源页面下载"))?;
            self.objects.get(target).send().await.map_err(|_| upstream("插件文件下载失败"))?
        } else { response };
        check_status(response.status())?;
        limited_body(response, limit).await
    }
}

fn upstream(message: &str) -> AppError {
    AppError::Provider { provider: "SkillHub", message: message.into() }
}
fn check_status(status: StatusCode) -> Result<()> {
    match status.as_u16() {
        200..=299 => Ok(()),
        404 => Err(AppError::NotFound("插件或指定版本不存在".into())),
        429 => Err(AppError::TooManyRequests("插件数据源限流，请稍后重试".into())),
        401 | 403 => Err(AppError::ServiceUnavailable("插件数据源暂不可用，请检查服务端接入配置".into())),
        _ => Err(upstream("插件数据源暂不可用，请稍后重试")),
    }
}
fn allowed_object(url: &Url) -> bool {
    // Current SkillHub downloads use Tencent COS. Fail closed if the upstream changes hosts.
    url.scheme() == "https" && url.username().is_empty() && url.password().is_none()
        && url.port_or_known_default() == Some(443)
        && url.host_str().is_some_and(|host| host.contains(".cos.") && host.ends_with(".myqcloud.com"))
}
async fn limited_body(mut response: reqwest::Response, max: usize) -> Result<Vec<u8>> {
    if response.content_length().is_some_and(|n| n > max as u64) {
        return Err(upstream("插件响应超过大小限制"));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| upstream("插件响应读取失败"))? {
        if bytes.len() + chunk.len() > max { return Err(upstream("插件响应超过大小限制")); }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
fn identifier(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max
        && value.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
fn version(value: &str) -> bool {
    !value.is_empty() && value.len() <= 44
        && value.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'-' | b'+'))
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Browse {
    keyword: String, page: Option<u32>, category: String, source: String, sort_by: Option<String>,
}
impl Browse {
    fn pairs(&self) -> Result<Vec<(&'static str, String)>> {
        let page = self.page.unwrap_or(1);
        let sort = self.sort_by.as_deref().unwrap_or("score");
        if self.keyword.trim().chars().count() > 80 || !(1..=10000).contains(&page)
            || (!self.category.is_empty() && !identifier(&self.category, 80))
            || !matches!(self.source.as_str(), "" | "official" | "community" | "enterprise")
            || !matches!(sort, "score" | "downloads" | "stars" | "installs" | "updated_at") {
            return Err(AppError::BadRequest("无效的插件搜索条件".into()));
        }
        Ok(vec![("keyword", self.keyword.trim().into()), ("page", page.to_string()),
            ("pageSize", PAGE_SIZE.to_string()), ("category", self.category.clone()),
            ("source", self.source.clone()), ("sortBy", sort.into()), ("order", "desc".into())])
    }
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Publisher { namespace: String }
impl Publisher {
    fn pairs(&self) -> Result<Vec<(&'static str, String)>> {
        if !self.namespace.is_empty() && !identifier(&self.namespace, 80) {
            return Err(AppError::BadRequest("无效的插件发布者".into()));
        }
        Ok(if self.namespace.is_empty() { vec![] } else { vec![("namespace", self.namespace.clone())] })
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release { version: String, #[serde(default)] namespace: String }
impl Release {
    fn pairs(&self, slug: &str) -> Result<Vec<(&'static str, String)>> {
        if !identifier(slug, 64) || !version(&self.version) {
            return Err(AppError::BadRequest("请选择有效的插件和固定版本".into()));
        }
        let mut pairs = Publisher { namespace: self.namespace.clone() }.pairs()?;
        pairs.push(("version", self.version.clone()));
        Ok(pairs)
    }
}
fn text<'a>(value: &'a Value, keys: &[&str]) -> &'a str {
    keys.iter().find_map(|key| value[*key].as_str().filter(|s| !s.trim().is_empty())).unwrap_or("")
}
fn namespace(value: &Value) -> &str {
    value["namespace"]["handle"].as_str().or(value["namespace"].as_str())
        .or(value["ownerName"].as_str()).filter(|v| identifier(v, 80)).unwrap_or("")
}
fn entry(value: &Value) -> Value {
    let icon = value["iconUrl"].as_str().and_then(|v| Url::parse(v).ok())
        .filter(|v| v.scheme() == "https" && v.username().is_empty() && v.password().is_none());
    json!({"slug": value["slug"], "name": text(value, &["name", "displayName", "slug"]),
        "description": text(value, &["description_zh", "summary_zh", "description", "summary"]),
        "version": value["version"], "namespace": namespace(value), "iconUrl": icon.map(|v| v.to_string()),
        "category": value["category"], "source": value["source"],
        "downloads": value["downloads"].as_u64().or(value["stats"]["downloads"].as_u64()).unwrap_or(0),
        "stars": value["stars"].as_u64().or(value["stats"]["stars"].as_u64()).unwrap_or(0),
        "labels": value["labels"]})
}
fn page(raw: &Value, current: u32) -> Result<Value> {
    let items = raw["data"]["skills"].as_array().ok_or_else(|| upstream("插件列表格式发生变化"))?;
    let total = raw["data"]["total"].as_u64().ok_or_else(|| upstream("插件列表缺少总数"))?;
    let results = items.iter().take(PAGE_SIZE as usize)
        .filter(|s| s["slug"].as_str().is_some_and(|v| identifier(v, 64)) && s["banned"] != true)
        .map(entry).collect::<Vec<_>>();
    Ok(json!({"provider":"skillhub", "results": results, "page": current, "total":total,
        "hasMore": u64::from(current) * PAGE_SIZE < total}))
}
fn categories(raw: &Value) -> Result<Value> {
    let items = raw["items"].as_array().ok_or_else(|| upstream("插件分类格式发生变化"))?;
    let results = items.iter().filter(|v| v["active"] != false && v["level"].as_u64() == Some(1))
        .filter_map(|v| {
            let key = v["key"].as_str().filter(|v| identifier(v, 80))?;
            let name = v["name"].as_str().filter(|v| !v.trim().is_empty())?;
            Some(json!({"key":key,"name":name}))
        }).collect::<Vec<_>>();
    Ok(json!({"results":results}))
}

async fn browse(State(catalog): State<Arc<Catalog>>, Query(query): Query<Browse>) -> Result<Json<Value>> {
    let raw = catalog.json("/api/skills", &query.pairs()?).await?;
    Ok(Json(page(&raw, query.page.unwrap_or(1))?))
}
async fn list_categories(State(catalog): State<Arc<Catalog>>) -> Result<Json<Value>> {
    Ok(Json(categories(&catalog.json("/api/v1/categories", &[]).await?)?))
}
async fn detail(State(catalog): State<Arc<Catalog>>, Path(slug): Path<String>, Query(query): Query<Publisher>) -> Result<Json<Value>> {
    if !identifier(&slug, 64) { return Err(AppError::BadRequest("无效的插件名称".into())); }
    let raw = catalog.json(&format!("/api/v1/skills/{slug}"), &query.pairs()?).await?;
    if raw["skill"]["slug"] != slug || raw["skill"]["banned"] == true {
        return Err(AppError::NotFound("插件暂不可用".into()));
    }
    let publisher = text(&raw["namespace"], &["handle"]);
    let publisher = if publisher.is_empty() { text(&raw["owner"], &["handle"]) } else { publisher };
    if (!query.namespace.is_empty() && query.namespace != publisher) || !identifier(publisher, 80) {
        return Err(upstream("插件发布者信息不一致，已停止加载"));
    }
    let mut skill = raw["skill"].clone();
    skill["namespace"] = json!(publisher);
    skill["version"] = raw["latestVersion"]["version"].clone();
    let source_url = format!("https://skillhub.cn/skills/{publisher}/{slug}");
    Ok(Json(json!({"entry":entry(&skill), "changelog":raw["latestVersion"]["changelog"],
        "sourceUrl":source_url, "publisher":publisher})))
}
async fn readme(State(catalog): State<Arc<Catalog>>, Path(slug): Path<String>, Query(query): Query<Release>) -> Result<Json<Value>> {
    let mut pairs = query.pairs(&slug)?;
    pairs.push(("path", "SKILL.md".into()));
    let bytes = catalog.object(&format!("/api/v1/skills/{slug}/file"), &pairs, 100 * 1024).await?;
    let content = String::from_utf8(bytes).map_err(|_| upstream("技能说明不是有效的 UTF-8 文本"))?;
    Ok(Json(json!({"content":content,"version":query.version})))
}
async fn download(State(catalog): State<Arc<Catalog>>, Path(slug): Path<String>, Query(query): Query<Release>) -> Result<Response> {
    let mut pairs = query.pairs(&slug)?;
    pairs.push(("slug", slug.clone()));
    let bytes = catalog.object("/api/v1/download", &pairs, 12 * 1024 * 1024).await?;
    if !bytes.starts_with(b"PK\x03\x04") { return Err(upstream("数据源未返回有效的 ZIP 插件包")); }
    let mut response = bytes.into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
    response.headers_mut().insert(header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{slug}-{}.zip\"", query.version))
            .map_err(|_| AppError::BadRequest("无效的文件名".into()))?);
    Ok(response)
}
async fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        response.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from_static("30"));
    }
    response
}
pub fn router() -> Router<AppState> {
    let api_key = std::env::var("SKILLHUB_API_KEY").ok().filter(|v| !v.trim().is_empty());
    Router::new()
        .route("/api/v1/ai/plugins", get(browse))
        .route("/api/v1/ai/plugins/categories", get(list_categories))
        .route("/api/v1/ai/plugins/{slug}", get(detail))
        .route("/api/v1/ai/plugins/{slug}/readme", get(readme))
        .route("/api/v1/ai/plugins/{slug}/download", get(download))
        .layer(axum::middleware::map_response(no_store))
        .with_state(Arc::new(Catalog::new(api_key)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct TestServer { origin: String, task: tokio::task::JoinHandle<()> }
    impl Drop for TestServer { fn drop(&mut self) { self.task.abort(); } }
    async fn serve(router: Router) -> TestServer {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
        TestServer { origin, task }
    }
    #[tokio::test]
    async fn metadata_cache_avoids_requests_and_expires() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let server = serve(Router::new().route("/metadata", get(move |headers: axum::http::HeaderMap| {
            let counter = counter.clone();
            async move {
                assert_eq!(headers["x-api-key"], "test-key");
                Json(json!({"requests":counter.fetch_add(1, Ordering::SeqCst) + 1}))
            }
        }))).await;
        let mut catalog = Catalog::new(Some("test-key".into()));
        catalog.origin = server.origin.clone();
        assert_eq!(catalog.json("/metadata", &[]).await.unwrap()["requests"], 1);
        assert_eq!(catalog.json("/metadata", &[]).await.unwrap()["requests"], 1);
        for value in catalog.cache.lock().unwrap().values_mut() { value.expires = Instant::now(); }
        assert_eq!(catalog.json("/metadata", &[]).await.unwrap()["requests"], 2);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
    #[tokio::test]
    async fn failures_are_not_cached_and_rate_limits_pause_upstream_requests() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let server = serve(Router::new().route("/limited", get(move || {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "60")])
            }
        }))).await;
        let mut catalog = Catalog::new(None);
        catalog.origin = server.origin.clone();
        assert!(matches!(catalog.json("/limited", &[]).await, Err(AppError::TooManyRequests(_))));
        assert!(matches!(catalog.json("/limited", &[]).await, Err(AppError::TooManyRequests(_))));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(catalog.cache.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn rejects_oversized_metadata_and_untrusted_file_redirects() {
        let server = serve(Router::new()
            .route("/large", get(|| async { "x".repeat(JSON_LIMIT + 1) }))
            .route("/file", get(|| async { axum::response::Redirect::temporary("http://127.0.0.1/private") }))).await;
        let mut catalog = Catalog::new(None);
        catalog.origin = server.origin.clone();
        assert!(catalog.json("/large", &[]).await.is_err());
        assert!(catalog.object("/file", &[], 100).await.is_err());
        assert!(catalog.cache.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn concurrency_limit_rejects_excess_work_without_queueing() {
        let catalog = Catalog::new(None);
        let _permits = catalog.requests.acquire_many(8).await.unwrap();
        assert!(matches!(catalog.json("/unused", &[]).await, Err(AppError::TooManyRequests(_))));
    }
    #[tokio::test]
    #[ignore = "manual upstream contract smoke test; requires network access"]
    async fn live_skillhub_contract() {
        let mut catalog = Catalog::new(std::env::var("SKILLHUB_API_KEY").ok());
        // Optional extra trust root is confined to this manual test, for development proxies.
        if let Ok(path) = std::env::var("SKILLHUB_SMOKE_CA_FILE") {
            let certificate = reqwest::Certificate::from_pem(&std::fs::read(path).unwrap()).unwrap();
            catalog.api = Client::builder().timeout(Duration::from_secs(20))
                .redirect(Policy::none()).add_root_certificate(certificate.clone()).build().unwrap();
            catalog.objects = Client::builder().timeout(Duration::from_secs(30))
                .redirect(Policy::custom(|attempt| {
                    if attempt.previous().len() < 3 && allowed_object(attempt.url()) { attempt.follow() }
                    else { attempt.stop() }
                })).add_root_certificate(certificate).build().unwrap();
        }
        let query = Browse { keyword:"find-skill-skillhub".into(), ..Browse::default() };
        let result = page(&catalog.json("/api/skills", &query.pairs().unwrap()).await.unwrap(), 1).unwrap();
        assert!(result["results"].as_array().unwrap().iter().any(|v| v["slug"] == "find-skill-skillhub"));
        assert!(!categories(&catalog.json("/api/v1/categories", &[]).await.unwrap()).unwrap()["results"].as_array().unwrap().is_empty());
        let coordinate = vec![("namespace", "user_290ac21c".into())];
        let info = catalog.json("/api/v1/skills/find-skill-skillhub", &coordinate).await.unwrap();
        assert_eq!(info["skill"]["slug"], "find-skill-skillhub");
        let file = vec![("namespace", "user_290ac21c".into()), ("version", "1.0.2".into()), ("path", "SKILL.md".into())];
        let document = catalog.object("/api/v1/skills/find-skill-skillhub/file", &file, 100 * 1024).await.unwrap();
        assert!(!std::str::from_utf8(&document).unwrap().trim().is_empty());
        let download = vec![("namespace", "user_290ac21c".into()), ("version", "1.0.2".into()), ("slug", "find-skill-skillhub".into())];
        assert!(catalog.object("/api/v1/download", &download, 12 * 1024 * 1024).await.unwrap().starts_with(b"PK\x03\x04"));
    }
    #[test]
    fn preserves_publisher_and_pagination_without_inventing_score() {
        let result = page(&json!({"data":{"total":49,"skills":[{"slug":"test-skill",
            "ownerName":"author_one","description":"English","description_zh":"中文", "score":100000}]}}), 2).unwrap();
        assert_eq!(result["hasMore"], true);
        assert_eq!(result["results"][0]["namespace"], "author_one");
        assert_eq!(result["results"][0]["description"], "中文");
        assert!(result["results"][0].get("score").is_none());
        assert!(page(&json!({"data":{}}), 1).is_err());
    }
    #[test]
    fn validates_filters_and_fixed_release_coordinates() {
        assert!(Browse { category:"办公".into(), ..Browse::default() }.pairs().is_err());
        assert!(Browse { source:"unknown".into(), ..Browse::default() }.pairs().is_err());
        assert!(Browse { page:Some(0), ..Browse::default() }.pairs().is_err());
        let pairs = Browse { keyword:"写作".into(), category:"content-creation".into(), ..Browse::default() }.pairs().unwrap();
        assert!(pairs.contains(&("keyword","写作".into())));
        assert!(pairs.contains(&("category","content-creation".into())));
        assert!(Release { version:"1.0.0".into(),namespace:"author_one".into() }.pairs("test-skill").is_ok());
        assert!(Release { version:"".into(),namespace:String::new() }.pairs("test-skill").is_err());
        assert!(Release { version:"1.0".into(),namespace:"../other".into() }.pairs("test-skill").is_err());
    }
    #[test]
    fn download_redirects_cannot_carry_credentials_or_target_arbitrary_hosts() {
        assert!(allowed_object(&Url::parse("https://skillhub-1388575217.cos.accelerate.myqcloud.com/a.zip").unwrap()));
        for url in ["http://a.cos.accelerate.myqcloud.com/x", "https://evil.example/x", "https://127.0.0.1/x",
            "https://a.cos.accelerate.myqcloud.com.evil.example/x", "https://user:password@a.cos.accelerate.myqcloud.com/x"] {
            assert!(!allowed_object(&Url::parse(url).unwrap()));
        }
    }
    #[test]
    fn skips_inactive_categories_and_unsafe_image_urls() {
        let result = categories(&json!({"items":[{"key":"office-efficiency","name":"办公","active":true,"level":1},
            {"key":"inactive","name":"旧","active":false,"level":1}]})).unwrap();
        assert_eq!(result["results"].as_array().unwrap().len(), 1);
        assert!(entry(&json!({"slug":"test-skill","iconUrl":"javascript:alert(1)"}))["iconUrl"].is_null());
    }
}

//! 上游目录适配器。`fetch_page` 只读上游并规范化，不写库；入库由 `importer` 完成。
pub mod generic;
pub mod skillhub;

use crate::package::sha256_hex;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::Json;
use std::{net::IpAddr, time::Duration};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SourceRow {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub kind: String,
    pub adapter: String,
    pub enabled: bool,
    pub config: Json<Value>,
    pub interval_minutes: i32,
    pub auto_import: String,
    pub last_job_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Params {
    pub keyword: String,
    pub category: String,
    pub source: String,
    pub sort_by: String,
    pub max_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Normalized {
    pub upstream_id: String,
    pub kind: String,
    pub namespace: String,
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub icon_url: Option<String>,
    pub license: Option<String>,
    pub origin_url: Option<String>,
    /// official / community / enterprise
    pub source: String,
    pub version: Option<String>,
    pub metadata: Value,
    /// 下载量、收藏量等易变数据，不参与内容哈希。
    #[serde(default)]
    pub stats: Value,
}

impl Normalized {
    pub fn content_hash(&self) -> String {
        let mut v = serde_json::to_value(self).unwrap_or(Value::Null);
        if let Some(o) = v.as_object_mut() {
            o.remove("stats");
        }
        sha256_hex(v.to_string().as_bytes())
    }

    pub fn search_text(&self) -> String {
        format!("{} {} {} {} {}", self.name, self.slug, self.summary, self.tags.join(" "), self.category.clone().unwrap_or_default())
            .to_lowercase()
    }
}

pub struct RawEntry {
    pub raw: Value,
    pub normalized: Normalized,
}

pub struct Page {
    pub entries: Vec<RawEntry>,
    pub total: Option<u64>,
    pub has_more: bool,
    /// 因标识非法等原因被跳过的条目数。
    pub skipped: usize,
    /// 上游整页响应，仅「试采」展示。
    pub response: Value,
}

#[derive(Debug)]
pub struct UpstreamError {
    pub message: String,
    pub transient: bool,
    pub retry_after: Option<u64>,
}

impl UpstreamError {
    pub fn permanent(message: impl Into<String>) -> Self {
        Self { message: message.into(), transient: false, retry_after: None }
    }
    pub fn transient(message: impl Into<String>) -> Self {
        Self { message: message.into(), transient: true, retry_after: None }
    }
}

impl From<String> for UpstreamError {
    fn from(message: String) -> Self {
        Self::permanent(message)
    }
}

pub type UpstreamResult<T> = Result<T, UpstreamError>;

/// 适配器运行所需的共享客户端与策略。
#[derive(Clone)]
pub struct Ctx {
    /// 不跟随重定向，用于 JSON 接口。
    pub http: reqwest::Client,
    /// 仅跟随到 HTTPS 的腾讯 COS 域名，用于下载安装包。
    pub objects: reqwest::Client,
    pub allow_private: bool,
}

pub fn build_clients() -> Result<(reqwest::Client, reqwest::Client), reqwest::Error> {
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Xiaoban-Market-Crawler/1.0")
        .build()?;
    let objects = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() < 3 && allowed_object(attempt.url()) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .user_agent("Xiaoban-Market-Crawler/1.0")
        .build()?;
    Ok((http, objects))
}

pub fn allowed_object(url: &Url) -> bool {
    url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && url.host_str().is_some_and(|h| h.contains(".cos.") && h.ends_with(".myqcloud.com"))
}

/// 管理员配置的上游地址：必须是 HTTPS，且不能指向回环 / 内网（测试可放开）。
pub fn validate_url(raw: &str, allow_private: bool) -> Result<Url, String> {
    let url = Url::parse(raw).map_err(|_| format!("无效的上游地址：{raw}"))?;
    if allow_private {
        return Ok(url);
    }
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err("上游地址必须是不带账号的 HTTPS 地址".into());
    }
    let host = url.host_str().ok_or("上游地址缺少主机名")?;
    let lower = host.to_ascii_lowercase();
    if lower == "localhost" || lower.ends_with(".local") || lower.ends_with(".internal") {
        return Err("上游地址不能指向本机或内网".into());
    }
    if let Ok(ip) = host.trim_matches(|c| c == '[' || c == ']').parse::<IpAddr>() {
        let private = match ip {
            IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local() || v.is_unspecified(),
            IpAddr::V6(v) => v.is_loopback() || v.is_unspecified() || (v.segments()[0] & 0xfe00) == 0xfc00,
        };
        if private {
            return Err("上游地址不能指向本机或内网".into());
        }
    }
    Ok(url)
}

/// 把 reqwest 错误展开成「外层: 内层」的因果链，方便管理员判断是 DNS、证书还是连接被拒。
fn describe(e: reqwest::Error) -> String {
    use std::error::Error;
    let e = e.without_url();
    let mut parts = vec![e.to_string()];
    let mut source = e.source();
    while let Some(inner) = source {
        let text = inner.to_string();
        if !parts.contains(&text) {
            parts.push(text);
        }
        source = inner.source();
    }
    parts.join(": ")
}

const JSON_LIMIT: usize = 4 * 1024 * 1024;

async fn read_limited(mut resp: reqwest::Response, max: usize) -> UpstreamResult<Vec<u8>> {
    if resp.content_length().is_some_and(|n| n > max as u64) {
        return Err(UpstreamError::permanent("上游响应过大"));
    }
    let mut out = Vec::new();
    while let Some(chunk) = resp.chunk().await.map_err(|e| UpstreamError::transient(format!("读取上游响应失败：{e}")))? {
        out.extend_from_slice(&chunk);
        if out.len() > max {
            return Err(UpstreamError::permanent("上游响应过大"));
        }
    }
    Ok(out)
}

fn classify(resp: &reqwest::Response) -> Option<UpstreamError> {
    let status = resp.status();
    if status.is_success() {
        return None;
    }
    let mut err = if status.as_u16() == 429 || status.is_server_error() {
        UpstreamError::transient(format!("上游暂时不可用（HTTP {status}）"))
    } else {
        UpstreamError::permanent(format!("上游返回 HTTP {status}"))
    };
    err.retry_after = resp.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok());
    Some(err)
}

pub async fn send_json(req: reqwest::RequestBuilder) -> UpstreamResult<Value> {
    let resp = req
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| UpstreamError::transient(format!("无法连接上游：{}", describe(e))))?;
    if let Some(err) = classify(&resp) {
        return Err(err);
    }
    let body = read_limited(resp, JSON_LIMIT).await?;
    serde_json::from_slice(&body).map_err(|_| UpstreamError::permanent("上游返回的不是有效 JSON"))
}

/// 下载安装包：API 域名可带密钥；若被重定向，只跟随到腾讯 COS 的 HTTPS 地址，且不再携带密钥。
pub async fn fetch_object(ctx: &Ctx, url: Url, api_key: Option<&str>, max: usize) -> UpstreamResult<Vec<u8>> {
    let mut req = ctx.http.get(url).timeout(Duration::from_secs(60));
    if let Some(key) = api_key {
        req = req.header("X-API-Key", key);
    }
    let resp = req.send().await.map_err(|e| UpstreamError::transient(format!("无法连接上游：{}", describe(e))))?;
    if resp.status().is_redirection() {
        let location = resp
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Url::parse(v).ok())
            .filter(allowed_object)
            .ok_or_else(|| UpstreamError::permanent("上游把下载重定向到了不被允许的地址"))?;
        return send_bytes(ctx.objects.get(location), max).await;
    }
    if let Some(err) = classify(&resp) {
        return Err(err);
    }
    read_limited(resp, max).await
}

pub async fn send_bytes(req: reqwest::RequestBuilder, max: usize) -> UpstreamResult<Vec<u8>> {
    let resp = req
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| UpstreamError::transient(format!("无法连接上游：{}", describe(e))))?;
    if let Some(err) = classify(&resp) {
        return Err(err);
    }
    read_limited(resp, max).await
}

/// 取字符串，空白视为缺失。
pub fn text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

pub fn first_text(v: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| text(&v[*k]))
}

pub fn tags_of(v: &Value) -> Vec<String> {
    let items: Vec<String> = match v {
        Value::Array(a) => a
            .iter()
            .filter_map(|x| text(x).or_else(|| first_text(x, &["name", "label", "title"])))
            .collect(),
        Value::String(s) => s.split([',', '，', ';']).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect(),
        _ => vec![],
    };
    let mut out: Vec<String> = Vec::new();
    for t in items {
        let t: String = t.chars().take(32).collect();
        if !out.contains(&t) {
            out.push(t);
        }
        if out.len() >= 12 {
            break;
        }
    }
    out
}

pub fn normalize_source_label(raw: Option<&str>) -> String {
    let lower = raw.unwrap_or("").to_lowercase();
    if lower.contains("official") || lower.contains("官方") {
        "official".into()
    } else if lower.contains("enterprise") || lower.contains("企业") {
        "enterprise".into()
    } else {
        "community".into()
    }
}

/// 把任意名称变成合法 slug：小写字母、数字、连字符，长度 2–64。纯中文名用来源 ID 的哈希兜底。
pub fn slugify(input: &str, fallback_seed: &str) -> String {
    let mut out = String::new();
    for c in input.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let mut out = out.trim_matches('-').to_string();
    out.truncate(64);
    let out = out.trim_matches('-').to_string();
    if out.len() < 2 {
        return format!("x-{}", &sha256_hex(fallback_seed.as_bytes())[..10]);
    }
    out
}

pub fn valid_namespace(s: &str) -> bool {
    !s.is_empty() && s.len() <= 80 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn valid_slug(s: &str) -> bool {
    s.len() > 1 && s.len() <= 64 && !s.starts_with('-') && !s.ends_with('-')
        && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// 抓取一页并规范化。
pub async fn fetch_page(ctx: &Ctx, source: &SourceRow, params: &Params, page: u32) -> UpstreamResult<Page> {
    match source.adapter.as_str() {
        "skillhub" => skillhub::fetch_page(ctx, source, params, page).await,
        "generic_json" => generic::fetch_page(ctx, source, params, page).await,
        other => Err(UpstreamError::permanent(format!("未知适配器：{other}"))),
    }
}

/// 入库时才需要的二次请求结果（安装包、专家正文、连接器 README）。
pub enum Enrichment {
    None,
    Package { version: String, changelog: String, bytes: Vec<u8> },
    /// SkillHub 专家：工作流正文、依赖清单、上游详情，以及已经下载并校验过的子技能。
    /// `missing` 是没能带上的子技能（下架、下载失败或超出大小上限）。
    Expert { content: String, dependencies: Value, detail: Value, children: Vec<crate::convert::Child>, missing: Vec<String> },
    Connector { readme: String, configs: Value },
}

pub async fn enrich(ctx: &Ctx, source: &SourceRow, n: &Normalized) -> UpstreamResult<Enrichment> {
    match source.adapter.as_str() {
        "skillhub" => skillhub::enrich(ctx, source, n).await,
        _ => Ok(Enrichment::None),
    }
}

pub fn masked_config(config: &Value) -> Value {
    let mut c = config.clone();
    if let Some(o) = c.as_object_mut() {
        if o.get("api_key").is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty())) {
            o.insert("api_key".into(), json!("***"));
        }
        if let Some(h) = o.get_mut("headers").and_then(|h| h.as_object_mut()) {
            for (k, v) in h.iter_mut() {
                let k = k.to_lowercase();
                if k.contains("authorization") || k.contains("token") || k.contains("key") || k.contains("cookie") {
                    *v = json!("***");
                }
            }
        }
    }
    c
}

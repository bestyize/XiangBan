//! 联网搜索与网页抓取：`/v1/search`、`/v1/fetch`。
//!
//! 对外格式以 DeepSeek 的 Search / Fetch 为准（请求 `query`、`count`、`need_content`、`ori_question`；
//! 结果 `results[]` 里的 `title`、`url`、`snippet`、`summary`、`date`、`content`、`rerank_score`、`authority`），
//! 客户端只认这一种。后面按配置顺序接多家服务商：前一家失败（超时、5xx、限流、返回格式不对）就换下一家。
//! 格式与 DeepSeek 不同的服务商，在 [`Provider`] 里加一个分支，把它的请求和结果改写成这个格式。
//! 密钥只在服务端，响应里不出现服务商名字。

use crate::{config::AppConfig, error::AppError};
use reqwest::{StatusCode, Url};
use serde::Serialize;
use serde_json::{json, Value};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::Duration};

/// DeepSeek 的查询上限，也是我们的。
pub const MAX_QUERY_CHARS: usize = 128;
pub const DEFAULT_COUNT: u64 = 10;
pub const MAX_COUNT: u64 = 50;
pub const MAX_QUESTION_CHARS: usize = 500;
pub const MAX_URL_CHARS: usize = 8192;
/// 一页正文最多返回这么多字；更长的截断并标记 `truncated`。
pub const MAX_PAGE_CHARS: usize = 60_000;
/// 搜索结果里附带正文时，每条最多这么多字。
pub const MAX_HIT_CONTENT_CHARS: usize = 8_000;
const SEARCH_TIMEOUT: Duration = Duration::from_secs(20);
const FETCH_TIMEOUT: Duration = Duration::from_secs(40);

/// `POST /v1/search` 的请求体，字段同 DeepSeek。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchRequest {
    /// 1–128 字，可以带 `site:` 等搜索语法。
    pub query: String,
    /// 1–50，默认 10。
    pub count: u64,
    /// 结果里附带网页正文，省一次抓取。
    pub need_content: bool,
    /// 用户原本的问题，用来给结果排序。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ori_question: Option<String>,
}

/// 一条结果，字段同 DeepSeek。没有的字段不出现。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchHit {
    pub title: String,
    pub url: String,
    pub snippet: String,
    /// 比 snippet 长、给模型读的摘要。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 发布时间（多为 `YYYY-MM-DD`）；不知道时为 null。
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// 与查询的相关度，0–3，1.5 及以上算高。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f64>,
    /// 来源权威度，0–4，3 及以上算权威。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<f64>,
}

/// `POST /v1/fetch` 的结果：网页标题与 Markdown 正文。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FetchedPage {
    pub url: String,
    pub title: String,
    pub content: String,
    pub truncated: bool,
}

fn json_body(body: &[u8]) -> Result<Value, AppError> {
    serde_json::from_slice(body).map_err(|_| AppError::BadRequest("请求体不是合法的 JSON".into()))
}

pub fn parse_search(body: &[u8]) -> Result<SearchRequest, AppError> {
    let value = json_body(body)?;
    let query = value.get("query").and_then(Value::as_str).map(str::trim).unwrap_or_default();
    if query.is_empty() || query.chars().count() > MAX_QUERY_CHARS {
        return Err(AppError::BadRequest(format!("query 需要 1 到 {MAX_QUERY_CHARS} 个字")));
    }
    let count = match value.get("count") {
        None | Some(Value::Null) => DEFAULT_COUNT,
        Some(v) => v.as_u64().filter(|n| (1..=MAX_COUNT).contains(n)).ok_or_else(|| AppError::BadRequest(format!("count 需要 1 到 {MAX_COUNT}")))?,
    };
    let need_content = match value.get("need_content") {
        None | Some(Value::Null) => false,
        Some(v) => v.as_bool().ok_or_else(|| AppError::BadRequest("need_content 需要是 true 或 false".into()))?,
    };
    let ori_question = value.get("ori_question").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
        .map(|s| s.chars().take(MAX_QUESTION_CHARS).collect());
    Ok(SearchRequest { query: query.to_string(), count, need_content, ori_question })
}

pub fn parse_fetch(body: &[u8]) -> Result<Url, AppError> {
    let value = json_body(body)?;
    let raw = value.get("url").and_then(Value::as_str).map(str::trim).unwrap_or_default();
    if raw.is_empty() || raw.chars().count() > MAX_URL_CHARS {
        return Err(AppError::BadRequest(format!("url 需要 1 到 {MAX_URL_CHARS} 个字符")));
    }
    let url = Url::parse(raw).map_err(|_| AppError::BadRequest("url 不是合法的网址".into()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::BadRequest("只能抓取 http 或 https 的公开网页，网址里不能带账号密码".into()));
    }
    Ok(url)
}

fn clip(text: &str, max: usize) -> (String, bool) {
    match text.char_indices().nth(max) {
        Some((at, _)) => (text[..at].to_string(), true),
        None => (text.to_string(), false),
    }
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// 把 DeepSeek 格式的结果规整一遍：去掉没有 http(s) 网址的条目、截短正文、只留约定的字段。
/// 格式相同的服务商可以直接用它；其他服务商先改写成 DeepSeek 格式再交给它。
pub fn normalize_hits(value: &Value, count: u64) -> Option<Vec<SearchHit>> {
    let list = value.get("results")?.as_array()?;
    Some(list.iter().filter_map(|item| {
        let url = text(item, "url")?;
        if !url.starts_with("http://") && !url.starts_with("https://") { return None; }
        Some(SearchHit {
            title: text(item, "title").unwrap_or_else(|| url.clone()),
            snippet: text(item, "snippet").unwrap_or_default(),
            summary: text(item, "summary"),
            date: text(item, "date"),
            content: text(item, "content").map(|c| clip(&c, MAX_HIT_CONTENT_CHARS).0),
            rerank_score: item.get("rerank_score").and_then(Value::as_f64),
            authority: item.get("authority").and_then(Value::as_f64),
            url,
        })
    }).take(count as usize).collect())
}

pub fn normalize_page(value: &Value, url: &Url) -> Option<FetchedPage> {
    let body = text(value, "content")?;
    let (content, truncated) = clip(&body, MAX_PAGE_CHARS);
    Some(FetchedPage { url: text(value, "url").unwrap_or_else(|| url.to_string()), title: text(value, "title").unwrap_or_default(), content, truncated })
}

/// 一家服务商这次没帮上忙。`retry` 为真时换下一家；为假说明问题在请求本身，换谁都一样。
#[derive(Debug)]
pub struct ProviderError {
    pub retry: bool,
    /// The caller's quota ran out (New API said 402): not worth trying anyone else.
    pub out_of_quota: bool,
    pub message: String,
}

impl ProviderError {
    fn next(message: impl Into<String>) -> Self { Self { retry: true, out_of_quota: false, message: message.into() } }
    fn status(status: StatusCode, body: &str) -> Self {
        let message = format!("{status}: {}", body.chars().take(300).collect::<String>());
        // 400/422 是查询本身不被接受；401/403/429/5xx 是我们这边或对方的问题，换一家可能就好。
        let code = status.as_u16();
        Self { retry: !matches!(code, 400 | 402 | 422), out_of_quota: code == 402, message }
    }
}

#[derive(Debug, Clone)]
pub enum Provider {
    /// `POST {base}/v1/tools/search`、`POST {base}/v1/tools/fetch`，格式即我们的格式。
    DeepSeek { base: String, key: String },
    /// 同样的两个接口经 New API 转发（它按 `deepseek-search` / `deepseek-fetch` 两个模型名选渠道、按次计费），
    /// 用调用者自己的 New API 令牌，所以用量记在 New API 里，网关不再另扣。
    NewApi { base: String },
}

impl Provider {
    pub fn name(&self) -> &'static str {
        match self { Provider::DeepSeek { .. } => "deepseek", Provider::NewApi { .. } => "newapi" }
    }

    /// New API bills these calls itself; the gateway charges only for providers it pays directly.
    pub fn billed_upstream(&self) -> bool { matches!(self, Provider::NewApi { .. }) }

    /// Where to send it and with which key. A New API call needs the caller's own key.
    fn target(&self, caller_key: Option<&str>) -> Result<(&str, String), ProviderError> {
        match self {
            Provider::DeepSeek { base, key } => Ok((base, key.clone())),
            Provider::NewApi { base } => caller_key.map(|key| (base.as_str(), format!("sk-{key}"))).ok_or_else(|| ProviderError::next("没有调用者的 New API 令牌")),
        }
    }

    async fn post(client: &reqwest::Client, url: String, key: &str, body: &impl Serialize, timeout: Duration) -> Result<Value, ProviderError> {
        let response = client.post(url).bearer_auth(key).json(body).timeout(timeout).send().await.map_err(|e| ProviderError::next(e.to_string()))?;
        let status = response.status();
        let text = response.text().await.map_err(|e| ProviderError::next(e.to_string()))?;
        if !status.is_success() { return Err(ProviderError::status(status, &text)); }
        serde_json::from_str(&text).map_err(|_| ProviderError::next("返回的不是 JSON"))
    }

    async fn search(&self, client: &reqwest::Client, request: &SearchRequest, caller_key: Option<&str>) -> Result<Vec<SearchHit>, ProviderError> {
        // Both speak DeepSeek's format; only where and with which key differ.
        let (base, key) = self.target(caller_key)?;
        let value = Self::post(client, format!("{base}/v1/tools/search"), &key, request, SEARCH_TIMEOUT).await?;
        normalize_hits(&value, request.count).ok_or_else(|| ProviderError::next("返回里没有 results"))
    }

    async fn fetch(&self, client: &reqwest::Client, url: &Url, caller_key: Option<&str>) -> Result<FetchedPage, ProviderError> {
        let (base, key) = self.target(caller_key)?;
        let value = Self::post(client, format!("{base}/v1/tools/fetch"), &key, &json!({ "url": url.as_str() }), FETCH_TIMEOUT).await?;
        normalize_page(&value, url).ok_or_else(|| ProviderError::next("返回里没有网页正文"))
    }
}

/// 同一家服务商的一组密钥。每次请求从下一个密钥开始，把流量分摊开（DeepSeek 每个密钥每秒限 5 次）；
/// 某个密钥失效、限流或超时，就换同组的下一个，整组都不行再换下一家。
struct Group {
    members: Vec<Provider>,
    next: AtomicUsize,
}

impl Group {
    /// 这一轮尝试密钥的顺序。
    fn order(&self) -> impl Iterator<Item = &Provider> {
        let len = self.members.len();
        let start = self.next.fetch_add(1, Ordering::Relaxed) % len.max(1);
        (0..len).map(move |i| &self.members[(start + i) % len])
    }
}

/// 配置好的服务商，按顺序尝试。
pub struct Search {
    client: reqwest::Client,
    groups: Vec<Group>,
}

impl Search {
    /// 每个服务商一组，各一个密钥。
    pub fn new(providers: Vec<Provider>) -> Self {
        Self::grouped(providers.into_iter().map(|p| vec![p]).collect())
    }

    /// 每组是同一家服务商的多个密钥，组按顺序尝试。空组忽略。
    pub fn grouped(groups: Vec<Vec<Provider>>) -> Self {
        let groups = groups.into_iter().filter(|g| !g.is_empty()).map(|members| Group { members, next: AtomicUsize::new(0) }).collect();
        Self { client: reqwest::Client::new(), groups }
    }

    /// `AI_SEARCH_PROVIDERS` 按顺序列出服务商；每家的密钥可以有多个（逗号分隔），没配密钥的那家跳过。
    /// `newapi` 不需要密钥：用调用者自己的 New API 令牌。
    pub fn from_config(config: &AppConfig) -> Self {
        let groups = config.search_providers.iter().map(|name| match name.as_str() {
            "newapi" => vec![Provider::NewApi { base: config.newapi_base_url.clone() }],
            "deepseek" => config.deepseek_api_keys.iter().map(|key| Provider::DeepSeek { base: config.deepseek_base_url.clone(), key: key.clone() }).collect(),
            other => { tracing::warn!("未知的搜索服务商 {other}，已忽略"); Vec::new() }
        }).collect();
        Self::grouped(groups)
    }

    pub fn available(&self) -> bool { !self.groups.is_empty() }

    /// The results, and whether the gateway should charge for them (false when New API already did).
    /// `caller_key` is the caller's New API key, for the `newapi` provider.
    pub async fn search(&self, request: &SearchRequest, caller_key: Option<&str>) -> Result<(Vec<SearchHit>, bool), AppError> {
        let mut tried = false;
        for provider in self.groups.iter().flat_map(Group::order) {
            match provider.search(&self.client, request, caller_key).await {
                Ok(hits) => return Ok((hits, !provider.billed_upstream())),
                Err(error) => {
                    tracing::warn!("搜索服务商 {} 失败：{}", provider.name(), error.message);
                    if error.out_of_quota { return Err(out_of_quota()); }
                    if !error.retry { return Err(AppError::BadRequest("这个搜索词没法查询，请换个说法".into())); }
                    tried = true;
                }
            }
        }
        Err(unavailable(tried))
    }

    pub async fn fetch(&self, url: &Url, caller_key: Option<&str>) -> Result<(FetchedPage, bool), AppError> {
        let mut tried = false;
        for provider in self.groups.iter().flat_map(Group::order) {
            match provider.fetch(&self.client, url, caller_key).await {
                Ok(page) => return Ok((page, !provider.billed_upstream())),
                Err(error) => {
                    tracing::warn!("抓取服务商 {} 失败：{}", provider.name(), error.message);
                    if error.out_of_quota { return Err(out_of_quota()); }
                    if !error.retry { return Err(AppError::BadRequest("这个网页没法抓取".into())); }
                    tried = true;
                }
            }
        }
        Err(unavailable(tried))
    }
}

fn out_of_quota() -> AppError {
    AppError::Coded { status: StatusCode::PAYMENT_REQUIRED, code: "insufficient_quota", message: "额度暂时不足，若刚兑换过请稍候几秒再试".into() }
}

fn unavailable(tried: bool) -> AppError {
    if tried { return AppError::BadGateway("联网搜索暂时不可用，请稍后再试".into()); }
    AppError::Coded { status: StatusCode::SERVICE_UNAVAILABLE, code: "search_unavailable", message: "小伴 AI 联网搜索暂未开放".into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::post, Json, Router};
    use std::sync::Arc;

    #[test]
    fn requests_use_deepseek_fields_and_limits() {
        let request = parse_search(r#"{"query":" site:gov.cn 个税 ","count":5,"need_content":true,"ori_question":"今年个税怎么算"}"#.as_bytes()).unwrap();
        assert_eq!(request, SearchRequest { query: "site:gov.cn 个税".into(), count: 5, need_content: true, ori_question: Some("今年个税怎么算".into()) });
        assert_eq!(parse_search(r#"{"query":"天气"}"#.as_bytes()).unwrap().count, DEFAULT_COUNT);
        assert!(parse_search(r#"{"query":""}"#.as_bytes()).is_err());
        assert!(parse_search(format!(r#"{{"query":"{}"}}"#, "字".repeat(129)).as_bytes()).is_err());
        assert!(parse_search(r#"{"query":"a","count":51}"#.as_bytes()).is_err());
        assert!(parse_search(r#"{"query":"a","need_content":"yes"}"#.as_bytes()).is_err());
        // Forwarded as is: DeepSeek reads the same body.
        assert_eq!(serde_json::to_value(parse_search(r#"{"query":"a"}"#.as_bytes()).unwrap()).unwrap(), json!({ "query": "a", "count": 10, "need_content": false }));
    }

    #[test]
    fn fetch_takes_public_web_addresses_only() {
        assert_eq!(parse_fetch(r#"{"url":"https://example.com/a?b=1"}"#.as_bytes()).unwrap().as_str(), "https://example.com/a?b=1");
        for bad in [r#"{"url":"file:///etc/passwd"}"#, r#"{"url":"https://user:pw@example.com"}"#, r#"{"url":"not a url"}"#, r#"{}"#] {
            assert!(parse_fetch(bad.as_bytes()).is_err(), "{bad}");
        }
    }

    #[test]
    fn results_keep_the_agreed_fields_and_drop_unusable_ones() {
        let value = json!({ "results": [
            { "title": "个税计算", "url": "https://www.gov.cn/a", "snippet": "摘要", "summary": "长摘要", "date": "2026-10-01", "rerank_score": 2.1, "authority": 4, "extra": "x" },
            { "title": "没有网址" },
            { "title": "不是网页", "url": "javascript:alert(1)" },
            { "url": "https://b.example/", "content": "正".repeat(MAX_HIT_CONTENT_CHARS + 10), "date": null },
        ]});
        let hits = normalize_hits(&value, 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(serde_json::to_value(&hits[0]).unwrap(), json!({ "title": "个税计算", "url": "https://www.gov.cn/a", "snippet": "摘要", "summary": "长摘要", "date": "2026-10-01", "rerank_score": 2.1, "authority": 4.0 }));
        assert_eq!(hits[1].title, "https://b.example/");
        assert_eq!(hits[1].content.as_ref().unwrap().chars().count(), MAX_HIT_CONTENT_CHARS);
        assert_eq!(normalize_hits(&value, 1).unwrap().len(), 1);
        assert_eq!(normalize_hits(&json!({ "results": [] }), 5), Some(vec![]));
        assert_eq!(normalize_hits(&json!({ "error": "x" }), 5), None);
    }

    #[test]
    fn long_pages_are_cut_and_say_so() {
        let url = Url::parse("https://example.com/").unwrap();
        let page = normalize_page(&json!({ "title": "T", "content": "字".repeat(MAX_PAGE_CHARS + 1) }), &url).unwrap();
        assert!(page.truncated);
        assert_eq!(page.content.chars().count(), MAX_PAGE_CHARS);
        assert_eq!(page.url, "https://example.com/");
        assert!(normalize_page(&json!({ "title": "空的" }), &url).is_none());
    }

    async fn upstream(status: u16, reply: Value, hits: Arc<AtomicUsize>, seen: Arc<std::sync::Mutex<Vec<Value>>>) -> String {
        let handler = move |Json(body): Json<Value>| {
            let (reply, hits, seen) = (reply.clone(), hits.clone(), seen.clone());
            async move {
                hits.fetch_add(1, Ordering::SeqCst);
                seen.lock().unwrap().push(body);
                (StatusCode::from_u16(status).unwrap(), Json(reply))
            }
        };
        let app = Router::new().route("/v1/tools/search", post(handler.clone())).route("/v1/tools/fetch", post(handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn a_failing_provider_hands_over_to_the_next_one() {
        let (down, up) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let first = upstream(503, json!({ "error": "busy" }), down.clone(), seen.clone()).await;
        let second = upstream(200, json!({ "results": [{ "title": "A", "url": "https://a.example/", "snippet": "s" }] }), up.clone(), seen.clone()).await;
        let search = Search::new(vec![Provider::DeepSeek { base: first, key: "k1".into() }, Provider::DeepSeek { base: second, key: "k2".into() }]);
        let request = parse_search(r#"{"query":"天气","count":3}"#.as_bytes()).unwrap();
        let (hits, ours) = search.search(&request, None).await.unwrap();
        assert!(ours, "a direct provider is charged by the gateway");
        assert_eq!(hits.len(), 1);
        assert_eq!((down.load(Ordering::SeqCst), up.load(Ordering::SeqCst)), (1, 1));
        assert_eq!(seen.lock().unwrap()[1], json!({ "query": "天气", "count": 3, "need_content": false }));
    }

    #[tokio::test]
    async fn a_rejected_query_is_not_retried_elsewhere_and_no_provider_means_unavailable() {
        let (first_hits, second_hits) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let first = upstream(400, json!({ "error": "bad query" }), first_hits.clone(), seen.clone()).await;
        let second = upstream(200, json!({ "results": [] }), second_hits.clone(), seen.clone()).await;
        let search = Search::new(vec![Provider::DeepSeek { base: first, key: "k".into() }, Provider::DeepSeek { base: second, key: "k".into() }]);
        let request = parse_search(r#"{"query":"x"}"#.as_bytes()).unwrap();
        assert!(matches!(search.search(&request, None).await, Err(AppError::BadRequest(_))));
        assert_eq!(second_hits.load(Ordering::SeqCst), 0);
        let none = Search::new(vec![]);
        assert!(!none.available());
        assert!(matches!(none.search(&request, None).await, Err(AppError::Coded { code: "search_unavailable", .. })));
    }

    #[tokio::test]
    async fn several_keys_take_turns_and_a_bad_key_hands_over_to_the_next() {
        let hits = Arc::new(AtomicUsize::new(0));
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let base = upstream(200, json!({ "results": [{ "title": "A", "url": "https://a.example/", "snippet": "s" }] }), hits.clone(), seen.clone()).await;
        let dead = upstream(401, json!({ "error": "invalid key" }), Arc::new(AtomicUsize::new(0)), seen.clone()).await;
        let keys = |k: &str, b: &str| Provider::DeepSeek { base: b.to_string(), key: k.into() };
        let search = Search::grouped(vec![vec![keys("k1", &base), keys("k2", &dead), keys("k3", &base)]]);
        let request = parse_search(r#"{"query":"x"}"#.as_bytes()).unwrap();
        // Round one starts at k1, round two at k2 (dead, so k3 answers), round three at k3.
        for _ in 0..3 { assert_eq!(search.search(&request, None).await.unwrap().0.len(), 1); }
        assert_eq!(hits.load(Ordering::SeqCst), 3, "every round was answered by a working key");
        assert_eq!(seen.lock().unwrap().len(), 4, "the dead key was tried once and passed over");
    }

    #[tokio::test]
    async fn fetch_returns_the_page_as_markdown() {
        let hits = Arc::new(AtomicUsize::new(0));
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let base = upstream(200, json!({ "title": "标题", "content": "# 正文" }), hits, seen.clone()).await;
        let search = Search::new(vec![Provider::DeepSeek { base, key: "k".into() }]);
        let (page, _) = search.fetch(&Url::parse("https://example.com/x").unwrap(), None).await.unwrap();
        assert_eq!((page.title.as_str(), page.content.as_str(), page.truncated), ("标题", "# 正文", false));
        assert_eq!(seen.lock().unwrap()[0], json!({ "url": "https://example.com/x" }));
    }
}

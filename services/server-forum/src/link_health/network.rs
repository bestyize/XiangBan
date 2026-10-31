//! Metadata-only link probes. All production DNS results must be public and are pinned into a
//! fresh credential-free, proxy-free client for each hop; the HTTP client never resolves again.
//! reqwest 0.12.28 ClientBuilder::resolve_to_addrs is used with redirects disabled.
use reqwest::{Client, Method, Url};
use serde::Serialize;
use std::{
    future::Future,
    net::{IpAddr, SocketAddr},
    time::{Duration, Instant},
};

pub const LINK_TIMEOUT: Duration = Duration::from_secs(10);
const HOSTS: &[&str] = &[
    "openai.com",
    "www.openai.com",
    "platform.openai.com",
    "developers.openai.com",
    "chatgpt.com",
    "anthropic.com",
    "www.anthropic.com",
    "claude.ai",
    "docs.claude.com",
    "docs.anthropic.com",
    "platform.claude.com",
    "console.anthropic.com",
    "deepmind.google",
    "research.google",
    "ai.google.dev",
    "aistudio.google.com",
    "gemini.google.com",
    "cloud.google.com",
    "blog.google",
    "github.com",
    "raw.githubusercontent.com",
    "artificialanalysis.ai",
    "openrouter.ai",
    "huggingface.co",
    "modelscope.cn",
    "www.modelscope.cn",
    "chat.deepseek.com",
    "platform.deepseek.com",
    "api.deepseek.com",
    "deepseek.com",
    "www.deepseek.com",
    "tongyi.aliyun.com",
    "help.aliyun.com",
    "www.doubao.com",
    "chatglm.cn",
    "www.kimi.com",
    "kimi.com",
    "moonshot.ai",
    "platform.moonshot.cn",
    "chat.xiangban.xyz",
];
#[derive(Clone, Debug, Serialize)]
pub struct Outcome {
    pub status: String,
    pub http_status: Option<i32>,
    pub final_url: Option<String>,
    pub error: Option<String>,
    pub duration_ms: i64,
}
impl Outcome {
    pub fn failure(status: &str, message: &str) -> Self {
        Self {
            status: status.into(),
            http_status: None,
            final_url: None,
            error: Some(message.into()),
            duration_ms: 0,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Headers {
    pub status: u16,
    pub location: Option<String>,
}
#[derive(Clone, Copy, Debug)]
pub enum Failure {
    Dns,
    PrivateDns,
    Network,
    Peer,
    Redirect,
}
impl Failure {
    fn result(self) -> Outcome {
        match self {
            Self::Dns => Outcome::failure("dns_error", "域名解析失败或超时"),
            Self::PrivateDns => Outcome::failure("blocked", "DNS 包含非公网地址，已阻止访问"),
            Self::Network => Outcome::failure("network_error", "连接、TLS 或响应超时，需人工复核"),
            Self::Peer => Outcome::failure("blocked", "连接地址与已校验 DNS 不一致"),
            Self::Redirect => Outcome::failure("blocked", "重定向地址无效或超过限制"),
        }
    }
}
pub fn normalized(raw: &str) -> Result<Url, &'static str> {
    if raw.len() > 2048 || raw.chars().any(char::is_control) {
        return Err("链接格式无效或过长");
    }
    let mut url = Url::parse(raw.trim()).map_err(|_| "链接格式无效")?;
    if url.scheme() != "https"
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("仅允许无凭据的 HTTPS 默认端口链接");
    }
    let host = url.host_str().ok_or("链接缺少主机")?;
    if !HOSTS.contains(&host) {
        return Err("来源主机不在固定官方或既有来源白名单");
    }
    if url.query_pairs().any(|(key, value)| {
        !matches!(
            key.as_ref(),
            "hl" | "lang"
                | "locale"
                | "tab"
                | "view"
                | "utm_source"
                | "utm_medium"
                | "utm_campaign"
        ) || value.len() > 100
            || !value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }) {
        return Err("查询参数不在安全展示参数白名单，已阻止访问");
    }
    url.set_fragment(None);
    Ok(url)
}
/// Unsafe candidates remain visible for remediation, but credentials are never copied into the
/// health table or API. This value is display-only and must never be used as a probe URL.
pub fn display_url(raw: &str) -> String {
    match Url::parse(raw.trim()) {
        Ok(mut u) if matches!(u.scheme(), "http" | "https") && u.host_str().is_some() => {
            let _ = u.set_username("");
            let _ = u.set_password(None);
            u.set_query(None);
            u.set_fragment(None);
            u.to_string().chars().take(2048).collect()
        }
        _ => "（无效链接）".into(),
    }
}
pub fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let o = ip.octets();
            !(o[0] == 0
                || o[0] == 10
                || o[0] == 127
                || o[0] >= 224
                || (o[0] == 100 && (64..=127).contains(&o[1]))
                || (o[0] == 169 && o[1] == 254)
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 192
                    && ((o[1] == 0 && (o[2] == 0 || o[2] == 2))
                        || (o[1] == 88 && o[2] == 99)
                        || o[1] == 168))
                || (o[0] == 198 && (o[1] == 18 || o[1] == 19 || (o[1] == 51 && o[2] == 100)))
                || (o[0] == 203 && o[1] == 0 && o[2] == 113))
        }
        IpAddr::V6(ip) => {
            let s = ip.segments();
            (s[0] & 0xe000) == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8))
                && s[0] != 0x2002
                && !(s[0] == 0x3fff && (s[1] & 0xf000) == 0)
        }
    }
}
fn valid_addresses(addrs: &[SocketAddr]) -> bool {
    !addrs.is_empty()
        && addrs.len() <= 32
        && addrs.iter().all(|a| a.port() == 443 && public_ip(a.ip()))
}
async fn dns(host: String) -> Result<Vec<SocketAddr>, Failure> {
    let rows = tokio::time::timeout(
        Duration::from_secs(3),
        tokio::net::lookup_host((host.as_str(), 443)),
    )
    .await
    .map_err(|_| Failure::Dns)?
    .map_err(|_| Failure::Dns)?
    .take(33)
    .collect::<Vec<_>>();
    if !valid_addresses(&rows) {
        return Err(Failure::PrivateDns);
    }
    Ok(rows)
}
async fn headers(url: Url, method: Method, addrs: Vec<SocketAddr>) -> Result<Headers, Failure> {
    let host = url.host_str().ok_or(Failure::Network)?.to_string();
    let http = Client::builder()
        .no_proxy()
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .redirect(reqwest::redirect::Policy::none())
        .referer(false)
        .pool_max_idle_per_host(0)
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(4))
        .http1_only()
        .resolve_to_addrs(&host, &addrs)
        .user_agent("XiaobanLinkHealth/1.0")
        .build()
        .map_err(|_| Failure::Network)?;
    let mut request = http
        .request(method.clone(), url)
        .header("accept-encoding", "identity");
    if method == Method::GET {
        request = request.header("range", "bytes=0-0")
    }
    let response = request.send().await.map_err(|_| Failure::Network)?;
    if !response
        .remote_addr()
        .is_some_and(|peer| addrs.contains(&peer))
    {
        return Err(Failure::Peer);
    }
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    if location.as_ref().is_some_and(|v| v.len() > 2048) {
        return Err(Failure::Redirect);
    }
    // Deliberately do not call bytes/text/chunk: even servers ignoring Range cannot make us read
    // an article body. Dropping this non-pooled response closes its connection.
    Ok(Headers {
        status: response.status().as_u16(),
        location,
    })
}
async fn probe_with<R, RF, T, TF>(raw: String, mut resolver: R, mut transport: T) -> Outcome
where
    R: FnMut(String) -> RF,
    RF: Future<Output = Result<Vec<SocketAddr>, Failure>>,
    T: FnMut(Url, Method, Vec<SocketAddr>) -> TF,
    TF: Future<Output = Result<Headers, Failure>>,
{
    let mut url = match normalized(&raw) {
        Ok(u) => u,
        Err(e) => return Outcome::failure("blocked", e),
    };
    for hop in 0..=2 {
        let addresses = match resolver(url.host_str().unwrap().to_string()).await {
            Ok(a) if valid_addresses(&a) => a,
            Ok(_) => return Failure::PrivateDns.result(),
            Err(e) => return e.result(),
        };
        let mut response = match transport(url.clone(), Method::HEAD, addresses.clone()).await {
            Ok(r) => r,
            Err(e) => return e.result(),
        };
        if matches!(response.status, 405 | 501) {
            response = match transport(url.clone(), Method::GET, addresses).await {
                Ok(r) => r,
                Err(e) => return e.result(),
            };
        }
        if matches!(response.status, 301 | 302 | 303 | 307 | 308) {
            if hop == 2 {
                return Failure::Redirect.result();
            }
            let Some(location) = response.location else {
                return Failure::Redirect.result();
            };
            let redirected = match url.join(&location) {
                Ok(u) => u,
                Err(_) => return Failure::Redirect.result(),
            };
            url = match normalized(redirected.as_str()) {
                Ok(u) => u,
                Err(e) => return Outcome::failure("blocked", e),
            };
            continue;
        }
        let (status, error) = match response.status {
            200..=299 => ("healthy", None),
            401 | 403 => ("restricted", Some("来源限制自动访问，请人工打开核对")),
            404 | 410 => ("broken", Some("来源返回不存在或已移除")),
            429 => ("rate_limited", Some("来源限流，本轮不自动重试")),
            500..=599 => ("unavailable", Some("来源暂时不可用")),
            _ => ("attention", Some("返回非预期状态，请人工复核")),
        };
        return Outcome {
            status: status.into(),
            http_status: Some(response.status as i32),
            final_url: Some(url.to_string()),
            error: error.map(str::to_string),
            duration_ms: 0,
        };
    }
    Failure::Redirect.result()
}
pub async fn probe(raw: String) -> Outcome {
    let start = Instant::now();
    let mut result = tokio::time::timeout(LINK_TIMEOUT, probe_with(raw, dns, headers))
        .await
        .unwrap_or_else(|_| Outcome::failure("timeout", "单链接检查超过十秒，本轮停止"));
    result.duration_ms = start.elapsed().as_millis().min(i64::MAX as u128) as i64;
    result
}
#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;

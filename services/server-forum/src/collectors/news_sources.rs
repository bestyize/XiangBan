//! Read-only official RSS/Atom adapters. The caller supplies the isolated collector client with
//! redirects disabled. Only the feed documents below are fetched; article pages/full text are not.
//! Checked 2026-10-31: all three fixed official feeds returned HTTP 200 and parseable RSS.
//! https://openai.com/news/rss.xml
//! https://deepmind.google/blog/rss.xml
//! https://research.google/blog/rss/
//! roxmltree 0.21.2 ParsingOptions: DTD disabled, external entity resolver absent, node count capped.
use super::CollectedItem;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use ring::digest;
use roxmltree::{Document, Node, ParsingOptions};
use serde_json::{json, Value};
use std::{collections::BTreeSet, time::Duration};

const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_ITEMS: usize = 10;
const MAX_NODES: u32 = 60_000;
const MAX_DEPTH: usize = 32;
const ATOM: &str = "http://www.w3.org/2005/Atom";
const TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Copy)]
struct Feed {
    id: &'static str,
    title: &'static str,
    url: &'static str,
    host: &'static str,
    paths: &'static [&'static str],
}
const FEEDS: &[Feed] = &[
    Feed {
        id: "openai_news",
        title: "OpenAI",
        url: "https://openai.com/news/rss.xml",
        host: "openai.com",
        paths: &["/"],
    },
    Feed {
        id: "google_deepmind",
        title: "Google DeepMind",
        url: "https://deepmind.google/blog/rss.xml",
        host: "deepmind.google",
        paths: &["/blog/"],
    },
    Feed {
        id: "google_research",
        title: "Google Research",
        url: "https://research.google/blog/rss/",
        host: "research.google",
        paths: &["/blog/"],
    },
];
fn invalid_config() -> AppError {
    AppError::BadRequest(
        "请选择官方日报来源：openai_news、google_deepmind、google_research；不能填写自定义 URL"
            .into(),
    )
}
/// Empty selection is allowed for an inactive source; enabling it requires at least one feed.
/// Unknown fields are rejected instead of being accepted as a hidden URL override.
pub fn sanitize_config(config: Value) -> AppResult<Value> {
    let obj = config.as_object().ok_or_else(invalid_config)?;
    if obj.keys().any(|k| k != "feed_ids") {
        return Err(invalid_config());
    }
    let ids = match obj.get("feed_ids") {
        Some(ids) => ids.as_array().ok_or_else(invalid_config)?,
        None => return Ok(json!({"feed_ids": []})),
    };
    if ids.len() > FEEDS.len() {
        return Err(invalid_config());
    }
    let mut clean = BTreeSet::new();
    for id in ids {
        let id = id.as_str().ok_or_else(invalid_config)?;
        if !FEEDS.iter().any(|feed| feed.id == id) || !clean.insert(id) {
            return Err(invalid_config());
        }
    }
    Ok(json!({"feed_ids": clean.into_iter().collect::<Vec<_>>()}))
}
fn failure(feed: Feed, category: &str) -> String {
    format!(
        "{}：{}；本次未写入候选，线上旧内容保留",
        feed.title, category
    )
}
async fn fetch_xml(
    http: &Client,
    feed: Feed,
    endpoint: &str,
    timeout: Duration,
) -> Result<Vec<u8>, String> {
    let mut response = http
        .get(endpoint)
        .header(
            reqwest::header::ACCEPT,
            "application/rss+xml, application/atom+xml, application/xml, text/xml",
        )
        .timeout(timeout)
        .send()
        .await
        .map_err(|_| failure(feed, "连接失败或超时"))?;
    if response.url().as_str() != endpoint {
        return Err(failure(feed, "来源发生重定向，已停止采集"));
    }
    let status = response.status();
    if !status.is_success() {
        let category = match status.as_u16() {
            429 => "HTTP 429，来源限流，请稍后重试",
            500..=599 => "HTTP 5xx，来源暂时不可用",
            300..=399 => "来源发生重定向，已停止采集",
            _ => "来源返回非成功状态",
        };
        return Err(failure(feed, category));
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_BODY as u64)
    {
        return Err(failure(feed, "响应体超过 2 MiB 限制"));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| failure(feed, "读取超时或响应中断"))?
    {
        if body.len().saturating_add(chunk.len()) > MAX_BODY {
            return Err(failure(feed, "响应体超过 2 MiB 限制"));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}
fn direct<'a, 'input>(parent: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    parent.children().find(|n| {
        n.is_element()
            && n.tag_name().name() == name
            && matches!(n.tag_name().namespace(), None | Some(ATOM))
    })
}
fn node_text(node: Node<'_, '_>) -> String {
    node.descendants()
        .filter_map(|n| n.is_text().then(|| n.text()).flatten())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn canonical_url(feed: Feed, value: &str) -> Result<String, String> {
    let mut url = Url::parse(value.trim()).map_err(|_| failure(feed, "条目链接格式无效"))?;
    if url.scheme() != "https"
        || url.host_str() != Some(feed.host)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some_and(|p| p != 443)
        || !feed.paths.iter().any(|p| url.path().starts_with(p))
        || url.as_str().len() > 500
    {
        return Err(failure(feed, "条目链接不属于官方来源"));
    }
    url.set_fragment(None);
    let pairs = url
        .query_pairs()
        .filter(|(key, _)| {
            !key.to_ascii_lowercase().starts_with("utm_")
                && !matches!(
                    key.to_ascii_lowercase().as_str(),
                    "ref" | "fbclid" | "gclid"
                )
        })
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    url.set_query(None);
    if !pairs.is_empty() {
        url.query_pairs_mut().extend_pairs(pairs);
    }
    let path = url.path().trim_end_matches('/').to_string();
    url.set_path(&path);
    Ok(url.to_string())
}
fn publication(item: Node<'_, '_>, atom: bool) -> Option<DateTime<Utc>> {
    let name = if atom { "published" } else { "pubDate" };
    // Atom updated is a revision time, not a substitute for an unknown publication date.
    let raw = node_text(direct(item, name)?);
    DateTime::parse_from_rfc2822(&raw)
        .or_else(|_| DateTime::parse_from_rfc3339(&raw))
        .ok()
        .map(|d| d.with_timezone(&Utc))
}
fn parse_feed(feed: Feed, body: &[u8]) -> Result<Vec<CollectedItem>, String> {
    if body.len() > MAX_BODY {
        return Err(failure(feed, "响应体超过 2 MiB 限制"));
    }
    let xml = std::str::from_utf8(body).map_err(|_| failure(feed, "XML 不是有效 UTF-8"))?;
    let doc = Document::parse_with_options(
        xml,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: MAX_NODES,
            entity_resolver: None,
        },
    )
    .map_err(|_| failure(feed, "XML 无效或超过安全限制"))?;
    if doc
        .descendants()
        .any(|n| n.ancestors().take(MAX_DEPTH + 1).count() > MAX_DEPTH)
    {
        return Err(failure(feed, "XML 嵌套层级超过安全限制"));
    }
    let root = doc.root_element();
    let atom = root.tag_name().name() == "feed" && root.tag_name().namespace() == Some(ATOM);
    let parent = if atom {
        root
    } else if root.tag_name().name() == "rss" && root.tag_name().namespace().is_none() {
        direct(root, "channel").ok_or_else(|| failure(feed, "RSS 缺少 channel"))?
    } else {
        return Err(failure(feed, "来源不是 RSS 或 Atom"));
    };
    let mut items = Vec::new();
    for item in parent
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == if atom { "entry" } else { "item" })
    {
        let title = direct(item, "title")
            .map(node_text)
            .filter(|s| {
                !s.is_empty() && s.chars().count() <= 300 && !s.chars().any(char::is_control)
            })
            .ok_or_else(|| failure(feed, "新闻标题缺失或过长"))?;
        let raw_link = if atom {
            item.children()
                .filter(|n| n.has_tag_name((ATOM, "link")))
                .find(|n| {
                    matches!(n.attribute("rel"), None | Some("alternate"))
                        && matches!(n.attribute("type"), None | Some("text/html"))
                })
                .and_then(|n| n.attribute("href"))
                .map(str::to_string)
        } else {
            direct(item, "link").map(node_text)
        }
        .ok_or_else(|| failure(feed, "新闻链接缺失"))?;
        let url = canonical_url(feed, &raw_link)?;
        let published = publication(item, atom);
        let hash = digest::digest(&digest::SHA256, url.as_bytes()).as_ref()[..24]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        items.push(CollectedItem {
            kind: "news".into(), key: format!("news-{hash}"),
            data: json!({"title": title, "source": feed.title, "url": url, "cover_url": "", "source_published_at": published.map(|d| d.to_rfc3339()), "time_basis": if published.is_some() {"source_published"} else {"collected"}}),
            sort_order: 0, published_at: published, source_name: feed.title.into(), source_url: url,
        });
    }
    if items.is_empty() {
        return Err(failure(feed, "来源没有可解析新闻条目"));
    }
    Ok(items)
}
fn select_candidates(mut items: Vec<CollectedItem>) -> Vec<CollectedItem> {
    items.sort_by(|a, b| {
        b.published_at
            .cmp(&a.published_at)
            .then_with(|| a.key.cmp(&b.key))
    });
    let mut seen = BTreeSet::new();
    items.retain(|i| seen.insert(i.source_url.clone()));
    items.truncate(MAX_ITEMS);
    items
}
/// Failure of any selected feed aborts this run. This cannot erase or publish existing content.
/// The draft sink additionally enforces the daily source quota across repeated runs.
pub async fn collect(http: &Client, config: &Value) -> Result<Vec<CollectedItem>, String> {
    let config = sanitize_config(config.clone()).map_err(|_| "官方日报来源配置无效".to_string())?;
    let ids = config["feed_ids"].as_array().expect("sanitized array");
    if ids.is_empty() {
        return Err("请先选择至少一个官方日报来源".into());
    }
    let mut collected = Vec::new();
    for id in ids {
        let feed = *FEEDS
            .iter()
            .find(|f| Some(f.id) == id.as_str())
            .expect("sanitized feed");
        let bytes = fetch_xml(http, feed, feed.url, TIMEOUT).await?;
        collected.extend(parse_feed(feed, &bytes)?);
    }
    Ok(select_candidates(collected))
}

#[cfg(test)]
#[path = "news_sources_tests.rs"]
mod tests;

//! 榜单项目的详情数据：README 抓取、简介与 README 的中文译文，以及把 Markdown 整理成客户端直接排版的纯文本块。
//! README 来自第三方，只输出文本块（不输出 HTML，不带图片和链接），客户端不需要再清洗。
use crate::{
    collectors::repo_sources::valid_name,
    translation::{self, Config},
};
use pulldown_cmark::{CodeBlockKind, Event, Options as MdOptions, Parser, Tag, TagEnd};
use ring::digest;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use tokio::{sync::Semaphore, task::JoinSet};

const GITHUB_API: &str = "https://api.github.com";
const README_BYTES: usize = 256 * 1024;
const README_CHARS: usize = 60_000;
const MAX_BLOCKS: usize = 600;
const BLOCK_CHARS: usize = 4000;

pub fn hash_hex(text: &str) -> String {
    digest::digest(&digest::SHA256, text.as_bytes()).as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cap(text: &str) -> String {
    text.chars().take(BLOCK_CHARS).collect()
}

/// 在段落边界截断，不把代码围栏截成半截；返回是否被截断。
pub fn truncate_md(md: &str, max_chars: usize) -> (String, bool) {
    if md.chars().count() <= max_chars {
        return (md.to_string(), false);
    }
    let cut: String = md.chars().take(max_chars).collect();
    let mut end = cut.len();
    if let Some(p) = cut.rfind("\n\n") {
        if p > cut.len() / 2 {
            end = p;
        }
    }
    let mut head = cut[..end].to_string();
    if head.matches("```").count() % 2 == 1 {
        if let Some(p) = head.rfind("```") {
            head.truncate(p);
        }
    }
    (head.trim_end().to_string(), true)
}

fn sanitize_md(raw: &str) -> String {
    let text: String = raw.replace("\r\n", "\n").chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\t').collect();
    truncate_md(&text, README_CHARS).0
}

fn mostly_cjk(text: &str) -> bool {
    let total = text.chars().filter(|c| c.is_alphanumeric()).count();
    let cjk = text.chars().filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)).count();
    total > 0 && cjk * 3 >= total
}

/// 译文简介：单行、去引号、限长。
fn clean_line(raw: &str) -> String {
    let line = collapse(&raw.chars().filter(|c| !c.is_control() || c.is_whitespace()).collect::<String>());
    line.trim_matches(|c| "\"'“”‘’「」".contains(c)).chars().take(240).collect()
}

/// Markdown 转文本块：h(l,x) p(x) li(x,d,n) quote(x) code(x,lang) table(rows) hr。
/// `drop_leading_h1` 去掉正文之前的一级标题（页面已有仓库名）。
pub fn readme_blocks(md: &str, drop_leading_h1: bool) -> Vec<Value> {
    let mut options = MdOptions::empty();
    options.insert(MdOptions::ENABLE_TABLES);
    options.insert(MdOptions::ENABLE_STRIKETHROUGH);
    options.insert(MdOptions::ENABLE_TASKLISTS);
    let mut out: Vec<Value> = Vec::new();
    let mut text = String::new();
    let mut heading: Option<u8> = None;
    let mut code_lang: Option<String> = None;
    let mut lists: Vec<Option<u64>> = Vec::new();
    let mut items: Vec<Option<u64>> = Vec::new();
    let (mut quote, mut image) = (0usize, 0usize);
    let mut table: Option<Vec<Vec<String>>> = None;
    let mut row: Vec<String> = Vec::new();
    let mut dropped = false;

    fn flush_item(out: &mut Vec<Value>, text: &mut String, items: &[Option<u64>], lists: &[Option<u64>]) {
        let t = collapse(text);
        text.clear();
        if !t.is_empty() && !items.is_empty() {
            out.push(json!({"t":"li","x":cap(&t),"d":lists.len().saturating_sub(1),"n":items.last().copied().flatten()}));
        }
    }

    for event in Parser::new_ext(md, options) {
        if out.len() >= MAX_BLOCKS {
            break;
        }
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_item(&mut out, &mut text, &items, &lists);
                heading = Some(level as u8);
                text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                let t = collapse(&text);
                text.clear();
                let level = heading.take().unwrap_or(2);
                if t.is_empty() {
                    continue;
                }
                if level == 1 && drop_leading_h1 && !dropped && out.iter().all(|b| b["t"] == "h") {
                    dropped = true;
                } else {
                    out.push(json!({"t":"h","l":level,"x":cap(&t)}));
                }
            }
            Event::Start(Tag::Paragraph) => {
                if !items.is_empty() && !text.is_empty() {
                    text.push(' ');
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if items.is_empty() {
                    let t = collapse(&text);
                    if !t.is_empty() {
                        out.push(json!({"t":if quote > 0 {"quote"} else {"p"},"x":cap(&t)}));
                    }
                    text.clear();
                }
            }
            Event::Start(Tag::BlockQuote(_)) => quote += 1,
            Event::End(TagEnd::BlockQuote(_)) => quote = quote.saturating_sub(1),
            Event::Start(Tag::List(start)) => {
                flush_item(&mut out, &mut text, &items, &lists);
                lists.push(start);
            }
            Event::End(TagEnd::List(_)) => {
                lists.pop();
            }
            Event::Start(Tag::Item) => {
                let ordinal = match lists.last_mut() {
                    Some(Some(n)) => {
                        let current = *n;
                        *n += 1;
                        Some(current)
                    }
                    _ => None,
                };
                items.push(ordinal);
                text.clear();
            }
            Event::End(TagEnd::Item) => {
                flush_item(&mut out, &mut text, &items, &lists);
                items.pop();
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_item(&mut out, &mut text, &items, &lists);
                code_lang = Some(match kind {
                    CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or("").chars().take(20).collect(),
                    CodeBlockKind::Indented => String::new(),
                });
                text.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                let lang = code_lang.take().unwrap_or_default();
                let body = text.trim_end_matches('\n').to_string();
                text.clear();
                if !body.trim().is_empty() {
                    out.push(json!({"t":"code","x":cap(&body),"lang":lang}));
                }
            }
            Event::Start(Tag::Image { .. }) => image += 1,
            Event::End(TagEnd::Image) => image = image.saturating_sub(1),
            Event::Start(Tag::Table(_)) => table = Some(Vec::new()),
            Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => row.clear(),
            Event::Start(Tag::TableCell) => text.clear(),
            Event::End(TagEnd::TableCell) => {
                row.push(cap(&collapse(&text)));
                text.clear();
            }
            Event::End(TagEnd::TableHead) | Event::End(TagEnd::TableRow) => {
                if let Some(rows) = table.as_mut() {
                    if rows.len() < 50 {
                        rows.push(row.iter().take(12).cloned().collect());
                    }
                }
                row.clear();
            }
            Event::End(TagEnd::Table) => {
                if let Some(rows) = table.take().filter(|r| !r.is_empty()) {
                    out.push(json!({"t":"table","rows":rows}));
                }
            }
            Event::Rule => out.push(json!({"t":"hr"})),
            Event::TaskListMarker(done) => text.push_str(if done { "☑ " } else { "☐ " }),
            Event::Text(t) | Event::Code(t) => {
                if image == 0 {
                    text.push_str(&t);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if code_lang.is_some() {
                    text.push('\n');
                } else if image == 0 {
                    text.push(' ');
                }
            }
            _ => {}
        }
    }
    out
}

pub async fn fetch_readme(http: &reqwest::Client, api: &str, full_name: &str) -> Result<Option<String>, String> {
    if !valid_name(full_name) {
        return Err("仓库名无效".into());
    }
    let mut response = http
        .get(format!("{api}/repos/{full_name}/readme"))
        .header("accept", "application/vnd.github.raw+json")
        .header("x-github-api-version", "2022-11-28")
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|_| "README 请求失败".to_string())?;
    match response.status().as_u16() {
        200 => {}
        404 => return Ok(None),
        code => return Err(format!("README 请求未成功（HTTP {code}）")),
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "README 读取失败".to_string())? {
        let room = README_BYTES.saturating_sub(bytes.len());
        bytes.extend_from_slice(&chunk[..chunk.len().min(room)]);
        if bytes.len() >= README_BYTES {
            break;
        }
    }
    let text = sanitize_md(&String::from_utf8_lossy(&bytes));
    Ok(Some(text).filter(|t| !t.trim().is_empty()))
}

/// 采集时调用：抓 README、翻译简介，并把中文简介写回榜单项。任何一步失败都不影响榜单发布。
pub async fn enrich(pool: &PgPool, http: &reqwest::Client, ranks: &mut [Value]) {
    enrich_with(pool, http, ranks, GITHUB_API).await
}

pub(crate) async fn enrich_with(pool: &PgPool, http: &reqwest::Client, ranks: &mut [Value], api: &str) {
    if let Err(error) = run_enrich(pool, http, ranks, api).await {
        tracing::warn!("榜单详情整理失败：{error}");
    }
}

type Row = (String, String, String);

async fn run_enrich(pool: &PgPool, http: &reqwest::Client, ranks: &mut [Value], api: &str) -> Result<(), sqlx::Error> {
    let cfg = match translation::load(pool).await {
        Ok(cfg) => cfg,
        Err(error) => {
            tracing::warn!("翻译服务设置不可用：{error}");
            None
        }
    };
    let entries: Vec<(String, String, String)> = ranks
        .iter()
        .filter_map(|r| {
            let name = r["full_name"].as_str()?;
            Some((name.to_ascii_lowercase(), name.to_string(), r["description"].as_str().unwrap_or("").to_string()))
        })
        .collect();
    let keys: Vec<String> = entries.iter().map(|e| e.0.clone()).collect();
    let old: Vec<Row> = sqlx::query_as("SELECT key,desc_en,desc_zh FROM repo_details WHERE key=ANY($1)").bind(&keys).fetch_all(pool).await?;
    let mut zh: HashMap<String, String> = HashMap::new();
    let old: HashMap<String, (String, String)> = old.into_iter().map(|(k, en, zh)| (k, (en, zh))).collect();
    for (key, _, desc) in &entries {
        if let Some((en, z)) = old.get(key) {
            if en == desc && !z.is_empty() {
                zh.insert(key.clone(), z.clone());
            }
        }
    }

    // README：并发抓取，整体限时；失败的项目保留旧数据。
    let gate = Arc::new(Semaphore::new(5));
    let mut fetches = JoinSet::new();
    for (key, name, _) in &entries {
        let (http, gate, api, key, name) = (http.clone(), gate.clone(), api.to_string(), key.clone(), name.clone());
        fetches.spawn(async move {
            let _permit = gate.acquire_owned().await;
            (key, fetch_readme(&http, &api, &name).await)
        });
    }
    let mut fetched: HashMap<String, Result<Option<String>, String>> = HashMap::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while let Ok(Some(joined)) = tokio::time::timeout_at(deadline, fetches.join_next()).await {
        if let Ok((key, result)) = joined {
            fetched.insert(key, result);
        }
    }
    fetches.abort_all();

    for (key, name, desc) in &entries {
        sqlx::query("INSERT INTO repo_details(key,full_name,desc_en) VALUES($1,$2,$3)
            ON CONFLICT(key) DO UPDATE SET full_name=$2, desc_zh=CASE WHEN repo_details.desc_en=$3 THEN repo_details.desc_zh ELSE '' END, desc_en=$3, updated_at=now()")
            .bind(key).bind(name).bind(desc).execute(pool).await?;
        match fetched.get(key) {
            Some(Ok(Some(md))) => {
                sqlx::query("UPDATE repo_details SET readme_status='ok',readme_md=$2,readme_hash=$3,fetched_at=now(),updated_at=now() WHERE key=$1")
                    .bind(key).bind(md).bind(hash_hex(md)).execute(pool).await?;
            }
            Some(Ok(None)) => {
                sqlx::query("UPDATE repo_details SET readme_status='none',readme_md='',readme_hash='',readme_zh='',readme_zh_hash='',readme_zh_truncated=false,fetched_at=now(),updated_at=now() WHERE key=$1")
                    .bind(key).execute(pool).await?;
            }
            _ => {}
        }
    }

    if let Some(cfg) = &cfg {
        if cfg.options.translate_summary {
            translate_summaries(pool, http, cfg, &entries, &mut zh).await;
        }
    }
    for rank in ranks.iter_mut() {
        if let Some(text) = rank["full_name"].as_str().and_then(|n| zh.get(&n.to_ascii_lowercase())) {
            rank["description_zh"] = json!(text);
        }
    }
    if let Some(cfg) = cfg.filter(|c| c.options.translate_readme) {
        tokio::spawn(translate_pending(pool.clone(), http.clone(), cfg));
    }
    Ok(())
}

async fn translate_summaries(pool: &PgPool, http: &reqwest::Client, cfg: &Config, entries: &[(String, String, String)], zh: &mut HashMap<String, String>) {
    let mut jobs = JoinSet::new();
    let gate = Arc::new(Semaphore::new(5));
    let mut started = 0;
    for (key, name, desc) in entries {
        if desc.trim().is_empty() || zh.contains_key(key) {
            continue;
        }
        if mostly_cjk(desc) {
            let _ = sqlx::query("UPDATE repo_details SET desc_zh=$2 WHERE key=$1 AND desc_en=$3").bind(key).bind(desc).bind(desc).execute(pool).await;
            zh.insert(key.clone(), desc.clone());
            continue;
        }
        if started >= cfg.options.max_new {
            break;
        }
        started += 1;
        let (http, cfg, gate, pool, key, name, desc) = (http.clone(), cfg.clone(), gate.clone(), pool.clone(), key.clone(), name.clone(), desc.clone());
        jobs.spawn(async move {
            let _permit = gate.acquire_owned().await;
            let begin = Instant::now();
            let result = translation::complete(&http, &cfg, translation::SUMMARY_PROMPT, &desc, 512, Duration::from_secs(30)).await.map(|t| clean_line(&t)).and_then(|t| if t.is_empty() { Err("译文为空".to_string()) } else { Ok(t) });
            let millis = begin.elapsed().as_millis();
            match &result {
                Ok(text) => {
                    let _ = sqlx::query("UPDATE repo_details SET desc_zh=$2 WHERE key=$1 AND desc_en=$3").bind(&key).bind(text).bind(&desc).execute(&pool).await;
                    translation::log(&pool, &name, "summary", true, "", millis).await;
                }
                Err(note) => translation::log(&pool, &name, "summary", false, note, millis).await,
            }
            (key, result.ok())
        });
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    while let Ok(Some(joined)) = tokio::time::timeout_at(deadline, jobs.join_next()).await {
        if let Ok((key, Some(text))) = joined {
            zh.insert(key, text);
        }
    }
    jobs.abort_all();
}

/// 后台翻译 README：不占采集任务的时限；按行认领，重复触发不会重复翻译同一份。
pub async fn translate_pending(pool: PgPool, http: reqwest::Client, cfg: Config) {
    let rows: Result<Vec<(String, String, String, String)>, _> = sqlx::query_as(
        "SELECT key,full_name,readme_md,readme_hash FROM repo_details
         WHERE readme_status='ok' AND readme_hash<>readme_zh_hash AND (translating_until IS NULL OR translating_until<now())
         ORDER BY updated_at DESC LIMIT $1",
    )
    .bind(cfg.options.max_new as i64)
    .fetch_all(&pool)
    .await;
    let Ok(rows) = rows else { return };
    let gate = Arc::new(Semaphore::new(3));
    let mut jobs = JoinSet::new();
    for (key, name, md, hash) in rows {
        let (http, cfg, gate, pool) = (http.clone(), cfg.clone(), gate.clone(), pool.clone());
        jobs.spawn(async move {
            let _permit = gate.acquire_owned().await;
            let claimed = sqlx::query("UPDATE repo_details SET translating_until=now()+interval '10 minutes' WHERE key=$1 AND readme_hash=$2 AND (translating_until IS NULL OR translating_until<now())")
                .bind(&key).bind(&hash).execute(&pool).await.map(|d| d.rows_affected() == 1).unwrap_or(false);
            if !claimed {
                return;
            }
            let (source, truncated) = truncate_md(&md, cfg.options.readme_max_chars);
            let tokens = ((cfg.options.readme_max_chars as f64 * 0.9) as u32).clamp(2048, 16000);
            let begin = Instant::now();
            let result = translation::complete(&http, &cfg, translation::README_PROMPT, &source, tokens, Duration::from_secs(180)).await;
            let millis = begin.elapsed().as_millis();
            match result {
                Ok(zh) => {
                    let _ = sqlx::query("UPDATE repo_details SET readme_zh=$2,readme_zh_hash=$3,readme_zh_truncated=$4,translating_until=NULL,updated_at=now() WHERE key=$1 AND readme_hash=$3")
                        .bind(&key).bind(sanitize_md(&zh)).bind(&hash).bind(truncated).execute(&pool).await;
                    translation::log(&pool, &name, "readme", true, if truncated { "已截断翻译" } else { "" }, millis).await;
                }
                Err(note) => {
                    let _ = sqlx::query("UPDATE repo_details SET translating_until=NULL WHERE key=$1").bind(&key).execute(&pool).await;
                    translation::log(&pool, &name, "readme", false, &note, millis).await;
                }
            }
        });
    }
    while jobs.join_next().await.is_some() {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collectors::credentials, translation::test_support::mock};

    fn kinds(blocks: &[Value]) -> Vec<String> {
        blocks.iter().map(|b| b["t"].as_str().unwrap().to_string()).collect()
    }

    #[test]
    fn markdown_becomes_plain_text_blocks_without_html_images_or_link_targets() {
        let md = "# Claude SEO\n\n[![build](https://img.shields.io/x.svg)](https://ci.example)\n\n<p align=\"center\"><img src=\"x.png\"></p>\n\nUniversal **SEO** skill with `code` and a [link](https://evil.example/x).\n\n## Features\n\n- Technical audit\n- Content analysis\n  - nested item\n\n1. First\n2. Second\n\n> quoted words\n\n```bash\n/plugin install claude-seo\necho done\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n---\n\n<script>alert(1)</script>\n";
        let blocks = readme_blocks(md, true);
        assert_eq!(kinds(&blocks), ["p", "h", "li", "li", "li", "li", "li", "quote", "code", "table", "hr"]);
        assert_eq!(blocks[0]["x"], "Universal SEO skill with code and a link.");
        assert_eq!((blocks[1]["l"].as_u64(), blocks[1]["x"].as_str()), (Some(2), Some("Features")));
        assert_eq!((blocks[2]["d"].as_u64(), blocks[3]["x"].as_str(), blocks[4]["d"].as_u64()), (Some(0), Some("Content analysis"), Some(1)));
        assert_eq!((blocks[5]["n"].as_u64(), blocks[6]["n"].as_u64()), (Some(1), Some(2)));
        assert_eq!((blocks[8]["lang"].as_str(), blocks[8]["x"].as_str()), (Some("bash"), Some("/plugin install claude-seo\necho done")));
        assert_eq!(blocks[9]["rows"], json!([["a", "b"], ["1", "2"]]));
        let all = serde_json::to_string(&blocks).unwrap();
        assert!(!all.contains("<script") && !all.contains("shields.io") && !all.contains("evil.example") && !all.contains("alert(1)"), "{all}");
    }

    #[test]
    fn only_the_leading_h1_is_dropped_and_only_when_asked() {
        assert_eq!(kinds(&readme_blocks("# Title\n\nbody\n\n# Later\n", true)), ["p", "h"]);
        assert_eq!(kinds(&readme_blocks("# Title\n\nbody\n", false)), ["h", "p"]);
        assert_eq!(kinds(&readme_blocks("text first\n\n# Title\n", true)), ["p", "h"]);
        assert!(readme_blocks("", true).is_empty() && readme_blocks("![only](img.png)\n", true).is_empty());
    }

    #[test]
    fn block_count_and_text_size_are_capped() {
        let many = (0..900).map(|i| format!("paragraph {i}")).collect::<Vec<_>>().join("\n\n");
        assert_eq!(readme_blocks(&many, false).len(), MAX_BLOCKS);
        let long = readme_blocks(&"字".repeat(20_000), false);
        assert_eq!(long[0]["x"].as_str().unwrap().chars().count(), BLOCK_CHARS);
    }

    #[test]
    fn truncation_cuts_at_paragraphs_and_never_inside_a_fence() {
        let md = format!("{}\n\n{}", "a".repeat(60), "b".repeat(60));
        let (head, cut) = truncate_md(&md, 100);
        assert!(cut && head == "a".repeat(60), "{head}");
        let fenced = format!("{}\n\n```\n{}\n```\n\n{}", "x".repeat(30), "y".repeat(80), "z".repeat(30));
        let (head, cut) = truncate_md(&fenced, 70);
        assert!(cut && head.matches("```").count() % 2 == 0, "{head}");
        assert_eq!(truncate_md("short", 100), ("short".to_string(), false));
    }

    #[test]
    fn language_and_line_helpers() {
        assert!(mostly_cjk("面向 Claude Code 的技能包") && !mostly_cjk("A Claude Code skill") && !mostly_cjk("1234"));
        assert_eq!(clean_line("\n“面向 Claude Code 的技能。”\n"), "面向 Claude Code 的技能。");
        assert_eq!(clean_line(&"字".repeat(400)).chars().count(), 240);
        assert_eq!(hash_hex("a").len(), 64);
        assert_ne!(hash_hex("a"), hash_hex("b"));
    }

    #[tokio::test]
    async fn readme_fetch_distinguishes_missing_from_unavailable() {
        let http = credentials::http_client().unwrap();
        let server = mock(|req| {
            if req.starts_with("GET /repos/owner/has/readme ") {
                assert!(req.to_lowercase().contains("accept: application/vnd.github.raw+json"));
                (200, "# Hi\r\n\0body".into())
            } else if req.starts_with("GET /repos/owner/none/readme ") {
                (404, "{}".into())
            } else {
                (403, "{\"message\":\"rate limit\"}".into())
            }
        }).await;
        assert_eq!(fetch_readme(&http, &server.origin, "owner/has").await.unwrap().as_deref(), Some("# Hi\nbody"));
        assert_eq!(fetch_readme(&http, &server.origin, "owner/none").await.unwrap(), None);
        let err = fetch_readme(&http, &server.origin, "owner/limited").await.unwrap_err();
        assert!(err.contains("403") && !err.contains("rate limit"), "{err}");
        assert!(fetch_readme(&http, &server.origin, "../etc/passwd").await.is_err());
    }
}

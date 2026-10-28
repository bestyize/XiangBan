//! 前端单页的托管，以及给搜索引擎的服务端渲染页面。
//!
//! - 静态资源：从 `web_dir`（Vite 构建产物）用 ServeDir 托管，`/assets/` 下带哈希的文件强缓存。
//! - SEO 页面：首页、福利羊毛、帖子、分类、标签直接输出带标题、摘要和正文的 HTML。
//!   做法是把启动时读好的 index.html 里的 `<title>`、description、canonical、OG 标签换成页面自己的，
//!   并把可读的语义 HTML 填进 `<div id="app"></div>`；浏览器里 Vue 挂载时会整体替换这块内容。
//! - 兜底：其余 GET 路径都返回 index.html（200），交给前端路由；`/api/*`、`/assets/*` 未命中返回 404。

use crate::{
    error::AppError,
    models::{self, ListFilter, TopicItem},
    state::AppState,
};
use axum::{
    extract::{Path, Request, State},
    http::{header, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use chrono::{DateTime, FixedOffset, Utc};
use serde_json::json;
use std::sync::Arc;
use tower_http::services::ServeDir;
use uuid::Uuid;

/// 前端没构建时用的最小外壳，保证 SEO 路由仍可访问。
const FALLBACK_INDEX: &str = r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>超级小伴</title><meta name="description" content="超级小伴"></head><body><div id="app"></div></body></html>"#;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// 启动时读好的 index.html，各 SEO 处理器共享。
#[derive(Clone)]
struct Index(Arc<String>);

/// 一个 SEO 页面要注入的内容。
struct Page {
    /// 完整的 `<title>` 文本（已带站点名）。
    title: String,
    description: String,
    /// 站内路径（已做过百分号编码），拼上 public_url 即 canonical。
    path: String,
    og_type: &'static str,
    /// 放进 `<div id="app">` 的预渲染 HTML。
    body: String,
    json_ld: Option<String>,
}

// ---------- 文本工具 ----------

/// HTML 转义，用于所有进入标签内容或属性的用户可控文本。
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// 路径片段的百分号编码：只放过 RFC 3986 的 unreserved 字节。
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// 折叠空白并按字符数截断，超长时以省略号结尾（总长不超过 `max`）。
fn truncate_chars(s: &str, max: usize) -> String {
    let flat = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let mut out: String = flat.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn cn_date(dt: DateTime<Utc>) -> String {
    let tz = FixedOffset::east_opt(8 * 3600).expect("固定时区");
    dt.with_timezone(&tz).format("%Y-%m-%d").to_string()
}

// ---------- index.html 注入 ----------

/// 删除 html 里所有以 `open`（如 `<meta`）开头、且整个标签满足 `pred` 的标签。
fn remove_tags(html: &str, open: &str, pred: impl Fn(&str) -> bool) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find(open) {
        let Some(len) = rest[start..].find('>') else { break };
        let end = start + len + 1;
        out.push_str(&rest[..start]);
        if !pred(&rest[start..end]) {
            out.push_str(&rest[start..end]);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn has_attr(tag: &str, name: &str, value: &str) -> bool {
    tag.contains(&format!("{name}=\"{value}\"")) || tag.contains(&format!("{name}='{value}'"))
}

fn is_robots_meta(tag: &str) -> bool {
    has_attr(tag, "name", "robots")
}

/// 在 `</head>` 前插入内容；没有 head 时放在最前面。
fn insert_in_head(html: &str, extra: &str) -> String {
    match html.find("</head>") {
        Some(i) => format!("{}{}\n{}", &html[..i], extra, &html[i..]),
        None => format!("{extra}{html}"),
    }
}

fn replace_title(html: &str, title: &str) -> String {
    let new = format!("<title>{}</title>", escape(title));
    if let Some(s) = html.find("<title") {
        if let Some(e) = html[s..].find("</title>") {
            return format!("{}{}{}", &html[..s], new, &html[s + e + "</title>".len()..]);
        }
    }
    insert_in_head(html, &new)
}

/// 把预渲染内容填进 `<div id="app">`；找不到挂载点就原样返回。
fn fill_app(html: &str, content: &str) -> String {
    for marker in ["<div id=\"app\"", "<div id='app'"] {
        let Some(s) = html.find(marker) else { continue };
        let Some(gt) = html[s..].find('>') else { continue };
        let inner = s + gt + 1;
        let Some(close) = html[inner..].find("</div>") else { continue };
        return format!("{}{}{}", &html[..inner], content, &html[inner + close..]);
    }
    html.to_string()
}

/// JSON-LD 放进 `<script>` 前把所有小于号换成 JSON 的 unicode 转义（u003c），防止值里的结束标签提前截断脚本。
fn json_ld_script(value: &serde_json::Value) -> String {
    let raw = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    format!("<script type=\"application/ld+json\">{}</script>", raw.replace('<', "\\u003c"))
}

/// 用页面内容改写 index.html：标题、description、canonical、OG、JSON-LD 和 #app 内容。
fn inject(index: &str, site_name: &str, base: &str, page: &Page) -> String {
    let mut html = remove_tags(index, "<meta", |t| {
        has_attr(t, "name", "description") || t.contains("property=\"og:") || t.contains("property='og:")
    });
    html = remove_tags(&html, "<link", |t| has_attr(t, "rel", "canonical"));
    html = replace_title(&html, &page.title);

    let url = escape(&format!("{base}{}", page.path));
    let mut head = format!(
        "<meta name=\"description\" content=\"{desc}\">\n<link rel=\"canonical\" href=\"{url}\">\n\
         <meta property=\"og:title\" content=\"{title}\">\n<meta property=\"og:description\" content=\"{desc}\">\n\
         <meta property=\"og:type\" content=\"{ty}\">\n<meta property=\"og:url\" content=\"{url}\">\n\
         <meta property=\"og:site_name\" content=\"{site}\">\n<meta property=\"og:locale\" content=\"zh_CN\">",
        desc = escape(&page.description),
        title = escape(&page.title),
        ty = page.og_type,
        site = escape(site_name),
    );
    if let Some(ld) = &page.json_ld {
        head.push('\n');
        head.push_str(ld);
    }
    html = insert_in_head(&html, &head);
    fill_app(&html, &page.body)
}

/// 404 页：仍是普通的 SPA 外壳（前端自己渲染“不存在”），只加 noindex。
fn inject_noindex(index: &str) -> String {
    let html = remove_tags(index, "<meta", is_robots_meta);
    insert_in_head(&html, "<meta name=\"robots\" content=\"noindex\">")
}

// ---------- 路由 ----------

pub fn routes(state: &AppState) -> Router<AppState> {
    let web_dir = state.config.web_dir.clone();
    let index_path = web_dir.join("index.html");
    let html = match std::fs::read_to_string(&index_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("读取 {} 失败（{e}），SEO 页面改用内置的最小 HTML 外壳，请先构建前端", index_path.display());
            FALLBACK_INDEX.to_string()
        }
    };
    let index = Index(Arc::new(html));

    // 静态文件没命中时走这里：返回 SPA 外壳，交给前端路由。
    let spa_index = index.clone();
    let spa = get(move |uri: Uri| {
        let index = spa_index.clone();
        async move { spa_fallback(uri.path(), &index) }
    });

    Router::new()
        .route("/", get(home))
        .route("/t/{id}", get(topic_page))
        .route("/c/{slug}", get(category_page))
        .route("/tag/{name}", get(tag_page))
        .route("/sitemap.xml", get(sitemap))
        .route("/robots.txt", get(robots))
        .fallback_service(ServeDir::new(&web_dir).append_index_html_on_directories(false).fallback(spa))
        .layer(Extension(index))
        .layer(middleware::from_fn(cache_headers))
}

fn spa_fallback(path: &str, index: &Index) -> Response {
    // 未知的接口和缺失的静态资源要给真 404，不能回 index.html 误导客户端和爬虫。
    if path == "/api" || path.starts_with("/api/") {
        return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "application/json")], r#"{"error":true,"code":"not_found","message":"接口不存在"}"#)
            .into_response();
    }
    if path.starts_with("/assets/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    html_response(StatusCode::OK, index.0.to_string())
}

/// 带哈希的 /assets/ 强缓存；HTML 一律 no-cache，保证发版后立刻生效。
async fn cache_headers(req: Request, next: Next) -> Response {
    let is_asset = req.uri().path().starts_with("/assets/");
    let mut res = next.run(req).await;
    if res.headers().contains_key(header::CACHE_CONTROL) {
        return res;
    }
    let ok = res.status().is_success() || res.status() == StatusCode::NOT_MODIFIED;
    let is_html = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/html"));
    let value = if is_asset && ok {
        Some(IMMUTABLE)
    } else if is_html {
        Some("no-cache")
    } else {
        None
    };
    if let Some(v) = value {
        res.headers_mut().insert(header::CACHE_CONTROL, v.parse().expect("静态头值"));
    }
    res
}

fn html_response(status: StatusCode, body: String) -> Response {
    (status, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], body).into_response()
}

/// 各 SEO 处理器共用的渲染上下文。
struct Ctx<'a> {
    base: &'a str,
    name: &'a str,
    index: &'a str,
}

impl<'a> Ctx<'a> {
    fn new(state: &'a AppState, index: &'a Index) -> Self {
        Ctx { base: &state.config.public_url, name: &state.config.site_name, index: &index.0 }
    }

    fn full_title(&self, t: &str) -> String {
        format!("{t} - {}", self.name)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    fn ok(&self, page: Page) -> Response {
        html_response(StatusCode::OK, inject(self.index, self.name, self.base, &page))
    }

    fn not_found(&self) -> Response {
        html_response(StatusCode::NOT_FOUND, inject_noindex(self.index))
    }
}

type Res = Result<Response, AppError>;

// ---------- HTML 片段 ----------

/// 帖子列表：标题链接 + 分类和作者。
fn topic_list(items: &[TopicItem]) -> String {
    if items.is_empty() {
        return "<p>暂无内容</p>".into();
    }
    let mut s = String::from("<ul>\n");
    for t in items {
        s.push_str(&format!("<li><a href=\"/t/{}\">{}</a>", t.id, escape(&t.title)));
        s.push_str(&format!(
            " <small>{} · {} · {}</small></li>\n",
            escape(&t.category.name),
            escape(&t.author.nickname),
            cn_date(t.created_at)
        ));
    }
    s.push_str("</ul>");
    s
}

fn tag_links(tags: &[String]) -> String {
    tags.iter()
        .map(|t| format!("<a href=\"/tag/{}\">#{}</a>", percent_encode(t), escape(t)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 全站导航：资源、周刊、项目和分类树，让不跑 JS 的爬虫能顺着链接爬到各板块。
async fn category_nav(state: &AppState) -> Result<String, AppError> {
    let rows: Vec<(i32, Option<i32>, String, String)> =
        sqlx::query_as("SELECT id, parent_id, slug, name FROM categories ORDER BY sort_order, id")
            .fetch_all(&state.pool)
            .await?;
    let mut s = String::from("<nav>\n<a href=\"/resources\">资源</a> · <a href=\"/weekly\">周刊</a> · <a href=\"/projects\">项目</a>\n<ul>\n");
    for (id, _, slug, name) in rows.iter().filter(|r| r.1.is_none()) {
        s.push_str(&format!("<li><a href=\"/c/{}\">{}</a>", percent_encode(slug), escape(name)));
        let children: Vec<_> = rows.iter().filter(|r| r.1 == Some(*id)).collect();
        if !children.is_empty() {
            s.push(' ');
            let links: Vec<String> = children
                .iter()
                .map(|(_, _, slug, name)| format!("<a href=\"/c/{}\">{}</a>", percent_encode(slug), escape(name)))
                .collect();
            s.push_str(&links.join(" · "));
        }
        s.push_str("</li>\n");
    }
    s.push_str("</ul>\n</nav>");
    Ok(s)
}

// ---------- 页面处理器 ----------

async fn home(State(state): State<AppState>, Extension(index): Extension<Index>) -> Res {
    let cx = Ctx::new(&state, &index);
    let topics = models::list_topics(
        &state.pool,
        &ListFilter { home: true, page_size: Some(30), ..Default::default() },
    )
    .await?;
    let nav = category_nav(&state).await?;
    let body = format!(
        "<h1>{name}：开发者和 AI 用户的社区</h1>\n\
         <p>{name}是开发者和 AI 用户的社区：技术交流、资源分享、福利羊毛和随便唠唠。福利羊毛里是大家分享的免费额度、试用和优惠。</p>\n\
         {nav}\n<h2>最新讨论</h2>\n{list}",
        name = escape(cx.name),
        list = topic_list(&topics.items),
    );
    Ok(cx.ok(Page {
        title: format!("{} - 开发者和 AI 用户的社区", cx.name),
        description: truncate_chars(
            &format!("{}是开发者和 AI 用户的社区：技术交流、资源分享、福利羊毛和随便唠唠。", cx.name),
            150,
        ),
        path: "/".into(),
        og_type: "website",
        body,
        json_ld: Some(json_ld_script(&json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": cx.name,
            "url": cx.url("/"),
            "inLanguage": "zh-CN",
        }))),
    }))
}

async fn topic_page(
    State(state): State<AppState>,
    Extension(index): Extension<Index>,
    Path(id): Path<String>,
) -> Res {
    let cx = Ctx::new(&state, &index);
    let Some(id) = id.parse::<i64>().ok().filter(|i| *i > 0) else { return Ok(cx.not_found()) };
    // 设了阅读权限的帖子不给搜索引擎看：当作不存在（noindex）。登录用户浏览器里的前端会带着令牌自己去取。
    let Some(topic) = models::get_topic(&state.pool, id)
        .await?
        .filter(|t| t.status == "normal" && t.min_read_level == 0)
    else {
        return Ok(cx.not_found());
    };
    // body_html 在写入时已由 ammonia 清洗过，可以原样输出。
    let (body_html,): (String,) =
        sqlx::query_as("SELECT body_html FROM topics WHERE id = $1").bind(id).fetch_one(&state.pool).await?;
    let posts: Vec<(i32, String, DateTime<Utc>, Uuid, String)> = sqlx::query_as(
        "SELECT p.floor, p.body_html, p.created_at, u.uid, u.nickname
         FROM posts p JOIN users u ON u.uid = p.author_uid
         WHERE p.topic_id = $1 AND p.status = 'normal' ORDER BY p.floor LIMIT 50",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let mut body = format!(
        "<article>\n<h1>{title}</h1>\n<p>分类：<a href=\"/c/{cslug}\">{cname}</a> · 作者：<a href=\"/u/{uid}\">{nick}</a> · <time datetime=\"{iso}\">{date}</time></p>\n",
        title = escape(&topic.title),
        cslug = percent_encode(&topic.category.slug),
        cname = escape(&topic.category.name),
        uid = topic.author.uid,
        nick = escape(&topic.author.nickname),
        iso = topic.created_at.to_rfc3339(),
        date = cn_date(topic.created_at),
    );
    body.push_str(&format!("<div>{body_html}</div>\n"));
    if !topic.tags.is_empty() {
        body.push_str(&format!("<p>{}</p>\n", tag_links(&topic.tags)));
    }
    body.push_str("</article>\n");
    if !posts.is_empty() {
        body.push_str(&format!("<section>\n<h2>回复（{}）</h2>\n", topic.reply_count));
        for (floor, html, at, uid, nick) in &posts {
            body.push_str(&format!(
                "<div>\n<p>#{floor} <a href=\"/u/{uid}\">{}</a> · <time datetime=\"{}\">{}</time></p>\n<div>{html}</div>\n</div>\n",
                escape(nick),
                at.to_rfc3339(),
                cn_date(*at),
            ));
        }
        body.push_str("</section>");
    }

    let path = format!("/t/{id}");
    let mut desc = topic.excerpt.clone();
    if desc.trim().is_empty() {
        desc = topic.title.clone();
    }
    let json_ld = json_ld_script(&json!({
        "@context": "https://schema.org",
        "@type": "DiscussionForumPosting",
        "@id": cx.url(&path),
        "url": cx.url(&path),
        "headline": truncate_chars(&topic.title, 110),
        "articleBody": topic.excerpt,
        "inLanguage": "zh-CN",
        "datePublished": topic.created_at.to_rfc3339(),
        "dateModified": topic.edited_at.unwrap_or(topic.last_reply_at).to_rfc3339(),
        "author": {
            "@type": "Person",
            "name": topic.author.nickname,
            "url": cx.url(&format!("/u/{}", topic.author.uid)),
        },
        "publisher": { "@type": "Organization", "name": cx.name },
        "commentCount": topic.reply_count,
        "interactionStatistic": [
            {
                "@type": "InteractionCounter",
                "interactionType": "https://schema.org/CommentAction",
                "userInteractionCount": topic.reply_count,
            },
            {
                "@type": "InteractionCounter",
                "interactionType": "https://schema.org/LikeAction",
                "userInteractionCount": topic.like_count,
            },
        ],
    }));
    Ok(cx.ok(Page {
        title: cx.full_title(&topic.title),
        description: truncate_chars(&desc, 150),
        path,
        og_type: "article",
        body,
        json_ld: Some(json_ld),
    }))
}

async fn category_page(
    State(state): State<AppState>,
    Extension(index): Extension<Index>,
    Path(slug): Path<String>,
) -> Res {
    let cx = Ctx::new(&state, &index);
    let cat: Option<(i32, String, String)> =
        sqlx::query_as("SELECT id, name, description FROM categories WHERE slug = $1").bind(&slug).fetch_optional(&state.pool).await?;
    let Some((cid, name, desc)) = cat else { return Ok(cx.not_found()) };
    let children: Vec<(String, String)> =
        sqlx::query_as("SELECT slug, name FROM categories WHERE parent_id = $1 ORDER BY sort_order, id")
            .bind(cid)
            .fetch_all(&state.pool)
            .await?;
    let topics = models::list_topics(
        &state.pool,
        &ListFilter { category: Some(slug.clone()), page_size: Some(30), ..Default::default() },
    )
    .await?;

    let mut body = format!("<h1>{}</h1>\n", escape(&name));
    if !desc.is_empty() {
        body.push_str(&format!("<p>{}</p>\n", escape(&desc)));
    }
    if !children.is_empty() {
        let links: Vec<String> = children
            .iter()
            .map(|(s, n)| format!("<a href=\"/c/{}\">{}</a>", percent_encode(s), escape(n)))
            .collect();
        body.push_str(&format!("<p>子分类：{}</p>\n", links.join(" · ")));
    }
    body.push_str(&format!("<p>共 {} 个帖子</p>\n{}", topics.total, topic_list(&topics.items)));
    Ok(cx.ok(Page {
        title: cx.full_title(&name),
        description: truncate_chars(
            &if desc.is_empty() { format!("{name}分类下的最新帖子与讨论") } else { format!("{name}：{desc}") },
            150,
        ),
        path: format!("/c/{}", percent_encode(&slug)),
        og_type: "website",
        body,
        json_ld: None,
    }))
}

async fn tag_page(
    State(state): State<AppState>,
    Extension(index): Extension<Index>,
    Path(name): Path<String>,
) -> Res {
    let cx = Ctx::new(&state, &index);
    let name = name.trim().to_lowercase();
    if name.is_empty() {
        return Ok(cx.not_found());
    }
    let topics = models::list_topics(
        &state.pool,
        &ListFilter { tag: Some(name.clone()), page_size: Some(30), ..Default::default() },
    )
    .await?;
    // 没有帖子的标签不收录，免得被当成空页面。
    if topics.total == 0 {
        return Ok(cx.not_found());
    }
    let body = format!(
        "<h1>#{}</h1>\n<p>共 {} 个帖子</p>\n{}",
        escape(&name),
        topics.total,
        topic_list(&topics.items)
    );
    Ok(cx.ok(Page {
        title: cx.full_title(&format!("#{name}")),
        description: truncate_chars(&format!("标签 {name} 下的帖子与讨论，共 {} 篇。", topics.total), 150),
        path: format!("/tag/{}", percent_encode(&name)),
        og_type: "website",
        body,
        json_ld: None,
    }))
}

// ---------- sitemap 与 robots ----------

fn sitemap_entry(out: &mut String, loc: &str, lastmod: Option<&str>) {
    out.push_str("<url><loc>");
    out.push_str(&escape(loc));
    out.push_str("</loc>");
    if let Some(m) = lastmod {
        out.push_str(&format!("<lastmod>{m}</lastmod>"));
    }
    out.push_str("</url>\n");
}

async fn sitemap(State(state): State<AppState>) -> Res {
    let base = state.config.public_url.as_str();
    let cats: Vec<(String,)> = sqlx::query_as("SELECT slug FROM categories ORDER BY sort_order, id").fetch_all(&state.pool).await?;
    // GREATEST 会忽略 NULL，所以没编辑过的帖子取最后回复时间。
    let topics: Vec<(i64, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, GREATEST(last_reply_at, edited_at) AS m FROM topics WHERE status = 'normal' AND min_read_level = 0 ORDER BY m DESC LIMIT 5000",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for p in ["/", "/resources", "/weekly", "/projects"] {
        sitemap_entry(&mut xml, &format!("{base}{p}"), None);
    }
    for (s,) in &cats {
        sitemap_entry(&mut xml, &format!("{base}/c/{}", percent_encode(s)), None);
    }
    for (id, m) in &topics {
        sitemap_entry(&mut xml, &format!("{base}/t/{id}"), Some(&m.format("%Y-%m-%d").to_string()));
    }
    xml.push_str("</urlset>\n");
    Ok((
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=3600")],
        xml,
    )
        .into_response())
}

async fn robots(State(state): State<AppState>) -> Response {
    let body = format!(
        "User-agent: *\nAllow: /\nDisallow: /admin\nDisallow: /api/\nDisallow: /new\nDisallow: /settings\nDisallow: /search\n\nSitemap: {}/sitemap.xml\n",
        state.config.public_url
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=3600")], body)
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"><title>超级小伴</title><meta name="description" content="x"><link rel="canonical" href="https://old/"><meta property="og:title" content="old"></head><body><div id="app"></div><script type="module" src="/assets/index-abc.js"></script></body></html>"#;

    fn page() -> Page {
        Page {
            title: "测试 <b> \"帖\" - 超级小伴".into(),
            description: "描述 & \"引号\"".into(),
            path: "/t/1".into(),
            og_type: "article",
            body: "<h1>正文</h1>".into(),
            json_ld: Some(json_ld_script(&json!({ "a": "</script><b>" }))),
        }
    }

    #[test]
    fn escape_covers_html_specials() {
        assert_eq!(escape("<script>alert(\"x\")</script> & 'y'"), "&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; &amp; &#39;y&#39;");
        assert_eq!(escape("普通文本"), "普通文本");
    }

    #[test]
    fn percent_encode_keeps_unreserved_only() {
        assert_eq!(percent_encode("abc-1.2_3~"), "abc-1.2_3~");
        assert_eq!(percent_encode("a b/c"), "a%20b%2Fc");
        assert_eq!(percent_encode("羊毛"), "%E7%BE%8A%E6%AF%9B");
        assert_eq!(percent_encode("c++"), "c%2B%2B");
    }

    #[test]
    fn truncate_collapses_and_limits() {
        assert_eq!(truncate_chars("a  b\n c", 10), "a b c");
        let t = truncate_chars(&"字".repeat(200), 150);
        assert_eq!(t.chars().count(), 150);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn inject_replaces_head_and_fills_app() {
        let out = inject(SAMPLE, "超级小伴", "https://chat.xiangban.xyz", &page());
        // 标题被替换且转义
        assert!(out.contains("<title>测试 &lt;b&gt; &quot;帖&quot; - 超级小伴</title>"));
        assert_eq!(out.matches("<title>").count(), 1);
        // 旧的 description / canonical / og 都被去掉，只剩一份新的
        assert_eq!(out.matches("name=\"description\"").count(), 1);
        assert!(out.contains("<meta name=\"description\" content=\"描述 &amp; &quot;引号&quot;\">"));
        assert_eq!(out.matches("rel=\"canonical\"").count(), 1);
        assert!(out.contains("<link rel=\"canonical\" href=\"https://chat.xiangban.xyz/t/1\">"));
        assert!(!out.contains("https://old/"));
        assert_eq!(out.matches("property=\"og:title\"").count(), 1);
        for p in ["og:description", "og:type\" content=\"article", "og:url\" content=\"https://chat.xiangban.xyz/t/1", "og:site_name\" content=\"超级小伴"] {
            assert!(out.contains(p), "缺少 {p}");
        }
        // 其余 head 内容保持不变
        assert!(out.contains("<meta charset=\"UTF-8\">"));
        assert!(out.contains("src=\"/assets/index-abc.js\""));
        // #app 填入预渲染内容，且 JSON-LD 里的 </script> 不会提前结束
        assert!(out.contains("<div id=\"app\"><h1>正文</h1></div>"));
        assert!(out.contains("application/ld+json"));
        assert!(!out.contains("</script><b>"));
        assert!(out.contains("\\u003c/script>"));
    }

    #[test]
    fn inject_adds_tags_when_missing() {
        let bare = "<html><head></head><body><div id=\"app\"></div></body></html>";
        let out = inject(bare, "站", "https://x.test", &page());
        assert!(out.contains("<title>"));
        assert!(out.contains("name=\"description\""));
        assert!(out.contains("rel=\"canonical\""));
        assert!(out.contains("<div id=\"app\"><h1>正文</h1></div>"));
    }

    #[test]
    fn noindex_page_keeps_shell_and_adds_robots() {
        let out = inject_noindex("<head><meta name=\"robots\" content=\"all\"></head><div id=\"app\"></div>");
        assert_eq!(out.matches("name=\"robots\"").count(), 1);
        assert!(out.contains("<meta name=\"robots\" content=\"noindex\">"));
        assert!(out.contains("<div id=\"app\"></div>"));
    }

    #[test]
    fn fill_app_without_mount_point_is_noop() {
        assert_eq!(fill_app("<body></body>", "x"), "<body></body>");
    }
}

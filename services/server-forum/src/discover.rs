//! 手机端社区的发现页内容：日报、头图、开源热榜、模型榜、追踪、导航、资料导读和 AI 工具。
//!
//! 这些条目都由后台手录（`/api/admin/discover/{kind}/{key}`），统一存在 `discover_items` 里，
//! `data` 的形状按 `kind` 在写入时校验；读取接口再按页面需要组装。

use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

pub const KINDS: &[&str] = &["banner", "news", "nav", "doc", "vendor", "model", "repo_issue", "quota", "launch", "promo", "tool"];

/// 模型榜分档：(档位, 名称, 下限)。阈值由后端给出，客户端只负责显示。
pub const TIERS: &[(&str, &str, f64)] =
    &[("S", "第一梯队", 70.0), ("A", "第二梯队", 63.0), ("B", "第三梯队", 56.0), ("C", "第四梯队", f64::NEG_INFINITY)];

pub fn tier_for(score: f64) -> &'static str {
    TIERS.iter().find(|(_, _, min)| score >= *min).map_or("C", |t| t.0)
}

pub fn is_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

/// 限时免费的进度：开始到结束之间走了多少（0～100）。
pub fn promo_progress(start: DateTime<Utc>, end: DateTime<Utc>, now: DateTime<Utc>) -> u8 {
    let total = (end - start).num_seconds();
    if total <= 0 {
        return 100;
    }
    let done = (now - start).num_seconds().clamp(0, total);
    (done * 100 / total) as u8
}

fn clean(raw: &str, what: &str, max: usize, required: bool) -> AppResult<String> {
    let t = raw.trim().to_string();
    if required && t.is_empty() {
        return Err(AppError::BadRequest(format!("{what}不能为空")));
    }
    if t.chars().count() > max {
        return Err(AppError::BadRequest(format!("{what}最多 {max} 个字")));
    }
    Ok(t)
}

fn web_url(raw: &str, what: &str, required: bool) -> AppResult<String> {
    let t = clean(raw, what, 500, required)?;
    if !t.is_empty() && !(t.starts_with("https://") || t.starts_with("http://")) {
        return Err(AppError::BadRequest(format!("{what}要以 http:// 或 https:// 开头")));
    }
    Ok(t)
}

fn color(raw: &str, what: &str) -> AppResult<String> {
    let t = raw.trim();
    let ok = t.len() == 7 && t.starts_with('#') && t[1..].chars().all(|c| c.is_ascii_hexdigit());
    if !ok {
        return Err(AppError::BadRequest(format!("{what}要写成 #RRGGBB")));
    }
    Ok(t.to_uppercase())
}

fn check_len<T>(items: &[T], what: &str, max: usize) -> AppResult<()> {
    if items.len() > max {
        return Err(AppError::BadRequest(format!("{what}最多 {max} 项")));
    }
    Ok(())
}

trait Check {
    fn check(&mut self) -> AppResult<()>;
}

#[derive(Deserialize, Serialize)]
pub struct Banner {
    #[serde(default)]
    tag: String,
    title: String,
    #[serde(default)]
    byline: String,
    /// 站内落点（`topic:12`、`pack:3`、`tool:cc-switch`）或 https 外链。
    #[serde(default)]
    link: String,
}

impl Check for Banner {
    fn check(&mut self) -> AppResult<()> {
        self.tag = clean(&self.tag, "标签", 12, false)?;
        self.title = clean(&self.title, "标题", 60, true)?;
        self.byline = clean(&self.byline, "署名", 40, false)?;
        self.link = clean(&self.link, "链接", 300, false)?;
        let internal = ["topic:", "pack:", "tool:", "weekly:"].iter().any(|p| self.link.starts_with(p));
        if !self.link.is_empty() && !internal {
            self.link = web_url(&self.link, "链接", false)?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct News {
    title: String,
    source: String,
    url: String,
    #[serde(default)]
    cover_url: String,
}

impl Check for News {
    fn check(&mut self) -> AppResult<()> {
        self.title = clean(&self.title, "标题", 80, true)?;
        self.source = clean(&self.source, "来源", 30, true)?;
        self.url = web_url(&self.url, "链接", true)?;
        self.cover_url = web_url(&self.cover_url, "封面地址", false)?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Nav {
    group: String,
    name: String,
    vendor: String,
    url: String,
    brand_color: String,
    tint: String,
}

impl Check for Nav {
    fn check(&mut self) -> AppResult<()> {
        self.group = clean(&self.group, "分组", 10, true)?;
        self.name = clean(&self.name, "名称", 20, true)?;
        self.vendor = clean(&self.vendor, "厂商", 20, true)?;
        self.url = web_url(&self.url, "官方入口", true)?;
        self.brand_color = color(&self.brand_color, "品牌色")?;
        self.tint = color(&self.tint, "底色")?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Doc {
    product: String,
    title: String,
    #[serde(default)]
    subtitle: String,
    /// `guide`（导读）或 `index`（索引）。
    label: String,
    /// 官方原文地址，点开直接跳转官网。
    url: String,
}

impl Check for Doc {
    fn check(&mut self) -> AppResult<()> {
        self.product = clean(&self.product, "产品", 20, true)?;
        self.title = clean(&self.title, "标题", 60, true)?;
        self.subtitle = clean(&self.subtitle, "说明", 80, false)?;
        if !["guide", "index"].contains(&self.label.as_str()) {
            return Err(AppError::BadRequest("标签只能是 guide 或 index".into()));
        }
        self.url = web_url(&self.url, "官方原文", true)?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Vendor {
    name: String,
    brand_color: String,
    tint: String,
}

impl Check for Vendor {
    fn check(&mut self) -> AppResult<()> {
        self.name = clean(&self.name, "厂商名", 20, true)?;
        self.brand_color = color(&self.brand_color, "品牌色")?;
        self.tint = color(&self.tint, "底色")?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Model {
    name: String,
    /// 厂商条目的 key。
    vendor: String,
    /// 智能指数、编程、数学、智能体：0～100，越高越好。
    iq: f64,
    #[serde(default)]
    code: f64,
    #[serde(default)]
    math: f64,
    #[serde(default)]
    agent: f64,
    /// 混合价格，美元/百万 token，越低越好。
    price: f64,
    /// 输出速度，token/秒。
    speed: f64,
    /// 首字延迟，秒。
    #[serde(default)]
    latency: f64,
    #[serde(default)]
    context: String,
    #[serde(default)]
    open: bool,
    #[serde(default)]
    reasoning: bool,
    #[serde(default)]
    cn: bool,
}

impl Check for Model {
    fn check(&mut self) -> AppResult<()> {
        self.name = clean(&self.name, "模型名", 40, true)?;
        self.vendor = clean(&self.vendor, "厂商", 40, true)?;
        self.context = clean(&self.context, "上下文", 12, false)?;
        for (what, v) in [("智能指数", self.iq), ("编程", self.code), ("数学", self.math), ("智能体", self.agent)] {
            if !(0.0..=100.0).contains(&v) {
                return Err(AppError::BadRequest(format!("{what}要在 0～100 之间")));
            }
        }
        for (what, v) in [("价格", self.price), ("速度", self.speed), ("延迟", self.latency)] {
            if !v.is_finite() || v < 0.0 {
                return Err(AppError::BadRequest(format!("{what}不能是负数")));
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct RepoRank {
    full_name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    language: String,
    #[serde(default)]
    language_color: String,
    star_total: i64,
    star_gain: i64,
    #[serde(default)]
    spark: Vec<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct RepoIssue {
    issue_no: i32,
    week_start: String,
    week_end: String,
    total_gain: i64,
    #[serde(default)]
    wow_pct: f64,
    #[serde(default)]
    daily_gain: Vec<i64>,
    ranks: Vec<RepoRank>,
}

impl Check for RepoIssue {
    fn check(&mut self) -> AppResult<()> {
        if self.issue_no < 1 {
            return Err(AppError::BadRequest("期数要从 1 开始".into()));
        }
        self.week_start = clean(&self.week_start, "开始日期", 12, true)?;
        self.week_end = clean(&self.week_end, "结束日期", 12, true)?;
        if self.daily_gain.len() != 7 && !self.daily_gain.is_empty() {
            return Err(AppError::BadRequest("每日新增要填 7 个数".into()));
        }
        check_len(&self.ranks, "榜单", 50)?;
        for r in &mut self.ranks {
            r.full_name = clean(&r.full_name, "仓库名", 80, true)?;
            if !r.full_name.contains('/') {
                return Err(AppError::BadRequest("仓库名要写成 owner/repo".into()));
            }
            r.description = clean(&r.description, "简介", 120, false)?;
            r.language = clean(&r.language, "语言", 20, false)?;
            if !r.language_color.is_empty() {
                r.language_color = color(&r.language_color, "语言颜色")?;
            }
            if r.spark.len() > 14 {
                return Err(AppError::BadRequest("趋势最多 14 个点".into()));
            }
        }
        self.ranks.sort_by(|a, b| b.star_gain.cmp(&a.star_gain));
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Quota {
    /// 本轮额度已过去的比例，0～100。
    progress_pct: u8,
    next_reset_at: DateTime<Utc>,
    #[serde(default)]
    recent_intervals: Vec<f64>,
}

impl Check for Quota {
    fn check(&mut self) -> AppResult<()> {
        if self.progress_pct > 100 {
            return Err(AppError::BadRequest("进度要在 0～100 之间".into()));
        }
        check_len(&self.recent_intervals, "重置间隔", 14)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Launch {
    title: String,
    #[serde(default)]
    tags: Vec<String>,
}

impl Check for Launch {
    fn check(&mut self) -> AppResult<()> {
        self.title = clean(&self.title, "标题", 80, true)?;
        check_len(&self.tags, "标签", 5)?;
        for t in &mut self.tags {
            *t = clean(t, "标签", 12, true)?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Promo {
    text: String,
    started_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

impl Check for Promo {
    fn check(&mut self) -> AppResult<()> {
        self.text = clean(&self.text, "说明", 80, true)?;
        if self.ends_at <= self.started_at {
            return Err(AppError::BadRequest("结束时间要晚于开始时间".into()));
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct ReleaseItem {
    /// `new`（新增）、`improve`（优化）、`fix`（修复）。
    kind: String,
    text: String,
}

#[derive(Deserialize, Serialize)]
pub struct Release {
    version: String,
    date: String,
    #[serde(default)]
    items: Vec<ReleaseItem>,
}

#[derive(Deserialize, Serialize)]
pub struct Artifact {
    /// `windows`、`macos`、`linux`。
    os: String,
    /// 文件说明，例如 ".msi 安装包"。
    desc: String,
    /// 字节数。
    size: i64,
    #[serde(default)]
    sha256: String,
    /// 小伴镜像地址；没有镜像就留空，客户端会隐藏这个来源。
    #[serde(default)]
    mirror_url: String,
    #[serde(default)]
    github_url: String,
}

#[derive(Deserialize, Serialize)]
pub struct Tool {
    name: String,
    developer: String,
    /// 例如 "桌面应用"。
    #[serde(default)]
    type_label: String,
    /// `open`、`closed`、`unknown`。
    license: String,
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    homepage_url: String,
    #[serde(default)]
    icon_url: String,
    /// 用这个标签统计相关讨论数。
    #[serde(default)]
    discussion_tag: String,
    #[serde(default)]
    releases: Vec<Release>,
    #[serde(default)]
    artifacts: Vec<Artifact>,
}

impl Check for Tool {
    fn check(&mut self) -> AppResult<()> {
        self.name = clean(&self.name, "名称", 40, true)?;
        self.developer = clean(&self.developer, "开发者", 40, true)?;
        self.type_label = clean(&self.type_label, "类型", 20, false)?;
        if !["open", "closed", "unknown"].contains(&self.license.as_str()) {
            return Err(AppError::BadRequest("许可证只能是 open、closed 或 unknown".into()));
        }
        self.description = clean(&self.description, "简介", 200, false)?;
        self.homepage_url = web_url(&self.homepage_url, "项目主页", false)?;
        self.icon_url = web_url(&self.icon_url, "图标地址", false)?;
        self.discussion_tag = clean(&self.discussion_tag, "讨论标签", 20, false)?;
        check_len(&self.platforms, "平台", 6)?;
        check_len(&self.releases, "更新记录", 30)?;
        check_len(&self.artifacts, "安装包", 12)?;
        for r in &mut self.releases {
            r.version = clean(&r.version, "版本号", 20, true)?;
            r.date = clean(&r.date, "日期", 12, true)?;
            check_len(&r.items, "更新条目", 20)?;
            for i in &mut r.items {
                if !["new", "improve", "fix"].contains(&i.kind.as_str()) {
                    return Err(AppError::BadRequest("更新类型只能是 new、improve 或 fix".into()));
                }
                i.text = clean(&i.text, "更新内容", 120, true)?;
            }
        }
        for a in &mut self.artifacts {
            if !["windows", "macos", "linux"].contains(&a.os.as_str()) {
                return Err(AppError::BadRequest("系统只能是 windows、macos 或 linux".into()));
            }
            a.desc = clean(&a.desc, "文件说明", 60, true)?;
            if a.size < 0 {
                return Err(AppError::BadRequest("大小不能是负数".into()));
            }
            a.sha256 = a.sha256.trim().to_lowercase();
            if !a.sha256.is_empty() && !(a.sha256.len() == 64 && a.sha256.chars().all(|c| c.is_ascii_hexdigit())) {
                return Err(AppError::BadRequest("SHA-256 要是 64 位十六进制".into()));
            }
            a.mirror_url = web_url(&a.mirror_url, "小伴镜像地址", false)?;
            a.github_url = web_url(&a.github_url, "GitHub 地址", false)?;
        }
        Ok(())
    }
}

fn go<T: DeserializeOwned + Serialize + Check>(data: Value) -> AppResult<Value> {
    let mut v: T = serde_json::from_value(data).map_err(|e| AppError::BadRequest(format!("内容格式不对：{e}")))?;
    v.check()?;
    serde_json::to_value(v).map_err(|e| AppError::Internal(e.to_string()))
}

/// 校验并整理某个类型的条目内容，只保留已知字段。
pub fn validate(kind: &str, data: Value) -> AppResult<Value> {
    match kind {
        "banner" => go::<Banner>(data),
        "news" => go::<News>(data),
        "nav" => go::<Nav>(data),
        "doc" => go::<Doc>(data),
        "vendor" => go::<Vendor>(data),
        "model" => go::<Model>(data),
        "repo_issue" => go::<RepoIssue>(data),
        "quota" => go::<Quota>(data),
        "launch" => go::<Launch>(data),
        "promo" => go::<Promo>(data),
        "tool" => go::<Tool>(data),
        _ => Err(AppError::BadRequest("未知的类型".into())),
    }
}

/// 条目 key：小写字母、数字、连字符、下划线。
pub fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 60 && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// 给已有的帖子补列表配图：从正文里取站内图片，只做一次。
pub async fn backfill_topic_images(pool: &PgPool) -> AppResult<()> {
    const NAME: &str = "topic-images-v1";
    let done: Option<(String,)> = sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(NAME).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }
    let rows: Vec<(i64, String)> = sqlx::query_as("SELECT id, body_md FROM topics WHERE body_md LIKE '%/api/images/%'").fetch_all(pool).await?;
    let mut tx = pool.begin().await?;
    for (id, body) in rows {
        sqlx::query("UPDATE topics SET images = $2 WHERE id = $1").bind(id).bind(crate::markdown::site_images(&body, 3)).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO data_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING").bind(NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

const SEED_NAME: &str = "discover-seed-v1";

/// 第一次启动时写入示例内容，让页面有东西可看。条目带 `sample: true`，客户端会标「示例数据」；
/// 后台删改后不会再被写回，真实内容录入后把对应示例删掉即可。
pub async fn seed(pool: &PgPool) -> AppResult<()> {
    let done: Option<(String,)> =
        sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(SEED_NAME).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    for (kind, key, order, data) in seed_items() {
        let mut data = validate(kind, data).map_err(|e| AppError::Internal(format!("示例 {kind}/{key} 无效：{e}")))?;
        data["sample"] = Value::Bool(sample_flag(kind));
        sqlx::query("INSERT INTO discover_items (kind, key, sort_order, data) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
            .bind(kind)
            .bind(key)
            .bind(order)
            .bind(data)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO data_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING").bind(SEED_NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

/// 导航里是各家的官方入口，是真实内容，其余都是示例。
fn sample_flag(kind: &str) -> bool {
    !matches!(kind, "nav" | "doc")
}

fn seed_items() -> Vec<(&'static str, &'static str, i32, Value)> {
    use serde_json::json;
    let now = Utc::now();
    let mut v: Vec<(&'static str, &'static str, i32, Value)> = vec![
        ("banner", "weekly-1", 0, json!({ "tag": "本周原创", "title": "示例：一周用下来，三款编程助手各适合谁", "byline": "小伴编辑部 · 8 分钟读完" })),
        ("news", "demo-1", 0, json!({ "title": "示例：某编程助手发布新版本，支持多项目并行", "source": "示例来源", "url": "https://chat.xiangban.xyz/" })),
        ("news", "demo-2", 1, json!({ "title": "示例：开源语音识别模型更新，中文准确率提升", "source": "示例来源", "url": "https://chat.xiangban.xyz/" })),
        ("news", "demo-3", 2, json!({ "title": "示例：一份可复用的团队 AI 使用规范", "source": "示例来源", "url": "https://chat.xiangban.xyz/" })),
        ("vendor", "demo-a", 0, json!({ "name": "示例厂商 A", "brand_color": "#2F6BFF", "tint": "#E4ECFF" })),
        ("vendor", "demo-b", 1, json!({ "name": "示例厂商 B", "brand_color": "#E3203C", "tint": "#FFE4E8" })),
        ("vendor", "demo-c", 2, json!({ "name": "示例厂商 C", "brand_color": "#2E6B47", "tint": "#E6F4EA" })),
        ("quota", "codex", 0, json!({ "progress_pct": 68, "next_reset_at": now + chrono::Duration::hours(3), "recent_intervals": [5.0, 5.0, 4.5, 5.0, 5.5, 5.0, 5.0] })),
        ("launch", "demo-1", 0, json!({ "title": "示例：某厂商发布新一代推理模型", "tags": ["API", "编程"] })),
        ("launch", "demo-2", 1, json!({ "title": "示例：7B 开源模型发布", "tags": ["开源", "7B"] })),
        ("promo", "demo-1", 0, json!({ "text": "示例：某编程助手限时免费试用", "started_at": now - chrono::Duration::days(4), "ends_at": now + chrono::Duration::days(3) })),
        ("tool", "cc-switch", 0, json!({
            "name": "CC Switch", "developer": "开源社区", "type_label": "桌面应用", "license": "open",
            "platforms": ["Windows", "macOS", "Linux"],
            "description": "在 Claude Code、Codex 等编程工具之间切换服务商配置，不用手改配置文件。",
            "homepage_url": "https://github.com/farion1231/cc-switch", "discussion_tag": "cc-switch",
            "releases": [{ "version": "v0.0.0", "date": "示例", "items": [{ "kind": "new", "text": "示例更新内容" }] }],
            "artifacts": [
                { "os": "windows", "desc": ".msi 安装包", "size": 12582912, "github_url": "https://github.com/farion1231/cc-switch/releases" },
                { "os": "macos", "desc": ".dmg（Apple 芯片 / Intel）", "size": 14680064, "github_url": "https://github.com/farion1231/cc-switch/releases" },
                { "os": "linux", "desc": ".AppImage / .deb", "size": 16777216, "github_url": "https://github.com/farion1231/cc-switch/releases" }
            ]
        })),
    ];
    for (i, (name, v_key, iq, price, speed)) in [
        ("示例模型 A", "demo-a", 72.0, 4.0, 90.0),
        ("示例模型 B", "demo-b", 66.0, 1.2, 140.0),
        ("示例模型 C", "demo-c", 58.0, 0.3, 210.0),
        ("示例模型 D", "demo-a", 51.0, 0.15, 260.0),
    ]
    .into_iter()
    .enumerate()
    {
        let key: &'static str = ["demo-a", "demo-b", "demo-c", "demo-d"][i];
        v.push(("model", key, i as i32, json!({
            "name": name, "vendor": v_key, "iq": iq, "code": iq - 3.0, "math": iq - 6.0, "agent": iq - 2.0,
            "price": price, "speed": speed, "latency": 0.9, "context": "200K", "open": i == 2, "reasoning": i < 2, "cn": i == 3,
        })));
    }
    v.push(("repo_issue", "1", 0, json!({
        "issue_no": 1, "week_start": "示例", "week_end": "示例", "total_gain": 12345, "wow_pct": 0.0,
        "daily_gain": [1200, 1500, 1800, 2100, 2000, 1900, 1845],
        "ranks": [
            { "full_name": "example-org/project-a", "description": "示例项目 A", "language": "TypeScript", "language_color": "#3178C6", "star_total": 12400, "star_gain": 3214, "spark": [300, 380, 450, 500, 520, 540, 524] },
            { "full_name": "example-org/project-b", "description": "示例项目 B", "language": "Python", "language_color": "#3572A5", "star_total": 8700, "star_gain": 2100, "spark": [200, 260, 300, 330, 340, 350, 320] },
            { "full_name": "example-org/project-c", "description": "示例项目 C", "language": "Rust", "language_color": "#DEA584", "star_total": 5100, "star_gain": 1500, "spark": [150, 180, 210, 230, 250, 240, 240] }
        ]
    })));
    for (i, (group, name, vendor, url, c, t)) in [
        ("海外", "Claude", "Anthropic", "https://claude.ai/", "#D97757", "#FBE9E1"),
        ("海外", "ChatGPT", "OpenAI", "https://chatgpt.com/", "#10A37F", "#DDF3EC"),
        ("海外", "Gemini", "Google", "https://gemini.google.com/", "#4285F4", "#E4EEFF"),
        ("国产", "DeepSeek", "深度求索", "https://chat.deepseek.com/", "#4D6BFE", "#E6EBFF"),
        ("国产", "Kimi", "月之暗面", "https://www.kimi.com/", "#222222", "#E8E8E8"),
        ("国产", "豆包", "字节跳动", "https://www.doubao.com/", "#2F6BFF", "#E4ECFF"),
        ("国产", "通义", "阿里云", "https://tongyi.aliyun.com/", "#615CED", "#EAE9FD"),
        ("国产", "智谱清言", "智谱 AI", "https://chatglm.cn/", "#3D5AFE", "#E6EAFF"),
    ]
    .into_iter()
    .enumerate()
    {
        let key: &'static str = ["claude", "chatgpt", "gemini", "deepseek", "kimi", "doubao", "tongyi", "chatglm"][i];
        v.push(("nav", key, i as i32, json!({ "group": group, "name": name, "vendor": vendor, "url": url, "brand_color": c, "tint": t })));
    }
    for (i, (key, product, title, sub, label, url)) in [
        ("claude-code", "Claude", "Claude Code 文档", "安装、配置和常用命令", "guide", "https://docs.claude.com/"),
        ("codex", "Codex", "Codex 文档", "命令行与云端任务", "guide", "https://developers.openai.com/codex"),
    ]
    .into_iter()
    .enumerate()
    {
        v.push(("doc", key, i as i32, json!({ "product": product, "title": title, "subtitle": sub, "label": label, "url": url })));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tiers_follow_thresholds() {
        assert_eq!(tier_for(72.0), "S");
        assert_eq!(tier_for(70.0), "S");
        assert_eq!(tier_for(69.9), "A");
        assert_eq!(tier_for(63.0), "A");
        assert_eq!(tier_for(56.0), "B");
        assert_eq!(tier_for(10.0), "C");
    }

    #[test]
    fn promo_progress_is_clamped() {
        let start = Utc::now() - chrono::Duration::days(1);
        let end = start + chrono::Duration::days(4);
        assert_eq!(promo_progress(start, end, start - chrono::Duration::days(1)), 0);
        assert_eq!(promo_progress(start, end, start + chrono::Duration::days(1)), 25);
        assert_eq!(promo_progress(start, end, end + chrono::Duration::days(9)), 100);
        assert_eq!(promo_progress(end, start, start), 100);
    }

    #[test]
    fn every_seed_item_is_valid() {
        for (kind, key, _, data) in seed_items() {
            assert!(valid_key(key), "{kind}/{key}");
            validate(kind, data).unwrap_or_else(|e| panic!("{kind}/{key}: {e}"));
        }
    }

    #[test]
    fn repo_ranks_are_sorted_by_gain() {
        let data = json!({
            "issue_no": 2, "week_start": "10.23", "week_end": "10.29", "total_gain": 10,
            "ranks": [
                { "full_name": "a/low", "star_total": 1, "star_gain": 5 },
                { "full_name": "b/high", "star_total": 1, "star_gain": 50 },
            ]
        });
        let out = validate("repo_issue", data).unwrap();
        assert_eq!(out["ranks"][0]["full_name"], "b/high");
    }

    #[test]
    fn rejects_bad_input() {
        assert!(validate("nav", json!({ "group": "海外", "name": "X", "vendor": "Y", "url": "javascript:alert(1)", "brand_color": "#000000", "tint": "#FFFFFF" })).is_err());
        assert!(validate("model", json!({ "name": "m", "vendor": "v", "iq": 120, "price": 1, "speed": 1 })).is_err());
        assert!(validate("tool", json!({ "name": "t", "developer": "d", "license": "mit" })).is_err());
        assert!(validate("tool", json!({ "name": "t", "developer": "d", "license": "open", "artifacts": [{ "os": "windows", "desc": "x", "size": 1, "sha256": "zz" }] })).is_err());
        assert!(validate("banner", json!({ "title": "t", "link": "ftp://x" })).is_err());
        assert!(validate("nope", json!({})).is_err());
        assert!(!valid_key("Bad Key"));
    }

    #[test]
    fn unknown_fields_are_dropped() {
        let out = validate("vendor", json!({ "name": "n", "brand_color": "#abcdef", "tint": "#ffffff", "evil": 1 })).unwrap();
        assert!(out.get("evil").is_none());
        assert_eq!(out["brand_color"], "#ABCDEF");
    }
}

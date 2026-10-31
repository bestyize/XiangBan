//! 手机端社区的发现页内容：日报、头图、开源热榜、模型榜、追踪、导航、资料导读和 AI 工具。
//!
//! 这些条目都由后台手录（`/api/admin/discover/{kind}/{key}`），统一存在 `discover_items` 里，
//! `data` 的形状按 `kind` 在写入时校验；读取接口再按页面需要组装。

use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

pub const KINDS: &[&str] = &["banner", "news", "nav", "doc", "vendor", "model", "repo_issue", "quota", "launch", "promo", "tool", "hotword"];

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
    /// 没有封面图时，缩略图和来源小圆点用的颜色（#RRGGBB）。
    #[serde(default)]
    accent: String,
}

impl Check for News {
    fn check(&mut self) -> AppResult<()> {
        self.title = clean(&self.title, "标题", 80, true)?;
        self.source = clean(&self.source, "来源", 30, true)?;
        self.url = web_url(&self.url, "链接", true)?;
        self.cover_url = web_url(&self.cover_url, "封面地址", false)?;
        if !self.accent.is_empty() {
            self.accent = color(&self.accent, "强调色")?;
        }
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
    /// 图标块里的小图标：`term`、`code`、`doc`、`mic`、`bolt`，认不出的客户端显示仓库名首字。
    #[serde(default)]
    icon: String,
    /// 图标块的底色（#RRGGBB）。
    #[serde(default)]
    tint: String,
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
            r.icon = clean(&r.icon, "图标", 12, false)?;
            if !r.tint.is_empty() {
                r.tint = color(&r.tint, "图标底色")?;
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
pub struct Hotword {
    text: String,
    /// 右侧的小红字，如「热」「新」，可空。
    #[serde(default)]
    tag: String,
}

impl Check for Hotword {
    fn check(&mut self) -> AppResult<()> {
        self.text = clean(&self.text, "热词", 20, true)?;
        self.tag = clean(&self.tag, "标记", 4, false)?;
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
        "hotword" => go::<Hotword>(data),
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

const SEED_NAME: &str = "discover-seed-v2";

/// 第一次启动时写入示例内容（数据取自设计稿），让页面有东西可看。带 `sample: true` 的条目客户端会标「示例数据」；
/// 导航和资料是真实的官方入口。已部署的库升级时，先清掉上一版写的示例条目再写新的；后台自己录入的真实内容不受影响。
/// 做完记进 `data_migrations`，后台删改后不会再被写回。
pub async fn seed(pool: &PgPool) -> AppResult<()> {
    let done: Option<(String,)> =
        sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(SEED_NAME).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    sqlx::query(
        "DELETE FROM discover_items WHERE data->>'sample' = 'true'
            OR (kind = 'doc' AND key IN ('claude-code', 'codex')) OR kind = 'hotword'",
    )
    .execute(&mut *tx)
    .await?;
    for (kind, key, order, age_minutes, data) in seed_items() {
        let mut data = validate(kind, data).map_err(|e| AppError::Internal(format!("示例 {kind}/{key} 无效：{e}")))?;
        data["sample"] = Value::Bool(sample_flag(kind));
        sqlx::query(
            "INSERT INTO discover_items (kind, key, sort_order, published_at, data)
             VALUES ($1, $2, $3, now() - make_interval(mins => $4), $5) ON CONFLICT DO NOTHING",
        )
        .bind(kind)
        .bind(key)
        .bind(order)
        .bind(age_minutes)
        .bind(data)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("INSERT INTO data_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING").bind(SEED_NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

/// 导航、资料和热词是真实内容，其余都是示例。
fn sample_flag(kind: &str) -> bool {
    !matches!(kind, "nav" | "doc" | "hotword")
}

type SeedItem = (&'static str, &'static str, i32, i32, Value);

fn seed_items() -> Vec<SeedItem> {
    use serde_json::json;
    let now = Utc::now();
    let site = "https://chat.xiangban.xyz/";
    let mut v: Vec<SeedItem> = vec![
        ("banner", "weekly-1", 0, 0, json!({ "tag": "本周原创", "title": "一周用下来，三款编程助手各适合谁", "byline": "小伴编辑部 · 8 分钟读完" })),
        ("quota", "codex", 0, 5, json!({ "progress_pct": 68, "next_reset_at": now + chrono::Duration::minutes(192), "recent_intervals": [5.0, 5.5, 4.0, 5.0, 6.0, 5.0, 4.5] })),
        ("launch", "demo-1", 0, 15, json!({ "title": "示例模型 G 发布", "tags": ["API", "编程"] })),
        ("launch", "demo-2", 1, 28 * 60, json!({ "title": "示例模型 H 开源", "tags": ["开源", "7B"] })),
        ("launch", "demo-3", 2, 96 * 60, json!({ "title": "示例模型 I 上线网页版", "tags": ["免费试用"] })),
        ("promo", "demo-1", 0, 0, json!({ "text": "示例工具：免费额度翻倍", "started_at": now - chrono::Duration::days(4), "ends_at": now + chrono::Duration::days(3) })),
        ("promo", "demo-2", 1, 0, json!({ "text": "示例模型：开放免费试用", "started_at": now - chrono::Duration::days(3), "ends_at": now + chrono::Duration::days(12) })),
    ];
    for (i, (title, source, accent, age)) in [
        ("示例：某厂商发布新一代编程模型，长任务更稳定", "示例媒体", "#3E5C8A", 120),
        ("示例：开源语音模型更新，手机上也能实时转写", "示例媒体", "#F4A6A0", 240),
        ("示例：三家 AI 公司同日调整订阅价格", "示例媒体", "#D9C7A8", 360),
        ("示例：新的智能体框架发布，主打本地运行", "示例媒体", "#4F7A5E", 480),
        ("示例：国产 AI 芯片出货量季度数据公布", "示例媒体", "#9FB6DC", 26 * 60),
    ]
    .into_iter()
    .enumerate()
    {
        let key: &'static str = ["demo-1", "demo-2", "demo-3", "demo-4", "demo-5"][i];
        v.push(("news", key, i as i32, age, json!({ "title": title, "source": source, "url": site, "accent": accent })));
    }
    // 厂商和模型：设计稿里的 6 家厂商、12 个示例模型。
    for (i, (key, name, c, t)) in [
        ("claude", "Claude", "#D97757", "#FBEFEA"),
        ("gemini", "Gemini", "#1A73E8", "#E8F0FE"),
        ("deepseek", "DeepSeek", "#5786FE", "#EEF2FF"),
        ("qwen", "Qwen", "#6950EF", "#EFECFD"),
        ("kimi", "Kimi", "#111111", "#F0F0F0"),
        ("minimax", "MiniMax", "#E73562", "#FDE8EE"),
    ]
    .into_iter()
    .enumerate()
    {
        v.push(("vendor", key, i as i32, 0, json!({ "name": name, "brand_color": c, "tint": t })));
    }
    // (id, 厂商, 智能, 编程, 数学, 智能体, 价格, 速度, 延迟, 上下文, 开源, 推理, 国产)
    let models: [(&'static str, &'static str, f64, f64, f64, f64, f64, f64, f64, &str, bool, bool, bool); 12] = [
        ("a", "claude", 73.0, 70.0, 92.0, 68.0, 3.4, 182.0, 0.9, "1M", false, true, false),
        ("b", "gemini", 71.0, 72.0, 88.0, 66.0, 6.0, 95.0, 1.4, "400K", false, true, false),
        ("c", "deepseek", 70.0, 66.0, 90.0, 61.0, 1.1, 140.0, 0.6, "256K", true, true, true),
        ("d", "claude", 68.0, 64.0, 86.0, 63.0, 1.9, 210.0, 0.5, "1M", false, false, false),
        ("e", "qwen", 66.0, 62.0, 84.0, 58.0, 0.6, 88.0, 1.1, "128K", true, true, true),
        ("f", "kimi", 64.0, 60.0, 80.0, 55.0, 0.4, 120.0, 0.7, "128K", true, false, true),
        ("g", "gemini", 62.0, 63.0, 76.0, 57.0, 2.5, 160.0, 0.6, "200K", false, false, false),
        ("h", "minimax", 60.0, 55.0, 78.0, 50.0, 0.3, 230.0, 0.4, "128K", true, false, true),
        ("i", "deepseek", 57.0, 52.0, 70.0, 46.0, 0.2, 300.0, 0.3, "64K", true, false, true),
        ("j", "qwen", 55.0, 50.0, 68.0, 44.0, 0.9, 75.0, 1.5, "1M", false, true, true),
        ("k", "kimi", 52.0, 46.0, 64.0, 40.0, 0.15, 260.0, 0.3, "32K", true, false, true),
        ("l", "minimax", 49.0, 44.0, 58.0, 37.0, 0.1, 340.0, 0.2, "32K", true, false, true),
    ];
    for (i, (id, vendor, iq, code, math, agent, price, speed, latency, ctx, open, reasoning, cn)) in models.into_iter().enumerate() {
        let name = format!("模型 {}", id.to_uppercase());
        v.push(("model", id, i as i32, 0, json!({
            "name": name, "vendor": vendor, "iq": iq, "code": code, "math": math, "agent": agent, "price": price,
            "speed": speed, "latency": latency, "context": ctx, "open": open, "reasoning": reasoning, "cn": cn,
        })));
    }
    // 开源热榜第 42 期：(仓库, 简介, 语言, 语言色, 总 star, 周新增, 趋势, 图标, 底色)
    v.push(("repo_issue", "42", 0, 0, json!({
        "issue_no": 42, "week_start": "10.23", "week_end": "10.29", "total_gain": 86420, "wow_pct": 12.0,
        "daily_gain": [62, 70, 58, 76, 88, 94, 100],
        "ranks": [
            { "full_name": "example/agent-shell", "description": "把终端里反复敲的命令，变成一句话就能调用的小工具。", "language": "Rust", "language_color": "#D9A07A", "star_total": 12400, "star_gain": 3214, "spark": [30, 42, 50, 61, 70, 86, 100], "icon": "term", "tint": "#E8EEF8" },
            { "full_name": "example/calendar-mcp", "description": "让助手直接读写日历和待办，不用再来回复制粘贴。", "language": "TypeScript", "language_color": "#5B8DD6", "star_total": 8100, "star_gain": 2876, "spark": [55, 40, 62, 58, 80, 74, 92], "icon": "code", "tint": "#E6F4EA" },
            { "full_name": "example/whiteboard-ocr", "description": "手机拍一张白板照片，几秒转成可编辑的表格。", "language": "Python", "language_color": "#E8C547", "star_total": 5600, "star_gain": 2105, "spark": [20, 26, 40, 66, 90, 84, 78], "icon": "doc", "tint": "#FDE4E1" },
            { "full_name": "example/page-rag", "description": "给长文档做段落级检索，回答时标出依据的页码。", "language": "Python", "language_color": "#E8C547", "star_total": 3900, "star_gain": 1642, "spark": [60, 64, 58, 70, 66, 72, 80], "icon": "doc", "tint": "#F1ECE4" },
            { "full_name": "example/offline-asr", "description": "在本机离线运行的语音转文字，不上传录音。", "language": "C++", "language_color": "#C27BA0", "star_total": 9700, "star_gain": 1398, "spark": [80, 70, 72, 60, 64, 70, 76], "icon": "mic", "tint": "#EDE7F6" },
            { "full_name": "example/web-replay", "description": "把常用网页操作录成可重复运行的自动化流程。", "language": "TypeScript", "language_color": "#5B8DD6", "star_total": 2200, "star_gain": 1120, "spark": [10, 18, 30, 44, 52, 70, 88], "icon": "bolt", "tint": "#E8EEF8" }
        ]
    })));
    // AI 工具：设计稿里的 5 个。版本号和日期是示例，CC Switch 的详情页内容取自稿子。
    let gh = "https://github.com/farion1231/cc-switch/releases";
    v.push(("tool", "claude-code", 0, 0, json!({
        "name": "Claude Code Desktop", "developer": "Anthropic", "type_label": "桌面应用", "license": "closed", "platforms": ["Windows", "macOS"],
        "description": "Anthropic 官方的编程助手桌面版。",
        "releases": [{ "version": "示例版本", "date": "10 月 26 日", "items": [] }]
    })));
    v.push(("tool", "chatgpt-desktop", 1, 0, json!({
        "name": "ChatGPT Desktop", "developer": "OpenAI", "type_label": "桌面应用", "license": "closed", "platforms": ["Windows", "macOS"],
        "description": "OpenAI 官方的 ChatGPT 桌面版。",
        "releases": [{ "version": "示例版本", "date": "10 月 24 日", "items": [] }]
    })));
    v.push(("tool", "opencode", 2, 0, json!({
        "name": "OpenCode", "developer": "开源社区", "type_label": "命令行工具", "license": "open", "platforms": ["Windows", "macOS", "Linux"],
        "description": "开源的终端编程助手。",
        "releases": [{ "version": "示例版本", "date": "10 月 22 日", "items": [] }]
    })));
    v.push(("tool", "cc-switch", 3, 0, json!({
        "name": "CC Switch", "developer": "开源社区", "type_label": "桌面应用", "license": "open", "platforms": ["Windows", "macOS", "Linux"],
        "description": "在 Claude Code、Codex 等编程工具之间切换服务商配置，不用手改配置文件。",
        "homepage_url": "https://github.com/farion1231/cc-switch", "discussion_tag": "cc-switch",
        "releases": [
            { "version": "v3.5.0", "date": "10 月 20 日", "items": [{ "kind": "new", "text": "支持 Codex 配置一键切换" }, { "kind": "improve", "text": "切换后自动重启终端里的会话" }, { "kind": "fix", "text": "macOS 上托盘图标偶尔不显示" }] },
            { "version": "v3.4.2", "date": "10 月 8 日", "items": [{ "kind": "fix", "text": "导入配置时中文路径报错" }, { "kind": "improve", "text": "启动速度提升约 30%" }] },
            { "version": "v3.4.0", "date": "9 月 26 日", "items": [{ "kind": "new", "text": "配置分组和搜索" }, { "kind": "new", "text": "深色模式" }] }
        ],
        "artifacts": [
            { "os": "windows", "desc": ".msi 安装包", "size": 12582912, "github_url": gh },
            { "os": "macos", "desc": ".dmg（Apple 芯片 / Intel）", "size": 14680064, "github_url": gh },
            { "os": "linux", "desc": ".AppImage / .deb", "size": 16777216, "github_url": gh }
        ]
    })));
    v.push(("tool", "deepseek-harness", 4, 0, json!({
        "name": "DeepSeek harness", "developer": "深度求索", "type_label": "命令行工具", "license": "unknown", "platforms": [],
        "description": "许可证还在核实，核实后再标注。"
    })));
    for (i, (key, group, name, vendor, url, c, t)) in [
        ("claude", "海外", "Claude", "Anthropic", "https://claude.ai/", "#D97757", "#FBEFEA"),
        ("chatgpt", "海外", "ChatGPT", "OpenAI", "https://chatgpt.com/", "#222222", "#FFFFFF"),
        ("gemini", "海外", "Gemini", "Google", "https://gemini.google.com/", "#1A73E8", "#E8F0FE"),
        ("deepseek", "国产", "DeepSeek", "深度求索", "https://chat.deepseek.com/", "#5786FE", "#EEF2FF"),
        ("kimi", "国产", "Kimi", "月之暗面", "https://www.kimi.com/", "#111111", "#F0F0F0"),
        ("doubao", "国产", "豆包", "字节跳动", "https://www.doubao.com/", "#3C8CFF", "#E8F1FF"),
        ("tongyi", "国产", "通义", "阿里云", "https://tongyi.aliyun.com/", "#6950EF", "#EFECFD"),
        ("chatglm", "国产", "智谱清言", "智谱", "https://chatglm.cn/", "#2D2D2D", "#F0F0F0"),
    ]
    .into_iter()
    .enumerate()
    {
        v.push(("nav", key, i as i32, 0, json!({ "group": group, "name": name, "vendor": vendor, "url": url, "brand_color": c, "tint": t })));
    }
    for (i, (key, product, title, sub, label, url)) in [
        ("claude-code", "Claude", "Claude Code 上手导读", "10 分钟读完", "guide", "https://docs.claude.com/"),
        ("claude-skills", "Claude", "技能（Skills）怎么写", "附示例仓库", "guide", "https://docs.claude.com/"),
        ("codex-start", "Codex", "Codex 命令行快速开始", "安装、登录、第一个任务", "guide", "https://developers.openai.com/codex"),
        ("codex-quota", "Codex", "额度和计费说明", "官方页面索引", "index", "https://developers.openai.com/codex"),
        ("skillhub-publish", "SkillHub", "如何发布一个技能", "从打包到审核", "guide", site),
    ]
    .into_iter()
    .enumerate()
    {
        v.push(("doc", key, i as i32, 0, json!({ "product": product, "title": title, "subtitle": sub, "label": label, "url": url })));
    }
    for (i, (key, text, tag)) in [
        ("w1", "桌面特效", "热"), ("w2", "番茄时钟", ""), ("w3", "周报", ""), ("w4", "MCP", "新"),
        ("w5", "整理下载文件夹", ""), ("w6", "动态壁纸", ""), ("w7", "本周周刊", ""), ("w8", "小游戏", ""),
    ]
    .into_iter()
    .enumerate()
    {
        v.push(("hotword", key, i as i32, 0, json!({ "text": text, "tag": tag })));
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
        for (kind, key, _, _, data) in seed_items() {
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

//! 帖子、羊毛的读模型与列表查询，前台和 SEO 页面共用。

use crate::{error::AppError, users::PublicUser};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::collections::HashMap;
use uuid::Uuid;

pub const DEAL_TYPES: &[&str] =
    &["free_credit", "signup_bonus", "trial", "free_limited", "invite", "student", "discount", "public_site"];
pub const REGIONS: &[&str] = &["any", "cn", "intl"];
pub const DEAL_STATUSES: &[&str] = &["valid", "suspect", "expired"];

/// 领取条件位标志，和前端 `CONDITIONS` 保持一致。
pub const CONDITIONS: &[(&str, i32)] = &[
    ("card", 1),
    ("phone_cn", 2),
    ("phone_intl", 4),
    ("edu", 8),
    ("work_email", 16),
    ("realname", 32),
];
pub const CONDITION_MASK: i32 = 63;

pub fn condition_bits(names: &str) -> i32 {
    names
        .split(',')
        .filter_map(|n| CONDITIONS.iter().find(|(k, _)| *k == n.trim()).map(|(_, b)| *b))
        .fold(0, |acc, b| acc | b)
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryRef {
    pub id: i32,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductRef {
    pub id: i32,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DealInfo {
    pub product: ProductRef,
    pub deal_type: String,
    pub value_text: String,
    pub conditions: i32,
    pub region: String,
    pub link_url: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub long_term: bool,
    pub deal_status: String,
    pub valid_votes: i32,
    pub invalid_votes: i32,
    pub last_verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopicItem {
    pub id: i64,
    pub title: String,
    pub excerpt: String,
    pub category: CategoryRef,
    pub author: PublicUser,
    pub tags: Vec<String>,
    pub status: String,
    pub pinned: bool,
    pub featured: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub like_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub last_reply_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deal: Option<DealInfo>,
}

#[derive(sqlx::FromRow)]
struct TopicRow {
    id: i64,
    title: String,
    excerpt: String,
    status: String,
    pinned: bool,
    featured: bool,
    locked: bool,
    reply_count: i32,
    like_count: i32,
    view_count: i32,
    created_at: DateTime<Utc>,
    last_reply_at: DateTime<Utc>,
    edited_at: Option<DateTime<Utc>>,
    category_id: i32,
    category_slug: String,
    category_name: String,
    author_uid: Uuid,
    author_nickname: String,
    author_avatar: Option<String>,
    author_level: i16,
    product_id: Option<i32>,
    product_slug: Option<String>,
    product_name: Option<String>,
    deal_type: Option<String>,
    value_text: Option<String>,
    conditions: Option<i32>,
    region: Option<String>,
    link_url: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    long_term: Option<bool>,
    deal_status: Option<String>,
    valid_votes: Option<i32>,
    invalid_votes: Option<i32>,
    last_verified_at: Option<DateTime<Utc>>,
}

impl TopicRow {
    fn into_item(self, tags: Vec<String>) -> TopicItem {
        let deal = match (self.product_id, self.deal_type) {
            (Some(pid), Some(deal_type)) => Some(DealInfo {
                product: ProductRef {
                    id: pid,
                    slug: self.product_slug.unwrap_or_default(),
                    name: self.product_name.unwrap_or_default(),
                },
                deal_type,
                value_text: self.value_text.unwrap_or_default(),
                conditions: self.conditions.unwrap_or(0),
                region: self.region.unwrap_or_else(|| "any".into()),
                link_url: self.link_url.unwrap_or_default(),
                expires_at: self.expires_at,
                long_term: self.long_term.unwrap_or(false),
                deal_status: self.deal_status.unwrap_or_else(|| "valid".into()),
                valid_votes: self.valid_votes.unwrap_or(0),
                invalid_votes: self.invalid_votes.unwrap_or(0),
                last_verified_at: self.last_verified_at.unwrap_or(self.created_at),
            }),
            _ => None,
        };
        TopicItem {
            id: self.id,
            title: self.title,
            excerpt: self.excerpt,
            category: CategoryRef { id: self.category_id, slug: self.category_slug, name: self.category_name },
            author: PublicUser {
                uid: self.author_uid,
                nickname: self.author_nickname,
                avatar_url: self.author_avatar,
                trust_level: self.author_level,
            },
            tags,
            status: self.status,
            pinned: self.pinned,
            featured: self.featured,
            locked: self.locked,
            reply_count: self.reply_count,
            like_count: self.like_count,
            view_count: self.view_count,
            created_at: self.created_at,
            last_reply_at: self.last_reply_at,
            edited_at: self.edited_at,
            deal,
        }
    }
}

const SELECT: &str = "SELECT t.id, t.title, t.excerpt, t.status, t.pinned, t.featured, t.locked,
        t.reply_count, t.like_count, t.view_count, t.created_at, t.last_reply_at, t.edited_at,
        c.id AS category_id, c.slug AS category_slug, c.name AS category_name,
        u.uid AS author_uid, u.nickname AS author_nickname, u.avatar_url AS author_avatar, u.trust_level AS author_level,
        p.id AS product_id, p.slug AS product_slug, p.name AS product_name,
        d.deal_type, d.value_text, d.conditions, d.region, d.link_url, d.expires_at, d.long_term,
        d.deal_status, d.valid_votes, d.invalid_votes, d.last_verified_at
    FROM topics t
    JOIN categories c ON c.id = t.category_id
    JOIN users u ON u.uid = t.author_uid
    LEFT JOIN deals d ON d.topic_id = t.id
    LEFT JOIN products p ON p.id = d.product_id";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ListFilter {
    /// 分类 slug；一级分类会包含它的子分类。
    pub category: Option<String>,
    pub tag: Option<String>,
    pub product: Option<String>,
    pub author: Option<Uuid>,
    pub q: Option<String>,
    /// latest（最新回复）/ new（最新发布）/ hot / expiring（即将过期）/ verified（最近验证）
    pub sort: Option<String>,
    /// 只看首页可见分类（排除闲聊等）
    #[serde(default)]
    pub home: bool,
    /// deal：只看羊毛；normal：只看普通帖
    pub kind: Option<String>,
    pub deal_type: Option<String>,
    /// 逗号分隔的条件名，命中任意一个的羊毛会被排除，例如 `card,realname` 表示"不要信用卡、不要实名"
    pub without: Option<String>,
    pub region: Option<String>,
    /// active（默认，有效 + 疑似失效）/ valid / all
    pub deal_status: Option<String>,
    #[serde(default)]
    pub featured: bool,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl ListFilter {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).clamp(1, 1000)
    }
    pub fn page_size(&self) -> i64 {
        self.page_size.unwrap_or(30).clamp(1, 50)
    }
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

fn push_filters(qb: &mut QueryBuilder<'_, Postgres>, f: &ListFilter) {
    qb.push(" WHERE t.status = 'normal'");
    if let Some(slug) = f.category.as_deref().filter(|s| !s.is_empty()) {
        qb.push(" AND (c.slug = ")
            .push_bind(slug.to_string())
            .push(" OR c.parent_id = (SELECT id FROM categories WHERE slug = ")
            .push_bind(slug.to_string())
            .push("))");
    }
    if let Some(tag) = f.tag.as_deref().filter(|s| !s.is_empty()) {
        qb.push(" AND t.id IN (SELECT tt.topic_id FROM topic_tags tt JOIN tags g ON g.id = tt.tag_id WHERE g.name = ")
            .push_bind(tag.to_lowercase())
            .push(")");
    }
    if let Some(product) = f.product.as_deref().filter(|s| !s.is_empty()) {
        qb.push(" AND p.slug = ").push_bind(product.to_string());
    }
    if let Some(author) = f.author {
        qb.push(" AND t.author_uid = ").push_bind(author);
    }
    if let Some(q) = f.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        qb.push(" AND (t.title ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR t.excerpt ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR p.name ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
    if f.home {
        qb.push(" AND c.on_home");
    }
    match f.kind.as_deref() {
        Some("deal") => {
            qb.push(" AND d.topic_id IS NOT NULL");
        }
        Some("normal") => {
            qb.push(" AND d.topic_id IS NULL");
        }
        _ => {}
    }
    if let Some(t) = f.deal_type.as_deref().filter(|t| DEAL_TYPES.contains(t)) {
        qb.push(" AND d.deal_type = ").push_bind(t.to_string());
    }
    if let Some(without) = f.without.as_deref() {
        let bits = condition_bits(without);
        if bits != 0 {
            qb.push(" AND (d.conditions & ").push_bind(bits).push(") = 0");
        }
    }
    if let Some(region) = f.region.as_deref().filter(|r| *r == "cn" || *r == "intl") {
        // 选"国内可用"时，"不限地区"的也算。
        qb.push(" AND d.region IN ('any', ").push_bind(region.to_string()).push(")");
    }
    match f.deal_status.as_deref() {
        Some("all") => {}
        Some("valid") => {
            qb.push(" AND (d.topic_id IS NULL OR d.deal_status = 'valid')");
        }
        _ => {
            qb.push(" AND (d.topic_id IS NULL OR d.deal_status <> 'expired')");
        }
    }
    if f.featured {
        qb.push(" AND t.featured");
    }
}

pub async fn list_topics(pool: &PgPool, f: &ListFilter) -> Result<Page<TopicItem>, AppError> {
    let mut count = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) FROM topics t JOIN categories c ON c.id = t.category_id
         LEFT JOIN deals d ON d.topic_id = t.id LEFT JOIN products p ON p.id = d.product_id",
    );
    push_filters(&mut count, f);
    let (total,): (i64,) = count.build_query_as().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(SELECT);
    push_filters(&mut qb, f);
    // 分类页置顶帖优先；首页和羊毛库不受置顶影响。
    let pinned_first = if f.category.is_some() && !f.home { "t.pinned DESC, " } else { "" };
    let order = match f.sort.as_deref() {
        Some("new") => "t.created_at DESC",
        Some("hot") => "t.hot_score DESC, t.created_at DESC",
        Some("expiring") => "d.expires_at ASC NULLS LAST, t.created_at DESC",
        Some("verified") => "d.last_verified_at DESC NULLS LAST",
        _ => "t.last_reply_at DESC",
    };
    qb.push(format!(" ORDER BY {pinned_first}{order}, t.id DESC LIMIT "))
        .push_bind(f.page_size())
        .push(" OFFSET ")
        .push_bind((f.page() - 1) * f.page_size());
    let rows: Vec<TopicRow> = qb.build_query_as().fetch_all(pool).await?;

    let items = attach_tags(pool, rows).await?;
    Ok(Page { items, page: f.page(), page_size: f.page_size(), total })
}

/// 读单个帖子，不过滤状态，由调用方判断可见性。
pub async fn get_topic(pool: &PgPool, id: i64) -> Result<Option<TopicItem>, AppError> {
    let row: Option<TopicRow> =
        sqlx::query_as(&format!("{SELECT} WHERE t.id = $1")).bind(id).fetch_optional(pool).await?;
    Ok(match row {
        Some(row) => attach_tags(pool, vec![row]).await?.pop(),
        None => None,
    })
}

async fn attach_tags(pool: &PgPool, rows: Vec<TopicRow>) -> Result<Vec<TopicItem>, AppError> {
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let pairs: Vec<(i64, String)> = if ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as(
            "SELECT tt.topic_id, g.name FROM topic_tags tt JOIN tags g ON g.id = tt.tag_id
             WHERE tt.topic_id = ANY($1) ORDER BY g.name",
        )
        .bind(&ids)
        .fetch_all(pool)
        .await?
    };
    let mut by_topic: HashMap<i64, Vec<String>> = HashMap::new();
    for (id, name) in pairs {
        by_topic.entry(id).or_default().push(name);
    }
    Ok(rows.into_iter().map(|r| {
        let tags = by_topic.remove(&r.id).unwrap_or_default();
        r.into_item(tags)
    }).collect())
}

/// 重新统计羊毛的票数并更新状态：失效票（加权）占比 ≥ 40% 且总票数 ≥ 5 时转为"疑似失效"，
/// 有效票重新占优后恢复"有效"；"已失效"只能由版主或到期设置，投票不会改回。
pub async fn recompute_deal(pool: &PgPool, topic_id: i64) -> Result<(), AppError> {
    sqlx::query(
        "WITH v AS (
            SELECT COALESCE(SUM(weight) FILTER (WHERE value = 1), 0)::int AS valid,
                   COALESCE(SUM(weight) FILTER (WHERE value = -1), 0)::int AS invalid,
                   COUNT(*)::int AS total
            FROM deal_votes WHERE topic_id = $1
         )
         UPDATE deals d SET valid_votes = v.valid, invalid_votes = v.invalid,
            deal_status = CASE
                WHEN d.deal_status = 'expired' THEN 'expired'
                WHEN v.total >= 5 AND v.invalid * 10 >= (v.valid + v.invalid) * 4 THEN 'suspect'
                ELSE 'valid' END
         FROM v WHERE d.topic_id = $1",
    )
    .bind(topic_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// 热度：(有效票×3 + 点赞×2 + 回复 − 失效票×3) / (发帖小时数 + 2)^1.5
pub const HOT_SCORE_SQL: &str = "(COALESCE(d.valid_votes, 0) * 3 + t.like_count * 2 + t.reply_count - COALESCE(d.invalid_votes, 0) * 3)::float8
    / power(EXTRACT(EPOCH FROM (now() - t.created_at)) / 3600.0 + 2, 1.5)";

pub async fn refresh_hot_scores(pool: &PgPool, topic_id: Option<i64>) -> Result<u64, AppError> {
    let sql = format!(
        "UPDATE topics t SET hot_score = {HOT_SCORE_SQL}
         FROM topics t2 LEFT JOIN deals d ON d.topic_id = t2.id
         WHERE t.id = t2.id AND {}",
        if topic_id.is_some() { "t.id = $1" } else { "t.created_at > now() - interval '7 days'" }
    );
    let mut q = sqlx::query(&sql);
    if let Some(id) = topic_id {
        q = q.bind(id);
    }
    Ok(q.execute(pool).await?.rows_affected())
}

/// 到期的羊毛标为已失效。
pub async fn expire_deals(pool: &PgPool) -> Result<u64, AppError> {
    Ok(sqlx::query(
        "UPDATE deals SET deal_status = 'expired'
         WHERE deal_status <> 'expired' AND NOT long_term AND expires_at IS NOT NULL AND expires_at < now()",
    )
    .execute(pool)
    .await?
    .rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_names_map_to_bits() {
        assert_eq!(condition_bits("card,realname"), 33);
        assert_eq!(condition_bits(" phone_cn , bogus"), 2);
        assert_eq!(condition_bits(""), 0);
    }
}

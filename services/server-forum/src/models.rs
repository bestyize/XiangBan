//! 帖子、羊毛的读模型与列表查询，前台和 SEO 页面共用。

use crate::{error::{AppError, AppResult}, users::{ForumUser, PublicUser}};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::collections::HashMap;
use uuid::Uuid;

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

/// 信任等级的名称，下标就是等级。
pub const TRUST_NAMES: [&str; 4] = ["新人", "成员", "老用户", "守护者"];

/// 谁在读：未登录是 level 0、uid None，只能读 `min_read_level = 0` 的帖子；
/// 登录用户按信任等级读，并且总能读自己发的帖子。
/// 默认值就是游客，所以没特意设置读者的查询（比如 SEO 页面）只会看到公开内容。
#[derive(Debug, Clone, Default)]
pub struct Reader {
    pub level: i16,
    pub uid: Option<Uuid>,
}

impl Reader {
    pub fn from_user(user: Option<&ForumUser>) -> Self {
        match user {
            Some(u) => Reader { level: u.trust_level.clamp(0, 3), uid: Some(u.uid) },
            None => Reader::default(),
        }
    }

    /// 能不能读一个阅读权限为 `min_read_level`、作者为 `author` 的帖子。
    pub fn check(&self, min_read_level: i16, author: Uuid) -> AppResult<()> {
        if min_read_level <= self.level || self.uid == Some(author) {
            return Ok(());
        }
        let need = TRUST_NAMES[min_read_level.clamp(0, 3) as usize];
        Err(AppError::Forbidden(if self.uid.is_none() {
            format!("这个帖子仅「{need}」及以上等级可见，请先登录")
        } else {
            format!("这个帖子仅「{need}」及以上等级可见，你目前是「{}」", TRUST_NAMES[self.level.clamp(0, 3) as usize])
        }))
    }
}

/// 读一个帖子的权限检查：帖子不存在报 404，等级不够报 403。
pub async fn ensure_readable(pool: &PgPool, reader: &Reader, topic_id: i64) -> AppResult<()> {
    let row: Option<(i16, Uuid)> = sqlx::query_as("SELECT min_read_level, author_uid FROM topics WHERE id = $1")
        .bind(topic_id)
        .fetch_optional(pool)
        .await?;
    let (min, author) = row.ok_or_else(|| AppError::not_found("帖子"))?;
    reader.check(min, author)
}

/// 同上，入口是回复 id。
pub async fn ensure_post_readable(pool: &PgPool, reader: &Reader, post_id: i64) -> AppResult<()> {
    let row: Option<(i16, Uuid)> = sqlx::query_as(
        "SELECT t.min_read_level, t.author_uid FROM posts p JOIN topics t ON t.id = p.topic_id WHERE p.id = $1",
    )
    .bind(post_id)
    .fetch_optional(pool)
    .await?;
    let (min, author) = row.ok_or_else(|| AppError::not_found("回复"))?;
    reader.check(min, author)
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryRef {
    pub id: i32,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DealInfo {
    pub value_text: String,
    pub conditions: i32,
    pub link_url: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub long_term: bool,
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
    pub pin_scope: String,
    pub featured: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub like_count: i32,
    pub view_count: i32,
    /// 阅读权限：0 所有人，1 成员以上，2 老用户以上，3 守护者以上。
    pub min_read_level: i16,
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
    pin_scope: String,
    featured: bool,
    locked: bool,
    reply_count: i32,
    like_count: i32,
    view_count: i32,
    min_read_level: i16,
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
    value_text: Option<String>,
    conditions: Option<i32>,
    link_url: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    long_term: Option<bool>,
}

impl TopicRow {
    fn into_item(self, tags: Vec<String>) -> TopicItem {
        // 有 deals 记录（value_text 一定非空列）就是羊毛帖；产品可以没有。
        let deal = self.value_text.map(|value_text| DealInfo {
            value_text,
            conditions: self.conditions.unwrap_or(0),
            link_url: self.link_url.unwrap_or_default(),
            expires_at: self.expires_at,
            long_term: self.long_term.unwrap_or(false),
        });
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
            pin_scope: self.pin_scope,
            featured: self.featured,
            locked: self.locked,
            reply_count: self.reply_count,
            like_count: self.like_count,
            view_count: self.view_count,
            min_read_level: self.min_read_level,
            created_at: self.created_at,
            last_reply_at: self.last_reply_at,
            edited_at: self.edited_at,
            deal,
        }
    }
}

const SELECT: &str = "SELECT t.id, t.title, t.excerpt, t.status, t.pinned, t.pin_scope, t.featured, t.locked,
        t.reply_count, t.like_count, t.view_count, t.min_read_level, t.created_at, t.last_reply_at, t.edited_at,
        c.id AS category_id, c.slug AS category_slug, c.name AS category_name,
        u.uid AS author_uid, u.nickname AS author_nickname, u.avatar_url AS author_avatar, u.trust_level AS author_level,
        d.value_text, d.conditions, d.link_url, d.expires_at, d.long_term
    FROM topics t
    JOIN categories c ON c.id = t.category_id
    JOIN users u ON u.uid = t.author_uid
    LEFT JOIN deals d ON d.topic_id = t.id";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ListFilter {
    /// 分类 slug；一级分类会包含它的子分类。
    pub category: Option<String>,
    pub tag: Option<String>,
    pub author: Option<Uuid>,
    /// 关注动态只允许服务端从 AuthUser 设置，查询参数不能指定其他用户。
    #[serde(skip)]
    pub followed_by: Option<Uuid>,
    pub q: Option<String>,
    /// latest（最新回复）/ new（最新发布）/ hot / expiring（即将过期）
    pub sort: Option<String>,
    /// 只看首页可见分类（排除闲聊等）
    #[serde(default)]
    pub home: bool,
    /// deal：只看羊毛；normal：只看普通帖
    pub kind: Option<String>,
    /// 默认不返回已到期的羊毛；用户主页、搜索、标签页等需要完整历史时设为 true。
    #[serde(default)]
    pub include_expired: bool,
    #[serde(default)]
    pub featured: bool,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    /// 谁在读。只能由服务端按登录态设置，`serde(skip)` 保证不能通过查询参数伪造。
    #[serde(skip)]
    pub reader: Reader,
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
    // 阅读权限：公开的、等级够的，以及读者自己发的。
    qb.push(" AND (t.min_read_level <= ").push_bind(f.reader.level);
    if let Some(uid) = f.reader.uid {
        qb.push(" OR t.author_uid = ").push_bind(uid);
    }
    qb.push(")");
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
    if let Some(author) = f.author {
        qb.push(" AND t.author_uid = ").push_bind(author);
    }
    if let Some(uid) = f.followed_by {
        qb.push(" AND t.author_uid IN (SELECT f.followed_uid FROM user_follows f
                 JOIN users followed ON followed.uid = f.followed_uid AND followed.status <> 'banned'
                 WHERE f.follower_uid = ")
            .push_bind(uid)
            .push(")");
    }
    if let Some(q) = f.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        qb.push(" AND (t.title ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR t.excerpt ILIKE ")
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
    if !f.include_expired {
        // 到期时间已过的羊毛不再展示（长期有效或没写有效期的一直展示）。
        qb.push(" AND (d.topic_id IS NULL OR d.long_term OR d.expires_at IS NULL OR d.expires_at > now())");
    }
    if f.featured {
        qb.push(" AND t.featured");
    }
}

pub async fn list_topics(pool: &PgPool, f: &ListFilter) -> Result<Page<TopicItem>, AppError> {
    let mut count = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) FROM topics t JOIN categories c ON c.id = t.category_id
         LEFT JOIN deals d ON d.topic_id = t.id",
    );
    push_filters(&mut count, f);
    let (total,): (i64,) = count.build_query_as().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(SELECT);
    push_filters(&mut qb, f);
    // 置顶按范围逐级生效：首页只提升全站置顶；板块页提升全站、所属一级板块、当前二级板块置顶。
    if let Some(slug) = f.category.as_deref().filter(|s| !s.is_empty()) {
        qb.push(" ORDER BY CASE WHEN t.pin_scope = 'site' THEN 0
            WHEN t.pin_scope = 'parent' AND (
                c.id = (SELECT id FROM categories WHERE slug = ")
            .push_bind(slug.to_string())
            .push(") OR c.parent_id = (SELECT id FROM categories WHERE slug = ")
            .push_bind(slug.to_string())
            .push(") OR c.parent_id = (SELECT parent_id FROM categories WHERE slug = ")
            .push_bind(slug.to_string())
            .push(")) THEN 1
            WHEN t.pin_scope = 'category' AND c.id = (SELECT id FROM categories WHERE slug = ")
            .push_bind(slug.to_string())
            .push(") THEN 2 ELSE 3 END, ");
    } else if f.home {
        qb.push(" ORDER BY CASE WHEN t.pin_scope = 'site' THEN 0 ELSE 1 END, ");
    } else {
        qb.push(" ORDER BY ");
    }
    let order = match f.sort.as_deref() {
        Some("new") => "t.created_at DESC",
        Some("hot") => "t.hot_score DESC, t.created_at DESC",
        Some("expiring") => "d.expires_at ASC NULLS LAST, t.created_at DESC",
        _ => "t.last_reply_at DESC",
    };
    qb.push(format!("{order}, t.id DESC LIMIT "))
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

/// 热度：(点赞×2 + 回复) / (发帖小时数 + 2)^1.5
pub const HOT_SCORE_SQL: &str = "(t.like_count * 2 + t.reply_count)::float8
    / power(EXTRACT(EPOCH FROM (now() - t.created_at)) / 3600.0 + 2, 1.5)";

pub async fn refresh_hot_scores(pool: &PgPool, topic_id: Option<i64>) -> Result<u64, AppError> {
    let sql = format!(
        "UPDATE topics t SET hot_score = {HOT_SCORE_SQL}
         WHERE {}",
        if topic_id.is_some() { "t.id = $1" } else { "t.created_at > now() - interval '7 days'" }
    );
    let mut q = sqlx::query(&sql);
    if let Some(id) = topic_id {
        q = q.bind(id);
    }
    Ok(q.execute(pool).await?.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn followed_feed_cannot_spoof_owner_or_reader() {
        use sqlx::Execute;
        let uid = Uuid::new_v4();
        let input = serde_json::json!({
            "followed_by": uid,
            "reader": { "uid": uid, "level": 3 }
        });
        let mut filter: ListFilter = serde_json::from_value(input).unwrap();
        assert!(filter.followed_by.is_none());
        assert!(filter.reader.uid.is_none());
        assert_eq!(filter.reader.level, 0);
        filter.followed_by = Some(uid);
        let mut qb = QueryBuilder::<Postgres>::new("SELECT t.id FROM topics t");
        push_filters(&mut qb, &filter);
        let query = qb.build();
        let sql = query.sql();
        assert!(sql.contains("t.status = 'normal'"));
        assert!(sql.contains("t.min_read_level <="));
        assert!(sql.contains("f.follower_uid ="));
        assert!(sql.contains("followed.status <> 'banned'"));
        assert!(sql.contains("d.expires_at > now()"));
    }

    fn reader(level: i16, uid: Option<Uuid>) -> Reader {
        Reader { level, uid }
    }

    #[test]
    fn guests_only_read_public_topics() {
        let author = Uuid::new_v4();
        let guest = Reader::default();
        assert!(guest.check(0, author).is_ok());
        for need in 1..=3 {
            let err = guest.check(need, author).unwrap_err().to_string();
            assert!(err.contains("请先登录"), "{err}");
        }
    }

    #[test]
    fn logged_in_readers_need_the_level() {
        let author = Uuid::new_v4();
        let member = reader(1, Some(Uuid::new_v4()));
        assert!(member.check(0, author).is_ok());
        assert!(member.check(1, author).is_ok());
        let err = member.check(2, author).unwrap_err().to_string();
        assert!(err.contains("老用户") && err.contains("成员"), "{err}");
        assert!(member.check(3, author).is_err());
        assert!(reader(3, Some(Uuid::new_v4())).check(3, author).is_ok());
    }

    #[test]
    fn authors_always_read_their_own_topics() {
        let me = Uuid::new_v4();
        // 即使后来被降级到比自己帖子的阅读权限还低，也能读自己的帖子。
        assert!(reader(0, Some(me)).check(3, me).is_ok());
        assert!(reader(0, Some(Uuid::new_v4())).check(3, me).is_err());
    }
}


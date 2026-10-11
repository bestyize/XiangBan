use crate::{
    auth::{AuthUser, MaybeUser},
    error::{AppError, AppResult},
    markdown,
    models::{self, ListFilter, Page, TopicItem, CONDITION_MASK, DEAL_TYPES, REGIONS},
    state::AppState,
    users,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

pub async fn list(State(state): State<AppState>, Query(f): Query<ListFilter>) -> AppResult<Json<Page<TopicItem>>> {
    Ok(Json(models::list_topics(&state.pool, &f).await?))
}

pub async fn list_deals(State(state): State<AppState>, Query(mut f): Query<ListFilter>) -> AppResult<Json<Page<TopicItem>>> {
    f.kind = Some("deal".into());
    if f.sort.is_none() {
        f.sort = Some("new".into());
    }
    Ok(Json(models::list_topics(&state.pool, &f).await?))
}

#[derive(Serialize)]
pub struct TopicDetail {
    #[serde(flatten)]
    pub topic: TopicItem,
    pub body_html: String,
    pub body_md: String,
    pub viewer: Viewer,
}

#[derive(Serialize, Default)]
pub struct Viewer {
    pub liked: bool,
    pub bookmarked: bool,
    /// 1 有效、-1 失效、0 未投
    pub vote: i16,
    pub can_edit: bool,
    pub can_reply: bool,
    pub can_vote: bool,
    /// TL3 守护者可以直接标记羊毛状态
    pub can_moderate: bool,
}

pub async fn get(State(state): State<AppState>, MaybeUser(user): MaybeUser, Path(id): Path<i64>) -> AppResult<Json<TopicDetail>> {
    let topic = models::get_topic(&state.pool, id).await?.ok_or_else(|| AppError::not_found("帖子"))?;
    let is_author = user.as_ref().is_some_and(|u| u.uid == topic.author.uid);
    if topic.status != "normal" && !is_author {
        return Err(AppError::not_found("帖子"));
    }
    let (body_html, body_md): (String, String) =
        sqlx::query_as("SELECT body_html, body_md FROM topics WHERE id = $1").bind(id).fetch_one(&state.pool).await?;

    sqlx::query("UPDATE topics SET view_count = view_count + 1 WHERE id = $1").bind(id).execute(&state.pool).await?;

    let mut viewer = Viewer::default();
    if let Some(u) = &user {
        // 第一次读这个帖子才计入阅读数，用于自动升级。
        let inserted = sqlx::query("INSERT INTO topic_reads (uid, topic_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(u.uid)
            .bind(id)
            .execute(&state.pool)
            .await?
            .rows_affected();
        if inserted > 0 {
            sqlx::query("UPDATE users SET read_count = read_count + 1 WHERE uid = $1").bind(u.uid).execute(&state.pool).await?;
            users::maybe_promote(&state.pool, &state.config.trust, u.uid).await?;
        }
        let (liked, bookmarked, vote): (bool, bool, Option<i16>) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE target_type = 'topic' AND target_id = $1 AND uid = $2),
                    EXISTS(SELECT 1 FROM bookmarks WHERE topic_id = $1 AND uid = $2),
                    (SELECT value FROM deal_votes WHERE topic_id = $1 AND uid = $2)",
        )
        .bind(id)
        .bind(u.uid)
        .fetch_one(&state.pool)
        .await?;
        let writable = u.status != "banned" && users::ensure_can_write(u).is_ok();
        viewer = Viewer {
            liked,
            bookmarked,
            vote: vote.unwrap_or(0),
            can_edit: is_author && !topic.locked && writable,
            can_reply: !topic.locked && writable,
            can_vote: topic.deal.is_some() && u.trust_level >= 1 && writable,
            can_moderate: u.trust_level >= 3,
        };
    }
    Ok(Json(TopicDetail { topic, body_html, body_md, viewer }))
}

#[derive(Deserialize)]
pub struct DealInput {
    pub product_id: i32,
    pub deal_type: String,
    #[serde(default)]
    pub value_text: String,
    #[serde(default)]
    pub conditions: i32,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub link_url: String,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub long_term: bool,
}

#[derive(Deserialize)]
pub struct TopicInput {
    pub category_id: i32,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub deal: Option<DealInput>,
}

#[derive(sqlx::FromRow)]
struct CategoryRules {
    id: i32,
    kind: String,
    min_post_level: i16,
    admin_only: bool,
    has_children: bool,
}

async fn category_rules(pool: &PgPool, id: i32) -> AppResult<CategoryRules> {
    sqlx::query_as(
        "SELECT c.id, c.kind, c.min_post_level, c.admin_only,
                EXISTS(SELECT 1 FROM categories k WHERE k.parent_id = c.id) AS has_children
         FROM categories c WHERE c.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("分类不存在".into()))
}

/// 推广参数默认不允许出现在羊毛链接里，站方统一管理推广链接。
const REFERRAL_PARAMS: &[&str] = &["ref", "aff", "affiliate", "referral", "referrer", "invite", "invite_code", "invitecode", "via", "fpr"];

pub fn validate_link(url: &str) -> AppResult<()> {
    if url.is_empty() {
        return Ok(());
    }
    if url.len() > 500 || !(url.starts_with("https://") || url.starts_with("http://")) || url.contains(char::is_whitespace) {
        return Err(AppError::BadRequest("活动链接需要是 http(s) 地址".into()));
    }
    if let Some((_, query)) = url.split_once('?') {
        for pair in query.split(['&', '#']) {
            let key = pair.split('=').next().unwrap_or("").to_lowercase();
            if REFERRAL_PARAMS.contains(&key.as_str()) {
                return Err(AppError::BadRequest("请贴官方活动页，不要带个人推广参数（如 ref、aff、invite）".into()));
            }
        }
    }
    Ok(())
}

fn clean_title(raw: &str) -> AppResult<String> {
    let title = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let len = title.chars().count();
    if !(4..=80).contains(&len) {
        return Err(AppError::BadRequest("标题需要 4-80 个字".into()));
    }
    Ok(title)
}

fn clean_body(raw: &str, min: usize) -> AppResult<String> {
    let body = raw.trim().to_string();
    let len = body.chars().count();
    if len < min || len > 20_000 {
        return Err(AppError::BadRequest(format!("正文需要 {min}-20000 个字")));
    }
    Ok(body)
}

fn clean_tags(raw: &[String]) -> AppResult<Vec<String>> {
    let mut tags: Vec<String> = Vec::new();
    for t in raw {
        let t: String = t.trim().trim_start_matches('#').to_lowercase().split_whitespace().collect::<Vec<_>>().join("-");
        if t.is_empty() {
            continue;
        }
        if t.chars().count() > 20 {
            return Err(AppError::BadRequest("每个标签最多 20 个字".into()));
        }
        if !tags.contains(&t) {
            tags.push(t);
        }
    }
    if tags.len() > 5 {
        return Err(AppError::BadRequest("最多 5 个标签".into()));
    }
    Ok(tags)
}

fn clean_deal(d: &DealInput) -> AppResult<()> {
    if !DEAL_TYPES.contains(&d.deal_type.as_str()) {
        return Err(AppError::BadRequest("羊毛类型无效".into()));
    }
    if d.value_text.chars().count() > 40 {
        return Err(AppError::BadRequest("价值描述最多 40 个字".into()));
    }
    if d.conditions & !CONDITION_MASK != 0 {
        return Err(AppError::BadRequest("领取条件无效".into()));
    }
    if let Some(r) = &d.region {
        if !REGIONS.contains(&r.as_str()) {
            return Err(AppError::BadRequest("地区无效".into()));
        }
    }
    validate_link(d.link_url.trim())
}

pub async fn save_tags(pool: &PgPool, topic_id: i64, tags: &[String]) -> AppResult<()> {
    sqlx::query("DELETE FROM topic_tags WHERE topic_id = $1").bind(topic_id).execute(pool).await?;
    for tag in tags {
        let (tag_id,): (i32,) = sqlx::query_as(
            "INSERT INTO tags (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
        )
        .bind(tag)
        .fetch_one(pool)
        .await?;
        sqlx::query("INSERT INTO topic_tags (topic_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(topic_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn ensure_product(pool: &PgPool, id: i32) -> AppResult<()> {
    let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM products WHERE id = $1").bind(id).fetch_optional(pool).await?;
    exists.map(|_| ()).ok_or_else(|| AppError::BadRequest("请选择关联产品".into()))
}

pub async fn create(State(state): State<AppState>, auth: AuthUser, Json(input): Json<TopicInput>) -> AppResult<Json<Value>> {
    let user = &auth.user;
    users::ensure_can_write(user)?;
    let cat = category_rules(&state.pool, input.category_id).await?;
    if cat.has_children {
        return Err(AppError::BadRequest("请选择具体的二级分类".into()));
    }
    if cat.admin_only {
        return Err(AppError::Forbidden("这个分类只有站务人员可以发帖".into()));
    }
    if user.trust_level < cat.min_post_level {
        return Err(AppError::Forbidden(format!("这个分类需要信任等级 {} 才能发帖", cat.min_post_level)));
    }
    users::ensure_daily_quota(&state.pool, user, "topics", &state.config.trust.daily_topics, "发帖").await?;

    let title = clean_title(&input.title)?;
    let body = clean_body(&input.body, if cat.kind == "deal" { 0 } else { 10 })?;
    let tags = clean_tags(&input.tags)?;
    let deal = match (cat.kind.as_str(), &input.deal) {
        ("deal", Some(d)) => {
            clean_deal(d)?;
            ensure_product(&state.pool, d.product_id).await?;
            Some(d)
        }
        ("deal", None) => return Err(AppError::BadRequest("羊毛帖需要填写羊毛信息".into())),
        (_, Some(_)) => return Err(AppError::BadRequest("只有羊毛库分类可以填写羊毛信息".into())),
        _ => None,
    };

    let trusted = user.trust_level >= 1;
    let html = markdown::render(&body, trusted);
    let excerpt = markdown::excerpt(&body, 140);

    let mut tx = state.pool.begin().await?;
    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO topics (category_id, author_uid, title, body_md, body_html, excerpt)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(cat.id)
    .bind(user.uid)
    .bind(&title)
    .bind(&body)
    .bind(&html)
    .bind(&excerpt)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(d) = deal {
        sqlx::query(
            "INSERT INTO deals (topic_id, product_id, deal_type, value_text, conditions, region, link_url, expires_at, long_term)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(d.product_id)
        .bind(&d.deal_type)
        .bind(d.value_text.trim())
        .bind(d.conditions)
        .bind(d.region.as_deref().unwrap_or("any"))
        .bind(d.link_url.trim())
        .bind(if d.long_term { None } else { d.expires_at })
        .bind(d.long_term)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    save_tags(&state.pool, id, &tags).await?;
    models::refresh_hot_scores(&state.pool, Some(id)).await?;
    Ok(Json(json!({ "id": id })))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Json(input): Json<TopicInput>,
) -> AppResult<Json<Value>> {
    let user = &auth.user;
    users::ensure_can_write(user)?;
    let topic = models::get_topic(&state.pool, id).await?.ok_or_else(|| AppError::not_found("帖子"))?;
    if topic.author.uid != user.uid || topic.status == "deleted" {
        return Err(AppError::Forbidden("只能编辑自己的帖子".into()));
    }
    if topic.locked {
        return Err(AppError::Forbidden("帖子已锁定，不能编辑".into()));
    }
    // 编辑不允许换到别的类型的分类，避免羊毛信息丢失。
    let cat = category_rules(&state.pool, input.category_id).await?;
    if cat.has_children || cat.admin_only || (cat.kind == "deal") != topic.deal.is_some() {
        return Err(AppError::BadRequest("不能移动到这个分类".into()));
    }
    let title = clean_title(&input.title)?;
    let body = clean_body(&input.body, if cat.kind == "deal" { 0 } else { 10 })?;
    let tags = clean_tags(&input.tags)?;
    let html = markdown::render(&body, user.trust_level >= 1);
    let excerpt = markdown::excerpt(&body, 140);

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE topics SET category_id = $2, title = $3, body_md = $4, body_html = $5, excerpt = $6, edited_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(cat.id)
    .bind(&title)
    .bind(&body)
    .bind(&html)
    .bind(&excerpt)
    .execute(&mut *tx)
    .await?;
    if let Some(d) = &input.deal {
        if topic.deal.is_some() {
            clean_deal(d)?;
            ensure_product(&state.pool, d.product_id).await?;
            sqlx::query(
                "UPDATE deals SET product_id = $2, deal_type = $3, value_text = $4, conditions = $5, region = $6,
                    link_url = $7, expires_at = $8, long_term = $9
                 WHERE topic_id = $1",
            )
            .bind(id)
            .bind(d.product_id)
            .bind(&d.deal_type)
            .bind(d.value_text.trim())
            .bind(d.conditions)
            .bind(d.region.as_deref().unwrap_or("any"))
            .bind(d.link_url.trim())
            .bind(if d.long_term { None } else { d.expires_at })
            .bind(d.long_term)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    save_tags(&state.pool, id, &tags).await?;
    Ok(Json(json!({ "id": id })))
}

pub async fn delete(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let done = sqlx::query("UPDATE topics SET status = 'deleted' WHERE id = $1 AND author_uid = $2 AND status <> 'deleted'")
        .bind(id)
        .bind(auth.user.uid)
        .execute(&state.pool)
        .await?
        .rows_affected();
    if done == 0 {
        return Err(AppError::Forbidden("只能删除自己的帖子".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct PreviewInput {
    pub body: String,
}

pub async fn preview(auth: AuthUser, Json(input): Json<PreviewInput>) -> AppResult<Json<Value>> {
    if input.body.chars().count() > 20_000 {
        return Err(AppError::BadRequest("内容太长".into()));
    }
    Ok(Json(json!({ "html": markdown::render(&input.body, auth.user.trust_level >= 1) })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn referral_links_are_rejected() {
        assert!(validate_link("https://claude.ai/upgrade").is_ok());
        assert!(validate_link("https://x.com/a?utm_source=x").is_ok());
        assert!(validate_link("https://x.com/a?ref=abc").is_err());
        assert!(validate_link("https://x.com/a?b=1&AFF=2").is_err());
        assert!(validate_link("javascript:alert(1)").is_err());
        assert!(validate_link("").is_ok());
    }

    #[test]
    fn tags_are_normalized() {
        let tags = clean_tags(&["#Claude".into(), "claude".into(), " Open AI ".into(), "".into()]).unwrap();
        assert_eq!(tags, vec!["claude", "open-ai"]);
        assert!(clean_tags(&(0..6).map(|i| format!("t{i}")).collect::<Vec<_>>()).is_err());
    }
}

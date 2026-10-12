use super::attachments::{self, AttachmentItem};
use crate::{
    auth::{AuthUser, MaybeUser},
    error::{AppError, AppResult},
    markdown,
    models::{self, ListFilter, Page, Reader, TopicItem, CONDITION_MASK, TRUST_NAMES},
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
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Query(mut f): Query<ListFilter>,
) -> AppResult<Json<Page<TopicItem>>> {
    f.reader = Reader::from_user(user.as_ref());
    Ok(Json(models::list_topics(&state.pool, &f).await?))
}

pub async fn list_deals(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Query(mut f): Query<ListFilter>,
) -> AppResult<Json<Page<TopicItem>>> {
    f.reader = Reader::from_user(user.as_ref());
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
    pub attachments: Vec<AttachmentItem>,
    pub viewer: Viewer,
}

#[derive(Serialize, Default)]
pub struct Viewer {
    pub liked: bool,
    pub bookmarked: bool,
    pub can_edit: bool,
    pub can_reply: bool,
    /// 守护者可以把正常状态的帖子置顶到所在板块（和首页）。
    pub can_pin: bool,
}

pub async fn get(State(state): State<AppState>, MaybeUser(user): MaybeUser, Path(id): Path<i64>) -> AppResult<Json<TopicDetail>> {
    let topic = models::get_topic(&state.pool, id).await?.ok_or_else(|| AppError::not_found("帖子"))?;
    let is_author = user.as_ref().is_some_and(|u| u.uid == topic.author.uid);
    if topic.status != "normal" && !is_author {
        return Err(AppError::not_found("帖子"));
    }
    // 阅读权限在读正文、记浏览数之前检查。
    Reader::from_user(user.as_ref()).check(topic.min_read_level, topic.author.uid)?;
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
        let (liked, bookmarked): (bool, bool) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE target_type = 'topic' AND target_id = $1 AND uid = $2),
                    EXISTS(SELECT 1 FROM bookmarks WHERE topic_id = $1 AND uid = $2)",
        )
        .bind(id)
        .bind(u.uid)
        .fetch_one(&state.pool)
        .await?;
        let writable = u.status != "banned" && users::ensure_can_write(u).is_ok();
        viewer = Viewer {
            liked,
            bookmarked,
            can_edit: is_author && !topic.locked && writable,
            can_reply: !topic.locked && writable,
            can_pin: u.trust_level >= GUARDIAN_LEVEL && topic.status == "normal",
        };
    }
    let attachments = attachments::list_for_topic(&state.pool, id).await?;
    Ok(Json(TopicDetail { topic, body_html, body_md, attachments, viewer }))
}

/// 羊毛信息，每一项发帖人都可以不填。
#[derive(Deserialize, Default)]
pub struct DealInput {
    #[serde(default)]
    pub value_text: String,
    /// 领取条件位标志，0 表示没有特殊条件或没写。
    #[serde(default)]
    pub conditions: i32,
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
    /// 谁能读：0 所有人，1 成员以上，2 老用户以上，3 守护者以上。
    /// 不能超过发帖人自己的信任等级。发帖不传按 0，编辑不传保持原样。
    pub min_read_level: Option<i16>,
    /// 先用 `POST /attachments` 上传得到 id。发帖不传按没有附件，编辑不传保持原样，传了就以这个列表为准。
    pub attachment_ids: Option<Vec<Uuid>>,
}

/// 守护者（站务人员）的信任等级。
pub const GUARDIAN_LEVEL: i16 = 3;

/// 发帖到某个分类需要的信任等级：「仅管理员可发」的分类要守护者，其余看分类的最低发帖等级。
///
/// 论坛的管理后台账号和论坛用户是两套体系，后台账号发不了帖，所以「仅管理员可发」实际指的是守护者。
pub fn required_post_level(min_post_level: i16, admin_only: bool) -> i16 {
    if admin_only {
        GUARDIAN_LEVEL
    } else {
        min_post_level.clamp(0, GUARDIAN_LEVEL)
    }
}

fn level_error(admin_only: bool, need: i16) -> AppError {
    AppError::Forbidden(if admin_only {
        "这个分类只有守护者（站务人员）可以发帖".into()
    } else {
        format!("这个分类需要信任等级 {need} 才能发帖")
    })
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

/// 标签的统一写法：去掉 #、小写、空白换成连字符。空串返回 None。
pub fn normalize_tag(raw: &str) -> Option<String> {
    let t = raw.trim().trim_start_matches('#').to_lowercase().split_whitespace().collect::<Vec<_>>().join("-");
    (!t.is_empty()).then_some(t)
}

fn clean_tags(raw: &[String]) -> AppResult<Vec<String>> {
    let mut tags: Vec<String> = Vec::new();
    for t in raw.iter().filter_map(|t| normalize_tag(t)) {
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
    if d.value_text.chars().count() > 40 {
        return Err(AppError::BadRequest("价值描述最多 40 个字".into()));
    }
    if d.conditions & !CONDITION_MASK != 0 {
        return Err(AppError::BadRequest("领取条件无效".into()));
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

/// 阅读权限只能设到不超过自己的信任等级。`current` 是编辑时帖子原来的值：保持不变总是允许
/// （比如发帖人后来被降级，编辑其他内容时不应该被迫改阅读权限）。
fn clean_read_level(wanted: Option<i16>, user_level: i16, current: Option<i16>) -> AppResult<i16> {
    let level = match wanted {
        Some(l) => l,
        None => return Ok(current.unwrap_or(0)),
    };
    if !(0..=3).contains(&level) {
        return Err(AppError::BadRequest("阅读权限无效".into()));
    }
    if level > user_level.clamp(0, 3) && Some(level) != current {
        return Err(AppError::Forbidden(format!(
            "阅读权限最高只能设到你自己的等级「{}」",
            TRUST_NAMES[user_level.clamp(0, 3) as usize]
        )));
    }
    Ok(level)
}

pub async fn create(State(state): State<AppState>, auth: AuthUser, Json(input): Json<TopicInput>) -> AppResult<Json<Value>> {
    let user = &auth.user;
    users::ensure_can_write(user)?;
    let cat = category_rules(&state.pool, input.category_id).await?;
    if cat.has_children {
        return Err(AppError::BadRequest("请选择具体的二级分类".into()));
    }
    let need = required_post_level(cat.min_post_level, cat.admin_only);
    if user.trust_level < need {
        return Err(level_error(cat.admin_only, need));
    }
    users::ensure_daily_quota(&state.pool, user, "topics", &state.config.trust.daily_topics, "发帖").await?;

    let title = clean_title(&input.title)?;
    let body = clean_body(&input.body, if cat.kind == "deal" { 0 } else { 10 })?;
    let tags = clean_tags(&input.tags)?;
    let read_level = clean_read_level(input.min_read_level, user.trust_level, None)?;
    let empty_deal = DealInput::default();
    let deal = match (cat.kind.as_str(), &input.deal) {
        ("deal", Some(d)) => {
            clean_deal(d)?;
            Some(d)
        }
        // 羊毛信息全是选填，什么都没填也照样发成羊毛帖。
        ("deal", None) => Some(&empty_deal),
        (_, Some(_)) => return Err(AppError::BadRequest("只有福利羊毛分类可以填写羊毛信息".into())),
        _ => None,
    };

    let trusted = user.trust_level >= 1;
    let html = markdown::render(&body, trusted);
    let excerpt = markdown::excerpt(&body, 140);

    let mut tx = state.pool.begin().await?;
    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO topics (category_id, author_uid, title, body_md, body_html, excerpt, min_read_level)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(cat.id)
    .bind(user.uid)
    .bind(&title)
    .bind(&body)
    .bind(&html)
    .bind(&excerpt)
    .bind(read_level)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(d) = deal {
        sqlx::query(
            "INSERT INTO deals (topic_id, value_text, conditions, link_url, expires_at, long_term)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(d.value_text.trim())
        .bind(d.conditions)
        .bind(d.link_url.trim())
        .bind(if d.long_term { None } else { d.expires_at })
        .bind(d.long_term)
        .execute(&mut *tx)
        .await?;
    }
    if let Some(ids) = &input.attachment_ids {
        attachments::set_for_topic(&mut tx, id, user.uid, ids).await?;
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
    if cat.has_children || (cat.kind == "deal") != topic.deal.is_some() {
        return Err(AppError::BadRequest("不能移动到这个分类".into()));
    }
    // 留在原分类总是允许（比如守护者后来被降级，仍然能改自己发的公告）；
    // 换到别的分类要有和在那里发帖一样的等级，否则等于绕过发帖限制。
    if cat.id != topic.category.id {
        let need = required_post_level(cat.min_post_level, cat.admin_only);
        if user.trust_level < need {
            return Err(level_error(cat.admin_only, need));
        }
    }
    let title = clean_title(&input.title)?;
    let body = clean_body(&input.body, if cat.kind == "deal" { 0 } else { 10 })?;
    let tags = clean_tags(&input.tags)?;
    let read_level = clean_read_level(input.min_read_level, user.trust_level, Some(topic.min_read_level))?;
    let html = markdown::render(&body, user.trust_level >= 1);
    let excerpt = markdown::excerpt(&body, 140);

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE topics SET category_id = $2, title = $3, body_md = $4, body_html = $5, excerpt = $6, min_read_level = $7,
            edited_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(cat.id)
    .bind(&title)
    .bind(&body)
    .bind(&html)
    .bind(&excerpt)
    .bind(read_level)
    .execute(&mut *tx)
    .await?;
    if let Some(d) = &input.deal {
        if topic.deal.is_some() {
            clean_deal(d)?;
            sqlx::query(
                "UPDATE deals SET value_text = $2, conditions = $3, link_url = $4, expires_at = $5, long_term = $6
                 WHERE topic_id = $1",
            )
            .bind(id)
            .bind(d.value_text.trim())
            .bind(d.conditions)
            .bind(d.link_url.trim())
            .bind(if d.long_term { None } else { d.expires_at })
            .bind(d.long_term)
            .execute(&mut *tx)
            .await?;
        }
    }
    let removed = match &input.attachment_ids {
        Some(ids) => attachments::set_for_topic(&mut tx, id, user.uid, ids).await?,
        None => Vec::new(),
    };
    tx.commit().await?;
    attachments::remove_files(&state.config.upload_dir, &removed).await;
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

/// 守护者置顶或取消置顶。后台账号和论坛用户是两套体系，这里的操作同样写进审计日志，操作人记为「守护者:昵称」。
async fn set_pinned(state: &AppState, auth: &AuthUser, id: i64, pinned: bool) -> AppResult<Json<Value>> {
    let user = &auth.user;
    if user.trust_level < GUARDIAN_LEVEL {
        return Err(AppError::Forbidden("只有守护者（站务人员）可以置顶帖子".into()));
    }
    users::ensure_can_write(user)?;
    let mut tx = state.pool.begin().await?;
    let (status, was): (String, bool) = sqlx::query_as("SELECT status, pinned FROM topics WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("帖子"))?;
    if status != "normal" {
        return Err(AppError::Conflict("只有正常状态的帖子可以置顶".into()));
    }
    if was != pinned {
        sqlx::query("UPDATE topics SET pinned = $2 WHERE id = $1").bind(id).bind(pinned).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO admin_audit (admin, action, target_type, target_id, detail) VALUES ($1, $2, 'topic', $3, $4)")
            .bind(format!("守护者:{}", user.nickname))
            .bind(if pinned { "topic.pin" } else { "topic.unpin" })
            .bind(id.to_string())
            .bind(format!("uid {}", user.uid))
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({ "pinned": pinned })))
}

pub async fn pin(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    set_pinned(&state, &auth, id, true).await
}

pub async fn unpin(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    set_pinned(&state, &auth, id, false).await
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
    fn read_level_cannot_exceed_own_trust_level() {
        // 发帖：不传默认公开；不超过自己等级都行
        assert_eq!(clean_read_level(None, 1, None).unwrap(), 0);
        assert_eq!(clean_read_level(Some(0), 1, None).unwrap(), 0);
        assert_eq!(clean_read_level(Some(1), 1, None).unwrap(), 1);
        // 超过自己等级被拒绝，并说明自己是什么等级
        let err = clean_read_level(Some(2), 1, None).unwrap_err().to_string();
        assert!(err.contains("成员"), "{err}");
        assert!(clean_read_level(Some(3), 2, None).is_err());
        assert_eq!(clean_read_level(Some(3), 3, None).unwrap(), 3);
        // 越界的值
        assert!(clean_read_level(Some(-1), 3, None).is_err());
        assert!(clean_read_level(Some(4), 3, None).is_err());
    }

    #[test]
    fn editing_keeps_the_existing_read_level_unless_changed() {
        // 编辑不传：保持原样
        assert_eq!(clean_read_level(None, 1, Some(2)).unwrap(), 2);
        // 发帖人后来被降级：原来的阅读权限原样保留是允许的，但不能借编辑设成别的更高值
        assert_eq!(clean_read_level(Some(2), 1, Some(2)).unwrap(), 2);
        assert!(clean_read_level(Some(3), 1, Some(2)).is_err());
        // 往下调总是允许
        assert_eq!(clean_read_level(Some(0), 1, Some(2)).unwrap(), 0);
    }

    #[test]
    fn required_post_level_treats_admin_only_as_guardian() {
        // 普通分类看最低发帖等级
        assert_eq!(required_post_level(1, false), 1);
        assert_eq!(required_post_level(0, false), 0);
        assert_eq!(required_post_level(3, false), 3);
        // 「仅管理员可发」= 守护者，不管最低发帖等级怎么设
        assert_eq!(required_post_level(1, true), 3);
        assert_eq!(required_post_level(0, true), 3);
        // 越界的最低发帖等级被夹在 0..=3
        assert_eq!(required_post_level(9, false), 3);
        assert_eq!(required_post_level(-1, false), 0);
        // 守护者能发，TL2 不能
        assert!(3 >= required_post_level(1, true));
        assert!(2 < required_post_level(1, true));
    }

    #[test]
    fn level_error_names_the_right_requirement() {
        assert!(level_error(true, 3).to_string().contains("守护者"));
        assert!(level_error(false, 2).to_string().contains("等级 2"));
    }

    #[test]
    fn normalize_tag_is_idempotent() {
        assert_eq!(normalize_tag(" #Windows "), Some("windows".into()));
        assert_eq!(normalize_tag("cc 授权"), Some("cc-授权".into()));
        assert_eq!(normalize_tag("  "), None);
        assert_eq!(normalize_tag("#"), None);
        let once = normalize_tag("Open AI").unwrap();
        assert_eq!(normalize_tag(&once), Some(once));
    }

    #[test]
    fn tags_are_normalized() {
        let tags = clean_tags(&["#Claude".into(), "claude".into(), " Open AI ".into(), "".into()]).unwrap();
        assert_eq!(tags, vec!["claude", "open-ai"]);
        assert!(clean_tags(&(0..6).map(|i| format!("t{i}")).collect::<Vec<_>>()).is_err());
    }
}

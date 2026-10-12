use super::attachments;
use crate::{
    auth::{AuthUser, MaybeUser},
    error::{AppError, AppResult},
    models::{self, ListFilter, Page, Reader, TopicItem},
    state::AppState,
    users::{self, PublicUser},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// 登录后和每次打开页面时调用：同步账号中心昵称头像、记访问、检查升级，返回本站资料。
pub async fn session(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Value>> {
    let uid = auth.user.uid;
    users::sync_profile(&state.http, &state.config.account_issuer, &auth.token, &state.pool, uid).await;
    users::record_visit(&state.pool, &state.config.trust, uid).await?;
    let user = users::find(&state.pool, uid).await?.ok_or_else(|| AppError::not_found("用户"))?;
    let level = user.trust_level.clamp(0, 3) as usize;
    let t = &state.config.trust;
    Ok(Json(json!({
        "user": user,
        "limits": {
            "daily_topics": t.daily_topics[level],
            "daily_posts": t.daily_posts[level],
            "tl1_days": t.tl1_days,
            "tl1_reads": t.tl1_reads,
            "image_max_bytes": if user.trust_level >= 1 { state.config.upload_limit_image } else { 0 },
            "upload_max_bytes": if user.trust_level >= 1 {
                attachments::upload_limit(&state.config, user.trust_level)
            } else {
                0
            },
        },
    })))
}

pub async fn bookmarks(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListFilter>,
) -> AppResult<Json<Page<TopicItem>>> {
    // 收藏之后帖子的阅读权限可能被调高，等级不够的不再显示。
    let level = auth.user.trust_level.clamp(0, 3);
    let ids: Vec<(i64,)> = sqlx::query_as(
        "SELECT b.topic_id FROM bookmarks b
         JOIN topics t ON t.id = b.topic_id AND t.status = 'normal' AND (t.min_read_level <= $2 OR t.author_uid = $1)
         WHERE b.uid = $1 ORDER BY b.created_at DESC LIMIT $3 OFFSET $4",
    )
    .bind(auth.user.uid)
    .bind(level)
    .bind(q.page_size())
    .bind((q.page() - 1) * q.page_size())
    .fetch_all(&state.pool)
    .await?;
    let (total,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bookmarks b
         JOIN topics t ON t.id = b.topic_id AND t.status = 'normal' AND (t.min_read_level <= $2 OR t.author_uid = $1)
         WHERE b.uid = $1",
    )
    .bind(auth.user.uid)
    .bind(level)
    .fetch_one(&state.pool)
    .await?;
    let mut items = Vec::with_capacity(ids.len());
    for (id,) in ids {
        if let Some(t) = models::get_topic(&state.pool, id).await? {
            items.push(t);
        }
    }
    Ok(Json(Page { items, page: q.page(), page_size: q.page_size(), total }))
}

#[derive(Serialize)]
pub struct Profile {
    #[serde(flatten)]
    pub user: PublicUser,
    pub points: i32,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub topic_count: i64,
    pub post_count: i64,
    pub deal_count: i64,
    pub banned: bool,
}

pub async fn profile(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Path(uid): Path<Uuid>,
) -> AppResult<Json<Profile>> {
    let reader = Reader::from_user(viewer.as_ref());
    let user = users::find(&state.pool, uid).await?.ok_or_else(|| AppError::not_found("用户"))?;
    let (topic_count, post_count, deal_count): (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM topics t WHERE t.author_uid = $1 AND t.status = 'normal'
                   AND (t.min_read_level <= $2 OR t.author_uid = $3)),
                (SELECT COUNT(*) FROM posts p JOIN topics t ON t.id = p.topic_id
                 WHERE p.author_uid = $1 AND p.status = 'normal' AND (t.min_read_level <= $2 OR t.author_uid = $3)),
                (SELECT COUNT(*) FROM topics t JOIN deals d ON d.topic_id = t.id
                 WHERE t.author_uid = $1 AND t.status = 'normal' AND (t.min_read_level <= $2 OR t.author_uid = $3))",
    )
    .bind(uid)
    .bind(reader.level)
    .bind(reader.uid)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(Profile {
        user: PublicUser { uid, nickname: user.nickname, avatar_url: user.avatar_url, trust_level: user.trust_level },
        points: user.points,
        created_at: user.created_at,
        last_seen_at: user.last_seen_at,
        topic_count,
        post_count,
        deal_count,
        banned: user.status == "banned",
    }))
}

pub async fn user_topics(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Path(uid): Path<Uuid>,
    Query(mut f): Query<ListFilter>,
) -> AppResult<Json<Page<TopicItem>>> {
    f.reader = Reader::from_user(viewer.as_ref());
    f.author = Some(uid);
    f.include_expired = true;
    if f.sort.is_none() {
        f.sort = Some("new".into());
    }
    Ok(Json(models::list_topics(&state.pool, &f).await?))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserReply {
    pub id: i64,
    pub topic_id: i64,
    pub topic_title: String,
    pub floor: i32,
    pub excerpt: String,
    pub created_at: DateTime<Utc>,
}

pub async fn user_posts(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Path(uid): Path<Uuid>,
    Query(q): Query<ListFilter>,
) -> AppResult<Json<Value>> {
    // 回复所在的帖子读者没有权限时，连帖子标题和回复摘要都不能露出来。
    let reader = Reader::from_user(viewer.as_ref());
    let rows: Vec<UserReply> = sqlx::query_as(
        "SELECT p.id, p.topic_id, t.title AS topic_title, p.floor, left(p.body_md, 160) AS excerpt, p.created_at
         FROM posts p JOIN topics t ON t.id = p.topic_id AND t.status = 'normal'
              AND (t.min_read_level <= $4 OR t.author_uid = $5)
         WHERE p.author_uid = $1 AND p.status = 'normal'
         ORDER BY p.created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(uid)
    .bind(q.page_size())
    .bind((q.page() - 1) * q.page_size())
    .bind(reader.level)
    .bind(reader.uid)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({ "items": rows, "page": q.page(), "page_size": q.page_size() })))
}

/// 搜索帖子（标题和摘要）。
pub async fn search(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Query(mut f): Query<ListFilter>,
) -> AppResult<Json<Page<TopicItem>>> {
    f.reader = Reader::from_user(viewer.as_ref());
    let q = f.q.clone().unwrap_or_default();
    let q = q.trim();
    if q.chars().count() < 1 || q.chars().count() > 50 {
        return Err(AppError::BadRequest("搜索词需要 1-50 个字".into()));
    }
    if f.sort.is_none() {
        f.sort = Some("new".into());
    }
    f.include_expired = true;
    Ok(Json(models::list_topics(&state.pool, &f).await?))
}

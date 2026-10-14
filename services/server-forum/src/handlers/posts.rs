use crate::{
    auth::{AuthUser, MaybeUser},
    error::{AppError, AppResult},
    markdown,
    models::{self, Reader},
    state::AppState,
    users::{self, PublicUser},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Serialize)]
pub struct PostItem {
    pub id: i64,
    pub floor: i32,
    pub reply_to: Option<i64>,
    /// 被回复楼层号，前端展示"回复 #3"
    pub reply_to_floor: Option<i32>,
    pub author: PublicUser,
    pub body_html: String,
    pub status: String,
    pub like_count: i32,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub liked: bool,
    pub mine: bool,
}

#[derive(sqlx::FromRow)]
struct PostRow {
    id: i64,
    floor: i32,
    reply_to: Option<i64>,
    reply_to_floor: Option<i32>,
    author_uid: Uuid,
    nickname: String,
    avatar_url: Option<String>,
    trust_level: i16,
    body_html: String,
    status: String,
    like_count: i32,
    created_at: DateTime<Utc>,
    edited_at: Option<DateTime<Utc>>,
    liked: bool,
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(topic_id): Path<i64>,
    Query(q): Query<PageQuery>,
) -> AppResult<Json<Value>> {
    models::ensure_readable(&state.pool, &Reader::from_user(user.as_ref()), topic_id).await?;
    let page = q.page.unwrap_or(1).clamp(1, 10_000);
    let size = q.page_size.unwrap_or(50).clamp(1, 100);
    let viewer = user.as_ref().map(|u| u.uid);
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE topic_id = $1")
        .bind(topic_id)
        .fetch_one(&state.pool)
        .await?;
    let rows: Vec<PostRow> = sqlx::query_as(
        "SELECT p.id, p.floor, p.reply_to, r.floor AS reply_to_floor, p.author_uid, u.nickname, u.avatar_url, u.trust_level,
                p.body_html, p.status, p.like_count, p.created_at, p.edited_at,
                ($4::uuid IS NOT NULL AND EXISTS(SELECT 1 FROM likes l WHERE l.target_type = 'post' AND l.target_id = p.id AND l.uid = $4)) AS liked
         FROM posts p JOIN users u ON u.uid = p.author_uid LEFT JOIN posts r ON r.id = p.reply_to
         WHERE p.topic_id = $1 ORDER BY p.floor LIMIT $2 OFFSET $3",
    )
    .bind(topic_id)
    .bind(size)
    .bind((page - 1) * size)
    .bind(viewer)
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<PostItem> = rows
        .into_iter()
        .map(|r| {
            // 被删除或隐藏的楼层保留位置，只显示占位文字。
            let visible = r.status == "normal";
            PostItem {
                id: r.id,
                floor: r.floor,
                reply_to: r.reply_to,
                reply_to_floor: r.reply_to_floor,
                mine: viewer == Some(r.author_uid),
                author: PublicUser { uid: r.author_uid, nickname: r.nickname, avatar_url: r.avatar_url, trust_level: r.trust_level },
                body_html: if visible { r.body_html } else { String::new() },
                status: r.status,
                like_count: r.like_count,
                created_at: r.created_at,
                edited_at: r.edited_at,
                liked: r.liked,
            }
        })
        .collect();
    Ok(Json(json!({ "items": items, "page": page, "page_size": size, "total": total })))
}

#[derive(Deserialize)]
pub struct PostInput {
    pub body: String,
    pub reply_to: Option<i64>,
}

fn clean_reply(raw: &str) -> AppResult<String> {
    let body = raw.trim().to_string();
    let len = body.chars().count();
    if !(2..=10_000).contains(&len) {
        return Err(AppError::BadRequest("回复需要 2-10000 个字".into()));
    }
    Ok(body)
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(topic_id): Path<i64>,
    Json(input): Json<PostInput>,
) -> AppResult<Json<Value>> {
    let user = &auth.user;
    users::ensure_can_write(user)?;
    models::ensure_readable(&state.pool, &Reader::from_user(Some(user)), topic_id).await?;
    let topic: Option<(String, bool)> = sqlx::query_as("SELECT status, locked FROM topics WHERE id = $1")
        .bind(topic_id)
        .fetch_optional(&state.pool)
        .await?;
    match topic {
        Some((status, _)) if status != "normal" => return Err(AppError::not_found("帖子")),
        Some((_, true)) => return Err(AppError::Forbidden("帖子已锁定，不能回复".into())),
        None => return Err(AppError::not_found("帖子")),
        _ => {}
    }
    users::ensure_daily_quota(&state.pool, user, "posts", &state.config.trust.daily_posts, "回复").await?;
    let body = clean_reply(&input.body)?;
    if let Some(reply_to) = input.reply_to {
        let ok: Option<(i64,)> = sqlx::query_as("SELECT id FROM posts WHERE id = $1 AND topic_id = $2")
            .bind(reply_to)
            .bind(topic_id)
            .fetch_optional(&state.pool)
            .await?;
        if ok.is_none() {
            return Err(AppError::BadRequest("回复的楼层不存在".into()));
        }
    }
    let html = markdown::render(&body, user.trust_level >= 1);

    // 锁住帖子行分配楼层号，避免并发回复撞号。
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT id FROM topics WHERE id = $1 FOR UPDATE").bind(topic_id).execute(&mut *tx).await?;
    let (id, floor): (i64, i32) = sqlx::query_as(
        "INSERT INTO posts (topic_id, author_uid, floor, reply_to, body_md, body_html)
         VALUES ($1, $2, (SELECT COALESCE(MAX(floor), 0) + 1 FROM posts WHERE topic_id = $1), $3, $4, $5)
         RETURNING id, floor",
    )
    .bind(topic_id)
    .bind(user.uid)
    .bind(input.reply_to)
    .bind(&body)
    .bind(&html)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE topics SET reply_count = reply_count + 1, last_reply_at = now(), last_reply_uid = $2 WHERE id = $1",
    )
    .bind(topic_id)
    .bind(user.uid)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    models::refresh_hot_scores(&state.pool, Some(topic_id)).await?;
    Ok(Json(json!({ "id": id, "floor": floor })))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Json(input): Json<PostInput>,
) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    let body = clean_reply(&input.body)?;
    let html = markdown::render(&body, auth.user.trust_level >= 1);
    let mut tx = state.pool.begin().await?;
    lock_topic_for_post(&mut tx, id).await?;
    let done = sqlx::query(
        "UPDATE posts SET body_md = $3, body_html = $4, edited_at = now()
         WHERE id = $1 AND author_uid = $2 AND status = 'normal'
           AND NOT (SELECT locked FROM topics WHERE topics.id = posts.topic_id)
           AND NOT EXISTS(SELECT 1 FROM help_requests WHERE accepted_post_id = posts.id)",
    )
    .bind(id)
    .bind(auth.user.uid)
    .bind(&body)
    .bind(&html)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if done == 0 {
        return Err(AppError::Forbidden("只能编辑自己的未采纳回复，已采纳回答保留原文".into()));
    }
    tx.commit().await?;
    Ok(Json(json!({ "ok": true, "body_html": html })))
}

pub async fn delete(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    lock_topic_for_post(&mut tx, id).await?;
    let topic: Option<(i64,)> = sqlx::query_as(
        "UPDATE posts SET status = 'deleted' WHERE id = $1 AND author_uid = $2 AND status = 'normal'
         AND NOT EXISTS(SELECT 1 FROM help_requests WHERE accepted_post_id = posts.id) RETURNING topic_id",
    )
    .bind(id)
    .bind(auth.user.uid)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((topic_id,)) = topic else {
        return Err(AppError::Forbidden("只能删除自己的未采纳回复".into()));
    };
    sqlx::query("UPDATE topics SET reply_count = GREATEST(reply_count - 1, 0) WHERE id = $1")
        .bind(topic_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

pub(super) async fn lock_topic_for_post(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, id: i64) -> AppResult<()> {
    let (topic,): (i64,) = sqlx::query_as("SELECT topic_id FROM posts WHERE id = $1")
        .bind(id).fetch_optional(&mut **tx).await?.ok_or_else(|| AppError::not_found("回复"))?;
    sqlx::query("SELECT id FROM topics WHERE id = $1 FOR UPDATE").bind(topic).execute(&mut **tx).await?;
    Ok(())
}

/// 编辑回复时取原文。
pub async fn source(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT body_md FROM posts WHERE id = $1 AND author_uid = $2 AND status = 'normal'")
        .bind(id)
        .bind(auth.user.uid)
        .fetch_optional(&state.pool)
        .await?;
    let (body,) = row.ok_or_else(|| AppError::not_found("回复"))?;
    Ok(Json(json!({ "body": body })))
}

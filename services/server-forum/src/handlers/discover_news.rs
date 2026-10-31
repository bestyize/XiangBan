//! 站内日报与评论。只公开已发布的条目；评论使用论坛账号、屏蔽与敏感词规则。
use crate::{auth::{AuthUser, MaybeUser}, error::{AppError, AppResult}, markdown, state::AppState, users};
use axum::{extract::{Path, Query, State}, Json};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

async fn article(pool: &PgPool, key: &str) -> AppResult<Value> {
    let row: Option<(DateTime<Utc>, Value)> = sqlx::query_as(
        "SELECT published_at,data FROM discover_items WHERE kind='news' AND key=$1 AND status='published' AND published_at<=now()"
    ).bind(key).fetch_optional(pool).await?;
    let (published, mut data) = row.ok_or_else(|| AppError::not_found("日报"))?;
    let body = data.get("body").and_then(Value::as_str).unwrap_or("");
    // Editorial Markdown comes from the reviewed publication workflow and still goes through ammonia.
    let html = markdown::render(body, true);
    if let Some(obj) = data.as_object_mut() {
        obj.remove("body");
        obj.insert("key".into(), json!(key));
        obj.insert("published_at".into(), json!(published));
        obj.insert("body_html".into(), json!(html));
    }
    Ok(data)
}

async fn ensure_article(pool: &PgPool, key: &str) -> AppResult<()> {
    let (published,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM discover_items WHERE kind='news' AND key=$1 AND status='published' AND published_at<=now())")
        .bind(key).fetch_one(pool).await?;
    if published { Ok(()) } else { Err(AppError::not_found("日报")) }
}

pub async fn detail(State(state): State<AppState>, Path(key): Path<String>) -> AppResult<Json<Value>> {
    Ok(Json(article(&state.pool, &key).await?))
}

#[derive(Deserialize)]
pub struct PageQuery { page: Option<i64> }

pub async fn list(State(state): State<AppState>, MaybeUser(viewer): MaybeUser, Path(key): Path<String>, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    ensure_article(&state.pool, &key).await?;
    let uid = viewer.map(|v| v.uid);
    let page = q.page.unwrap_or(1).clamp(1, 10_000);
    let rows: Vec<(i64, Option<i64>, String, DateTime<Utc>, Uuid, String, Option<String>, i64, bool)> = sqlx::query_as(
        "SELECT c.id,c.reply_to,c.body,c.created_at,u.uid,u.nickname,u.avatar_url,
         (SELECT count(*) FROM discover_news_comment_likes l WHERE l.comment_id=c.id),
         EXISTS(SELECT 1 FROM discover_news_comment_likes l WHERE l.comment_id=c.id AND l.uid=$2)
         FROM discover_news_comments c JOIN users u ON u.uid=c.author_uid
         WHERE c.news_key=$1 AND c.status='normal' AND u.status<>'banned'
         AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$2 AND b.blocked=c.author_uid)
         ORDER BY c.id DESC LIMIT 30 OFFSET $3"
    ).bind(&key).bind(uid).bind((page-1)*30).fetch_all(&state.pool).await?;
    let (total,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_news_comments c JOIN users u ON u.uid=c.author_uid WHERE c.news_key=$1 AND c.status='normal' AND u.status<>'banned' AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$2 AND b.blocked=c.author_uid)")
        .bind(&key).bind(uid).fetch_one(&state.pool).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id,reply_to,body,created_at,author,nickname,avatar_url,likes,liked)| json!({
        "id":id,"reply_to":reply_to,"body":body,"created_at":created_at,"author": {"uid":author,"nickname":nickname,"avatar_url":avatar_url},
        "like_count":likes,"liked":liked,"mine":uid==Some(author)
    })).collect();
    Ok(Json(json!({"items":items,"page":page,"page_size":30,"total":total})))
}

#[derive(Deserialize)]
pub struct CommentInput { body: String, reply_to: Option<i64> }

pub async fn create(State(state): State<AppState>, auth: AuthUser, Path(key): Path<String>, Json(input): Json<CommentInput>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    ensure_article(&state.pool, &key).await?;
    let body = input.body.trim();
    if !(2..=2000).contains(&body.chars().count()) { return Err(AppError::BadRequest("评论需要 2–2000 个字".into())); }
    if crate::moderation::screen(&state.pool, &[body]).await?.is_some() {
        return Err(AppError::BadRequest("评论暂不能发布，请修改后重试".into()));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))").bind(format!("news-comment:{}", auth.user.uid)).execute(&mut *tx).await?;
    let (recent, daily): (bool, i64) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM discover_news_comments WHERE author_uid=$1 AND created_at>now()-interval '15 seconds'), count(*) FROM discover_news_comments WHERE author_uid=$1 AND created_at>=date_trunc('day',now())")
        .bind(auth.user.uid).fetch_one(&mut *tx).await?;
    if recent || daily >= 50 { return Err(AppError::TooManyRequests { message: "评论太频繁，请稍后再试".into(), retry_after: 15 }); }
    if let Some(parent) = input.reply_to {
        let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM discover_news_comments WHERE id=$1 AND news_key=$2 AND status='normal')")
            .bind(parent).bind(&key).fetch_one(&mut *tx).await?;
        if !exists { return Err(AppError::BadRequest("回复的评论不存在".into())); }
    }
    let (id,): (i64,) = sqlx::query_as("INSERT INTO discover_news_comments(news_key,author_uid,reply_to,body) VALUES($1,$2,$3,$4) RETURNING id")
        .bind(key).bind(auth.user.uid).bind(input.reply_to).bind(body).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id})))
}

pub async fn delete(State(state): State<AppState>, auth: AuthUser, Path((key,id)): Path<(String,i64)>) -> AppResult<Json<Value>> {
    ensure_article(&state.pool, &key).await?;
    let done = sqlx::query("UPDATE discover_news_comments SET status='deleted' WHERE news_key=$1 AND id=$2 AND author_uid=$3 AND status='normal'")
        .bind(&key).bind(id).bind(auth.user.uid).execute(&state.pool).await?;
    if done.rows_affected()==0 { return Err(AppError::not_found("评论")); }
    Ok(Json(json!({"ok":true})))
}

pub async fn like(State(state): State<AppState>, auth: AuthUser, Path((key,id)): Path<(String,i64)>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    ensure_article(&state.pool, &key).await?;
    let done = sqlx::query("INSERT INTO discover_news_comment_likes(comment_id,uid) SELECT c.id,$3 FROM discover_news_comments c JOIN users u ON u.uid=c.author_uid WHERE c.news_key=$1 AND c.id=$2 AND c.status='normal' AND u.status<>'banned' AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$3 AND b.blocked=c.author_uid) ON CONFLICT DO NOTHING")
        .bind(&key).bind(id).bind(auth.user.uid).execute(&state.pool).await?;
    // Repeated likes are idempotent; an unavailable comment is checked separately.
    if done.rows_affected()==0 {
        let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM discover_news_comment_likes l JOIN discover_news_comments c ON c.id=l.comment_id JOIN users u ON u.uid=c.author_uid WHERE c.news_key=$1 AND c.id=$2 AND l.uid=$3 AND c.status='normal' AND u.status<>'banned' AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$3 AND b.blocked=c.author_uid))")
            .bind(&key).bind(id).bind(auth.user.uid).fetch_one(&state.pool).await?;
        if !exists { return Err(AppError::not_found("评论")); }
    }
    Ok(Json(json!({"ok":true})))
}

pub async fn unlike(State(state): State<AppState>, auth: AuthUser, Path((key,id)): Path<(String,i64)>) -> AppResult<Json<Value>> {
    ensure_article(&state.pool, &key).await?;
    sqlx::query("DELETE FROM discover_news_comment_likes l USING discover_news_comments c WHERE l.comment_id=c.id AND c.news_key=$1 AND c.id=$2 AND l.uid=$3")
        .bind(key).bind(id).bind(auth.user.uid).execute(&state.pool).await?;
    Ok(Json(json!({"ok":true})))
}

//! 站内日报与精选作品的评论。两者共用账号、图片附件、屏蔽与敏感词规则。
use crate::{auth::{AuthUser, MaybeUser}, error::{AppError, AppResult}, markdown, state::AppState, users};
use axum::{extract::{Path, Query, State}, routing::{delete as delete_route, get, put}, Extension, Json, Router};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use super::images::{self, CommentImageTarget};

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

// 日报的 key 只允许小写字母、数字、连字符和下划线。作品用详情路径作存储键，
// 因此沿用原评论表和附件关联也不会与同名日报冲突。
const IDEA_KEY_PREFIX: &str = "/ideas/works/";

#[derive(Clone, Copy)]
pub(super) enum CommentScope {
    Daily,
    Idea,
}

pub(super) fn comment_routes(path: &str, scope: CommentScope) -> Router<AppState> {
    Router::new()
        .route(path, get(list).post(create))
        .route(&format!("{path}/{{id}}"), delete_route(delete))
        .route(&format!("{path}/{{id}}/like"), put(like).delete(unlike))
        .layer(Extension(scope))
}

#[derive(Deserialize)]
struct IdeaCatalog {
    items: Vec<IdeaCatalogItem>,
}

#[derive(Deserialize)]
struct IdeaCatalogItem {
    id: String,
    kind: String,
}

impl CommentScope {
    async fn key(self, state: &AppState, key: String) -> AppResult<String> {
        if matches!(self, Self::Daily) {
            if key.starts_with(IDEA_KEY_PREFIX) {
                return Err(AppError::not_found("日报"));
            }
            ensure_article(&state.pool, &key).await?;
            return Ok(key);
        }

        if !crate::discover::valid_key(&key) {
            return Err(AppError::not_found("创意作品"));
        }
        // 与网页和手机读取同一份随网页产物发布的目录。
        let bytes = tokio::fs::read(state.config.web_dir.join("ideas-catalog.json")).await.map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::not_found("创意作品")
            } else {
                AppError::Internal(format!("读取创意目录失败: {error}"))
            }
        })?;
        let catalog: IdeaCatalog = serde_json::from_slice(&bytes)
            .map_err(|error| AppError::Internal(format!("创意目录格式错误: {error}")))?;
        if !catalog.items.iter().any(|item| item.id == key && matches!(item.kind.as_str(), "work" | "exhibit")) {
            return Err(AppError::not_found("创意作品"));
        }
        Ok(format!("{IDEA_KEY_PREFIX}{key}"))
    }
}

#[derive(Deserialize)]
struct PageQuery { page: Option<i64> }

async fn list(State(state): State<AppState>, Extension(scope): Extension<CommentScope>, MaybeUser(viewer): MaybeUser, Path(key): Path<String>, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let key = scope.key(&state, key).await?;
    let uid = viewer.map(|v| v.uid);
    let page = q.page.unwrap_or(1).clamp(1, 10_000);
    let rows: Vec<(i64, Option<i64>, String, DateTime<Utc>, Uuid, String, Option<String>, i64, bool, Vec<Uuid>)> = sqlx::query_as(
        "SELECT c.id,c.reply_to,c.body,c.created_at,u.uid,u.nickname,u.avatar_url,
         (SELECT count(*) FROM discover_news_comment_likes l WHERE l.comment_id=c.id),
         EXISTS(SELECT 1 FROM discover_news_comment_likes l WHERE l.comment_id=c.id AND l.uid=$2),
         ARRAY(SELECT a.image_id FROM discover_comment_images a WHERE a.daily_comment_id=c.id ORDER BY a.position)
         FROM discover_news_comments c JOIN users u ON u.uid=c.author_uid
         WHERE c.news_key=$1 AND c.status='normal' AND u.status<>'banned'
         AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$2 AND b.blocked=c.author_uid)
         ORDER BY c.id DESC LIMIT 30 OFFSET $3"
    ).bind(&key).bind(uid).bind((page-1)*30).fetch_all(&state.pool).await?;
    let (total,): (i64,) = sqlx::query_as("SELECT count(*) FROM discover_news_comments c JOIN users u ON u.uid=c.author_uid WHERE c.news_key=$1 AND c.status='normal' AND u.status<>'banned' AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$2 AND b.blocked=c.author_uid)")
        .bind(&key).bind(uid).fetch_one(&state.pool).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id,reply_to,body,created_at,author,nickname,avatar_url,likes,liked,image_ids)| json!({
        "id":id,"reply_to":reply_to,"body":body,"created_at":created_at,"author": {"uid":author,"nickname":nickname,"avatar_url":avatar_url},
        "like_count":likes,"liked":liked,"mine":uid==Some(author),"images":images::comment_urls(image_ids)
    })).collect();
    Ok(Json(json!({"items":items,"page":page,"page_size":30,"total":total})))
}

#[derive(Deserialize)]
struct CommentInput {
    body: String,
    reply_to: Option<i64>,
    #[serde(default)]
    image_ids: Vec<Uuid>,
}

async fn create(State(state): State<AppState>, Extension(scope): Extension<CommentScope>, auth: AuthUser, Path(key): Path<String>, Json(input): Json<CommentInput>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    let key = scope.key(&state, key).await?;
    let body = input.body.trim();
    images::validate_comment_content(body, &input.image_ids)?;
    if crate::moderation::screen(&state.pool, &[body]).await?.is_some() {
        return Err(AppError::BadRequest("评论暂不能发布，请修改后重试".into()));
    }
    let mut tx = state.pool.begin().await?;
    let is_idea = matches!(scope, CommentScope::Idea);
    let lock = if is_idea { "idea-comment" } else { "news-comment" };
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))").bind(format!("{lock}:{}", auth.user.uid)).execute(&mut *tx).await?;
    let (recent, daily): (bool, i64) = sqlx::query_as(
        "SELECT coalesce(bool_or(created_at>now()-interval '15 seconds'), false),
                count(*) FILTER (WHERE created_at>=date_trunc('day',now()))
         FROM discover_news_comments WHERE author_uid=$1
           AND created_at>=LEAST(date_trunc('day',now()),now()-interval '15 seconds')
           AND starts_with(news_key,$2)=$3"
    ).bind(auth.user.uid).bind(IDEA_KEY_PREFIX).bind(is_idea).fetch_one(&mut *tx).await?;
    if recent || daily >= 50 { return Err(AppError::TooManyRequests { message: "评论太频繁，请稍后再试".into(), retry_after: 15 }); }
    if let Some(parent) = input.reply_to {
        let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM discover_news_comments WHERE id=$1 AND news_key=$2 AND status='normal')")
            .bind(parent).bind(&key).fetch_one(&mut *tx).await?;
        if !exists { return Err(AppError::BadRequest("回复的评论不存在".into())); }
    }
    let (id,): (i64,) = sqlx::query_as("INSERT INTO discover_news_comments(news_key,author_uid,reply_to,body) VALUES($1,$2,$3,$4) RETURNING id")
        .bind(key).bind(auth.user.uid).bind(input.reply_to).bind(body).fetch_one(&mut *tx).await?;
    images::attach_to_comment(&mut tx, CommentImageTarget::Daily(id), auth.user.uid, &input.image_ids).await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id})))
}

async fn delete(State(state): State<AppState>, Extension(scope): Extension<CommentScope>, auth: AuthUser, Path((key,id)): Path<(String,i64)>) -> AppResult<Json<Value>> {
    let key = scope.key(&state, key).await?;
    let (deleted,): (bool,) = sqlx::query_as(
        "WITH deleted AS (
            UPDATE discover_news_comments SET status='deleted'
            WHERE news_key=$1 AND id=$2 AND author_uid=$3 AND status='normal' RETURNING id
         ), removed_images AS (
            DELETE FROM discover_comment_images WHERE daily_comment_id IN (SELECT id FROM deleted)
         ) SELECT EXISTS(SELECT 1 FROM deleted)")
        .bind(&key).bind(id).bind(auth.user.uid).fetch_one(&state.pool).await?;
    if !deleted { return Err(AppError::not_found("评论")); }
    Ok(Json(json!({"ok":true})))
}

async fn like(State(state): State<AppState>, Extension(scope): Extension<CommentScope>, auth: AuthUser, Path((key,id)): Path<(String,i64)>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    let key = scope.key(&state, key).await?;
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

async fn unlike(State(state): State<AppState>, Extension(scope): Extension<CommentScope>, auth: AuthUser, Path((key,id)): Path<(String,i64)>) -> AppResult<Json<Value>> {
    let key = scope.key(&state, key).await?;
    sqlx::query("DELETE FROM discover_news_comment_likes l USING discover_news_comments c WHERE l.comment_id=c.id AND c.news_key=$1 AND c.id=$2 AND l.uid=$3")
        .bind(key).bind(id).bind(auth.user.uid).execute(&state.pool).await?;
    Ok(Json(json!({"ok":true})))
}

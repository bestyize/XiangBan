//! 榜单项目详情页：仓库卡片、中文 README 和本站评论。只公开出现在已发布榜单里的项目；
//! 评论沿用论坛账号、屏蔽与敏感词规则。
use crate::{
    auth::{AuthUser, MaybeUser},
    collectors::repo_sources::valid_name,
    error::{AppError, AppResult},
    moderation, repo_details,
    state::AppState,
    users,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

const PUBLISHED: &str = "kind='repo_issue' AND status='published' AND published_at<=now()";
const HAS_REPO: &str = "EXISTS (SELECT 1 FROM jsonb_array_elements(data->'ranks') r WHERE lower(r->>'full_name')=$1)";

fn parse(owner: &str, repo: &str) -> AppResult<String> {
    let name = format!("{owner}/{repo}");
    if !valid_name(&name) {
        return Err(AppError::not_found("项目"));
    }
    Ok(name.to_ascii_lowercase())
}

async fn ensure_repo(pool: &PgPool, key: &str) -> AppResult<()> {
    let (found,): (bool,) = sqlx::query_as(&format!("SELECT EXISTS(SELECT 1 FROM discover_items WHERE {PUBLISHED} AND {HAS_REPO})")).bind(key).fetch_one(pool).await?;
    if found { Ok(()) } else { Err(AppError::not_found("项目")) }
}

type Detail = (String, String, String, String, String, bool, Option<DateTime<Utc>>);

pub async fn detail(State(state): State<AppState>, Path((owner, repo)): Path<(String, String)>) -> AppResult<Json<Value>> {
    let key = parse(&owner, &repo)?;
    let pool = &state.pool;
    // 取最近一期含该项目的榜单；项目掉出本周榜后仍可打开，卡片显示它最近一次上榜的数据。
    let found: Option<(String, Value)> = sqlx::query_as(&format!(
        "SELECT key,data FROM discover_items WHERE {PUBLISHED} AND {HAS_REPO} ORDER BY (data->>'issue_no')::int DESC LIMIT 1"
    ))
    .bind(&key)
    .fetch_optional(pool)
    .await?;
    let (issue_key, issue) = found.ok_or_else(|| AppError::not_found("项目"))?;
    let ranks = issue["ranks"].as_array().cloned().unwrap_or_default();
    let (index, rank) = ranks
        .iter()
        .enumerate()
        .find(|(_, r)| r["full_name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(&key)))
        .ok_or_else(|| AppError::not_found("项目"))?;
    let latest: Option<(String,)> = sqlx::query_as(&format!("SELECT key FROM discover_items WHERE {PUBLISHED} ORDER BY (data->>'issue_no')::int DESC LIMIT 1")).fetch_optional(pool).await?;
    let row: Option<Detail> = sqlx::query_as("SELECT desc_zh,readme_status,readme_md,readme_zh,readme_zh_hash,readme_zh_truncated,fetched_at FROM repo_details WHERE key=$1")
        .bind(&key)
        .fetch_optional(pool)
        .await?;
    let name = rank["full_name"].as_str().unwrap_or_default();
    let (owner_name, repo_name) = name.split_once('/').unwrap_or((name, ""));
    let text = |field: &str| rank[field].as_str().unwrap_or_default().to_string();
    let (desc_zh_row, status, md, zh_md, _, zh_truncated, fetched_at) = row.unwrap_or_default();
    let zh = if rank["description_zh"].as_str().is_some_and(|d| !d.is_empty()) { text("description_zh") } else { desc_zh_row };
    let status = if status.is_empty() { "unavailable".to_string() } else { status };
    let blocks = if status == "ok" { repo_details::readme_blocks(&md, true) } else { Vec::new() };
    let zh_blocks = if status == "ok" && !zh_md.trim().is_empty() { Some(repo_details::readme_blocks(&zh_md, true)) } else { None };
    Ok(Json(json!({
        "full_name":name,"owner":owner_name,"repo":repo_name,"url":format!("https://github.com/{name}"),
        "description":text("description"),"description_zh":zh,"language":text("language"),"language_color":text("language_color"),
        "star_total":rank["star_total"],"star_gain":rank["star_gain"],
        "rank":index + 1,"is_latest":latest.is_some_and(|(k,)| k == issue_key),"issue_key":issue_key,
        "week_end":issue["week_end"],"as_of":issue["as_of"],"metric":issue["metric"],"source":issue["source"],
        "readme":{"status":status,"blocks":blocks,"zh_blocks":zh_blocks,"zh_truncated":zh_blocks.is_some() && zh_truncated,"fetched_at":fetched_at},
    })))
}

#[derive(Deserialize)]
pub struct PageQuery {
    page: Option<i64>,
}

pub async fn list(State(state): State<AppState>, MaybeUser(viewer): MaybeUser, Path((owner, repo)): Path<(String, String)>, Query(q): Query<PageQuery>) -> AppResult<Json<Value>> {
    let key = parse(&owner, &repo)?;
    ensure_repo(&state.pool, &key).await?;
    let uid = viewer.map(|v| v.uid);
    let page = q.page.unwrap_or(1).clamp(1, 10_000);
    let rows: Vec<(i64, Option<i64>, String, DateTime<Utc>, Uuid, String, Option<String>, i64, bool)> = sqlx::query_as(
        "SELECT c.id,c.reply_to,c.body,c.created_at,u.uid,u.nickname,u.avatar_url,
         (SELECT count(*) FROM repo_comment_likes l WHERE l.comment_id=c.id),
         EXISTS(SELECT 1 FROM repo_comment_likes l WHERE l.comment_id=c.id AND l.uid=$2)
         FROM repo_comments c JOIN users u ON u.uid=c.author_uid
         WHERE c.repo_key=$1 AND c.status='normal' AND u.status<>'banned'
         AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$2 AND b.blocked=c.author_uid)
         ORDER BY c.id DESC LIMIT 30 OFFSET $3",
    )
    .bind(&key)
    .bind(uid)
    .bind((page - 1) * 30)
    .fetch_all(&state.pool)
    .await?;
    let (total,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM repo_comments c JOIN users u ON u.uid=c.author_uid WHERE c.repo_key=$1 AND c.status='normal' AND u.status<>'banned'
         AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$2 AND b.blocked=c.author_uid)",
    )
    .bind(&key)
    .bind(uid)
    .fetch_one(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, reply_to, body, created_at, author, nickname, avatar_url, likes, liked)| {
            json!({"id":id,"reply_to":reply_to,"body":body,"created_at":created_at,"author":{"uid":author,"nickname":nickname,"avatar_url":avatar_url},
                "like_count":likes,"liked":liked,"mine":uid == Some(author)})
        })
        .collect();
    Ok(Json(json!({"items":items,"page":page,"page_size":30,"total":total})))
}

#[derive(Deserialize)]
pub struct CommentInput {
    body: String,
    reply_to: Option<i64>,
}

pub async fn create(State(state): State<AppState>, auth: AuthUser, Path((owner, repo)): Path<(String, String)>, Json(input): Json<CommentInput>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    let key = parse(&owner, &repo)?;
    ensure_repo(&state.pool, &key).await?;
    let body = input.body.trim();
    if !(2..=2000).contains(&body.chars().count()) {
        return Err(AppError::BadRequest("评论需要 2–2000 个字".into()));
    }
    if moderation::screen(&state.pool, &[body]).await?.is_some() {
        return Err(AppError::BadRequest("评论暂不能发布，请修改后重试".into()));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))").bind(format!("repo-comment:{}", auth.user.uid)).execute(&mut *tx).await?;
    let (recent, daily): (bool, i64) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM repo_comments WHERE author_uid=$1 AND created_at>now()-interval '15 seconds'),
         count(*) FROM repo_comments WHERE author_uid=$1 AND created_at>=date_trunc('day',now())",
    )
    .bind(auth.user.uid)
    .fetch_one(&mut *tx)
    .await?;
    if recent || daily >= 50 {
        return Err(AppError::TooManyRequests { message: "评论太频繁，请稍后再试".into(), retry_after: 15 });
    }
    if let Some(parent) = input.reply_to {
        let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM repo_comments WHERE id=$1 AND repo_key=$2 AND status='normal')").bind(parent).bind(&key).fetch_one(&mut *tx).await?;
        if !exists {
            return Err(AppError::BadRequest("回复的评论不存在".into()));
        }
    }
    let (id,): (i64,) = sqlx::query_as("INSERT INTO repo_comments(repo_key,author_uid,reply_to,body) VALUES($1,$2,$3,$4) RETURNING id")
        .bind(&key)
        .bind(auth.user.uid)
        .bind(input.reply_to)
        .bind(body)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id})))
}

pub async fn delete(State(state): State<AppState>, auth: AuthUser, Path((owner, repo, id)): Path<(String, String, i64)>) -> AppResult<Json<Value>> {
    let key = parse(&owner, &repo)?;
    let done = sqlx::query("UPDATE repo_comments SET status='deleted' WHERE repo_key=$1 AND id=$2 AND author_uid=$3 AND status='normal'")
        .bind(&key)
        .bind(id)
        .bind(auth.user.uid)
        .execute(&state.pool)
        .await?;
    if done.rows_affected() == 0 {
        return Err(AppError::not_found("评论"));
    }
    Ok(Json(json!({"ok":true})))
}

pub async fn like(State(state): State<AppState>, auth: AuthUser, Path((owner, repo, id)): Path<(String, String, i64)>) -> AppResult<Json<Value>> {
    users::ensure_can_write(&auth.user)?;
    let key = parse(&owner, &repo)?;
    let done = sqlx::query(
        "INSERT INTO repo_comment_likes(comment_id,uid) SELECT c.id,$3 FROM repo_comments c JOIN users u ON u.uid=c.author_uid
         WHERE c.repo_key=$1 AND c.id=$2 AND c.status='normal' AND u.status<>'banned'
         AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$3 AND b.blocked=c.author_uid) ON CONFLICT DO NOTHING",
    )
    .bind(&key)
    .bind(id)
    .bind(auth.user.uid)
    .execute(&state.pool)
    .await?;
    // 重复点赞是幂等的；评论不可见时才报不存在。
    if done.rows_affected() == 0 {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM repo_comment_likes l JOIN repo_comments c ON c.id=l.comment_id JOIN users u ON u.uid=c.author_uid
             WHERE c.repo_key=$1 AND c.id=$2 AND l.uid=$3 AND c.status='normal' AND u.status<>'banned'
             AND NOT EXISTS(SELECT 1 FROM user_blocks b WHERE b.blocker=$3 AND b.blocked=c.author_uid))",
        )
        .bind(&key)
        .bind(id)
        .bind(auth.user.uid)
        .fetch_one(&state.pool)
        .await?;
        if !exists {
            return Err(AppError::not_found("评论"));
        }
    }
    Ok(Json(json!({"ok":true})))
}

pub async fn unlike(State(state): State<AppState>, auth: AuthUser, Path((owner, repo, id)): Path<(String, String, i64)>) -> AppResult<Json<Value>> {
    let key = parse(&owner, &repo)?;
    sqlx::query("DELETE FROM repo_comment_likes l USING repo_comments c WHERE l.comment_id=c.id AND c.repo_key=$1 AND c.id=$2 AND l.uid=$3")
        .bind(&key)
        .bind(id)
        .bind(auth.user.uid)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"ok":true})))
}

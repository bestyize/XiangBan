//! 求助悬赏：积分先暂存，只有采纳或退款两种结算路径。
use crate::{auth::AuthUser, error::{AppError, AppResult}, models::Reader, points, state::AppState};
use axum::{extract::{Path, State}, Json};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

fn default_days() -> i64 { 7 }
#[derive(Deserialize)]
pub struct HelpInput {
    #[serde(default)] pub bounty: i32,
    #[serde(default = "default_days")] pub days: i64,
}

#[derive(sqlx::FromRow)]
struct Request {
    bounty: i32, status: String, expires_at: DateTime<Utc>,
}

pub async fn create_for_topic(
    tx: &mut Transaction<'_, Postgres>, id: i64, uid: Uuid, category: i32, input: Option<&HelpInput>,
) -> AppResult<()> {
    let (is_help,): (bool,) = sqlx::query_as("SELECT slug = 'share-help' FROM categories WHERE id = $1")
        .bind(category).fetch_one(&mut **tx).await?;
    if !is_help {
        if input.is_some() { return Err(AppError::BadRequest("只有资源求助板块可以设置悬赏".into())); }
        return Ok(());
    }
    let bounty = input.map_or(0, |i| i.bounty);
    let days = input.map_or(7, |i| i.days);
    if !(0..=1000).contains(&bounty) || ![7, 14, 30].contains(&days) {
        return Err(AppError::BadRequest("悬赏为 0～1000 的整数，有效期可选 7、14、30 天".into()));
    }
    points::lock_users(tx, &[uid]).await?;
    points::change(tx, uid, -bounty, "bounty_escrow", &format!("bounty_escrow:{id}"),
        None, &id.to_string(), "发布求助，暂存悬赏积分").await?;
    sqlx::query("INSERT INTO help_requests (topic_id, bounty, expires_at) VALUES ($1, $2, $3)")
        .bind(id).bind(bounty).bind(Utc::now() + Duration::days(days)).execute(&mut **tx).await?;
    Ok(())
}

pub async fn ensure_category(tx: &mut Transaction<'_, Postgres>, id: i64, category: i32) -> AppResult<()> {
    let (is_help, has_help, author): (bool, bool, Uuid) = sqlx::query_as(
        "SELECT c.slug = 'share-help', EXISTS(SELECT 1 FROM help_requests WHERE topic_id = $1), t.author_uid
         FROM categories c CROSS JOIN topics t WHERE c.id = $2 AND t.id = $1",
    ).bind(id).bind(category).fetch_one(&mut **tx).await?;
    if has_help && !is_help { return Err(AppError::BadRequest("求助帖子不能移出资源求助板块，请保留求助记录".into())); }
    if is_help && !has_help { create_for_topic(tx, id, author, category, None).await?; }
    Ok(())
}

#[derive(Deserialize)]
pub struct AcceptInput { pub post_id: i64 }

pub async fn accept(
    State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>, Json(input): Json<AcceptInput>,
) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let (author, status, min_read): (Uuid, String, i16) = sqlx::query_as(
        "SELECT author_uid, status, min_read_level FROM topics WHERE id = $1 FOR UPDATE",
    ).bind(id).fetch_optional(&mut *tx).await?.ok_or_else(|| AppError::not_found("帖子"))?;
    Reader::from_user(Some(&auth.user)).check(min_read, author)?;
    if author != auth.user.uid || status != "normal" { return Err(AppError::Forbidden("只有求助者可以采纳正常帖子中的回答".into())); }
    let request: Request = sqlx::query_as("SELECT bounty, status, expires_at FROM help_requests WHERE topic_id = $1 FOR UPDATE")
        .bind(id).fetch_optional(&mut *tx).await?.ok_or_else(|| AppError::not_found("求助"))?;
    if request.status != "open" || request.expires_at <= Utc::now() { return Err(AppError::Conflict("求助已结束或到期，不能再采纳".into())); }
    let (winner,): (Uuid,) = sqlx::query_as(
        "SELECT p.author_uid FROM posts p JOIN users u ON u.uid = p.author_uid
         WHERE p.id = $1 AND p.topic_id = $2 AND p.status = 'normal' AND u.status <> 'banned' FOR UPDATE OF p",
    ).bind(input.post_id).bind(id).fetch_optional(&mut *tx).await?.ok_or_else(|| AppError::BadRequest("回答不存在或已不可见".into()))?;
    if winner == author { return Err(AppError::BadRequest("不能采纳自己的回答".into())); }
    points::lock_users(&mut tx, &[author, winner]).await?;
    points::change(&mut tx, winner, request.bounty, "bounty_income", &format!("bounty_settle:{id}"),
        Some(author), &id.to_string(), "回答被采纳，获得悬赏").await?;
    sqlx::query("UPDATE help_requests SET status = 'solved', accepted_post_id = $2, settled_at = now() WHERE topic_id = $1")
        .bind(id).bind(input.post_id).execute(&mut *tx).await?;
    let (title,): (String,) = sqlx::query_as("SELECT title FROM topics WHERE id = $1").bind(id).fetch_one(&mut *tx).await?;
    crate::notifications::notify(&mut tx, winner, &crate::notifications::Note {
        kind: crate::notifications::ACCEPTED, actor: Some(author), topic_id: Some(id), post_id: Some(input.post_id),
        pack_id: None, title: &title, excerpt: "",
    }).await?;
    tx.commit().await?;
    Ok(Json(json!({ "status": "solved", "accepted_post_id": input.post_id, "bounty": request.bounty })))
}

/// 调用方已锁帖子行。隐藏、删除、到期和主动关闭都通过这里退款。
pub async fn refund(tx: &mut Transaction<'_, Postgres>, id: i64, outcome: &str) -> AppResult<i32> {
    let request: Option<Request> = sqlx::query_as("SELECT bounty, status, expires_at FROM help_requests WHERE topic_id = $1 FOR UPDATE")
        .bind(id).fetch_optional(&mut **tx).await?;
    let Some(request) = request.filter(|r| r.status == "open") else { return Ok(0) };
    let (author,): (Uuid,) = sqlx::query_as("SELECT author_uid FROM topics WHERE id = $1")
        .bind(id).fetch_one(&mut **tx).await?;
    points::lock_users(tx, &[author]).await?;
    let returned = points::change(tx, author, request.bounty, "bounty_refund", &format!("bounty_settle:{id}"),
        None, &id.to_string(), if outcome == "expired" { "求助到期，退还悬赏" } else { "求助关闭，退还悬赏" }).await?;
    sqlx::query("UPDATE help_requests SET status = $2, settled_at = now() WHERE topic_id = $1")
        .bind(id).bind(outcome).execute(&mut **tx).await?;
    Ok(returned)
}

pub async fn close(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let (author,): (Uuid,) = sqlx::query_as("SELECT author_uid FROM topics WHERE id = $1 FOR UPDATE")
        .bind(id).fetch_optional(&mut *tx).await?.ok_or_else(|| AppError::not_found("帖子"))?;
    if author != auth.user.uid { return Err(AppError::Forbidden("只有求助者可以关闭求助".into())); }
    let (status, expires): (String, DateTime<Utc>) = sqlx::query_as("SELECT status, expires_at FROM help_requests WHERE topic_id = $1")
        .bind(id).fetch_optional(&mut *tx).await?.ok_or_else(|| AppError::not_found("求助"))?;
    if status == "solved" { return Err(AppError::Conflict("已经采纳回答，不能退还悬赏".into())); }
    let outcome = if expires <= Utc::now() { "expired" } else { "closed" };
    let returned = refund(&mut tx, id, outcome).await?;
    tx.commit().await?;
    Ok(Json(json!({ "status": if status == "open" { outcome } else { &status }, "refunded": returned })))
}

pub async fn expire(pool: &PgPool) -> AppResult<usize> {
    let ids: Vec<(i64,)> = sqlx::query_as(
        "SELECT h.topic_id FROM help_requests h JOIN topics t ON t.id = h.topic_id
         WHERE h.status = 'open' AND (h.expires_at <= now() OR t.status <> 'normal') ORDER BY h.topic_id LIMIT 200",
    ).fetch_all(pool).await?;
    for (id,) in &ids {
        let mut tx = pool.begin().await?;
        sqlx::query("SELECT id FROM topics WHERE id = $1 FOR UPDATE").bind(id).execute(&mut *tx).await?;
        refund(&mut tx, *id, "expired").await?;
        tx.commit().await?;
    }
    Ok(ids.len())
}

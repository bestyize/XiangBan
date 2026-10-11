//! 点赞、收藏、举报。

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{self, Reader},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

async fn toggle_like(state: &AppState, auth: &AuthUser, target: &str, id: i64, on: bool) -> AppResult<Json<Value>> {
    let table = if target == "topic" { "topics" } else { "posts" };
    // 没有阅读权限的帖子不能点赞（否则能靠猜 id 探测）。
    let reader = Reader::from_user(Some(&auth.user));
    if target == "topic" {
        models::ensure_readable(&state.pool, &reader, id).await?;
    } else {
        models::ensure_post_readable(&state.pool, &reader, id).await?;
    }
    let exists: Option<(i64,)> = sqlx::query_as(&format!("SELECT id FROM {table} WHERE id = $1 AND status = 'normal'"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("内容"));
    }
    let changed = if on {
        sqlx::query("INSERT INTO likes (target_type, target_id, uid) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
    } else {
        sqlx::query("DELETE FROM likes WHERE target_type = $1 AND target_id = $2 AND uid = $3")
    }
    .bind(target)
    .bind(id)
    .bind(auth.user.uid)
    .execute(&state.pool)
    .await?
    .rows_affected();
    let delta = if changed == 0 { 0 } else if on { 1 } else { -1 };
    let (count,): (i32,) = sqlx::query_as(&format!(
        "UPDATE {table} SET like_count = GREATEST(like_count + $2, 0) WHERE id = $1 RETURNING like_count"
    ))
    .bind(id)
    .bind(delta)
    .fetch_one(&state.pool)
    .await?;
    if target == "topic" && delta != 0 {
        models::refresh_hot_scores(&state.pool, Some(id)).await?;
    }
    Ok(Json(json!({ "liked": on, "like_count": count })))
}

pub async fn like_topic(State(s): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    toggle_like(&s, &auth, "topic", id, true).await
}
pub async fn unlike_topic(State(s): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    toggle_like(&s, &auth, "topic", id, false).await
}
pub async fn like_post(State(s): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    toggle_like(&s, &auth, "post", id, true).await
}
pub async fn unlike_post(State(s): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    toggle_like(&s, &auth, "post", id, false).await
}

pub async fn bookmark(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    models::ensure_readable(&state.pool, &Reader::from_user(Some(&auth.user)), id).await?;
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM topics WHERE id = $1 AND status = 'normal'")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("帖子"));
    }
    sqlx::query("INSERT INTO bookmarks (uid, topic_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(auth.user.uid)
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "bookmarked": true })))
}

pub async fn unbookmark(State(state): State<AppState>, auth: AuthUser, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    sqlx::query("DELETE FROM bookmarks WHERE uid = $1 AND topic_id = $2")
        .bind(auth.user.uid)
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "bookmarked": false })))
}

pub const REPORT_REASONS: &[&str] = &["spam", "abuse", "illegal", "copyright", "account_trade", "politics", "other"];

#[derive(Deserialize)]
pub struct ReportInput {
    pub target_type: String,
    pub target_id: i64,
    pub reason: String,
    #[serde(default)]
    pub detail: String,
}

pub async fn report(State(state): State<AppState>, auth: AuthUser, Json(input): Json<ReportInput>) -> AppResult<Json<Value>> {
    let table = match input.target_type.as_str() {
        "topic" => "topics",
        "post" => "posts",
        _ => return Err(AppError::BadRequest("举报对象无效".into())),
    };
    if !REPORT_REASONS.contains(&input.reason.as_str()) {
        return Err(AppError::BadRequest("请选择举报原因".into()));
    }
    let reader = Reader::from_user(Some(&auth.user));
    if input.target_type == "topic" {
        models::ensure_readable(&state.pool, &reader, input.target_id).await?;
    } else {
        models::ensure_post_readable(&state.pool, &reader, input.target_id).await?;
    }
    let detail: String = input.detail.trim().chars().take(500).collect();
    let exists: Option<(i64,)> = sqlx::query_as(&format!("SELECT id FROM {table} WHERE id = $1"))
        .bind(input.target_id)
        .fetch_optional(&state.pool)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("内容"));
    }
    let (today,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM reports WHERE reporter_uid = $1 AND created_at > now() - interval '1 day'",
    )
    .bind(auth.user.uid)
    .fetch_one(&state.pool)
    .await?;
    if today >= 30 {
        return Err(AppError::TooManyRequests { message: "今天举报次数太多了".into(), retry_after: 3600 });
    }
    sqlx::query(
        "INSERT INTO reports (target_type, target_id, reporter_uid, reason, detail) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (target_type, target_id, reporter_uid) DO UPDATE SET reason = EXCLUDED.reason, detail = EXCLUDED.detail",
    )
    .bind(&input.target_type)
    .bind(input.target_id)
    .bind(auth.user.uid)
    .bind(&input.reason)
    .bind(detail)
    .execute(&state.pool)
    .await?;
    Ok(Json(json!({ "ok": true })))
}

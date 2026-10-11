//! 投票、点赞、收藏、举报。

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{self, DEAL_STATUSES},
    state::AppState,
    users,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct VoteInput {
    /// valid / invalid / none（撤销）
    pub value: String,
}

async fn deal_snapshot(state: &AppState, topic_id: i64) -> AppResult<Value> {
    let (status, valid, invalid, verified): (String, i32, i32, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "SELECT deal_status, valid_votes, invalid_votes, last_verified_at FROM deals WHERE topic_id = $1",
    )
    .bind(topic_id)
    .fetch_one(&state.pool)
    .await?;
    Ok(json!({ "deal_status": status, "valid_votes": valid, "invalid_votes": invalid, "last_verified_at": verified }))
}

pub async fn vote(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(topic_id): Path<i64>,
    Json(input): Json<VoteInput>,
) -> AppResult<Json<Value>> {
    let user = &auth.user;
    users::ensure_can_write(user)?;
    if user.trust_level < 1 {
        return Err(AppError::Forbidden("信任等级 1 以上才能投票".into()));
    }
    let deal: Option<(String,)> = sqlx::query_as(
        "SELECT t.status FROM deals d JOIN topics t ON t.id = d.topic_id WHERE d.topic_id = $1",
    )
    .bind(topic_id)
    .fetch_optional(&state.pool)
    .await?;
    match deal {
        Some((s,)) if s == "normal" => {}
        _ => return Err(AppError::not_found("羊毛")),
    }
    let weight: i16 = if user.trust_level >= 2 { 2 } else { 1 };
    match input.value.as_str() {
        "none" => {
            sqlx::query("DELETE FROM deal_votes WHERE topic_id = $1 AND uid = $2")
                .bind(topic_id)
                .bind(user.uid)
                .execute(&state.pool)
                .await?;
        }
        "valid" | "invalid" => {
            let value: i16 = if input.value == "valid" { 1 } else { -1 };
            sqlx::query(
                "INSERT INTO deal_votes (topic_id, uid, value, weight) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (topic_id, uid) DO UPDATE SET value = EXCLUDED.value, weight = EXCLUDED.weight, updated_at = now()",
            )
            .bind(topic_id)
            .bind(user.uid)
            .bind(value)
            .bind(weight)
            .execute(&state.pool)
            .await?;
            if value == 1 {
                sqlx::query("UPDATE deals SET last_verified_at = now() WHERE topic_id = $1")
                    .bind(topic_id)
                    .execute(&state.pool)
                    .await?;
            }
        }
        _ => return Err(AppError::BadRequest("投票只能是有效或失效".into())),
    }
    models::recompute_deal(&state.pool, topic_id).await?;
    models::refresh_hot_scores(&state.pool, Some(topic_id)).await?;
    let mut snapshot = deal_snapshot(&state, topic_id).await?;
    snapshot["vote"] = json!(match input.value.as_str() { "valid" => 1, "invalid" => -1, _ => 0 });
    Ok(Json(snapshot))
}

#[derive(Deserialize)]
pub struct StatusInput {
    pub status: String,
}

/// TL3 守护者直接标记羊毛状态。
pub async fn set_deal_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(topic_id): Path<i64>,
    Json(input): Json<StatusInput>,
) -> AppResult<Json<Value>> {
    if auth.user.trust_level < 3 {
        return Err(AppError::Forbidden("需要守护者权限".into()));
    }
    if !DEAL_STATUSES.contains(&input.status.as_str()) {
        return Err(AppError::BadRequest("状态无效".into()));
    }
    let done = sqlx::query("UPDATE deals SET deal_status = $2 WHERE topic_id = $1")
        .bind(topic_id)
        .bind(&input.status)
        .execute(&state.pool)
        .await?
        .rows_affected();
    if done == 0 {
        return Err(AppError::not_found("羊毛"));
    }
    Ok(Json(deal_snapshot(&state, topic_id).await?))
}

async fn toggle_like(state: &AppState, auth: &AuthUser, target: &str, id: i64, on: bool) -> AppResult<Json<Value>> {
    let table = if target == "topic" { "topics" } else { "posts" };
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

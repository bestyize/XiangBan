//! 屏蔽用户：屏蔽后互相取消关注；对方的帖子不再出现在列表和搜索里，回复折叠，也不会再给你发通知。
//! 对方看不出自己被屏蔽（不报错、不提示），所以不会因此被激怒或去换号骚扰。

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn block(State(state): State<AppState>, auth: AuthUser, Path(uid): Path<Uuid>) -> AppResult<Json<Value>> {
    if uid == auth.user.uid {
        return Err(AppError::BadRequest("不能屏蔽自己".into()));
    }
    let exists: Option<(Uuid,)> = sqlx::query_as("SELECT uid FROM users WHERE uid = $1").bind(uid).fetch_optional(&state.pool).await?;
    if exists.is_none() {
        return Err(AppError::not_found("用户"));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("INSERT INTO user_blocks (blocker, blocked) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(auth.user.uid)
        .bind(uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM user_follows WHERE (follower_uid = $1 AND followed_uid = $2) OR (follower_uid = $2 AND followed_uid = $1)")
        .bind(auth.user.uid)
        .bind(uid)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({ "blocked": true })))
}

pub async fn unblock(State(state): State<AppState>, auth: AuthUser, Path(uid): Path<Uuid>) -> AppResult<Json<Value>> {
    sqlx::query("DELETE FROM user_blocks WHERE blocker = $1 AND blocked = $2").bind(auth.user.uid).bind(uid).execute(&state.pool).await?;
    Ok(Json(json!({ "blocked": false })))
}

pub async fn list(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Value>> {
    let rows: Vec<(Uuid, String, Option<String>, i16, DateTime<Utc>)> = sqlx::query_as(
        "SELECT u.uid, u.nickname, u.avatar_url, u.trust_level, b.created_at
         FROM user_blocks b JOIN users u ON u.uid = b.blocked WHERE b.blocker = $1 ORDER BY b.created_at DESC LIMIT 500",
    )
    .bind(auth.user.uid)
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(uid, nickname, avatar_url, trust_level, at)| json!({ "uid": uid, "nickname": nickname, "avatar_url": avatar_url, "trust_level": trust_level, "blocked_at": at }))
        .collect();
    Ok(Json(json!({ "items": items })))
}

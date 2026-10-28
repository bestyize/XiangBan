//! 我的通知：列表、未读数、标记已读。

use crate::{
    auth::AuthUser,
    error::AppResult,
    models::Page,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    page: Option<i64>,
    #[serde(default)]
    unread: bool,
}

pub async fn list(State(state): State<AppState>, auth: AuthUser, Query(q): Query<ListQuery>) -> AppResult<Json<Page<Value>>> {
    let page = q.page.unwrap_or(1).max(1);
    let size = 30;
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications WHERE uid = $1 AND ($2 = false OR read_at IS NULL)")
        .bind(auth.user.uid)
        .bind(q.unread)
        .fetch_one(&state.pool)
        .await?;
    type Row = (i64, String, Option<Uuid>, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>, String, String, bool, DateTime<Utc>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT n.id, n.kind, n.actor_uid, u.nickname, u.avatar_url, n.topic_id, n.post_id, n.pack_id, n.title, n.excerpt,
                n.read_at IS NOT NULL, n.created_at
         FROM notifications n LEFT JOIN users u ON u.uid = n.actor_uid
         WHERE n.uid = $1 AND ($2 = false OR n.read_at IS NULL)
         ORDER BY n.id DESC LIMIT $3 OFFSET $4",
    )
    .bind(auth.user.uid)
    .bind(q.unread)
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&state.pool)
    .await?;
    let items = rows
        .into_iter()
        .map(|(id, kind, actor_uid, nickname, avatar, topic_id, post_id, pack_id, title, excerpt, read, at)| {
            json!({
                "id": id, "kind": kind,
                "actor": actor_uid.map(|uid| json!({ "uid": uid, "nickname": nickname, "avatar_url": avatar })),
                "topic_id": topic_id, "post_id": post_id, "pack_id": pack_id,
                "title": title, "excerpt": excerpt, "read": read, "created_at": at,
            })
        })
        .collect();
    Ok(Json(Page { items, page, page_size: size, total }))
}

pub async fn unread_count(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Value>> {
    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications WHERE uid = $1 AND read_at IS NULL")
        .bind(auth.user.uid)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(json!({ "unread": n })))
}

#[derive(Deserialize)]
pub struct ReadInput {
    /// 不传就是全部标为已读。
    ids: Option<Vec<i64>>,
}

pub async fn mark_read(State(state): State<AppState>, auth: AuthUser, body: axum::body::Bytes) -> AppResult<Json<Value>> {
    // 不带请求体（或 null）就是全部已读。
    let ids = match body.iter().all(u8::is_ascii_whitespace) || body.as_ref() == b"null" {
        true => None,
        false => serde_json::from_slice::<ReadInput>(&body).map_err(|_| crate::error::AppError::BadRequest("请求内容无效".into()))?.ids,
    };
    let done = match ids {
        Some(ids) => sqlx::query("UPDATE notifications SET read_at = now() WHERE uid = $1 AND read_at IS NULL AND id = ANY($2)")
            .bind(auth.user.uid)
            .bind(&ids)
            .execute(&state.pool)
            .await?,
        None => sqlx::query("UPDATE notifications SET read_at = now() WHERE uid = $1 AND read_at IS NULL")
            .bind(auth.user.uid)
            .execute(&state.pool)
            .await?,
    };
    Ok(Json(json!({ "marked": done.rows_affected() })))
}

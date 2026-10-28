//! 后台：敏感词表与待审队列（内容包、帖子、回复、方案命中复核词或刚发布的未审内容都会进队列）。

use crate::{
    error::{AppError, AppResult},
    handlers::admin::{audit, AuthAdmin},
    moderation::{clean_word, BLOCK, REVIEW},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/words", get(list_words).post(add_words))
        .route("/words/{id}", delete(delete_word))
        .route("/moderation", get(queue))
        .route("/moderation/{id}/resolve", post(resolve))
}

async fn list_words(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Value>> {
    let rows: Vec<(i32, String, String, DateTime<Utc>)> =
        sqlx::query_as("SELECT id, word, action, created_at FROM sensitive_words ORDER BY action, word").fetch_all(&state.pool).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id, word, action, at)| json!({ "id": id, "word": word, "action": action, "created_at": at })).collect();
    Ok(Json(json!({ "items": items })))
}

#[derive(Deserialize)]
pub struct AddWords {
    /// 每行一个词，也可以用逗号分隔。
    words: String,
    /// `block` 拦截，`review` 复核。
    action: String,
}

async fn add_words(State(state): State<AppState>, AuthAdmin(admin): AuthAdmin, Json(input): Json<AddWords>) -> AppResult<Json<Value>> {
    if input.action != BLOCK && input.action != REVIEW {
        return Err(AppError::BadRequest("action 只能是 block 或 review".into()));
    }
    let words: Vec<String> = input.words.split(['\n', ',', '，', '、']).filter_map(clean_word).collect();
    if words.is_empty() {
        return Err(AppError::BadRequest("没有可用的词：每个词至少 2 个字，最长 40 个字".into()));
    }
    if words.len() > 500 {
        return Err(AppError::BadRequest("一次最多添加 500 个词".into()));
    }
    let mut tx = state.pool.begin().await?;
    let mut added = 0;
    for word in &words {
        // 已有的词以最新一次的动作为准。
        let r = sqlx::query(
            "INSERT INTO sensitive_words (word, action, created_by) VALUES ($1, $2, $3)
             ON CONFLICT (word) DO UPDATE SET action = EXCLUDED.action",
        )
        .bind(word)
        .bind(&input.action)
        .bind(&admin.username)
        .execute(&mut *tx)
        .await?;
        added += r.rows_affected();
    }
    audit(&mut tx, &admin, "words.add", "words", 0, format!("{} 个，{}", words.len(), input.action)).await?;
    tx.commit().await?;
    Ok(Json(json!({ "added": added, "submitted": words.len() })))
}

async fn delete_word(State(state): State<AppState>, AuthAdmin(admin): AuthAdmin, Path(id): Path<i32>) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let deleted = sqlx::query("DELETE FROM sensitive_words WHERE id = $1").bind(id).execute(&mut *tx).await?;
    if deleted.rows_affected() == 0 {
        return Err(AppError::not_found("词条"));
    }
    audit(&mut tx, &admin, "words.delete", "words", id, "").await?;
    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

async fn queue(State(state): State<AppState>, _admin: AuthAdmin) -> AppResult<Json<Value>> {
    type Row = (i64, String, i64, String, DateTime<Utc>, Option<String>, Option<String>, Option<String>, Option<i64>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT q.id, q.target_type, q.target_id, q.note, q.created_at,
                COALESCE(p.title, t.title, left(po.body_md, 60), s.title) AS title,
                p.kind,
                COALESCE(pu.nickname, tu.nickname, pou.nickname, su.nickname) AS author,
                COALESCE(po.topic_id, t.id) AS topic_id
         FROM moderation_queue q
         LEFT JOIN packs p ON q.target_type = 'pack' AND p.id = q.target_id
         LEFT JOIN users pu ON pu.uid = p.author_uid
         LEFT JOIN topics t ON q.target_type = 'topic' AND t.id = q.target_id
         LEFT JOIN users tu ON tu.uid = t.author_uid
         LEFT JOIN posts po ON q.target_type = 'post' AND po.id = q.target_id
         LEFT JOIN users pou ON pou.uid = po.author_uid
         LEFT JOIN schemes s ON q.target_type = 'scheme' AND s.id = q.target_id
         LEFT JOIN users su ON su.uid = s.author_uid
         WHERE q.status = 'pending' ORDER BY q.created_at LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, target_type, target_id, note, created_at, title, kind, author, topic_id)| {
            // 帖子和回复要到帖子管理里隐藏（用 topic_id 链接过去）；这里只负责标记“已处理”。
            json!({ "id": id, "target_type": target_type, "target_id": target_id, "note": note, "created_at": created_at,
                    "title": title, "kind": kind, "author": author, "topic_id": topic_id })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

#[derive(Deserialize)]
pub struct Resolve {
    /// `approve`：没问题；`reject`：下架（只对内容包和方案；帖子与回复请在帖子管理里隐藏，再标记已处理）。
    action: String,
    #[serde(default)]
    note: String,
}

async fn resolve(State(state): State<AppState>, AuthAdmin(admin): AuthAdmin, Path(id): Path<i64>, Json(input): Json<Resolve>) -> AppResult<Json<Value>> {
    if input.action != "approve" && input.action != "reject" {
        return Err(AppError::BadRequest("action 只能是 approve 或 reject".into()));
    }
    if input.note.chars().count() > 200 {
        return Err(AppError::BadRequest("备注最长 200 个字".into()));
    }
    let mut tx = state.pool.begin().await?;
    let row: Option<(String, i64)> =
        sqlx::query_as("SELECT target_type, target_id FROM moderation_queue WHERE id = $1 AND status = 'pending' FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let (target_type, target_id) = row.ok_or_else(|| AppError::not_found("待审记录"))?;
    if input.action == "reject" {
        let done = match target_type.as_str() {
            "pack" => sqlx::query("UPDATE packs SET status = 'rejected', updated_at = now() WHERE id = $1").bind(target_id).execute(&mut *tx).await?,
            "scheme" => sqlx::query("UPDATE schemes SET status = 'removed' WHERE id = $1").bind(target_id).execute(&mut *tx).await?,
            _ => return Err(AppError::BadRequest("帖子和回复请在帖子管理里隐藏，再标记为已处理".into())),
        };
        if done.rows_affected() == 0 {
            return Err(AppError::not_found("内容"));
        }
    } else if target_type == "pack" {
        // 通过审核的内容包升为“已审核”，和后台内容包页的“通过”一致。
        sqlx::query("UPDATE packs SET trust_level = LEAST(trust_level, 1), updated_at = now() WHERE id = $1").bind(target_id).execute(&mut *tx).await?;
    }
    if target_type == "pack" {
        crate::handlers::content::notify_pack_author(&mut tx, target_id, &input.action, input.note.trim()).await?;
    }
    let status = if input.action == "approve" { "approved" } else { "rejected" };
    sqlx::query("UPDATE moderation_queue SET status = $2, note = CASE WHEN $3 = '' THEN note ELSE $3 END, handled_by = $4, handled_at = now() WHERE id = $1")
        .bind(id)
        .bind(status)
        .bind(input.note.trim())
        .bind(&admin.username)
        .execute(&mut *tx)
        .await?;
    audit(&mut tx, &admin, &format!("moderation.{}", input.action), &target_type, target_id, input.note.trim()).await?;
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "status": status })))
}

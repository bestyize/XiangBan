//! Works use the same topics/posts/likes and reply notifications as ordinary posts.
use super::{check_kind, require_visible};
use crate::{
    error::{AppError, AppResult},
    users::{self, ForumUser},
};
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

pub async fn state(
    pool: &PgPool,
    user: Option<&ForumUser>,
    kind: &str,
    id: i64,
) -> AppResult<Value> {
    let mut conn = pool.acquire().await?;
    require_visible(&mut conn, kind, id).await?;
    let topic: Option<(i64,)> = sqlx::query_as("SELECT topic_id FROM content_discussions WHERE kind=$1 AND target_id=$2 UNION SELECT topic_id FROM packs WHERE $1='pack' AND id=$2 AND topic_id IS NOT NULL LIMIT 1")
        .bind(kind).bind(id).fetch_optional(&mut *conn).await?;
    let writable = user.is_some_and(|u| u.status != "banned" && users::ensure_can_write(u).is_ok());
    let Some((topic_id,)) = topic else {
        return Ok(
            json!({"topic_id":null,"like_count":0,"reply_count":0,"liked":false,"can_reply":writable,"locked":false}),
        );
    };
    let row: Option<(i32,i32,bool,bool)> = sqlx::query_as("SELECT like_count,reply_count,locked,EXISTS(SELECT 1 FROM likes WHERE target_type='topic' AND target_id=t.id AND uid=$2) FROM topics t WHERE id=$1 AND status='normal' AND min_read_level=0")
        .bind(topic_id).bind(user.map(|u| u.uid)).fetch_optional(&mut *conn).await?;
    let (likes, replies, locked, liked) = row.ok_or_else(|| AppError::not_found("讨论"))?;
    Ok(
        json!({"topic_id":topic_id,"like_count":likes,"reply_count":replies,"liked":liked,"can_reply":writable && !locked,"locked":locked}),
    )
}

/// Only an explicit authenticated interaction creates a thread; reading never writes.
/// Lock the content row so concurrent clients cannot create duplicate discussions.
pub async fn ensure(pool: &PgPool, user: &ForumUser, kind: &str, id: i64) -> AppResult<Value> {
    check_kind(kind)?;
    users::ensure_can_write(user)?;
    let mut tx = pool.begin().await?;
    let table = if kind == "pack" { "packs" } else { "schemes" };
    let row: Option<(Option<Uuid>, String, String)> = sqlx::query_as(&format!(
        "SELECT author_uid,title,summary FROM {table} WHERE id=$1 FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let (author, title, summary) = row.ok_or_else(|| AppError::not_found("内容"))?;
    require_visible(&mut tx, kind, id).await?;
    let existing: Option<(i64,)> = sqlx::query_as("SELECT topic_id FROM content_discussions WHERE kind=$1 AND target_id=$2 UNION SELECT topic_id FROM packs WHERE $1='pack' AND id=$2 AND topic_id IS NOT NULL LIMIT 1")
        .bind(kind).bind(id).fetch_optional(&mut *tx).await?;
    let (topic_id, managed) = if let Some((topic_id,)) = existing {
        (topic_id, false)
    } else {
        // Imported works may have no account author. This identity is explicitly a
        // discussion host, never attributed to the visitor who first comments.
        let author = if let Some(author) = author {
            author
        } else {
            let uid = Uuid::parse_str("00000000-0000-4000-8000-000000000001").unwrap();
            sqlx::query(
                "INSERT INTO users(uid,nickname) VALUES($1,'社区作品讨论') ON CONFLICT DO NOTHING",
            )
            .bind(uid)
            .execute(&mut *tx)
            .await?;
            uid
        };
        let pack_kind: Option<String> = if kind == "pack" {
            sqlx::query_scalar("SELECT kind FROM packs WHERE id=$1")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
        } else {
            None
        };
        let category_slug = match pack_kind.as_deref() {
            Some("game") => "xiaoban-game",
            Some("wallpaper" | "effect" | "widget") | None => "xiaoban-wallpaper",
            _ => "tech-creative",
        };
        let (category,): (i32,) = sqlx::query_as("SELECT id FROM categories WHERE slug=$1")
            .bind(category_slug)
            .fetch_one(&mut *tx)
            .await?;
        let path = if kind == "pack" { "packs" } else { "schemes" };
        let body = format!("{summary}\n\n[查看作品](/{path}/{id})");
        let html = crate::markdown::render(&body, true);
        let (topic_id,): (i64,) = sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html,excerpt) VALUES($1,$2,$3,$4,$5,$6) RETURNING id")
            .bind(category).bind(author).bind(&title).bind(body).bind(html).bind(&summary).fetch_one(&mut *tx).await?;
        if kind == "pack" {
            sqlx::query("UPDATE packs SET topic_id=$2 WHERE id=$1")
                .bind(id)
                .bind(topic_id)
                .execute(&mut *tx)
                .await?;
        }
        (topic_id, true)
    };
    sqlx::query("INSERT INTO content_discussions(kind,target_id,topic_id,managed) VALUES($1,$2,$3,$4) ON CONFLICT(kind,target_id) DO NOTHING")
        .bind(kind).bind(id).bind(topic_id).bind(managed).execute(&mut *tx).await?;
    tx.commit().await?;
    state(pool, Some(user), kind, id).await
}

/// Apply content visibility even when a caller uses the normal topic/reply URL.
pub async fn ensure_visible(pool: &PgPool, topic_id: i64) -> AppResult<()> {
    let targets: Vec<(String,i64)> = sqlx::query_as("SELECT kind,target_id FROM content_discussions WHERE topic_id=$1 UNION SELECT 'pack',id FROM packs WHERE topic_id=$1")
        .bind(topic_id).fetch_all(pool).await?;
    if targets.is_empty() {
        return Ok(());
    }
    let mut conn = pool.acquire().await?;
    for (kind, id) in targets {
        require_visible(&mut conn, &kind, id).await?;
    }
    let (visible,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM topics WHERE id=$1 AND status='normal' AND min_read_level=0)",
    )
    .bind(topic_id)
    .fetch_one(&mut *conn)
    .await?;
    if !visible {
        return Err(AppError::not_found("讨论"));
    }
    Ok(())
}

/// Writers lock the parent before the topic so removal and an interaction serialize.
pub async fn lock_for_topic(conn: &mut PgConnection, topic_id: i64) -> AppResult<()> {
    let targets: Vec<(String,i64)> = sqlx::query_as("SELECT kind,target_id FROM content_discussions WHERE topic_id=$1 UNION SELECT 'pack',id FROM packs WHERE topic_id=$1 ORDER BY 1,2")
        .bind(topic_id).fetch_all(&mut *conn).await?;
    for (kind, id) in targets {
        let table = if kind == "pack" { "packs" } else { "schemes" };
        sqlx::query(&format!("SELECT id FROM {table} WHERE id=$1 FOR UPDATE"))
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?;
        require_visible(conn, &kind, id).await?;
    }
    Ok(())
}

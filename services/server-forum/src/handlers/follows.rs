use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{self, ListFilter, Page, Reader, TopicItem},
    state::AppState,
    users::{self, PublicUser},
};
use axum::{extract::{Path, Query, State}, Json};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct FollowState {
    pub following: bool,
    pub following_count: i64,
    pub follower_count: i64,
}

// 和列表使用相同的封禁过滤条件，不缓存计数，重复关注不会累加。
pub async fn stats(pool: &PgPool, uid: Uuid, viewer: Option<Uuid>) -> AppResult<FollowState> {
    let (following, following_count, follower_count): (bool, i64, i64) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM user_follows WHERE follower_uid = $2 AND followed_uid = $1),
                (SELECT COUNT(*) FROM user_follows f JOIN users u ON u.uid = f.followed_uid
                 WHERE f.follower_uid = $1 AND u.status <> 'banned'),
                (SELECT COUNT(*) FROM user_follows f JOIN users u ON u.uid = f.follower_uid
                 WHERE f.followed_uid = $1 AND u.status <> 'banned')",
    )
    .bind(uid)
    .bind(viewer)
    .fetch_one(pool)
    .await?;
    Ok(FollowState { following, following_count, follower_count })
}

pub async fn follow(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(uid): Path<Uuid>,
) -> AppResult<Json<FollowState>> {
    set_follow(&state.pool, auth.user.uid, uid, true).await.map(Json)
}

pub async fn unfollow(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(uid): Path<Uuid>,
) -> AppResult<Json<FollowState>> {
    set_follow(&state.pool, auth.user.uid, uid, false).await.map(Json)
}

async fn set_follow(pool: &PgPool, follower: Uuid, uid: Uuid, on: bool) -> AppResult<FollowState> {
    if on && follower == uid {
        return Err(AppError::BadRequest("不能关注自己".into()));
    }
    let target = users::find(pool, uid).await?.ok_or_else(|| AppError::not_found("用户"))?;
    if on && target.status == "banned" {
        return Err(AppError::Forbidden("该用户已被封禁，无法关注".into()));
    }
    let sql = if on {
        "INSERT INTO user_follows (follower_uid, followed_uid) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    } else {
        // 已封禁的目标也可以取消关注。
        "DELETE FROM user_follows WHERE follower_uid = $1 AND followed_uid = $2"
    };
    sqlx::query(sql)
        .bind(follower)
        .bind(uid)
        .execute(pool)
        .await?;
    stats(pool, uid, Some(follower)).await
}

pub async fn following(
    State(state): State<AppState>,
    Path(uid): Path<Uuid>,
    Query(q): Query<ListFilter>,
) -> AppResult<Json<Page<PublicUser>>> {
    connections(&state.pool, uid, &q, false).await.map(Json)
}

pub async fn followers(
    State(state): State<AppState>,
    Path(uid): Path<Uuid>,
    Query(q): Query<ListFilter>,
) -> AppResult<Json<Page<PublicUser>>> {
    connections(&state.pool, uid, &q, true).await.map(Json)
}

async fn connections(pool: &PgPool, uid: Uuid, q: &ListFilter, followers: bool) -> AppResult<Page<PublicUser>> {
    users::find(pool, uid).await?.ok_or_else(|| AppError::not_found("用户"))?;
    // 列名只来自内部常量，不拼接请求内容。
    let (owner, other) = if followers { ("followed_uid", "follower_uid") } else { ("follower_uid", "followed_uid") };
    let from = format!("FROM user_follows f JOIN users u ON u.uid = f.{other}
                       WHERE f.{owner} = $1 AND u.status <> 'banned'");
    let (total,): (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) {from}"))
        .bind(uid)
        .fetch_one(pool)
        .await?;
    let rows: Vec<(Uuid, String, Option<String>, i16)> = sqlx::query_as(&format!(
        "SELECT u.uid, u.nickname, u.avatar_url, u.trust_level {from}
         ORDER BY f.created_at DESC, u.uid LIMIT $2 OFFSET $3",
    ))
    .bind(uid)
    .bind(q.page_size())
    .bind((q.page() - 1) * q.page_size())
    .fetch_all(pool)
    .await?;
    let items = rows.into_iter().map(|(uid, nickname, avatar_url, trust_level)| {
        PublicUser { uid, nickname, avatar_url, trust_level }
    }).collect();
    Ok(Page { items, page: q.page(), page_size: q.page_size(), total })
}

pub async fn topics(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(mut f): Query<ListFilter>,
) -> AppResult<Json<Page<TopicItem>>> {
    f.reader = Reader::from_user(Some(&auth.user));
    f.followed_by = Some(auth.user.uid);
    f.home = false;
    f.include_expired = false;
    f.sort = Some("new".into());
    Ok(Json(models::list_topics(&state.pool, &f).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TEST_DATABASE_URL 指向测试库，每次使用独立 schema。
    #[tokio::test]
    async fn relationships_are_idempotent_and_paginated() -> Result<(), Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else { return Ok(()) };
        let admin = crate::db::connect(&url, None).await?;
        let schema = format!("follow_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        let result = async {
            crate::db::migrate(&pool).await?;
            let a = Uuid::new_v4();
            let b = Uuid::new_v4();
            let c = Uuid::new_v4();
            for uid in [a, b, c] {
                sqlx::query("INSERT INTO users (uid, nickname) VALUES ($1, 'test')")
                    .bind(uid).execute(&pool).await?;
            }
            assert!(set_follow(&pool, a, a, true).await.is_err());
            assert!(set_follow(&pool, a, Uuid::new_v4(), true).await.is_err());
            set_follow(&pool, a, b, true).await?;
            set_follow(&pool, a, b, true).await?;
            let b_stats = stats(&pool, b, Some(a)).await?;
            assert!(b_stats.following);
            assert_eq!(b_stats.follower_count, 1);
            assert_eq!(stats(&pool, a, None).await?.following_count, 1);
            assert!(!stats(&pool, b, None).await?.following);
            set_follow(&pool, a, c, true).await?;
            let q = ListFilter { page_size: Some(1), ..Default::default() };
            let first = connections(&pool, a, &q, false).await?;
            assert_eq!(first.total, 2);
            assert_eq!(first.items.len(), 1);
            let second = connections(&pool, a, &ListFilter { page: Some(2), ..q }, false).await?;
            assert_ne!(first.items[0].uid, second.items[0].uid);
            assert_eq!(connections(&pool, b, &ListFilter::default(), true).await?.items[0].uid, a);
            sqlx::query("UPDATE users SET status = 'banned' WHERE uid = $1").bind(b).execute(&pool).await?;
            assert!(set_follow(&pool, c, b, true).await.is_err());
            assert_eq!(stats(&pool, a, None).await?.following_count, 1);
            set_follow(&pool, a, b, false).await?;
            set_follow(&pool, a, b, false).await?;
            assert!(!stats(&pool, b, Some(a)).await?.following);
            Ok::<_, Box<dyn std::error::Error>>(())
        }.await;
        pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&admin).await?;
        result
    }
}

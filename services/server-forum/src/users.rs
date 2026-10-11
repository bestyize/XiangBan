use crate::{config::TrustRules, error::AppError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ForumUser {
    pub uid: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub trust_level: i16,
    pub points: i32,
    pub status: String,
    pub muted_until: Option<DateTime<Utc>>,
    pub read_count: i32,
    pub visit_days: i32,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

/// 其他用户可见的资料，不含状态和阅读统计。
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PublicUser {
    pub uid: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub trust_level: i16,
}

const COLUMNS: &str =
    "uid, nickname, avatar_url, trust_level, points, status, muted_until, read_count, visit_days, created_at, last_seen_at";

pub async fn find(pool: &PgPool, uid: Uuid) -> Result<Option<ForumUser>, AppError> {
    Ok(sqlx::query_as(&format!("SELECT {COLUMNS} FROM users WHERE uid = $1")).bind(uid).fetch_optional(pool).await?)
}

pub async fn load_or_create(pool: &PgPool, uid: Uuid) -> Result<ForumUser, AppError> {
    if let Some(user) = find(pool, uid).await? {
        return Ok(user);
    }
    // 昵称先用占位，前端登录后调用 /api/session 从账号中心同步。
    let placeholder = format!("小伴{}", &uid.simple().to_string()[..6]);
    sqlx::query("INSERT INTO users (uid, nickname) VALUES ($1, $2) ON CONFLICT (uid) DO NOTHING")
        .bind(uid)
        .bind(placeholder)
        .execute(pool)
        .await?;
    find(pool, uid).await?.ok_or_else(|| AppError::Internal("建档后读不到用户".into()))
}

#[derive(Deserialize)]
struct AccountMe {
    /// 账号中心里唯一且不可修改的用户名，论坛直接把它当作展示名。
    username: String,
    avatar_url: Option<String>,
}

/// 用用户自己的访问令牌读取账号中心资料。账号中心不可用时保留本地缓存，不影响登录。
pub async fn sync_profile(http: &reqwest::Client, account_base: &str, token: &str, pool: &PgPool, uid: Uuid) {
    let resp = http.get(format!("{account_base}/api/me")).bearer_auth(token).send().await;
    let me = match resp {
        Ok(r) if r.status().is_success() => r.json::<AccountMe>().await.ok(),
        Ok(r) => {
            tracing::warn!("账号中心 /api/me 返回 {}", r.status());
            None
        }
        Err(e) => {
            tracing::warn!("读取账号中心资料失败: {e}");
            None
        }
    };
    if let Some(me) = me {
        let nickname: String = me.username.trim().chars().take(24).collect();
        if !nickname.is_empty() {
            let _ = sqlx::query("UPDATE users SET nickname = $2, avatar_url = $3 WHERE uid = $1")
                .bind(uid)
                .bind(nickname)
                .bind(me.avatar_url.filter(|u| u.starts_with("https://")))
                .execute(pool)
                .await;
        }
    }
}

/// 记一次访问（按自然日计访问天数），并检查 TL0 → TL1 自动升级。
pub async fn record_visit(pool: &PgPool, rules: &TrustRules, uid: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET
            visit_days = visit_days + CASE WHEN last_visit_date IS DISTINCT FROM CURRENT_DATE THEN 1 ELSE 0 END,
            last_visit_date = CURRENT_DATE,
            last_seen_at = now()
         WHERE uid = $1",
    )
    .bind(uid)
    .execute(pool)
    .await?;
    maybe_promote(pool, rules, uid).await
}

pub async fn maybe_promote(pool: &PgPool, rules: &TrustRules, uid: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET trust_level = 1
         WHERE uid = $1 AND trust_level = 0 AND status = 'active'
           AND created_at <= now() - make_interval(days => $2::int)
           AND read_count >= $3",
    )
    .bind(uid)
    .bind(rules.tl1_days as i32)
    .bind(rules.tl1_reads as i32)
    .execute(pool)
    .await?;
    Ok(())
}

/// 禁言中的用户不能发帖、回复。
pub fn ensure_can_write(user: &ForumUser) -> Result<(), AppError> {
    if user.status == "muted" && user.muted_until.is_none_or(|t| t > Utc::now()) {
        return Err(AppError::Forbidden(match user.muted_until {
            Some(t) => format!("你已被禁言，{} 解除", t.with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap()).format("%m-%d %H:%M")),
            None => "你已被禁言".into(),
        }));
    }
    Ok(())
}

/// 按等级的每日上限检查；limits 下标为信任等级。
pub async fn ensure_daily_quota(
    pool: &PgPool,
    user: &ForumUser,
    table: &str,
    limits: &[i64; 4],
    what: &str,
) -> Result<(), AppError> {
    let limit = limits[user.trust_level.clamp(0, 3) as usize];
    if limit <= 0 {
        return Err(AppError::Forbidden(format!(
            "新人暂时不能{what}，注册满几天、多看看帖子后会自动升级"
        )));
    }
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE author_uid = $1 AND created_at > now() - interval '1 day'");
    let (count,): (i64,) = sqlx::query_as(&sql).bind(user.uid).fetch_one(pool).await?;
    if count >= limit {
        return Err(AppError::TooManyRequests { message: format!("今天{what}次数已达上限（{limit}）"), retry_after: 3600 });
    }
    Ok(())
}

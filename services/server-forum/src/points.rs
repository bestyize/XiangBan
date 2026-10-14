//! 论坛积分账本。所有余额变更先按 UUID 顺序锁用户，再与业务记录一起提交。
use crate::{auth::AuthUser, error::{AppError, AppResult}, models::{ListFilter, Page}, state::AppState};
use axum::{extract::{Query, State}, Json};
use chrono::{Datelike, FixedOffset, NaiveDate, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub const MAX_ATTACHMENT_PRICE: i32 = 10;
pub const REGISTER_REWARD: i32 = 50;
pub const INVITE_REWARD: i32 = 30;
pub const INVITE_DAILY_LIMIT: i64 = 3;

pub fn today() -> NaiveDate {
    Utc::now().with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap()).date_naive()
}

pub async fn lock_users(tx: &mut Transaction<'_, Postgres>, ids: &[Uuid]) -> AppResult<()> {
    let mut ids = ids.to_vec();
    ids.sort();
    ids.dedup();
    let rows: Vec<(Uuid,)> = sqlx::query_as("SELECT uid FROM users WHERE uid = ANY($1) ORDER BY uid FOR UPDATE")
        .bind(&ids).fetch_all(&mut **tx).await?;
    if rows.len() != ids.len() { return Err(AppError::not_found("用户")); }
    Ok(())
}

/// 返回本次实际变化量；相同事件重复请求返回 0。调用方必须已锁住用户行。
pub async fn change(
    tx: &mut Transaction<'_, Postgres>, uid: Uuid, delta: i32, kind: &str,
    key: &str, other: Option<Uuid>, reference: &str, description: &str,
) -> AppResult<i32> {
    if delta == 0 { return Ok(0); }
    let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM points_ledger WHERE uid = $1 AND event_key = $2)")
        .bind(uid).bind(key).fetch_one(&mut **tx).await?;
    if exists { return Ok(0); }
    let balance: Option<(i32,)> = sqlx::query_as(
        "UPDATE users SET points = (points::bigint + $2::bigint)::int WHERE uid = $1
         AND points::bigint + $2::bigint BETWEEN 0 AND 2147483647 RETURNING points",
    ).bind(uid).bind(delta as i64).fetch_optional(&mut **tx).await?;
    let (balance,) = balance.ok_or_else(|| AppError::BadRequest(
        if delta < 0 { "积分不足，可以通过每日访问、签到和贡献内容获取积分".into() } else { "积分余额已达到上限".into() }
    ))?;
    // 日奖励的账本日期与唯一事件日期一致，避免事务恰好跨过午夜时日期错位。
    let reward_day = if matches!(kind, "visit" | "check_in") {
        key.split_once(':').and_then(|(_, d)| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()).unwrap_or_else(today)
    } else { today() };
    sqlx::query(
        "INSERT INTO points_ledger (uid, delta, balance_after, kind, event_key, counterparty, reference, description, reward_day)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    ).bind(uid).bind(delta).bind(balance).bind(kind).bind(key).bind(other)
        .bind(reference).bind(description).bind(reward_day).execute(&mut **tx).await?;
    Ok(delta)
}

pub async fn on_visit(pool: &PgPool, uid: Uuid, invite: Option<String>) -> AppResult<()> {
    let inviter = match invite.and_then(|s| Uuid::parse_str(&s).ok()).filter(|i| *i != uid) {
        Some(i) => sqlx::query_as::<_, (Uuid,)>("SELECT uid FROM users WHERE uid = $1 AND status <> 'banned'")
            .bind(i).fetch_optional(pool).await?.map(|r| r.0),
        None => None,
    };
    let mut tx = pool.begin().await?;
    lock_users(&mut tx, &[uid]).await?;
    let (initialized,): (bool,) = sqlx::query_as("SELECT points_initialized FROM users WHERE uid = $1")
        .bind(uid).fetch_one(&mut *tx).await?;
    if !initialized {
        change(&mut tx, uid, REGISTER_REWARD, "registration", "registration", None, "", "首次加入论坛").await?;
        sqlx::query("INSERT INTO forum_registrations (uid, inviter_uid) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(uid).bind(inviter).execute(&mut *tx).await?;
        sqlx::query("UPDATE users SET points_initialized = true WHERE uid = $1").bind(uid).execute(&mut *tx).await?;
    }
    change(&mut tx, uid, 1, "visit", &format!("visit:{}", today()), None, "", "每日登录访问").await?;
    tx.commit().await?;
    Ok(())
}

pub async fn check_in(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Value>> {
    let uid = auth.user.uid;
    // 首次进站的客户端也可调用；通常已由 /session 完成。
    on_visit(&state.pool, uid, None).await?;
    let invited_by: Option<(Option<Uuid>,)> = sqlx::query_as("SELECT inviter_uid FROM forum_registrations WHERE uid = $1")
        .bind(uid).fetch_optional(&state.pool).await?;
    let inviter = invited_by.and_then(|r| r.0);
    let mut tx = state.pool.begin().await?;
    let mut owners = vec![uid];
    if let Some(i) = inviter { owners.push(i); }
    lock_users(&mut tx, &owners).await?;
    let day = today();
    let mut earned = change(&mut tx, uid, 1, "check_in", &format!("check_in:{day}"), None, "", "每日签到").await?;
    let first = NaiveDate::from_ymd_opt(day.year(), day.month(), 1).unwrap();
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM points_ledger WHERE uid = $1 AND kind = 'check_in' AND reward_day >= $2 AND reward_day <= $3",
    ).bind(uid).bind(first).bind(day).fetch_one(&mut *tx).await?;
    for (need, bonus) in [(7, 3), (20, 5)] {
        if count >= need {
            earned += change(&mut tx, uid, bonus, "check_in_bonus",
                &format!("check_in_bonus:{}:{need}", day.format("%Y-%m")), None, "",
                &format!("本月累计签到 {need} 天")).await?;
        }
    }
    if let Some(inviter) = inviter {
        let (status,): (String,) = sqlx::query_as("SELECT invite_status FROM forum_registrations WHERE uid = $1 FOR UPDATE")
            .bind(uid).fetch_one(&mut *tx).await?;
        if status == "pending" {
            let (allowed,): (bool,) = sqlx::query_as("SELECT status <> 'banned' FROM users WHERE uid = $1")
                .bind(inviter).fetch_one(&mut *tx).await?;
            let (used,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM points_ledger WHERE uid = $1 AND kind = 'invitation' AND reward_day = $2",
            ).bind(inviter).bind(day).fetch_one(&mut *tx).await?;
            let outcome = if !allowed { "ineligible" } else if used >= INVITE_DAILY_LIMIT { "capped" } else {
                change(&mut tx, inviter, INVITE_REWARD, "invitation", &format!("invitation:{uid}"),
                    Some(uid), &uid.to_string(), "邀请的新用户完成首次签到").await?;
                "rewarded"
            };
            sqlx::query("UPDATE forum_registrations SET invite_status = $2 WHERE uid = $1")
                .bind(uid).bind(outcome).execute(&mut *tx).await?;
        }
    }
    let (balance,): (i32,) = sqlx::query_as("SELECT points FROM users WHERE uid = $1").bind(uid).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({ "earned": earned, "balance": balance, "month_check_ins": count, "checked_in": true })))
}

pub async fn wallet(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Value>> {
    let uid = auth.user.uid;
    let day = today();
    let first = NaiveDate::from_ymd_opt(day.year(), day.month(), 1).unwrap();
    let (balance, visited, checked_in, month_check_ins): (i32, bool, bool, i64) = sqlx::query_as(
        "SELECT points,
            EXISTS(SELECT 1 FROM points_ledger WHERE uid = $1 AND kind = 'visit' AND reward_day = $2),
            EXISTS(SELECT 1 FROM points_ledger WHERE uid = $1 AND kind = 'check_in' AND reward_day = $2),
            (SELECT COUNT(*) FROM points_ledger WHERE uid = $1 AND kind = 'check_in' AND reward_day BETWEEN $3 AND $2)
         FROM users WHERE uid = $1",
    ).bind(uid).bind(day).bind(first).fetch_one(&state.pool).await?;
    let (invites, rewarded, income): (i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(*) FILTER (WHERE invite_status = 'rewarded'),
                COUNT(*) FILTER (WHERE invite_status = 'rewarded') * 30
         FROM forum_registrations WHERE inviter_uid = $1",
    ).bind(uid).fetch_one(&state.pool).await?;
    Ok(Json(json!({ "balance": balance, "visited_today": visited, "checked_in_today": checked_in,
        "month_check_ins": month_check_ins, "invite_code": uid.simple().to_string(), "invites": invites,
        "rewarded_invites": rewarded, "invite_income": income, "day": day,
        "rules": { "registration": REGISTER_REWARD, "visit": 1, "check_in": 1, "invitation": INVITE_REWARD,
            "invite_daily_limit": INVITE_DAILY_LIMIT, "max_attachment_price": MAX_ATTACHMENT_PRICE } })))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct LedgerItem {
    pub id: i64, pub delta: i32, pub balance_after: i32, pub kind: String,
    pub reference: String, pub description: String, pub created_at: chrono::DateTime<Utc>,
}

pub async fn ledger(State(state): State<AppState>, auth: AuthUser, Query(q): Query<ListFilter>) -> AppResult<Json<Page<LedgerItem>>> {
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM points_ledger WHERE uid = $1")
        .bind(auth.user.uid).fetch_one(&state.pool).await?;
    let items = sqlx::query_as(
        "SELECT id, delta, balance_after, kind, reference, description, created_at FROM points_ledger
         WHERE uid = $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
    ).bind(auth.user.uid).bind(q.page_size()).bind((q.page() - 1) * q.page_size()).fetch_all(&state.pool).await?;
    Ok(Json(Page { items, page: q.page(), page_size: q.page_size(), total }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct InviteItem {
    pub nickname: String, pub invite_status: String, pub created_at: chrono::DateTime<Utc>,
}

pub async fn invites(State(state): State<AppState>, auth: AuthUser, Query(q): Query<ListFilter>) -> AppResult<Json<Page<InviteItem>>> {
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM forum_registrations WHERE inviter_uid = $1")
        .bind(auth.user.uid).fetch_one(&state.pool).await?;
    let items = sqlx::query_as(
        "SELECT u.nickname, r.invite_status, r.created_at FROM forum_registrations r JOIN users u ON u.uid = r.uid
         WHERE r.inviter_uid = $1 ORDER BY r.created_at DESC, r.uid LIMIT $2 OFFSET $3",
    ).bind(auth.user.uid).bind(q.page_size()).bind((q.page() - 1) * q.page_size()).fetch_all(&state.pool).await?;
    Ok(Json(Page { items, page: q.page(), page_size: q.page_size(), total }))
}

#[cfg(test)]
mod tests;

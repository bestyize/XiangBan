//! 额度桶的读写。所有改动额度的路径（试用、兑换、管理员调整、对账）都经过这里，保证每次变化都留下流水并推进 `change_seq`。

use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgConnection, PgPool, Postgres, Transaction};

/// 可用的桶：没关闭、有余量、没过期。
pub const LIVE: &str = "closed_at IS NULL AND remaining > 0 AND (expires_at IS NULL OR expires_at > now())";

pub async fn live_total(conn: &mut PgConnection, binding_id: i64) -> Result<i64, AppError> {
    let (total,): (i64,) =
        sqlx::query_as(&format!("SELECT COALESCE(SUM(remaining), 0)::BIGINT FROM quota_buckets WHERE binding_id = $1 AND {LIVE}"))
            .bind(binding_id)
            .fetch_one(conn)
            .await?;
    Ok(total)
}

/// 付费额度（月卡、加油包、管理员补发）；试用额度不算。
pub async fn paid_total(pool: &PgPool, binding_id: i64) -> Result<i64, AppError> {
    let (total,): (i64,) = sqlx::query_as(&format!(
        "SELECT COALESCE(SUM(remaining), 0)::BIGINT FROM quota_buckets WHERE binding_id = $1 AND kind <> 'trial' AND {LIVE}"
    ))
    .bind(binding_id)
    .fetch_one(pool)
    .await?;
    Ok(total)
}

pub async fn add_ledger(
    conn: &mut PgConnection,
    binding_id: i64,
    bucket_id: Option<i64>,
    kind: &str,
    delta: i64,
    note: &str,
) -> Result<(), AppError> {
    let balance = live_total(conn, binding_id).await?;
    sqlx::query("INSERT INTO quota_ledger (binding_id, bucket_id, kind, delta, balance_after, note) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(binding_id)
        .bind(bucket_id)
        .bind(kind)
        .bind(delta)
        .bind(balance)
        .bind(note)
        .execute(conn)
        .await?;
    Ok(())
}

/// 新增一个额度桶并记流水。试用桶每个绑定只能有一个：已存在时返回 `None`，什么都不改。
pub async fn grant(
    tx: &mut Transaction<'_, Postgres>,
    binding_id: i64,
    kind: &str,
    quota: i64,
    expires_at: Option<DateTime<Utc>>,
    source: &str,
    note: &str,
) -> Result<Option<i64>, AppError> {
    let row: Option<(i64,)> = sqlx::query_as(
        "INSERT INTO quota_buckets (binding_id, kind, total, remaining, expires_at, source)
         VALUES ($1, $2, $3, $3, $4, $5) ON CONFLICT DO NOTHING RETURNING id",
    )
    .bind(binding_id)
    .bind(kind)
    .bind(quota)
    .bind(expires_at)
    .bind(source)
    .fetch_optional(&mut **tx)
    .await?;
    let Some((bucket_id,)) = row else { return Ok(None) };
    add_ledger(tx, binding_id, Some(bucket_id), "grant", quota, note).await?;
    sqlx::query("UPDATE bindings SET change_seq = change_seq + 1 WHERE id = $1").bind(binding_id).execute(&mut **tx).await?;
    Ok(Some(bucket_id))
}

/// 作废一个还没关闭的桶（管理员扣回），把剩余额度记为负向流水。返回所属绑定和被扣回的额度。
pub async fn void_bucket(
    tx: &mut Transaction<'_, Postgres>,
    bucket_id: i64,
    reason: &str,
    note: &str,
) -> Result<Option<(i64, i64)>, AppError> {
    let row: Option<(i64, i64)> = sqlx::query_as(
        "WITH old AS (SELECT id, binding_id, remaining FROM quota_buckets WHERE id = $1 AND closed_at IS NULL FOR UPDATE)
         UPDATE quota_buckets b SET remaining = 0, closed_at = now(), closed_reason = $2
         FROM old WHERE b.id = old.id RETURNING old.binding_id, old.remaining",
    )
    .bind(bucket_id)
    .bind(reason)
    .fetch_optional(&mut **tx)
    .await?;
    let Some((binding_id, remaining)) = row else { return Ok(None) };
    add_ledger(tx, binding_id, Some(bucket_id), "void", -remaining, note).await?;
    sqlx::query("UPDATE bindings SET change_seq = change_seq + 1 WHERE id = $1").bind(binding_id).execute(&mut **tx).await?;
    Ok(Some((binding_id, remaining)))
}

#[derive(Debug, Serialize, FromRow)]
pub struct BucketRow {
    pub id: i64,
    pub kind: String,
    pub total: i64,
    pub remaining: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_reason: Option<String>,
}

pub async fn buckets(pool: &PgPool, binding_id: i64, only_live: bool) -> Result<Vec<BucketRow>, AppError> {
    let filter = if only_live { format!("AND {LIVE}") } else { String::new() };
    Ok(sqlx::query_as(&format!(
        "SELECT id, kind, total, remaining, expires_at, source, created_at, closed_at, closed_reason
         FROM quota_buckets WHERE binding_id = $1 {filter} ORDER BY created_at DESC, id DESC LIMIT 100"
    ))
    .bind(binding_id)
    .fetch_all(pool)
    .await?)
}

#[derive(Debug, Serialize, FromRow)]
pub struct LedgerRow {
    pub id: i64,
    pub bucket_id: Option<i64>,
    pub kind: String,
    pub delta: i64,
    pub balance_after: i64,
    pub note: String,
    pub created_at: DateTime<Utc>,
}

pub async fn ledger(pool: &PgPool, binding_id: i64, before: Option<i64>, limit: i64) -> Result<Vec<LedgerRow>, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, bucket_id, kind, delta, balance_after, note, created_at FROM quota_ledger
         WHERE binding_id = $1 AND ($2::BIGINT IS NULL OR id < $2) ORDER BY id DESC LIMIT $3",
    )
    .bind(binding_id)
    .bind(before)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?)
}

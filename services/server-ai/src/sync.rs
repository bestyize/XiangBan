//! 对账：把 New API 里用户钱包的变化折算进额度桶，再把桶的合计推回 New API。
//!
//! 约定：`bindings.synced_quota` 是上一次对账结束时我们认为 New API 钱包里的额度。
//! 这一轮读到的实际值比它小，差额就是这段时间的消耗；比它大，说明有人在 New API 里手工加过额度，记为异常。
//! 推送一律用增减而不是覆盖：覆盖会抹掉读取与写入之间发生的消耗。

use crate::{
    error::AppError,
    ledger::{allocate, BucketView},
    newapi::QuotaMode,
    quota::{self, LIVE},
    state::AppState,
};
use chrono::{DateTime, Utc};
use sqlx::{PgConnection, Postgres, Transaction};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    Skipped,
    Synced { consumed: i64, pushed: i64 },
}

pub async fn sync_binding(state: &AppState, binding_id: i64) -> Result<SyncOutcome, AppError> {
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)").bind(binding_id).execute(&mut *tx).await?;

    let row: Option<(i64, i64, i64)> = sqlx::query_as(
        "SELECT newapi_user_id, synced_quota, change_seq FROM bindings
         WHERE id = $1 AND status = 'active' AND newapi_user_id IS NOT NULL",
    )
    .bind(binding_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((user_id, synced, seq)) = row else { return Ok(SyncOutcome::Skipped) };

    let actual = state.newapi.get_user(user_id).await?.quota;
    let consumed = synced - actual;

    if consumed > 0 {
        settle_consumption(&mut tx, binding_id, consumed).await?;
    } else if consumed < 0 {
        let detail = format!("New API 钱包比预期多 {}：可能有人在 New API 后台手工加了额度，已按实际值重新对齐", -consumed);
        sqlx::query("INSERT INTO sync_anomalies (binding_id, kind, detail) VALUES ($1, 'external_credit', $2)")
            .bind(binding_id)
            .bind(detail)
            .execute(&mut *tx)
            .await?;
        tracing::warn!("binding {binding_id}: New API 钱包比预期多 {}", -consumed);
    }

    close_empty_and_expired(&mut tx, binding_id).await?;

    let target = quota::live_total(&mut tx, binding_id).await?;
    let delta = target - actual;
    let mut new_actual = actual;
    if delta > 0 {
        state.newapi.adjust_quota(user_id, QuotaMode::Add, delta).await?;
        new_actual += delta;
    } else if delta < 0 {
        // 钱包不会减到负数；多出来的部分是别处已经透支掉的，没法再扣。
        let amount = (-delta).min(actual.max(0));
        state.newapi.adjust_quota(user_id, QuotaMode::Subtract, amount).await?;
        new_actual -= amount;
    }

    sqlx::query("UPDATE bindings SET synced_quota = $2, synced_seq = $3, last_synced_at = now() WHERE id = $1")
        .bind(binding_id)
        .bind(new_actual)
        .bind(seq)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(SyncOutcome::Synced { consumed: consumed.max(0), pushed: new_actual - actual })
}

async fn settle_consumption(tx: &mut Transaction<'_, Postgres>, binding_id: i64, consumed: i64) -> Result<(), AppError> {
    let live: Vec<(i64, i64, Option<DateTime<Utc>>)> = sqlx::query_as(&format!(
        "SELECT id, remaining, expires_at FROM quota_buckets WHERE binding_id = $1 AND {LIVE} FOR UPDATE"
    ))
    .bind(binding_id)
    .fetch_all(&mut **tx)
    .await?;
    let views: Vec<BucketView> =
        live.into_iter().map(|(id, remaining, expires_at)| BucketView { id, remaining, expires_at }).collect();

    let (takes, overdraft) = allocate(consumed, &views);
    for (bucket_id, take) in takes {
        sqlx::query("UPDATE quota_buckets SET remaining = remaining - $2 WHERE id = $1")
            .bind(bucket_id)
            .bind(take)
            .execute(&mut **tx)
            .await?;
        quota::add_ledger(tx, binding_id, Some(bucket_id), "consume", -take, "").await?;
    }
    if overdraft > 0 {
        quota::add_ledger(tx, binding_id, None, "consume_overdraft", -overdraft, "额度已用完后产生的透支").await?;
    }
    Ok(())
}

async fn close_empty_and_expired(tx: &mut Transaction<'_, Postgres>, binding_id: i64) -> Result<(), AppError> {
    sqlx::query("UPDATE quota_buckets SET closed_at = now(), closed_reason = 'spent' WHERE binding_id = $1 AND closed_at IS NULL AND remaining <= 0")
        .bind(binding_id)
        .execute(&mut **tx)
        .await?;

    let expired: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT id, remaining FROM quota_buckets
         WHERE binding_id = $1 AND closed_at IS NULL AND expires_at IS NOT NULL AND expires_at <= now() FOR UPDATE",
    )
    .bind(binding_id)
    .fetch_all(&mut **tx)
    .await?;
    for (bucket_id, remaining) in expired {
        sqlx::query("UPDATE quota_buckets SET remaining = 0, closed_at = now(), closed_reason = 'expired' WHERE id = $1")
            .bind(bucket_id)
            .execute(&mut **tx)
            .await?;
        quota::add_ledger(tx, binding_id, Some(bucket_id), "expire", -remaining, "").await?;
    }
    Ok(())
}

/// 需要对账的绑定：有未推送的变化、最近在用、很久没对账，或者有桶到期了。
pub async fn due_bindings(conn: &mut PgConnection, limit: i64) -> Result<Vec<i64>, AppError> {
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM bindings b WHERE status = 'active' AND newapi_user_id IS NOT NULL AND (
             change_seq <> synced_seq
             OR last_used_at > now() - interval '15 minutes'
             OR last_synced_at IS NULL
             OR last_synced_at < now() - interval '30 minutes'
             OR EXISTS (SELECT 1 FROM quota_buckets q WHERE q.binding_id = b.id AND q.closed_at IS NULL
                        AND q.expires_at IS NOT NULL AND q.expires_at <= now())
         ) ORDER BY last_synced_at NULLS FIRST LIMIT $1",
    )
    .bind(limit)
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn run_loop(state: AppState) {
    let mut ticker = tokio::time::interval(Duration::from_secs(state.config.sync_interval_secs));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        ticker.tick().await;
        if !state.newapi.is_configured() {
            continue;
        }
        let ids = match state.pool.acquire().await {
            Ok(mut conn) => due_bindings(&mut conn, 200).await,
            Err(e) => Err(e.into()),
        };
        match ids {
            Ok(ids) => {
                for id in ids {
                    if let Err(e) = sync_binding(&state, id).await {
                        tracing::warn!("binding {id} 对账失败: {e}");
                    }
                }
            }
            Err(e) => tracing::warn!("查询待对账绑定失败: {e}"),
        }
    }
}

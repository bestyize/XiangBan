use super::Store;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheTotals {
    pub entries: u64,
    pub bytes: u64,
    pub oldest: Option<DateTime<Utc>>,
}

impl Store {
    /// Returns the cached payload if it is younger than `ttl_secs`, and in the same statement marks
    /// it as just used. That update is what moves an entry away from the eviction end of the LRU order.
    pub async fn cache_hit(&self, key: &str, ttl_secs: u64) -> Result<Option<String>, AppError> {
        let row = sqlx::query(
            "UPDATE search_cache
             SET last_access_at = clock_timestamp(), hit_count = hit_count + 1
             WHERE cache_key = $1 AND created_at > clock_timestamp() - make_interval(secs => $2)
             RETURNING payload",
        )
        .bind(key)
        .bind(ttl_secs as f64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.get("payload")))
    }

    pub async fn cache_put(&self, key: &str, payload: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO search_cache (cache_key, payload, size_bytes, hit_count, created_at, last_access_at)
             VALUES ($1, $2, $3, 0, clock_timestamp(), clock_timestamp())
             ON CONFLICT (cache_key) DO UPDATE SET
                payload = EXCLUDED.payload,
                size_bytes = EXCLUDED.size_bytes,
                hit_count = 0,
                created_at = EXCLUDED.created_at,
                last_access_at = EXCLUDED.last_access_at",
        )
        .bind(key)
        .bind(payload)
        .bind(payload.len() as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn cache_totals(&self) -> Result<CacheTotals, AppError> {
        let row = sqlx::query(
            "SELECT COUNT(*)::BIGINT AS entries, COALESCE(SUM(size_bytes), 0)::BIGINT AS bytes, MIN(created_at) AS oldest
             FROM search_cache",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(CacheTotals {
            entries: row.get::<i64, _>("entries") as u64,
            bytes: row.get::<i64, _>("bytes") as u64,
            oldest: row.get("oldest"),
        })
    }

    /// Removes entries older than `ttl_secs`.
    pub async fn cache_delete_expired(&self, ttl_secs: u64) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM search_cache WHERE created_at <= clock_timestamp() - make_interval(secs => $1)")
            .bind(ttl_secs as f64)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Removes the `count` least recently used entries.
    pub async fn cache_delete_lru(&self, count: u64) -> Result<u64, AppError> {
        let result = sqlx::query(
            "DELETE FROM search_cache WHERE cache_key IN (
                SELECT cache_key FROM search_cache ORDER BY last_access_at ASC LIMIT $1
             )",
        )
        .bind(count as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn cache_clear(&self) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM search_cache").execute(&self.pool).await?;
        Ok(result.rows_affected())
    }
}

use super::Store;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct KeyRow {
    pub id: i64,
    pub provider: String,
    pub label: String,
    pub api_key: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ProxyRow {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl Store {
    pub async fn list_keys(&self) -> Result<Vec<KeyRow>, AppError> {
        Ok(sqlx::query_as::<_, KeyRow>(
            "SELECT id, provider, label, api_key, enabled, created_at FROM provider_keys ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn insert_key(&self, provider: &str, label: &str, api_key: &str) -> Result<i64, AppError> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO provider_keys (provider, label, api_key) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(provider)
        .bind(label)
        .bind(api_key)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn update_key(
        &self,
        provider: &str,
        id: i64,
        label: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            "UPDATE provider_keys SET label = COALESCE($3::text, label), enabled = COALESCE($4::boolean, enabled)
             WHERE provider = $1 AND id = $2",
        )
        .bind(provider)
        .bind(id)
        .bind(label)
        .bind(enabled)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_key(&self, provider: &str, id: i64) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM provider_keys WHERE provider = $1 AND id = $2")
            .bind(provider)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_keys(&self, provider: &str) -> Result<i64, AppError> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM provider_keys WHERE provider = $1")
            .bind(provider)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn list_proxies(&self) -> Result<Vec<ProxyRow>, AppError> {
        Ok(sqlx::query_as::<_, ProxyRow>("SELECT id, name, url, enabled, created_at FROM proxies ORDER BY id")
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn get_proxy(&self, id: i64) -> Result<Option<ProxyRow>, AppError> {
        Ok(
            sqlx::query_as::<_, ProxyRow>("SELECT id, name, url, enabled, created_at FROM proxies WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn insert_proxy(&self, name: &str, url: &str) -> Result<i64, AppError> {
        let id: i64 = sqlx::query_scalar("INSERT INTO proxies (name, url) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(url)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn update_proxy(&self, id: i64, name: Option<&str>, enabled: Option<bool>) -> Result<bool, AppError> {
        let result = sqlx::query(
            "UPDATE proxies SET name = COALESCE($2::text, name), enabled = COALESCE($3::boolean, enabled) WHERE id = $1",
        )
        .bind(id)
        .bind(name)
        .bind(enabled)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_proxy(&self, id: i64) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM proxies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

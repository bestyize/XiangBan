use super::Store;
use crate::error::AppError;
use sqlx::Row;
use std::collections::HashMap;

impl Store {
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        Ok(sqlx::query_scalar("SELECT value FROM app_settings WHERE key = $1").bind(key).fetch_optional(&self.pool).await?)
    }
    pub async fn all_settings(&self) -> Result<HashMap<String, String>, AppError> {
        let rows = sqlx::query("SELECT key, value FROM app_settings").fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("key"), r.get::<String, _>("value")))
            .collect())
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO app_settings (key, value) VALUES ($1, $2)
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Stores a value only if the key is not set yet. Returns whether this call stored it.
    pub async fn set_setting_if_absent(&self, key: &str, value: &str) -> Result<bool, AppError> {
        let result = sqlx::query("INSERT INTO app_settings (key, value) VALUES ($1, $2) ON CONFLICT (key) DO NOTHING")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

use super::Store;
use crate::error::AppError;
use sqlx::Row;
use std::collections::HashMap;

impl Store {
    pub async fn update_desktop_config<F>(
        &self,
        change: F,
    ) -> Result<crate::desktop::DesktopConfig, AppError>
    where
        F: FnOnce(&mut crate::desktop::DesktopConfig) -> Result<(), AppError>,
    {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("INSERT INTO app_settings (key,value) VALUES ('desktop.config',$1) ON CONFLICT (key) DO NOTHING")
            .bind(serde_json::to_string(&crate::desktop::DesktopConfig::default()).unwrap()).execute(&mut *transaction).await?;
        let value: String = sqlx::query_scalar(
            "SELECT value FROM app_settings WHERE key='desktop.config' FOR UPDATE",
        )
        .fetch_one(&mut *transaction)
        .await?;
        let mut config: crate::desktop::DesktopConfig =
            serde_json::from_str(&value).map_err(|e| AppError::Internal(e.to_string()))?;
        change(&mut config)?;
        config.validate()?;
        sqlx::query("UPDATE app_settings SET value=$1 WHERE key='desktop.config'")
            .bind(serde_json::to_string(&config).unwrap())
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(config)
    }
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        Ok(
            sqlx::query_scalar("SELECT value FROM app_settings WHERE key = $1")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?,
        )
    }
    pub async fn all_settings(&self) -> Result<HashMap<String, String>, AppError> {
        let rows = sqlx::query("SELECT key, value FROM app_settings")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("key"), r.get::<String, _>("value")))
            .collect())
    }

    /// Settings whose key matches a SQL `LIKE` pattern. Used for small, fixed key families so the
    /// whole table (which also holds one row per cached media file) is never loaded.
    pub async fn settings_matching(&self, pattern: &str) -> Result<HashMap<String, String>, AppError> {
        let rows = sqlx::query("SELECT key, value FROM app_settings WHERE key LIKE $1")
            .bind(pattern)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("key"), r.get::<String, _>("value")))
            .collect())
    }

    pub async fn delete_setting(&self, key: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM app_settings WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
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
        let result = sqlx::query(
            "INSERT INTO app_settings (key, value) VALUES ($1, $2) ON CONFLICT (key) DO NOTHING",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

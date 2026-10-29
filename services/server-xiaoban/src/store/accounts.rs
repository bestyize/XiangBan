use super::Store;
use crate::error::AppError;
use sqlx::Row;

#[derive(Debug, Clone)]
pub struct AdminAccount {
    pub username: String,
    pub password_hash: String,
    pub is_default_password: bool,
}

impl Store {
    pub async fn admin_account(&self) -> Result<Option<AdminAccount>, AppError> {
        let row = sqlx::query("SELECT username, password_hash, is_default_password FROM admin_account WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| AdminAccount {
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            is_default_password: r.get("is_default_password"),
        }))
    }

    pub async fn save_admin_account(&self, account: &AdminAccount) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO admin_account (id, username, password_hash, is_default_password, updated_at)
             VALUES (1, $1, $2, $3, NOW())
             ON CONFLICT (id) DO UPDATE SET
                username = EXCLUDED.username,
                password_hash = EXCLUDED.password_hash,
                is_default_password = EXCLUDED.is_default_password,
                updated_at = NOW()",
        )
        .bind(&account.username)
        .bind(&account.password_hash)
        .bind(account.is_default_password)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Creates the account only when none exists yet, so two instances starting together
    /// cannot overwrite each other. Returns whether this call created it.
    pub async fn create_admin_account_if_absent(&self, account: &AdminAccount) -> Result<bool, AppError> {
        let result = sqlx::query(
            "INSERT INTO admin_account (id, username, password_hash, is_default_password)
             VALUES (1, $1, $2, $3)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(&account.username)
        .bind(&account.password_hash)
        .bind(account.is_default_password)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

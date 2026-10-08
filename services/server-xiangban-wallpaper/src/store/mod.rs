//! PostgreSQL tables behind the admin console: the admin account, provider API keys, the proxy
//! pool, runtime settings and the persistent search cache. It shares the connection pool that
//! `Database` opened, so there is only one set of connections to the server.

mod accounts;
mod cache;
mod credentials;
mod settings;

pub use accounts::AdminAccount;
pub use cache::CacheTotals;
pub use credentials::{KeyRow, ProxyRow};

use crate::error::AppError;
use sqlx::PgPool;

#[derive(Clone)]
pub struct Store {
    pool: PgPool,
}

impl Store {
    pub async fn init(pool: PgPool) -> Result<Self, AppError> {
        let sql = include_str!("../../migrations/20261009000000_admin.sql");
        sqlx::raw_sql(sql).execute(&pool).await?;
        Ok(Self { pool })
    }
}

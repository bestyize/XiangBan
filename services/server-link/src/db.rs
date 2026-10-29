use crate::error::AppError;
use sqlx::{postgres::{PgConnectOptions, PgPoolOptions}, PgPool};
use std::str::FromStr;

pub async fn connect(url: &str, schema: Option<&str>) -> Result<PgPool, AppError> {
    let mut options = PgConnectOptions::from_str(url).map_err(|e| AppError::Internal(format!("database url: {e}")))?;
    if let Some(schema) = schema {
        options = options.options([("search_path", schema)]);
    }
    Ok(PgPoolOptions::new().max_connections(10).connect_with(options).await?)
}

pub async fn migrate(pool: &PgPool) -> Result<(), AppError> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    Ok(())
}

/// 只存设备（谁的、什么类型、叫什么），不存任何消息内容。
/// 旧版本的 `pair_codes`、`pairings` 表（配对流程）已不再使用，留在库里不动。
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS devices (
    id           TEXT PRIMARY KEY,
    uid          UUID NOT NULL,
    kind         TEXT NOT NULL CHECK (kind IN ('desktop', 'phone')),
    name         TEXT NOT NULL,
    public_key   TEXT,
    install_id   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ,
    revoked_at   TIMESTAMPTZ
);
ALTER TABLE devices ADD COLUMN IF NOT EXISTS install_id TEXT;
ALTER TABLE devices ALTER COLUMN public_key DROP NOT NULL;
-- 旧版本（配对 + 公钥）登记的设备没有 install_id，新版客户端不会再用，直接作废。
UPDATE devices SET revoked_at = now() WHERE install_id IS NULL AND revoked_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS devices_uid_install_idx ON devices (uid, install_id) WHERE revoked_at IS NULL AND install_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS devices_uid_idx ON devices (uid) WHERE revoked_at IS NULL;
"#;

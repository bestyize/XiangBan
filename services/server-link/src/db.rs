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

/// 只存关系，不存任何消息内容。
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS devices (
    id           TEXT PRIMARY KEY,
    uid          UUID NOT NULL,
    kind         TEXT NOT NULL CHECK (kind IN ('desktop', 'phone')),
    name         TEXT NOT NULL,
    public_key   TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ,
    revoked_at   TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS devices_uid_key_idx ON devices (uid, public_key) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS devices_uid_idx ON devices (uid) WHERE revoked_at IS NULL;

-- 一次性配对码，只存哈希。redeems 是被兑换的次数，used_at 是桌面确认的时间。
CREATE TABLE IF NOT EXISTS pair_codes (
    code_hash  TEXT PRIMARY KEY,
    desktop_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    uid        UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    redeems    INT NOT NULL DEFAULT 0,
    used_at    TIMESTAMPTZ
);

-- status: pending 等桌面确认；active 已配对；rejected 被桌面拒绝；revoked 已解除。
CREATE TABLE IF NOT EXISTS pairings (
    id           UUID PRIMARY KEY,
    uid          UUID NOT NULL,
    desktop_id   TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    phone_id     TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    code_hash    TEXT,
    status       TEXT NOT NULL CHECK (status IN ('pending', 'active', 'rejected', 'revoked')),
    phone_proof  TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    activated_at TIMESTAMPTZ,
    revoked_at   TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS pairings_active_idx ON pairings (desktop_id, phone_id) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS pairings_desktop_idx ON pairings (desktop_id, status);
CREATE INDEX IF NOT EXISTS pairings_phone_idx ON pairings (phone_id, status);
"#;

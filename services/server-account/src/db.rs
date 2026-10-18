use crate::error::AppError;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::str::FromStr;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    uid           UUID PRIMARY KEY,
    username      TEXT NOT NULL,
    username_key  TEXT NOT NULL,
    avatar_url    TEXT,
    status        TEXT NOT NULL DEFAULT 'active',
    password_hash TEXT,
    failed_logins INT NOT NULL DEFAULT 0,
    locked_until  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);

ALTER TABLE users ADD COLUMN IF NOT EXISTS provision_request_id UUID;
CREATE UNIQUE INDEX IF NOT EXISTS users_provision_request_idx ON users (provision_request_id) WHERE provision_request_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS identities (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    kind            TEXT NOT NULL,
    identifier_hash BYTEA NOT NULL,
    identifier_enc  BYTEA NOT NULL,
    verified_at     TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (kind, identifier_hash)
);
CREATE INDEX IF NOT EXISTS identities_uid_idx ON identities (uid);

CREATE TABLE IF NOT EXISTS verification_codes (
    id              BIGSERIAL PRIMARY KEY,
    kind            TEXT NOT NULL,
    identifier_hash BYTEA NOT NULL,
    purpose         TEXT NOT NULL,
    code_hash       BYTEA NOT NULL,
    attempts        INT NOT NULL DEFAULT 0,
    ip              TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    consumed_at     TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS verification_codes_ident_idx ON verification_codes (kind, identifier_hash, created_at DESC);
CREATE INDEX IF NOT EXISTS verification_codes_ip_idx ON verification_codes (ip, created_at DESC);

CREATE TABLE IF NOT EXISTS clients (
    client_id  TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    enabled    BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id          UUID PRIMARY KEY,
    uid         UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    client_id   TEXT NOT NULL REFERENCES clients(client_id),
    family_id   UUID NOT NULL,
    token_hash  BYTEA NOT NULL UNIQUE,
    device_name TEXT,
    ip          TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked_at  TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS refresh_tokens_uid_idx ON refresh_tokens (uid);
CREATE INDEX IF NOT EXISTS refresh_tokens_family_idx ON refresh_tokens (family_id);

CREATE TABLE IF NOT EXISTS admins (
    id            UUID PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    token_version INT NOT NULL DEFAULT 0,
    failed_logins INT NOT NULL DEFAULT 0,
    locked_until  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS admin_audit (
    id             BIGSERIAL PRIMARY KEY,
    admin_username TEXT NOT NULL,
    action         TEXT NOT NULL,
    target         TEXT,
    detail         TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS system_settings (
    key        TEXT PRIMARY KEY,
    value_enc  BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS users_created_idx ON users (created_at DESC);

-- 昵称改为不可修改的唯一用户名。旧库必须先清空 users（ADD COLUMN ... NOT NULL 在有数据时会失败，这是有意的）。
ALTER TABLE users ADD COLUMN IF NOT EXISTS username TEXT NOT NULL;
ALTER TABLE users ADD COLUMN IF NOT EXISTS username_key TEXT NOT NULL;
ALTER TABLE users DROP COLUMN IF EXISTS nickname;
CREATE UNIQUE INDEX IF NOT EXISTS users_username_key_idx ON users (username_key);

-- 账号 ID：建号时由序列分配的数字，从 13586 起，对外以字符串展示，之后任何接口都不会修改它。
-- 旧库已有用户时，ADD COLUMN 会按物理顺序给他们补号。
CREATE SEQUENCE IF NOT EXISTS user_account_id_seq START WITH 13586 MINVALUE 13586;
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_id BIGINT NOT NULL DEFAULT nextval('user_account_id_seq');
CREATE UNIQUE INDEX IF NOT EXISTS users_account_id_idx ON users (account_id);
"#;

/// `schema` 仅供测试隔离使用：连接级 search_path。
pub async fn connect(url: &str, schema: Option<&str>) -> Result<PgPool, AppError> {
    let mut options = PgConnectOptions::from_str(url)
        .map_err(|e| AppError::Internal(format!("invalid ACCOUNT_DATABASE_URL: {e}")))?;
    if let Some(schema) = schema {
        options = options.options([("search_path", schema)]);
    }
    Ok(PgPoolOptions::new().max_connections(10).connect_with(options).await?)
}

pub async fn migrate(pool: &PgPool) -> Result<(), AppError> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    Ok(())
}

pub async fn seed_clients(pool: &PgPool, clients: &[(String, String)]) -> Result<(), AppError> {
    for (id, name) in clients {
        sqlx::query("INSERT INTO clients (client_id, name) VALUES ($1, $2) ON CONFLICT (client_id) DO NOTHING")
            .bind(id)
            .bind(name)
            .execute(pool)
            .await?;
    }
    Ok(())
}


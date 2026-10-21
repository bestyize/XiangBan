use crate::error::AppError;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::str::FromStr;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS products (
    product_id  TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    client_ids  TEXT[] NOT NULL DEFAULT '{}',
    trial_quota BIGINT NOT NULL DEFAULT 0,
    trial_days  INT NOT NULL DEFAULT 7,
    enabled     BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS product_tiers (
    product_id    TEXT NOT NULL REFERENCES products(product_id) ON DELETE CASCADE,
    tier          TEXT NOT NULL,
    display_name  TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    model         TEXT NOT NULL,
    requires_paid BOOLEAN NOT NULL DEFAULT false,
    is_default    BOOLEAN NOT NULL DEFAULT false,
    sort_order    INT NOT NULL DEFAULT 0,
    enabled       BOOLEAN NOT NULL DEFAULT true,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (product_id, tier)
);
CREATE UNIQUE INDEX IF NOT EXISTS product_tiers_default_idx ON product_tiers (product_id) WHERE is_default;
ALTER TABLE product_tiers ADD COLUMN IF NOT EXISTS reasoning_efforts TEXT[] DEFAULT NULL;

-- 每个 (账号, 产品) 对应 New API 里一个独立用户和一个无限额度的令牌，额度只由用户钱包决定。
-- change_seq 在额度桶变化时自增，synced_seq 记录已推送到哪一版，二者不等说明还有变化没同步到 New API。
CREATE TABLE IF NOT EXISTS bindings (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL,
    product_id      TEXT NOT NULL REFERENCES products(product_id),
    status          TEXT NOT NULL DEFAULT 'pending',
    newapi_user_id  BIGINT,
    newapi_username TEXT,
    newapi_key_enc  BYTEA,
    synced_quota    BIGINT NOT NULL DEFAULT 0,
    change_seq      BIGINT NOT NULL DEFAULT 0,
    synced_seq      BIGINT NOT NULL DEFAULT 0,
    last_used_at    TIMESTAMPTZ,
    last_synced_at  TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (uid, product_id)
);
CREATE INDEX IF NOT EXISTS bindings_uid_idx ON bindings (uid);

CREATE TABLE IF NOT EXISTS quota_buckets (
    id            BIGSERIAL PRIMARY KEY,
    binding_id    BIGINT NOT NULL REFERENCES bindings(id) ON DELETE CASCADE,
    kind          TEXT NOT NULL,
    total         BIGINT NOT NULL,
    remaining     BIGINT NOT NULL,
    expires_at    TIMESTAMPTZ,
    source        TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at     TIMESTAMPTZ,
    closed_reason TEXT
);
CREATE INDEX IF NOT EXISTS quota_buckets_open_idx ON quota_buckets (binding_id) WHERE closed_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS quota_buckets_trial_idx ON quota_buckets (binding_id) WHERE kind = 'trial';

-- 只追加：每一次额度变化都留一行，delta 为带符号的变化量，balance_after 是该次变化后的可用总额。
CREATE TABLE IF NOT EXISTS quota_ledger (
    id            BIGSERIAL PRIMARY KEY,
    binding_id    BIGINT NOT NULL REFERENCES bindings(id) ON DELETE CASCADE,
    bucket_id     BIGINT,
    kind          TEXT NOT NULL,
    delta         BIGINT NOT NULL,
    balance_after BIGINT NOT NULL,
    note          TEXT NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS quota_ledger_binding_idx ON quota_ledger (binding_id, id DESC);
CREATE INDEX IF NOT EXISTS quota_ledger_created_idx ON quota_ledger (created_at DESC);

CREATE TABLE IF NOT EXISTS sync_anomalies (
    id          BIGSERIAL PRIMARY KEY,
    binding_id  BIGINT NOT NULL REFERENCES bindings(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    detail      TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS sync_anomalies_open_idx ON sync_anomalies (created_at DESC) WHERE resolved_at IS NULL;

-- 一批兑换码对应淘宝上的一个商品：同产品、同类型、同面额、同有效期。有效期从用户兑换那天算起。
CREATE TABLE IF NOT EXISTS redeem_batches (
    id         BIGSERIAL PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES products(product_id),
    kind       TEXT NOT NULL CHECK (kind IN ('topup', 'monthly')),
    quota      BIGINT NOT NULL CHECK (quota > 0),
    valid_days INT NOT NULL CHECK (valid_days > 0),
    note       TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 只存带密钥的哈希：数据库泄露也看不到可用的码。明文只在生成批次时返回一次。
CREATE TABLE IF NOT EXISTS redeem_codes (
    id          BIGSERIAL PRIMARY KEY,
    batch_id    BIGINT NOT NULL REFERENCES redeem_batches(id),
    code_hash   BYTEA NOT NULL UNIQUE,
    status      TEXT NOT NULL DEFAULT 'unused' CHECK (status IN ('unused', 'redeemed', 'void')),
    redeemed_by UUID,
    binding_id  BIGINT REFERENCES bindings(id),
    bucket_id   BIGINT REFERENCES quota_buckets(id),
    redeemed_at TIMESTAMPTZ,
    voided_at   TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS redeem_codes_batch_idx ON redeem_codes (batch_id, status);
CREATE INDEX IF NOT EXISTS redeem_codes_user_idx ON redeem_codes (redeemed_by) WHERE redeemed_by IS NOT NULL;

-- 后台里填写的运行时设置（如 New API 管理令牌），值已加密。
CREATE TABLE IF NOT EXISTS system_settings (
    key        TEXT PRIMARY KEY,
    value_enc  BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

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
"#;

/// `schema` 仅供测试隔离使用：连接级 search_path。
pub async fn connect(url: &str, schema: Option<&str>) -> Result<PgPool, AppError> {
    let mut options =
        PgConnectOptions::from_str(url).map_err(|e| AppError::Internal(format!("invalid AI_DATABASE_URL: {e}")))?;
    if let Some(schema) = schema {
        options = options.options([("search_path", schema)]);
    }
    Ok(PgPoolOptions::new().max_connections(10).connect_with(options).await?)
}

pub async fn migrate(pool: &PgPool) -> Result<(), AppError> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    Ok(())
}

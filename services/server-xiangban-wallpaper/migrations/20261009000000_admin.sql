-- Admin console: single administrator account
CREATE TABLE IF NOT EXISTS admin_account (
    id SMALLINT PRIMARY KEY CHECK (id = 1),
    username VARCHAR(64) NOT NULL,
    password_hash TEXT NOT NULL,
    is_default_password BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Provider API keys (several per provider; requests rotate across the enabled ones)
CREATE TABLE IF NOT EXISTS provider_keys (
    id BIGSERIAL PRIMARY KEY,
    provider VARCHAR(32) NOT NULL,
    label VARCHAR(64) NOT NULL,
    api_key TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_provider_keys_provider ON provider_keys(provider);

-- Outbound proxy pool
CREATE TABLE IF NOT EXISTS proxies (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    url TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Runtime settings (cache lifetime, proxy strategy, per-provider proxy switch, ...)
CREATE TABLE IF NOT EXISTS app_settings (
    key VARCHAR(128) PRIMARY KEY,
    value TEXT NOT NULL
);

-- Aggregated search results. Expiry is judged against the current TTL setting when an entry is
-- read, so changing the TTL in the console applies to entries that already exist.
CREATE TABLE IF NOT EXISTS search_cache (
    cache_key TEXT PRIMARY KEY,
    payload TEXT NOT NULL,
    size_bytes INT NOT NULL,
    hit_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_access_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_search_cache_lru ON search_cache(last_access_at);
CREATE INDEX IF NOT EXISTS idx_search_cache_created ON search_cache(created_at);

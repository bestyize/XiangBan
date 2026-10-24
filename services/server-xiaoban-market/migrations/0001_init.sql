-- 小伴市场：目录、版本、分类、采集工作台。
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE admin_users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 技能 / 专家 / 连接器各用一套分类字典。
CREATE TABLE categories (
    id         SERIAL PRIMARY KEY,
    kind       TEXT NOT NULL CHECK (kind IN ('skill', 'expert', 'connector')),
    slug       TEXT NOT NULL,
    name       TEXT NOT NULL,
    name_en    TEXT NOT NULL DEFAULT '',
    sort_order INT  NOT NULL DEFAULT 0,
    active     BOOLEAN NOT NULL DEFAULT true,
    UNIQUE (kind, slug)
);

-- 上游分类名到本地分类的记忆映射：入库时管理员选过一次，之后同名上游分类自动归入。
CREATE TABLE category_aliases (
    kind        TEXT NOT NULL,
    alias       TEXT NOT NULL,
    category_id INT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (kind, alias)
);

CREATE TABLE items (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind           TEXT NOT NULL CHECK (kind IN ('skill', 'expert', 'connector')),
    namespace      TEXT NOT NULL,
    slug           TEXT NOT NULL,
    name           TEXT NOT NULL,
    summary        TEXT NOT NULL DEFAULT '',
    description    TEXT NOT NULL DEFAULT '',
    category_id    INT REFERENCES categories(id) ON DELETE SET NULL,
    tags           TEXT[] NOT NULL DEFAULT '{}',
    icon_url       TEXT,
    source         TEXT NOT NULL DEFAULT 'community' CHECK (source IN ('official', 'community', 'enterprise')),
    origin         TEXT NOT NULL DEFAULT 'manual',
    origin_ref     TEXT,
    origin_url     TEXT,
    license        TEXT,
    status         TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'review', 'published', 'offline')),
    featured       BOOLEAN NOT NULL DEFAULT false,
    download_count BIGINT NOT NULL DEFAULT 0,
    favorite_count BIGINT NOT NULL DEFAULT 0,
    latest_version TEXT,
    metadata       JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- 详情页「试试这样用」的示例提问，字符串数组。
    prompts        JSONB NOT NULL DEFAULT '[]'::jsonb,
    search_text    TEXT NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at   TIMESTAMPTZ,
    UNIQUE (kind, namespace, slug)
);
CREATE UNIQUE INDEX items_origin_idx ON items (kind, origin, origin_ref) WHERE origin_ref IS NOT NULL;
CREATE INDEX items_listing_idx ON items (kind, status, category_id);
CREATE INDEX items_search_trgm_idx ON items USING gin (search_text gin_trgm_ops);

CREATE TABLE versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id         UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    version         TEXT NOT NULL,
    changelog       TEXT NOT NULL DEFAULT '',
    manifest        JSONB NOT NULL,
    manifest_sha256 TEXT NOT NULL,
    package_sha256  TEXT NOT NULL,
    package_size    BIGINT NOT NULL,
    yanked          BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (item_id, version)
);

-- 同一天同一来源对同一版本只计一次下载。
CREATE TABLE download_log (
    version_id UUID NOT NULL REFERENCES versions(id) ON DELETE CASCADE,
    day        DATE NOT NULL,
    client     TEXT NOT NULL,
    PRIMARY KEY (version_id, day, client)
);

-- 采集来源：一行对应一个上游板块（如 ModelScope 的 MCP 广场）。
CREATE TABLE sources (
    id               SERIAL PRIMARY KEY,
    key              TEXT NOT NULL UNIQUE,
    name             TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('skill', 'expert', 'connector')),
    adapter          TEXT NOT NULL CHECK (adapter IN ('skillhub', 'generic_json')),
    enabled          BOOLEAN NOT NULL DEFAULT true,
    config           JSONB NOT NULL DEFAULT '{}'::jsonb,
    interval_minutes INT NOT NULL DEFAULT 0,
    -- none：只进暂存区；draft / published：采集后自动入库为该状态。
    auto_import      TEXT NOT NULL DEFAULT 'none' CHECK (auto_import IN ('none', 'draft', 'published')),
    last_job_at      TIMESTAMPTZ
);

CREATE TABLE crawl_jobs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id   INT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    trigger     TEXT NOT NULL DEFAULT 'manual',
    params      JSONB NOT NULL DEFAULT '{}'::jsonb,
    stats       JSONB NOT NULL DEFAULT '{}'::jsonb,
    error       TEXT,
    created_by  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);
CREATE INDEX crawl_jobs_source_idx ON crawl_jobs (source_id, created_at DESC);
-- 每个来源同一时刻只允许一个未结束的任务。
CREATE UNIQUE INDEX crawl_jobs_active_idx ON crawl_jobs (source_id) WHERE status IN ('queued', 'running');

CREATE TABLE crawl_logs (
    id      BIGSERIAL PRIMARY KEY,
    job_id  UUID NOT NULL REFERENCES crawl_jobs(id) ON DELETE CASCADE,
    ts      TIMESTAMPTZ NOT NULL DEFAULT now(),
    level   TEXT NOT NULL DEFAULT 'info',
    message TEXT NOT NULL
);
CREATE INDEX crawl_logs_job_idx ON crawl_logs (job_id, id);

-- 采集暂存区：保留上游原文与规范化结果，由工作台人工审核后入库。
CREATE TABLE staged_items (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id     INT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    upstream_id   TEXT NOT NULL,
    kind          TEXT NOT NULL,
    name          TEXT NOT NULL,
    summary       TEXT NOT NULL DEFAULT '',
    category      TEXT,
    raw           JSONB NOT NULL,
    normalized    JSONB NOT NULL,
    content_hash  TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'imported', 'ignored')),
    item_id       UUID REFERENCES items(id) ON DELETE SET NULL,
    import_error  TEXT,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_job_id   UUID REFERENCES crawl_jobs(id) ON DELETE SET NULL,
    search_text   TEXT NOT NULL DEFAULT '',
    UNIQUE (source_id, upstream_id)
);
CREATE INDEX staged_listing_idx ON staged_items (source_id, status, last_seen_at DESC);
CREATE INDEX staged_search_trgm_idx ON staged_items USING gin (search_text gin_trgm_ops);

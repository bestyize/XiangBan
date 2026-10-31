//! 采集器仅持有来源配置、运行记录及草稿版本关系，不另建发布存储。
use sqlx::PgPool;

pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(r#"
CREATE TABLE IF NOT EXISTS collection_sources (
    id TEXT PRIMARY KEY,
    credential BYTEA,
    hint TEXT,
    enabled BOOLEAN NOT NULL DEFAULT false,
    interval_minutes INT NOT NULL DEFAULT 1440 CHECK(interval_minutes BETWEEN 60 AND 10080),
    last_success_at TIMESTAMPTZ,
    last_error TEXT,
    next_run_at TIMESTAMPTZ NOT NULL DEFAULT (date_trunc('day',(now()-interval '4 hours') AT TIME ZONE 'UTC') AT TIME ZONE 'UTC' + interval '1 day 4 hours'),
    lease_until TIMESTAMPTZ,
    active_run UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO collection_sources(id) VALUES ('artificial_analysis'),('openrouter') ON CONFLICT DO NOTHING;
CREATE TABLE IF NOT EXISTS collection_runs (
    id UUID PRIMARY KEY,
    source TEXT NOT NULL REFERENCES collection_sources(id),
    status TEXT NOT NULL CHECK(status IN ('running','succeeded','failed')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    item_count INT NOT NULL DEFAULT 0,
    error TEXT,
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS collection_runs_source_date ON collection_runs(source,started_at DESC);
CREATE TABLE IF NOT EXISTS collection_run_items (
    run_id UUID NOT NULL REFERENCES collection_runs(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    key TEXT NOT NULL,
    revision BIGINT NOT NULL,
    PRIMARY KEY(run_id,kind,key)
);
CREATE TABLE IF NOT EXISTS collection_owned_items (
    source TEXT NOT NULL REFERENCES collection_sources(id),
    kind TEXT NOT NULL,
    key TEXT NOT NULL,
    revision BIGINT NOT NULL,
    PRIMARY KEY(kind,key)
);
"#).execute(pool).await?;
    Ok(())
}

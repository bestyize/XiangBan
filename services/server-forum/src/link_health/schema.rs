use sqlx::PgPool;
pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(
        r#"
CREATE TABLE IF NOT EXISTS link_health_settings (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    enabled BOOLEAN NOT NULL DEFAULT false,
    interval_hours INT NOT NULL DEFAULT 24 CHECK(interval_hours BETWEEN 24 AND 720),
    next_run_at TIMESTAMPTZ NOT NULL DEFAULT now()+interval '24 hours',
    active_run UUID,
    lease_until TIMESTAMPTZ
);
INSERT INTO link_health_settings(singleton)VALUES(true)ON CONFLICT DO NOTHING;
CREATE TABLE IF NOT EXISTS link_health_runs (
    id UUID PRIMARY KEY,
    status TEXT NOT NULL CHECK(status IN('running','succeeded','partial','failed')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    total_candidates INT NOT NULL,
    selected_count INT NOT NULL,
    checked_count INT NOT NULL DEFAULT 0,
    healthy_count INT NOT NULL DEFAULT 0,
    attention_count INT NOT NULL DEFAULT 0,
    blocked_count INT NOT NULL DEFAULT 0,
    error TEXT
);
CREATE INDEX IF NOT EXISTS link_health_runs_date ON link_health_runs(started_at DESC);
CREATE TABLE IF NOT EXISTS link_health_results (
    id TEXT PRIMARY KEY CHECK(id~'^[a-f0-9]{64}$'),
    url TEXT NOT NULL,
    host TEXT NOT NULL,
    refs JSONB NOT NULL,
    status TEXT NOT NULL,
    http_status INT,
    final_url TEXT,
    error TEXT,
    duration_ms BIGINT NOT NULL,
    last_checked_at TIMESTAMPTZ NOT NULL,
    last_success_at TIMESTAMPTZ,
    failures INT NOT NULL DEFAULT 0
);
"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

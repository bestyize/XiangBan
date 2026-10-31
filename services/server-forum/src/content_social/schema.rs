use sqlx::PgPool;
pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(r#"
CREATE TABLE IF NOT EXISTS content_discussions (
    kind TEXT NOT NULL CHECK(kind IN ('pack','scheme')),
    target_id BIGINT NOT NULL CHECK(target_id>0),
    topic_id BIGINT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    managed BOOLEAN NOT NULL DEFAULT true,
    PRIMARY KEY(kind,target_id)
);
CREATE INDEX IF NOT EXISTS content_discussions_topic_idx ON content_discussions(topic_id);
CREATE TABLE IF NOT EXISTS content_bookmarks (
    uid UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('pack','scheme')),
    target_id BIGINT NOT NULL CHECK(target_id>0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY(uid,kind,target_id)
);
CREATE INDEX IF NOT EXISTS content_bookmarks_user_order ON content_bookmarks(uid,created_at DESC);
CREATE SEQUENCE IF NOT EXISTS content_report_revision_seq;
CREATE TABLE IF NOT EXISTS content_reports (
    id BIGSERIAL PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('pack','scheme')),
    target_id BIGINT NOT NULL CHECK(target_id>0),
    reporter_uid UUID NOT NULL REFERENCES users(uid),
    reason TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open','resolved','dismissed')),
    revision BIGINT NOT NULL DEFAULT nextval('content_report_revision_seq'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    handled_by TEXT,
    handled_at TIMESTAMPTZ,
    result TEXT NOT NULL DEFAULT ''
);
CREATE UNIQUE INDEX IF NOT EXISTS content_reports_open_unique ON content_reports(reporter_uid,kind,target_id) WHERE status='open';
CREATE INDEX IF NOT EXISTS content_reports_queue ON content_reports(status,created_at);
"#).execute(pool).await?;
    Ok(())
}

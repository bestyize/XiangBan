use sqlx::PgPool;
pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(
        r#"
CREATE TABLE IF NOT EXISTS community_featured_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    revision BIGINT NOT NULL DEFAULT 1
);
INSERT INTO community_featured_state(singleton) VALUES(true) ON CONFLICT DO NOTHING;
CREATE TABLE IF NOT EXISTS community_featured (
    pack_id BIGINT PRIMARY KEY REFERENCES packs(id) ON DELETE CASCADE,
    sort_order INT NOT NULL DEFAULT 0
);
CREATE SEQUENCE IF NOT EXISTS community_editor_revision_seq;
CREATE TABLE IF NOT EXISTS community_challenges (
    key TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL CHECK(ends_at>starts_at),
    submission_tag TEXT NOT NULL,
    topic_id BIGINT REFERENCES topics(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','published','closed')),
    revision BIGINT NOT NULL DEFAULT nextval('community_editor_revision_seq'),
    created_by TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS community_challenges_dates ON community_challenges(status,starts_at DESC);
CREATE TABLE IF NOT EXISTS community_challenge_entries (
    challenge_key TEXT NOT NULL REFERENCES community_challenges(key) ON DELETE CASCADE,
    pack_id BIGINT NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
    submitted_by UUID NOT NULL REFERENCES users(uid),
    featured BOOLEAN NOT NULL DEFAULT false,
    sort_order INT NOT NULL DEFAULT 0,
    revision BIGINT NOT NULL DEFAULT nextval('community_editor_revision_seq'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY(challenge_key,pack_id)
);
"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

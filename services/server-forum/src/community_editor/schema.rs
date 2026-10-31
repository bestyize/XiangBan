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
"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

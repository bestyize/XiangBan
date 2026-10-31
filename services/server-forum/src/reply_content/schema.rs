use sqlx::PgPool;
pub async fn init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(r#"
 CREATE TABLE IF NOT EXISTS reply_work_refs (
   post_id BIGINT PRIMARY KEY REFERENCES posts(id) ON DELETE CASCADE,
   refs JSONB NOT NULL DEFAULT '[]' CHECK(jsonb_typeof(refs)='array')
 );
 ALTER TABLE attachments ADD COLUMN IF NOT EXISTS post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE;
 CREATE INDEX IF NOT EXISTS attachments_reply_idx ON attachments(post_id) WHERE post_id IS NOT NULL;
 "#).execute(pool).await?;
    Ok(())
}

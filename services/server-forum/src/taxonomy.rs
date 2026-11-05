//! Canonical visible categories. Legacy slugs are input aliases, never duplicate categories.
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use sqlx::PgPool;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Deserialize)]
struct Node { id: String, label: String }
#[derive(Deserialize)]
struct Channel { id: String, parent: String, label: String, description: String }
#[derive(Deserialize)]
struct Manifest { parents: Vec<Node>, channels: Vec<Channel>, aliases: BTreeMap<String, String> }
fn manifest() -> &'static Manifest {
    static VALUE: OnceLock<Manifest> = OnceLock::new();
    VALUE.get_or_init(|| serde_json::from_str(include_str!("../editorial/channels.json")).expect("valid channel manifest"))
}
pub fn canonical(slug: &str) -> &str {
    manifest().aliases.get(slug).map(String::as_str).unwrap_or(slug)
}
pub fn equivalents(raw: &str) -> Vec<String> {
    let mut result = Vec::new();
    for slug in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let name = canonical(slug);
        for value in [slug, name].into_iter().chain(manifest().aliases.iter().filter(|(_, v)| v.as_str() == name).map(|(k, _)| k.as_str())) {
            if !result.iter().any(|s| s == value) { result.push(value.to_string()); }
        }
    }
    result
}

/// Rename rows in place so topics, drafts, permissions, reports and help escrows keep their IDs.
/// Run once after the original seeds; later administrator edits are never overwritten.
pub async fn migrate(pool: &PgPool) -> AppResult<()> {
    const NAME: &str = "canonical-community-channels-v1";
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))").bind(NAME).execute(&mut *tx).await?;
    let (done,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM data_migrations WHERE name=$1)").bind(NAME).fetch_one(&mut *tx).await?;
    if done { return Ok(()); }
    for (old, new) in &manifest().aliases {
        let old_id: Option<(i32,)> = sqlx::query_as("SELECT id FROM categories WHERE slug=$1 FOR UPDATE").bind(old).fetch_optional(&mut *tx).await?;
        if let Some((id,)) = old_id {
            let (collision,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM categories WHERE slug=$1 AND id<>$2)").bind(new).bind(id).fetch_one(&mut *tx).await?;
            if collision { return Err(AppError::Conflict(format!("分类迁移冲突：{old} 与 {new} 同时存在，未修改任何数据"))); }
            sqlx::query("UPDATE categories SET slug=$2 WHERE id=$1").bind(id).bind(new).execute(&mut *tx).await?;
        }
    }
    for (order, parent) in manifest().parents.iter().enumerate() {
        sqlx::query("INSERT INTO categories(slug,name,sort_order,award_points) VALUES($1,$2,$3,false) ON CONFLICT(slug) DO UPDATE SET name=$2,sort_order=$3")
            .bind(&parent.id).bind(&parent.label).bind((order * 100) as i32).execute(&mut *tx).await?;
    }
    for (order, channel) in manifest().channels.iter().enumerate() {
        sqlx::query("INSERT INTO categories(slug,name,description,parent_id,sort_order,award_points,admin_only,min_post_level) VALUES($1,$2,$3,(SELECT id FROM categories WHERE slug=$4),$5,$6,$7,$8) ON CONFLICT(slug) DO UPDATE SET name=$2,description=$3,parent_id=(SELECT id FROM categories WHERE slug=$4)")
            .bind(&channel.id).bind(&channel.label).bind(&channel.description).bind(&channel.parent).bind(order as i32)
            .bind(channel.parent == "learn" || channel.parent == "resources")
            .bind(matches!(channel.id.as_str(), "site-notice"|"site-rules"|"site-reports"))
            .bind(if channel.parent == "site" {3i16} else {1i16}).execute(&mut *tx).await?;
    }
    // Explicit existing topic tags identify the two newly separated tutorial categories.
    for (tag, category) in [("claude-code","learn-claude-code"),("codex","learn-codex")] {
        sqlx::query("UPDATE topics SET category_id=(SELECT id FROM categories WHERE slug=$2) WHERE category_id IN (SELECT c.id FROM categories c JOIN categories p ON p.id=c.parent_id WHERE p.slug='learn') AND id IN (SELECT tt.topic_id FROM topic_tags tt JOIN tags t ON t.id=tt.tag_id WHERE t.name=$1)")
            .bind(tag).bind(category).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO data_migrations(name) VALUES($1)").bind(NAME).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO admin_audit(admin,action,target_type,target_id,detail) VALUES('migration','community.taxonomy.migrate','categories',$1,'Canonical current channels; IDs and legacy URL aliases preserved')")
        .bind(NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

/// Initialize navigation policy once; subsequent administrator changes remain authoritative.
pub async fn migrate_sections(pool: &PgPool) -> AppResult<()> {
    const NAME: &str = "community-sections-v2";
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))").bind(NAME).execute(&mut *tx).await?;
    let (done,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM data_migrations WHERE name=$1)").bind(NAME).fetch_one(&mut *tx).await?;
    if done { return Ok(()); }
    sqlx::query("UPDATE categories c SET section=CASE COALESCE(p.slug,c.slug) WHEN 'create' THEN 'ideas' WHEN 'resources' THEN 'resources' WHEN 'site' THEN 'site' ELSE 'discuss' END FROM (SELECT id,parent_id,slug FROM categories) p WHERE p.id=COALESCE(c.parent_id,c.id)").execute(&mut *tx).await?;
    sqlx::query("UPDATE categories SET name='创意' WHERE slug='create'").execute(&mut *tx).await?;
    // Remove retired category records without restoring or deleting any existing article.
    for (old,new) in [("resource-tools","resource-open"),("resource-docs","resource-open"),("resource-music","casual-screen"),("casual-media","casual-screen"),("create-guides","learn-workflows"),("share-learn","resource-open"),("share-music","casual-screen"),("xiaoban-app","learn-workflows")] {
        sqlx::query("UPDATE topics SET category_id=(SELECT id FROM categories WHERE slug=$2) WHERE category_id=(SELECT id FROM categories WHERE slug=$1)").bind(old).bind(new).execute(&mut *tx).await?;
        sqlx::query("UPDATE editor_drafts SET category_id=(SELECT id FROM categories WHERE slug=$2) WHERE category_id=(SELECT id FROM categories WHERE slug=$1)").bind(old).bind(new).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM categories WHERE slug=$1").bind(old).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO data_migrations(name) VALUES($1)").bind(NAME).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO admin_audit(admin,action,target_type,target_id,detail) VALUES('migration','community.sections.migrate','categories',$1,'Unified sections; removed retired categories and reassigned article/draft references without changing article status')").bind(NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

/// Rename only the original built-in label; preserve administrator-customized names.
pub async fn migrate_labels(pool: &PgPool) -> AppResult<()> {
    const NAME: &str = "community-category-labels-v3";
    let mut tx=pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))").bind(NAME).execute(&mut *tx).await?;
    let (done,):(bool,)=sqlx::query_as("SELECT EXISTS(SELECT 1 FROM data_migrations WHERE name=$1)").bind(NAME).fetch_one(&mut *tx).await?;
    if done { return Ok(()); }
    sqlx::query("UPDATE categories SET name='开源讨论' WHERE slug='resource-open' AND name='开源精选'").execute(&mut *tx).await?;
    sqlx::query("INSERT INTO data_migrations(name) VALUES($1)").bind(NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aliases_are_bidirectional_without_duplicate_rows() {
        assert_eq!(canonical("share-help"), "resource-help");
        assert_eq!(equivalents("share-help"), equivalents("resource-help").into_iter().rev().collect::<Vec<_>>());
        assert_eq!(canonical("casual-work"), "casual-work");
    }
    #[tokio::test]
    async fn taxonomy_preserves_ids_history_and_legacy_queries() -> Result<(), Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else { return Ok(()); };
        let schema = format!("taxonomy_test_{}", uuid::Uuid::new_v4().simple());
        let admin = sqlx::PgPool::connect(&url).await?;
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?; crate::db::seed(&pool).await?;
        let (id,): (i32,) = sqlx::query_as("SELECT id FROM categories WHERE slug='tech-llm'").fetch_one(&pool).await?;
        let uid = uuid::Uuid::new_v4();
        sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'migration editor',3)").bind(uid).execute(&pool).await?;
        let (topic,): (i64,) = sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html,excerpt) VALUES($1,$2,'Existing topic','body','body','body') RETURNING id").bind(id).bind(uid).fetch_one(&pool).await?;
        migrate(&pool).await?; migrate(&pool).await?;
        let (new_id,): (i32,) = sqlx::query_as("SELECT id FROM categories WHERE slug='learn-local'").fetch_one(&pool).await?;
        assert_eq!(id,new_id);
        let (legacy_count,): (i64,) = sqlx::query_as("SELECT count(*) FROM categories WHERE slug LIKE 'tech%' OR slug LIKE 'share%' OR slug LIKE 'xiaoban%'").fetch_one(&pool).await?;
        assert_eq!(legacy_count,0);
        for alias in ["tech-llm","learn-local","tech","learn"] {
            let page = crate::models::list_topics(&pool,&crate::models::ListFilter{category:Some(alias.into()),..Default::default()}).await?;
            assert!(page.items.iter().any(|t|t.id==topic));
        }
        sqlx::query("UPDATE categories SET name='Edited after migration' WHERE id=$1").bind(id).execute(&pool).await?;
        migrate(&pool).await?;
        let (name,): (String,) = sqlx::query_as("SELECT name FROM categories WHERE id=$1").bind(id).fetch_one(&pool).await?;
        assert_eq!(name,"Edited after migration");
        pool.close().await;sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&admin).await?;
        admin.close().await;Ok(())
    }
    #[tokio::test]
    async fn sections_remove_retired_categories_without_restoring_posts() -> Result<(), Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else { return Ok(()); };
        let schema = format!("sections_test_{}", uuid::Uuid::new_v4().simple());
        let admin = sqlx::PgPool::connect(&url).await?;
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await?;
        let pool = crate::db::connect(&url, Some(&schema)).await?;
        crate::db::migrate(&pool).await?; crate::db::seed(&pool).await?; migrate(&pool).await?;
        sqlx::query("INSERT INTO categories(slug,name,parent_id) VALUES('resource-docs','Old docs',(SELECT id FROM categories WHERE slug='resources'))").execute(&pool).await?;
        let uid = uuid::Uuid::new_v4();
        sqlx::query("INSERT INTO users(uid,nickname,trust_level) VALUES($1,'test author',3)").bind(uid).execute(&pool).await?;
        let (topic,): (i64,) = sqlx::query_as("INSERT INTO topics(category_id,author_uid,title,body_md,body_html,excerpt,status) VALUES((SELECT id FROM categories WHERE slug='resource-docs'),$1,'Deleted article','body','body','body','deleted') RETURNING id").bind(uid).fetch_one(&pool).await?;
        migrate_sections(&pool).await?;
        let row: (String,String,String) = sqlx::query_as("SELECT c.slug,c.section,t.status FROM topics t JOIN categories c ON c.id=t.category_id WHERE t.id=$1").bind(topic).fetch_one(&pool).await?;
        assert_eq!(row,("resource-open".into(),"resources".into(),"deleted".into()));
        let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM categories WHERE slug IN ('resource-docs','resource-tools','resource-music','casual-media','create-guides')").fetch_one(&pool).await?;
        assert_eq!(count,0);
        sqlx::query("UPDATE categories SET status='archived',posting_enabled=false WHERE slug='casual-work'").execute(&pool).await?;
        migrate_sections(&pool).await?;
        let (status,): (String,) = sqlx::query_as("SELECT status FROM categories WHERE slug='casual-work'").fetch_one(&pool).await?;
        assert_eq!(status,"archived");
        sqlx::query("UPDATE categories SET name='开源精选' WHERE slug='resource-open'").execute(&pool).await?;
        migrate_labels(&pool).await?;
        let (name,):(String,)=sqlx::query_as("SELECT name FROM categories WHERE slug='resource-open'").fetch_one(&pool).await?;
        assert_eq!(name,"开源讨论");
        sqlx::query("UPDATE categories SET name='自定义讨论' WHERE slug='resource-open'").execute(&pool).await?;
        migrate_labels(&pool).await?;
        let (name,):(String,)=sqlx::query_as("SELECT name FROM categories WHERE slug='resource-open'").fetch_one(&pool).await?;
        assert_eq!(name,"自定义讨论");
        pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&admin).await?;
        Ok(())
    }

}

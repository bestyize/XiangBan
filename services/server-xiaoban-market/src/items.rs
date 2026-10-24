//! 条目写入共用的小工具。
use crate::{error::Result, upstream::slugify};
use sqlx::PgPool;

pub fn search_text(name: &str, slug: &str, namespace: &str, summary: &str, tags: &[String], category: &str) -> String {
    format!("{name} {slug} {namespace} {summary} {} {category}", tags.join(" ")).to_lowercase()
}

/// 把上游分类名解析成本地分类 ID：显式指定 > 已记住的映射 > 同名 / 同 slug 分类 > 新建。
pub async fn resolve_category(pool: &PgPool, kind: &str, label: Option<&str>, explicit: Option<i32>) -> Result<Option<i32>> {
    if explicit.is_some() {
        return Ok(explicit);
    }
    let Some(label) = label.map(str::trim).filter(|l| !l.is_empty()) else {
        return Ok(None);
    };
    let alias = label.to_lowercase();
    if let Some(id) = sqlx::query_scalar::<_, i32>("SELECT category_id FROM category_aliases WHERE kind = $1 AND alias = $2")
        .bind(kind)
        .bind(&alias)
        .fetch_optional(pool)
        .await?
    {
        return Ok(Some(id));
    }
    let slug = slugify(label, label);
    if let Some(id) = sqlx::query_scalar::<_, i32>("SELECT id FROM categories WHERE kind = $1 AND (slug = $2 OR lower(name) = $3) ORDER BY id LIMIT 1")
        .bind(kind)
        .bind(&slug)
        .bind(&alias)
        .fetch_optional(pool)
        .await?
    {
        return Ok(Some(id));
    }
    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO categories (kind, slug, name, sort_order) VALUES ($1, $2, $3, 1000)
         ON CONFLICT (kind, slug) DO UPDATE SET slug = EXCLUDED.slug RETURNING id",
    )
    .bind(kind)
    .bind(&slug)
    .bind(label)
    .fetch_one(pool)
    .await?;
    Ok(Some(id))
}

//! 条目写入共用的小工具。
use crate::{error::Result, upstream::slugify};
use sqlx::PgPool;

pub fn search_text(name: &str, slug: &str, namespace: &str, summary: &str, tags: &[String], category: &str) -> String {
    format!("{name} {slug} {namespace} {summary} {} {category}", tags.join(" ")).to_lowercase()
}

/// 上游分类 → 本地中文分类：(类型, 上游分类名小写, 本地 slug, 本地中文名)。
/// 本地 slug 在预置分类里存在时归入它；不存在就用这个 slug 和中文名新建，不再建出英文名分类。
/// SkillHub 的技能分类和专家场景是英文 slug，MCP 分类本身就是中文。
pub const KNOWN_CATEGORIES: &[(&str, &str, &str, &str)] = &[
    ("skill", "content-creation", "content", "内容创作"),
    ("skill", "office-efficiency", "office", "办公协同"),
    ("skill", "data-analysis", "data", "数据分析"),
    ("skill", "knowledge-management", "knowledge", "知识管理"),
    ("skill", "design-media", "media", "图像媒体"),
    ("skill", "dev-programming", "dev", "开发工具"),
    ("skill", "education", "education", "教育学习"),
    ("skill", "ai-agent", "ai-agent", "AI 智能体"),
    ("skill", "it-ops-security", "it-ops-security", "运维与安全"),
    ("skill", "business-ops", "business-ops", "商业运营"),
    ("skill", "professional", "professional", "专业领域"),
    ("skill", "life-service", "life-service", "生活服务"),
    ("expert", "tech", "engineering", "技术工程"),
    ("expert", "finance", "finance", "金融投资"),
    ("expert", "legal", "legal", "法务合规"),
    ("expert", "lifestyle", "life", "生活服务"),
    ("expert", "media", "content", "内容创作"),
    ("expert", "academic", "education", "教育学习"),
    ("expert", "hr", "office", "办公行政"),
    ("expert", "healthcare", "healthcare", "医疗健康"),
    ("expert", "mysticism", "mysticism", "命理玄学"),
    ("connector", "开发者工具", "dev", "开发协作"),
    ("connector", "文档工具", "docs", "办公文档"),
    ("connector", "数据库与文件", "database", "数据库"),
    ("connector", "搜索与信息检索", "web", "搜索与网页"),
    ("connector", "浏览器自动化", "web", "搜索与网页"),
    ("connector", "内容抓取", "web", "搜索与网页"),
    ("connector", "设计与创意", "design", "设计"),
    ("connector", "社交媒体", "chat", "通信"),
];

pub fn known_category(kind: &str, label: &str) -> Option<(&'static str, &'static str)> {
    let label = label.trim().to_lowercase();
    KNOWN_CATEGORIES.iter().find(|(k, l, _, _)| *k == kind && *l == label).map(|(_, _, slug, name)| (*slug, *name))
}

async fn ensure_category(pool: &PgPool, kind: &str, slug: &str, name: &str) -> Result<i32> {
    let id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO categories (kind, slug, name, sort_order) VALUES ($1, $2, $3, 1000)
         ON CONFLICT (kind, slug) DO UPDATE SET slug = EXCLUDED.slug RETURNING id",
    )
    .bind(kind)
    .bind(slug)
    .bind(name)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// 启动时把早先按英文名自动建出的分类并入中文分类（幂等）。
/// 只处理名字仍等于上游原名的分类；管理员改过名的不动。
pub async fn normalize_categories(pool: &PgPool) -> Result<()> {
    for (kind, label, local_slug, local_name) in KNOWN_CATEGORIES {
        let target = ensure_category(pool, kind, local_slug, local_name).await?;
        // 本地分类本身是未翻译的英文名（slug 即上游 slug）时，改成中文名。
        sqlx::query("UPDATE categories SET name = $3 WHERE kind = $1 AND slug = $2 AND name = $2 AND name <> $3")
            .bind(kind)
            .bind(local_slug)
            .bind(local_name)
            .execute(pool)
            .await?;
        let src_slug = slugify(label, label);
        let sources: Vec<i32> = sqlx::query_scalar("SELECT id FROM categories WHERE kind = $1 AND id <> $2 AND (slug = $3 OR lower(name) = $4)")
            .bind(kind)
            .bind(target)
            .bind(&src_slug)
            .bind(label)
            .fetch_all(pool)
            .await?;
        for src in sources {
            sqlx::query(
                "UPDATE items SET category_id = $2, search_text = search_text || ' ' || lower($3) WHERE category_id = $1",
            )
            .bind(src)
            .bind(target)
            .bind(local_name)
            .execute(pool)
            .await?;
            sqlx::query("DELETE FROM categories WHERE id = $1").bind(src).execute(pool).await?;
        }
    }
    Ok(())
}

/// 把上游分类名解析成本地分类 ID：显式指定 > 已记住的映射 > 内置映射 > 同名 / 同 slug 分类 > 新建。
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
    if let Some((slug, name)) = known_category(kind, label) {
        return Ok(Some(ensure_category(pool, kind, slug, name).await?));
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
    Ok(Some(ensure_category(pool, kind, &slug, label).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_labels_map_to_chinese_categories() {
        assert_eq!(known_category("skill", "Content-Creation"), Some(("content", "内容创作")));
        assert_eq!(known_category("expert", "tech"), Some(("engineering", "技术工程")));
        assert_eq!(known_category("connector", "浏览器自动化"), Some(("web", "搜索与网页")));
        // 类型不同不串：技能的 media 不是专家的 media。
        assert_eq!(known_category("skill", "media"), None);
        assert_eq!(known_category("skill", "tech"), None);
    }

    #[test]
    fn mapping_table_has_no_duplicate_keys() {
        let mut seen = std::collections::HashSet::new();
        for (k, l, _, _) in KNOWN_CATEGORIES {
            assert!(seen.insert((*k, *l)), "{k}/{l} listed twice");
        }
    }
}

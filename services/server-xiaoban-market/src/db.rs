use crate::{
    auth::hash_password,
    config::Config,
    error::{AppError, Result},
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect(url: &str) -> Result<PgPool> {
    Ok(PgPoolOptions::new().max_connections(10).connect(url).await?)
}

pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await.map_err(|e| AppError::Internal(format!("数据库迁移失败：{e}")))
}

const CATEGORIES: [(&str, &[(&str, &str)]); 3] = [
    (
        "skill",
        &[
            ("office", "办公协同"),
            ("docs", "文档处理"),
            ("dev", "开发工具"),
            ("data", "数据分析"),
            ("content", "内容创作"),
            ("productivity", "效率工具"),
            ("knowledge", "知识管理"),
            ("web", "网页与搜索"),
            ("media", "图像媒体"),
        ],
    ),
    (
        "expert",
        &[
            ("product", "产品设计"),
            ("engineering", "技术工程"),
            ("content", "内容创作"),
            ("finance", "金融投资"),
            ("office", "办公行政"),
            ("education", "教育学习"),
            ("life", "生活服务"),
            ("legal", "法务合规"),
        ],
    ),
    (
        "connector",
        &[
            ("dev", "开发协作"),
            ("docs", "办公文档"),
            ("database", "数据库"),
            ("web", "搜索与网页"),
            ("chat", "通信"),
            ("design", "设计"),
            ("files", "本地文件"),
        ],
    ),
];

pub async fn seed(pool: &PgPool, config: &Config) -> Result<()> {
    let has_admin: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM admin_users)").fetch_one(pool).await?;
    if !has_admin {
        let generated = config.admin_pass.is_none();
        let password = config.admin_pass.clone().unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string()[..16].to_string());
        sqlx::query("INSERT INTO admin_users (username, password_hash) VALUES ($1, $2)")
            .bind(&config.admin_user)
            .bind(hash_password(&password)?)
            .execute(pool)
            .await?;
        if generated {
            tracing::warn!("已创建管理员「{}」，初始密码：{}（只显示这一次，请登录后修改）", config.admin_user, password);
        }
    }

    for (kind, items) in CATEGORIES {
        for (i, (slug, name)) in items.iter().enumerate() {
            sqlx::query("INSERT INTO categories (kind, slug, name, sort_order) VALUES ($1, $2, $3, $4) ON CONFLICT (kind, slug) DO NOTHING")
                .bind(kind)
                .bind(slug)
                .bind(name)
                .bind(i as i32)
                .execute(pool)
                .await?;
        }
    }

    // 采集来源默认只手动触发：定时间隔为 0，不会在未配置时访问外网。
    let skillhub = json!({ "base_url": "https://api.skillhub.cn", "origin": "skillhub" });
    let modelscope = |page: &str| json!({ "origin": "modelscope", "origin_page": page, "verified": false, "source_label": "community" });
    let sources = [
        ("skillhub-skills", "SkillHub · 技能", "skill", "skillhub", skillhub.clone()),
        ("skillhub-experts", "SkillHub · 专家", "expert", "skillhub", skillhub.clone()),
        ("skillhub-connectors", "SkillHub · 连接器", "connector", "skillhub", skillhub),
        ("modelscope-mcp", "魔搭 · MCP", "connector", "generic_json", modelscope("https://www.modelscope.cn/mcp")),
        ("modelscope-skills", "魔搭 · 技能", "skill", "generic_json", modelscope("https://www.modelscope.cn/skills")),
        ("modelscope-agents", "魔搭 · 智能体", "expert", "generic_json", modelscope("https://www.modelscope.cn/agents")),
    ];
    for (key, name, kind, adapter, config) in sources {
        sqlx::query("INSERT INTO sources (key, name, kind, adapter, config) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (key) DO NOTHING")
            .bind(key)
            .bind(name)
            .bind(kind)
            .bind(adapter)
            .bind(sqlx::types::Json(config))
            .execute(pool)
            .await?;
    }
    Ok(())
}

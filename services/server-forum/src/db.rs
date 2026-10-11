use crate::error::AppError;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::str::FromStr;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    uid             UUID PRIMARY KEY,
    nickname        TEXT NOT NULL,
    avatar_url      TEXT,
    trust_level     SMALLINT NOT NULL DEFAULT 0,
    points          INT NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'active',
    muted_until     TIMESTAMPTZ,
    read_count      INT NOT NULL DEFAULT 0,
    visit_days      INT NOT NULL DEFAULT 0,
    last_visit_date DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS categories (
    id             SERIAL PRIMARY KEY,
    parent_id      INT REFERENCES categories(id),
    slug           TEXT NOT NULL UNIQUE,
    name           TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    kind           TEXT NOT NULL DEFAULT 'normal',
    sort_order     INT NOT NULL DEFAULT 0,
    min_post_level SMALLINT NOT NULL DEFAULT 1,
    on_home        BOOLEAN NOT NULL DEFAULT true,
    award_points   BOOLEAN NOT NULL DEFAULT true,
    admin_only     BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS products (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    vendor      TEXT NOT NULL DEFAULT '',
    website     TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS topics (
    id             BIGSERIAL PRIMARY KEY,
    category_id    INT NOT NULL REFERENCES categories(id),
    author_uid     UUID NOT NULL REFERENCES users(uid),
    title          TEXT NOT NULL,
    body_md        TEXT NOT NULL,
    body_html      TEXT NOT NULL,
    excerpt        TEXT NOT NULL DEFAULT '',
    status         TEXT NOT NULL DEFAULT 'normal',
    pinned         BOOLEAN NOT NULL DEFAULT false,
    featured       BOOLEAN NOT NULL DEFAULT false,
    locked         BOOLEAN NOT NULL DEFAULT false,
    reply_count    INT NOT NULL DEFAULT 0,
    like_count     INT NOT NULL DEFAULT 0,
    view_count     INT NOT NULL DEFAULT 0,
    hot_score      DOUBLE PRECISION NOT NULL DEFAULT 0,
    last_reply_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_reply_uid UUID,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    edited_at      TIMESTAMPTZ
);
-- 帖子阅读权限：0 所有人（含游客）、1 成员以上、2 老用户以上、3 守护者以上。
ALTER TABLE topics ADD COLUMN IF NOT EXISTS min_read_level SMALLINT NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS topics_category_idx ON topics (category_id, last_reply_at DESC);
CREATE INDEX IF NOT EXISTS topics_hot_idx ON topics (hot_score DESC);
CREATE INDEX IF NOT EXISTS topics_created_idx ON topics (created_at DESC);
CREATE INDEX IF NOT EXISTS topics_author_idx ON topics (author_uid, created_at DESC);

CREATE TABLE IF NOT EXISTS deals (
    topic_id   BIGINT PRIMARY KEY REFERENCES topics(id) ON DELETE CASCADE,
    product_id INT REFERENCES products(id),
    value_text TEXT NOT NULL DEFAULT '',
    conditions INT NOT NULL DEFAULT 0,
    link_url   TEXT NOT NULL DEFAULT '',
    expires_at TIMESTAMPTZ,
    long_term  BOOLEAN NOT NULL DEFAULT false
);
CREATE INDEX IF NOT EXISTS deals_product_idx ON deals (product_id);
CREATE INDEX IF NOT EXISTS deals_expires_idx ON deals (expires_at);

CREATE TABLE IF NOT EXISTS posts (
    id         BIGSERIAL PRIMARY KEY,
    topic_id   BIGINT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    author_uid UUID NOT NULL REFERENCES users(uid),
    floor      INT NOT NULL,
    reply_to   BIGINT,
    body_md    TEXT NOT NULL,
    body_html  TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'normal',
    like_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    edited_at  TIMESTAMPTZ,
    UNIQUE (topic_id, floor)
);
CREATE INDEX IF NOT EXISTS posts_author_idx ON posts (author_uid, created_at DESC);

CREATE TABLE IF NOT EXISTS tags (
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS topic_tags (
    topic_id BIGINT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    tag_id   INT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (topic_id, tag_id)
);
CREATE INDEX IF NOT EXISTS topic_tags_tag_idx ON topic_tags (tag_id);

CREATE TABLE IF NOT EXISTS likes (
    target_type TEXT NOT NULL,
    target_id   BIGINT NOT NULL,
    uid         UUID NOT NULL REFERENCES users(uid),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (target_type, target_id, uid)
);

CREATE TABLE IF NOT EXISTS bookmarks (
    uid        UUID NOT NULL REFERENCES users(uid),
    topic_id   BIGINT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (uid, topic_id)
);

CREATE TABLE IF NOT EXISTS topic_reads (
    uid      UUID NOT NULL REFERENCES users(uid),
    topic_id BIGINT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    read_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (uid, topic_id)
);

CREATE TABLE IF NOT EXISTS reports (
    id           BIGSERIAL PRIMARY KEY,
    target_type  TEXT NOT NULL,
    target_id    BIGINT NOT NULL,
    reporter_uid UUID NOT NULL REFERENCES users(uid),
    reason       TEXT NOT NULL,
    detail       TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'open',
    handled_by   TEXT,
    handled_at   TIMESTAMPTZ,
    result       TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (target_type, target_id, reporter_uid)
);
CREATE INDEX IF NOT EXISTS reports_status_idx ON reports (status, created_at DESC);

CREATE TABLE IF NOT EXISTS admins (
    id            UUID PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    token_version INT NOT NULL DEFAULT 0,
    failed_logins INT NOT NULL DEFAULT 0,
    locked_until  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS admin_audit (
    id          BIGSERIAL PRIMARY KEY,
    admin       TEXT NOT NULL,
    action      TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id   TEXT NOT NULL,
    detail      TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
"#;

pub async fn connect(url: &str, schema: Option<&str>) -> Result<PgPool, AppError> {
    let mut options = PgConnectOptions::from_str(url).map_err(|e| AppError::Internal(format!("database url: {e}")))?;
    if let Some(schema) = schema {
        options = options.options([("search_path", schema)]);
    }
    Ok(PgPoolOptions::new().max_connections(10).connect_with(options).await?)
}

pub async fn migrate(pool: &PgPool) -> Result<(), AppError> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    // 搜索用的三元组索引是可选的：数据库没有 pg_trgm 扩展时退化为普通 ILIKE 扫描。
    if sqlx::raw_sql("CREATE EXTENSION IF NOT EXISTS pg_trgm").execute(pool).await.is_ok() {
        let _ = sqlx::raw_sql(
            "CREATE INDEX IF NOT EXISTS topics_title_trgm_idx ON topics USING gin (title gin_trgm_ops)",
        )
        .execute(pool)
        .await;
    }
    Ok(())
}

/// 分类种子：(slug, 名称, 说明, 类型, 父 slug, 最低发帖等级, 进首页, 加积分, 仅管理员)
const CATEGORIES: &[(&str, &str, &str, &str, Option<&str>, i16, bool, bool, bool)] = &[
    // 板块顺序就是导航顺序：福利羊毛只是其中一个板块，不排在最前面。
    ("tech", "技术交流", "开发、运维和模型部署", "normal", None, 1, true, true, false),
    ("tech-ai-app", "AI 应用开发", "", "normal", Some("tech"), 1, true, true, false),
    ("tech-dev", "前端 · 后端 · 移动端", "", "normal", Some("tech"), 1, true, true, false),
    ("tech-ops", "服务器与运维", "", "normal", Some("tech"), 1, true, true, false),
    ("tech-models", "模型部署", "本地跑模型、推理部署", "normal", Some("tech"), 1, true, true, false),
    ("share", "资源分享", "合规的开源项目、工具和学习资料", "normal", None, 1, true, true, false),
    ("share-oss", "开源项目", "", "normal", Some("share"), 1, true, true, false),
    ("share-tools", "工具软件", "免费软件和效率工具", "normal", Some("share"), 1, true, true, false),
    ("share-learn", "学习资料", "课程、电子书、文档", "normal", Some("share"), 1, true, true, false),
    ("share-prompts", "提示词 · 工作流", "Prompt、Agent 配置、自动化流程", "normal", Some("share"), 1, true, true, false),
    ("share-made", "自制作品", "自己做的工具和资源", "normal", Some("share"), 1, true, true, false),
    ("deals", "福利羊毛", "大家分享的免费额度、试用和优惠", "deal", None, 1, true, true, false),
    ("deals-ai", "AI 羊毛", "大模型 API、AI 对话、编程工具、绘图视频音频的免费额度和优惠", "deal", Some("deals"), 1, true, true, false),
    ("deals-cloud", "云服务器 · GPU", "VPS 促销、GPU 试用、算力赠金", "deal", Some("deals"), 1, true, true, false),
    ("deals-dev", "开发者福利", "域名、CDN、数据库、学生包", "deal", Some("deals"), 1, true, true, false),
    ("deals-public", "公益站", "只收录信息，站方不做担保", "deal", Some("deals"), 1, true, true, false),
    ("deals-other", "其他福利", "不属于以上分类的福利和优惠", "deal", Some("deals"), 1, true, true, false),
    ("casual", "随便唠唠", "什么都可以聊，别引战", "normal", None, 1, true, false, false),
    ("casual-work", "职场", "加班、跳槽、远程办公", "normal", Some("casual"), 1, true, false, false),
    ("casual-offwork", "下班了", "今日下班、打卡、吐槽", "normal", Some("casual"), 1, true, false, false),
    ("casual-gear", "数码装备", "键盘、显示器、NAS", "normal", Some("casual"), 1, true, false, false),
    ("casual-indie", "副业与独立开发", "", "normal", Some("casual"), 1, true, false, false),
    ("casual-fun", "吃喝玩乐", "", "normal", Some("casual"), 1, true, false, false),
    ("casual-chat", "闲聊灌水", "不进首页，不加积分", "normal", Some("casual"), 1, false, false, false),
    ("site", "站务", "公告、规则和反馈", "normal", None, 1, true, false, false),
    ("site-notice", "公告", "", "normal", Some("site"), 1, true, false, true),
    ("site-rules", "规则", "", "normal", Some("site"), 1, true, false, true),
    ("site-feedback", "建议反馈", "", "normal", Some("site"), 1, true, false, false),
    ("site-reports", "举报处理公示", "", "normal", Some("site"), 1, true, false, true),
];

/// 产品种子：(slug, 名称, 厂商, 官网)
const PRODUCTS: &[(&str, &str, &str, &str)] = &[
    ("openai", "OpenAI", "OpenAI", "https://openai.com"),
    ("claude", "Claude", "Anthropic", "https://claude.ai"),
    ("gemini", "Gemini", "Google", "https://gemini.google.com"),
    ("deepseek", "DeepSeek", "深度求索", "https://www.deepseek.com"),
    ("qwen", "通义千问", "阿里云", "https://tongyi.aliyun.com"),
    ("kimi", "Kimi", "月之暗面", "https://kimi.com"),
    ("zhipu", "智谱 GLM", "智谱 AI", "https://open.bigmodel.cn"),
    ("doubao", "豆包", "字节跳动", "https://www.doubao.com"),
    ("siliconflow", "硅基流动", "SiliconFlow", "https://siliconflow.cn"),
    ("openrouter", "OpenRouter", "OpenRouter", "https://openrouter.ai"),
    ("groq", "Groq", "Groq", "https://groq.com"),
    ("cursor", "Cursor", "Anysphere", "https://cursor.com"),
    ("github-copilot", "GitHub Copilot", "GitHub", "https://github.com/features/copilot"),
    ("windsurf", "Windsurf", "Windsurf", "https://windsurf.com"),
    ("midjourney", "Midjourney", "Midjourney", "https://www.midjourney.com"),
    ("suno", "Suno", "Suno", "https://suno.com"),
    ("elevenlabs", "ElevenLabs", "ElevenLabs", "https://elevenlabs.io"),
    ("huggingface", "Hugging Face", "Hugging Face", "https://huggingface.co"),
    ("cloudflare", "Cloudflare", "Cloudflare", "https://www.cloudflare.com"),
    ("vercel", "Vercel", "Vercel", "https://vercel.com"),
    ("aws", "AWS", "Amazon", "https://aws.amazon.com"),
    ("google-cloud", "Google Cloud", "Google", "https://cloud.google.com"),
    ("azure", "Azure", "Microsoft", "https://azure.microsoft.com"),
    ("aliyun", "阿里云", "阿里巴巴", "https://www.aliyun.com"),
    ("tencent-cloud", "腾讯云", "腾讯", "https://cloud.tencent.com"),
    ("volcengine", "火山引擎", "字节跳动", "https://www.volcengine.com"),
    ("vultr", "Vultr", "Vultr", "https://www.vultr.com"),
    ("digitalocean", "DigitalOcean", "DigitalOcean", "https://www.digitalocean.com"),
    ("other", "其他", "", ""),
];

/// 只在表为空时写入种子，后台改过的分类和产品不会被覆盖。
pub async fn seed(pool: &PgPool) -> Result<(), AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories").fetch_one(pool).await?;
    if count == 0 {
        for (i, (slug, name, desc, kind, parent, level, home, points, admin_only)) in CATEGORIES.iter().enumerate() {
            sqlx::query(
                "INSERT INTO categories (parent_id, slug, name, description, kind, sort_order, min_post_level, on_home, award_points, admin_only)
                 VALUES ((SELECT id FROM categories WHERE slug = $1), $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            )
            .bind(parent)
            .bind(slug)
            .bind(name)
            .bind(desc)
            .bind(kind)
            .bind(i as i32)
            .bind(level)
            .bind(home)
            .bind(points)
            .bind(admin_only)
            .execute(pool)
            .await?;
        }
    }
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products").fetch_one(pool).await?;
    if count == 0 {
        for (slug, name, vendor, website) in PRODUCTS {
            sqlx::query("INSERT INTO products (slug, name, vendor, website) VALUES ($1, $2, $3, $4)")
                .bind(slug)
                .bind(name)
                .bind(vendor)
                .bind(website)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

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

-- 一次性的数据迁移记录：做过的不再重复做，后台之后的改动不会被覆盖。
CREATE TABLE IF NOT EXISTS data_migrations (
    name       TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 内置标签：发帖时按这个分类给出一组可一键选用的标签（追加列，已部署的库启动时自动补上）。
ALTER TABLE categories ADD COLUMN IF NOT EXISTS preset_tags TEXT[] NOT NULL DEFAULT '{}';

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
    value_text TEXT NOT NULL DEFAULT '',
    conditions INT NOT NULL DEFAULT 0,
    link_url   TEXT NOT NULL DEFAULT '',
    expires_at TIMESTAMPTZ,
    long_term  BOOLEAN NOT NULL DEFAULT false
);
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
    ("tech", "技术交流", "AI 大模型、AI 提效、AI 硬件和 AI 创意的技术交流", "normal", None, 1, true, true, false),
    ("tech-llm", "AI 大模型", "本地跑大模型、模型评测、自建 API 中转的技术教程（只讲技术，不推广、不出售中转服务）", "normal", Some("tech"), 1, true, true, false),
    ("tech-productivity", "AI 提效", "用 AI 重塑工作流，加速研发和日常工作", "normal", Some("tech"), 1, true, true, false),
    ("tech-hardware", "AI 硬件", "小智机器人、开源蝴蝶飞行器、机器狗、桌宠机器人等 AI 硬件", "normal", Some("tech"), 1, true, true, false),
    ("tech-creative", "AI 创意", "AI 绘画、视频、音乐、写作等创意玩法和作品，也可以分享提示词与工作流", "normal", Some("tech"), 1, true, true, false),
    ("share", "资源分享", "分享合法来源的软件、影视、音乐和学习资料", "normal", None, 1, true, true, false),
    ("share-software", "精品软件", "好用的正版、开源和免费软件，不要贴破解版", "normal", Some("share"), 1, true, true, false),
    ("share-movies", "电影资源", "片单、影评，以及公版和授权影片，不要贴盗版下载链接", "normal", Some("share"), 1, true, true, false),
    ("share-music", "音乐资源", "歌单、乐评，以及公版和授权音乐，不要贴盗版下载链接", "normal", Some("share"), 1, true, true, false),
    ("share-learn", "学习资料", "课程、电子书、文档和笔记，请只分享可以公开的资料", "normal", Some("share"), 1, true, true, false),
    ("deals", "福利羊毛", "大家分享的免费额度、试用和优惠", "deal", None, 1, true, true, false),
    ("deals-ai", "AI 羊毛", "大模型 API、AI 对话、编程工具、绘图视频音频的免费额度和优惠", "deal", Some("deals"), 1, true, true, false),
    ("deals-cloud", "云服务器 · GPU", "VPS 促销、GPU 试用、算力赠金", "deal", Some("deals"), 1, true, true, false),
    ("deals-dev", "开发者福利", "域名、CDN、数据库、学生包", "deal", Some("deals"), 1, true, true, false),
    ("deals-public", "公益站", "只收录信息，站方不做担保", "deal", Some("deals"), 1, true, true, false),
    ("deals-other", "其他福利", "不属于以上分类的福利和优惠", "deal", Some("deals"), 1, true, true, false),
    ("casual", "随便唠唠", "什么都可以聊，别引战", "normal", None, 1, true, false, false),
    ("casual-work", "职场", "加班、跳槽、远程办公", "normal", Some("casual"), 1, true, false, false),
    ("casual-offwork", "下班了", "今日下班、打卡、吐槽", "normal", Some("casual"), 1, true, false, false),
    ("casual-loss", "亏了吗", "今天亏了吗？聊聊股票、基金、副业的盈亏和心态；只聊经验和心情，不荐股、不拉群、不带单", "normal", Some("casual"), 1, true, false, false),
    ("casual-dream", "梦想和人生", "聊聊理想、目标、转行和人生的选择，分享你的故事和想法", "normal", Some("casual"), 1, true, false, false),
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

/// 各分类内置的标签（slug → 标签）。写入时的形态和发帖时一致：小写、不含空格。
const PRESET_TAGS: &[(&str, &[&str])] = &[
    ("share-software", &["windows", "macos", "linux", "android", "ios", "开源", "免费", "效率工具", "开发工具", "设计工具"]),
    ("share-movies", &["片单推荐", "影评", "纪录片", "动画", "科幻", "悬疑", "喜剧", "经典老片", "公版", "授权"]),
    ("share-music", &["歌单推荐", "乐评", "古典", "流行", "摇滚", "电子", "爵士", "纯音乐", "独立音乐", "公版"]),
    ("share-learn", &["编程", "ai", "英语", "考研", "考证", "设计", "数学", "电子书", "公开课", "笔记"]),
];

fn preset_tags_for(slug: &str) -> Vec<String> {
    PRESET_TAGS
        .iter()
        .find(|(s, _)| *s == slug)
        .map(|(_, tags)| tags.iter().map(|t| t.to_string()).collect())
        .unwrap_or_default()
}

/// 只在表为空时写入种子，后台改过的分类不会被覆盖。
pub async fn seed(pool: &PgPool) -> Result<(), AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories").fetch_one(pool).await?;
    if count == 0 {
        for (i, (slug, name, desc, kind, parent, level, home, points, admin_only)) in CATEGORIES.iter().enumerate() {
            sqlx::query(
                "INSERT INTO categories (parent_id, slug, name, description, kind, sort_order, min_post_level, on_home, award_points, admin_only, preset_tags)
                 VALUES ((SELECT id FROM categories WHERE slug = $1), $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
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
            .bind(preset_tags_for(slug))
            .execute(pool)
            .await?;
        }
        // 新库的种子已经是最新结构，不需要再迁移。
        for m in BOARD_MIGRATIONS {
            sqlx::query("INSERT INTO data_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING").bind(m.name).execute(pool).await?;
        }
    } else {
        for m in BOARD_MIGRATIONS {
            apply_board_migration(pool, m).await?;
        }
    }
    Ok(())
}

/// 一次板块调整：把某个一级分类下的二级板块换成种子里的新板块，旧板块里的帖子并入指定的新板块。
struct BoardMigration {
    /// 记录名，做过一次就写进 `data_migrations`，不再重复。
    name: &'static str,
    /// 一级分类的 slug。
    parent: &'static str,
    /// 这次要写入（新增或按种子覆盖名称、说明、标签）的板块 slug，都是 `parent` 下的二级板块。其余板块不动。
    boards: &'static [&'static str],
    /// (旧板块 slug, 帖子并入的新板块 slug)
    moves: &'static [(&'static str, &'static str)],
}

const BOARD_MIGRATIONS: &[BoardMigration] = &[
    // 资源分享：开源项目、工具软件、提示词·工作流、自制作品 → 精品软件 / 电影资源 / 音乐资源 / 学习资料
    BoardMigration {
        name: "share-boards-v1",
        parent: "share",
        boards: &["share-software", "share-movies", "share-music", "share-learn"],
        moves: &[
            ("share-oss", "share-software"),
            ("share-tools", "share-software"),
            ("share-made", "share-software"),
            ("share-prompts", "share-learn"),
        ],
    },
    // 技术交流：AI 应用开发、前端·后端·移动端、服务器与运维、模型部署 → AI 大模型 / AI 提效 / AI 硬件
    BoardMigration {
        name: "tech-boards-v1",
        parent: "tech",
        boards: &["tech-llm", "tech-productivity", "tech-hardware"],
        moves: &[
            ("tech-ai-app", "tech-productivity"),
            ("tech-dev", "tech-productivity"),
            ("tech-ops", "tech-llm"),
            ("tech-models", "tech-llm"),
        ],
    },
    // 技术交流：新增"AI 创意"，放在最后
    BoardMigration { name: "tech-creative-v1", parent: "tech", boards: &["tech-creative"], moves: &[] },
    // 随便唠唠：新增"亏了吗"，放在"下班了"后面
    BoardMigration { name: "casual-loss-v1", parent: "casual", boards: &["casual-loss"], moves: &[] },
    // 随便唠唠：新增"梦想和人生"，放在"亏了吗"后面
    BoardMigration { name: "casual-dream-v1", parent: "casual", boards: &["casual-dream"], moves: &[] },
];

/// 已部署的库执行一次板块调整：`boards` 里的板块用种子里的名称、说明和内置标签写入（已有的同 slug 板块被更新），
/// 旧板块的帖子并入指定的新板块后删除旧板块，一级分类的说明也一起更新。新板块插在种子里前一个板块的后面，已有分类的排序不动。
///
/// 做完记进 `data_migrations`，之后不再执行，所以管理员后来改过、甚至重新建过同名 slug 的分类都不会被动到。
async fn apply_board_migration(pool: &PgPool, m: &BoardMigration) -> Result<(), AppError> {
    let done: Option<(String,)> =
        sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(m.name).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;
    let parent: Option<(i32, i32)> =
        sqlx::query_as("SELECT id, sort_order FROM categories WHERE slug = $1").bind(m.parent).fetch_optional(&mut *tx).await?;
    // 一级分类不在（被管理员删了）就没有可调整的，记一笔免得每次启动都查。
    if let Some((parent_id, parent_order)) = parent {
        let seed_children: Vec<_> = CATEGORIES.iter().filter(|c| c.4 == Some(m.parent)).collect();
        for (idx, (slug, name, desc, kind, _, level, home, points, admin_only)) in seed_children.iter().enumerate() {
            if !m.boards.contains(slug) {
                continue;
            }
            let existed: Option<(i32,)> =
                sqlx::query_as("SELECT sort_order FROM categories WHERE slug = $1").bind(slug).fetch_optional(&mut *tx).await?;
            // 新板块插在种子里排在它前面的那个板块后面（第一个就插在一级分类后面）；
            // 前一个板块不在了就退回到一级分类后面。插入点之后的分类整体顺延一位，所以后台改过的排序保持相对顺序。
            let pred_slug = if idx == 0 { m.parent } else { seed_children[idx - 1].0 };
            let pred_order: i32 = sqlx::query_as::<_, (i32,)>("SELECT sort_order FROM categories WHERE slug = $1")
                .bind(pred_slug)
                .fetch_optional(&mut *tx)
                .await?
                .map(|r| r.0)
                .unwrap_or(parent_order);
            if existed.is_none() {
                sqlx::query("UPDATE categories SET sort_order = sort_order + 1 WHERE sort_order > $1").bind(pred_order).execute(&mut *tx).await?;
            }
            sqlx::query(
                "INSERT INTO categories (parent_id, slug, name, description, kind, sort_order, min_post_level, on_home, award_points, admin_only, preset_tags)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                 ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description,
                    preset_tags = EXCLUDED.preset_tags",
            )
            .bind(parent_id)
            .bind(slug)
            .bind(name)
            .bind(desc)
            .bind(kind)
            .bind(pred_order + 1)
            .bind(level)
            .bind(home)
            .bind(points)
            .bind(admin_only)
            .bind(preset_tags_for(slug))
            .execute(&mut *tx)
            .await?;
        }
        if let Some(desc) = CATEGORIES.iter().find(|c| c.0 == m.parent).map(|c| c.2) {
            sqlx::query("UPDATE categories SET description = $1 WHERE id = $2").bind(desc).bind(parent_id).execute(&mut *tx).await?;
        }
        for (old, target) in m.moves {
            let old_id: Option<(i32,)> =
                sqlx::query_as("SELECT id FROM categories WHERE slug = $1").bind(old).fetch_optional(&mut *tx).await?;
            let Some((old_id,)) = old_id else { continue };
            sqlx::query("UPDATE topics SET category_id = (SELECT id FROM categories WHERE slug = $1) WHERE category_id = $2")
                .bind(target)
                .bind(old_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM categories WHERE id = $1").bind(old_id).execute(&mut *tx).await?;
        }
    }
    sqlx::query("INSERT INTO data_migrations (name) VALUES ($1)").bind(m.name).execute(&mut *tx).await?;
    tx.commit().await?;
    tracing::info!("板块调整 {} 已完成", m.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::topics::normalize_tag;

    #[test]
    fn preset_tags_are_stored_exactly_as_users_would_type_them() {
        for (slug, tags) in PRESET_TAGS {
            assert!(CATEGORIES.iter().any(|c| c.0 == *slug), "内置标签指向了不存在的分类 {slug}");
            assert!(tags.len() <= 20, "{slug} 内置标签太多");
            let mut seen = std::collections::HashSet::new();
            for t in *tags {
                // 点一下就把这个标签原样加到帖子上，所以必须已经是归一化后的写法。
                assert_eq!(normalize_tag(t).as_deref(), Some(*t), "{slug} 里的 {t:?} 不是归一化后的写法");
                assert!(t.chars().count() <= 20, "{slug} 里的 {t:?} 太长");
                assert!(seen.insert(*t), "{slug} 里的 {t:?} 重复");
            }
        }
    }

    fn children(parent: &str) -> Vec<&'static str> {
        CATEGORIES.iter().filter(|c| c.4 == Some(parent)).map(|c| c.1).collect()
    }

    #[test]
    fn share_boards_are_the_four_requested_ones() {
        assert_eq!(children("share"), ["精品软件", "电影资源", "音乐资源", "学习资料"]);
        for (slug, tags) in PRESET_TAGS {
            assert!(!tags.is_empty(), "{slug} 没有内置标签");
        }
    }

    #[test]
    fn tech_boards_are_the_requested_ones_in_order() {
        assert_eq!(children("tech"), ["AI 大模型", "AI 提效", "AI 硬件", "AI 创意"]);
    }

    #[test]
    fn casual_has_the_loss_and_dream_boards_after_offwork() {
        let names = children("casual");
        let at = |n: &str| names.iter().position(|x| *x == n).unwrap_or_else(|| panic!("缺少板块 {n}"));
        assert_eq!(at("亏了吗"), at("下班了") + 1);
        assert_eq!(at("梦想和人生"), at("亏了吗") + 1);
    }

    #[test]
    fn board_migrations_are_consistent_with_the_seed() {
        let mut names = std::collections::HashSet::new();
        for m in BOARD_MIGRATIONS {
            assert!(names.insert(m.name), "迁移名重复 {}", m.name);
            for slug in m.boards {
                assert!(
                    CATEGORIES.iter().any(|c| c.0 == *slug && c.4 == Some(m.parent)),
                    "{} 要写入的 {slug} 不是 {} 下的板块",
                    m.name,
                    m.parent
                );
            }
            assert!(CATEGORIES.iter().any(|c| c.0 == m.parent && c.4.is_none()), "{} 的一级分类不在种子里", m.name);
            for (old, target) in m.moves {
                // 旧板块不能再出现在种子里（否则新库里它会被迁移掉），并入的目标必须是这个一级分类下的新板块。
                assert!(!CATEGORIES.iter().any(|c| c.0 == *old), "旧板块 {old} 不应该还在种子里");
                assert!(m.boards.contains(target), "{old} 并入的目标 {target} 必须是这次写入的板块");
            }
        }
    }

    #[test]
    fn category_slugs_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for c in CATEGORIES {
            assert!(seen.insert(c.0), "重复的分类 slug {}", c.0);
        }
    }
}

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

CREATE TABLE IF NOT EXISTS user_follows (
    follower_uid UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    followed_uid UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (follower_uid, followed_uid),
    CHECK (follower_uid <> followed_uid)
);
CREATE INDEX IF NOT EXISTS user_follows_following_idx ON user_follows (follower_uid, created_at DESC, followed_uid);
CREATE INDEX IF NOT EXISTS user_follows_followers_idx ON user_follows (followed_uid, created_at DESC, follower_uid);

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
    pin_scope      TEXT NOT NULL DEFAULT 'none',
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
ALTER TABLE topics ADD COLUMN IF NOT EXISTS pin_scope TEXT NOT NULL DEFAULT 'none';
UPDATE topics SET pin_scope = 'site' WHERE pinned AND pin_scope = 'none';
CREATE INDEX IF NOT EXISTS topics_category_idx ON topics (category_id, last_reply_at DESC);
CREATE INDEX IF NOT EXISTS topics_hot_idx ON topics (hot_score DESC);
CREATE INDEX IF NOT EXISTS topics_created_idx ON topics (created_at DESC);
CREATE INDEX IF NOT EXISTS topics_author_idx ON topics (author_uid, created_at DESC);

CREATE TABLE IF NOT EXISTS editor_authors (
    uid UUID PRIMARY KEY REFERENCES users(uid),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS editor_drafts (
    id UUID PRIMARY KEY,
    author_uid UUID NOT NULL REFERENCES users(uid),
    external_id TEXT NOT NULL,
    category_id INT NOT NULL REFERENCES categories(id),
    title TEXT NOT NULL,
    body_md TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    topic_id BIGINT REFERENCES topics(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(author_uid, external_id)
);
CREATE TABLE IF NOT EXISTS editor_tokens (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    author_uid UUID NOT NULL REFERENCES editor_authors(uid),
    can_publish BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

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

-- 帖子附件：文件本体放在磁盘（FORUM_UPLOAD_DIR/{id}），这里只记元数据。
-- topic_id 为空表示已上传、还没挂到帖子上，超过一天没挂的由定时任务清理。
CREATE TABLE IF NOT EXISTS attachments (
    id           UUID PRIMARY KEY,
    topic_id     BIGINT REFERENCES topics(id) ON DELETE CASCADE,
    uploader_uid UUID NOT NULL REFERENCES users(uid),
    filename     TEXT NOT NULL,
    size         BIGINT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS attachments_topic_idx ON attachments (topic_id);
CREATE INDEX IF NOT EXISTS attachments_pending_idx ON attachments (uploader_uid, created_at) WHERE topic_id IS NULL;

-- 正文插图：文件在磁盘（FORUM_UPLOAD_DIR/img-{id}），公开按 id 访问。
-- 上传一天后正文里没有引用它的（帖子和回复的 Markdown 里搜不到 id）由定时任务清理。
CREATE TABLE IF NOT EXISTS images (
    id           UUID PRIMARY KEY,
    uploader_uid UUID NOT NULL REFERENCES users(uid),
    mime         TEXT NOT NULL,
    size         BIGINT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS images_uploader_idx ON images (uploader_uid, created_at DESC);

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

ALTER TABLE images ADD COLUMN IF NOT EXISTS editor_pending BOOLEAN NOT NULL DEFAULT false;
CREATE TABLE IF NOT EXISTS editor_draft_assets (
    draft_id UUID NOT NULL REFERENCES editor_drafts(id) ON DELETE CASCADE,
    image_id UUID NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    PRIMARY KEY(draft_id, path)
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
    sqlx::raw_sql(POINTS_SCHEMA).execute(pool).await?;
    sqlx::raw_sql(CONTENT_SCHEMA).execute(pool).await?;
    sqlx::raw_sql(COMPLIANCE_SCHEMA).execute(pool).await?;
    sqlx::raw_sql(DISCOVER_SCHEMA).execute(pool).await?;
    crate::collectors::schema::init(pool).await?;
    crate::community_editor::schema::init(pool).await?;
    crate::tool_mirrors::init(pool).await?;
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

const POINTS_SCHEMA: &str = r#"
-- 已有用户不补发注册奖励；新增用户首次加入论坛时领取一次。
ALTER TABLE users ADD COLUMN IF NOT EXISTS points_initialized BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE users ALTER COLUMN points_initialized SET DEFAULT false;
ALTER TABLE attachments ADD COLUMN IF NOT EXISTS price INT NOT NULL DEFAULT 0 CHECK (price BETWEEN 0 AND 10);
CREATE TABLE IF NOT EXISTS points_ledger (
    id BIGSERIAL PRIMARY KEY,
    uid UUID NOT NULL REFERENCES users(uid),
    delta INT NOT NULL CHECK (delta <> 0),
    balance_after INT NOT NULL CHECK (balance_after >= 0),
    kind TEXT NOT NULL,
    event_key TEXT NOT NULL,
    counterparty UUID REFERENCES users(uid),
    reference TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL,
    reward_day DATE NOT NULL DEFAULT (now() AT TIME ZONE 'Asia/Shanghai')::date,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (uid, event_key)
);
CREATE INDEX IF NOT EXISTS points_ledger_uid_idx ON points_ledger (uid, id DESC);
CREATE INDEX IF NOT EXISTS points_ledger_rewards_idx ON points_ledger (uid, kind, reward_day);
CREATE TABLE IF NOT EXISTS forum_registrations (
    uid UUID PRIMARY KEY REFERENCES users(uid),
    inviter_uid UUID REFERENCES users(uid),
    invite_status TEXT NOT NULL DEFAULT 'pending' CHECK (invite_status IN ('pending', 'rewarded', 'capped', 'ineligible')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (uid <> inviter_uid)
);
CREATE INDEX IF NOT EXISTS forum_registrations_inviter_idx ON forum_registrations (inviter_uid, created_at DESC);
CREATE TABLE IF NOT EXISTS attachment_purchases (
    attachment_id UUID NOT NULL REFERENCES attachments(id) ON DELETE CASCADE,
    buyer_uid UUID NOT NULL REFERENCES users(uid),
    seller_uid UUID NOT NULL REFERENCES users(uid),
    price INT NOT NULL CHECK (price BETWEEN 1 AND 10),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (attachment_id, buyer_uid),
    CHECK (buyer_uid <> seller_uid)
);
CREATE TABLE IF NOT EXISTS help_requests (
    topic_id BIGINT PRIMARY KEY REFERENCES topics(id),
    bounty INT NOT NULL DEFAULT 0 CHECK (bounty BETWEEN 0 AND 1000),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'solved', 'closed', 'expired')),
    accepted_post_id BIGINT REFERENCES posts(id),
    expires_at TIMESTAMPTZ NOT NULL,
    settled_at TIMESTAMPTZ,
    CHECK ((status = 'solved') = (accepted_post_id IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS help_requests_expiry_idx ON help_requests (expires_at) WHERE status = 'open';
"#;

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
    ("share-help", "资源求助", "求助合法来源的软件、学习资料和资源，可使用积分悬赏", "normal", Some("share"), 1, true, true, false),
    ("xiaoban", "小伴产品", "小伴系列产品的官方介绍与更新：壁纸、游戏、App。由站方发布，欢迎在帖子下回复反馈", "normal", None, 3, true, false, false),
    ("xiaoban-wallpaper", "小伴壁纸", "小伴壁纸的下载、更新和使用技巧，也欢迎分享你自己做的壁纸、特效、组件资源包", "normal", Some("xiaoban"), 0, true, false, false),
    ("xiaoban-game", "小伴游戏", "小伴游戏的新玩法、攻略和更新，也欢迎分享你自己做的游戏资源包", "normal", Some("xiaoban"), 0, true, false, false),
    ("xiaoban-app", "小伴App", "小伴 App 的功能介绍、更新和使用教程（官方发布，欢迎回复反馈）", "normal", Some("xiaoban"), 3, true, false, false),
    ("casual", "随便唠唠", "什么都可以聊，别引战", "normal", None, 1, true, false, false),
    ("casual-work", "职场", "加班、跳槽、远程办公", "normal", Some("casual"), 1, true, false, false),
    ("casual-offwork", "下班了", "今日下班、打卡、吐槽", "normal", Some("casual"), 1, true, false, false),
    ("casual-loss", "亏了吗", "今天亏了吗？聊聊股票、基金、副业的盈亏和心态；只聊经验和心情，不荐股、不拉群、不带单", "normal", Some("casual"), 1, true, false, false),
    ("casual-dream", "梦想和人生", "聊聊理想、目标、转行和人生的选择，分享你的故事和想法", "normal", Some("casual"), 1, true, false, false),
    ("casual-gear", "数码装备", "键盘、显示器、NAS", "normal", Some("casual"), 1, true, false, false),
    ("casual-indie", "副业与独立开发", "", "normal", Some("casual"), 1, true, false, false),
    ("casual-fun", "吃喝玩乐", "", "normal", Some("casual"), 1, true, false, false),
    ("casual-media", "片单乐评", "分享片单、歌单和乐评，只聊内容，不贴盗版链接", "normal", Some("casual"), 1, true, false, false),
    ("casual-think", "哲思", "聊聊想法和观点，不吵架", "normal", Some("casual"), 1, true, false, false),
    ("casual-help", "互助", "日常的小事，互相帮个忙", "normal", Some("casual"), 1, true, false, false),
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
    ("share-movies", &["片单推荐", "影评", "纪录片", "动画", "科幻", "悬疑", "喜剧", "经典老片", "公版", "授权", "4k", "美剧"]),
    ("share-music", &["歌单推荐", "乐评", "古典", "流行", "摇滚", "电子", "爵士", "纯音乐", "独立音乐", "公版"]),
    ("xiaoban-wallpaper", &["更新日志", "下载", "使用教程", "桌面版", "手机版", "网站", "动态壁纸", "桌宠"]),
    ("xiaoban-game", &["更新日志", "新游戏", "玩法介绍", "攻略", "水墨剑客", "暮色滑翔"]),
    ("xiaoban-app", &["更新日志", "下载", "使用教程", "新功能", "相伴", "下班了么", "android", "桌面版"]),
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
    apply_movie_tag_migration(pool).await?;
    apply_open_share_boards_migration(pool).await?;
    remove_deals(pool).await?;
    Ok(())
}

/// 福利羊毛功能整体下线：删除羊毛分类下的全部帖子（连同回复、点赞、收藏、标签关联、附件记录）、
/// 相关举报和站方稿件，再删掉羊毛分类和 `deals` 表。只做一次，记进 `data_migrations`。
/// 附件和插图的磁盘文件不在这里清理，运维可以按 `attachments`/`images` 里没有记录的文件另行清除。
async fn remove_deals(pool: &PgPool) -> Result<(), AppError> {
    const NAME: &str = "remove-deals-v1";
    let done: Option<(String,)> =
        sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(NAME).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    sqlx::raw_sql(
        "DELETE FROM reports WHERE (target_type = 'topic' AND target_id IN
                (SELECT id FROM topics WHERE category_id IN (SELECT id FROM categories WHERE kind = 'deal')))
            OR (target_type = 'post' AND target_id IN
                (SELECT p.id FROM posts p JOIN topics t ON t.id = p.topic_id
                 WHERE t.category_id IN (SELECT id FROM categories WHERE kind = 'deal')));
         DELETE FROM editor_drafts WHERE category_id IN (SELECT id FROM categories WHERE kind = 'deal')
            OR topic_id IN (SELECT id FROM topics WHERE category_id IN (SELECT id FROM categories WHERE kind = 'deal'));
         DELETE FROM topics WHERE category_id IN (SELECT id FROM categories WHERE kind = 'deal');
         DELETE FROM categories WHERE kind = 'deal' AND parent_id IS NOT NULL;
         DELETE FROM categories WHERE kind = 'deal';
         DROP TABLE IF EXISTS deals;",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO data_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING").bind(NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

/// 已部署的论坛把小伴壁纸、小伴游戏两个板块放开给所有人发帖（含新人），分享作品资源包用。
/// 只在最低发帖等级还是原来的守护者时才改，说明文字也只在没被改过时才换，做完记进 `data_migrations`，之后后台怎么调都不再动。
async fn apply_open_share_boards_migration(pool: &PgPool) -> Result<(), AppError> {
    const NAME: &str = "xiaoban-share-boards-open-v1";
    let done: Option<(String,)> =
        sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(NAME).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    for (slug, old, new) in [
        ("xiaoban-wallpaper", "小伴壁纸的下载、更新和使用技巧（官方发布，欢迎回复反馈）", "小伴壁纸的下载、更新和使用技巧，也欢迎分享你自己做的壁纸、特效、组件资源包"),
        ("xiaoban-game", "小伴游戏的新玩法、攻略和更新（官方发布，欢迎回复反馈）", "小伴游戏的新玩法、攻略和更新，也欢迎分享你自己做的游戏资源包"),
    ] {
        sqlx::query("UPDATE categories SET min_post_level = 0 WHERE slug = $1 AND min_post_level = 3 AND NOT admin_only")
            .bind(slug)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE categories SET description = $3 WHERE slug = $1 AND description = $2")
            .bind(slug)
            .bind(old)
            .bind(new)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO data_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING").bind(NAME).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

/// 一次板块调整：把某个一级分类下的二级板块换成种子里的新板块，旧板块里的帖子并入指定的新板块。
struct BoardMigration {
    /// 记录名，做过一次就写进 `data_migrations`，不再重复。
    name: &'static str,
    /// 一级分类的 slug。
    parent: &'static str,
    /// 这个一级分类在库里还没有时，按种子新增它（插在种子里前一个一级分类的整棵子树后面）。
    create_parent: bool,
    /// 这次要写入（新增或按种子覆盖名称、说明、标签）的板块 slug，都是 `parent` 下的二级板块。其余板块不动。
    boards: &'static [&'static str],
    /// (旧板块 slug, 帖子并入的新板块 slug)
    moves: &'static [(&'static str, &'static str)],
}

const BOARD_MIGRATIONS: &[BoardMigration] = &[
    BoardMigration { name: "share-help-v1", parent: "share", create_parent: false, boards: &["share-help"], moves: &[] },
    // 资源分享：开源项目、工具软件、提示词·工作流、自制作品 → 精品软件 / 电影资源 / 音乐资源 / 学习资料
    BoardMigration {
        name: "share-boards-v1",
        parent: "share",
        create_parent: false,
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
        create_parent: false,
        boards: &["tech-llm", "tech-productivity", "tech-hardware"],
        moves: &[
            ("tech-ai-app", "tech-productivity"),
            ("tech-dev", "tech-productivity"),
            ("tech-ops", "tech-llm"),
            ("tech-models", "tech-llm"),
        ],
    },
    // 技术交流：新增"AI 创意"，放在最后
    BoardMigration { name: "tech-creative-v1", parent: "tech", create_parent: false, boards: &["tech-creative"], moves: &[] },
    // 随便唠唠：新增"亏了吗"，放在"下班了"后面
    BoardMigration { name: "casual-loss-v1", parent: "casual", create_parent: false, boards: &["casual-loss"], moves: &[] },
    // 随便唠唠：新增"梦想和人生"，放在"亏了吗"后面
    BoardMigration { name: "casual-dream-v1", parent: "casual", create_parent: false, boards: &["casual-dream"], moves: &[] },
    // 随便唠唠：手机端休闲栏的筛选需要 片单乐评 / 哲思 / 互助，放在“吃喝玩乐”后面
    BoardMigration { name: "casual-boards-v2", parent: "casual", create_parent: false, boards: &["casual-media", "casual-think", "casual-help"], moves: &[] },
    // 小伴产品：新增一级分类和 小伴壁纸 / 小伴游戏 / 小伴App，放在福利羊毛之后、随便唠唠之前
    BoardMigration {
        name: "xiaoban-boards-v1",
        parent: "xiaoban",
        create_parent: true,
        boards: &["xiaoban-wallpaper", "xiaoban-game", "xiaoban-app"],
        moves: &[],
    },
];

/// 新增一个一级分类：库里已经有同 slug 的（比如管理员自己建过）就不动。
/// 插在种子里前一个一级分类的整棵子树后面，其后的分类整体顺延一位；前一个一级分类不在库里就追加到最后。
async fn create_root_board(conn: &mut sqlx::PgConnection, slug: &str) -> Result<(), AppError> {
    let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM categories WHERE slug = $1").bind(slug).fetch_optional(&mut *conn).await?;
    if exists.is_some() {
        return Ok(());
    }
    let Some(idx) = CATEGORIES.iter().position(|c| c.0 == slug && c.4.is_none()) else { return Ok(()) };
    let (_, name, desc, kind, _, level, home, points, admin_only) = CATEGORIES[idx];
    let pred_root = CATEGORIES[..idx].iter().rev().find(|c| c.4.is_none()).map(|c| c.0);

    let pred_end: Option<i32> = match pred_root {
        Some(pred) => {
            let (end,): (i32,) = sqlx::query_as(
                "SELECT COALESCE(MAX(c.sort_order), -1) FROM categories c
                 WHERE c.slug = $1 OR c.parent_id = (SELECT id FROM categories WHERE slug = $1)",
            )
            .bind(pred)
            .fetch_one(&mut *conn)
            .await?;
            (end >= 0).then_some(end)
        }
        None => None,
    };
    let order = match pred_end {
        Some(end) => {
            sqlx::query("UPDATE categories SET sort_order = sort_order + 1 WHERE sort_order > $1").bind(end).execute(&mut *conn).await?;
            end + 1
        }
        None => {
            let (max,): (i32,) = sqlx::query_as("SELECT COALESCE(MAX(sort_order), -1) FROM categories").fetch_one(&mut *conn).await?;
            max + 1
        }
    };
    sqlx::query(
        "INSERT INTO categories (parent_id, slug, name, description, kind, sort_order, min_post_level, on_home, award_points, admin_only, preset_tags)
         VALUES (NULL, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(slug)
    .bind(name)
    .bind(desc)
    .bind(kind)
    .bind(order)
    .bind(level)
    .bind(home)
    .bind(points)
    .bind(admin_only)
    .bind(preset_tags_for(slug))
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// 已部署的库执行一次板块调整：`boards` 里的板块用种子里的名称、说明和内置标签写入（已有的同 slug 板块被更新），
/// 旧板块的帖子并入指定的新板块后删除旧板块，一级分类的说明也一起更新。新板块插在种子里前一个板块的后面，已有分类的排序不动。
///
/// 做完记进 `data_migrations`，之后不再执行，所以管理员后来改过、甚至重新建过同名 slug 的分类都不会被动到。
/// 一次性更新已部署论坛中的电影板块标签，之后不覆盖管理员修改。
async fn apply_movie_tag_migration(pool: &PgPool) -> Result<(), AppError> {
    const NAME: &str = "share-movies-tags-4k-tv-v1";
    let done: Option<(String,)> =
        sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(NAME).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;
    let changed = sqlx::query(
        "UPDATE categories SET preset_tags = preset_tags || ARRAY(            SELECT tag FROM unnest(ARRAY['4k', '美剧']::TEXT[]) AS new_tags(tag)            WHERE NOT (tag = ANY(categories.preset_tags))         ) WHERE slug = 'share-movies'",
    )
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if changed > 0 {
        sqlx::query("INSERT INTO data_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(NAME)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}
async fn apply_board_migration(pool: &PgPool, m: &BoardMigration) -> Result<(), AppError> {
    let done: Option<(String,)> =
        sqlx::query_as("SELECT name FROM data_migrations WHERE name = $1").bind(m.name).fetch_optional(pool).await?;
    if done.is_some() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;
    if m.create_parent {
        create_root_board(&mut tx, m.parent).await?;
    }
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
        // 调整已有一级分类下的板块时顺便刷新它的说明；新增一级分类的迁移不刷新（新建时已经是种子里的说明，
        // 管理员事先自己建过同 slug 分类的话，说明也不该被覆盖）。
        if !m.create_parent {
            if let Some(desc) = CATEGORIES.iter().find(|c| c.0 == m.parent).map(|c| c.2) {
                sqlx::query("UPDATE categories SET description = $1 WHERE id = $2").bind(desc).bind(parent_id).execute(&mut *tx).await?;
            }
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
        assert_eq!(children("share"), ["精品软件", "电影资源", "音乐资源", "学习资料", "资源求助"]);
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
    fn xiaoban_is_a_root_between_share_and_casual_with_open_share_boards() {
        let roots: Vec<&str> = CATEGORIES.iter().filter(|c| c.4.is_none()).map(|c| c.0).collect();
        assert_eq!(roots, ["tech", "share", "xiaoban", "casual", "site"]);
        assert_eq!(children("xiaoban"), ["小伴壁纸", "小伴游戏", "小伴App"]);
        // 官方板块只有守护者能发帖，但不是 admin_only（admin_only 会拦住所有人，包括守护者）。
        // 小伴壁纸、小伴游戏用来分享用户自己的资源包，所有人（含新人）都能发。
        for c in CATEGORIES.iter().filter(|c| c.0 == "xiaoban" || c.4 == Some("xiaoban")) {
            let open = c.0 == "xiaoban-wallpaper" || c.0 == "xiaoban-game";
            assert_eq!(c.5, if open { 0 } else { 3 }, "{} 的最低发帖等级不对", c.0);
            assert!(!c.8, "{} 不应该设成 admin_only", c.0);
        }
        // 创建一级分类的迁移必须带上它在种子里的全部二级板块。
        let m = BOARD_MIGRATIONS.iter().find(|m| m.parent == "xiaoban").expect("缺少小伴产品的迁移");
        assert!(m.create_parent);
        assert_eq!(m.boards, ["xiaoban-wallpaper", "xiaoban-game", "xiaoban-app"]);
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



/// 社区内容层：内容包、版本、应用计数、方案、项目、周刊与审核队列。
const CONTENT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS packs (
    id            BIGSERIAL PRIMARY KEY,
    topic_id      BIGINT REFERENCES topics(id) ON DELETE SET NULL,
    kind          TEXT NOT NULL,
    slug          TEXT NOT NULL UNIQUE,
    title         TEXT NOT NULL,
    summary       TEXT NOT NULL DEFAULT '',
    tags          TEXT[] NOT NULL DEFAULT '{}',
    author_uid    UUID REFERENCES users(uid),
    trust_level   SMALLINT NOT NULL DEFAULT 2 CHECK (trust_level BETWEEN 0 AND 2),
    status        TEXT NOT NULL DEFAULT 'published' CHECK (status IN ('pending', 'published', 'rejected', 'removed')),
    current_version INT NOT NULL DEFAULT 1,
    cover_key     TEXT,
    apply_count   BIGINT NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS packs_list_idx ON packs (status, kind, updated_at DESC);
CREATE INDEX IF NOT EXISTS packs_hot_idx ON packs (status, apply_count DESC);
CREATE INDEX IF NOT EXISTS packs_author_idx ON packs (author_uid, created_at DESC);

CREATE TABLE IF NOT EXISTS pack_versions (
    pack_id       BIGINT NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
    version       INT NOT NULL,
    label         TEXT NOT NULL,
    sha256        TEXT NOT NULL CHECK (length(sha256) = 64),
    size          BIGINT NOT NULL CHECK (size > 0),
    manifest      JSONB NOT NULL,
    storage       TEXT NOT NULL DEFAULT 'resource',
    file_key      TEXT NOT NULL,
    mirrors       JSONB NOT NULL DEFAULT '[]',
    changelog     TEXT NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (pack_id, version)
);

-- 直传凭证签发记录：资源机回调确认文件落盘并校验过哈希后才算 verified，发布内容包只认已确认的上传。
CREATE TABLE IF NOT EXISTS pack_uploads (
    file_key      TEXT PRIMARY KEY,
    uid           UUID NOT NULL REFERENCES users(uid),
    kind          TEXT NOT NULL,
    size          BIGINT NOT NULL,
    sha256        TEXT NOT NULL,
    manifest      JSONB,
    verified      BOOLEAN NOT NULL DEFAULT false,
    used          BOOLEAN NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS pack_applies (
    pack_id       BIGINT NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
    day           DATE NOT NULL,
    count         INT NOT NULL DEFAULT 0,
    PRIMARY KEY (pack_id, day)
);

CREATE TABLE IF NOT EXISTS schemes (
    id            BIGSERIAL PRIMARY KEY,
    author_uid    UUID NOT NULL REFERENCES users(uid),
    title         TEXT NOT NULL,
    summary       TEXT NOT NULL DEFAULT '',
    items         JSONB NOT NULL DEFAULT '[]',
    status        TEXT NOT NULL DEFAULT 'published' CHECK (status IN ('published', 'removed')),
    apply_count   BIGINT NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS schemes_list_idx ON schemes (status, created_at DESC);

CREATE TABLE IF NOT EXISTS projects (
    slug          TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    summary       TEXT NOT NULL DEFAULT '',
    body_md       TEXT NOT NULL DEFAULT '',
    body_html     TEXT NOT NULL DEFAULT '',
    links         JSONB NOT NULL DEFAULT '[]',
    status        TEXT NOT NULL DEFAULT 'published' CHECK (status IN ('draft', 'published')),
    sort_order    INT NOT NULL DEFAULT 0,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS weekly_issues (
    week          TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    summary       TEXT NOT NULL DEFAULT '',
    items         JSONB NOT NULL DEFAULT '[]',
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published')),
    published_at  TIMESTAMPTZ,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS moderation_queue (
    id            BIGSERIAL PRIMARY KEY,
    target_type   TEXT NOT NULL,
    target_id     BIGINT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    note          TEXT NOT NULL DEFAULT '',
    handled_by    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    handled_at    TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS moderation_pending_idx ON moderation_queue (target_type, target_id) WHERE status = 'pending';
"#;

#[cfg(test)]
mod removal_tests {
    use uuid::Uuid;

    /// 旧库里有羊毛分类、帖子、回复、点赞、收藏、举报和 `deals` 表：迁移后羊毛相关全部消失，其他内容不动，且只执行一次。
    #[tokio::test]
    async fn deals_are_removed_with_everything_attached() -> Result<(), Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("TEST_DATABASE_URL") else { return Ok(()) };
        let admin = super::connect(&url, None).await?;
        let schema = format!("deals_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await?;
        let pool = super::connect(&url, Some(&schema)).await?;
        super::migrate(&pool).await?;
        sqlx::raw_sql("CREATE TABLE deals (topic_id BIGINT PRIMARY KEY REFERENCES topics(id) ON DELETE CASCADE, value_text TEXT NOT NULL DEFAULT '')")
            .execute(&pool)
            .await?;
        let uid = Uuid::new_v4();
        sqlx::query("INSERT INTO users (uid, nickname) VALUES ($1, 'u')").bind(uid).execute(&pool).await?;
        let cat = |slug: &'static str, kind: &'static str, parent: Option<i32>| {
            let pool = pool.clone();
            async move {
                sqlx::query_as::<_, (i32,)>("INSERT INTO categories (parent_id, slug, name, kind) VALUES ($1, $2, $2, $3) RETURNING id")
                    .bind(parent)
                    .bind(slug)
                    .bind(kind)
                    .fetch_one(&pool)
                    .await
                    .map(|r| r.0)
            }
        };
        let deal_root = cat("deals", "deal", None).await?;
        let deal_child = cat("deals-ai", "deal", Some(deal_root)).await?;
        let normal = cat("tech", "normal", None).await?;
        let mut topics = Vec::new();
        for (c, title) in [(deal_child, "羊毛帖"), (normal, "普通帖")] {
            let (id,): (i64,) = sqlx::query_as("INSERT INTO topics (category_id, author_uid, title, body_md, body_html) VALUES ($1, $2, $3, 'x', 'x') RETURNING id")
                .bind(c).bind(uid).bind(title).fetch_one(&pool).await?;
            let (post,): (i64,) = sqlx::query_as("INSERT INTO posts (topic_id, author_uid, floor, body_md, body_html) VALUES ($1, $2, 1, 'r', 'r') RETURNING id")
                .bind(id).bind(uid).fetch_one(&pool).await?;
            sqlx::query("INSERT INTO bookmarks (uid, topic_id) VALUES ($1, $2)").bind(uid).bind(id).execute(&pool).await?;
            sqlx::query("INSERT INTO reports (target_type, target_id, reporter_uid, reason) VALUES ('post', $1, $2, 'spam')").bind(post).bind(uid).execute(&pool).await?;
            topics.push(id);
        }
        sqlx::query("INSERT INTO deals (topic_id, value_text) VALUES ($1, '免费')").bind(topics[0]).execute(&pool).await?;

        super::remove_deals(&pool).await?;
        super::remove_deals(&pool).await?;

        let count = |sql: &'static str| {
            let pool = pool.clone();
            async move { sqlx::query_as::<_, (i64,)>(sql).fetch_one(&pool).await.map(|r| r.0) }
        };
        assert_eq!(count("SELECT COUNT(*) FROM topics").await?, 1);
        assert_eq!(count("SELECT COUNT(*) FROM posts").await?, 1);
        assert_eq!(count("SELECT COUNT(*) FROM bookmarks").await?, 1);
        assert_eq!(count("SELECT COUNT(*) FROM reports").await?, 1);
        assert_eq!(count("SELECT COUNT(*) FROM categories WHERE kind = 'deal'").await?, 0);
        assert_eq!(count("SELECT COUNT(*) FROM categories").await?, 1);
        assert_eq!(count("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = current_schema() AND table_name = 'deals'").await?, 0);
        assert_eq!(count("SELECT COUNT(*) FROM data_migrations WHERE name = 'remove-deals-v1'").await?, 1);

        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&admin).await?;
        Ok(())
    }
}

/// 开放用户发布前的合规项：敏感词表。
const COMPLIANCE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sensitive_words (
    id         SERIAL PRIMARY KEY,
    word       TEXT NOT NULL UNIQUE,
    action     TEXT NOT NULL CHECK (action IN ('block', 'review')),
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 屏蔽用户：屏蔽者在列表、搜索、回复里不再看到对方的内容，也不会收到对方带来的通知。
CREATE TABLE IF NOT EXISTS user_blocks (
    blocker    UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    blocked    UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker, blocked),
    CHECK (blocker <> blocked)
);
CREATE INDEX IF NOT EXISTS user_blocks_blocked_idx ON user_blocks (blocked);

-- 站内通知：标题与摘要是写入时的快照。
CREATE TABLE IF NOT EXISTS notifications (
    id         BIGSERIAL PRIMARY KEY,
    uid        UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    kind       TEXT NOT NULL,
    actor_uid  UUID REFERENCES users(uid) ON DELETE SET NULL,
    topic_id   BIGINT REFERENCES topics(id) ON DELETE CASCADE,
    post_id    BIGINT,
    pack_id    BIGINT REFERENCES packs(id) ON DELETE CASCADE,
    title      TEXT NOT NULL DEFAULT '',
    excerpt    TEXT NOT NULL DEFAULT '',
    read_at    TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS notifications_user_idx ON notifications (uid, id DESC);
CREATE INDEX IF NOT EXISTS notifications_unread_idx ON notifications (uid) WHERE read_at IS NULL;

-- 匿名埋点：某匿名设备某天某事件几次。不记账号、内容、搜索词，不保存行为流。
CREATE TABLE IF NOT EXISTS events_daily (
    day    DATE NOT NULL,
    name   TEXT NOT NULL,
    dim    TEXT NOT NULL DEFAULT '-',
    device TEXT NOT NULL,
    count  INT NOT NULL DEFAULT 1,
    PRIMARY KEY (day, name, dim, device)
);
CREATE INDEX IF NOT EXISTS events_daily_device_idx ON events_daily (device, day);
"#;

/// 手机端社区新增的内容：后台手录的发现页条目（日报、榜单、导航、工具等）、投票、话题配图。
const DISCOVER_SCHEMA: &str = r#"
-- 发现页条目：按 kind 区分，data 的形状由 discover.rs 里各类型的结构体校验。
CREATE TABLE IF NOT EXISTS discover_items (
    kind         TEXT NOT NULL,
    key          TEXT NOT NULL,
    sort_order   INT NOT NULL DEFAULT 0,
    status       TEXT NOT NULL DEFAULT 'published' CHECK (status IN ('draft', 'published')),
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    data         JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (kind, key)
);
CREATE INDEX IF NOT EXISTS discover_items_order_idx ON discover_items (kind, status, sort_order, published_at DESC);

-- 编辑副本与线上条目分离；审核绑定 revision，修改会撤销定时发布。
CREATE SEQUENCE IF NOT EXISTS discover_draft_revision_seq;
CREATE TABLE IF NOT EXISTS discover_drafts (
    kind TEXT NOT NULL,
    key TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    data JSONB NOT NULL,
    published_at TIMESTAMPTZ,
    source_name TEXT NOT NULL DEFAULT '',
    source_url TEXT NOT NULL DEFAULT '',
    revision BIGINT NOT NULL DEFAULT nextval('discover_draft_revision_seq'),
    base_updated_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'scheduled', 'published')),
    publish_after TIMESTAMPTZ,
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ,
    last_error TEXT,
    updated_by TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (kind, key),
    CHECK ((status = 'scheduled') = (publish_after IS NOT NULL)),
    CHECK (status = 'draft' OR (reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS discover_drafts_due_idx ON discover_drafts (publish_after) WHERE status = 'scheduled';

-- 本地采集凭据：仅允许读写指定类型的草稿，不具有发布或管理权限。
CREATE TABLE IF NOT EXISTS discover_tokens (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    kinds TEXT[] NOT NULL CHECK (cardinality(kinds) > 0),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 列表卡片用的配图：发帖时从正文里取前 3 张站内图片。
ALTER TABLE topics ADD COLUMN IF NOT EXISTS images TEXT[] NOT NULL DEFAULT '{}';

CREATE TABLE IF NOT EXISTS polls (
    topic_id   BIGINT PRIMARY KEY REFERENCES topics(id) ON DELETE CASCADE,
    options    TEXT[] NOT NULL CHECK (cardinality(options) BETWEEN 2 AND 6),
    closes_at  TIMESTAMPTZ
);
CREATE TABLE IF NOT EXISTS poll_votes (
    topic_id   BIGINT NOT NULL REFERENCES polls(topic_id) ON DELETE CASCADE,
    uid        UUID NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    option_idx SMALLINT NOT NULL CHECK (option_idx >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (topic_id, uid)
);
"#;

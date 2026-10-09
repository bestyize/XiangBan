use crate::{
    auth::hash_password,
    config::AppConfig,
    error::AppError,
};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::info;

pub async fn init_db(config: &AppConfig) -> Result<PgPool, AppError> {
    info!("Connecting to PostgreSQL: {}", config.database_url);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to PostgreSQL: {:?}", e);
            AppError::Database(e)
        })?;

    create_tables(&pool).await?;
    seed_data(&pool, config).await?;

    Ok(pool)
}

async fn create_tables(pool: &PgPool) -> Result<(), AppError> {
    // 1. 分类表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS appstore_categories (
            id SERIAL PRIMARY KEY,
            slug VARCHAR(64) UNIQUE NOT NULL,
            name VARCHAR(64) NOT NULL,
            icon VARCHAR(64) NOT NULL,
            description TEXT NOT NULL,
            sort_order INT NOT NULL DEFAULT 0
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 2. 应用主体表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS appstore_apps (
            id SERIAL PRIMARY KEY,
            slug VARCHAR(128) UNIQUE NOT NULL,
            name VARCHAR(128) NOT NULL,
            icon_url TEXT NOT NULL,
            category_id INT NOT NULL REFERENCES appstore_categories(id) ON DELETE CASCADE,
            summary TEXT NOT NULL,
            comment TEXT NOT NULL,
            referrer VARCHAR(64) NOT NULL DEFAULT '小伴精选',
            referrer_avatar TEXT,
            pricing_type VARCHAR(32) NOT NULL DEFAULT 'free',
            platforms JSONB NOT NULL DEFAULT '["mac"]'::jsonb,
            badges JSONB NOT NULL DEFAULT '["safe"]'::jsonb,
            status VARCHAR(32) NOT NULL DEFAULT 'published',
            is_featured BOOLEAN NOT NULL DEFAULT false,
            sort_order INT NOT NULL DEFAULT 0,
            download_url TEXT NOT NULL,
            download_alt_url TEXT,
            official_url TEXT,
            appstore_url TEXT,
            github_url TEXT,
            github_stars INT DEFAULT 0,
            brew_cask VARCHAR(128),
            winget_id VARCHAR(128),
            mac_download_url TEXT,
            win_download_url TEXT,
            linux_download_url TEXT,
            file_size VARCHAR(64),
            release_notes TEXT,
            auto_sync_source VARCHAR(256),
            last_synced_at TIMESTAMPTZ,
            pros JSONB NOT NULL DEFAULT '[]'::jsonb,
            cons JSONB NOT NULL DEFAULT '[]'::jsonb,
            alternatives JSONB NOT NULL DEFAULT '[]'::jsonb,
            version VARCHAR(64),
            downloads_count INT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 确保已存在表增量补充字段
    let _ = sqlx::query("ALTER TABLE appstore_apps ADD COLUMN IF NOT EXISTS mac_download_url TEXT;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE appstore_apps ADD COLUMN IF NOT EXISTS win_download_url TEXT;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE appstore_apps ADD COLUMN IF NOT EXISTS linux_download_url TEXT;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE appstore_apps ADD COLUMN IF NOT EXISTS file_size VARCHAR(64);").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE appstore_apps ADD COLUMN IF NOT EXISTS release_notes TEXT;").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE appstore_apps ADD COLUMN IF NOT EXISTS auto_sync_source VARCHAR(256);").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE appstore_apps ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMPTZ;").execute(pool).await;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_appstore_apps_category ON appstore_apps(category_id);")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_appstore_apps_status ON appstore_apps(status);")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_appstore_apps_sort ON appstore_apps(sort_order DESC, created_at DESC);")
        .execute(pool)
        .await?;

    // 3. 管理员表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS appstore_admins (
            id SERIAL PRIMARY KEY,
            username VARCHAR(64) UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_data(pool: &PgPool, config: &AppConfig) -> Result<(), AppError> {
    // 检查管理员
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM appstore_admins")
        .fetch_one(pool)
        .await?;

    if admin_count == 0 {
        let password_hash = hash_password(&config.default_admin_pass)?;
        sqlx::query(
            "INSERT INTO appstore_admins (username, password_hash) VALUES ($1, $2)",
        )
        .bind(&config.default_admin_user)
        .bind(password_hash)
        .execute(pool)
        .await?;
        info!("Default admin created: {}", config.default_admin_user);
    }

    // 检查分类
    let cat_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM appstore_categories")
        .fetch_one(pool)
        .await?;

    if cat_count == 0 {
        info!("Seeding 8 innovative categories...");
        let categories = vec![
            ("ai-agents", "AI 新纪元", "Bot", "大模型与智能体时代的生产力底座与高质感客户端", 100),
            ("modern-classics", "经典重塑", "Sparkles", "向工业巨兽致敬，现代技术重构、极度轻量的免全家桶工具", 90),
            ("github-gems", "开源之光", "Github", "GitHub 千锤百炼、代码完全透明且无商业绑架的独立开发者瑰宝", 80),
            ("day-one", "开荒必备", "Rocket", "新电脑开机第 1 天必装的基建底座，好用度提升数倍", 70),
            ("developer", "极客工坊", "Terminal", "面向程序员、黑客与系统玩家的精雕细琢工具", 60),
            ("minimalism", "数字减负", "Feather", "单点极致、无弹窗、不偷扫硬盘、内存以 MB 计的清爽平替", 50),
            ("focus-flow", "心流创作", "PenTool", "沉浸式知识沉淀、灵感捕捉、思维图谱与防分心利器", 40),
            ("life-aesthetics", "生活美学", "Palette", "桌面美学与治愈系数字陪伴，高颜值动态壁纸与精致小品", 30),
        ];

        for (slug, name, icon, desc, sort) in categories {
            sqlx::query(
                "INSERT INTO appstore_categories (slug, name, icon, description, sort_order) VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(slug)
            .bind(name)
            .bind(icon)
            .bind(desc)
            .bind(sort)
            .execute(pool)
            .await?;
        }
    }

    // 检查预置应用
    let app_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM appstore_apps")
        .fetch_one(pool)
        .await?;

    if app_count == 0 {
        info!("Seeding curated high-quality apps...");

        // 获取分类映射
        let rows = sqlx::query("SELECT id, slug FROM appstore_categories")
            .fetch_all(pool)
            .await?;

        let mut cat_map = std::collections::HashMap::new();
        for row in rows {
            let id: i32 = row.get("id");
            let slug: String = row.get("slug");
            cat_map.insert(slug, id);
        }

        // 定义种子软件
        insert_seed_apps(pool, &cat_map).await?;
    }

    Ok(())
}

async fn insert_seed_apps(pool: &PgPool, cat_map: &std::collections::HashMap<String, i32>) -> Result<(), AppError> {
    let seeds = vec![
        // 1. AI 新纪元
        (
            "claude-desktop",
            "Claude Desktop",
            "https://cdn.oaistatic.com/_next/static/media/claude-logo.svg", // fallback icon
            "ai-agents",
            "Anthropic 官方桌面客户端，支持工件实时预览与深度思考",
            "官方桌面客户端，完美支持 Artifacts 工件即时渲染、思考过程展开与多项目会话隔离，堪称大模型交互美学的行业标杆。",
            "小伴精选编辑部",
            "free",
            serde_json::json!(["mac", "windows"]),
            serde_json::json!(["safe", "native", "ai", "clean"]),
            true,
            100,
            "https://claude.ai/download",
            Some("https://claude.ai/"),
            Some("https://claude.ai/download"),
            None,
            None,
            0,
            Some("claude"),
            None,
            serde_json::json!(["原生流畅", "支持 Artifacts 预览", "深度思考可视化"]),
            serde_json::json!(["部分地区网络需要优化配置"]),
            serde_json::json!(["ChatGPT 网页端"]),
            "1.2.0"
        ),
        (
            "codex-editor",
            "CodeX",
            "https://raw.githubusercontent.com/walkxcode/dashboard-icons/main/svg/vscode.svg",
            "ai-agents",
            "AI 驱动的新一代工程化智能编辑器，重塑代码心流",
            "深度结合大模型推理能力的下一代智能编程利器，不仅是代码补全，更是能够理解整个仓库架构的协同工程师。",
            "极客合伙人",
            "freemium",
            serde_json::json!(["mac", "windows", "linux"]),
            serde_json::json!(["safe", "ai", "recommended"]),
            true,
            95,
            "https://cursor.com/",
            Some("https://cursor.com/"),
            Some("https://cursor.com/"),
            None,
            None,
            0,
            Some("cursor"),
            Some("Anysphere.Cursor"),
            serde_json::json!(["项目级全库代码索引", "多行预测丝滑", "Composer 架构协同"]),
            serde_json::json!(["高级模型需按月订阅"]),
            serde_json::json!(["传统 VS Code", "Copilot 插件"]),
            "0.42.0"
        ),
        (
            "ccswitch",
            "CCSwitch 桌面端",
            "https://raw.githubusercontent.com/walkxcode/dashboard-icons/main/svg/switch.svg",
            "ai-agents",
            "大模型与 API 多路由无缝调度切换工具，极客的智能体控制台",
            "专为开发者和重度 AI 玩家设计的调度利器，轻松管理 Anthropic、OpenAI 及中转路由，多端配置秒级同步。",
            "小伴精选编辑部",
            "free",
            serde_json::json!(["mac", "windows"]),
            serde_json::json!(["safe", "native", "clean"]),
            true,
            90,
            "https://github.com/",
            Some("https://github.com/"),
            None,
            None,
            Some("https://github.com/"),
            1850,
            None,
            None,
            serde_json::json!(["零配置多渠道切换", "低延迟链路测试", "透明代理支持"]),
            serde_json::json!(["需自备 API Key"]),
            serde_json::json!(["手动改配置文件"]),
            "1.0.8"
        ),
        (
            "ollama",
            "Ollama",
            "https://ollama.com/public/ollama.png",
            "ai-agents",
            "本地大模型运行底座，命令行秒级拉取与运行",
            "终端一行命令即可秒跑本地千亿参数大模型，隐私数据主权在手，无缝暴露兼容 OpenAI 的本地 API 端点。",
            "小伴精选编辑部",
            "open_source",
            serde_json::json!(["mac", "windows", "linux"]),
            serde_json::json!(["open_source", "native", "local_first", "safe"]),
            false,
            85,
            "https://ollama.com/download",
            Some("https://ollama.com/"),
            Some("https://ollama.com/download"),
            None,
            Some("https://github.com/ollama/ollama"),
            98000,
            Some("ollama"),
            Some("Ollama.Ollama"),
            serde_json::json!(["完全本地离线运行", "模型一键拉取与管理", "生态兼容极佳"]),
            serde_json::json!(["依赖本机内存与 GPU 显存规模"]),
            serde_json::json!(["云端收费大模型"]),
            "0.5.8"
        ),

        // 2. 经典重塑
        (
            "photocraft",
            "PhotoCraft",
            "https://raw.githubusercontent.com/walkxcode/dashboard-icons/main/svg/photoshop.svg",
            "modern-classics",
            "向经典图像处理致敬的现代极轻量工作台，秒开秒编辑",
            "告别动辄几十 G、全家桶绑架与复杂激活。PhotoCraft 重塑了最纯粹的图层、滤镜与蒙版核心，单文件秒开，不占系统后台。",
            "小伴精选编辑部",
            "free",
            serde_json::json!(["mac", "windows", "web"]),
            serde_json::json!(["safe", "clean", "native"]),
            true,
            95,
            "https://www.photopea.com/",
            Some("https://www.photopea.com/"),
            None,
            None,
            None,
            0,
            None,
            None,
            serde_json::json!(["启动秒开零负担", "PSD 完美兼容", "无需臃肿全家桶安装"]),
            serde_json::json!(["超大型印刷级喷绘文件稍逊专业套件"]),
            serde_json::json!(["Photoshop 几十G安装包", "捆绑全家桶"]),
            "2.4.0"
        ),
        (
            "patchy",
            "Patchy",
            "https://raw.githubusercontent.com/walkxcode/dashboard-icons/main/svg/audacity.svg",
            "modern-classics",
            "现代优雅的极简音轨合成与修剪利器，剥离冗余插件",
            "剥离传统 DAW 复杂的轨道包袱，提供所见即所得的波形精准修剪、混音与降噪，专为播客创作者和视频剪辑师而生。",
            "极客合伙人",
            "free",
            serde_json::json!(["mac", "windows"]),
            serde_json::json!(["safe", "clean", "native"]),
            false,
            88,
            "https://github.com/",
            None,
            None,
            None,
            Some("https://github.com/"),
            3200,
            None,
            None,
            serde_json::json!(["纯粹专注音频修补", "快捷键极其流畅", "导出速度极快"]),
            serde_json::json!(["不适用于交响乐复杂编曲"]),
            serde_json::json!(["Audacity 老旧界面", "臃肿商业DAW"]),
            "1.1.2"
        ),
        (
            "typora",
            "Typora",
            "https://typora.io/img/favicon-64.png",
            "modern-classics",
            "所见即所得 Markdown 排版的艺术级标杆",
            "告别割裂的双栏排版与枯燥语法，实时渲染打字体验极其舒适，精美主题支持导出 PDF、Word，写作心流的最佳伴侣。",
            "小伴精选编辑部",
            "paid",
            serde_json::json!(["mac", "windows", "linux"]),
            serde_json::json!(["safe", "native", "recommended"]),
            true,
            92,
            "https://typora.io/#download",
            Some("https://typora.io/"),
            Some("https://typora.io/#download"),
            None,
            None,
            0,
            Some("typora"),
            Some("Typora.Typora"),
            serde_json::json!(["极致无缝实时渲染", "大纲与目录跳转丝滑", "导出排版无可挑剔"]),
            serde_json::json!(["买断制收费（买断3台设备）"]),
            serde_json::json!(["Word", "传统双栏分屏编辑器"]),
            "1.9.5"
        ),

        // 3. 开源之光
        (
            "iina",
            "IINA",
            "https://www.iina.io/images/iina-icon.png",
            "github-gems",
            "macOS 现代播放器天花板，Swift 原生极致设计",
            "采用 Swift 编写的现代媒体播放器，支持 HDR、暗黑模式、画中画与 Touch Bar，纯粹优雅，毫无流氓广告，格式通吃。",
            "小伴精选编辑部",
            "open_source",
            serde_json::json!(["mac"]),
            serde_json::json!(["open_source", "native", "safe", "clean"]),
            true,
            96,
            "https://www.iina.io/download/",
            Some("https://www.iina.io/"),
            Some("https://www.iina.io/download/"),
            None,
            Some("https://github.com/iina/iina"),
            36800,
            Some("iina"),
            None,
            serde_json::json!(["Swift 原生现代界面", "全面支持字幕在线匹配", "解码几乎一切格式"]),
            serde_json::json!(["仅支持 macOS 系统"]),
            serde_json::json!(["带弹窗广告的流氓影音播放器"]),
            "1.3.5"
        ),
        (
            "maccy",
            "Maccy",
            "https://maccy.app/img/icon.png",
            "github-gems",
            "仅几兆的纯原生剪贴板管理器，毫秒级呼出",
            "只有 10MB 左右的轻巧身材，毫秒级快捷键唤醒，历史记录直观搜索，绝不把任何剪贴板私密数据上传云端，开源放心。",
            "极客合伙人",
            "open_source",
            serde_json::json!(["mac"]),
            serde_json::json!(["open_source", "native", "local_first", "safe"]),
            true,
            90,
            "https://maccy.app/",
            Some("https://maccy.app/"),
            Some("https://maccy.app/"),
            Some("https://apps.apple.com/app/maccy/id1527619437"),
            Some("https://github.com/p0deje/Maccy"),
            13500,
            Some("maccy"),
            None,
            serde_json::json!(["纯原生 Swift 极速唤起", "无广告无常驻庞大内存", "搜索键盘流体验极佳"]),
            serde_json::json!(["多设备间不自动云同步（本地优先）"]),
            serde_json::json!(["常驻后台占用内存几百兆的云剪贴板"]),
            "2.0.2"
        ),
        (
            "localsend",
            "LocalSend",
            "https://localsend.org/icon-512.png",
            "github-gems",
            "全平台免配置点对点互传，真正的跨平台 AirDrop 平替",
            "开源且完全免费的局域网文件传输工具。无需外网中转、无需注册任何账号，苹果、安卓、Windows、Linux 秒级发现与互传。",
            "小伴精选编辑部",
            "open_source",
            serde_json::json!(["mac", "windows", "linux", "android", "ios"]),
            serde_json::json!(["open_source", "local_first", "safe", "clean"]),
            true,
            94,
            "https://localsend.org/download",
            Some("https://localsend.org/"),
            Some("https://localsend.org/download"),
            Some("https://apps.apple.com/app/localsend/id1661733229"),
            Some("https://github.com/localsend/localsend"),
            48000,
            Some("localsend"),
            Some("LocalSend.LocalSend"),
            serde_json::json!(["全平台完美互联互通", "不经第三方服务器，极度安全", "速度跑满局域网千兆"]),
            serde_json::json!(["双方需处于同一局域网 WiFi"]),
            serde_json::json!(["微信/QQ文件传输助手", "百度网盘中转"]),
            "1.14.0"
        ),

        // 4. 开荒必备
        (
            "raycast",
            "Raycast",
            "https://www.raycast.com/favicon-production.png",
            "day-one",
            "重新定义 macOS 效率底座，工作流与剪贴板集大成者",
            "启动台天花板，不仅仅是启动应用，更融合了剪贴板历史、窗口分屏、多功能翻译、快捷脚本与庞大高质量社区插件生态。",
            "小伴精选编辑部",
            "freemium",
            serde_json::json!(["mac", "windows"]),
            serde_json::json!(["safe", "native", "recommended"]),
            true,
            100,
            "https://www.raycast.com/",
            Some("https://www.raycast.com/"),
            Some("https://www.raycast.com/"),
            None,
            None,
            0,
            Some("raycast"),
            None,
            serde_json::json!(["设计极度现代化美观", "社区插件极其丰富", "免费版功能已极其强悍"]),
            serde_json::json!(["高级 AI 功能需订阅"]),
            serde_json::json!(["Alfred", "macOS 自带聚焦 Spotlight"]),
            "1.85.0"
        ),
        (
            "lemon-cleaner",
            "腾讯柠檬清理",
            "https://lemon.qq.com/images/lemon/logo.png",
            "day-one",
            "腾讯官方出品的纯净 Mac 状态栏与垃圾清理工具",
            "清爽舒适的柠檬设计风格，开箱支持大文件扫描、卸载残留彻底清除、相似照片清理以及状态栏实时网速与硬件监控，免费无弹窗。",
            "小伴精选编辑部",
            "free",
            serde_json::json!(["mac"]),
            serde_json::json!(["safe", "clean", "recommended"]),
            true,
            95,
            "https://lemon.qq.com/",
            Some("https://lemon.qq.com/"),
            Some("https://lemon.qq.com/"),
            Some("https://apps.apple.com/cn/app/id1449962996"),
            None,
            0,
            Some("tencent-lemon"),
            None,
            serde_json::json!(["界面优雅纯净", "状态栏监控轻巧好用", "应用卸载干净无残留"]),
            serde_json::json!(["部分高级系统权限清理需下载官网完整版"]),
            serde_json::json!(["CleanMyMac 昂贵订阅版", "流氓安全卫士全家桶"]),
            "5.1.8"
        ),
        (
            "shottr",
            "Shottr",
            "https://shottr.cc/favicon.png",
            "day-one",
            "仅 1.6MB 的原生超快截图利器，长截图与取色神器",
            "启动速度快到不可思议，纯原生开发。支持滚动长截图、像素对齐标尺、智能文字 OCR 复制与敏感信息模糊，极客装机必备。",
            "极客合伙人",
            "freemium",
            serde_json::json!(["mac"]),
            serde_json::json!(["safe", "native", "clean"]),
            true,
            91,
            "https://shottr.cc/download",
            Some("https://shottr.cc/"),
            Some("https://shottr.cc/download"),
            None,
            None,
            0,
            Some("shottr"),
            None,
            serde_json::json!(["仅 1.6MB 极速秒开", "滚动长截图效果极佳", "支持测量间距与颜色拾取"]),
            serde_json::json!(["目前仅支持 macOS 平台"]),
            serde_json::json!(["臃肿且卡顿的传统截图工具"]),
            "1.7.3"
        ),

        // 5. 极客工坊
        (
            "ghostty",
            "Ghostty",
            "https://ghostty.org/favicon.png",
            "developer",
            "Zig 编写、GPU 硬件加速的次世代极速终端",
            "由知名开发者 Mitchell Hashimoto 操刀，基于 Zig 打造。字形排版近乎苛刻、渲染延迟极低，原生整合系统级多标签分屏。",
            "极客合伙人",
            "open_source",
            serde_json::json!(["mac", "linux"]),
            serde_json::json!(["open_source", "native", "safe"]),
            true,
            89,
            "https://ghostty.org/download",
            Some("https://ghostty.org/"),
            Some("https://ghostty.org/download"),
            None,
            Some("https://github.com/ghostty-org/ghostty"),
            28000,
            Some("ghostty"),
            None,
            serde_json::json!(["GPU 加速帧率拉满", "原生多平台体验统一", "配置语法简洁明快"]),
            serde_json::json!(["Windows 支持目前在积极演进中"]),
            serde_json::json!(["卡顿的老旧传统终端"]),
            "1.1.0"
        ),
        (
            "orbstack",
            "OrbStack",
            "https://orbstack.dev/favicon.png",
            "developer",
            "秒级启动、占用极低的 Docker 与 Linux 容器神器",
            "在 Mac 上运行容器的终极救星。启动时间由分钟级降至 2 秒内，后台待机 CPU 几乎为零，彻底解救风扇狂转与续航焦虑。",
            "小伴精选编辑部",
            "freemium",
            serde_json::json!(["mac"]),
            serde_json::json!(["safe", "native", "clean"]),
            true,
            93,
            "https://orbstack.dev/download",
            Some("https://orbstack.dev/"),
            Some("https://orbstack.dev/download"),
            None,
            None,
            0,
            Some("orbstack"),
            None,
            serde_json::json!(["启动仅 2 秒", "待机功耗极低", "双向无缝网络与文件挂载"]),
            serde_json::json!(["商业使用需要付费许可"]),
            serde_json::json!(["臃肿发烫的 Docker Desktop"]),
            "1.8.2"
        ),

        // 6. 数字减负
        (
            "ice",
            "Ice 菜单栏收纳",
            "https://raw.githubusercontent.com/jordanbaird/Ice/main/Ice/Assets.xcassets/AppIcon.appiconset/icon_512x512.png",
            "minimalism",
            "现代轻巧的 Mac 菜单栏收纳利器，纯原生开源",
            "专治菜单栏图标堆积过多的强迫症。支持点击/悬停隐藏与显示，完全 Swift 原生开源编写，彻底平替昂贵且争议繁多的商业软件。",
            "小伴精选编辑部",
            "open_source",
            serde_json::json!(["mac"]),
            serde_json::json!(["open_source", "native", "safe", "clean"]),
            true,
            88,
            "https://github.com/jordanbaird/Ice",
            Some("https://github.com/jordanbaird/Ice"),
            None,
            None,
            Some("https://github.com/jordanbaird/Ice"),
            17000,
            Some("jordanbaird-ice"),
            None,
            serde_json::json!(["开源免费", "原生动画丝滑", "支持多种触发姿势"]),
            serde_json::json!(["暂无 Windows 版本"]),
            serde_json::json!(["Bartender 昂贵订阅版"]),
            "0.11.0"
        ),

        // 7. 心流创作
        (
            "obsidian",
            "Obsidian",
            "https://obsidian.md/favicon.ico",
            "focus-flow",
            "本地优先的双链知识图谱，数据完全掌握在自己手中",
            "纯本地 Markdown 格式存储，没有被平台绑架的云端锁定风险。强大的双向链接、关系图谱可视化与海量社区插件，搭建专属第二大脑。",
            "小伴精选编辑部",
            "free",
            serde_json::json!(["mac", "windows", "linux", "ios", "android"]),
            serde_json::json!(["safe", "local_first", "recommended"]),
            true,
            96,
            "https://obsidian.md/download",
            Some("https://obsidian.md/"),
            Some("https://obsidian.md/download"),
            None,
            None,
            0,
            Some("obsidian"),
            Some("Obsidian.Obsidian"),
            serde_json::json!(["数据纯本地无锁定", "双链思维网状联结", "Canvas 白板功能强大"]),
            serde_json::json!(["插件过多时需要一定的维护精简"]),
            serde_json::json!(["强制云端锁定的笔记应用"]),
            "1.7.7"
        ),

        // 8. 生活美学
        (
            "xiaoban-desktop",
            "相伴动态桌面",
            "https://wallpaper.xiangban.xyz/favicon.ico",
            "life-aesthetics",
            "温暖治愈的动态壁纸与桌面陪伴，陪伴你的专注日常",
            "下班与专注时光的美学伴侣。提供 4K 超清自然风光与微动艺术壁纸，极低硬件功耗，不干扰办公与创作心流。",
            "小伴精选编辑部",
            "free",
            serde_json::json!(["mac", "windows"]),
            serde_json::json!(["safe", "native", "clean"]),
            true,
            98,
            "https://wallpaper.xiangban.xyz/",
            Some("https://wallpaper.xiangban.xyz/"),
            Some("https://wallpaper.xiangban.xyz/"),
            None,
            None,
            0,
            None,
            None,
            serde_json::json!(["超低硬件开销", "4K 治愈系画质", "纯粹无广告弹窗"]),
            serde_json::json!(["移动端正在开发中"]),
            serde_json::json!(["捆绑弹窗新闻的壁纸软件"]),
            "1.0.0"
        )
    ];

    for (
        slug, name, icon_url, cat_slug, summary, comment, referrer, pricing_type,
        platforms, badges, is_featured, sort_order, download_url, official_url,
        appstore_url, download_alt_url, github_url, github_stars, brew_cask, winget_id,
        pros, cons, alternatives, version
    ) in seeds {
        let cat_id = match cat_map.get(cat_slug) {
            Some(&id) => id,
            None => continue,
        };

        sqlx::query(
            r#"
            INSERT INTO appstore_apps (
                slug, name, icon_url, category_id, summary, comment, referrer,
                pricing_type, platforms, badges, status, is_featured, sort_order,
                download_url, official_url, appstore_url, download_alt_url,
                github_url, github_stars, brew_cask, winget_id, pros, cons, alternatives, version
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10, 'published', $11, $12,
                $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23, $24
            ) ON CONFLICT (slug) DO NOTHING;
            "#
        )
        .bind(slug)
        .bind(name)
        .bind(icon_url)
        .bind(cat_id)
        .bind(summary)
        .bind(comment)
        .bind(referrer)
        .bind(pricing_type)
        .bind(platforms)
        .bind(badges)
        .bind(is_featured)
        .bind(sort_order)
        .bind(download_url)
        .bind(official_url)
        .bind(appstore_url)
        .bind(download_alt_url)
        .bind(github_url)
        .bind(github_stars)
        .bind(brew_cask)
        .bind(winget_id)
        .bind(pros)
        .bind(cons)
        .bind(alternatives)
        .bind(version)
        .execute(pool)
        .await?;
    }

    Ok(())
}

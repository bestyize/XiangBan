use std::path::PathBuf;

/// 信任等级的自动升级门槛，都可以用环境变量调整。
#[derive(Clone, Debug)]
pub struct TrustRules {
    /// TL0 → TL1：注册天数与读过的帖子数。
    pub tl1_days: i64,
    pub tl1_reads: i64,
    /// 每天的发帖、回复上限，按等级 0..=3。
    pub daily_topics: [i64; 4],
    pub daily_posts: [i64; 4],
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    /// 对外地址，用于 SEO 页面的 canonical 与 sitemap。
    pub public_url: String,
    pub site_name: String,
    /// 账号中心签发者，同时是账号中心的地址。
    pub account_issuer: String,
    pub account_jwks_url: String,
    pub account_client_id: String,
    pub cors_origins: Vec<String>,
    pub account_provision_key: Option<String>,
    pub admin_user: Option<String>,
    pub admin_password: Option<String>,
    pub admin_jwt_secret: Vec<u8>,
    pub admin_token_ttl_secs: i64,
    pub web_dir: PathBuf,
    pub trust: TrustRules,
    /// 附件存放目录。
    pub upload_dir: PathBuf,
    /// 单个附件的大小上限（字节）：普通用户、守护者（站务）。
    pub upload_limit_user: u64,
    pub upload_limit_admin: u64,
    /// 正文插图的大小上限（字节），所有人一样。
    pub upload_limit_image: u64,
}

const MB: u64 = 1024 * 1024;

fn get(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn num(key: &str, default: i64) -> i64 {
    get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn list4(key: &str, default: [i64; 4]) -> [i64; 4] {
    let Some(raw) = get(key) else { return default };
    let parsed: Vec<i64> = raw.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    parsed.try_into().unwrap_or(default)
}

impl AppConfig {
    pub fn from_env() -> Self {
        let _ = dotenvy::from_filename("services/server-forum/.env");
        let _ = dotenvy::dotenv();

        let account_issuer = get("FORUM_ACCOUNT_ISSUER")
            .unwrap_or_else(|| "https://account.xiangban.xyz".into())
            .trim_end_matches('/')
            .to_string();
        let account_jwks_url =
            get("FORUM_ACCOUNT_JWKS_URL").unwrap_or_else(|| format!("{account_issuer}/.well-known/jwks.json"));

        let admin_jwt_secret = match get("FORUM_ADMIN_JWT_SECRET") {
            Some(s) if s.len() >= 32 => s.into_bytes(),
            _ => {
                tracing::warn!("FORUM_ADMIN_JWT_SECRET 未设置或少于 32 位，使用随机密钥，重启后管理员需重新登录");
                let a = uuid::Uuid::new_v4();
                let b = uuid::Uuid::new_v4();
                [a.as_bytes().as_slice(), b.as_bytes().as_slice()].concat()
            }
        };

        AppConfig {
            host: get("FORUM_HOST").unwrap_or_else(|| "127.0.0.1".into()),
            port: num("FORUM_PORT", 55010) as u16,
            database_url: get("FORUM_DATABASE_URL")
                .unwrap_or_else(|| "postgres://xiaban@localhost:5432/xiaban_forum".into()),
            public_url: get("FORUM_PUBLIC_URL")
                .unwrap_or_else(|| "https://chat.xiangban.xyz".into())
                .trim_end_matches('/')
                .to_string(),
            site_name: get("FORUM_SITE_NAME").unwrap_or_else(|| "超级小伴".into()),
            account_issuer,
            account_jwks_url,
            account_client_id: get("FORUM_ACCOUNT_CLIENT_ID").unwrap_or_else(|| "forum-web".into()),
            cors_origins: get("FORUM_CORS_ORIGINS")
                .map(|raw| raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
            account_provision_key: get("FORUM_ACCOUNT_PROVISION_KEY").filter(|v| v.len() >= 32),
            admin_user: get("FORUM_ADMIN_USER"),
            admin_password: get("FORUM_ADMIN_PASSWORD"),
            admin_jwt_secret,
            admin_token_ttl_secs: num("FORUM_ADMIN_TOKEN_TTL_SECS", 8 * 3600),
            web_dir: PathBuf::from(get("FORUM_WEB_DIR").unwrap_or_else(|| "services/server-forum/web/dist".into())),
            trust: TrustRules {
                tl1_days: num("FORUM_TL1_DAYS", 3),
                tl1_reads: num("FORUM_TL1_READS", 30),
                daily_topics: list4("FORUM_DAILY_TOPICS", [2, 5, 20, 50]),
                daily_posts: list4("FORUM_DAILY_POSTS", [10, 60, 200, 500]),
            },
            upload_dir: PathBuf::from(get("FORUM_UPLOAD_DIR").unwrap_or_else(|| "services/server-forum/data/uploads".into())),
            upload_limit_user: num("FORUM_UPLOAD_MAX_MB_USER", 5).max(1) as u64 * MB,
            upload_limit_admin: num("FORUM_UPLOAD_MAX_MB_ADMIN", 30).max(1) as u64 * MB,
            upload_limit_image: num("FORUM_UPLOAD_MAX_MB_IMAGE", 5).max(1) as u64 * MB,
        }
    }
}


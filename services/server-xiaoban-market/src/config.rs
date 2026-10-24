use std::{env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub admin_user: String,
    pub admin_pass: Option<String>,
    pub data_dir: PathBuf,
    pub web_dir: String,
    /// 仅测试使用：允许采集器访问回环 / 内网地址。
    pub allow_private_upstream: bool,
    /// 账号中心登录：设置了 issuer 才启用。令牌的 `iss` 必须等于它，`aud` 必须等于 `account_client_id`。
    pub account_issuer: Option<String>,
    pub account_jwks_url: String,
    /// 浏览器访问账号中心 API 的地址（默认同 issuer）。
    pub account_url: String,
    pub account_client_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let jwt_secret = env::var("MARKET_JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("未设置 MARKET_JWT_SECRET，已生成临时密钥；重启后所有登录会失效");
            format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple())
        });
        let issuer = env::var("MARKET_ACCOUNT_ISSUER").ok().filter(|v| !v.is_empty());
        Self {
            host: env::var("MARKET_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("MARKET_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(55010),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres@localhost:5432/xiaoban_market".into()),
            jwt_secret,
            admin_user: env::var("MARKET_ADMIN_USER").unwrap_or_else(|_| "admin".into()),
            admin_pass: env::var("MARKET_ADMIN_PASS").ok().filter(|v| !v.is_empty()),
            data_dir: env::var("MARKET_DATA_DIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("data")),
            web_dir: env::var("MARKET_WEB_DIR")
                .unwrap_or_else(|_| "services/server-xiaoban-market/web/dist".into()),
            allow_private_upstream: env::var("MARKET_ALLOW_PRIVATE_UPSTREAM").is_ok_and(|v| v == "true"),
            account_jwks_url: env::var("MARKET_ACCOUNT_JWKS_URL")
                .ok()
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| format!("{}/.well-known/jwks.json", issuer.clone().unwrap_or_default().trim_end_matches('/'))),
            account_url: env::var("MARKET_ACCOUNT_URL").ok().filter(|v| !v.is_empty()).unwrap_or_else(|| issuer.clone().unwrap_or_default()),
            account_client_id: env::var("MARKET_ACCOUNT_CLIENT_ID").ok().filter(|v| !v.is_empty()).unwrap_or_else(|| "xiaoban-market".into()),
            account_issuer: issuer,
        }
    }
}

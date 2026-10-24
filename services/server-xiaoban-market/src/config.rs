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
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let jwt_secret = env::var("MARKET_JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("未设置 MARKET_JWT_SECRET，已生成临时密钥；重启后所有登录会失效");
            format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple())
        });
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
        }
    }
}

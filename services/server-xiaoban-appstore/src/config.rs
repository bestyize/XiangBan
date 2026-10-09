use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub default_admin_user: String,
    pub default_admin_pass: String,
    pub web_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let host = env::var("APPSTORE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("APPSTORE_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(55008);

        // 默认连接本地 PostgreSQL (用户 xiaban，库 xiaban)
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://xiaban@localhost:5432/xiaban".to_string());

        let jwt_secret = env::var("APPSTORE_JWT_SECRET")
            .unwrap_or_else(|_| "xiaoban-appstore-super-secret-key-2026-curated".to_string());

        let default_admin_user = env::var("APPSTORE_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
        let default_admin_pass = env::var("APPSTORE_ADMIN_PASS").unwrap_or_else(|_| "admin123".to_string());

        let web_dir = env::var("APPSTORE_WEB_DIR").unwrap_or_else(|_| "services/server-xiaoban-appstore/web/dist".to_string());

        Self {
            host,
            port,
            database_url,
            jwt_secret,
            default_admin_user,
            default_admin_pass,
            web_dir,
        }
    }
}

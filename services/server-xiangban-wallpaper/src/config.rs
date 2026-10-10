use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: Option<String>,
    pub unsplash_access_key: Option<String>,
    pub pixabay_api_key: Option<String>,
    pub pexels_api_key: Option<String>,
    pub okslop_api_key: Option<String>,
    pub bing_api_key: Option<String>,
    pub enable_demo_data: bool,
    /// Initial search-cache lifetime; once saved from the admin console the stored value wins.
    pub cache_ttl_secs: u64,
    pub auto_sync_interval_mins: u64,
    /// 统一账号中心发行者 URL，例如 https://account.xiangban.xyz
    pub account_issuer: String,
    /// 账号中心 JWKS 公钥端点，默认 {account_issuer}/.well-known/jwks.json
    pub account_jwks_url: Option<String>,
    /// 允许访问本服务的应用 client_id 列表
    pub account_allowed_clients: Vec<String>,
    /// 可选的离线静态 Ed25519 公钥（base64url 格式），用于单测或断网环境
    pub account_public_key: Option<String>,
}

impl AppConfig {
    pub fn resolved_jwks_url(&self) -> String {
        self.account_jwks_url.clone().unwrap_or_else(|| {
            format!("{}/.well-known/jwks.json", self.account_issuer.trim_end_matches('/'))
        })
    }

    pub fn from_env() -> Self {
        // Load .env if present
        let _ = dotenvy::dotenv();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let database_url = env::var("DATABASE_URL").ok().filter(|s| !s.trim().is_empty());
        let unsplash_access_key = env::var("UNSPLASH_ACCESS_KEY").ok().filter(|s| !s.trim().is_empty());
        let pixabay_api_key = env::var("PIXABAY_API_KEY").ok().filter(|s| !s.trim().is_empty());
        let pexels_api_key = env::var("PEXELS_API_KEY").ok().filter(|s| !s.trim().is_empty());
        let okslop_api_key = env::var("OKSLOP_API_KEY").ok().filter(|s| !s.trim().is_empty());
        let bing_api_key = env::var("BING_API_KEY").ok().filter(|s| !s.trim().is_empty());

        let enable_demo_data = env::var("ENABLE_DEMO_DATA")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        let cache_ttl_secs = env::var("CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);

        let auto_sync_interval_mins = env::var("AUTO_SYNC_INTERVAL_MINS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let account_issuer = env::var("ACCOUNT_ISSUER")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://account.xiangban.xyz".to_string());

        let account_jwks_url = env::var("ACCOUNT_JWKS_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let account_allowed_clients = env::var("ACCOUNT_ALLOWED_CLIENTS")
            .or_else(|_| env::var("ACCOUNT_CLIENTS"))
            .unwrap_or_else(|_| {
                "xiangban-wallpaper-web,xiaoban-desktop-wallpaper,xiaoban-mobile-wallpaper".to_string()
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let account_public_key = env::var("ACCOUNT_PUBLIC_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Self {
            server_host,
            server_port,
            database_url,
            unsplash_access_key,
            pixabay_api_key,
            pexels_api_key,
            okslop_api_key,
            bing_api_key,
            enable_demo_data,
            cache_ttl_secs,
            auto_sync_interval_mins,
            account_issuer,
            account_jwks_url,
            account_allowed_clients,
            account_public_key,
        }
    }
}

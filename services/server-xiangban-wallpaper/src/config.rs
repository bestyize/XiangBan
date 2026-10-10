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
}

impl AppConfig {
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
        }
    }
}

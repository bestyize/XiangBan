use std::{env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub account_issuer: String,
    pub account_jwks_url: String,
    pub newapi_base_url: String,
    pub newapi_admin_token: Option<String>,
    pub newapi_default_ratio: f64,
    pub quota_per_unit: i64,
    pub data_dir: PathBuf,
    pub master_key_base64: Option<String>,
    pub admin_user: Option<String>,
    pub admin_password: Option<String>,
    pub admin_token_ttl_secs: i64,
    pub sync_interval_secs: u64,
    pub trust_proxy: bool,
    pub cors_origins: Vec<String>,
    pub web_dir: PathBuf,
    pub max_body_bytes: usize,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self::from_lookup(|key| env::var(key).ok().filter(|v| !v.trim().is_empty()))
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Self {
        let num = |key: &str, default: i64| get(key).and_then(|v| v.parse().ok()).unwrap_or(default);
        let issuer = get("AI_ACCOUNT_ISSUER").unwrap_or_else(|| "https://account.xiangban.xyz".into());
        let jwks_url = get("AI_ACCOUNT_JWKS_URL")
            .unwrap_or_else(|| format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/')));

        Self {
            host: get("AI_HOST").unwrap_or_else(|| "127.0.0.1".into()),
            port: num("AI_PORT", 55010) as u16,
            database_url: get("AI_DATABASE_URL")
                .unwrap_or_else(|| "postgres://xiaban@localhost:5432/xiaban_ai".into()),
            account_issuer: issuer,
            account_jwks_url: jwks_url,
            newapi_base_url: get("AI_NEWAPI_BASE_URL")
                .unwrap_or_else(|| "http://127.0.0.1:3000".into())
                .trim_end_matches('/')
                .to_string(),
            newapi_admin_token: get("AI_NEWAPI_ADMIN_TOKEN"),
            newapi_default_ratio: get("AI_NEWAPI_DEFAULT_RATIO").and_then(|v| v.parse().ok()).unwrap_or(37.5),
            quota_per_unit: num("AI_QUOTA_PER_UNIT", 500_000).max(1),
            data_dir: PathBuf::from(get("AI_DATA_DIR").unwrap_or_else(|| "ai_data".into())),
            master_key_base64: get("AI_MASTER_KEY_BASE64"),
            admin_user: get("AI_ADMIN_USER"),
            admin_password: get("AI_ADMIN_PASSWORD"),
            admin_token_ttl_secs: num("AI_ADMIN_TOKEN_TTL_SECS", 8 * 3600),
            sync_interval_secs: num("AI_SYNC_INTERVAL_SECS", 60).max(5) as u64,
            trust_proxy: get("AI_TRUST_PROXY").map(|v| v == "true" || v == "1").unwrap_or(false),
            cors_origins: get("AI_CORS_ORIGINS")
                .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
            web_dir: PathBuf::from(get("AI_WEB_DIR").unwrap_or_else(|| "services/server-ai/web/dist".into())),
            max_body_bytes: num("AI_MAX_BODY_BYTES", 8 * 1024 * 1024).max(1024) as usize,
        }
    }
}

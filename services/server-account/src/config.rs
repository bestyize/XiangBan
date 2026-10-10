use std::{env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct SmsConfig {
    pub sdk_app_id: String,
    pub sign_name: String,
    pub template_id: String,
    pub region: String,
}

#[derive(Clone, Debug)]
pub struct SesConfig {
    pub from: String,
    pub template_id: u64,
    pub subject: String,
    pub region: String,
}

#[derive(Clone, Debug)]
pub struct TencentConfig {
    pub secret_id: String,
    pub secret_key: String,
    pub sms: Option<SmsConfig>,
    pub ses: Option<SesConfig>,
}

#[derive(Clone, Debug)]
pub enum SenderConfig {
    Console,
    Tencent(TencentConfig),
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub issuer: String,
    pub data_dir: PathBuf,
    pub jwt_pkcs8_base64: Option<String>,
    pub master_key_base64: Option<String>,
    pub access_ttl_secs: i64,
    pub refresh_ttl_days: i64,
    pub code_ttl_secs: i64,
    pub resend_cooldown_secs: i64,
    pub max_code_attempts: i32,
    pub daily_per_identifier: i64,
    pub daily_per_ip: i64,
    pub trust_proxy: bool,
    pub cors_origins: Vec<String>,
    pub email_clients: Vec<String>,
    pub clients: Vec<(String, String)>,
    pub sender: SenderConfig,
    pub admin_user: Option<String>,
    pub admin_password: Option<String>,
    pub admin_token_ttl_secs: i64,
    pub web_dir: PathBuf,
}

const DEFAULT_CLIENTS: &[(&str, &str)] = &[
    ("xiaban-web", "相伴网页"),
    ("xiaban-api", "相伴主 API"),
    ("xiangban-wallpaper-web", "小伴壁纸网站"),
    ("xiaoban-desktop-wallpaper", "小伴桌面壁纸"),
    ("xiaoban-mobile-wallpaper", "小伴手机壁纸"),
];

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self::from_lookup(|key| env::var(key).ok().filter(|v| !v.trim().is_empty()))
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Self {
        let num = |key: &str, default: i64| get(key).and_then(|v| v.parse().ok()).unwrap_or(default);
        let list = |key: &str| -> Vec<String> {
            get(key)
                .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default()
        };

        let clients = match get("ACCOUNT_CLIENTS") {
            Some(raw) => raw
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|item| match item.split_once(':') {
                    Some((id, name)) => (id.trim().to_string(), name.trim().to_string()),
                    None => (item.to_string(), item.to_string()),
                })
                .collect(),
            None => DEFAULT_CLIENTS.iter().map(|(i, n)| (i.to_string(), n.to_string())).collect(),
        };

        let sender = match get("ACCOUNT_SENDER").as_deref() {
            Some("tencent") => SenderConfig::Tencent(TencentConfig {
                secret_id: get("TENCENT_SECRET_ID").unwrap_or_default(),
                secret_key: get("TENCENT_SECRET_KEY").unwrap_or_default(),
                sms: get("TENCENT_SMS_SDK_APP_ID").map(|sdk_app_id| SmsConfig {
                    sdk_app_id,
                    sign_name: get("TENCENT_SMS_SIGN_NAME").unwrap_or_default(),
                    template_id: get("TENCENT_SMS_TEMPLATE_ID").unwrap_or_default(),
                    region: get("TENCENT_SMS_REGION").unwrap_or_else(|| "ap-guangzhou".into()),
                }),
                ses: get("TENCENT_SES_FROM").map(|from| SesConfig {
                    from,
                    template_id: get("TENCENT_SES_TEMPLATE_ID").and_then(|v| v.parse().ok()).unwrap_or(0),
                    subject: get("TENCENT_SES_SUBJECT").unwrap_or_else(|| "相伴验证码".into()),
                    region: get("TENCENT_SES_REGION").unwrap_or_else(|| "ap-hongkong".into()),
                }),
            }),
            _ => SenderConfig::Console,
        };

        Self {
            host: get("ACCOUNT_HOST").unwrap_or_else(|| "0.0.0.0".into()),
            port: num("ACCOUNT_PORT", 55009) as u16,
            database_url: get("ACCOUNT_DATABASE_URL")
                .unwrap_or_else(|| "postgres://xiaban@localhost:5432/xiaban_account".into()),
            issuer: get("ACCOUNT_ISSUER").unwrap_or_else(|| "https://account.xiangban.xyz".into()),
            data_dir: PathBuf::from(get("ACCOUNT_DATA_DIR").unwrap_or_else(|| "account_data".into())),
            jwt_pkcs8_base64: get("ACCOUNT_JWT_PKCS8_BASE64"),
            master_key_base64: get("ACCOUNT_MASTER_KEY_BASE64"),
            access_ttl_secs: num("ACCOUNT_ACCESS_TTL_SECS", 1800),
            refresh_ttl_days: num("ACCOUNT_REFRESH_TTL_DAYS", 30),
            code_ttl_secs: num("ACCOUNT_CODE_TTL_SECS", 300),
            resend_cooldown_secs: num("ACCOUNT_RESEND_COOLDOWN_SECS", 60),
            max_code_attempts: num("ACCOUNT_MAX_CODE_ATTEMPTS", 5) as i32,
            daily_per_identifier: num("ACCOUNT_DAILY_PER_IDENTIFIER", 10),
            daily_per_ip: num("ACCOUNT_DAILY_PER_IP", 30),
            trust_proxy: get("ACCOUNT_TRUST_PROXY").map(|v| v == "true" || v == "1").unwrap_or(false),
            cors_origins: list("ACCOUNT_CORS_ORIGINS"),
            email_clients: match get("ACCOUNT_EMAIL_CLIENTS") {
                Some(_) => list("ACCOUNT_EMAIL_CLIENTS"),
                None => vec!["xiaban-web".into(), "xiaban-api".into()],
            },
            clients,
            sender,
            admin_user: get("ACCOUNT_ADMIN_USER"),
            admin_password: get("ACCOUNT_ADMIN_PASSWORD"),
            admin_token_ttl_secs: num("ACCOUNT_ADMIN_TOKEN_TTL_SECS", 8 * 3600),
            web_dir: PathBuf::from(
                get("ACCOUNT_WEB_DIR").unwrap_or_else(|| "services/server-account/web/dist".into()),
            ),
        }
    }
}

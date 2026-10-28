use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    /// 账号中心签发者与公钥地址。
    pub account_issuer: String,
    pub account_jwks_url: String,
    /// 接受的客户端编号（令牌的 aud）：桌面端与手机端各一个。
    pub client_ids: Vec<String>,
    pub cors_origins: Vec<String>,
    /// 配对码有效期。
    pub pair_code_ttl: Duration,
    /// 一个配对码最多被兑换几次（被桌面拒绝的也算），超过即作废。
    pub pair_code_max_redeems: i32,
    /// 每个账号每分钟最多兑换几次配对码。
    pub pair_redeems_per_minute: usize,
    /// 每台设备每小时最多申请几次配对码。
    pub pair_codes_per_hour: usize,
    /// 每个账号最多登记多少台设备。
    pub max_devices_per_user: i64,
    /// 对端离线时，每个对端最多暂存几条通知，以及暂存多久。
    pub notify_queue_cap: usize,
    pub notify_ttl: Duration,
    /// 每个连接每秒的消息数与字节数上限。
    pub msgs_per_sec: f64,
    pub bytes_per_sec: f64,
}

fn get(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn num<T: std::str::FromStr>(key: &str, default: T) -> T {
    get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        let issuer = get("LINK_ACCOUNT_ISSUER").unwrap_or_else(|| "https://account.xiangban.xyz".into());
        Self {
            host: get("LINK_HOST").unwrap_or_else(|| "0.0.0.0".into()),
            port: num("LINK_PORT", 55030),
            database_url: get("LINK_DATABASE_URL").unwrap_or_else(|| "postgres://xiaban:xiaban@localhost:5432/xiaoban_link".into()),
            account_jwks_url: get("LINK_ACCOUNT_JWKS_URL").unwrap_or_else(|| format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/'))),
            account_issuer: issuer,
            client_ids: get("LINK_ACCOUNT_CLIENT_IDS")
                .unwrap_or_else(|| "xiaoban-desktop-wallpaper,xiaoban-android".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            cors_origins: get("LINK_CORS_ORIGINS").map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default(),
            pair_code_ttl: Duration::from_secs(num("LINK_PAIR_CODE_TTL_SECS", 300)),
            pair_code_max_redeems: num("LINK_PAIR_CODE_MAX_REDEEMS", 5),
            pair_redeems_per_minute: num("LINK_PAIR_REDEEMS_PER_MINUTE", 10),
            pair_codes_per_hour: num("LINK_PAIR_CODES_PER_HOUR", 20),
            max_devices_per_user: num("LINK_MAX_DEVICES_PER_USER", 20),
            notify_queue_cap: num("LINK_NOTIFY_QUEUE_CAP", 50),
            notify_ttl: Duration::from_secs(num("LINK_NOTIFY_TTL_SECS", 24 * 3600)),
            msgs_per_sec: num("LINK_MSGS_PER_SEC", 30.0),
            bytes_per_sec: num("LINK_BYTES_PER_SEC", 1024.0 * 1024.0),
        }
    }
}

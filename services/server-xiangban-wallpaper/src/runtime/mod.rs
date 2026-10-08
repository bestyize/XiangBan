//! Hot configuration for outbound provider traffic: which API keys and proxies are in use,
//! how they rotate, and which of them are currently resting after a failure.
//!
//! Everything the admin console edits lands here through `reload`, so a change takes effect on
//! the next request without a restart.

mod upstream;

pub use upstream::Upstream;

use crate::{
    config::AppConfig,
    error::AppError,
    store::{KeyRow, ProxyRow, Store},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, RwLock},
    time::{Duration, Instant},
};

pub const PROVIDER_IDS: [&str; 3] = ["unsplash", "pixabay", "pexels"];

pub fn is_known_provider(id: &str) -> bool {
    PROVIDER_IDS.contains(&id)
}

const SETTING_CACHE_ENABLED: &str = "cache.enabled";
const SETTING_CACHE_TTL: &str = "cache.ttl_secs";
const SETTING_CACHE_MAX_ENTRIES: &str = "cache.max_entries";
const SETTING_PROXY_STRATEGY: &str = "proxy.strategy";
const SETTING_ENV_KEYS_IMPORTED: &str = "bootstrap.env_keys_imported";

fn use_proxy_setting(provider: &str) -> String {
    format!("provider.{provider}.use_proxy")
}

pub const MIN_CACHE_TTL_SECS: u64 = 1;
pub const MAX_CACHE_TTL_SECS: u64 = 30 * 24 * 3600;
pub const MAX_CACHE_ENTRIES: u64 = 100_000;
const DEFAULT_CACHE_ENTRIES: u64 = 1000;

const KEY_COOLDOWN_AUTH: Duration = Duration::from_secs(600);
const KEY_COOLDOWN_RATE_LIMIT: Duration = Duration::from_secs(120);
const PROXY_COOLDOWN: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyStrategy {
    RoundRobin,
    Random,
}

impl ProxyStrategy {
    fn parse(s: &str) -> Self {
        if s == "random" {
            Self::Random
        } else {
            Self::RoundRobin
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::RoundRobin => "round_robin",
            Self::Random => "random",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheSettings {
    pub enabled: bool,
    pub ttl_secs: u64,
    pub max_entries: u64,
}

impl CacheSettings {
    pub fn validate(&self) -> Result<(), AppError> {
        if !(MIN_CACHE_TTL_SECS..=MAX_CACHE_TTL_SECS).contains(&self.ttl_secs) {
            return Err(AppError::BadRequest(format!(
                "缓存有效期需在 {MIN_CACHE_TTL_SECS} 秒到 {MAX_CACHE_TTL_SECS} 秒（30 天）之间"
            )));
        }
        if !(1..=MAX_CACHE_ENTRIES).contains(&self.max_entries) {
            return Err(AppError::BadRequest(format!("缓存条数上限需在 1 到 {MAX_CACHE_ENTRIES} 之间")));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub id: i64,
    pub value: String,
}

#[derive(Clone)]
struct ProxyEntry {
    id: i64,
    client: reqwest::Client,
}

struct Snapshot {
    keys: HashMap<String, Vec<KeyEntry>>,
    proxies: Vec<ProxyEntry>,
    direct: reqwest::Client,
    strategy: ProxyStrategy,
    use_proxy: HashMap<String, bool>,
    cache: CacheSettings,
}

#[derive(Debug, Clone, Default)]
pub struct KeyHealth {
    pub requests: u64,
    pub failures: u64,
    pub last_status: Option<u16>,
    cooldown_until: Option<Instant>,
}

impl KeyHealth {
    pub fn cooldown_remaining(&self) -> Option<Duration> {
        self.cooldown_until.and_then(|t| t.checked_duration_since(Instant::now()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProxyHealth {
    pub requests: u64,
    pub failures: u64,
    pub last_error: Option<String>,
    cooldown_until: Option<Instant>,
}

impl ProxyHealth {
    pub fn cooldown_remaining(&self) -> Option<Duration> {
        self.cooldown_until.and_then(|t| t.checked_duration_since(Instant::now()))
    }
}

#[derive(Default)]
struct State {
    keys: HashMap<i64, KeyHealth>,
    proxies: HashMap<i64, ProxyHealth>,
    key_cursor: HashMap<String, usize>,
    proxy_cursor: usize,
}

struct Inner {
    snapshot: RwLock<Arc<Snapshot>>,
    state: Mutex<State>,
    default_cache_ttl: u64,
}

#[derive(Clone)]
pub struct Runtime {
    inner: Arc<Inner>,
}

pub(crate) struct ChosenProxy {
    pub id: i64,
    pub client: reqwest::Client,
}

fn build_client(proxy_url: Option<&str>) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(6))
        .user_agent(concat!("WallpaperCollect/", env!("CARGO_PKG_VERSION")));
    if let Some(url) = proxy_url {
        // `no_proxy` drops the system proxy so that only the configured one is used.
        builder = builder.no_proxy().proxy(reqwest::Proxy::all(url)?);
    }
    builder.build()
}

impl Snapshot {
    fn build(
        key_rows: Vec<KeyRow>,
        proxy_rows: Vec<ProxyRow>,
        settings: &HashMap<String, String>,
        default_cache_ttl: u64,
    ) -> Self {
        let mut keys: HashMap<String, Vec<KeyEntry>> = HashMap::new();
        for row in key_rows.into_iter().filter(|k| k.enabled) {
            keys.entry(row.provider).or_default().push(KeyEntry {
                id: row.id,
                value: row.api_key,
            });
        }

        let proxies = proxy_rows
            .into_iter()
            .filter(|p| p.enabled)
            .filter_map(|p| match build_client(Some(&p.url)) {
                Ok(client) => Some(ProxyEntry { id: p.id, client }),
                Err(e) => {
                    tracing::warn!("Skipping proxy '{}': {e}", p.name);
                    None
                }
            })
            .collect();

        let use_proxy = PROVIDER_IDS
            .iter()
            .map(|p| {
                let on = settings.get(&use_proxy_setting(p)).is_some_and(|v| v == "true");
                (p.to_string(), on)
            })
            .collect();

        let cache = CacheSettings {
            enabled: settings.get(SETTING_CACHE_ENABLED).map_or(true, |v| v != "false"),
            ttl_secs: settings
                .get(SETTING_CACHE_TTL)
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_cache_ttl)
                .clamp(MIN_CACHE_TTL_SECS, MAX_CACHE_TTL_SECS),
            max_entries: settings
                .get(SETTING_CACHE_MAX_ENTRIES)
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_CACHE_ENTRIES)
                .clamp(1, MAX_CACHE_ENTRIES),
        };

        Self {
            keys,
            proxies,
            direct: build_client(None).unwrap_or_default(),
            strategy: settings
                .get(SETTING_PROXY_STRATEGY)
                .map_or(ProxyStrategy::RoundRobin, |v| ProxyStrategy::parse(v)),
            use_proxy,
            cache,
        }
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Runtime {
    fn with_snapshot(snapshot: Snapshot, default_cache_ttl: u64) -> Self {
        Self {
            inner: Arc::new(Inner {
                snapshot: RwLock::new(Arc::new(snapshot)),
                state: Mutex::new(State::default()),
                default_cache_ttl,
            }),
        }
    }

    /// No keys, no proxies, default cache settings. Used by tests.
    pub fn empty() -> Self {
        Self::with_snapshot(Snapshot::build(Vec::new(), Vec::new(), &HashMap::new(), 3600), 3600)
    }

    /// Keys taken straight from environment variables, for running without PostgreSQL.
    pub fn from_env(config: &AppConfig) -> Self {
        let env_keys = env_keys(config);
        let rows = env_keys
            .into_iter()
            .enumerate()
            .map(|(i, (provider, key))| KeyRow {
                // Negative ids cannot collide with database rows.
                id: -(i as i64) - 1,
                provider: provider.to_string(),
                label: ".env".to_string(),
                api_key: key,
                enabled: true,
                created_at: chrono::Utc::now(),
            })
            .collect();
        let snapshot = Snapshot::build(rows, Vec::new(), &HashMap::new(), config.cache_ttl_secs);
        Self::with_snapshot(snapshot, config.cache_ttl_secs)
    }

    /// Loads configuration from PostgreSQL. On the first run, keys from the `.env` file are
    /// copied into the database so they can be managed from the console from then on.
    pub async fn load(store: &Store, config: &AppConfig) -> Result<Self, AppError> {
        if store.set_setting_if_absent(SETTING_ENV_KEYS_IMPORTED, "true").await? {
            for (provider, key) in env_keys(config) {
                store.insert_key(provider, "从 .env 导入", &key).await?;
            }
        }
        let snapshot = Self::read_snapshot(store, config.cache_ttl_secs).await?;
        Ok(Self::with_snapshot(snapshot, config.cache_ttl_secs))
    }

    async fn read_snapshot(store: &Store, default_cache_ttl: u64) -> Result<Snapshot, AppError> {
        let keys = store.list_keys().await?;
        let proxies = store.list_proxies().await?;
        let settings = store.all_settings().await?;
        Ok(Snapshot::build(keys, proxies, &settings, default_cache_ttl))
    }

    /// Re-reads keys, proxies and settings. Cooldowns and counters survive because they are keyed by id.
    pub async fn reload(&self, store: &Store) -> Result<(), AppError> {
        let snapshot = Self::read_snapshot(store, self.inner.default_cache_ttl).await?;
        *self.inner.snapshot.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(snapshot);
        Ok(())
    }

    fn snapshot(&self) -> Arc<Snapshot> {
        self.inner.snapshot.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn upstream(&self, provider: &'static str) -> Upstream {
        Upstream::new(self.clone(), provider)
    }

    pub fn cache_settings(&self) -> CacheSettings {
        self.snapshot().cache
    }

    pub fn proxy_strategy(&self) -> ProxyStrategy {
        self.snapshot().strategy
    }

    pub fn uses_proxy(&self, provider: &str) -> bool {
        self.snapshot().use_proxy.get(provider).copied().unwrap_or(false)
    }

    /// A provider counts as configured while it has at least one enabled key, even if all of
    /// them are resting at the moment.
    pub fn has_keys(&self, provider: &str) -> bool {
        self.snapshot().keys.get(provider).is_some_and(|k| !k.is_empty())
    }

    pub(crate) fn has_available_key(&self, provider: &str) -> bool {
        let snapshot = self.snapshot();
        let Some(keys) = snapshot.keys.get(provider) else {
            return false;
        };
        let state = lock(&self.inner.state);
        let now = Instant::now();
        keys.iter().any(|k| !is_cooling(state.keys.get(&k.id).and_then(|h| h.cooldown_until), now))
    }

    pub(crate) fn direct_client(&self) -> reqwest::Client {
        self.snapshot().direct.clone()
    }

    /// Next enabled key in rotation that is not resting, or `None` if there is none.
    pub(crate) fn pick_key(&self, provider: &str) -> Option<KeyEntry> {
        let snapshot = self.snapshot();
        let keys = snapshot.keys.get(provider).filter(|k| !k.is_empty())?;
        let mut state = lock(&self.inner.state);
        let now = Instant::now();
        let start = state.key_cursor.get(provider).copied().unwrap_or(0);
        for offset in 0..keys.len() {
            let index = (start + offset) % keys.len();
            let key = &keys[index];
            if !is_cooling(state.keys.get(&key.id).and_then(|h| h.cooldown_until), now) {
                state.key_cursor.insert(provider.to_string(), index + 1);
                return Some(key.clone());
            }
        }
        None
    }

    /// The proxy to use for this provider, or `None` to connect directly.
    pub(crate) fn pick_proxy(&self, provider: &str) -> Option<ChosenProxy> {
        let snapshot = self.snapshot();
        if !snapshot.use_proxy.get(provider).copied().unwrap_or(false) || snapshot.proxies.is_empty() {
            return None;
        }
        let count = snapshot.proxies.len();
        let mut state = lock(&self.inner.state);
        let now = Instant::now();
        let start = match snapshot.strategy {
            ProxyStrategy::RoundRobin => state.proxy_cursor % count,
            ProxyStrategy::Random => (uuid::Uuid::new_v4().as_u128() % count as u128) as usize,
        };
        // If every proxy is resting, try the first candidate anyway rather than bypassing the pool.
        let index = (0..count)
            .map(|offset| (start + offset) % count)
            .find(|&i| !is_cooling(state.proxies.get(&snapshot.proxies[i].id).and_then(|h| h.cooldown_until), now))
            .unwrap_or(start);
        state.proxy_cursor = index + 1;
        let chosen = &snapshot.proxies[index];
        Some(ChosenProxy {
            id: chosen.id,
            client: chosen.client.clone(),
        })
    }

    pub(crate) fn record_response(&self, key_id: i64, proxy_id: Option<i64>, status: u16, retry_after: Option<Duration>) {
        let mut state = lock(&self.inner.state);
        let key = state.keys.entry(key_id).or_default();
        key.requests += 1;
        key.last_status = Some(status);
        match status {
            429 => {
                key.failures += 1;
                let wait = retry_after.unwrap_or(KEY_COOLDOWN_RATE_LIMIT).clamp(Duration::from_secs(5), Duration::from_secs(3600));
                key.cooldown_until = Some(Instant::now() + wait);
            }
            // Unsplash reports an exhausted hourly quota as 403, so 401/403 rest the key too.
            401 | 403 => {
                key.failures += 1;
                key.cooldown_until = Some(Instant::now() + KEY_COOLDOWN_AUTH);
            }
            500..=599 => key.failures += 1,
            _ => key.cooldown_until = None,
        }
        if let Some(id) = proxy_id {
            let proxy = state.proxies.entry(id).or_default();
            proxy.requests += 1;
            proxy.last_error = None;
            proxy.cooldown_until = None;
        }
    }

    pub(crate) fn record_network_error(&self, proxy_id: Option<i64>, message: &str) {
        let Some(id) = proxy_id else {
            return;
        };
        let mut state = lock(&self.inner.state);
        let proxy = state.proxies.entry(id).or_default();
        proxy.requests += 1;
        proxy.failures += 1;
        proxy.last_error = Some(message.to_string());
        proxy.cooldown_until = Some(Instant::now() + PROXY_COOLDOWN);
    }

    pub fn key_health(&self, id: i64) -> KeyHealth {
        lock(&self.inner.state).keys.get(&id).cloned().unwrap_or_default()
    }

    pub fn proxy_health(&self, id: i64) -> ProxyHealth {
        lock(&self.inner.state).proxies.get(&id).cloned().unwrap_or_default()
    }

    #[cfg(test)]
    fn rest_key(&self, id: i64, wait: Duration) {
        lock(&self.inner.state).keys.entry(id).or_default().cooldown_until = Some(Instant::now() + wait);
    }
}

fn is_cooling(until: Option<Instant>, now: Instant) -> bool {
    until.is_some_and(|t| t > now)
}

fn env_keys(config: &AppConfig) -> Vec<(&'static str, String)> {
    [
        ("unsplash", &config.unsplash_access_key),
        ("pixabay", &config.pixabay_api_key),
        ("pexels", &config.pexels_api_key),
    ]
    .into_iter()
    .filter_map(|(provider, key)| key.as_ref().map(|k| (provider, k.clone())))
    .collect()
}

/* ---- persistence of the settings edited in the console ---- */

pub async fn save_cache_settings(store: &Store, settings: &CacheSettings) -> Result<(), AppError> {
    settings.validate()?;
    store.set_setting(SETTING_CACHE_ENABLED, if settings.enabled { "true" } else { "false" }).await?;
    store.set_setting(SETTING_CACHE_TTL, &settings.ttl_secs.to_string()).await?;
    store.set_setting(SETTING_CACHE_MAX_ENTRIES, &settings.max_entries.to_string()).await
}

pub async fn save_proxy_strategy(store: &Store, strategy: ProxyStrategy) -> Result<(), AppError> {
    store.set_setting(SETTING_PROXY_STRATEGY, strategy.as_str()).await
}

pub async fn save_use_proxy(store: &Store, provider: &str, on: bool) -> Result<(), AppError> {
    store.set_setting(&use_proxy_setting(provider), if on { "true" } else { "false" }).await
}

/* ---- proxy connectivity test ---- */

#[derive(Debug, Clone, Serialize)]
pub struct ProxyProbe {
    pub target: &'static str,
    pub ok: bool,
    pub status: Option<u16>,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// Requests each provider's API host through the proxy. Any HTTP answer, even a 401, proves the
/// proxy can reach that host; only a connection failure or timeout counts as a failed probe.
pub async fn probe_proxy(url: &str) -> Vec<ProxyProbe> {
    const TARGETS: [(&str, &str); 3] = [
        ("Unsplash", "https://api.unsplash.com/"),
        ("Pixabay", "https://pixabay.com/api/"),
        ("Pexels", "https://api.pexels.com/v1/"),
    ];

    let client = match build_client(Some(url)) {
        Ok(c) => c,
        Err(e) => {
            let error = upstream::describe(e);
            return TARGETS
                .iter()
                .map(|(target, _)| ProxyProbe {
                    target,
                    ok: false,
                    status: None,
                    latency_ms: 0,
                    error: Some(error.clone()),
                })
                .collect();
        }
    };

    let probes = TARGETS.iter().map(|(target, endpoint)| {
        let client = client.clone();
        async move {
            let started = Instant::now();
            match client.get(*endpoint).send().await {
                Ok(res) => ProxyProbe {
                    target,
                    ok: true,
                    status: Some(res.status().as_u16()),
                    latency_ms: started.elapsed().as_millis() as u64,
                    error: None,
                },
                Err(e) => ProxyProbe {
                    target,
                    ok: false,
                    status: None,
                    latency_ms: started.elapsed().as_millis() as u64,
                    error: Some(upstream::describe(e)),
                },
            }
        }
    });
    futures::future::join_all(probes).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_row(id: i64, provider: &str, enabled: bool) -> KeyRow {
        KeyRow {
            id,
            provider: provider.to_string(),
            label: format!("k{id}"),
            api_key: format!("secret-{id}"),
            enabled,
            created_at: chrono::Utc::now(),
        }
    }

    fn runtime(keys: Vec<KeyRow>, settings: &[(&str, &str)]) -> Runtime {
        let settings = settings.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        Runtime::with_snapshot(Snapshot::build(keys, Vec::new(), &settings, 3600), 3600)
    }

    #[test]
    fn keys_rotate_round_robin_and_skip_disabled() {
        let rt = runtime(
            vec![key_row(1, "pexels", true), key_row(2, "pexels", false), key_row(3, "pexels", true)],
            &[],
        );
        let picked: Vec<i64> = (0..4).map(|_| rt.pick_key("pexels").unwrap().id).collect();
        assert_eq!(picked, vec![1, 3, 1, 3]);
        assert!(rt.pick_key("unsplash").is_none());
    }

    #[test]
    fn resting_key_is_skipped_until_cooldown_ends() {
        let rt = runtime(vec![key_row(1, "pexels", true), key_row(2, "pexels", true)], &[]);
        rt.rest_key(1, Duration::from_secs(60));
        assert!((0..3).all(|_| rt.pick_key("pexels").unwrap().id == 2));

        rt.rest_key(2, Duration::from_secs(60));
        assert!(rt.pick_key("pexels").is_none());
        assert!(rt.has_keys("pexels"), "resting keys still count as configured");
        assert!(!rt.has_available_key("pexels"));
    }

    #[test]
    fn rate_limit_response_rests_the_key() {
        let rt = runtime(vec![key_row(1, "pexels", true)], &[]);
        rt.record_response(1, None, 429, Some(Duration::from_secs(30)));
        let health = rt.key_health(1);
        assert_eq!(health.failures, 1);
        assert!(health.cooldown_remaining().is_some());
        assert!(rt.pick_key("pexels").is_none());

        rt.record_response(1, None, 200, None);
        assert!(rt.pick_key("pexels").is_some(), "a success clears the cooldown");
    }

    #[test]
    fn settings_fall_back_to_defaults_and_are_clamped() {
        let rt = runtime(vec![], &[]);
        assert_eq!(
            rt.cache_settings(),
            CacheSettings { enabled: true, ttl_secs: 3600, max_entries: 1000 }
        );
        assert_eq!(rt.proxy_strategy(), ProxyStrategy::RoundRobin);
        assert!(!rt.uses_proxy("pexels"));

        let rt = runtime(
            vec![],
            &[
                ("cache.enabled", "false"),
                ("cache.ttl_secs", "99999999999"),
                ("proxy.strategy", "random"),
                ("provider.pexels.use_proxy", "true"),
            ],
        );
        let cache = rt.cache_settings();
        assert!(!cache.enabled);
        assert_eq!(cache.ttl_secs, MAX_CACHE_TTL_SECS);
        assert_eq!(rt.proxy_strategy(), ProxyStrategy::Random);
        assert!(rt.uses_proxy("pexels"));
    }

    #[test]
    fn cache_settings_validation() {
        let ok = CacheSettings { enabled: true, ttl_secs: 60, max_entries: 10 };
        assert!(ok.validate().is_ok());
        assert!(CacheSettings { ttl_secs: 0, ..ok }.validate().is_err());
        assert!(CacheSettings { ttl_secs: MAX_CACHE_TTL_SECS + 1, ..ok }.validate().is_err());
        assert!(CacheSettings { max_entries: 0, ..ok }.validate().is_err());
        assert!(CacheSettings { max_entries: MAX_CACHE_ENTRIES + 1, ..ok }.validate().is_err());
    }
}

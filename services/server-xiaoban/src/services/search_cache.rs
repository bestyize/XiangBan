//! Persistent cache of aggregated search results.
//!
//! Identical requests are answered from PostgreSQL instead of calling the providers again, which
//! spares their rate limits. Entries expire after a configurable lifetime, and when the entry
//! limit is exceeded the least recently used ones are evicted first.

use crate::{
    error::AppError,
    models::{SearchQuery, WallpaperItem},
    runtime::Runtime,
    store::Store,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSearch {
    pub items: Vec<WallpaperItem>,
    pub providers: Vec<String>,
    /// Decided before any filtering, so a cached short page still knows another one follows.
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    /// False when PostgreSQL is not configured, in which case nothing is cached.
    pub available: bool,
    pub entries: u64,
    pub bytes: u64,
    /// Requests answered from the cache since the server started.
    pub hits: u64,
    /// Requests that had to call the providers since the server started.
    pub misses: u64,
    pub oldest_age_secs: Option<u64>,
}

type InflightMap = Mutex<HashMap<String, Arc<AsyncMutex<()>>>>;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

struct Inner {
    store: Option<Store>,
    runtime: Runtime,
    hits: AtomicU64,
    misses: AtomicU64,
    inflight: Arc<InflightMap>,
    maintenance: AsyncMutex<()>,
}

#[derive(Clone)]
pub struct SearchCache {
    inner: Arc<Inner>,
}

/// Held while one request fetches a given query, so identical requests arriving meanwhile wait
/// for its result instead of each calling the providers.
pub struct Flight {
    map: Arc<InflightMap>,
    key: String,
    guard: Option<OwnedMutexGuard<()>>,
}

impl Drop for Flight {
    fn drop(&mut self) {
        let Some(guard) = self.guard.take() else {
            return;
        };
        let mut map = lock(&self.map);
        // The map and this guard hold one reference each; more means another request is waiting.
        if Arc::strong_count(OwnedMutexGuard::mutex(&guard)) == 2 {
            map.remove(&self.key);
        }
        drop(map);
        drop(guard);
    }
}

impl SearchCache {
    pub fn new(store: Option<Store>, runtime: Runtime) -> Self {
        Self {
            inner: Arc::new(Inner {
                store,
                runtime,
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
                inflight: Arc::new(Mutex::new(HashMap::new())),
                maintenance: AsyncMutex::new(()),
            }),
        }
    }

    /// Two requests share an entry only when every parameter is identical.
    pub fn key(query: &SearchQuery) -> String {
        // Bump the version whenever the shape or media URLs of cached items change, so old entries are never served.
        format!(
            "v4|q={}|category={}|media={:?}|orientation={:?}|provider={}|page={}|per_page={}|sort={}",
            query.get_q(),
            query.category.as_deref().unwrap_or(""),
            query.get_media_type(),
            query.get_orientation(),
            query.provider.as_deref().unwrap_or("all"),
            query.get_page(),
            query.get_per_page(),
            query.get_sort(),
        )
    }

    /// Whether results are currently being stored and served.
    pub fn is_active(&self) -> bool {
        self.inner.store.is_some() && self.inner.runtime.cache_settings().enabled
    }

    /// Returns a fresh entry and counts a hit. A storage error is treated as a miss, so a
    /// database problem slows searches down but never breaks them.
    pub async fn lookup(&self, key: &str) -> Option<CachedSearch> {
        if !self.is_active() {
            return None;
        }
        let store = self.inner.store.as_ref()?;
        let ttl = self.inner.runtime.cache_settings().ttl_secs;

        let payload = match store.cache_hit(key, ttl).await {
            Ok(Some(payload)) => payload,
            Ok(None) => return None,
            Err(e) => {
                warn!("Search cache read failed: {e}");
                return None;
            }
        };

        match serde_json::from_str(&payload) {
            Ok(found) => {
                self.inner.hits.fetch_add(1, Ordering::Relaxed);
                Some(found)
            }
            // An entry written by an older version; the fresh result will overwrite it.
            Err(e) => {
                warn!("Ignoring unreadable search cache entry: {e}");
                None
            }
        }
    }

    pub fn record_miss(&self) {
        self.inner.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Serializes concurrent fetches of the same query. `None` when caching is off, because then
    /// there would be no result to share.
    pub async fn flight(&self, key: &str) -> Option<Flight> {
        if !self.is_active() {
            return None;
        }
        let mutex = lock(&self.inner.inflight)
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone();
        let guard = mutex.lock_owned().await;
        Some(Flight {
            map: self.inner.inflight.clone(),
            key: key.to_string(),
            guard: Some(guard),
        })
    }

    pub async fn put(&self, key: &str, value: &CachedSearch) {
        if !self.is_active() {
            return;
        }
        let Some(store) = self.inner.store.as_ref() else {
            return;
        };
        let payload = match serde_json::to_string(value) {
            Ok(p) => p,
            Err(e) => {
                warn!("Could not serialize search result for caching: {e}");
                return;
            }
        };
        if let Err(e) = store.cache_put(key, &payload).await {
            warn!("Search cache write failed: {e}");
            return;
        }
        self.enforce_limits().await;
    }

    /// Drops expired entries, then the least recently used ones beyond the entry limit.
    pub async fn enforce_limits(&self) {
        let Some(store) = self.inner.store.as_ref() else {
            return;
        };
        let settings = self.inner.runtime.cache_settings();
        let _serialized = self.inner.maintenance.lock().await;

        let result: Result<(), AppError> = async {
            store.cache_delete_expired(settings.ttl_secs).await?;
            let totals = store.cache_totals().await?;
            if totals.entries > settings.max_entries {
                store.cache_delete_lru(totals.entries - settings.max_entries).await?;
            }
            Ok(())
        }
        .await;

        if let Err(e) = result {
            warn!("Search cache cleanup failed: {e}");
        }
    }

    pub async fn clear(&self) -> Result<u64, AppError> {
        match self.inner.store.as_ref() {
            Some(store) => store.cache_clear().await,
            None => Ok(0),
        }
    }

    pub async fn stats(&self) -> Result<CacheStats, AppError> {
        let hits = self.inner.hits.load(Ordering::Relaxed);
        let misses = self.inner.misses.load(Ordering::Relaxed);
        let Some(store) = self.inner.store.as_ref() else {
            return Ok(CacheStats {
                available: false,
                entries: 0,
                bytes: 0,
                hits,
                misses,
                oldest_age_secs: None,
            });
        };
        let totals = store.cache_totals().await?;
        Ok(CacheStats {
            available: true,
            entries: totals.entries,
            bytes: totals.bytes,
            hits,
            misses,
            oldest_age_secs: totals
                .oldest
                .map(|t| (chrono::Utc::now() - t).num_seconds().max(0) as u64),
        })
    }
}

use super::search_cache::{CachedSearch, SearchCache};
use crate::{
    db::Database,
    error::AppError,
    models::{
        category,
        query::{SearchQuery, SearchResult},
        MediaType, Orientation, WallpaperItem,
    },
    providers::ProviderRegistry,
};
use futures::future::join_all;
use std::{borrow::Cow, sync::Arc, time::Instant};
use tracing::{debug, error};

/// How long one provider may take, retries across keys and proxies included.
const PROVIDER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);

#[derive(Clone)]
pub struct WallpaperAggregator {
    registry: ProviderRegistry,
    db: Database,
    cache: SearchCache,
}

impl WallpaperAggregator {
    pub fn new(registry: ProviderRegistry, db: Database, cache: SearchCache) -> Self {
        Self { registry, db, cache }
    }

    /// A category without free text searches for the category's keywords.
    fn resolve_category(query: &SearchQuery) -> Result<Cow<'_, SearchQuery>, AppError> {
        let Some(id) = query.category.as_deref().filter(|c| !c.is_empty()) else {
            return Ok(Cow::Borrowed(query));
        };
        let category =
            category::find(id).ok_or_else(|| AppError::BadRequest(format!("Unknown category: {id}")))?;
        if !query.get_q().is_empty() {
            return Ok(Cow::Borrowed(query));
        }
        Ok(Cow::Owned(SearchQuery {
            q: Some(category.keyword.to_string()),
            ..query.clone()
        }))
    }

    fn from_cache(hit: CachedSearch, query: &SearchQuery, started: Instant) -> SearchResult {
        SearchResult {
            total: hit.items.len(),
            page: query.get_page(),
            per_page: query.get_per_page(),
            has_more: hit.has_more,
            items: hit.items,
            providers: vec!["cache".to_string()],
            execution_time_ms: started.elapsed().as_millis() as u64,
        }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResult, AppError> {
        let started = Instant::now();
        let query = Self::resolve_category(query)?;
        let query: &SearchQuery = &query;
        let cache_key = SearchCache::key(query);

        if let Some(hit) = self.cache.lookup(&cache_key).await {
            debug!("Cache hit for key: {}", cache_key);
            return Ok(Self::from_cache(hit, query, started));
        }

        // Identical requests arriving while this one fetches wait here and then find the result cached.
        let _flight = self.cache.flight(&cache_key).await;
        if _flight.is_some() {
            if let Some(hit) = self.cache.lookup(&cache_key).await {
                debug!("Cache hit after waiting for key: {}", cache_key);
                return Ok(Self::from_cache(hit, query, started));
            }
        }
        self.cache.record_miss();

        let target_provider = query.provider.as_deref().unwrap_or("all");
        let providers_to_query = if target_provider != "all" && !target_provider.is_empty() {
            if let Some(p) = self.registry.get(target_provider) {
                vec![p]
            } else {
                return Err(AppError::BadRequest(format!(
                    "Unknown provider: {target_provider}"
                )));
            }
        } else {
            // Filter providers that are configured or demo
            self.registry
                .all()
                .iter()
                .filter(|p| p.is_configured())
                .cloned()
                .collect()
        };

        let mut provider_names = Vec::new();
        let mut futures = Vec::new();

        let req_media = query.get_media_type();

        for provider in &providers_to_query {
            // Check if provider supports requested media type
            if !provider.supported_media_types().contains(&req_media) {
                continue;
            }

            provider_names.push(provider.name().to_string());
            let prov = Arc::clone(provider);
            let q_clone = query.clone();

            futures.push(async move {
                match tokio::time::timeout(PROVIDER_TIMEOUT, prov.search(&q_clone)).await {
                    Ok(Ok(items)) => (items, true),
                    Ok(Err(e)) => {
                        error!("Provider {} error: {}", prov.name(), e);
                        (Vec::new(), false)
                    }
                    Err(_) => {
                        error!("Provider {} timed out", prov.name());
                        (Vec::new(), false)
                    }
                }
            });
        }

        let outcomes = join_all(futures).await;
        let all_succeeded = outcomes.iter().all(|(_, ok)| *ok);
        let results_by_provider = outcomes.into_iter().map(|(items, _)| items).collect();

        // Interleave and deduplicate results
        let mut interleaved = self.interleave_results(results_by_provider);

        // Whether another page exists is judged before filtering: a page that the orientation filter
        // thins out is still followed by more pages from the providers.
        let has_more = interleaved.len() >= query.get_per_page() as usize;

        // Orientation filtering
        let wanted = query.get_orientation();
        if wanted != Orientation::All {
            interleaved.retain(|item| item.orientation == wanted);
        }

        // A result that is missing a provider's share because of an error or timeout must not be
        // kept: it would be served as if complete until it expires.
        if all_succeeded && !provider_names.is_empty() {
            self.cache
                .put(
                    &cache_key,
                    &CachedSearch {
                        items: interleaved.clone(),
                        providers: provider_names.clone(),
                        has_more,
                    },
                )
                .await;
        }

        Ok(SearchResult {
            total: interleaved.len(),
            page: query.get_page(),
            per_page: query.get_per_page(),
            has_more,
            items: interleaved,
            providers: provider_names,
            execution_time_ms: started.elapsed().as_millis() as u64,
        })
    }

    pub async fn curated(&self, page: u32, per_page: u32) -> Result<SearchResult, AppError> {
        let query = SearchQuery {
            q: None,
            category: None,
            media_type: Some(MediaType::Image),
            orientation: None,
            provider: None,
            page: Some(page),
            per_page: Some(per_page),
            sort: Some("popular".to_string()),
        };
        self.search(&query).await
    }

    pub async fn videos(&self, page: u32, per_page: u32, q: Option<String>) -> Result<SearchResult, AppError> {
        let query = SearchQuery {
            q,
            category: None,
            media_type: Some(MediaType::Video),
            orientation: None,
            provider: None,
            page: Some(page),
            per_page: Some(per_page),
            sort: Some("popular".to_string()),
        };
        self.search(&query).await
    }

    pub async fn detail(&self, provider_id: &str, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        // Check DB first
        let full_id = if id.starts_with(provider_id) {
            id.to_string()
        } else {
            format!("{provider_id}_{id}")
        };

        if let Ok(Some(item)) = self.db.get_wallpaper(&full_id).await {
            return Ok(Some(item));
        }

        // Fallback to provider
        if let Some(prov) = self.registry.get(provider_id) {
            let res = prov.detail(id).await?;
            if let Some(item) = &res {
                let _ = self.db.upsert_wallpapers(&[item.clone()]).await;
            }
            return Ok(res);
        }

        Ok(None)
    }

    fn interleave_results(&self, mut provider_results: Vec<Vec<WallpaperItem>>) -> Vec<WallpaperItem> {
        let mut interleaved = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        let max_len = provider_results.iter().map(|v| v.len()).max().unwrap_or(0);

        for i in 0..max_len {
            for list in &mut provider_results {
                if i < list.len() {
                    let item = &list[i];
                    if seen_ids.insert(item.id.clone()) {
                        interleaved.push(item.clone());
                    }
                }
            }
        }

        interleaved
    }
}

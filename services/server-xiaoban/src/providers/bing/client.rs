use super::dto::{BingImage, BingResponse};
use crate::{error::AppError, runtime::Upstream};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

pub const BASE_URL: &str = "https://www.bing.com";
pub const DEFAULT_MARKET: &str = "zh-CN";
const CACHE_TTL: Duration = Duration::from_secs(600);

fn provider_error(message: impl Into<String>) -> AppError {
    AppError::Provider {
        provider: "bing",
        message: message.into(),
    }
}

#[derive(Clone)]
pub struct BingClient {
    upstream: Upstream,
    base_url: String,
    cache: Arc<RwLock<Option<(Instant, Vec<BingImage>)>>>,
}

impl BingClient {
    pub fn new(upstream: Upstream) -> Self {
        Self {
            upstream,
            base_url: BASE_URL.to_string(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(upstream: Upstream, base_url: String) -> Self {
        Self {
            upstream,
            base_url,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn is_configured(&self) -> bool {
        true
    }

    /// Fetches a batch of Bing images from the HPImageArchive endpoint.
    pub async fn fetch_batch(&self, idx: u32, n: u32, mkt: &str) -> Result<Vec<BingImage>, AppError> {
        let n_clamped = n.clamp(1, 8);
        let url = format!(
            "{}/HPImageArchive.aspx?format=js&idx={}&n={}&mkt={}",
            self.base_url.trim_end_matches('/'),
            idx,
            n_clamped,
            mkt
        );

        let response = self
            .upstream
            .send_public(|client| client.get(&url).header("Accept", "application/json"))
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(provider_error(format!(
                "Bing returned HTTP status {status} for {url}"
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| provider_error(format!("Failed to read Bing response body: {e}")))?;

        let parsed: BingResponse = serde_json::from_str(&body)
            .map_err(|e| provider_error(format!("Failed to parse Bing response JSON: {e}")))?;

        Ok(parsed.images)
    }

    /// Gets recent Bing images across recent days, using in-memory cache to minimize upstream requests.
    pub async fn get_archive_images(&self) -> Result<Vec<BingImage>, AppError> {
        {
            let cache_lock = self.cache.read().await;
            if let Some((ts, items)) = &*cache_lock {
                if ts.elapsed() < CACHE_TTL {
                    return Ok(items.clone());
                }
            }
        }

        // Fetch today (idx=0) and earlier days (idx=8)
        let first_batch = self.fetch_batch(0, 8, DEFAULT_MARKET).await;

        let items = match first_batch {
            Ok(mut batch_0) => {
                // Best-effort fetch for historical batch (idx=8)
                if let Ok(batch_8) = self.fetch_batch(8, 8, DEFAULT_MARKET).await {
                    for img in batch_8 {
                        let is_duplicate = batch_0.iter().any(|existing| {
                            existing.urlbase == img.urlbase
                                || (existing.hsh.is_some() && existing.hsh == img.hsh)
                        });
                        if !is_duplicate {
                            batch_0.push(img);
                        }
                    }
                }
                batch_0
            }
            Err(err) => {
                // If network failed but we have stale cache, serve stale cache as fallback
                let cache_lock = self.cache.read().await;
                if let Some((_, items)) = &*cache_lock {
                    return Ok(items.clone());
                }
                return Err(err);
            }
        };

        {
            let mut cache_lock = self.cache.write().await;
            *cache_lock = Some((Instant::now(), items.clone()));
        }

        Ok(items)
    }
}

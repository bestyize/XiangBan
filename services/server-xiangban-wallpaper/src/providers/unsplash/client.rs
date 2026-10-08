use super::dto::{UnsplashPhoto, UnsplashSearchResponse};
use crate::{error::AppError, models::Orientation, runtime::Upstream};
use reqwest::header::AUTHORIZATION;

#[derive(Clone)]
pub struct UnsplashClient {
    upstream: Upstream,
    base_url: String,
}

fn provider_error(message: String) -> AppError {
    AppError::Provider {
        provider: "unsplash",
        message,
    }
}

impl UnsplashClient {
    pub fn new(upstream: Upstream) -> Self {
        Self {
            upstream,
            base_url: "https://api.unsplash.com".to_string(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.upstream.is_configured()
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response, AppError> {
        self.upstream
            .send(|client, key| {
                client
                    .get(url)
                    .header(AUTHORIZATION, format!("Client-ID {}", key.trim()))
                    .header("Accept-Version", "v1")
            })
            .await
    }

    pub async fn search(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
        orientation: &Orientation,
    ) -> Result<Vec<UnsplashPhoto>, AppError> {
        let mut url = format!(
            "{}/search/photos?query={}&page={}&per_page={}",
            self.base_url,
            urlencoding::encode(query),
            page,
            per_page.min(30)
        );

        match orientation {
            Orientation::Landscape => url.push_str("&orientation=landscape"),
            Orientation::Portrait => url.push_str("&orientation=portrait"),
            Orientation::Squarish => url.push_str("&orientation=squarish"),
            Orientation::All => {}
        }

        let res = self.get(&url).await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(provider_error(format!("API returned {status}: {text}")));
        }

        let data: UnsplashSearchResponse = res
            .json()
            .await
            .map_err(|e| provider_error(format!("Failed to parse response: {e}")))?;

        Ok(data.results)
    }

    pub async fn curated(&self, page: u32, per_page: u32) -> Result<Vec<UnsplashPhoto>, AppError> {
        let url = format!(
            "{}/photos?page={}&per_page={}&order_by=popular",
            self.base_url,
            page,
            per_page.min(30)
        );

        let res = self.get(&url).await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(provider_error(format!("API returned {status}: {text}")));
        }

        res.json()
            .await
            .map_err(|e| provider_error(format!("Failed to parse response: {e}")))
    }

    pub async fn detail(&self, id: &str) -> Result<Option<UnsplashPhoto>, AppError> {
        let url = format!("{}/photos/{}", self.base_url, urlencoding::encode(id));

        let res = self.get(&url).await?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(provider_error(format!("API returned {status}: {text}")));
        }

        let photo: UnsplashPhoto = res
            .json()
            .await
            .map_err(|e| provider_error(format!("Failed to parse detail: {e}")))?;

        Ok(Some(photo))
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::new();
        for b in s.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(b as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", b));
                }
            }
        }
        encoded
    }
}

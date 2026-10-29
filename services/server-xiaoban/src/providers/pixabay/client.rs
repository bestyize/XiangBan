use super::dto::{PixabayImageHit, PixabayImageResponse, PixabayVideoHit, PixabayVideoResponse};
use crate::{error::AppError, models::Orientation, runtime::Upstream};

#[derive(Clone)]
pub struct PixabayClient {
    upstream: Upstream,
    image_base_url: String,
    video_base_url: String,
}

fn provider_error(message: String) -> AppError {
    AppError::Provider {
        provider: "pixabay",
        message,
    }
}

impl PixabayClient {
    pub fn new(upstream: Upstream) -> Self {
        Self {
            upstream,
            image_base_url: "https://pixabay.com/api/".to_string(),
            video_base_url: "https://pixabay.com/api/videos/".to_string(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.upstream.is_configured()
    }

    /// Pixabay takes its key as a query parameter, added per attempt so key rotation can swap it.
    async fn get(&self, url: &str) -> Result<reqwest::Response, AppError> {
        self.upstream
            .send(|client, key| client.get(url).query(&[("key", key)]))
            .await
    }

    pub async fn search_images(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
        orientation: &Orientation,
    ) -> Result<Vec<PixabayImageHit>, AppError> {
        let mut url = format!(
            "{}?image_type=photo&safesearch=true&page={}&per_page={}",
            self.image_base_url,
            page,
            per_page.clamp(3, 100)
        );

        if !query.is_empty() {
            url.push_str(&format!("&q={}", urlencoding::encode(query)));
        } else {
            url.push_str("&editors_choice=true&order=popular");
        }

        match orientation {
            Orientation::Landscape => url.push_str("&orientation=horizontal"),
            Orientation::Portrait => url.push_str("&orientation=vertical"),
            Orientation::All | Orientation::Squarish => url.push_str("&orientation=all"),
        }

        let res = self.get(&url).await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(provider_error(format!("API returned {status}: {text}")));
        }

        let data: PixabayImageResponse = res
            .json()
            .await
            .map_err(|e| provider_error(format!("Failed to parse image response: {e}")))?;

        Ok(data.hits)
    }

    pub async fn search_videos(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<PixabayVideoHit>, AppError> {
        let mut url = format!(
            "{}?safesearch=true&page={}&per_page={}&order=popular",
            self.video_base_url,
            page,
            per_page.clamp(3, 100)
        );

        if !query.is_empty() {
            url.push_str(&format!("&q={}", urlencoding::encode(query)));
        }

        let res = self.get(&url).await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(provider_error(format!("API returned {status}: {text}")));
        }

        let data: PixabayVideoResponse = res
            .json()
            .await
            .map_err(|e| provider_error(format!("Failed to parse video response: {e}")))?;

        Ok(data.hits)
    }

    pub async fn detail_image(&self, id: &str) -> Result<Option<PixabayImageHit>, AppError> {
        let url = format!("{}?id={}", self.image_base_url, urlencoding::encode(id));
        let res = self.get(&url).await?;

        if !res.status().is_success() {
            return Ok(None);
        }

        let data: PixabayImageResponse = res
            .json()
            .await
            .map_err(|e| provider_error(format!("Failed to parse image detail: {e}")))?;

        Ok(data.hits.into_iter().next())
    }

    pub async fn detail_video(&self, id: &str) -> Result<Option<PixabayVideoHit>, AppError> {
        let url = format!("{}?id={}", self.video_base_url, urlencoding::encode(id));
        let res = self.get(&url).await?;

        if !res.status().is_success() {
            return Ok(None);
        }

        let data: PixabayVideoResponse = res
            .json()
            .await
            .map_err(|e| provider_error(format!("Failed to parse video detail: {e}")))?;

        Ok(data.hits.into_iter().next())
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

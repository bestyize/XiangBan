use super::dto::{PexelsPhoto, PexelsPhotoResponse, PexelsVideo, PexelsVideoResponse};
use crate::{error::AppError, models::Orientation, runtime::Upstream};
use reqwest::header::AUTHORIZATION;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct PexelsClient {
    upstream: Upstream,
    base_url: String,
}

impl PexelsClient {
    pub fn new(upstream: Upstream) -> Self {
        Self {
            upstream,
            base_url: "https://api.pexels.com".to_string(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.upstream.is_configured()
    }

    fn err(message: String) -> AppError {
        AppError::Provider {
            provider: "pexels",
            message,
        }
    }

    /// Returns `Ok(None)` on 404, otherwise the parsed JSON body.
    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<Option<T>, AppError> {
        let res = self
            .upstream
            .send(|client, key| client.get(url).header(AUTHORIZATION, key.trim()))
            .await?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(Self::err(format!("API returned {status}: {text}")));
        }

        res.json()
            .await
            .map(Some)
            .map_err(|e| Self::err(format!("Failed to parse response: {e}")))
    }

    fn orientation_param(orientation: &Orientation) -> &'static str {
        match orientation {
            Orientation::Landscape => "&orientation=landscape",
            Orientation::Portrait => "&orientation=portrait",
            Orientation::Squarish => "&orientation=square",
            Orientation::All => "",
        }
    }

    pub async fn search_photos(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
        orientation: &Orientation,
    ) -> Result<Vec<PexelsPhoto>, AppError> {
        let url = format!(
            "{}/v1/search?query={}&page={}&per_page={}{}",
            self.base_url,
            urlencoding::encode(query),
            page,
            per_page.min(80),
            Self::orientation_param(orientation)
        );
        let data: Option<PexelsPhotoResponse> = self.get(&url).await?;
        Ok(data.map(|d| d.photos).unwrap_or_default())
    }

    pub async fn curated_photos(&self, page: u32, per_page: u32) -> Result<Vec<PexelsPhoto>, AppError> {
        let url = format!(
            "{}/v1/curated?page={}&per_page={}",
            self.base_url,
            page,
            per_page.min(80)
        );
        let data: Option<PexelsPhotoResponse> = self.get(&url).await?;
        Ok(data.map(|d| d.photos).unwrap_or_default())
    }

    pub async fn detail_photo(&self, id: &str) -> Result<Option<PexelsPhoto>, AppError> {
        let url = format!("{}/v1/photos/{}", self.base_url, urlencoding::encode(id));
        self.get(&url).await
    }

    pub async fn search_videos(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
        orientation: &Orientation,
    ) -> Result<Vec<PexelsVideo>, AppError> {
        // The popular list has no orientation option, so a filter needs the search endpoint.
        let url = if query.trim().is_empty() && *orientation == Orientation::All {
            format!(
                "{}/videos/popular?page={}&per_page={}",
                self.base_url,
                page,
                per_page.min(80)
            )
        } else {
            let keyword = if query.trim().is_empty() { "wallpaper" } else { query };
            format!(
                "{}/videos/search?query={}&page={}&per_page={}{}",
                self.base_url,
                urlencoding::encode(keyword),
                page,
                per_page.min(80),
                Self::orientation_param(orientation)
            )
        };
        let data: Option<PexelsVideoResponse> = self.get(&url).await?;
        Ok(data.map(|d| d.videos).unwrap_or_default())
    }

    pub async fn detail_video(&self, id: &str) -> Result<Option<PexelsVideo>, AppError> {
        let url = format!("{}/videos/videos/{}", self.base_url, urlencoding::encode(id));
        self.get(&url).await
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

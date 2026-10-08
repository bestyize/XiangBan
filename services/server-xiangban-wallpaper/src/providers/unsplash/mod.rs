pub mod client;
pub mod dto;
pub mod mapper;

use async_trait::async_trait;
use crate::{
    error::AppError,
    models::{MediaType, Orientation, ProviderInfo, SearchQuery, WallpaperItem},
    providers::traits::WallpaperProvider,
    runtime::Runtime,
};
use client::UnsplashClient;
use mapper::map_unsplash_photo;

/// Searched when the user filters by orientation without typing a keyword.
const FILTER_KEYWORD: &str = "wallpaper";

pub struct UnsplashProvider {
    client: UnsplashClient,
}

impl UnsplashProvider {
    pub fn new(runtime: &Runtime) -> Self {
        Self {
            client: UnsplashClient::new(runtime.upstream("unsplash")),
        }
    }
}

#[async_trait]
impl WallpaperProvider for UnsplashProvider {
    fn id(&self) -> &'static str {
        "unsplash"
    }

    fn name(&self) -> &'static str {
        "Unsplash"
    }

    fn is_configured(&self) -> bool {
        self.client.is_configured()
    }

    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image]
    }

    fn provider_info(&self) -> ProviderInfo {
        let is_conf = self.is_configured();
        ProviderInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            description: "Over 3 million free high-resolution photos and wallpapers shared by photographers worldwide.".to_string(),
            is_configured: is_conf,
            supported_media_types: self.supported_media_types(),
            official_url: "https://unsplash.com".to_string(),
            status: if is_conf { "ready".to_string() } else { "missing_api_key".to_string() },
        }
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        let q = query.get_q();
        let orientation = query.get_orientation();
        // The photo list has no orientation option, so a filter needs the search endpoint.
        let photos = if q.is_empty() && orientation == Orientation::All {
            self.client.curated(query.get_page(), query.get_per_page()).await?
        } else {
            let keyword = if q.is_empty() { FILTER_KEYWORD } else { q };
            self.client
                .search(keyword, query.get_page(), query.get_per_page(), &orientation)
                .await?
        };

        Ok(photos.into_iter().map(map_unsplash_photo).collect())
    }

    async fn curated(&self, page: u32, per_page: u32) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        let photos = self.client.curated(page, per_page).await?;
        Ok(photos.into_iter().map(map_unsplash_photo).collect())
    }

    async fn videos(&self, _query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        // Unsplash does not host video media natively
        Ok(Vec::new())
    }

    async fn detail(&self, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(None);
        }

        let photo = self.client.detail(id).await?;
        Ok(photo.map(map_unsplash_photo))
    }
}

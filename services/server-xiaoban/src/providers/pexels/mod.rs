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
use client::PexelsClient;
use mapper::{map_pexels_photo, map_pexels_video};

/// Searched when the user filters by orientation without typing a keyword.
const FILTER_KEYWORD: &str = "wallpaper";

pub struct PexelsProvider {
    client: PexelsClient,
}

impl PexelsProvider {
    pub fn new(runtime: &Runtime) -> Self {
        Self {
            client: PexelsClient::new(runtime.upstream("pexels")),
        }
    }
}

#[async_trait]
impl WallpaperProvider for PexelsProvider {
    fn id(&self) -> &'static str {
        "pexels"
    }

    fn name(&self) -> &'static str {
        "Pexels"
    }

    fn is_configured(&self) -> bool {
        self.client.is_configured()
    }

    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image, MediaType::Video]
    }

    fn provider_info(&self) -> ProviderInfo {
        let is_conf = self.is_configured();
        ProviderInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            description: "Free high-quality stock photos and HD/4K videos contributed by a global creator community.".to_string(),
            is_configured: is_conf,
            supported_media_types: self.supported_media_types(),
            official_url: "https://www.pexels.com".to_string(),
            status: if is_conf { "ready".to_string() } else { "missing_api_key".to_string() },
        }
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        if query.get_media_type() == MediaType::Video {
            return self.videos(query).await;
        }

        let q = query.get_q();
        let orientation = query.get_orientation();
        // The curated feed has no orientation option, so a filter needs the search endpoint.
        let photos = if q.is_empty() && orientation == Orientation::All {
            self.client.curated_photos(query.get_page(), query.get_per_page()).await?
        } else {
            let keyword = if q.is_empty() { FILTER_KEYWORD } else { q };
            self.client
                .search_photos(keyword, query.get_page(), query.get_per_page(), &orientation)
                .await?
        };

        Ok(photos.into_iter().map(map_pexels_photo).collect())
    }

    async fn curated(&self, page: u32, per_page: u32) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        let photos = self.client.curated_photos(page, per_page).await?;
        Ok(photos.into_iter().map(map_pexels_photo).collect())
    }

    async fn videos(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        let videos = self
            .client
            .search_videos(
                query.get_q(),
                query.get_page(),
                query.get_per_page(),
                &query.get_orientation(),
            )
            .await?;
        Ok(videos.into_iter().map(map_pexels_video).collect())
    }

    async fn detail(&self, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(None);
        }

        if let Some(video_id) = id.strip_prefix("v_") {
            let video = self.client.detail_video(video_id).await?;
            Ok(video.map(map_pexels_video))
        } else {
            let photo = self.client.detail_photo(id).await?;
            Ok(photo.map(map_pexels_photo))
        }
    }
}

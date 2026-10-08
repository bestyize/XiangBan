pub mod client;
pub mod dto;
pub mod mapper;

use async_trait::async_trait;
use crate::{
    error::AppError,
    models::{MediaType, ProviderInfo, SearchQuery, WallpaperItem},
    providers::traits::WallpaperProvider,
    runtime::Runtime,
};
use client::PixabayClient;
use mapper::{map_pixabay_image, map_pixabay_video};

pub struct PixabayProvider {
    client: PixabayClient,
}

impl PixabayProvider {
    pub fn new(runtime: &Runtime) -> Self {
        Self {
            client: PixabayClient::new(runtime.upstream("pixabay")),
        }
    }
}

#[async_trait]
impl WallpaperProvider for PixabayProvider {
    fn id(&self) -> &'static str {
        "pixabay"
    }

    fn name(&self) -> &'static str {
        "Pixabay"
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
            description: "Over 4 million high quality stock images, illustrations, and 4K/HD video wallpapers.".to_string(),
            is_configured: is_conf,
            supported_media_types: self.supported_media_types(),
            official_url: "https://pixabay.com".to_string(),
            status: if is_conf { "ready".to_string() } else { "missing_api_key".to_string() },
        }
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        match query.get_media_type() {
            MediaType::Video => {
                let videos = self
                    .client
                    .search_videos(query.get_q(), query.get_page(), query.get_per_page())
                    .await?;
                Ok(videos.into_iter().map(map_pixabay_video).collect())
            }
            _ => {
                let images = self
                    .client
                    .search_images(
                        query.get_q(),
                        query.get_page(),
                        query.get_per_page(),
                        &query.get_orientation(),
                    )
                    .await?;
                Ok(images.into_iter().map(map_pixabay_image).collect())
            }
        }
    }

    async fn curated(&self, page: u32, per_page: u32) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        let images = self
            .client
            .search_images("", page, per_page, &crate::models::Orientation::All)
            .await?;
        Ok(images.into_iter().map(map_pixabay_image).collect())
    }

    async fn videos(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(Vec::new());
        }

        let videos = self
            .client
            .search_videos(query.get_q(), query.get_page(), query.get_per_page())
            .await?;
        Ok(videos.into_iter().map(map_pixabay_video).collect())
    }

    async fn detail(&self, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        if !self.is_configured() {
            return Ok(None);
        }

        if let Some(video_id) = id.strip_prefix("v_") {
            let hit = self.client.detail_video(video_id).await?;
            Ok(hit.map(map_pixabay_video))
        } else {
            let hit = self.client.detail_image(id).await?;
            Ok(hit.map(map_pixabay_image))
        }
    }
}

pub mod client;
pub mod dto;
pub mod mapper;

use crate::{
    error::AppError,
    models::{MediaType, Orientation, ProviderInfo, SearchQuery, WallpaperItem},
    providers::traits::WallpaperProvider,
    runtime::Runtime,
};
use async_trait::async_trait;
use client::BingClient;
use mapper::{extract_source_id, map_bing_image};

pub struct BingProvider {
    client: BingClient,
}

impl BingProvider {
    pub fn new(runtime: &Runtime) -> Self {
        Self {
            client: BingClient::new(runtime.upstream("bing")),
        }
    }
}

#[async_trait]
impl WallpaperProvider for BingProvider {
    fn id(&self) -> &'static str {
        "bing"
    }

    fn name(&self) -> &'static str {
        "Bing"
    }

    fn is_configured(&self) -> bool {
        // Bing official wallpaper archive is open and public, always ready to collect.
        true
    }

    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image]
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            description: "Official Bing Daily Wallpaper archive (Ultra HD 4K & 1080p curated daily photos).".to_string(),
            is_configured: true,
            supported_media_types: self.supported_media_types(),
            official_url: "https://www.bing.com".to_string(),
            status: "ready".to_string(),
        }
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        // Bing only provides images
        if let Some(media) = query.media_type {
            if media != MediaType::Image {
                return Ok(Vec::new());
            }
        }

        // Bing wallpapers are Landscape 16:9
        let orientation = query.get_orientation();
        if orientation == Orientation::Portrait || orientation == Orientation::Squarish {
            return Ok(Vec::new());
        }

        let images = self.client.get_archive_images().await?;
        let items: Vec<WallpaperItem> = images.into_iter().map(map_bing_image).collect();

        let q = query.get_q().to_lowercase();
        let filtered: Vec<WallpaperItem> = if q.is_empty() {
            items
        } else {
            items
                .into_iter()
                .filter(|item| {
                    let title = item.title.to_lowercase();
                    let desc = item
                        .description
                        .as_deref()
                        .map(str::to_lowercase)
                        .unwrap_or_default();
                    q.split_whitespace().any(|word| {
                        title.contains(word)
                            || desc.contains(word)
                            || item.tags.iter().any(|t| t.to_lowercase().contains(word))
                    })
                })
                .collect()
        };

        let page = query.get_page().max(1) as usize;
        let per_page = query.get_per_page().max(1) as usize;
        let skip = (page - 1) * per_page;

        Ok(filtered.into_iter().skip(skip).take(per_page).collect())
    }

    async fn curated(&self, page: u32, per_page: u32) -> Result<Vec<WallpaperItem>, AppError> {
        let images = self.client.get_archive_images().await?;
        let items: Vec<WallpaperItem> = images.into_iter().map(map_bing_image).collect();

        let page_idx = page.max(1) as usize;
        let per_page_idx = per_page.max(1) as usize;
        let skip = (page_idx - 1) * per_page_idx;

        Ok(items.into_iter().skip(skip).take(per_page_idx).collect())
    }

    async fn videos(&self, _query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        // Bing only provides still images
        Ok(Vec::new())
    }

    async fn detail(&self, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        let clean_id = id.strip_prefix("bing_").unwrap_or(id);

        let images = self.client.get_archive_images().await?;
        for img in images {
            if extract_source_id(&img) == clean_id {
                return Ok(Some(map_bing_image(img)));
            }
        }

        // If not in recent archive but is a valid OHR identifier, construct detail item directly
        if clean_id.starts_with("OHR.") {
            let img = dto::BingImage {
                startdate: None,
                fullstartdate: None,
                enddate: None,
                url: format!("/th?id={clean_id}_1920x1080.jpg&pid=hp"),
                urlbase: Some(format!("/th?id={clean_id}")),
                copyright: None,
                copyrightlink: None,
                title: None,
                hsh: None,
                wp: None,
            };
            return Ok(Some(map_bing_image(img)));
        }

        Ok(None)
    }
}

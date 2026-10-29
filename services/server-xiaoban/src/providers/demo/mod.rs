pub mod data;

use async_trait::async_trait;
use crate::{
    error::AppError,
    models::{MediaType, Orientation, ProviderInfo, SearchQuery, WallpaperItem},
    providers::traits::WallpaperProvider,
};
use data::get_demo_wallpapers;

pub struct DemoProvider;

impl DemoProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DemoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WallpaperProvider for DemoProvider {
    fn id(&self) -> &'static str {
        "demo"
    }

    fn name(&self) -> &'static str {
        "Featured / Local Demo"
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Image, MediaType::Video]
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            description: "Built-in curated showcase wallpapers and 4K live video wallpapers.".to_string(),
            is_configured: true,
            supported_media_types: self.supported_media_types(),
            official_url: "https://github.com/bestyize/WallpaperCollect".to_string(),
            status: "ready".to_string(),
        }
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        let all = get_demo_wallpapers();
        let q = query.get_q().to_lowercase();
        let target_media = query.get_media_type();
        let target_orient = query.get_orientation();

        let filtered = all
            .into_iter()
            .filter(|item| {
                // Media type filter
                if query.media_type.is_some() && item.media_type != target_media {
                    return false;
                }
                // Orientation filter
                if target_orient != Orientation::All && item.orientation != target_orient {
                    return false;
                }
                // Keyword match: any word of the query is enough, so multi-word category
                // keywords such as "galaxy stars space" still find something.
                if !q.is_empty() {
                    let title = item.title.to_lowercase();
                    let description = item.description.as_deref().map(str::to_lowercase).unwrap_or_default();
                    return q.split_whitespace().any(|word| {
                        title.contains(word)
                            || item.tags.iter().any(|t| t.contains(word))
                            || description.contains(word)
                    });
                }
                true
            })
            .collect();

        Ok(filtered)
    }

    async fn curated(&self, _page: u32, _per_page: u32) -> Result<Vec<WallpaperItem>, AppError> {
        Ok(get_demo_wallpapers())
    }

    async fn videos(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError> {
        let all = get_demo_wallpapers();
        let q = query.get_q().to_lowercase();

        let filtered = all
            .into_iter()
            .filter(|item| {
                if item.media_type != MediaType::Video {
                    return false;
                }
                if !q.is_empty() {
                    return item.title.to_lowercase().contains(&q)
                        || item.tags.iter().any(|t| t.contains(&q));
                }
                true
            })
            .collect();

        Ok(filtered)
    }

    async fn detail(&self, id: &str) -> Result<Option<WallpaperItem>, AppError> {
        let all = get_demo_wallpapers();
        let item = all.into_iter().find(|i| i.source_id == id || i.id == id);
        Ok(item)
    }
}

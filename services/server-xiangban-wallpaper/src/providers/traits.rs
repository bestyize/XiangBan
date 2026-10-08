use crate::{
    error::AppError,
    models::{MediaType, ProviderInfo, SearchQuery, WallpaperItem},
};
use async_trait::async_trait;

#[async_trait]
pub trait WallpaperProvider: Send + Sync {
    /// Unique provider identifier (e.g. "unsplash", "pixabay")
    fn id(&self) -> &'static str;

    /// Human readable name
    fn name(&self) -> &'static str;

    /// Whether this provider has valid credentials / API keys configured
    fn is_configured(&self) -> bool;

    /// Which media types are supported (Images, Videos, etc.)
    fn supported_media_types(&self) -> Vec<MediaType>;

    /// General provider metadata and status
    fn provider_info(&self) -> ProviderInfo;

    /// Search wallpapers with unified query parameters
    async fn search(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError>;

    /// Get curated / trending / editorial wallpapers
    #[allow(dead_code)]
    async fn curated(&self, page: u32, per_page: u32) -> Result<Vec<WallpaperItem>, AppError>;

    /// Search specifically for video wallpapers / live wallpapers
    async fn videos(&self, query: &SearchQuery) -> Result<Vec<WallpaperItem>, AppError>;

    /// Get single wallpaper details by provider-specific source ID
    async fn detail(&self, id: &str) -> Result<Option<WallpaperItem>, AppError>;
}

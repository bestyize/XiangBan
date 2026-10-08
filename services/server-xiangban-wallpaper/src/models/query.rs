use super::wallpaper::{MediaType, Orientation, WallpaperItem};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub media_type: Option<MediaType>,
    #[serde(default)]
    pub orientation: Option<Orientation>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub per_page: Option<u32>,
    #[serde(default)]
    pub sort: Option<String>,
}

impl SearchQuery {
    pub fn get_q(&self) -> &str {
        self.q.as_deref().unwrap_or("").trim()
    }

    pub fn get_page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> u32 {
        self.per_page.unwrap_or(24).clamp(1, 100)
    }

    pub fn get_orientation(&self) -> Orientation {
        self.orientation.unwrap_or(Orientation::All)
    }

    pub fn get_media_type(&self) -> MediaType {
        self.media_type.unwrap_or(MediaType::Image)
    }

    pub fn get_sort(&self) -> &str {
        self.sort.as_deref().unwrap_or("popular")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub items: Vec<WallpaperItem>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
    pub has_more: bool,
    pub providers: Vec<String>,
    pub execution_time_ms: u64,
}

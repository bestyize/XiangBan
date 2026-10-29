use super::wallpaper::MediaType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_configured: bool,
    pub supported_media_types: Vec<MediaType>,
    pub official_url: String,
    pub status: String, // "ready", "missing_api_key", "error"
}

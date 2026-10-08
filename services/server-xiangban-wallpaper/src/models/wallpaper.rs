use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    LivePhoto,
}

impl Default for MediaType {
    fn default() -> Self {
        Self::Image
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    All,
    Landscape,
    Portrait,
    Squarish,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::All
    }
}

impl Orientation {
    pub fn from_dimensions(width: u32, height: u32) -> Self {
        if width == 0 || height == 0 {
            return Self::All;
        }
        let ratio = width as f32 / height as f32;
        if ratio > 1.2 {
            Self::Landscape
        } else if ratio < 0.85 {
            Self::Portrait
        } else {
            Self::Squarish
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: String,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        let aspect_ratio = if width > 0 && height > 0 {
            let gcd = num_gcd(width, height);
            let w = width / gcd;
            let h = height / gcd;
            // Simplify common ratios
            if (w == 16 && h == 9) || (w == 4 && h == 3) || (w == 21 && h == 9) || (w == 9 && h == 16) {
                format!("{w}:{h}")
            } else {
                let ratio = (width as f32 / height as f32 * 100.0).round() / 100.0;
                format!("{ratio:.2}:1")
            }
        } else {
            "unknown".to_string()
        };

        Self {
            width,
            height,
            aspect_ratio,
        }
    }
}

fn num_gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.max(1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoQuality {
    pub quality: String, // "4K", "1080p", "720p", "preview"
    pub url: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMeta {
    pub duration_secs: u32,
    pub preview_url: String,
    pub qualities: Vec<VideoQuality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadUrls {
    pub original: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small: Option<String>,
    pub thumb: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub views: u64,
    pub likes: u64,
    pub downloads: u64,
}

fn default_status() -> String {
    "published".to_string()
}

fn default_source_type() -> String {
    "collected".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperItem {
    pub id: String,
    pub provider: String,
    pub source_id: String,
    pub media_type: MediaType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub resolution: Resolution,
    pub orientation: Orientation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dominant_color: Option<String>,
    pub preview_url: String,
    pub regular_url: String,
    pub downloads: DownloadUrls,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoMeta>,
    pub author: Author,
    pub stats: Stats,
    pub source_page_url: String,
    pub license: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_source_type")]
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collected_at: Option<DateTime<Utc>>,
}

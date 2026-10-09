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

/// A file we downloaded and stored under `uploads/`. The hash and size describe this stored file,
/// never the provider's original.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyFile {
    /// Site-relative path such as `/uploads/desktop-<sha256>.mp4`.
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub mime: String,
}

impl CopyFile {
    /// Only plain file names directly under `/uploads/` are ever handed to clients. Rows are
    /// written by this service, but the check keeps a bad value from becoming an off-site URL.
    pub fn is_safe(&self) -> bool {
        self.path.strip_prefix("/uploads/").is_some_and(|name| {
            !name.is_empty()
                && !name.contains("..")
                && name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
        })
    }
}

/// Our own stored copy of an item's media, kept next to (never instead of) the provider's links.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCopy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<CopyFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<CopyFile>,
}

impl LocalCopy {
    pub fn is_empty(&self) -> bool {
        self.media.is_none() && self.cover.is_none()
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_copy: Option<LocalCopy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collected_at: Option<DateTime<Utc>>,
}

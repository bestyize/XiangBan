#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PexelsPhotoResponse {
    #[serde(default)]
    pub photos: Vec<PexelsPhoto>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsPhoto {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub url: String,
    pub photographer: String,
    pub photographer_url: Option<String>,
    pub avg_color: Option<String>,
    pub alt: Option<String>,
    pub src: PexelsPhotoSrc,
}

#[derive(Debug, Deserialize)]
pub struct PexelsPhotoSrc {
    pub original: String,
    pub large2x: String,
    pub large: String,
    pub medium: String,
    pub small: String,
    pub tiny: String,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideoResponse {
    #[serde(default)]
    pub videos: Vec<PexelsVideo>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideo {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub url: String,
    pub image: String,
    #[serde(default)]
    pub duration: u32,
    pub user: PexelsVideoUser,
    #[serde(default)]
    pub video_files: Vec<PexelsVideoFile>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideoUser {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PexelsVideoFile {
    pub quality: Option<String>,
    pub file_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub link: String,
}

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PixabayImageResponse {
    pub total: usize,
    #[serde(rename = "totalHits")]
    pub total_hits: usize,
    pub hits: Vec<PixabayImageHit>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayImageHit {
    pub id: u64,
    #[serde(rename = "pageURL")]
    pub page_url: String,
    #[serde(rename = "type")]
    pub hit_type: String,
    pub tags: String,
    #[serde(rename = "previewURL")]
    pub preview_url: String,
    #[serde(rename = "previewWidth")]
    pub preview_width: Option<u32>,
    #[serde(rename = "previewHeight")]
    pub preview_height: Option<u32>,
    #[serde(rename = "webformatURL")]
    pub webformat_url: String,
    #[serde(rename = "webformatWidth")]
    pub webformat_width: Option<u32>,
    #[serde(rename = "webformatHeight")]
    pub webformat_height: Option<u32>,
    #[serde(rename = "largeImageURL")]
    pub large_image_url: String,
    #[serde(rename = "fullHDURL")]
    pub full_hd_url: Option<String>,
    #[serde(rename = "imageURL")]
    pub image_url: Option<String>,
    #[serde(rename = "imageWidth")]
    pub image_width: u32,
    #[serde(rename = "imageHeight")]
    pub image_height: u32,
    pub views: Option<u64>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    pub comments: Option<u64>,
    pub user_id: u64,
    pub user: String,
    #[serde(rename = "userImageURL")]
    pub user_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideoResponse {
    pub total: usize,
    #[serde(rename = "totalHits")]
    pub total_hits: usize,
    pub hits: Vec<PixabayVideoHit>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideoHit {
    pub id: u64,
    #[serde(rename = "pageURL")]
    pub page_url: String,
    #[serde(rename = "type")]
    pub hit_type: String,
    pub tags: String,
    pub duration: u32,
    #[serde(rename = "picture_id")]
    pub picture_id: Option<String>,
    pub videos: PixabayVideoFiles,
    pub views: Option<u64>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    pub comments: Option<u64>,
    pub user_id: u64,
    pub user: String,
    #[serde(rename = "userImageURL")]
    pub user_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideoFiles {
    pub large: Option<PixabayVideoDetail>,
    pub medium: Option<PixabayVideoDetail>,
    pub small: Option<PixabayVideoDetail>,
    pub tiny: Option<PixabayVideoDetail>,
}

#[derive(Debug, Deserialize)]
pub struct PixabayVideoDetail {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: Option<u64>,
}

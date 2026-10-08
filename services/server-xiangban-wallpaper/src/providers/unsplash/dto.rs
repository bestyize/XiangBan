#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UnsplashSearchResponse {
    pub total: usize,
    pub total_pages: usize,
    pub results: Vec<UnsplashPhoto>,
}

#[derive(Debug, Deserialize)]
pub struct UnsplashPhoto {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub color: Option<String>,
    pub description: Option<String>,
    pub alt_description: Option<String>,
    pub urls: UnsplashUrls,
    pub links: UnsplashLinks,
    pub likes: Option<u64>,
    pub user: UnsplashUser,
    #[serde(default)]
    pub tags: Option<Vec<UnsplashTag>>,
}

#[derive(Debug, Deserialize)]
pub struct UnsplashUrls {
    pub raw: String,
    pub full: String,
    pub regular: String,
    pub small: String,
    pub thumb: String,
}

#[derive(Debug, Deserialize)]
pub struct UnsplashLinks {
    pub html: String,
    pub download: Option<String>,
    pub download_location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnsplashUser {
    pub id: String,
    pub username: String,
    pub name: String,
    pub portfolio_url: Option<String>,
    pub profile_image: Option<UnsplashProfileImage>,
}

#[derive(Debug, Deserialize)]
pub struct UnsplashProfileImage {
    pub small: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnsplashTag {
    pub title: String,
}

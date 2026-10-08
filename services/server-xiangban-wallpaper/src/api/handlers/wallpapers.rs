use crate::{
    api::state::AppState,
    error::AppError,
    models::{query::SearchQuery, MediaType, SearchResult, WallpaperItem},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CuratedParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
pub struct VideoParams {
    pub q: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    24
}

/// GET /api/v1/wallpapers/search
pub async fn search_wallpapers(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResult>, AppError> {
    let result = state.db.search_published_wallpapers(&query).await?;
    Ok(Json(result))
}

/// GET /api/v1/wallpapers/curated
pub async fn get_curated_wallpapers(
    State(state): State<AppState>,
    Query(params): Query<CuratedParams>,
) -> Result<Json<SearchResult>, AppError> {
    let query = SearchQuery {
        media_type: Some(MediaType::Image),
        page: Some(params.page),
        per_page: Some(params.per_page),
        ..Default::default()
    };
    let result = state.db.search_published_wallpapers(&query).await?;
    Ok(Json(result))
}

/// GET /api/v1/wallpapers/videos
pub async fn get_video_wallpapers(
    State(state): State<AppState>,
    Query(params): Query<VideoParams>,
) -> Result<Json<SearchResult>, AppError> {
    let query = SearchQuery {
        media_type: Some(MediaType::Video),
        q: params.q,
        page: Some(params.page),
        per_page: Some(params.per_page),
        ..Default::default()
    };
    let result = state.db.search_published_wallpapers(&query).await?;
    Ok(Json(result))
}

/// GET /api/v1/wallpapers/:provider/:id
pub async fn get_wallpaper_detail(
    State(state): State<AppState>,
    Path((provider, id)): Path<(String, String)>,
) -> Result<Json<WallpaperItem>, AppError> {
    let full_id = if id.starts_with(&format!("{provider}_")) {
        id.clone()
    } else {
        format!("{provider}_{id}")
    };

    if let Some(item) = state.db.get_wallpaper(&full_id).await? {
        return Ok(Json(item));
    }

    if let Some(item) = state.db.get_wallpaper(&id).await? {
        return Ok(Json(item));
    }

    let item = state
        .aggregator
        .detail(&provider, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Wallpaper '{id}' not found in provider '{provider}'")))?;

    Ok(Json(item))
}

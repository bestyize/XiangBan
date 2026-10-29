use crate::{
    api::state::AppState,
    error::AppError,
    hotlink::Overrides,
    models::{query::SearchQuery, MediaType},
    public_item::{public_item, public_result, PublicItem, PublicSearchResult},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

/// The stored hotlink overrides. Without PostgreSQL, or if they cannot be read, the defaults
/// apply: a settings hiccup must not take the public catalogue down.
async fn overrides(state: &AppState) -> Overrides {
    let Ok(store) = state.admin.store() else {
        return Overrides::default();
    };
    Overrides::load(store).await.unwrap_or_else(|error| {
        tracing::warn!("Could not read hotlink settings, using defaults: {error}");
        Overrides::default()
    })
}

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
) -> Result<Json<PublicSearchResult>, AppError> {
    let result = state.db.search_published_wallpapers(&query).await?;
    Ok(Json(public_result(result, &overrides(&state).await)))
}

/// GET /api/v1/wallpapers/curated
pub async fn get_curated_wallpapers(
    State(state): State<AppState>,
    Query(params): Query<CuratedParams>,
) -> Result<Json<PublicSearchResult>, AppError> {
    let query = SearchQuery {
        media_type: Some(MediaType::Image),
        page: Some(params.page),
        per_page: Some(params.per_page),
        ..Default::default()
    };
    let result = state.db.search_published_wallpapers(&query).await?;
    Ok(Json(public_result(result, &overrides(&state).await)))
}

/// GET /api/v1/wallpapers/videos
pub async fn get_video_wallpapers(
    State(state): State<AppState>,
    Query(params): Query<VideoParams>,
) -> Result<Json<PublicSearchResult>, AppError> {
    let query = SearchQuery {
        media_type: Some(MediaType::Video),
        q: params.q,
        page: Some(params.page),
        per_page: Some(params.per_page),
        ..Default::default()
    };
    let result = state.db.search_published_wallpapers(&query).await?;
    Ok(Json(public_result(result, &overrides(&state).await)))
}

/// GET /api/v1/wallpapers/:provider/:id
pub async fn get_wallpaper_detail(
    State(state): State<AppState>,
    Path((provider, id)): Path<(String, String)>,
) -> Result<Json<PublicItem>, AppError> {
    let full_id = if id.starts_with(&format!("{provider}_")) {
        id.clone()
    } else {
        format!("{provider}_{id}")
    };

    if let Some(item) = state.db.get_wallpaper(&full_id).await? {
        let policy = overrides(&state).await.policy(&item);
        return Ok(Json(public_item(item, policy)));
    }

    if let Some(item) = state.db.get_wallpaper(&id).await? {
        let policy = overrides(&state).await.policy(&item);
        return Ok(Json(public_item(item, policy)));
    }

    let item = state
        .aggregator
        .detail(&provider, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Wallpaper '{id}' not found in provider '{provider}'")))?;

    // Not in our catalogue, so there is no stored copy to offer.
    Ok(Json(PublicItem { item, fallback: None }))
}

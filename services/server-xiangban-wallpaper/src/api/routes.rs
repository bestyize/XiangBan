use super::{
    handlers::{
        auth::{get_auth_config, get_current_user},
        categories::list_categories,
        collector::trigger_collector_sync,
        favorites::{add_favorite, check_favorites, is_favorite, list_favorites, remove_favorite},
        health::health_check,
        providers::list_providers,
        wallpapers::{
            get_curated_wallpapers, get_video_wallpapers, get_wallpaper_detail,
            search_wallpapers,
        },
    },
    state::AppState,
};
use crate::admin::routes::admin_router;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn create_api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/desktop/product", get(crate::desktop::product))
        .route("/api/v1/desktop/updates", get(crate::desktop::updates))
        .route("/api/v1/desktop/announcements", get(crate::desktop::announcements))
        .route("/api/v1/desktop/wallpapers/{id}", get(crate::desktop::wallpaper))
        .route("/api/v1/desktop/downloads/{id}", get(crate::app_packages::download))
        .route_layer(axum::middleware::map_response(crate::desktop::no_store))
        .route("/api/v1/ai/providers", get(crate::ai_providers::get))
        // Account & User endpoints
        .route("/api/v1/auth/config", get(get_auth_config))
        .route("/api/v1/user/me", get(get_current_user))
        // Wallpaper endpoints
        .route("/api/v1/wallpapers/search", get(search_wallpapers))
        .route("/api/v1/wallpapers/curated", get(get_curated_wallpapers))
        .route("/api/v1/wallpapers/videos", get(get_video_wallpapers))
        .route("/api/v1/wallpapers/{provider}/{id}", get(get_wallpaper_detail))
        // Providers info
        .route("/api/v1/providers", get(list_providers))
        .route("/api/v1/categories", get(list_categories))
        // User favorites
        .route("/api/v1/favorites", get(list_favorites).post(add_favorite))
        .route("/api/v1/favorites/check", get(check_favorites))
        .route("/api/v1/favorites/{id}", delete(remove_favorite))
        .route("/api/v1/favorites/{id}/status", get(is_favorite))
        // Collector sync
        .route("/api/v1/collector/sync", post(trigger_collector_sync))
        // Admin console
        .nest("/api/v1/admin", admin_router(state.clone()))
        // Health check
        .route("/api/v1/health", get(health_check))
        .route("/health", get(health_check))
        .with_state(state)
}

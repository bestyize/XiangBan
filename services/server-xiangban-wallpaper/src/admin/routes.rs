use super::{handlers::*, middleware::require_admin, wallpaper_handlers::*};
use crate::api::state::AppState;
use axum::{
    http::{header, HeaderValue},
    middleware::{from_fn_with_state, map_response},
    response::Response,
    routing::{get, patch, post, put},
    Router,
};

/// Admin responses carry session state and key metadata, so no browser or proxy may keep a copy.
async fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// Mounted under `/api/v1/admin`. Everything except login sits behind `require_admin`, so a route
/// added to the protected group is private by default.
pub fn admin_router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/desktop", get(crate::desktop::admin_get).put(crate::desktop::admin_save))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/account", put(update_account))
        .route("/providers", get(list_providers))
        .route("/providers/{provider}/keys", post(add_key))
        .route("/providers/{provider}/keys/{id}", patch(update_key).delete(delete_key))
        .route("/providers/{provider}/settings", put(set_provider_settings))
        .route("/proxies", get(list_proxies).post(add_proxy))
        .route("/proxies/settings", put(set_proxy_settings))
        .route("/proxies/{id}", patch(update_proxy).delete(delete_proxy))
        .route("/proxies/{id}/test", post(test_proxy))
        .route("/cache", get(get_cache).delete(clear_cache))
        .route("/cache/settings", put(set_cache_settings))
        // 采集工作台
        .route("/collector/search", get(collector_search))
        .route("/collector/import", post(collector_import))
        .route("/collector/check-status", post(collector_check_status))
        // 图库管理与自定义上架
        .route("/wallpapers", get(admin_list_wallpapers))
        .route("/wallpapers/custom", post(admin_create_custom_wallpaper))
        .route("/wallpapers/upload", post(admin_upload_file).layer(axum::extract::DefaultBodyLimit::max(257 * 1024 * 1024)))
        .route("/wallpapers/{id}", put(admin_update_wallpaper).delete(admin_delete_wallpaper))
        .route("/wallpapers/{id}/status", put(admin_update_wallpaper_status))
        .route_layer(from_fn_with_state(state, require_admin));

    Router::new()
        .route("/auth/login", post(login))
        .merge(protected)
        .layer(map_response(no_store))
}

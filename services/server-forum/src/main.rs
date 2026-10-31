mod analytics;
mod auth;
mod config;
mod collectors;
mod community_editor;
mod content;
mod db;
mod discover;
mod error;
mod handlers;
mod jobs;
mod markdown;
mod models;
mod moderation;
mod notifications;
mod points;
mod seo;
mod state;
mod users;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::{get, post, put},
    Router,
};
use config::AppConfig;
use handlers::{admin, analytics as metrics, attachments, blocks, content as packs, discover as discover_api, follows, help, images, interact, me, meta, moderation as review, notifications as inbox, posts, topics};
use state::AppState;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(meta::health))
        .route("/config", get(meta::site_config))
        .route("/categories", get(meta::categories))
        .route("/tags", get(meta::popular_tags))
        .route("/session", post(me::session))
        .route("/me/bookmarks", get(me::bookmarks))
        .route("/me/points", get(points::wallet))
        .route("/me/points/ledger", get(points::ledger))
        .route("/me/points/invites", get(points::invites))
        .route("/me/check-in", post(points::check_in))
        .route("/me/following/topics", get(follows::topics))
        .route("/users/{uid}", get(me::profile))
        .route("/users/{uid}/follow", put(follows::follow).delete(follows::unfollow))
        .route("/users/{uid}/block", put(blocks::block).delete(blocks::unblock))
        .route("/me/blocks", get(blocks::list))
        .route("/events", post(metrics::ingest))
        .route("/me/notifications", get(inbox::list))
        .route("/me/notifications/unread", get(inbox::unread_count))
        .route("/me/notifications/read", post(inbox::mark_read))
        .route("/users/{uid}/following", get(follows::following))
        .route("/users/{uid}/followers", get(follows::followers))
        .route("/users/{uid}/topics", get(me::user_topics))
        .route("/users/{uid}/posts", get(me::user_posts))
        .route("/search", get(me::search))
        .route("/topics", get(topics::list).post(topics::create))
        .route("/markdown/preview", post(topics::preview))
        .route("/topics/{id}", get(topics::get).patch(topics::update).delete(topics::delete))
        .route("/topics/{id}/posts", get(posts::list).post(posts::create))
        .route("/topics/{id}/pin", put(topics::pin).delete(topics::unpin))
        .route("/topics/{id}/help/accept", post(help::accept))
        .route("/topics/{id}/help/close", post(help::close))
        .route("/topics/{id}/like", put(interact::like_topic).delete(interact::unlike_topic))
        .route("/topics/{id}/bookmark", put(interact::bookmark).delete(interact::unbookmark))
        .route("/posts/{id}", axum::routing::patch(posts::update).delete(posts::delete))
        .route("/posts/{id}/source", get(posts::source))
        .route("/posts/{id}/like", put(interact::like_post).delete(interact::unlike_post))
        .route("/reports", post(interact::report))
        .route(
            "/attachments",
            post(attachments::upload).layer(DefaultBodyLimit::max(
                (state.config.upload_limit_user.max(state.config.upload_limit_admin) + 1024 * 1024) as usize,
            )),
        )
        .route(
            "/images",
            post(images::upload).layer(DefaultBodyLimit::max((state.config.upload_limit_image + 1024 * 1024) as usize)),
        )
        .route("/images/{id}", get(images::show))
        .route("/attachments/{id}", get(attachments::download).delete(attachments::delete_pending))
        .route("/attachments/{id}/purchase", post(attachments::purchase))
        .merge(packs::routes())
        .merge(discover_api::routes())
        .merge(handlers::model_rankings::routes())
        .merge(handlers::community_editor::public_routes())
        .nest("/admin", admin::routes().merge(packs::admin_routes()).merge(discover_api::admin_routes()).merge(handlers::discover_editor::admin_routes()).merge(review::admin_routes()).merge(metrics::admin_routes()).nest("/collection", handlers::collection::routes()).nest("/community-editorial", handlers::community_editor::admin_routes()))
        .nest("/editor", handlers::editor::routes())
        .nest("/discover-editor", handlers::discover_editor::editor_routes());

    // 凭证只走 Authorization 头，不用 Cookie，跨域放开不构成 CSRF 面。
    let origins: Vec<HeaderValue> = state.config.cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
    let cors = if origins.is_empty() { cors.allow_origin(Any) } else { cors.allow_origin(origins) };

    Router::new()
        .nest("/api", api)
        .merge(seo::routes(&state))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server_forum=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::from_env();
    let pool = db::connect(&config.database_url, None).await?;
    db::migrate(&pool).await?;
    db::seed(&pool).await?;
    discover::seed(&pool).await?;
    discover::backfill_topic_images(&pool).await?;
    admin::seed_admin(&pool, &config).await?;

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("server-forum issuer={} client={} extra_clients={:?} listening on {addr}", config.account_issuer, config.account_client_id, config.account_extra_client_ids);

    let verifier = auth::JwksVerifier::new(
        config.account_issuer.clone(),
        config.account_jwks_url.clone(),
        config.account_client_id.clone(),
    )
    .with_extra_audiences(&config.account_extra_client_ids);
    let state = AppState {
        pool,
        config: Arc::new(config),
        verifier: Arc::new(verifier),
        http: reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?,
    };
    jobs::spawn(state.clone());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, build_router(state).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}



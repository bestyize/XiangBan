use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, warn};
use wallpaper_collect::{
    admin::{AdminService, DEFAULT_PASSWORD, DEFAULT_USERNAME},
    api::{create_api_router, AppState},
    config::AppConfig,
    db::Database,
    providers::ProviderRegistry,
    runtime::Runtime,
    services::{SearchCache, WallpaperAggregator, WallpaperCollector},
    store::Store,
    web::static_handler,
};

/// Opens the admin tables and loads keys, proxies and settings from PostgreSQL.
async fn init_admin_storage(pool: PgPool, config: &AppConfig) -> Result<(Store, Runtime), wallpaper_collect::error::AppError> {
    let store = Store::init(pool).await?;
    let runtime = Runtime::load(&store, config).await?;
    Ok((store, runtime))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging / tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wallpaper_collect=info,tower_http=info".into()),
        )
        .init();

    // Load configuration
    let config = AppConfig::from_env();

    info!("============================================================");
    info!("   WallpaperCollect - 壁纸采集、搜索与聚合服务 (Rust Axum)");
    info!("============================================================");

    // Initialize Database (PostgreSQL with in-memory fallback)
    let db = Database::init(config.database_url.as_deref()).await;

    // The admin console, API keys, proxy pool and search cache live in PostgreSQL. Without it the
    // service still runs, using keys from the environment and no cache.
    let (store, runtime) = match db.pg_pool() {
        Some(pool) => match init_admin_storage(pool, &config).await {
            Ok((store, runtime)) => (Some(store), runtime),
            Err(e) => {
                warn!("Admin console unavailable: {e}");
                (None, Runtime::from_env(&config))
            }
        },
        None => {
            warn!("PostgreSQL is not connected: the admin console and the search cache are disabled.");
            (None, Runtime::from_env(&config))
        }
    };

    // Initialize Provider Registry
    let registry = ProviderRegistry::new(&runtime, config.enable_demo_data);
    let cache = SearchCache::new(store.clone(), runtime.clone());
    let admin = AdminService::bootstrap(store).await?;

    // List provider status in console
    for prov in registry.list_info() {
        let status = if prov.is_configured { "Ready ✓" } else { "Missing API Key (Disabled)" };
        info!("Provider [{}]: {} ({})", prov.id, prov.name, status);
    }

    // Initialize Aggregator Service
    let aggregator = Arc::new(WallpaperAggregator::new(
        registry.clone(),
        db.clone(),
        cache.clone(),
    ));

    // Initialize Collector Service
    let collector = Arc::new(WallpaperCollector::new(
        registry.clone(),
        db.clone(),
        config.auto_sync_interval_mins,
    ));

    // Start background crawler / collector loop
    collector.clone().start_background_loop();

    // Build Application State
    let verifier = Arc::new(wallpaper_collect::auth::JwksVerifier::new(&config));
    let state = AppState {
        aggregator,
        collector,
        registry,
        db,
        runtime,
        cache,
        admin,
        verifier,
        config: config.clone(),
    };

    // Ensure local uploads directory exists
    tokio::fs::create_dir_all("uploads").await?;

    // Build Axum Router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_api_router(state.clone())
        .nest_service("/uploads", ServeDir::new("uploads"))
        .fallback(static_handler)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("🚀 Server running at: http://{}", addr);
    info!("🌐 Open in your browser: http://localhost:{}", config.server_port);
    info!("📖 API Base: http://localhost:{}/api/v1/wallpapers/search", config.server_port);
    if state.admin.is_available() {
        info!("🔧 Admin console: http://localhost:{}/admin", config.server_port);
        if state.admin.account_info().await.is_ok_and(|a| a.is_default_password) {
            warn!("The admin password is still the default ({DEFAULT_USERNAME} / {DEFAULT_PASSWORD}). Change it before exposing this server.");
        }
    }
    info!("============================================================");

    axum::serve(listener, app).await?;

    Ok(())
}

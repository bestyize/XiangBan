mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;
mod storage;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    routing::{delete, get, post},
    Router,
};
use config::AppConfig;
use db::DbPool;
use handlers::{
    auth as auth_h,
    files::{self as file_h, FileHandlerState},
    public::{self as public_h, PublicHandlerState},
    shares as share_h,
    system as system_h,
};
use services::{AuthService, FileService, ShareService};
use std::net::SocketAddr;
use std::path::Path;
use storage::StorageManager;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pandown_backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. 加载配置
    let config = AppConfig::load();
    tracing::info!("Starting PanDown Backend on {}:{}", config.host, config.port);

    // 3. 初始化数据库与存储
    let db = DbPool::new(&config.db_path)?;
    let storage = StorageManager::new(&config.storage_dir).await?;

    // 4. 初始化业务服务
    let auth_service = AuthService::new(db.clone(), config.clone());
    let file_service = FileService::new(db.clone(), storage.clone());
    let share_service = ShareService::new(db.clone(), config.clone());

    // 5. 组合各个子模块状态
    let file_handler_state = FileHandlerState {
        file_service: file_service.clone(),
        storage: storage.clone(),
    };

    let public_handler_state = PublicHandlerState {
        share_service: share_service.clone(),
        storage: storage.clone(),
    };

    // 6. 构造 API 路由
    let auth_routes = Router::new()
        .route("/login", post(auth_h::login))
        .route("/me", get(auth_h::get_me))
        .route("/change-password", post(auth_h::change_password))
        .with_state(auth_service);

    let file_routes = Router::new()
        .route("/list", get(file_h::list_files))
        .route("/mkdir", post(file_h::create_folder))
        .route(
            "/upload",
            post(file_h::upload_file).layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)),
        )
        .route("/search", get(file_h::search_files))
        .route("/rename/{id}", post(file_h::rename_file))
        .route("/{id}", delete(file_h::delete_file))
        .route("/download/{id}", get(file_h::download_file))
        .route("/preview/{id}", get(file_h::preview_file))
        .with_state(file_handler_state);

    let share_routes = Router::new()
        .route("/create", post(share_h::create_share))
        .route("/my", get(share_h::list_my_shares))
        .route(
            "/{code}",
            delete(share_h::delete_share).patch(share_h::update_share),
        )
        .with_state(share_service);

    let public_routes = Router::new()
        .route("/shares/{code}/info", get(public_h::get_share_info))
        .route("/shares/{code}/verify", post(public_h::verify_share))
        .route("/shares/{code}/content", get(public_h::get_share_content))
        .route("/shares/{code}/download/{file_id}", get(public_h::download_share_file))
        .route("/shares/{code}/preview/{file_id}", get(public_h::preview_share_file))
        .with_state(public_handler_state);

    let system_routes = Router::new()
        .route("/stats", get(system_h::get_stats))
        .with_state(file_service);

    let api_router = Router::new()
        .nest("/auth", auth_routes)
        .nest("/files", file_routes)
        .nest("/shares", share_routes)
        .nest("/public", public_routes)
        .nest("/system", system_routes);

    // 7. CORS 中间件配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-share-token"),
        ]);

    // 8. 根路由组织与前端静态托管
    let mut app = Router::new()
        .nest("/api", api_router)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors);

    // 检测前端 dist 目录：如果存在则挂载单页应用支持
    let frontend_dist = Path::new("frontend/dist");
    let alt_frontend_dist = Path::new("../frontend/dist");
    let dist_path = if frontend_dist.exists() {
        Some(frontend_dist)
    } else if alt_frontend_dist.exists() {
        Some(alt_frontend_dist)
    } else {
        None
    };

    if let Some(path) = dist_path {
        tracing::info!("挂载前端静态文件目录: {:?}", path);
        let index_file = path.join("index.html");
        let serve_dir = ServeDir::new(path).fallback(ServeFile::new(index_file));
        app = app.fallback_service(serve_dir);
    } else {
        app = app.route(
            "/",
            get(|| async {
                "PanDown 后端服务正在运行。前端正在构建中或请使用 vite 开发服务器。"
            }),
        );
    }

    // 9. 启动服务监听
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("监听地址 '{}:{}' 无效: {}", config.host, config.port, e))?;
    tracing::info!("==================================================");
    tracing::info!("  PanDown 网盘已就绪");
    tracing::info!("  服务地址: http://{}:{}", config.host, config.port);
    tracing::info!("  默认管理员: admin");
    tracing::info!("  默认密码:   123456");
    tracing::info!("  数据目录:   {}", config.data_dir.display());
    tracing::info!("==================================================");

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        anyhow::anyhow!(
            "无法绑定端口 {}:{} —— {}（该端口可能已被占用，可用 --port <端口> 指定其它端口）",
            config.host,
            config.port,
            e
        )
    })?;
    axum::serve(listener, app).await?;

    Ok(())
}

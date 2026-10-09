mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod services;

use axum::{
    http::{header, Method},
    response::{Html, IntoResponse, Response},
    routing::{get, patch, post, put},
    Router,
};
use config::AppConfig;
use std::path::Path;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;

async fn health_check() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "server-xiaoban-appstore",
        "version": "0.1.0"
    }))
}

async fn fallback_page() -> Response {
    Html(r#"
    <!DOCTYPE html>
    <html lang="zh-CN">
    <head>
        <meta charset="utf-8"/>
        <title>小伴应用精选 - Xiaoban AppStore</title>
        <style>
            body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; background: #fafafa; color: #333; }
            .card { background: #fff; padding: 40px; border-radius: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.06); text-align: center; max-width: 480px; }
            h1 { color: #f59e0b; margin-top: 0; }
            a { color: #2563eb; text-decoration: none; font-weight: 500; }
        </style>
    </head>
    <body>
        <div class="card">
            <h1>小伴应用精选 (Xiaoban AppStore)</h1>
            <p>只收录最优质的软件 · 腾讯柠檬极简风</p>
            <p>后端服务已就绪！前端开发环境请访问 <a href="http://localhost:5173" target="_blank">http://localhost:5173</a></p>
            <p>API 健康检查：<a href="/api/health">/api/health</a> | 应用列表：<a href="/api/apps">/api/apps</a></p>
        </div>
    </body>
    </html>
    "#).into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server_xiaoban_appstore=info,tower_http=info".into()),
        )
        .init();

    info!("============================================================");
    info!("   小伴应用精选 - Xiaoban AppStore (Rust Axum + PostgreSQL)");
    info!("============================================================");

    let config = AppConfig::from_env();

    // 初始化数据库与种子数据
    let pool = db::init_db(&config).await?;

    // API 路由构建
    let api_router = Router::new()
        // 公开接口
        .route("/health", get(health_check))
        .route("/apps", get(handlers::apps::list_apps))
        .route("/apps/{slug}", get(handlers::apps::get_app_by_slug))
        .route("/apps/{slug}/download", post(handlers::apps::track_download))
        .route("/apps/{slug}/direct-download", get(handlers::apps::direct_download))
        .route("/categories", get(handlers::apps::list_categories))
        .route("/featured", get(handlers::apps::get_featured_apps))
        // 管理后台接口
        .route("/admin/login", post(handlers::admin::admin_login))
        .route("/admin/me", get(handlers::admin::get_me))
        .route("/admin/account", put(handlers::admin::change_admin_credentials))
        .route("/admin/stats", get(handlers::admin::admin_get_stats))
        .route("/admin/apps", get(handlers::admin::admin_list_apps).post(handlers::admin::admin_create_app))
        .route("/admin/apps/{id}", put(handlers::admin::admin_update_app).delete(handlers::admin::admin_delete_app))
        .route("/admin/apps/{id}/status", patch(handlers::admin::admin_update_status))
        .route("/admin/apps/{id}/sync-version", post(handlers::admin::sync_app_version))
        .route("/admin/apps/sync-all", post(handlers::admin::sync_all_versions))
        .route("/admin/categories", get(handlers::admin::admin_list_categories))
        .with_state(pool);

    // 跨域支持 (允许 Vite 开发服务器请求)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    let mut app = Router::new()
        .nest("/api", api_router)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    // 静态文件托管与 SPA 单页回退
    if Path::new(&config.web_dir).exists() {
        let index_file = format!("{}/index.html", config.web_dir);
        let serve_dir = ServeDir::new(&config.web_dir).fallback(ServeFile::new(index_file));
        app = app.fallback_service(serve_dir);
    } else {
        app = app.fallback(fallback_page);
    }

    let addr = format!("{}:{}", config.host, config.port);
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

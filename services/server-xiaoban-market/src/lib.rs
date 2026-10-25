//! 小伴市场：专家、技能、连接器的自建目录服务。
pub mod account;
pub mod auth;
pub mod config;
pub mod crawler;
pub mod db;
pub mod error;
pub mod handlers;
pub mod importer;
pub mod items;
pub mod package;
pub mod plugin;
pub mod state;
pub mod storage;
pub mod upstream;

use axum::{
    http::{header, Method},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use config::Config;
use error::{AppError, Result};
use state::AppState;
use std::{path::Path, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

/// 连接数据库、迁移、写入种子数据，并构造共享状态。
pub async fn build_state(config: Config) -> Result<AppState> {
    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;
    db::seed(&pool, &config).await?;
    crawler::recover(&pool).await?;
    if let Err(e) = importer::align_stored_packages(&pool, &config.data_dir).await {
        tracing::warn!("整理已入库的技能包失败，稍后重启会再试：{e}");
    }
    match importer::backfill_expert_packages(&pool, &config.data_dir).await {
        Ok(0) => {}
        Ok(n) => tracing::info!("已为 {n} 个专家补上安装包"),
        Err(e) => tracing::warn!("为专家补安装包失败，稍后重启会再试：{e}"),
    }
    let (http, objects) = upstream::build_clients().map_err(|e| AppError::Internal(format!("HTTP 客户端初始化失败：{e}")))?;
    let account = config.account_issuer.clone().map(|issuer| Arc::new(account::AccountVerifier::new(issuer, config.account_jwks_url.clone(), config.account_client_id.clone())));
    Ok(AppState { pool, config: Arc::new(config), http, objects, runner: Arc::new(crawler::Runner::new()), account })
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "server-xiaoban-market", "version": env!("CARGO_PKG_VERSION") }))
}

async fn landing() -> impl IntoResponse {
    Html("<!doctype html><meta charset=utf-8><title>小伴市场</title><body style=\"font-family:system-ui;padding:48px\"><h1>小伴市场</h1><p>服务已就绪。前端尚未构建：目录 API 在 <code>/api/v1/market/skills</code>，后台 API 在 <code>/api/v1/admin</code>。</p>")
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .nest("/market", handlers::public::router())
        .nest("/admin", handlers::admin::router());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT, header::RANGE]);
    let mut app = Router::new().route("/api/health", get(health)).nest("/api/v1", api).with_state(state.clone());
    if Path::new(&state.config.web_dir).exists() {
        let index = format!("{}/index.html", state.config.web_dir);
        app = app.fallback_service(ServeDir::new(&state.config.web_dir).fallback(ServeFile::new(index)));
    } else {
        app = app.fallback(landing);
    }
    app.layer(cors).layer(CompressionLayer::new()).layer(TraceLayer::new_for_http())
}

mod admin_auth;
mod binding;
mod config;
mod db;
mod error;
mod extract;
mod handlers;
mod jwks;
mod keys;
mod ledger;
mod newapi;
mod password;
mod products;
mod proxy;
mod quota;
mod state;
mod sync;

#[cfg(test)]
mod tests;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    response::Redirect,
    routing::{get, post, put},
    Router,
};
use config::AppConfig;
use handlers::{admin, meta, user};
use state::AppState;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(meta::health))
        .route("/ai/tiers", get(user::tiers))
        .route("/ai/balance", get(user::balance))
        .route("/ai/ledger", get(user::ledger))
        .route("/admin/login", post(admin::login))
        .route("/admin/me", get(admin::me))
        .route("/admin/password", post(admin::change_password))
        .route("/admin/overview", get(admin::overview))
        .route("/admin/products", get(admin::list_products))
        .route("/admin/products/{product_id}", put(admin::upsert_product))
        .route("/admin/products/{product_id}/tiers", get(admin::list_tiers))
        .route("/admin/products/{product_id}/tiers/{tier}", put(admin::upsert_tier).delete(admin::delete_tier))
        .route("/admin/models", get(admin::models))
        .route("/admin/bindings", get(admin::list_bindings))
        .route("/admin/bindings/{id}", get(admin::get_binding))
        .route("/admin/bindings/{id}/grant", post(admin::grant))
        .route("/admin/buckets/{id}/void", post(admin::void_bucket))
        .route("/admin/anomalies", get(admin::anomalies))
        .route("/admin/anomalies/{id}/resolve", post(admin::resolve_anomaly))
        .route("/admin/audit", get(admin::audit_log));

    let v1 = Router::new()
        .route("/models", get(proxy::list_models))
        .route("/chat/completions", post(proxy::chat_completions))
        .layer(DefaultBodyLimit::max(state.config.max_body_bytes));

    // 凭证只走 Authorization 头，不使用 Cookie，因此跨域放开不构成 CSRF 面。
    let origins: Vec<HeaderValue> = state.config.cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
    let cors = if origins.is_empty() { cors.allow_origin(Any) } else { cors.allow_origin(origins) };

    let mut router = Router::new().nest("/api", api).nest("/v1", v1);

    let web_dir = &state.config.web_dir;
    if web_dir.join("index.html").is_file() {
        let spa = ServeDir::new(web_dir).fallback(ServeFile::new(web_dir.join("index.html")));
        router = router.route("/", get(|| async { Redirect::temporary("/admin/") })).nest_service("/admin", spa);
    } else {
        tracing::warn!("未找到管理后台前端 {}，只提供 API；构建方式见 README", web_dir.display());
    }

    router.layer(cors).layer(TraceLayer::new_for_http()).with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("gen-key") {
        println!("AI_MASTER_KEY_BASE64={}", keys::generate_master_key());
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "server_ai=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::from_env();
    let keys = keys::Keys::load(&config)?;
    let pool = db::connect(&config.database_url, None).await?;
    db::migrate(&pool).await?;
    admin_auth::seed(&pool, &config).await?;

    if config.newapi_admin_token.is_none() {
        tracing::warn!("未设置 AI_NEWAPI_ADMIN_TOKEN：无法开通用户、对账，也无法转发调用");
    }
    let dummy_password_hash = password::hash("dummy-password-for-timing".into()).await?;
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("server-ai issuer={} newapi={} listening on {addr}", config.account_issuer, config.newapi_base_url);

    let state = AppState {
        pool,
        verifier: Arc::new(jwks::JwksVerifier::new(config.account_issuer.clone(), config.account_jwks_url.clone())),
        newapi: Arc::new(newapi::NewApi::new(config.newapi_base_url.clone(), config.newapi_admin_token.clone())),
        config: Arc::new(config),
        keys: Arc::new(keys),
        dummy_password_hash: Arc::new(dummy_password_hash),
    };
    tokio::spawn(sync::run_loop(state.clone()));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, build_router(state).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

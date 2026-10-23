mod admin_auth;
mod binding;
mod codes;
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
mod reasoning;
mod proxy;
mod quota;
mod ratelimit;
mod redeem;
mod settings;
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
        .route("/ai/redeem", post(user::redeem))
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
        .route("/admin/batches", get(admin::list_batches).post(admin::create_batch))
        .route("/admin/batches/{id}/void", post(admin::void_batch))
        .route("/admin/codes/lookup", post(admin::lookup_code))
        .route("/admin/codes/{id}/void", post(admin::void_code))
        .route("/admin/settings", get(admin::get_settings))
        .route("/admin/settings/newapi", put(admin::put_newapi_token))
        .route("/admin/settings/newapi/test", post(admin::test_newapi))
        .route("/admin/anomalies", get(admin::anomalies))
        .route("/admin/anomalies/{id}/resolve", post(admin::resolve_anomaly))
        .route("/admin/audit", get(admin::audit_log));

    let v1 = Router::new()
        .route("/models", get(proxy::list_models))
        .route("/chat/completions", post(proxy::chat_completions))
        .route("/responses", post(proxy::responses))
        .route("/messages", post(proxy::messages))
        .route("/audio/speech", post(proxy::audio_speech))
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

    let dummy_password_hash = password::hash("dummy-password-for-timing".into()).await?;
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("server-ai issuer={} newapi={} listening on {addr}", config.account_issuer, config.newapi_base_url);

    // 后台里填写的令牌优先于环境变量；读不出来（例如主密钥换过）时退回环境变量，并在后台提示重新填写。
    let newapi = Arc::new(newapi::NewApi::new(config.newapi_base_url.clone(), config.newapi_admin_token.clone()));
    match settings::load_newapi_token(&pool, &keys).await {
        Ok(Some(stored)) => {
            newapi.set_admin_token(Some(stored.token));
            info!("已从数据库加载 New API 管理令牌");
        }
        Ok(None) => {}
        Err(e) => tracing::warn!("后台保存的 New API 令牌读不出来，暂用环境变量: {e}"),
    }
    if !newapi.is_configured() {
        tracing::warn!("尚未设置 New API 管理令牌：登录后台「网关连接」页填写之前，无法开通用户、对账，也不能转发调用");
    }
    let (redeem_user_limiter, redeem_ip_limiter) = state::redeem_limiters();
    let state = AppState {
        pool,
        redeem_user_limiter,
        redeem_ip_limiter,
        verifier: Arc::new(jwks::JwksVerifier::new(config.account_issuer.clone(), config.account_jwks_url.clone())),
        newapi,
        config: Arc::new(config),
        keys: Arc::new(keys),
        dummy_password_hash: Arc::new(dummy_password_hash),
    };
    tokio::spawn(sync::run_loop(state.clone()));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, build_router(state).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

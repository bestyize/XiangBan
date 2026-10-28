mod auth;
mod config;
mod db;
mod error;
mod frames;
mod handlers;
mod hub;
mod ratelimit;
mod state;
mod ws;

#[cfg(test)]
mod tests;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{delete, get, post},
    Router,
};
use config::Config;
use handlers::{devices, pairing};
use state::AppState;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(handlers::health))
        .route("/devices", post(devices::create).get(devices::list))
        .route("/devices/{id}", delete(devices::remove))
        .route("/pair-codes", post(pairing::create_code))
        .route("/pair", post(pairing::redeem))
        .route("/pairings", get(pairing::list))
        .route("/pairings/{id}", delete(pairing::remove))
        .route("/pairings/{id}/confirm", post(pairing::confirm))
        .route("/pairings/{id}/reject", post(pairing::reject))
        .route("/ws", get(ws::connect));

    // 凭证只走 Authorization 头或 WebSocket 子协议，不用 Cookie，跨域放开不构成 CSRF 面。
    let origins: Vec<HeaderValue> = state.config.cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
    let cors = if origins.is_empty() { cors.allow_origin(Any) } else { cors.allow_origin(origins) };

    Router::new().nest("/api/link", api).layer(cors).layer(TraceLayer::new_for_http()).with_state(state)
}

/// 清理过期的配对码和早已结束的配对记录。
async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM pair_codes WHERE expires_at < now() - interval '1 day' OR used_at < now() - interval '1 day'").execute(pool).await;
    let _ = sqlx::query("DELETE FROM pairings WHERE status IN ('rejected', 'revoked') AND COALESCE(revoked_at, created_at) < now() - interval '30 days'").execute(pool).await;
    let _ = sqlx::query("DELETE FROM pairings WHERE status = 'pending' AND created_at < now() - interval '1 day'").execute(pool).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "server_link=info,tower_http=info".into()))
        .init();

    let config = Config::from_env();
    let pool = db::connect(&config.database_url, None).await?;
    db::migrate(&pool).await?;

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("server-link issuer={} clients={:?} listening on {addr}", config.account_issuer, config.client_ids);

    let verifier = auth::JwksVerifier::new(config.account_issuer.clone(), config.account_jwks_url.clone(), config.client_ids.clone());
    let state = AppState {
        pool: pool.clone(),
        hub: Arc::new(hub::Hub::new(config.notify_queue_cap, config.notify_ttl)),
        limiter: Arc::new(ratelimit::WindowLimiter::default()),
        verifier: Arc::new(verifier),
        config: Arc::new(config),
    };
    tokio::spawn(async move {
        loop {
            cleanup(&pool).await;
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, build_router(state).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

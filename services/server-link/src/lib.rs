//! 手机与桌面互联的中转服务。二进制入口在 main.rs；其他 crate 的集成测试可以通过 `testing` 特性拉起一个真实服务。

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod frames;
pub mod handlers;
pub mod hub;
pub mod ratelimit;
pub mod state;
pub mod ws;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

#[cfg(test)]
mod tests;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{delete, get, post},
    Router,
};
use handlers::{devices, pairing};
use state::AppState;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

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
pub async fn cleanup(pool: &sqlx::PgPool) {
    let _ = sqlx::query("DELETE FROM pair_codes WHERE expires_at < now() - interval '1 day' OR used_at < now() - interval '1 day'").execute(pool).await;
    let _ = sqlx::query("DELETE FROM pairings WHERE status IN ('rejected', 'revoked') AND COALESCE(revoked_at, created_at) < now() - interval '30 days'").execute(pool).await;
    let _ = sqlx::query("DELETE FROM pairings WHERE status = 'pending' AND created_at < now() - interval '1 day'").execute(pool).await;
}


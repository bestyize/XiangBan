mod admin_auth;
mod codes;
mod config;
mod crypto;
mod db;
mod error;
mod extract;
mod handlers;
mod identifier;
mod keys;
mod password;
mod ratelimit;
mod registration;
mod sender;
mod sessions;
mod state;
mod tencent;
mod tokens;
mod username;
mod users;

#[cfg(test)]
mod tests;

use axum::{
    http::{header, HeaderValue, Method},
    response::Redirect,
    routing::{get, patch, post},
    Router,
};
use config::AppConfig;
use handlers::{admin, auth, me, meta};
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
        .route("/auth/code/send", post(auth::send_code))
        .route("/auth/code/login", post(auth::login_with_code))
        .route("/auth/register", post(auth::register))
        .route("/auth/username/available", get(auth::username_available))
        .route("/auth/password/login", post(auth::login_with_password))
        .route("/auth/password/reset", post(auth::reset_password))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/logout-all", post(auth::logout_all))
        .route("/me", get(me::get_me).patch(me::update_me))
        .route("/me/password", post(me::set_password))
        .route("/me/identities/code", post(me::send_bind_code))
        .route("/me/identities", post(me::bind_identity))
        .route("/admin/login", post(admin::login))
        .route("/admin/me", get(admin::me))
        .route("/admin/password", post(admin::change_password))
        .route("/admin/stats", get(admin::stats))
        .route("/admin/users", get(admin::list_users))
        .route("/admin/users/{uid}", get(admin::get_user))
        .route("/admin/users/{uid}/status", post(admin::set_user_status))
        .route("/admin/users/{uid}/username", post(admin::set_user_username))
        .route("/admin/users/{uid}/revoke-sessions", post(admin::revoke_user_sessions))
        .route("/admin/clients", get(admin::list_clients).post(admin::create_client))
        .route("/admin/clients/{client_id}", patch(admin::update_client))
        .route("/admin/audit", get(admin::audit_log))
        .route("/admin/sender", get(admin::get_sender).put(admin::update_sender))
        .route("/admin/sender/test", post(admin::test_sender));

    // 凭证只走 Authorization 头，不使用 Cookie，因此跨域放开不构成 CSRF 面。
    let origins: Vec<HeaderValue> = state.config.cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
    let cors = if origins.is_empty() { cors.allow_origin(Any) } else { cors.allow_origin(origins) };

    let mut router = Router::new().nest("/api", api).route("/.well-known/jwks.json", get(meta::jwks));

    let web_dir = &state.config.web_dir;
    if web_dir.join("index.html").is_file() {
        let spa = ServeDir::new(web_dir).fallback(ServeFile::new(web_dir.join("index.html")));
        router = router
            .route("/", get(|| async { Redirect::temporary("/admin/") }))
            .nest_service("/admin", spa);
    } else {
        tracing::warn!("未找到管理后台前端 {}，只提供 API；构建方式见 README", web_dir.display());
    }

    router
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("gen-keys") {
        let (jwt, master) = keys::generate_env_values();
        println!("ACCOUNT_JWT_PKCS8_BASE64={jwt}\nACCOUNT_MASTER_KEY_BASE64={master}");
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server_account=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::from_env();
    let keys = keys::Keys::load(&config)?;
    let sender = sender::Sender::from_config(&config.sender)?;

    let pool = db::connect(&config.database_url, None).await?;
    db::migrate(&pool).await?;
    db::seed_clients(&pool, &config.clients).await?;
    admin_auth::seed(&pool, &config).await?;

    if let Ok(Some((val_enc,))) = sqlx::query_as::<_, (Vec<u8>,)>("SELECT value_enc FROM system_settings WHERE key = 'sender_config'")
        .fetch_optional(&pool)
        .await
    {
        if let Ok(dec) = keys.sub.decrypt("sender_config", &val_enc) {
            if let Ok(saved) = serde_json::from_str::<sender::SavedSenderConfig>(&dec) {
                if let Ok(kind) = saved.to_sender_kind(&config) {
                    sender.update(kind);
                    info!("已从数据库加载发信配置 (provider={})", saved.provider);
                }
            }
        }
    }

    let dummy_password_hash = password::hash("dummy-password-for-timing".into()).await?;
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("server-account kid={} issuer={} listening on {addr}", keys.kid, config.issuer);

    let state = AppState {
        pool,
        config: Arc::new(config),
        keys: Arc::new(keys),
        sender: Arc::new(sender),
        dummy_password_hash: Arc::new(dummy_password_hash),
        username_limiter: state::username_limiter(),
    };
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, build_router(state).into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

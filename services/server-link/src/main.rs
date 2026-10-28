use server_link::{auth, build_router, cleanup, config::Config, db, hub, ratelimit, state::AppState};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tracing::info;

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

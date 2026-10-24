use server_xiaoban_market::{build_state, config::Config, crawler, router};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "server_xiaoban_market=info,tower_http=info".into()))
        .init();
    let config = Config::from_env();
    let addr = format!("{}:{}", config.host, config.port);
    let state = build_state(config).await?;
    tokio::spawn(crawler::scheduler(state.clone()));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("小伴市场已启动：http://{addr}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}

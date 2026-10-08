use std::{env, net::SocketAddr, path::PathBuf};
use xiaoban_wallpaper_server::{api, auth, AppState, Config};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "xiaoban_wallpaper_server=info,tower_http=info".into()),
        )
        .init();
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("PORT").unwrap_or_else(|_| "55006".into());
    let address: SocketAddr = format!("{host}:{port}").parse()?;
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("请配置本服务独立的 PostgreSQL DATABASE_URL"))?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    let origin = env::var("PUBLIC_ORIGIN").unwrap_or_else(|_| format!("http://127.0.0.1:{port}"));
    let origin_url = url::Url::parse(&origin)?;
    if origin_url.origin().ascii_serialization() != origin
        || !["http", "https"].contains(&origin_url.scheme())
    {
        anyhow::bail!("PUBLIC_ORIGIN 需要完整 HTTP(S) 源，不带路径或末尾斜杠");
    }
    let secure_cookie = env::var("SECURE_COOKIE")
        .map(|s| s == "true")
        .unwrap_or(origin.starts_with("https://"));
    if origin.starts_with("https://") && !secure_cookie {
        anyhow::bail!("HTTPS 部署应启用 SECURE_COOKIE=true");
    }
    let allow_loopback_resources =
        cfg!(debug_assertions) && env::var("ALLOW_LOOPBACK_RESOURCES").as_deref() == Ok("true");
    let frontend_dist = env::var_os("FRONTEND_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/dist"));
    let state = AppState::new(
        pool,
        Config {
            public_origin: origin,
            secure_cookie,
            allow_loopback_resources,
            frontend_dist,
        },
    );
    auth::initialize(&state)
        .await
        .map_err(|e| anyhow::anyhow!("初始化失败：{}", e.1))?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address,"小伴桌面元数据服务已启动");
    axum::serve(
        listener,
        api::router(state.clone()).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    state.pool.close().await;
    Ok(())
}
async fn shutdown() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("无法注册 SIGTERM");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

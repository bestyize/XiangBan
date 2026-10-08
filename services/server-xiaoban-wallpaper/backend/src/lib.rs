pub mod api;
pub mod auth;
pub mod error;
pub mod probe;
pub mod protocol;
use std::{
    collections::HashMap,
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
#[derive(Clone)]
pub struct Config {
    pub public_origin: String,
    pub secure_cookie: bool,
    pub allow_loopback_resources: bool,
    pub frontend_dist: PathBuf,
}
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Config,
    pub login_limits: Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>,
}
impl AppState {
    pub fn new(pool: sqlx::PgPool, config: Config) -> Self {
        Self {
            pool,
            config,
            login_limits: Default::default(),
        }
    }
}

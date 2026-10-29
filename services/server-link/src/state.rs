use crate::{auth::JwksVerifier, config::Config, hub::Hub};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub verifier: Arc<JwksVerifier>,
    pub hub: Arc<Hub>,
}

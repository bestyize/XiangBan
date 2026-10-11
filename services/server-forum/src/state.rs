use crate::{auth::JwksVerifier, config::AppConfig};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub verifier: Arc<JwksVerifier>,
    pub http: reqwest::Client,
}

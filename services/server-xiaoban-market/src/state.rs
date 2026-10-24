use crate::{account::AccountVerifier, config::Config, crawler::Runner, upstream::Ctx};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub http: reqwest::Client,
    pub objects: reqwest::Client,
    pub runner: Arc<Runner>,
    pub account: Option<Arc<AccountVerifier>>,
}

impl AppState {
    pub fn ctx(&self) -> Ctx {
        Ctx { http: self.http.clone(), objects: self.objects.clone(), allow_private: self.config.allow_private_upstream }
    }
}

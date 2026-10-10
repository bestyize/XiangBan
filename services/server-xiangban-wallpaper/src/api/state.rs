use crate::{
    admin::AdminService,
    config::AppConfig,
    db::Database,
    providers::ProviderRegistry,
    runtime::Runtime,
    services::{SearchCache, WallpaperAggregator, WallpaperCollector},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub aggregator: Arc<WallpaperAggregator>,
    pub collector: Arc<WallpaperCollector>,
    pub registry: ProviderRegistry,
    pub db: Database,
    pub runtime: Runtime,
    pub cache: SearchCache,
    pub admin: AdminService,
    pub verifier: Arc<crate::auth::JwksVerifier>,
    #[allow(dead_code)]
    pub config: AppConfig,
}

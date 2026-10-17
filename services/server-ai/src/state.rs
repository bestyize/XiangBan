use crate::{config::AppConfig, jwks::JwksVerifier, keys::Keys, newapi::NewApi};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub keys: Arc<Keys>,
    pub verifier: Arc<JwksVerifier>,
    pub newapi: Arc<NewApi>,
    /// 管理员不存在时用于抹平密码校验耗时。
    pub dummy_password_hash: Arc<String>,
}

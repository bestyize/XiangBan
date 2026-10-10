use crate::{config::AppConfig, keys::Keys, sender::Sender};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub keys: Arc<Keys>,
    pub sender: Arc<Sender>,
    /// 账号不存在时用于抹平密码校验耗时。
    pub dummy_password_hash: Arc<String>,
}

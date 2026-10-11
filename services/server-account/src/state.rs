use crate::{config::AppConfig, keys::Keys, ratelimit::WindowLimiter, sender::Sender};
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};

/// 用户名查重每个 IP 每分钟的次数：够边输入边查，挡得住批量探测。
const USERNAME_CHECKS_PER_MINUTE: u32 = 60;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub keys: Arc<Keys>,
    pub sender: Arc<Sender>,
    /// 账号不存在时用于抹平密码校验耗时。
    pub dummy_password_hash: Arc<String>,
    pub username_limiter: Arc<WindowLimiter>,
}

pub fn username_limiter() -> Arc<WindowLimiter> {
    Arc::new(WindowLimiter::new(USERNAME_CHECKS_PER_MINUTE, Duration::from_secs(60)))
}

use crate::{config::AppConfig, jwks::JwksVerifier, keys::Keys, newapi::NewApi, ratelimit::FailureLimiter};
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};

/// 同一个账号在窗口内最多输错几次兑换码。
const REDEEM_FAILURES_PER_USER: u32 = 8;
/// 同一个来源 IP 的上限放宽一些，因为一个出口 IP 后面可能有很多正常用户（宿舍、公司）。
const REDEEM_FAILURES_PER_IP: u32 = 60;
const REDEEM_WINDOW: Duration = Duration::from_secs(15 * 60);

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub keys: Arc<Keys>,
    pub verifier: Arc<JwksVerifier>,
    pub newapi: Arc<NewApi>,
    /// 管理员不存在时用于抹平密码校验耗时。
    pub dummy_password_hash: Arc<String>,
    pub redeem_user_limiter: Arc<FailureLimiter>,
    pub redeem_ip_limiter: Arc<FailureLimiter>,
}

pub fn redeem_limiters() -> (Arc<FailureLimiter>, Arc<FailureLimiter>) {
    (
        Arc::new(FailureLimiter::new(REDEEM_FAILURES_PER_USER, REDEEM_WINDOW)),
        Arc::new(FailureLimiter::new(REDEEM_FAILURES_PER_IP, REDEEM_WINDOW)),
    )
}

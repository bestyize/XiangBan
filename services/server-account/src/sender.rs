use crate::{
    config::SenderConfig,
    error::AppError,
    identifier::{Identifier, Kind},
    tencent::TencentSender,
};

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct SentCode {
    pub target: String,
    pub code: String,
}

pub enum Sender {
    /// 仅开发使用：验证码写入日志。
    Console,
    Tencent(TencentSender),
    #[cfg(test)]
    Capture(std::sync::Mutex<Vec<SentCode>>),
}

impl Sender {
    pub fn from_config(config: &SenderConfig) -> Result<Self, AppError> {
        match config {
            SenderConfig::Console => {
                tracing::warn!("ACCOUNT_SENDER=console：验证码只写入日志，生产环境必须设为 tencent");
                Ok(Sender::Console)
            }
            SenderConfig::Tencent(cfg) => Ok(Sender::Tencent(TencentSender::new(cfg)?)),
        }
    }

    pub async fn send_code(&self, target: &Identifier, code: &str, ttl_minutes: i64) -> Result<(), AppError> {
        match self {
            Sender::Console => {
                tracing::info!("[console sender] 验证码 {code} -> {}", target.masked());
                Ok(())
            }
            Sender::Tencent(t) => match target.kind {
                Kind::Phone => t.send_sms(&target.value, code, ttl_minutes).await,
                Kind::Email => t.send_email(&target.value, code, ttl_minutes).await,
            },
            #[cfg(test)]
            Sender::Capture(sent) => {
                sent.lock().unwrap().push(SentCode { target: target.value.clone(), code: code.to_string() });
                Ok(())
            }
        }
    }
}

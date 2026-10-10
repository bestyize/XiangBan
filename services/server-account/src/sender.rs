use crate::{
    config::SenderConfig,
    error::AppError,
    identifier::{Identifier, Kind},
    tencent::TencentSender,
};
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::sync::Mutex;
use std::sync::{Arc, RwLock};

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct SentCode {
    pub target: String,
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub secure: bool,
    pub username: String,
    pub password: String,
    pub from_name: String,
    pub from_email: String,
}

impl SmtpConfig {
    pub fn build_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, AppError> {
        let creds = Credentials::new(self.username.clone(), self.password.clone());
        let builder = if self.secure || self.port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host)
                .map_err(|e| AppError::BadRequest(format!("SMTP 主机地址无效: {e}")))?
                .port(self.port)
                .credentials(creds)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)
                .map_err(|e| AppError::BadRequest(format!("SMTP 主机地址无效: {e}")))?
                .port(self.port)
                .credentials(creds)
        };
        Ok(builder.build())
    }

    pub async fn send_verification_email(&self, to: &str, code: &str, ttl_minutes: i64) -> Result<(), AppError> {
        let from_mailbox: Mailbox = format!("{} <{}>", self.from_name, self.from_email)
            .parse()
            .map_err(|e| AppError::BadRequest(format!("发件人邮箱格式无效: {e}")))?;
        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|_| AppError::BadRequest("收件人邮箱格式无效".into()))?;

        let subject = format!("【{}】登录验证码：{}", self.from_name, code);
        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background-color: #f7f9fa; padding: 30px 15px; margin: 0;">
  <div style="max-width: 520px; margin: 0 auto; background: #ffffff; border-radius: 12px; padding: 36px 32px; box-shadow: 0 4px 16px rgba(0,0,0,0.06); border: 1px solid #ebeef2;">
    <div style="font-size: 20px; font-weight: 700; color: #1a1a2e; margin-bottom: 24px; border-bottom: 1px solid #f0f2f5; padding-bottom: 16px;">
      {from_name}
    </div>
    <p style="font-size: 15px; color: #4a5568; line-height: 1.6; margin: 0 0 16px;">
      你好！你正在进行相伴身份验证，本次验证码为：
    </p>
    <div style="background: #f0edff; border: 1px dashed #7c3aed; border-radius: 8px; padding: 18px 24px; text-align: center; margin: 24px 0;">
      <span style="font-size: 32px; font-weight: 800; letter-spacing: 6px; color: #6d28d9; font-family: monospace;">
        {code}
      </span>
    </div>
    <p style="font-size: 13px; color: #718096; line-height: 1.6; margin: 0 0 8px;">
      • 验证码有效时间为 <strong>{ttl_minutes} 分钟</strong>，请勿将验证码泄露给他人。<br>
      • 如非本人操作，请忽略此邮件。
    </p>
  </div>
</body>
</html>"#,
            from_name = self.from_name,
            code = code,
            ttl_minutes = ttl_minutes
        );

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| AppError::Internal(format!("构建邮件失败: {e}")))?;

        let transport = self.build_transport()?;
        transport
            .send(email)
            .await
            .map_err(|e| AppError::BadRequest(format!("SMTP 发送失败: {e}")))?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum SenderKind {
    Console,
    Tencent(TencentSender),
    Smtp(SmtpConfig),
    #[cfg(test)]
    Capture(Arc<Mutex<Vec<SentCode>>>),
}

#[derive(Clone)]
pub struct Sender {
    inner: Arc<RwLock<SenderKind>>,
}

impl Sender {
    pub fn from_config(config: &SenderConfig) -> Result<Self, AppError> {
        let kind = match config {
            SenderConfig::Console => {
                tracing::warn!("ACCOUNT_SENDER=console：验证码只写入日志，生产环境必须设为 tencent 或配置 SMTP");
                SenderKind::Console
            }
            SenderConfig::Tencent(cfg) => SenderKind::Tencent(TencentSender::new(cfg)?),
        };
        Ok(Self {
            inner: Arc::new(RwLock::new(kind)),
        })
    }

    #[cfg(test)]
    pub fn capture() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SenderKind::Capture(Arc::new(Mutex::new(Vec::new()))))),
        }
    }

    #[cfg(test)]
    pub fn last_code(&self) -> Option<String> {
        let guard = self.inner.read().unwrap();
        match &*guard {
            SenderKind::Capture(sent) => sent.lock().unwrap().last().map(|s| s.code.clone()),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn sent_count(&self) -> usize {
        let guard = self.inner.read().unwrap();
        match &*guard {
            SenderKind::Capture(sent) => sent.lock().unwrap().len(),
            _ => 0,
        }
    }

    pub fn update(&self, new_kind: SenderKind) {
        let mut guard = self.inner.write().unwrap();
        *guard = new_kind;
    }

    pub fn current_type(&self) -> String {
        let guard = self.inner.read().unwrap();
        match &*guard {
            SenderKind::Console => "console".into(),
            SenderKind::Tencent(_) => "tencent".into(),
            SenderKind::Smtp(_) => "smtp".into(),
            #[cfg(test)]
            SenderKind::Capture(_) => "capture".into(),
        }
    }

    pub fn get_smtp_config(&self) -> Option<SmtpConfig> {
        let guard = self.inner.read().unwrap();
        match &*guard {
            SenderKind::Smtp(cfg) => Some(cfg.clone()),
            _ => None,
        }
    }

    pub async fn send_code(&self, target: &Identifier, code: &str, ttl_minutes: i64) -> Result<(), AppError> {
        let kind = { self.inner.read().unwrap().clone() };
        match kind {
            SenderKind::Console => {
                tracing::info!("[console sender] 验证码 {code} -> {}", target.masked());
                Ok(())
            }
            SenderKind::Tencent(t) => match target.kind {
                Kind::Phone => t.send_sms(&target.value, code, ttl_minutes).await,
                Kind::Email => t.send_email(&target.value, code, ttl_minutes).await,
            },
            SenderKind::Smtp(smtp) => match target.kind {
                Kind::Email => smtp.send_verification_email(&target.value, code, ttl_minutes).await,
                Kind::Phone => Err(AppError::BadRequest("当前已配置 SMTP 发信通道，暂不支持手机短信发送，请使用邮箱验证码".into())),
            },
            #[cfg(test)]
            SenderKind::Capture(sent) => {
                sent.lock().unwrap().push(SentCode {
                    target: target.value.clone(),
                    code: code.to_string(),
                });
                Ok(())
            }
        }
    }

    pub async fn send_test_email(&self, to: &str, temp_cfg: Option<&SmtpConfig>) -> Result<(), AppError> {
        if let Some(cfg) = temp_cfg {
            cfg.send_verification_email(to, "888888", 10).await
        } else {
            let kind = { self.inner.read().unwrap().clone() };
            match kind {
                SenderKind::Smtp(cfg) => cfg.send_verification_email(to, "888888", 10).await,
                SenderKind::Console => {
                    tracing::info!("[console test] 测试验证码 888888 -> {to}");
                    Ok(())
                }
                SenderKind::Tencent(t) => t.send_email(to, "888888", 10).await,
                #[cfg(test)]
                SenderKind::Capture(_) => Ok(()),
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedSenderConfig {
    pub provider: String,
    pub smtp: Option<SmtpConfig>,
}

impl SavedSenderConfig {
    pub fn to_sender_kind(&self, app_cfg: &crate::config::AppConfig) -> Result<SenderKind, AppError> {
        match self.provider.as_str() {
            "smtp" => {
                let smtp = self
                    .smtp
                    .as_ref()
                    .ok_or_else(|| AppError::BadRequest("缺少 SMTP 配置参数".into()))?;
                Ok(SenderKind::Smtp(smtp.clone()))
            }
            "tencent" => {
                if let SenderConfig::Tencent(cfg) = &app_cfg.sender {
                    Ok(SenderKind::Tencent(TencentSender::new(cfg)?))
                } else {
                    Err(AppError::BadRequest("环境变量中未配置腾讯云发信密钥".into()))
                }
            }
            _ => Ok(SenderKind::Console),
        }
    }
}

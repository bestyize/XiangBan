use crate::{
    config::{SesConfig, SmsConfig, TencentConfig},
    crypto,
    error::AppError,
};
use chrono::{TimeZone, Utc};
use ring::hmac;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone)]
pub struct TencentSender {
    client: reqwest::Client,
    secret_id: String,
    secret_key: String,
    sms: Option<SmsConfig>,
    ses: Option<SesConfig>,
}

fn hmac_sha256(key: &[u8], msg: &str) -> Vec<u8> {
    hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, key), msg.as_bytes()).as_ref().to_vec()
}

/// 腾讯云 API 3.0 TC3-HMAC-SHA256 签名，返回 Authorization 头。
pub fn authorization(secret_id: &str, secret_key: &str, service: &str, host: &str, action: &str, payload: &str, ts: i64) -> String {
    let date = Utc.timestamp_opt(ts, 0).single().unwrap_or_default().format("%Y-%m-%d").to_string();
    let canonical = format!(
        "POST\n/\n\ncontent-type:application/json; charset=utf-8\nhost:{host}\nx-tc-action:{}\n\ncontent-type;host;x-tc-action\n{}",
        action.to_lowercase(),
        crypto::hex(&crypto::sha256(payload.as_bytes()))
    );
    let scope = format!("{date}/{service}/tc3_request");
    let to_sign = format!(
        "TC3-HMAC-SHA256\n{ts}\n{scope}\n{}",
        crypto::hex(&crypto::sha256(canonical.as_bytes()))
    );
    let k_date = hmac_sha256(format!("TC3{secret_key}").as_bytes(), &date);
    let k_service = hmac_sha256(&k_date, service);
    let k_signing = hmac_sha256(&k_service, "tc3_request");
    let signature = crypto::hex(&hmac_sha256(&k_signing, &to_sign));
    format!(
        "TC3-HMAC-SHA256 Credential={secret_id}/{scope}, SignedHeaders=content-type;host;x-tc-action, Signature={signature}"
    )
}

impl TencentSender {
    pub fn new(config: &TencentConfig) -> Result<Self, AppError> {
        if config.secret_id.is_empty() || config.secret_key.is_empty() {
            return Err(AppError::Internal("ACCOUNT_SENDER=tencent 需要 TENCENT_SECRET_ID / TENCENT_SECRET_KEY".into()));
        }
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| AppError::Internal(format!("http client: {e}")))?,
            secret_id: config.secret_id.clone(),
            secret_key: config.secret_key.clone(),
            sms: config.sms.clone(),
            ses: config.ses.clone(),
        })
    }

    async fn call(&self, service: &str, host: &str, action: &str, version: &str, region: &str, payload: Value) -> Result<Value, AppError> {
        let body = payload.to_string();
        let ts = Utc::now().timestamp();
        let auth = authorization(&self.secret_id, &self.secret_key, service, host, action, &body, ts);
        let response = self
            .client
            .post(format!("https://{host}/"))
            .header("Authorization", auth)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("X-TC-Action", action)
            .header("X-TC-Version", version)
            .header("X-TC-Timestamp", ts.to_string())
            .header("X-TC-Region", region)
            .body(body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("tencent {service}/{action} request failed: {e}");
                AppError::BadGateway("验证码发送失败，请稍后再试".into())
            })?;
        let json: Value = response.json().await.map_err(|e| {
            tracing::error!("tencent {service}/{action} bad response: {e}");
            AppError::BadGateway("验证码发送失败，请稍后再试".into())
        })?;
        if let Some(err) = json.pointer("/Response/Error") {
            tracing::error!("tencent {service}/{action} error: {err}");
            return Err(AppError::BadGateway("验证码发送失败，请稍后再试".into()));
        }
        Ok(json)
    }

    pub async fn send_sms(&self, phone: &str, code: &str, minutes: i64) -> Result<(), AppError> {
        let sms = self
            .sms
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("暂未开通手机号验证码，请使用邮箱".into()))?;
        let payload = json!({
            "PhoneNumberSet": [phone],
            "SmsSdkAppId": sms.sdk_app_id,
            "SignName": sms.sign_name,
            "TemplateId": sms.template_id,
            "TemplateParamSet": [code, minutes.to_string()],
        });
        let json = self
            .call("sms", "sms.tencentcloudapi.com", "SendSms", "2021-01-11", &sms.region, payload)
            .await?;
        let status = json.pointer("/Response/SendStatusSet/0/Code").and_then(Value::as_str);
        if status != Some("Ok") {
            tracing::error!("tencent sms rejected: {:?}", json.pointer("/Response/SendStatusSet/0"));
            return Err(AppError::BadGateway("验证码发送失败，请稍后再试".into()));
        }
        Ok(())
    }

    pub async fn send_email(&self, email: &str, code: &str, minutes: i64) -> Result<(), AppError> {
        let ses = self
            .ses
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("暂未开通邮箱验证码，请使用手机号".into()))?;
        let payload = json!({
            "FromEmailAddress": ses.from,
            "Destination": [email],
            "Subject": ses.subject,
            "Template": {
                "TemplateID": ses.template_id,
                "TemplateData": json!({ "code": code, "minutes": minutes.to_string() }).to_string(),
            },
        });
        self.call("ses", "ses.tencentcloudapi.com", "SendEmail", "2020-10-02", &ses.region, payload)
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_has_expected_shape_and_is_deterministic() {
        let a = authorization("AKID", "secret", "sms", "sms.tencentcloudapi.com", "SendSms", "{}", 1_700_000_000);
        let b = authorization("AKID", "secret", "sms", "sms.tencentcloudapi.com", "SendSms", "{}", 1_700_000_000);
        assert_eq!(a, b);
        assert!(a.starts_with("TC3-HMAC-SHA256 Credential=AKID/2023-11-14/sms/tc3_request, SignedHeaders=content-type;host;x-tc-action, Signature="));
        assert_eq!(a.rsplit("Signature=").next().unwrap().len(), 64);
        assert_ne!(a, authorization("AKID", "secret", "sms", "sms.tencentcloudapi.com", "SendSms", "{\"a\":1}", 1_700_000_000));
    }
}

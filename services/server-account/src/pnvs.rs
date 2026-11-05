//! 阿里云号码认证（非普通 SMS）。使用预置签名、模板发送本地生成的验证码，
//! 校验仍走账号中心的一次性 HMAC 记录、用途隔离、过期与尝试次数限制。
use crate::{codes::Purpose, config::PnvsConfig, crypto, error::AppError};
use chrono::Utc;
use ring::hmac;
use serde_json::{json, Value};
use std::{collections::BTreeMap, time::Duration};

const HOST: &str = "dypnsapi.aliyuncs.com";
const ACTION: &str = "SendSmsVerifyCode";
const VERSION: &str = "2017-05-25";

#[derive(Clone)]
pub struct PnvsSender {
    client: reqwest::Client,
    config: PnvsConfig,
    endpoint: String,
}

/// ACS3 使用 RFC3986 编码，空格为 %20，不能使用表单编码中的 +。
fn encode(value: &str) -> String {
    let mut result = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            result.push(byte as char);
        } else {
            use std::fmt::Write;
            write!(result, "%{byte:02X}").unwrap();
        }
    }
    result
}

fn authorization(config: &PnvsConfig, query: &str, date: &str, nonce: &str) -> String {
    let payload_hash = crypto::hex(&crypto::sha256(b""));
    let headers = format!(
        "host:{HOST}\nx-acs-action:{ACTION}\nx-acs-content-sha256:{payload_hash}\nx-acs-date:{date}\nx-acs-signature-nonce:{nonce}\nx-acs-version:{VERSION}\n"
    );
    let signed = "host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version";
    let canonical = format!("POST\n/\n{query}\n{headers}\n{signed}\n{payload_hash}");
    let to_sign = format!("ACS3-HMAC-SHA256\n{}", crypto::hex(&crypto::sha256(canonical.as_bytes())));
    let signature = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, config.access_key_secret.as_bytes()), to_sign.as_bytes());
    format!("ACS3-HMAC-SHA256 Credential={},SignedHeaders={signed},Signature={}", config.access_key_id, crypto::hex(signature.as_ref()))
}

impl PnvsSender {
    pub fn new(config: &PnvsConfig) -> Result<Self, AppError> {
        if [&config.access_key_id, &config.access_key_secret, &config.sign_name].iter().any(|s| s.trim().is_empty()) {
            return Err(AppError::Internal("启用 aliyun_pnvs 需要配置 ALIYUN_PNVS_ACCESS_KEY_ID、ALIYUN_PNVS_ACCESS_KEY_SECRET 和 ALIYUN_PNVS_SIGN_NAME".into()));
        }
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| AppError::Internal("无法创建短信客户端".into()))?,
            config: config.clone(),
            endpoint: format!("https://{HOST}/"),
        })
    }

    fn query(&self, phone: &str, code: &str, minutes: i64, purpose: Purpose) -> String {
        let template = match purpose {
            Purpose::Login => "100001",
            Purpose::ResetPassword => "100003",
            Purpose::Bind => "100005",
        };
        let mut params = BTreeMap::from([
            ("CountryCode", "86".to_string()),
            ("PhoneNumber", phone.strip_prefix("+86").unwrap_or(phone).to_string()),
            ("SignName", self.config.sign_name.clone()),
            ("TemplateCode", template.to_string()),
            ("TemplateParam", json!({"code": code, "min": minutes.to_string()}).to_string()),
            ("ReturnVerifyCode", "false".to_string()),
        ]);
        if let Some(name) = &self.config.scheme_name {
            params.insert("SchemeName", name.clone());
        }
        params.into_iter().map(|(k, v)| format!("{}={}", encode(k), encode(&v))).collect::<Vec<_>>().join("&")
    }

    pub async fn send_sms(&self, phone: &str, code: &str, minutes: i64, purpose: Purpose) -> Result<(), AppError> {
        let query = self.query(phone, code, minutes, purpose);
        let date = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = uuid::Uuid::new_v4().to_string();
        let response = self.client.post(format!("{}?{query}", self.endpoint))
            .header("host", HOST)
            .header("x-acs-action", ACTION)
            .header("x-acs-version", VERSION)
            .header("x-acs-date", &date)
            .header("x-acs-signature-nonce", &nonce)
            .header("x-acs-content-sha256", crypto::hex(&crypto::sha256(b"")))
            .header("authorization", authorization(&self.config, &query, &date, &nonce))
            .send().await.map_err(|_| {
                // 请求 URL 包含手机号和验证码，不记录 reqwest 错误或完整请求。
                tracing::warn!("PNVS 短信请求失败");
                AppError::BadGateway("验证码发送失败，请稍后再试".into())
            })?;
        let status = response.status();
        let body: Value = response.json().await.map_err(|_| AppError::BadGateway("验证码发送失败，请稍后再试".into()))?;
        if !status.is_success() || body["Code"].as_str() != Some("OK") || body["Success"].as_bool() != Some(true) {
            // 不记录响应正文（可能包含验证码、手机号），仅记录诊断标识。
            tracing::warn!(status = %status, code = ?body["Code"].as_str(), request_id = ?body["RequestId"].as_str(), "PNVS 短信被拒绝");
            return Err(AppError::BadGateway("验证码发送失败，请稍后再试".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> PnvsConfig {
        PnvsConfig { access_key_id: "test-id".into(), access_key_secret: "test-secret".into(), sign_name: "测试签名".into(), scheme_name: None }
    }

    #[test]
    fn query_uses_pnvs_templates_local_code_and_rfc3986() {
        let sender = PnvsSender::new(&config()).unwrap();
        let query = sender.query("+8613800138000", "012345", 5, Purpose::Login);
        assert!(query.contains("PhoneNumber=13800138000"));
        assert!(query.contains("TemplateCode=100001"));
        assert!(query.contains("TemplateParam=%7B%22code%22%3A%22012345%22%2C%22min%22%3A%225%22%7D"));
        assert!(sender.query("13800138000", "012345", 5, Purpose::ResetPassword).contains("TemplateCode=100003"));
        assert!(sender.query("13800138000", "012345", 5, Purpose::Bind).contains("TemplateCode=100005"));
        assert_eq!(encode("a +/~*"), "a%20%2B%2F~%2A");
        let mut missing = config();
        missing.sign_name.clear();
        assert!(PnvsSender::new(&missing).is_err());
    }

    #[test]
    fn acs3_signature_matches_independent_hmac_vector() {
        // 使用 Python hashlib/hmac 独立计算的固定向量，防止规范化或签名拼接回归。
        let auth = authorization(&config(), "PhoneNumber=13800138000", "2026-11-05T00:00:00Z", "fixed-nonce");
        assert!(auth.ends_with("Signature=775d861391df51fd14ebd6cdcd95f81503eb4c00ef12056cccb7fc6f59727e4c"));
    }

    #[tokio::test]
    async fn sends_signed_request_and_rejects_upstream_failures() {
        use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};
        for (status, body, ok) in [
            (200, r#"{"Code":"OK","Success":true}"#, true),
            (200, r#"{"Code":"BUSINESS_LIMIT_CONTROL","Success":false}"#, false),
            (200, r#"{"Code":"OK","Success":false}"#, false),
            (500, r#"{"Code":"OK","Success":true}"#, false),
            (200, "not json", false),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let mut sender = PnvsSender::new(&config()).unwrap();
            sender.endpoint = format!("http://{}/", listener.local_addr().unwrap());
            let server = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                loop {
                    let mut buf = [0; 4096];
                    let n = socket.read(&mut buf).await.unwrap();
                    assert!(n > 0);
                    request.extend_from_slice(&buf[..n]);
                    if request.windows(4).any(|w| w == b"\r\n\r\n") { break; }
                }
                let request = String::from_utf8(request).unwrap();
                assert!(request.starts_with("POST /?CountryCode=86&"));
                assert!(request.contains("authorization: ACS3-HMAC-SHA256 Credential=test-id,"));
                assert!(request.contains("x-acs-action: SendSmsVerifyCode"));
                socket.write_all(format!("HTTP/1.1 {status} Response\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            });
            assert_eq!(sender.send_sms("+8613800138000", "012345", 5, Purpose::Login).await.is_ok(), ok);
            server.await.unwrap();
        }
    }
}

//! 账号中心访问令牌的离线验签（JWKS 公钥，EdDSA）。
//! 与论坛等业务服务不同，这里同时服务多个产品，所以不固定 `aud`，而是把它交给调用方去匹配产品。

use crate::error::AppError;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct AccessClaims {
    sub: String,
    aud: String,
}

#[derive(Deserialize)]
struct JwksResponse {
    keys: Vec<JwkItem>,
}

#[derive(Deserialize)]
struct JwkItem {
    kty: String,
    kid: Option<String>,
    x: Option<String>,
}

struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    expires_at: Instant,
}

pub struct Verified {
    pub uid: Uuid,
    pub client_id: String,
}

pub struct JwksVerifier {
    http: reqwest::Client,
    issuer: String,
    jwks_url: String,
    cache: RwLock<JwksCache>,
    /// 测试用：固定公钥，不拉取 JWKS。
    static_key: Option<DecodingKey>,
}

impl JwksVerifier {
    pub fn new(issuer: String, jwks_url: String) -> Self {
        Self {
            http: reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap_or_default(),
            issuer,
            jwks_url,
            cache: RwLock::new(JwksCache { keys: HashMap::new(), expires_at: Instant::now() }),
            static_key: None,
        }
    }

    #[cfg(test)]
    pub fn with_static_key(issuer: String, key: DecodingKey) -> Self {
        let mut v = Self::new(issuer, String::new());
        v.static_key = Some(key);
        v
    }

    pub async fn verify(&self, token: &str) -> Result<Verified, AppError> {
        let header = decode_header(token).map_err(|_| AppError::Unauthorized("登录凭证格式无效".into()))?;
        if header.alg != Algorithm::EdDSA {
            return Err(AppError::Unauthorized("不支持的签名算法".into()));
        }
        let key = self.key_for(header.kid.as_deref()).await?;

        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[&self.issuer]);
        validation.validate_aud = false;
        validation.leeway = 10;

        let data = decode::<AccessClaims>(token, &key, &validation).map_err(|err| match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::Unauthorized("登录已过期，请重新登录".into()),
            _ => AppError::Unauthorized("登录凭证验证失败".into()),
        })?;
        let uid = Uuid::parse_str(&data.claims.sub).map_err(|_| AppError::Unauthorized("登录凭证无效".into()))?;
        Ok(Verified { uid, client_id: data.claims.aud })
    }

    async fn key_for(&self, kid: Option<&str>) -> Result<DecodingKey, AppError> {
        if let Some(key) = &self.static_key {
            return Ok(key.clone());
        }
        {
            let cache = self.cache.read().await;
            if Instant::now() < cache.expires_at {
                if let Some(key) = kid.and_then(|k| cache.keys.get(k)) {
                    return Ok(key.clone());
                }
            }
        }
        // 缓存过期或出现新 kid（账号中心轮换密钥）时重新拉取。
        self.refresh().await?;
        let cache = self.cache.read().await;
        match kid {
            Some(k) => cache.keys.get(k).cloned().ok_or_else(|| AppError::Unauthorized("未知的签名密钥".into())),
            None => cache.keys.values().next().cloned().ok_or_else(|| AppError::Unauthorized("没有可用的签名公钥".into())),
        }
    }

    async fn refresh(&self) -> Result<(), AppError> {
        let resp = self.http.get(&self.jwks_url).send().await.map_err(|e| {
            tracing::warn!("拉取 JWKS 失败 {}: {e}", self.jwks_url);
            AppError::ServiceUnavailable("账号服务暂时不可用".into())
        })?;
        if !resp.status().is_success() {
            return Err(AppError::ServiceUnavailable("账号服务暂时不可用".into()));
        }
        let jwks: JwksResponse = resp.json().await.map_err(|e| AppError::Internal(format!("解析 JWKS 失败: {e}")))?;
        let keys: HashMap<_, _> = jwks
            .keys
            .into_iter()
            .filter(|k| k.kty == "OKP")
            .filter_map(|k| Some((k.kid?, DecodingKey::from_ed_components(&k.x?).ok()?)))
            .collect();
        if keys.is_empty() {
            return Err(AppError::Internal("JWKS 中没有 Ed25519 公钥".into()));
        }
        *self.cache.write().await = JwksCache { keys, expires_at: Instant::now() + Duration::from_secs(3600) };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Claims {
        iss: String,
        sub: String,
        aud: String,
        exp: i64,
    }

    #[tokio::test]
    async fn verifies_issuer_and_expiry_and_returns_audience() {
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let x = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
        let enc = EncodingKey::from_ed_der(pkcs8.as_ref());
        let verifier = JwksVerifier::with_static_key("https://account.test".into(), DecodingKey::from_ed_components(&x).unwrap());

        let uid = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp();
        let sign = |iss: &str, aud: &str, exp: i64| {
            let claims = Claims { iss: iss.into(), sub: uid.to_string(), aud: aud.into(), exp };
            encode(&Header::new(Algorithm::EdDSA), &claims, &enc).unwrap()
        };

        let ok = verifier.verify(&sign("https://account.test", "xiangban-wallpaper-web", now + 600)).await.unwrap();
        assert_eq!((ok.uid, ok.client_id.as_str()), (uid, "xiangban-wallpaper-web"));
        assert!(verifier.verify(&sign("https://account.test", "x", now - 600)).await.is_err());
        assert!(verifier.verify(&sign("https://evil.test", "x", now + 600)).await.is_err());
        assert!(verifier.verify("not-a-jwt").await.is_err());
    }
}

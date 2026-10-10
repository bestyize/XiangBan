use crate::{api::state::AppState, config::AppConfig, error::AppError};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    #[serde(default)]
    pub iat: i64,
    #[serde(default)]
    pub jti: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    pub uid: Uuid,
    pub client_id: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts).ok_or_else(|| AppError::Unauthorized("缺少登录凭证".into()))?;
        state.verifier.verify(token).await
    }
}

/// 可选鉴权提取器：未携带 Authorization 头返回 None；携带了但凭证无效则返回 Unauthorized 错误。
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        match extract_bearer_token(parts) {
            Some(token) => state.verifier.verify(token).await.map(|u| OptionalAuthUser(Some(u))),
            None => Ok(OptionalAuthUser(None)),
        }
    }
}

fn extract_bearer_token<'a>(parts: &'a Parts) -> Option<&'a str> {
    parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
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

pub struct JwksVerifier {
    http: reqwest::Client,
    issuer: String,
    jwks_url: String,
    allowed_clients: Vec<String>,
    cache: Arc<RwLock<JwksCache>>,
    static_key: Option<DecodingKey>,
}

impl JwksVerifier {
    pub fn new(config: &AppConfig) -> Self {
        let static_key = config.account_public_key.as_deref().and_then(|pk| {
            DecodingKey::from_ed_components(pk).ok()
        });

        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            issuer: config.account_issuer.clone(),
            jwks_url: config.resolved_jwks_url(),
            allowed_clients: config.account_allowed_clients.clone(),
            cache: Arc::new(RwLock::new(JwksCache {
                keys: HashMap::new(),
                expires_at: Instant::now(),
            })),
            static_key,
        }
    }

    pub fn with_static_key(issuer: String, allowed_clients: Vec<String>, key: DecodingKey) -> Self {
        Self {
            http: reqwest::Client::new(),
            issuer,
            jwks_url: String::new(),
            allowed_clients,
            cache: Arc::new(RwLock::new(JwksCache {
                keys: HashMap::new(),
                expires_at: Instant::now() + Duration::from_secs(3600),
            })),
            static_key: Some(key),
        }
    }

    pub async fn verify(&self, token: &str) -> Result<AuthUser, AppError> {
        let header = decode_header(token).map_err(|_| AppError::Unauthorized("登录凭证格式无效".into()))?;
        if header.alg != Algorithm::EdDSA {
            return Err(AppError::Unauthorized("不支持的签名算法".into()));
        }

        let kid = header.kid;
        let decoding_key = self.get_decoding_key(kid.as_deref()).await?;

        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&self.allowed_clients);
        validation.leeway = 10;

        let token_data = decode::<AccessClaims>(token, &decoding_key, &validation)
            .map_err(|err| match err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Unauthorized("登录已过期，请重新登录".into())
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                    AppError::Unauthorized("登录凭证签发者不匹配".into())
                }
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    AppError::Unauthorized("该凭证无权访问小伴壁纸服务".into())
                }
                _ => AppError::Unauthorized("登录凭证验证失败".into()),
            })?;

        let uid = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Unauthorized("用户唯一标识格式无效".into()))?;

        Ok(AuthUser {
            uid,
            client_id: token_data.claims.aud,
        })
    }

    async fn get_decoding_key(&self, kid: Option<&str>) -> Result<DecodingKey, AppError> {
        // 1. 如果配置了静态密钥，且没有提供 kid 或匹配，优先返回
        if let Some(static_key) = &self.static_key {
            return Ok(static_key.clone());
        }

        // 2. 尝试从缓存中读取
        {
            let cache = self.cache.read().await;
            if Instant::now() < cache.expires_at {
                if let Some(kid) = kid {
                    if let Some(key) = cache.keys.get(kid) {
                        return Ok(key.clone());
                    }
                } else if let Some((_, first_key)) = cache.keys.iter().next() {
                    return Ok(first_key.clone());
                }
            }
        }

        // 3. 缓存未命中或已过期：拉取最新 JWKS
        self.refresh_jwks().await?;

        // 4. 再次查找
        let cache = self.cache.read().await;
        if let Some(kid) = kid {
            cache.keys.get(kid).cloned().ok_or_else(|| {
                AppError::Unauthorized(format!("未知签名密钥 kid: {kid}"))
            })
        } else {
            cache.keys.values().next().cloned().ok_or_else(|| {
                AppError::Unauthorized("未找到可用的签名公钥".into())
            })
        }
    }

    async fn refresh_jwks(&self) -> Result<(), AppError> {
        if self.jwks_url.is_empty() {
            return Err(AppError::Unauthorized("未配置 JWKS 公钥地址".into()));
        }

        debug!("Fetching JWKS from {}", self.jwks_url);
        let resp = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to fetch JWKS from {}: {e}", self.jwks_url);
                AppError::ServiceUnavailable("账号认证服务不可用".into())
            })?;

        if !resp.status().is_success() {
            warn!("JWKS endpoint returned status {}", resp.status());
            return Err(AppError::ServiceUnavailable("账号认证服务返回异常".into()));
        }

        let jwks: JwksResponse = resp.json().await.map_err(|e| {
            warn!("Failed to parse JWKS json: {e}");
            AppError::Internal(format!("解析 JWKS 响应失败: {e}"))
        })?;

        let mut new_keys = HashMap::new();
        for item in jwks.keys {
            if item.kty == "OKP" {
                if let (Some(kid), Some(x)) = (item.kid, item.x) {
                    if let Ok(key) = DecodingKey::from_ed_components(&x) {
                        new_keys.insert(kid, key);
                    }
                }
            }
        }

        if new_keys.is_empty() {
            return Err(AppError::Internal("JWKS 中未找到有效的 Ed25519 签名公钥".into()));
        }

        let mut cache = self.cache.write().await;
        cache.keys = new_keys;
        cache.expires_at = Instant::now() + Duration::from_secs(3600); // 缓存 1 小时
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[tokio::test]
    async fn test_auth_user_verification() {
        // 生成测试用 Ed25519 密钥对
        // Ed25519 PKCS8 示例或使用 ring 生成
        let key_pair = ring::signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = ring::signature::Ed25519KeyPair::from_pkcs8(key_pair.as_ref()).unwrap();
        use ring::signature::KeyPair;
        let public_bytes = pair.public_key().as_ref();
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use base64::Engine;
        let x_b64 = URL_SAFE_NO_PAD.encode(public_bytes);

        let encoding = EncodingKey::from_ed_der(key_pair.as_ref());
        let decoding = DecodingKey::from_ed_components(&x_b64).unwrap();

        let issuer = "https://account.xiangban.xyz".to_string();
        let client_id = "xiangban-wallpaper-web".to_string();
        let verifier = JwksVerifier::with_static_key(
            issuer.clone(),
            vec![client_id.clone(), "xiaoban-desktop-wallpaper".into()],
            decoding,
        );

        let uid = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp();
        let claims = AccessClaims {
            iss: issuer.clone(),
            sub: uid.to_string(),
            aud: client_id.clone(),
            exp: now + 3600,
            iat: now,
            jti: Some("test-jti".into()),
        };

        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some("test-kid".into());
        let token = encode(&header, &claims, &encoding).unwrap();

        // 验证有效 token
        let user = verifier.verify(&token).await.unwrap();
        assert_eq!(user.uid, uid);
        assert_eq!(user.client_id, client_id);

        // 验证过期 token
        let expired_claims = AccessClaims {
            exp: now - 100,
            ..claims.clone()
        };
        let expired_token = encode(&header, &expired_claims, &encoding).unwrap();
        let err = verifier.verify(&expired_token).await.unwrap_err();
        assert!(matches!(err, AppError::Unauthorized(_)));

        // 验证非法 issuer
        let wrong_iss_claims = AccessClaims {
            iss: "https://evil.example.com".into(),
            ..claims.clone()
        };
        let wrong_iss_token = encode(&header, &wrong_iss_claims, &encoding).unwrap();
        let err = verifier.verify(&wrong_iss_token).await.unwrap_err();
        assert!(matches!(err, AppError::Unauthorized(_)));

        // 验证非法 audience
        let wrong_aud_claims = AccessClaims {
            aud: "unauthorized-client".into(),
            ..claims
        };
        let wrong_aud_token = encode(&header, &wrong_aud_claims, &encoding).unwrap();
        let err = verifier.verify(&wrong_aud_token).await.unwrap_err();
        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}

//! 账号中心访问令牌的离线验签（JWKS 公钥，EdDSA），以及请求里的登录用户提取器。

use crate::{error::AppError, state::AppState, users::{self, ForumUser}};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
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
    /// 接受的令牌受众：网页的 `FORUM_ACCOUNT_CLIENT_ID`，加上手机应用等其他客户端。
    audiences: Vec<String>,
    cache: RwLock<JwksCache>,
    /// 测试用：固定公钥，不拉取 JWKS。
    static_key: Option<DecodingKey>,
}

impl JwksVerifier {
    pub fn new(issuer: String, jwks_url: String, client_id: String) -> Self {
        Self {
            http: reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap_or_default(),
            issuer,
            jwks_url,
            audiences: vec![client_id],
            cache: RwLock::new(JwksCache { keys: HashMap::new(), expires_at: Instant::now() }),
            static_key: None,
        }
    }

    /// 追加可接受的受众（例如 `xiaoban-android`）；重复或空值忽略。
    pub fn with_extra_audiences(mut self, extra: &[String]) -> Self {
        for aud in extra {
            let aud = aud.trim();
            if !aud.is_empty() && !self.audiences.iter().any(|a| a == aud) {
                self.audiences.push(aud.to_string());
            }
        }
        self
    }

    #[cfg(test)]
    pub fn with_static_key(issuer: String, client_id: String, key: DecodingKey) -> Self {
        let mut v = Self::new(issuer, String::new(), client_id);
        v.static_key = Some(key);
        v
    }

    pub async fn verify(&self, token: &str) -> Result<Uuid, AppError> {
        let header = decode_header(token).map_err(|_| AppError::Unauthorized("登录凭证格式无效".into()))?;
        if header.alg != Algorithm::EdDSA {
            return Err(AppError::Unauthorized("不支持的签名算法".into()));
        }
        let key = self.key_for(header.kid.as_deref()).await?;

        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&self.audiences);
        validation.leeway = 10;

        let data = decode::<AccessClaims>(token, &key, &validation).map_err(|err| match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::Unauthorized("登录已过期，请重新登录".into()),
            _ => AppError::Unauthorized("登录凭证验证失败".into()),
        })?;
        Uuid::parse_str(&data.claims.sub).map_err(|_| AppError::Unauthorized("登录凭证无效".into()))
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
        let jwks: JwksResponse =
            resp.json().await.map_err(|e| AppError::Internal(format!("解析 JWKS 失败: {e}")))?;
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

pub fn bearer(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// 已登录用户。首次出现的 uid 会在本地建档；被封禁的用户直接拒绝。
pub struct AuthUser {
    pub user: ForumUser,
    pub token: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer(parts).ok_or_else(|| AppError::Unauthorized("请先登录".into()))?.to_string();
        let uid = state.verifier.verify(&token).await?;
        let user = users::load_or_create(&state.pool, uid).await?;
        if user.status == "banned" {
            return Err(AppError::Forbidden("账号已被封禁".into()));
        }
        Ok(AuthUser { user, token })
    }
}

/// 可选登录：没带凭证为 None；带了但无效则报 401，让前端及时刷新令牌。
pub struct MaybeUser(pub Option<ForumUser>);

impl FromRequestParts<AppState> for MaybeUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        match bearer(parts) {
            None => Ok(MaybeUser(None)),
            Some(token) => {
                let uid = state.verifier.verify(token).await?;
                Ok(MaybeUser(users::find(&state.pool, uid).await?))
            }
        }
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
    async fn verifies_issuer_audience_and_expiry() {
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let x = URL_SAFE_NO_PAD.encode(pair.public_key().as_ref());
        let enc = EncodingKey::from_ed_der(pkcs8.as_ref());
        let verifier = JwksVerifier::with_static_key(
            "https://account.test".into(),
            "forum-web".into(),
            DecodingKey::from_ed_components(&x).unwrap(),
        );

        let uid = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp();
        let sign = |iss: &str, aud: &str, exp: i64| {
            let claims = Claims { iss: iss.into(), sub: uid.to_string(), aud: aud.into(), exp };
            encode(&Header::new(Algorithm::EdDSA), &claims, &enc).unwrap()
        };

        assert_eq!(verifier.verify(&sign("https://account.test", "forum-web", now + 600)).await.unwrap(), uid);
        assert!(verifier.verify(&sign("https://account.test", "forum-web", now - 600)).await.is_err());
        assert!(verifier.verify(&sign("https://evil.test", "forum-web", now + 600)).await.is_err());
        assert!(verifier.verify(&sign("https://account.test", "xiaban-web", now + 600)).await.is_err());
        // 没有追加受众时，手机应用的令牌不被接受。
        assert!(verifier.verify(&sign("https://account.test", "xiaoban-android", now + 600)).await.is_err());

        let verifier = verifier.with_extra_audiences(&["xiaoban-android".into(), " ".into(), "forum-web".into()]);
        assert_eq!(verifier.audiences, vec!["forum-web".to_string(), "xiaoban-android".to_string()]);
        assert_eq!(verifier.verify(&sign("https://account.test", "xiaoban-android", now + 600)).await.unwrap(), uid);
        assert_eq!(verifier.verify(&sign("https://account.test", "forum-web", now + 600)).await.unwrap(), uid);
        assert!(verifier.verify(&sign("https://account.test", "xiaban-web", now + 600)).await.is_err(), "其他受众仍然被拒绝");
        assert!(verifier.verify(&sign("https://evil.test", "xiaoban-android", now + 600)).await.is_err());
    }
}

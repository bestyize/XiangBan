//! 相伴账号中心登录：用 JWKS 公钥离线验签账号中心签发的访问令牌（EdDSA），不回源、不共享密钥。
//! 令牌的 `iss` 必须等于配置的 issuer，`aud` 必须等于本应用登记在账号中心的 client_id。
use crate::error::AppError;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Deserialize)]
struct Claims {
    sub: String,
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kty: String,
    kid: Option<String>,
    x: Option<String>,
}

struct Cache {
    keys: HashMap<String, DecodingKey>,
    expires_at: Instant,
}

pub struct AccountVerifier {
    http: reqwest::Client,
    issuer: String,
    jwks_url: String,
    client_id: String,
    cache: RwLock<Cache>,
}

impl AccountVerifier {
    pub fn new(issuer: String, jwks_url: String, client_id: String) -> Self {
        Self {
            http: reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap_or_default(),
            issuer,
            jwks_url,
            client_id,
            cache: RwLock::new(Cache { keys: HashMap::new(), expires_at: Instant::now() }),
        }
    }

    /// 返回账号 ID。
    pub async fn verify(&self, token: &str) -> Result<Uuid, AppError> {
        let header = decode_header(token).map_err(|_| AppError::Unauthorized("账号凭证格式无效".into()))?;
        if header.alg != Algorithm::EdDSA {
            return Err(AppError::Unauthorized("不支持的签名算法".into()));
        }
        let key = self.key_for(header.kid.as_deref()).await?;
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.client_id]);
        validation.leeway = 10;
        let data = decode::<Claims>(token, &key, &validation).map_err(|err| match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::Unauthorized("账号登录已过期，请重新登录".into()),
            jsonwebtoken::errors::ErrorKind::InvalidAudience => AppError::Unauthorized("该账号凭证不是为小伴市场后台签发的，请先在账号中心登记应用".into()),
            _ => AppError::Unauthorized("账号凭证验证失败".into()),
        })?;
        Uuid::parse_str(&data.claims.sub).map_err(|_| AppError::Unauthorized("账号凭证无效".into()))
    }

    async fn key_for(&self, kid: Option<&str>) -> Result<DecodingKey, AppError> {
        {
            let cache = self.cache.read().await;
            if Instant::now() < cache.expires_at {
                if let Some(key) = kid.and_then(|k| cache.keys.get(k)) {
                    return Ok(key.clone());
                }
            }
        }
        // 缓存过期，或出现新的 kid（账号中心轮换密钥）时重新拉取。
        self.refresh().await?;
        let cache = self.cache.read().await;
        match kid {
            Some(k) => cache.keys.get(k).cloned().ok_or_else(|| AppError::Unauthorized("未知的签名密钥".into())),
            None => cache.keys.values().next().cloned().ok_or_else(|| AppError::Unauthorized("没有可用的签名公钥".into())),
        }
    }

    async fn refresh(&self) -> Result<(), AppError> {
        let unavailable = || AppError::Conflict("账号中心暂时不可用，请稍后重试或使用本地管理员登录".into());
        let resp = self.http.get(&self.jwks_url).send().await.map_err(|e| {
            tracing::warn!("拉取 JWKS 失败 {}: {}", self.jwks_url, e.without_url());
            unavailable()
        })?;
        if !resp.status().is_success() {
            return Err(unavailable());
        }
        let jwks: Jwks = resp.json().await.map_err(|e| AppError::Internal(format!("解析 JWKS 失败：{e}")))?;
        let keys: HashMap<_, _> = jwks
            .keys
            .into_iter()
            .filter(|k| k.kty == "OKP")
            .filter_map(|k| Some((k.kid?, DecodingKey::from_ed_components(&k.x?).ok()?)))
            .collect();
        if keys.is_empty() {
            return Err(AppError::Internal("JWKS 中没有 Ed25519 公钥".into()));
        }
        *self.cache.write().await = Cache { keys, expires_at: Instant::now() + Duration::from_secs(3600) };
        Ok(())
    }
}

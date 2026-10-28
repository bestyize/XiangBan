//! 账号中心访问令牌的离线验签（JWKS 公钥，EdDSA）。令牌 `aud` 必须是配置里的客户端编号之一。
use crate::{error::AppError, state::AppState};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap},
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
    client_ids: Vec<String>,
    cache: RwLock<JwksCache>,
    /// 测试用：固定公钥，不拉取 JWKS。
    static_key: Option<DecodingKey>,
}

impl JwksVerifier {
    pub fn new(issuer: String, jwks_url: String, client_ids: Vec<String>) -> Self {
        Self {
            http: reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap_or_default(),
            issuer,
            jwks_url,
            client_ids,
            cache: RwLock::new(JwksCache { keys: HashMap::new(), expires_at: Instant::now() }),
            static_key: None,
        }
    }

    /// 测试用：固定公钥，不拉取 JWKS。
    #[cfg(any(test, feature = "testing"))]
    pub fn with_static_key(issuer: String, client_ids: Vec<String>, key: DecodingKey) -> Self {
        let mut v = Self::new(issuer, String::new(), client_ids);
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
        validation.set_audience(&self.client_ids);
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

pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// WebSocket 子协议名和令牌前缀：`new WebSocket(url, ["xb-link", "xiaoban-token.<令牌>"])`。
pub const WS_PROTOCOL: &str = "xb-link";
const TOKEN_PREFIX: &str = "xiaoban-token.";

/// 令牌可以来自 Authorization 头，也可以来自 WebSocket 子协议（部分客户端设不了请求头）。
pub fn ws_token(headers: &HeaderMap) -> Option<String> {
    if let Some(t) = bearer(headers) {
        return Some(t.to_string());
    }
    let protocols = headers.get(header::SEC_WEBSOCKET_PROTOCOL).and_then(|v| v.to_str().ok())?;
    protocols.split(',').map(str::trim).find_map(|p| p.strip_prefix(TOKEN_PREFIX)).filter(|s| !s.is_empty()).map(str::to_string)
}

/// 已登录用户（账号编号）。
pub struct AuthUser(pub Uuid);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer(&parts.headers).ok_or_else(|| AppError::Unauthorized("请先登录".into()))?;
        Ok(AuthUser(state.verifier.verify(token).await?))
    }
}

//! 凭据使用部署侧独立主密钥加密；HTTP 错误和响应正文不得进入日志。
use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::STANDARD, Engine};
use ring::{
    aead,
    rand::{SecureRandom, SystemRandom},
};

fn master_key() -> AppResult<aead::LessSafeKey> {
    let raw = std::env::var("FORUM_COLLECTION_MASTER_KEY")
        .ok()
        .and_then(|s| STANDARD.decode(s.trim()).ok())
        .filter(|b| b.len() == 32)
        .ok_or_else(|| AppError::ServiceUnavailable("采集加密主密钥未配置或无效".into()))?;
    key(&raw)
}
fn key(raw: &[u8]) -> AppResult<aead::LessSafeKey> {
    aead::UnboundKey::new(&aead::AES_256_GCM, raw)
        .map(aead::LessSafeKey::new)
        .map_err(|_| AppError::ServiceUnavailable("采集加密主密钥无效".into()))
}
fn encrypt_with(k: &aead::LessSafeKey, source: &str, secret: &str) -> AppResult<Vec<u8>> {
    let mut nonce = [0u8; 12];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| AppError::Internal("生成凭据随机数失败".into()))?;
    let mut data = secret.as_bytes().to_vec();
    k.seal_in_place_append_tag(
        aead::Nonce::assume_unique_for_key(nonce),
        aead::Aad::from(source.as_bytes()),
        &mut data,
    )
    .map_err(|_| AppError::Internal("加密凭据失败".into()))?;
    let mut result = vec![1];
    result.extend_from_slice(&nonce);
    result.extend(data);
    Ok(result)
}
fn decrypt_with(k: &aead::LessSafeKey, source: &str, sealed: &[u8]) -> AppResult<String> {
    let failed =
        || AppError::ServiceUnavailable("无法解密采集凭据，请检查主密钥或重新保存来源凭据".into());
    if sealed.len() < 30 || sealed[0] != 1 {
        return Err(failed());
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&sealed[1..13]);
    let mut body = sealed[13..].to_vec();
    let raw = k
        .open_in_place(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(source.as_bytes()),
            &mut body,
        )
        .map_err(|_| failed())?;
    String::from_utf8(raw.to_vec()).map_err(|_| failed())
}
pub fn encrypt(source: &str, secret: &str) -> AppResult<Vec<u8>> {
    encrypt_with(&master_key()?, source, secret)
}
pub fn decrypt(source: &str, sealed: &[u8]) -> AppResult<String> {
    decrypt_with(&master_key()?, source, sealed)
}
pub fn hint(secret: &str) -> String {
    format!("••••{}", &secret[secret.len().saturating_sub(4)..])
}

/// 独立请求客户端，禁重定向，避免将鉴权头转发到其他站点。
pub fn http_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("XiaobanCommunityCollector/1.0")
        .build()
        .map_err(|_| AppError::Internal("初始化采集网络客户端失败".into()))
}

pub async fn validate(source: &str, secret: &str, http: &reqwest::Client) -> AppResult<()> {
    let result = if matches!(source, "github_repos" | "github_tools") {
        super::repo_sources::verify_credential(secret, http).await
    } else {
        super::model_sources::verify_credential(source, secret, http).await
    };
    result.map_err(|_| AppError::BadRequest("来源凭据验证失败，请检查密钥、权限或稍后重试".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credentials_are_authenticated_bound_to_source_and_randomized() {
        let key = key(&[7u8; 32]).unwrap();
        let a = encrypt_with(&key, "openrouter", "test-secret-key").unwrap();
        let b = encrypt_with(&key, "openrouter", "test-secret-key").unwrap();
        assert_ne!(a, b);
        assert_eq!(
            decrypt_with(&key, "openrouter", &a).unwrap(),
            "test-secret-key"
        );
        assert!(decrypt_with(&key, "artificial_analysis", &a).is_err());
        assert!(decrypt_with(&super::key(&[8u8; 32]).unwrap(), "openrouter", &a).is_err());
        let mut corrupt = a;
        corrupt[14] ^= 1;
        assert!(decrypt_with(&key, "openrouter", &corrupt).is_err());
        assert!(decrypt_with(&key, "openrouter", &[]).is_err());
        assert_eq!(hint("test-secret-key"), "••••-key");
    }
}

use crate::{
    config::AppConfig,
    crypto::{self, SubKeys},
    error::AppError,
};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair},
};
use std::{fs, io::Write, path::Path};

pub struct Keys {
    pub sub: SubKeys,
    pub kid: String,
    pub public_x: String,
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn load(config: &AppConfig) -> Result<Self, AppError> {
        let pkcs8 = match &config.jwt_pkcs8_base64 {
            Some(b64) => STANDARD
                .decode(b64.trim())
                .map_err(|_| AppError::Internal("ACCOUNT_JWT_PKCS8_BASE64 is not valid base64".into()))?,
            None => read_or_create(&config.data_dir.join("jwt_ed25519.pk8"), generate_pkcs8)?,
        };
        let master = match &config.master_key_base64 {
            Some(b64) => STANDARD
                .decode(b64.trim())
                .map_err(|_| AppError::Internal("ACCOUNT_MASTER_KEY_BASE64 is not valid base64".into()))?,
            None => read_or_create(&config.data_dir.join("master.key"), || crypto::random_bytes::<32>().to_vec())?,
        };
        Self::from_parts(&pkcs8, &master)
    }

    pub fn from_parts(pkcs8: &[u8], master: &[u8]) -> Result<Self, AppError> {
        if master.len() < 32 {
            return Err(AppError::Internal("master key must be at least 32 bytes".into()));
        }
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8)
            .map_err(|_| AppError::Internal("invalid Ed25519 PKCS#8 signing key".into()))?;
        let public = pair.public_key().as_ref().to_vec();
        Ok(Self {
            sub: SubKeys::derive(master),
            kid: crypto::hex(&crypto::sha256(&public)[..8]),
            public_x: URL_SAFE_NO_PAD.encode(&public),
            encoding: EncodingKey::from_ed_der(pkcs8),
            decoding: DecodingKey::from_ed_der(&public),
        })
    }

    #[cfg(test)]
    pub fn ephemeral() -> Self {
        Self::from_parts(&generate_pkcs8(), &crypto::random_bytes::<32>()).expect("ephemeral keys")
    }
}

/// 生成可直接写入 .env 的签名私钥与主密钥（base64）。不要用 openssl 生成签名私钥：它输出 PKCS#8 v1，ring 只接受 v2。
pub fn generate_env_values() -> (String, String) {
    (STANDARD.encode(generate_pkcs8()), STANDARD.encode(crypto::random_bytes::<32>()))
}

fn generate_pkcs8() -> Vec<u8> {
    Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
        .expect("Ed25519 key generation")
        .as_ref()
        .to_vec()
}

fn read_or_create(path: &Path, create: impl FnOnce() -> Vec<u8>) -> Result<Vec<u8>, AppError> {
    let io = |e: std::io::Error| AppError::Internal(format!("{}: {e}", path.display()));
    if path.exists() {
        return fs::read(path).map_err(io);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(io)?;
    }
    let bytes = create();
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).and_then(|mut f| f.write_all(&bytes)).map_err(io)?;
    tracing::warn!("generated new key file {}", path.display());
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(jwt: Option<&str>, master: Option<&str>, dir: &Path) -> AppConfig {
        let dir = dir.display().to_string();
        AppConfig::from_lookup(|key| match key {
            "ACCOUNT_JWT_PKCS8_BASE64" => jwt.map(str::to_string),
            "ACCOUNT_MASTER_KEY_BASE64" => master.map(str::to_string),
            "ACCOUNT_DATA_DIR" => Some(dir.clone()),
            _ => None,
        })
    }

    #[test]
    fn generated_values_load_from_env_without_touching_disk() {
        let (jwt, master) = generate_env_values();
        let dir = std::env::temp_dir().join(format!("acct_keys_{}", crypto::hex(&crypto::random_bytes::<8>())));
        let config = config(Some(&jwt), Some(&master), &dir);

        let first = Keys::load(&config).unwrap();
        let second = Keys::load(&config).unwrap();
        assert!(!dir.exists(), "env-provided keys must not create the data dir");
        assert_eq!(first.kid, second.kid);
        assert_eq!(first.public_x, second.public_x);
        assert_eq!(first.sub.identifier_hash("email", "a@b.com"), second.sub.identifier_hash("email", "a@b.com"));
    }

    #[test]
    fn rejects_short_master_key_and_bad_base64() {
        let (jwt, _) = generate_env_values();
        let dir = std::env::temp_dir().join("acct_keys_unused");
        let short = STANDARD.encode([1u8; 16]);
        assert!(Keys::load(&config(Some(&jwt), Some(&short), &dir)).is_err());
        assert!(Keys::load(&config(Some("not base64!!"), Some(&short), &dir)).is_err());
        assert!(Keys::load(&config(Some(&STANDARD.encode([0u8; 40])), Some(&STANDARD.encode([1u8; 32])), &dir)).is_err());
    }
}

use crate::{config::AppConfig, error::AppError};
use base64::{engine::general_purpose::STANDARD, Engine};
use ring::{
    aead, hmac,
    rand::{SecureRandom, SystemRandom},
};
use std::{fs, io::Write, path::Path};

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    SystemRandom::new().fill(&mut buf).expect("system RNG unavailable");
    buf
}

/// 主密钥派生的子密钥：管理员令牌签名、各绑定 New API Key 的加密。
pub struct Keys {
    pub admin_jwt_secret: Vec<u8>,
    aead: aead::LessSafeKey,
    code_mac: hmac::Key,
}

impl Keys {
    pub fn load(config: &AppConfig) -> Result<Self, AppError> {
        let master = match &config.master_key_base64 {
            Some(b64) => STANDARD
                .decode(b64.trim())
                .map_err(|_| AppError::Internal("AI_MASTER_KEY_BASE64 is not valid base64".into()))?,
            None => read_or_create(&config.data_dir.join("master.key"))?,
        };
        Self::from_master(&master)
    }

    pub fn from_master(master: &[u8]) -> Result<Self, AppError> {
        if master.len() < 32 {
            return Err(AppError::Internal("master key must be at least 32 bytes".into()));
        }
        let root = hmac::Key::new(hmac::HMAC_SHA256, master);
        let sub = |label: &[u8]| hmac::sign(&root, label).as_ref().to_vec();
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &sub(b"newapi-key-aead")).expect("32-byte AES-256 key");
        Ok(Self {
            admin_jwt_secret: sub(b"admin-jwt"),
            aead: aead::LessSafeKey::new(key),
            code_mac: hmac::Key::new(hmac::HMAC_SHA256, &sub(b"redeem-code-hash")),
        })
    }

    /// 兑换码的查找哈希。带密钥，所以拿到数据库也没法离线穷举；输入必须是规范化后的码。
    pub fn code_hash(&self, canonical: &str) -> Vec<u8> {
        hmac::sign(&self.code_mac, canonical.as_bytes()).as_ref().to_vec()
    }

    #[cfg(test)]
    pub fn ephemeral() -> Self {
        Self::from_master(&random_bytes::<32>()).expect("ephemeral keys")
    }

    /// 输出 nonce(12) || 密文 || tag(16)，`aad` 绑定用途，防止密文被挪作他用。
    pub fn encrypt(&self, aad: &str, plain: &str) -> Result<Vec<u8>, AppError> {
        let nonce_bytes = random_bytes::<12>();
        let mut buf = plain.as_bytes().to_vec();
        self.aead
            .seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce_bytes), aead::Aad::from(aad.as_bytes()), &mut buf)
            .map_err(|_| AppError::Internal("encryption failed".into()))?;
        let mut out = nonce_bytes.to_vec();
        out.extend(buf);
        Ok(out)
    }

    pub fn decrypt(&self, aad: &str, data: &[u8]) -> Result<String, AppError> {
        if data.len() < 12 + 16 {
            return Err(AppError::Internal("ciphertext too short".into()));
        }
        let (nonce, rest) = data.split_at(12);
        let mut buf = rest.to_vec();
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce).map_err(|_| AppError::Internal("bad nonce".into()))?;
        let plain = self
            .aead
            .open_in_place(nonce, aead::Aad::from(aad.as_bytes()), &mut buf)
            .map_err(|_| AppError::Internal("decryption failed".into()))?;
        String::from_utf8(plain.to_vec()).map_err(|_| AppError::Internal("plaintext not utf-8".into()))
    }
}

/// 生成可直接写入 .env 的主密钥（base64）。
pub fn generate_master_key() -> String {
    STANDARD.encode(random_bytes::<32>())
}

fn read_or_create(path: &Path) -> Result<Vec<u8>, AppError> {
    let io = |e: std::io::Error| AppError::Internal(format!("{}: {e}", path.display()));
    if path.exists() {
        return fs::read(path).map_err(io);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(io)?;
    }
    let bytes = random_bytes::<32>().to_vec();
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).and_then(|mut f| f.write_all(&bytes)).map_err(io)?;
    tracing::warn!("generated new master key file {}", path.display());
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_roundtrip_and_aad_binding() {
        let keys = Keys::ephemeral();
        let enc = keys.encrypt("newapi_key", "sk-abc").unwrap();
        assert_eq!(keys.decrypt("newapi_key", &enc).unwrap(), "sk-abc");
        assert!(keys.decrypt("other", &enc).is_err());
        assert_ne!(enc, keys.encrypt("newapi_key", "sk-abc").unwrap());
    }

    #[test]
    fn code_hashes_are_stable_per_key_and_differ_between_keys() {
        let (a, b) = (Keys::from_master(&[1u8; 32]).unwrap(), Keys::from_master(&[2u8; 32]).unwrap());
        assert_eq!(a.code_hash("ABCDEFGHJKLMNPQR"), a.code_hash("ABCDEFGHJKLMNPQR"));
        assert_ne!(a.code_hash("ABCDEFGHJKLMNPQR"), a.code_hash("ABCDEFGHJKLMNPQS"));
        assert_ne!(a.code_hash("ABCDEFGHJKLMNPQR"), b.code_hash("ABCDEFGHJKLMNPQR"));
    }

    #[test]
    fn rejects_short_master_key() {
        assert!(Keys::from_master(&[1u8; 16]).is_err());
    }
}

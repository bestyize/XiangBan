use crate::error::AppError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::{
    aead, digest, hmac,
    rand::{SecureRandom, SystemRandom},
};

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    SystemRandom::new().fill(&mut buf).expect("system RNG unavailable");
    buf
}

pub fn random_token() -> String {
    URL_SAFE_NO_PAD.encode(random_bytes::<32>())
}

/// 6 位数字验证码，拒绝采样保证均匀分布。
pub fn random_code() -> String {
    loop {
        let n = u32::from_be_bytes(random_bytes::<4>());
        if n < 4_294_000_000 {
            return format!("{:06}", n % 1_000_000);
        }
    }
}

pub fn sha256(data: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, data).as_ref().to_vec()
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 由主密钥派生的子密钥：标识符查找哈希、验证码哈希、标识符加密。
pub struct SubKeys {
    id_mac: hmac::Key,
    code_mac: hmac::Key,
    aead: aead::LessSafeKey,
    /// 管理员令牌的 HS256 密钥：与用户令牌的 Ed25519 密钥完全独立，业务服务拿不到。
    pub admin_jwt_secret: Vec<u8>,
    /// 注册凭证（验证码通过后、设置用户名密码前）的 HS256 密钥，独立于管理员与用户令牌。
    pub register_jwt_secret: Vec<u8>,
}

impl SubKeys {
    pub fn derive(master: &[u8]) -> Self {
        let root = hmac::Key::new(hmac::HMAC_SHA256, master);
        let sub = |label: &[u8]| hmac::sign(&root, label).as_ref().to_vec();
        let aead_key = aead::UnboundKey::new(&aead::AES_256_GCM, &sub(b"identifier-aead"))
            .expect("32-byte AES-256 key");
        Self {
            id_mac: hmac::Key::new(hmac::HMAC_SHA256, &sub(b"identifier-hash")),
            code_mac: hmac::Key::new(hmac::HMAC_SHA256, &sub(b"code-hash")),
            aead: aead::LessSafeKey::new(aead_key),
            admin_jwt_secret: sub(b"admin-jwt"),
            register_jwt_secret: sub(b"register-jwt"),
        }
    }

    pub fn identifier_hash(&self, kind: &str, value: &str) -> Vec<u8> {
        hmac::sign(&self.id_mac, format!("{kind}\n{value}").as_bytes()).as_ref().to_vec()
    }

    fn code_message(kind: &str, value: &str, purpose: &str, code: &str) -> String {
        format!("{kind}\n{value}\n{purpose}\n{code}")
    }

    pub fn code_hash(&self, kind: &str, value: &str, purpose: &str, code: &str) -> Vec<u8> {
        hmac::sign(&self.code_mac, Self::code_message(kind, value, purpose, code).as_bytes())
            .as_ref()
            .to_vec()
    }

    /// 常量时间比较已存储的验证码哈希。
    pub fn code_matches(&self, kind: &str, value: &str, purpose: &str, code: &str, stored: &[u8]) -> bool {
        hmac::verify(&self.code_mac, Self::code_message(kind, value, purpose, code).as_bytes(), stored).is_ok()
    }

    /// 输出 nonce(12) || 密文 || tag(16)，kind 作为附加认证数据。
    pub fn encrypt(&self, kind: &str, plain: &str) -> Result<Vec<u8>, AppError> {
        let nonce_bytes = random_bytes::<12>();
        let mut buf = plain.as_bytes().to_vec();
        self.aead
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(nonce_bytes),
                aead::Aad::from(kind.as_bytes()),
                &mut buf,
            )
            .map_err(|_| AppError::Internal("identifier encryption failed".into()))?;
        let mut out = nonce_bytes.to_vec();
        out.extend(buf);
        Ok(out)
    }

    pub fn decrypt(&self, kind: &str, data: &[u8]) -> Result<String, AppError> {
        if data.len() < 12 + 16 {
            return Err(AppError::Internal("identifier ciphertext too short".into()));
        }
        let (nonce, rest) = data.split_at(12);
        let mut buf = rest.to_vec();
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
            .map_err(|_| AppError::Internal("bad nonce".into()))?;
        let plain = self
            .aead
            .open_in_place(nonce, aead::Aad::from(kind.as_bytes()), &mut buf)
            .map_err(|_| AppError::Internal("identifier decryption failed".into()))?;
        String::from_utf8(plain.to_vec()).map_err(|_| AppError::Internal("identifier not utf-8".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_roundtrip_and_aad_binding() {
        let keys = SubKeys::derive(&[7u8; 32]);
        let enc = keys.encrypt("email", "a@b.com").unwrap();
        assert_eq!(keys.decrypt("email", &enc).unwrap(), "a@b.com");
        assert!(keys.decrypt("phone", &enc).is_err());
        assert_ne!(enc, keys.encrypt("email", "a@b.com").unwrap());
    }

    #[test]
    fn hashes_are_stable_and_domain_separated() {
        let keys = SubKeys::derive(&[7u8; 32]);
        assert_eq!(keys.identifier_hash("email", "a@b.com"), keys.identifier_hash("email", "a@b.com"));
        assert_ne!(keys.identifier_hash("email", "x"), keys.identifier_hash("phone", "x"));
        assert_ne!(
            keys.code_hash("email", "a@b.com", "login", "123456"),
            keys.code_hash("email", "a@b.com", "bind", "123456")
        );
    }

    #[test]
    fn code_is_six_digits() {
        for _ in 0..100 {
            let c = random_code();
            assert_eq!(c.len(), 6);
            assert!(c.chars().all(|ch| ch.is_ascii_digit()));
        }
    }
}

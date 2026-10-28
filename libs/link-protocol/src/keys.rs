//! 设备密钥：P-256。私钥 32 字节，公钥用 SEC1 未压缩格式（65 字节，首字节 0x04）。
use crate::Error;
use p256::{ecdh::diffie_hellman, PublicKey, SecretKey};
use rand_core::OsRng;

pub struct DeviceKey {
    secret: SecretKey,
}

impl DeviceKey {
    pub fn generate() -> Self {
        Self { secret: SecretKey::random(&mut OsRng) }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, Error> {
        Ok(Self { secret: SecretKey::from_slice(bytes).map_err(|_| Error::BadKey)? })
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes().into()
    }

    pub fn public_bytes(&self) -> [u8; 65] {
        let point = self.secret.public_key().to_sec1_bytes();
        let mut out = [0u8; 65];
        out.copy_from_slice(&point);
        out
    }

    /// ECDH 共享秘密（x 坐标，32 字节）。
    pub fn agree(&self, peer_public: &[u8]) -> Result<[u8; 32], Error> {
        let peer = PublicKey::from_sec1_bytes(peer_public).map_err(|_| Error::BadKey)?;
        let shared = diffie_hellman(self.secret.to_nonzero_scalar(), peer.as_affine());
        let mut out = [0u8; 32];
        out.copy_from_slice(shared.raw_secret_bytes());
        Ok(out)
    }
}

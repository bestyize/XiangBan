//! 配对：二维码载荷、手机的证明、两端的配对密钥。
//!
//! 流程见规范。要点：二维码走带外（摄像头），里面有电脑的公钥和一次性密钥 `s`。
//! 手机用 `s` 给自己的公钥做证明，电脑验证通过才认这个手机；中转看不到 `s`，所以冒充不了手机。
//! 两端再把 ECDH 的结果和 `s` 一起派生出配对密钥 PRK，之后所有消息密钥都从 PRK 来。
use crate::{keys::DeviceKey, Error};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

pub const QR_PREFIX: &str = "xbl1:";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QrPayload {
    /// 中转服务地址
    pub r: String,
    /// 电脑设备编号
    pub d: String,
    /// 一次性配对码（中转用）
    pub c: String,
    /// 电脑公钥，base64url
    pub k: String,
    /// 一次性密钥 32 字节，base64url
    pub s: String,
}

impl QrPayload {
    pub fn encode(&self) -> String {
        format!("{QR_PREFIX}{}", B64.encode(serde_json::to_vec(self).expect("serialize")))
    }

    pub fn decode(text: &str) -> Result<Self, Error> {
        let body = text.strip_prefix(QR_PREFIX).ok_or(Error::BadEncoding)?;
        let bytes = B64.decode(body).map_err(|_| Error::BadEncoding)?;
        let payload: Self = serde_json::from_slice(&bytes).map_err(|_| Error::BadEncoding)?;
        if B64.decode(&payload.k).map_err(|_| Error::BadEncoding)?.len() != 65 || B64.decode(&payload.s).map_err(|_| Error::BadEncoding)?.len() != 32 {
            return Err(Error::BadEncoding);
        }
        Ok(payload)
    }

    pub fn desktop_public(&self) -> Result<Vec<u8>, Error> { B64.decode(&self.k).map_err(|_| Error::BadEncoding) }
    pub fn secret(&self) -> Result<[u8; 32], Error> {
        B64.decode(&self.s).map_err(|_| Error::BadEncoding)?.try_into().map_err(|_| Error::BadEncoding)
    }
}

type HmacSha256 = Hmac<Sha256>;

fn proof_input(phone_id: &str, desktop_id: &str, phone_public: &[u8]) -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(b"xb-link/1 pair-proof\0");
    m.extend_from_slice(phone_id.as_bytes());
    m.push(0);
    m.extend_from_slice(desktop_id.as_bytes());
    m.push(0);
    m.extend_from_slice(phone_public);
    m
}

/// 手机用二维码里的一次性密钥证明“这个公钥是扫了码的人给的”。
pub fn make_proof(secret: &[u8; 32], phone_id: &str, desktop_id: &str, phone_public: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac key");
    mac.update(&proof_input(phone_id, desktop_id, phone_public));
    mac.finalize().into_bytes().into()
}

/// 电脑校验手机的证明（常量时间比较）。
pub fn verify_proof(secret: &[u8; 32], phone_id: &str, desktop_id: &str, phone_public: &[u8], proof: &[u8]) -> Result<(), Error> {
    let expected = make_proof(secret, phone_id, desktop_id, phone_public);
    if expected.ct_eq(proof).into() { Ok(()) } else { Err(Error::BadProof) }
}

/// 配对密钥：两端各自算出同一个值，长期保存（手机在 Keystore，电脑在钥匙串）。
/// PRK = HKDF-Extract(salt = 一次性密钥, ikm = ECDH 结果)。
pub fn derive_pairing_key(own: &DeviceKey, peer_public: &[u8], secret: &[u8; 32]) -> Result<[u8; 32], Error> {
    let shared = own.agree(peer_public)?;
    let (prk, _) = Hkdf::<Sha256>::extract(Some(secret), &shared);
    Ok(prk.into())
}

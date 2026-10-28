//! 与 server-forum `src/content.rs` 约定一致的签名：下载地址、上传凭证、回调。
//! 两边各有一份，靠同一组测试向量防止漂移（见 tests）。

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::hmac;
use serde::{Deserialize, Serialize};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 {
        return None;
    }
    (0..value.len() / 2).map(|i| u8::from_str_radix(value.get(i * 2..i * 2 + 2)?, 16).ok()).collect()
}

fn sign(secret: &[u8], message: &[u8]) -> String {
    hex(hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, secret), message).as_ref())
}

fn verify(secret: &[u8], message: &[u8], signature: &str) -> bool {
    from_hex(signature).is_some_and(|raw| hmac::verify(&hmac::Key::new(hmac::HMAC_SHA256, secret), message, &raw).is_ok())
}

/// 下载请求：签名覆盖 `GET\n{file_key}\n{exp}`。
pub fn verify_download(file_key: &str, exp: i64, sig: &str, now: i64, secret: &[u8]) -> bool {
    exp >= now && verify(secret, format!("GET\n{file_key}\n{exp}").as_bytes(), sig)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UploadClaims {
    pub uid: String,
    pub kind: String,
    pub size: u64,
    pub sha256: String,
    pub file_key: String,
    pub exp: i64,
}

/// 上传凭证：`base64url(JSON声明).hex(HMAC(secret, "PUT\n" + base64url部分))`。
pub fn verify_upload_ticket(ticket: &str, now: i64, secret: &[u8]) -> Result<UploadClaims, &'static str> {
    let (body, sig) = ticket.split_once('.').ok_or("上传凭证格式无效")?;
    if !verify(secret, format!("PUT\n{body}").as_bytes(), sig) {
        return Err("上传凭证无效");
    }
    let raw = URL_SAFE_NO_PAD.decode(body).map_err(|_| "上传凭证无效")?;
    let claims: UploadClaims = serde_json::from_slice(&raw).map_err(|_| "上传凭证无效")?;
    if claims.exp < now {
        return Err("上传凭证已过期，请重新开始上传");
    }
    Ok(claims)
}

/// 回调论坛：对请求体做 HMAC，放在 `X-Signature` 头里。
pub fn sign_callback(body: &[u8], secret: &[u8]) -> String {
    sign(secret, body)
}

#[cfg(test)]
pub fn issue_upload_ticket(claims: &UploadClaims, secret: &[u8]) -> String {
    let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).unwrap());
    format!("{body}.{}", sign(secret, format!("PUT\n{body}").as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"0123456789abcdef0123456789abcdef";

    /// 固定向量：论坛侧 `content::tests::signature_vectors` 断言同样的值。
    #[test]
    fn signature_vectors() {
        assert_eq!(
            sign(SECRET, b"GET\npacks/effect/a.zip\n1900000000"),
            "f8b2ad6a870024d63002c60ec4468de0988451eb421ef3dc8378fd5e5fcd714f"
        );
    }

    #[test]
    fn ticket_and_download_round_trip() {
        let claims = UploadClaims {
            uid: "u".into(),
            kind: "effect".into(),
            size: 10,
            sha256: "a".repeat(64),
            file_key: "packs/effect/x.zip".into(),
            exp: 2_000_000_000,
        };
        let ticket = issue_upload_ticket(&claims, SECRET);
        assert_eq!(verify_upload_ticket(&ticket, 1, SECRET).unwrap(), claims);
        assert!(verify_upload_ticket(&ticket, 2_000_000_001, SECRET).is_err());
        assert!(verify_upload_ticket(&ticket, 1, b"another-secret-another-secret-0000").is_err());
        let sig = sign(SECRET, b"GET\nk\n10");
        assert!(verify_download("k", 10, &sig, 5, SECRET));
        assert!(!verify_download("k", 10, &sig, 11, SECRET));
        assert!(!verify_download("k2", 10, &sig, 5, SECRET));
    }
}

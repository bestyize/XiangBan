//! 加密信封。中转能看到：协议版本、双方设备编号、会话标识 `ep`、序号 `seq`、密文长度。看不到正文。
//!
//! 密钥：`K = HKDF-Expand(HKDF-Extract(salt = ep, ikm = PRK), "xb-link/1 key " + from + ">" + to, 32)`。
//! `ep` 是 16 字节：前 8 字节是发送方**持久化的启动计数**（每次启动加一，大端），后 8 字节随机。
//! 每次启动换新的 `ep`，所以同一把密钥下序号不会重复；接收方按启动计数判断新旧，旧会话的信封不会被重放进来。
//! 随机数：12 字节 = 4 个 0 + 序号（8 字节大端）。认证附加数据绑定版本、双方、`ep`、`seq`。
use crate::{Error, MAX_CIPHERTEXT, PAD_BLOCK, VERSION};
use aes_gcm::{aead::{Aead, KeyInit, Payload}, Aes256Gcm, Nonce};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub v: u8,
    pub from: String,
    pub to: String,
    /// 发送方本次启动的会话标识，16 字节，base64url
    pub ep: String,
    pub seq: u64,
    /// 密文加认证标签，base64url
    pub ct: String,
}

fn message_key(prk: &[u8; 32], ep: &[u8], from: &str, to: &str) -> [u8; 32] {
    let (prk_e, _) = Hkdf::<Sha256>::extract(Some(ep), prk);
    let hk = Hkdf::<Sha256>::from_prk(&prk_e).expect("prk length");
    let mut info = Vec::new();
    info.extend_from_slice(b"xb-link/1 key ");
    info.extend_from_slice(from.as_bytes());
    info.push(b'>');
    info.extend_from_slice(to.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(&info, &mut key).expect("32 bytes");
    key
}

fn nonce(seq: u64) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[4..].copy_from_slice(&seq.to_be_bytes());
    n
}

fn aad(from: &str, to: &str, ep: &[u8], seq: u64) -> Vec<u8> {
    let mut a = Vec::new();
    a.extend_from_slice(b"xb-link/1\0");
    a.extend_from_slice(from.as_bytes());
    a.push(0);
    a.extend_from_slice(to.as_bytes());
    a.push(0);
    a.extend_from_slice(ep);
    a.extend_from_slice(&seq.to_be_bytes());
    a
}

/// ISO/IEC 7816-4 补齐：末尾加 0x80，再补 0 到 [`PAD_BLOCK`] 的整数倍。
pub fn pad(plain: &[u8]) -> Vec<u8> {
    let mut out = plain.to_vec();
    out.push(0x80);
    while out.len() % PAD_BLOCK != 0 {
        out.push(0);
    }
    out
}

pub fn unpad(mut padded: Vec<u8>) -> Result<Vec<u8>, Error> {
    while padded.last() == Some(&0) {
        padded.pop();
    }
    if padded.pop() != Some(0x80) {
        return Err(Error::BadPadding);
    }
    Ok(padded)
}

/// 由持久化的启动计数加 8 字节随机数组成 `ep`。计数必须在每次启动时加一并落盘。
pub fn make_ep(start_counter: u64) -> [u8; 16] {
    let mut ep = [0u8; 16];
    ep[..8].copy_from_slice(&start_counter.to_be_bytes());
    rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut ep[8..]);
    ep
}

/// 取出 `ep` 里的启动计数。
pub fn ep_counter(ep: &[u8]) -> Option<u64> {
    ep.get(..8).and_then(|b| b.try_into().ok()).map(u64::from_be_bytes)
}

/// 发送方：每次启动用 [`Sender::new`]（传入已加一并落盘的启动计数），序号从 0 递增。
pub struct Sender {
    prk: [u8; 32],
    from: String,
    to: String,
    ep: [u8; 16],
    next_seq: u64,
}

impl Sender {
    pub fn new(prk: [u8; 32], from: &str, to: &str, start_counter: u64) -> Self {
        Self::with_ep(prk, from, to, make_ep(start_counter))
    }

    /// 测试向量用：指定 `ep`。正式使用请用 [`Sender::new`]。
    pub fn with_ep(prk: [u8; 32], from: &str, to: &str, ep: [u8; 16]) -> Self {
        Self { prk, from: from.into(), to: to.into(), ep, next_seq: 0 }
    }

    pub fn seal(&mut self, plain: &[u8]) -> Result<Envelope, Error> {
        let seq = self.next_seq;
        let env = seal_with(&self.prk, &self.from, &self.to, &self.ep, seq, plain)?;
        self.next_seq += 1;
        Ok(env)
    }
}

pub fn seal_with(prk: &[u8; 32], from: &str, to: &str, ep: &[u8; 16], seq: u64, plain: &[u8]) -> Result<Envelope, Error> {
    let padded = pad(plain);
    if padded.len() + 16 > MAX_CIPHERTEXT {
        return Err(Error::TooLarge);
    }
    let key = message_key(prk, ep, from, to);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("key length");
    let ct = cipher
        .encrypt(&Nonce::from(nonce(seq)), Payload { msg: &padded, aad: &aad(from, to, ep, seq) })
        .map_err(|_| Error::Decrypt)?;
    Ok(Envelope { v: VERSION, from: from.into(), to: to.into(), ep: B64.encode(ep), seq, ct: B64.encode(ct) })
}

/// 解开信封。`expected_from`/`expected_to` 是接收方认定的配对双方，信封里写的必须一致。
pub fn open(prk: &[u8; 32], expected_from: &str, expected_to: &str, env: &Envelope) -> Result<Vec<u8>, Error> {
    if env.v != VERSION {
        return Err(Error::UnsupportedVersion(env.v));
    }
    if env.from != expected_from || env.to != expected_to {
        return Err(Error::Decrypt);
    }
    let ep = B64.decode(&env.ep).map_err(|_| Error::BadEncoding)?;
    if ep.len() != 16 {
        return Err(Error::BadEncoding);
    }
    let ct = B64.decode(&env.ct).map_err(|_| Error::BadEncoding)?;
    if ct.len() > MAX_CIPHERTEXT {
        return Err(Error::TooLarge);
    }
    let key = message_key(prk, &ep, &env.from, &env.to);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("key length");
    let padded = cipher
        .decrypt(&Nonce::from(nonce(env.seq)), Payload { msg: &ct, aad: &aad(&env.from, &env.to, &ep, env.seq) })
        .map_err(|_| Error::Decrypt)?;
    unpad(padded)
}

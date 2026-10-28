//! 小伴手机与桌面互联协议 v1 的参考实现。规范见 `docs/agent/设备互联协议.md`。
//!
//! 只做三件事：配对时的密钥协商与互相证明、应用消息的加密信封、接收端的重放防护。
//! 网络、存储、界面都不在这里，两端（Rust 桌面、Kotlin 手机）都必须通过 `vectors/v1.json`。

pub mod envelope;
pub mod keys;
pub mod pairing;
pub mod replay;
pub mod vectors;

pub const VERSION: u8 = 1;
/// 明文按这个粒度补齐，隐藏消息的真实长度。
pub const PAD_BLOCK: usize = 256;
/// 单个信封里密文的最大字节数（含认证标签）。
pub const MAX_CIPHERTEXT: usize = 256 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// 公钥、密钥或编码格式不对。
    BadKey,
    BadEncoding,
    /// 认证失败：被篡改、密钥不对、来往方向或序号对不上。
    Decrypt,
    /// 补齐格式不对。
    BadPadding,
    TooLarge,
    UnsupportedVersion(u8),
    /// 重放或乱序：序号不比已收到的大，或来自已被取代的会话。
    Replay,
    /// 配对证明不匹配：对方不持有二维码里的一次性密钥。
    BadProof,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

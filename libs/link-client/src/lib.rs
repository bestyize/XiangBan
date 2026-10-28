//! 小伴桌面端的互联客户端。协议见 `docs/agent/设备互联协议.md`，加密由 `xiaoban-link-protocol` 完成。
//!
//! 宿主（Tauri 应用）做三件事：提供令牌（[`TokenFn`]）、提供存储（[`store::LinkStore`]）、消费事件（[`LinkEvent`]）。
//! 本库负责：登记设备、生成配对二维码、验证手机的配对证明并确认、保持 WebSocket 连接（断线自动重连）、
//! 加解密收发、重放防护、解除配对。

pub mod api;
mod client;
pub mod store;

pub use client::{Link, LinkConfig, LinkEvent, PairingOffer, PeerSummary};

use std::{future::Future, pin::Pin, sync::Arc};

/// 取当前可用的账号访问令牌；没登录返回 `None`。每次请求和每次重连都会调用，宿主负责刷新。
pub type TokenFn = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Option<String>> + Send>> + Send + Sync>;

/// 固定令牌（测试或脚本用）。
pub fn static_token(token: impl Into<String>) -> TokenFn {
    let token: String = token.into();
    Arc::new(move || {
        let t = token.clone();
        Box::pin(async move { Some(t) })
    })
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("没有可用的登录令牌")]
    NoToken,
    #[error("网络错误：{0}")]
    Network(String),
    #[error("服务返回 {status} {code}：{message}")]
    Server { status: u16, code: String, message: String },
    #[error("存储错误：{0}")]
    Store(String),
    #[error("没有连上中转服务")]
    NotConnected,
    #[error("没有这个已配对的设备")]
    UnknownPeer,
    #[error("发送队列已满")]
    Congested,
    #[error("协议错误：{0}")]
    Protocol(String),
}

impl From<xiaoban_link_protocol::Error> for Error {
    fn from(e: xiaoban_link_protocol::Error) -> Self { Error::Protocol(e.to_string()) }
}

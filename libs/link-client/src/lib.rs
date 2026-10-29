//! 小伴桌面端的互联客户端。协议见 `docs/agent/设备互联协议.md`。
//!
//! 同一个账号登录的手机和电脑自动互相发现，不需要配对：本库用账号令牌向中转登记这台设备，
//! 保持一条 WebSocket 连接，收到“同账号下对端设备”的列表与上下线通知，并收发文本消息。
//!
//! 宿主（Tauri 应用）做三件事：提供令牌（[`TokenFn`]）、提供存储（[`store::LinkStore`]，只存设备编号）、消费事件（[`LinkEvent`]）。

pub mod api;
mod client;
pub mod store;

pub use client::{Link, LinkConfig, LinkEvent, PeerSummary};

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
    #[error("没有这个设备")]
    UnknownPeer,
    #[error("发送队列已满")]
    Congested,
    #[error("消息不是合法文本")]
    BadMessage,
}

//! 小伴社区内容包的客户端逻辑：链接解析（`link`）、下载与校验（`client`）、压缩包检查与载荷（`pack`）。
//! 只做“把一个内容包安全地取到本机并看懂”，不碰界面，也不决定怎么应用——应用必须由宿主在用户确认后进行。

pub mod client;
pub mod link;
pub mod pack;

pub use client::{Community, DownloadInfo, Mirror};
pub use link::{parse_link, Link};
pub use pack::{inspect, is_pure_data, rules_for, size_limit, Inspected};

//! Backend-injected, loopback-only voice HTTP service. No audio files or outbound
//! connections are created by the library. See the crate README for host duties.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audio;
pub mod backends;
mod mcp;
mod openai;
pub mod security;
mod server;

pub use backends::*;
pub use server::{start, RunningServer, ServerConfig};

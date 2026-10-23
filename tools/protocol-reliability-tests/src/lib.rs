//! Compile host-independent product protocol logic without building Tauri or a database.
#![allow(dead_code)]
#[path = "../../../client/xiaoban-desktop-wallpaper/src-tauri/src/ai_reasoning.rs"]
mod ai_reasoning;
#[path = "../../../client/xiaoban-desktop-wallpaper/src-tauri/src/chat/run_history.rs"]
mod run_history;
#[path = "../../../services/server-ai/src/protocol_limits.rs"]
mod protocol_limits;

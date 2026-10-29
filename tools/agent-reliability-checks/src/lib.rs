//! Compile the desktop's pure reliability modules without a platform WebView.
//! These are the production modules themselves, including their original tests.
#![allow(dead_code)]

#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/broker.rs"]
mod broker;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/compact.rs"]
mod compact;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/run_history.rs"]
mod run_history;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/git_command.rs"]
mod git_command;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/git_snapshot.rs"]
mod git_snapshot;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/checkpoints.rs"]
mod checkpoints;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/task_outcome.rs"]
mod task_outcome;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/task_runtime.rs"]
mod task_runtime;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/agent_settings.rs"]
mod agent_settings;

#[path = "../../../client/xiaoban-desktop/src-tauri/src/ai_reasoning.rs"]
mod ai_reasoning;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/model_selection.rs"]
mod model_selection;
#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/sessions.rs"]
mod sessions;

#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/outbox.rs"]
mod outbox;

#[path = "../../../client/xiaoban-desktop/src-tauri/src/chat/mentions.rs"]
mod mentions;

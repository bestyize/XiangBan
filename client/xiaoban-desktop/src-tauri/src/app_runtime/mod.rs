//! V2 App Runtime module for Xiaoban Desktop.

pub mod protocol;
pub mod network;
pub mod storage;
pub mod window;

pub use protocol::serve;
pub use window::AppRuntimeState;

use serde_json::Value;
use tauri::{AppHandle, Manager, WebviewWindow};

fn caller_app_id<'a>(window: &'a WebviewWindow) -> Result<&'a str, String> {
    window::app_id_from_label(window.label()).ok_or_else(|| "非合法应用窗口调用".into())
}

#[tauri::command]
pub async fn app_runtime_open(
    app: AppHandle,
    id: String,
    title: Option<String>,
    orientation: Option<String>,
    entry: Option<String>,
) -> Result<(), String> {
    window::open_app(
        &app,
        &id,
        title.as_deref().unwrap_or(""),
        orientation.as_deref(),
        entry.as_deref(),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub fn app_runtime_close(app: AppHandle, id: String) -> Result<(), String> {
    window::close_app(&app, &id)
}

#[tauri::command]
pub fn app_runtime_ready(window: WebviewWindow) -> Result<(), String> {
    let _app_id = caller_app_id(&window)?;
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
pub fn app_runtime_exit(window: WebviewWindow) -> Result<(), String> {
    let _app_id = caller_app_id(&window)?;
    window.close().map_err(|e| format!("退出应用失败: {e}"))
}

#[tauri::command]
pub fn app_runtime_storage_get(
    window: WebviewWindow,
    key: String,
) -> Result<Option<Value>, String> {
    let app_id = caller_app_id(&window)?;
    let root = crate::storage::root(window.app_handle())?;
    storage::get(&root, app_id, &key)
}

#[tauri::command]
pub fn app_runtime_storage_set(
    window: WebviewWindow,
    key: String,
    value: Value,
) -> Result<(), String> {
    let app_id = caller_app_id(&window)?;
    let root = crate::storage::root(window.app_handle())?;
    storage::set(&root, app_id, &key, value)
}

#[tauri::command]
pub fn app_runtime_storage_remove(
    window: WebviewWindow,
    key: String,
) -> Result<(), String> {
    let app_id = caller_app_id(&window)?;
    let root = crate::storage::root(window.app_handle())?;
    storage::remove(&root, app_id, &key)
}

#[tauri::command]
pub fn app_runtime_storage_keys(
    window: WebviewWindow,
) -> Result<Vec<String>, String> {
    let app_id = caller_app_id(&window)?;
    let root = crate::storage::root(window.app_handle())?;
    storage::keys(&root, app_id)
}

#[tauri::command]
pub fn app_runtime_game_progress(
    window: WebviewWindow,
    score: u64,
) -> Result<(), String> {
    let app_id = caller_app_id(&window)?;
    eprintln!("[V2 App Runtime] Game progress reported for {app_id}: score = {score}");
    Ok(())
}

#[tauri::command]
pub fn app_runtime_game_result(
    window: WebviewWindow,
    score: u64,
) -> Result<(), String> {
    let app_id = caller_app_id(&window)?;
    eprintln!("[V2 App Runtime] Game result reported for {app_id}: final score = {score}");
    Ok(())
}

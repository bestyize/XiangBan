//! V2 App Runtime module for Xiaoban Desktop.

pub mod protocol;
pub mod network;
pub mod pack;
pub mod storage;
pub mod window;

pub use protocol::serve;
pub use window::AppRuntimeState;

use serde_json::Value;
use tauri::{AppHandle, Manager, WebviewWindow};
use base64::{engine::general_purpose::STANDARD, Engine};
use tauri_plugin_dialog::DialogExt;
use std::sync::atomic::{AtomicBool, Ordering};

static MBTI_SAVE_BUSY: AtomicBool = AtomicBool::new(false);

fn validate_mbti_png(data: &str) -> Result<Vec<u8>, String> {
    let encoded = data.strip_prefix("data:image/png;base64,").ok_or("图片格式无效")?;
    if encoded.len() > 2_800_000 { return Err("图片太大".into()); }
    let bytes = STANDARD.decode(encoded).map_err(|_| "图片格式无效")?;
    if bytes.len() > 2 * 1024 * 1024 || image::guess_format(&bytes).ok() != Some(image::ImageFormat::Png) {
        return Err("图片格式无效".into());
    }
    let reader = image::ImageReader::with_format(std::io::Cursor::new(&bytes), image::ImageFormat::Png);
    let (width,height) = reader.into_dimensions().map_err(|_| "图片格式无效")?;
    if width != 1080 || height != 1440 { return Err("图片尺寸无效".into()); }
    Ok(bytes)
}

#[tauri::command]
pub async fn app_runtime_save_mbti_poster(window: WebviewWindow, image: String) -> Result<String, String> {
    if caller_app_id(&window)? != "mbti" { return Err("当前作品不支持此操作".into()); }
    if MBTI_SAVE_BUSY.swap(true, Ordering::SeqCst) { return Err("已有保存窗口打开".into()); }
    let app = window.app_handle().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let bytes = validate_mbti_png(&image)?;
        let Some(file) = app.dialog().file().set_title("保存人格海报").add_filter("PNG 图片", &["png"])
            .set_file_name("小伴人格画像.png").blocking_save_file() else { return Ok("cancelled".to_string()); };
        let path = file.into_path().map_err(|_| "请选择本机文件")?;
        std::fs::write(path, bytes).map_err(|_| "图片保存失败")?;
        Ok("saved".to_string())
    }).await.map_err(|_| "图片保存失败".to_string());
    MBTI_SAVE_BUSY.store(false, Ordering::SeqCst);
    result?
}

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

#[cfg(test)]
mod mbti_export_tests {
    use super::*;
    use std::io::Cursor;

    fn png(width: u32, height: u32) -> String {
        let image = image::DynamicImage::new_rgba8(width, height);
        let mut bytes = Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        format!("data:image/png;base64,{}", STANDARD.encode(bytes.into_inner()))
    }

    #[test]
    fn only_bounded_png_with_poster_dimensions_is_accepted() {
        assert!(validate_mbti_png(&png(1080, 1440)).is_ok());
        assert!(validate_mbti_png(&png(16, 16)).is_err());
        assert!(validate_mbti_png("data:image/jpeg;base64,AA==").is_err());
        assert!(validate_mbti_png("data:image/png;base64,!!!!").is_err());
        assert!(validate_mbti_png(&format!("data:image/png;base64,{}", "A".repeat(2_800_001))).is_err());
    }
}

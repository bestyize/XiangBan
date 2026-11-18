//! The console submits a prepared wallpaper frame. Only the native host can write it or change
//! the current user's lock screen.
#[cfg(any(windows, test))]
mod picture;
#[cfg(windows)]
mod windows;

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum PictureResult {
    #[cfg(windows)]
    Applied,
    #[cfg(windows)]
    Manual { id: String },
}

#[tauri::command]
pub fn lock_screen_picture_supported() -> bool {
    cfg!(windows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PictureSource {
    url: String,
    video: bool,
}

#[tauri::command]
pub async fn lock_screen_wallpaper_source(
    window: tauri::WebviewWindow,
    id: String,
) -> Result<PictureSource, String> {
    if !lock_screen_picture_supported() {
        return Err("当前平台不支持设置独立锁屏图片".into());
    }
    use crate::{content, library};
    use tauri::Manager;

    let app = window.app_handle().clone();
    let wallpaper_id = id.clone();
    let source = tauri::async_runtime::spawn_blocking(move || -> Result<Option<PictureSource>, String> {
        let state = app.state::<content::ContentState>();
        let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
        let root = content::root(&app)?;
        let item = library::all_items(&root).into_iter().find(|item| item.id == wallpaper_id)
            .ok_or("找不到所选壁纸")?;
        library::validate_stored(&item)?;
        if item.kind != "image" {
            if let Some(cover) = &item.cover {
                let ready = library::download(&root, cover, |_| {});
                if ready.is_ok() {
                    return Ok(Some(PictureSource { url: library::asset_url(cover), video: false }));
                }
                if item.kind == "canvas" { ready?; }
            }
        }
        match item.kind.as_str() {
            "image" | "video" => {
                library::download(&root, &item.resource, |_| {})?;
                Ok(Some(PictureSource { url: library::asset_url(&item.resource), video: item.kind == "video" }))
            }
            "web" if crate::local_media::is_local(&item) => Ok(None),
            _ => Err("这幅壁纸没有可用的静态锁屏画面".into()),
        }
    }).await.map_err(|e| e.to_string())??;
    match source {
        Some(source) => Ok(source),
        None => Ok(PictureSource { url: content::wallpaper_ensure_cover(window, id).await?, video: false }),
    }
}

#[tauri::command]
pub async fn lock_screen_apply_picture(
    window: tauri::WebviewWindow,
    jpeg: String,
) -> Result<PictureResult, String> {
    #[cfg(windows)]
    {
        let app = window.app_handle().clone();
        tauri::async_runtime::spawn_blocking(move || {
            let _operation = picture::OPERATION.lock().unwrap_or_else(|e| e.into_inner());
            let directory = crate::content::root(&app)?.join("lock-screen");
            let (id, path) = picture::prepare(&directory, &jpeg)?;
            match windows::apply_picture(&path) {
                Ok(()) => {
                    picture::remove_previous(&directory, &path);
                    Ok(PictureResult::Applied)
                }
                Err(error) => {
                    // The system may prohibit changing the image for this account. Keep the
                    // exact prepared frame for the explicit save-and-open-settings action.
                    eprintln!("Windows lock screen image was not applied: {error}");
                    Ok(PictureResult::Manual { id })
                }
            }
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(windows))]
    {
        let _ = (window, jpeg);
        Err("当前平台不支持设置独立锁屏图片".into())
    }
}

#[tauri::command]
pub async fn lock_screen_save_picture(
    window: tauri::WebviewWindow,
    id: String,
) -> Result<bool, String> {
    #[cfg(windows)]
    {
        use tauri_plugin_dialog::DialogExt;

        let app = window.app_handle().clone();
        tauri::async_runtime::spawn_blocking(move || {
            let path = picture::prepared_path(&crate::content::root(&app)?.join("lock-screen"), &id)?;
            let bytes = std::fs::read(&path).map_err(|_| "找不到准备好的锁屏图片，请重新应用")?;
            let Some(destination) = app
                .dialog()
                .file()
                .set_parent(&window)
                .set_title("保存锁屏图片")
                .add_filter("JPEG 图片", &["jpg"])
                .set_file_name("小伴锁屏.jpg")
                .blocking_save_file()
            else {
                return Ok(false);
            };
            let destination = destination.into_path().map_err(|_| "请选择本机文件")?;
            std::fs::write(destination, bytes).map_err(|e| format!("锁屏图片保存失败：{e}"))?;
            windows::open_settings().map_err(|_| "图片已保存，无法打开系统锁屏设置")?;
            Ok(true)
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(windows))]
    {
        let _ = (window, id);
        Err("当前平台不支持打开 Windows 锁屏设置".into())
    }
}

//! Independent, trusted interactive scenes. Desktop backgrounds keep their own lifecycle.
use serde_json::{json, Value};
use std::{fs, sync::Mutex};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Default)]
pub struct RitualState {
    operation: tauri::async_runtime::Mutex<()>,
    io: Mutex<()>,
}
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if label == "ritual" { return Some(["close_ritual_space", "read_ritual_journal", "write_ritual_journal"].contains(&command)); }
    if ["open_ritual_space", "close_ritual_space", "read_ritual_journal", "write_ritual_journal"].contains(&command) { return Some(label == "control"); }
    None
}
#[tauri::command]
pub async fn open_ritual_space(app: tauri::AppHandle, scene: String, fortune_id: Option<u32>) -> Result<(), String> {
    if !["fortune", "pond"].contains(&scene.as_str()) { return Err("没有找到这个互动场景".into()); }
    if fortune_id.is_some_and(|id| id >= 12) { return Err("好签编号无效".into()); }
    let suffix=fortune_id.map(|id|format!("&sign={id}")).unwrap_or_default();
    let state = app.state::<RitualState>();
    let _operation = state.operation.lock().await;
    if crate::games::is_active(&app) { return Err("请先退出游戏空间，再进入互动壁纸".into()); }
    if let Some(window) = app.get_webview_window("ritual") {
        window.eval(format!("location.replace('index.html?scene=ritual&ritual={scene}{suffix}');")).map_err(|e| e.to_string())?;
        window.unminimize().and_then(|_| window.show()).and_then(|_| window.set_focus()).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let config = crate::settings(&app);
    let builder = WebviewWindowBuilder::new(&app, "ritual", WebviewUrl::App(format!("index.html?scene=ritual&ritual={scene}{suffix}").into()))
        .title("小伴桌面 · 互动壁纸").decorations(false).fullscreen(true).visible(false)
        .initialization_script(format!("window.__XIAOBAN_RITUAL_CONFIG__={};", json!({"paused": config.paused, "fps": config.fps})))
        .always_on_top(false).skip_taskbar(false);
    #[cfg(windows)] let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(&app)?));
    #[cfg(target_os = "macos")] let builder = builder.incognito(true);
    let window = builder.build().map_err(|e| e.to_string())?;
    if let Err(error) = window.show().and_then(|_| window.set_focus()) { let _ = window.destroy(); return Err(error.to_string()); }
    Ok(())
}
#[tauri::command]
pub fn close_ritual_space(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("ritual") { window.destroy().map_err(|e| e.to_string())?; }
    if let Some(window) = app.get_webview_window("control") { window.show().and_then(|_| window.set_focus()).map_err(|e| e.to_string())?; }
    Ok(())
}
fn validate_journal(data: &Value) -> Result<(), String> {
    if data["version"] != 1 { return Err("心愿记录版本不正确".into()); }
    let saved = data["saved"].as_array().ok_or("收藏记录格式不正确")?;
    let wishes = data["wishes"].as_array().ok_or("心愿记录格式不正确")?;
    if saved.len() > 12 || saved.iter().any(|id| id.as_u64().is_none_or(|id| id >= 12)) || wishes.len() > 24 { return Err("心愿记录超出限制".into()); }
    for wish in wishes {
        if wish["id"].as_str().is_none_or(|s| s.len() > 64)
            || wish["text"].as_str().is_none_or(|s| s.trim().is_empty() || s.chars().count() > 60)
            || !["平安", "顺遂", "相逢", "自在"].contains(&wish["theme"].as_str().unwrap_or(""))
            || wish["x"].as_f64().is_none_or(|v| !(0.1..=0.9).contains(&v))
            || wish["y"].as_f64().is_none_or(|v| !(0.25..=0.85).contains(&v)) { return Err("心愿记录格式不正确".into()); }
    }
    if serde_json::to_vec(data).map_err(|e| e.to_string())?.len() > 16384 { return Err("心愿记录过大".into()); }
    Ok(())
}
#[tauri::command]
pub fn read_ritual_journal(app: tauri::AppHandle) -> Result<Value, String> {
    let state = app.state::<RitualState>(); let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    let path = crate::storage::root(&app)?.join("config/rituals.json");
    match fs::metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(json!({"version": 1, "saved": [], "wishes": []})),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if metadata.len() > 16384 => return Err("心愿记录过大，原记录已保留".into()),
        _ => {}
    }
    let data: Value = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| format!("心愿记录无法读取，原记录已保留：{e}"))?;
    validate_journal(&data)?; Ok(data)
}
#[tauri::command]
pub fn write_ritual_journal(app: tauri::AppHandle, data: Value) -> Result<(), String> {
    validate_journal(&data)?;
    let state = app.state::<RitualState>(); let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
    let folder = crate::storage::root(&app)?.join("config");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let temporary = folder.join("rituals.tmp");
    fs::write(&temporary, serde_json::to_vec(&data).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, folder.join("rituals.json")).map_err(|e| format!("保存失败，原记录已保留：{e}"))?;
    drop(_io);
    if crate::settings(&app).wallpaper_id == "builtin-wish-pond" { crate::content::sync(&app); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journal_rejects_unbounded_and_invalid_wishes() {
        let mut data = json!({"version":1,"saved":[0,11],"wishes":[{"id":"one","text":"愿你平安","theme":"平安","x":0.5,"y":0.6}]});
        assert!(validate_journal(&data).is_ok());
        data["wishes"][0]["text"] = json!("福".repeat(61)); assert!(validate_journal(&data).is_err());
        data["wishes"] = json!([]); data["saved"] = json!([12]); assert!(validate_journal(&data).is_err());
    }
    #[test]
    fn scene_has_only_its_own_commands() {
        assert_eq!(permitted("ritual", "write_ritual_journal"), Some(true));
        assert_eq!(permitted("ritual", "apply_wallpaper"), Some(false));
        assert_eq!(permitted("media-a", "open_ritual_space"), Some(false));
    }
}

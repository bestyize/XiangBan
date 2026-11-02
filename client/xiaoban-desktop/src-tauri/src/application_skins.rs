//! Application skin commands and Agent actions share one package parser and persistent state.
mod package;
use package::*;
use serde_json::{json, Value};
use std::{fs, path::Path, sync::Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

static LOCK: Mutex<()> = Mutex::new(());
fn changed(app: &tauri::AppHandle) {
    // A notification only; the console reloads validated state instead of trusting event data.
    let _ = app.emit_to("control", "application-skins-changed", ());
}
fn ensure_control(w: &tauri::WebviewWindow) -> Result<(), String> {
    if w.label() == "control" {
        Ok(())
    } else {
        Err("只能从控制台管理皮肤".into())
    }
}

pub(crate) fn read_format() -> Value {
    format_guide()
}
pub(crate) fn agent_list(app: &tauri::AppHandle) -> Result<Value, String> {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    catalog(&crate::storage::root(app)?)
}
pub(crate) fn apply(app: &tauri::AppHandle, id: Option<&str>) -> Result<Value, String> {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let state = package::apply(&crate::storage::root(app)?, id)?;
    changed(app);
    Ok(state)
}

pub(crate) fn create(
    app: &tauri::AppHandle,
    workspace: &Path,
    args: &Value,
    ctx: &agent_kernel::ToolCtx,
) -> Result<Value, String> {
    let bytes = if let Some(source) = args.get("source_dir").and_then(Value::as_str) {
        if ["id", "name", "background_image", "author"]
            .iter()
            .any(|key| args.get(key).is_some())
        {
            return Err("source_dir 与图片创建参数不能同时使用".into());
        }
        from_directory(workspace, source)?
    } else {
        let id = args["id"].as_str().ok_or("缺少皮肤 id")?;
        let name = args["name"].as_str().ok_or("缺少皮肤 name")?;
        let author = match args.get("author") {
            None => "小伴",
            Some(v) => v.as_str().ok_or("author 必须是文字")?,
        };
        let source = args["background_image"]
            .as_str()
            .ok_or("缺少背景图片 background_image")?;
        let path = if let Some(id) = source.strip_prefix("generated:") {
            crate::chat::image_generation::skin_image_path(app, id)?
        } else {
            workspace_path(workspace, source)?
        };
        from_image(id, name, author, &read_image(&path)?)?
    };
    let root = crate::storage::root(app)?;
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if ctx.is_cancelled() {
        return Err("皮肤创作已取消，未安装或应用".into());
    }
    let result = deliver(&root, workspace, &bytes)?;
    changed(app);
    Ok(result)
}

pub(crate) fn export_to_workspace(
    app: &tauri::AppHandle,
    workspace: &Path,
    id: &str,
) -> Result<Value, String> {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bytes = read(&file(&crate::storage::root(app)?, id)?)?;
    let skin = parse(&bytes)?;
    let output = workspace
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(format!("skin-{id}.zip"));
    write_new(&output, &bytes)?;
    Ok(
        json!({"id":id,"name":skin["name"],"packagePath":output.display().to_string(),"bytes":bytes.len()}),
    )
}

#[tauri::command]
pub fn skin_list(window: tauri::WebviewWindow) -> Result<Value, String> {
    ensure_control(&window)?;
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    listing(&crate::storage::root(window.app_handle())?)
}
#[tauri::command]
pub fn skin_apply(window: tauri::WebviewWindow, id: Option<String>) -> Result<Value, String> {
    ensure_control(&window)?;
    apply(window.app_handle(), id.as_deref())
}
#[tauri::command]
pub fn skin_delete(window: tauri::WebviewWindow, id: String) -> Result<Value, String> {
    ensure_control(&window)?;
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = crate::storage::root(window.app_handle())?;
    let path = file(&root, &id)?;
    let was_active = listing(&root)?["activeId"].as_str() == Some(id.as_str());
    // A failed deletion must not change the selected skin.
    fs::remove_file(path).map_err(|e| e.to_string())?;
    if was_active {
        save_active(&root, None)?;
    }
    let state = listing(&root)?;
    changed(window.app_handle());
    Ok(state)
}
#[tauri::command]
pub async fn skin_import(window: tauri::WebviewWindow) -> Result<Option<Value>, String> {
    ensure_control(&window)?;
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(p) = app
            .dialog()
            .file()
            .set_title("导入皮肤")
            .add_filter("小伴皮肤", &["zip"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = p.into_path().map_err(|_| "请选择本机文件")?;
        let data = read(&path)?;
        let root = crate::storage::root(&app)?;
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let skin = install(&root, &data)?;
        changed(&app);
        Ok(Some(skin))
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn skin_export(window: tauri::WebviewWindow, id: String) -> Result<bool, String> {
    ensure_control(&window)?;
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let root = crate::storage::root(&app)?;
        let data = {
            let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let bytes = read(&file(&root, &id)?)?;
            parse(&bytes)?;
            bytes
        };
        let Some(p) = app
            .dialog()
            .file()
            .set_title("导出皮肤")
            .add_filter("小伴皮肤", &["zip"])
            .set_file_name(format!("skin-{id}.zip"))
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = p.into_path().map_err(|_| "请选择本机文件")?;
        fs::write(path, data).map_err(|e| e.to_string())?;
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

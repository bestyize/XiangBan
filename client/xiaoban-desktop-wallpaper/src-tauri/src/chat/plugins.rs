//! WorkBuddy 格式的插件（专家、专家团、连接器、技能包）：本地安装、列表和卸载。
//! 和 AI 商店其他命令一样：只有控制窗口能调用，网络和文件工作不占用界面线程，安装前先审查、按清单摘要锁定。
//! 安装只写文件，不执行插件里的任何代码；插件自带的技能由技能加载器作为只读根读取。
use super::{store, Dirs};
use agent_skill_discovery::plugin;
use serde_json::{json, Value};
use std::path::PathBuf;

const MAX_PLUGIN_BYTES: u64 = 12 * 1024 * 1024;

fn read_zip(path: &str) -> Result<Vec<u8>, String> {
    let path = PathBuf::from(path);
    let meta = std::fs::metadata(&path).map_err(|e| format!("无法读取插件包：{e}"))?;
    if !meta.is_file() || meta.len() > MAX_PLUGIN_BYTES {
        return Err("插件包必须是不超过 12 MiB 的 ZIP 文件".into());
    }
    std::fs::read(&path).map_err(|e| format!("无法读取插件包：{e}"))
}

/// 已安装的插件（每个插件的安装回执）。
#[tauri::command]
pub async fn ai_plugin_installed(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        Ok(Value::Array(plugin::installed(&Dirs::of(&app)?.plugins())))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 审查本地插件包：返回名称、版本、类型、文件清单、提示和清单摘要，不写任何文件。
#[tauri::command]
pub async fn ai_plugin_inspect_file(window: tauri::WebviewWindow, path: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || Ok(plugin::inspect_zip(&read_zip(&path)?)?.summary()))
        .await
        .map_err(|e| e.to_string())?
}

/// 安装已审查的本地插件包。`sha256` 是审查时看到的清单摘要，包被换过就拒绝。
#[tauri::command]
pub async fn ai_plugin_install_file(app: tauri::AppHandle, window: tauri::WebviewWindow, path: String, sha256: String) -> Result<Value, String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let pkg = plugin::inspect_zip(&read_zip(&path)?)?;
        if !pkg.manifest_sha256.eq_ignore_ascii_case(&sha256) {
            return Err("插件包与审查时不一致，已拒绝安装".to_string());
        }
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        let dirs = Dirs::of(&app)?;
        plugin::install(&dirs.plugins(), &dirs.plugin_backups(), &pkg, "local", json!({"service": "local-file"}))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_plugin_remove(app: tauri::AppHandle, window: tauri::WebviewWindow, name: String) -> Result<(), String> {
    store::control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = store::MUTATIONS.lock().map_err(|e| e.to_string())?;
        plugin::remove(&Dirs::of(&app)?.plugins(), &name)
    })
    .await
    .map_err(|e| e.to_string())?
}

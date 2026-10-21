//! Control-window storefront. Network/archive work stays off the UI thread.
use super::{Dirs, sync_community_skill, sync_computer_use_skill};
use agent_kernel::skills::Skills;
use agent_skill_discovery::SkillDiscovery;
use serde_json::{json, Value};
use std::{fs, path::Path, sync::Mutex};

static MUTATIONS: Mutex<()> = Mutex::new(());

fn control(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "control" { Ok(()) } else { Err("仅控制台可以管理 AI 商店".into()) }
}
fn discovery(app: &tauri::AppHandle) -> Result<SkillDiscovery, String> {
    Ok(SkillDiscovery::new(Dirs::of(app)?.skills(), crate::ai_config::default_workspace(app)?))
}
fn removable(path: &Path, slug: &str) -> bool {
    // Only packages installed by SkillHub can be removed here; bundled and user-authored
    // skill directories never become deletable merely because they are writable.
    if matches!(slug, "xiaoban-community" | "open-computer-use" | "file-analysis") { return false; }
    fs::symlink_metadata(path).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
        && fs::read_to_string(path.join(".skillhub-lock.json")).ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .is_some_and(|v| v["provider"] == "skillhub" && v["slug"] == slug)
}

#[tauri::command]
pub async fn ai_store_installed(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = MUTATIONS.lock().map_err(|e| e.to_string())?;
        let dirs = Dirs::of(&app)?;
        let config = crate::ai_config::current(&app)?;
        sync_community_skill(&dirs.skills());
        sync_computer_use_skill(&dirs.skills(), config.open_computer_use);
        let skills = Skills::new().read_only_root(dirs.chat.join("bundled-skills")).writable_root(dirs.skills());
        Ok(json!(skills.list().iter().map(|s| {
            let managed = s.writable && removable(&s.dir, &s.name);
            let lock = fs::read_to_string(s.dir.join(".skillhub-lock.json")).ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or(Value::Null);
            json!({"slug":s.name,"name":s.name,"description":s.description,
                "version":lock["version"].as_str().unwrap_or(""),"removable":managed,
                "source":if managed {"SkillHub"} else if s.agent_created {"自建"} else {"本地"}})
        }).collect::<Vec<_>>()))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_store_browse(window: tauri::WebviewWindow, keyword: String, page: u32) -> Result<Value, String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || SkillDiscovery::browse(&keyword, page))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_store_inspect(app: tauri::AppHandle, window: tauri::WebviewWindow, slug: String, version: String) -> Result<Value, String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || discovery(&app)?.inspect(&slug, &version))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_store_install(app: tauri::AppHandle, window: tauri::WebviewWindow, slug: String, version: String, sha256: String) -> Result<(), String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = MUTATIONS.lock().map_err(|e| e.to_string())?;
        discovery(&app)?.install_reviewed(&slug, &version, &sha256)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_store_uninstall(app: tauri::AppHandle, window: tauri::WebviewWindow, slug: String) -> Result<(), String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = MUTATIONS.lock().map_err(|e| e.to_string())?;
        let root = Dirs::of(&app)?.skills();
        let skill = Skills::new().writable_root(root.clone()).find(&slug).ok_or("技能不存在")?;
        if !removable(&skill.dir, &slug) { return Err("只能卸载通过 SkillHub 安装的技能".into()); }
        fs::remove_dir_all(skill.dir).map_err(|e| e.to_string())
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_owned_registry_packages_can_be_removed() {
        let root = std::env::temp_dir().join(format!("xiaoban-store-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        assert!(!removable(&root, "demo-skill"));
        fs::write(root.join(".skillhub-lock.json"), r#"{"provider":"skillhub","slug":"demo-skill"}"#).unwrap();
        assert!(removable(&root, "demo-skill"));
        assert!(!removable(&root, "xiaoban-community"));
        assert!(!removable(&root, "other-skill"));
        fs::remove_dir_all(root).unwrap();
    }
}

//! Control-window storefront. Network/archive work stays off the UI thread.
use super::{Dirs, sync_community_skill, sync_computer_use_skill};
use agent_kernel::skills::Skills;
use agent_skill_discovery::SkillDiscovery;
use serde_json::{json, Value};
use std::{fs, path::Path, sync::Mutex};

pub(super) static MUTATIONS: Mutex<()> = Mutex::new(());

#[path = "store_catalog.rs"]
mod catalog;

#[tauri::command]
pub async fn ai_store_catalog(window:tauri::WebviewWindow,kind:String,keyword:String,page:u32,category:String)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||agent_skill_discovery::catalog::browse(&kind,&keyword,page,&category)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_experts_installed(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||{
        let _guard=MUTATIONS.lock().map_err(|e|e.to_string())?;
        let dirs=Dirs::of(&app)?;
        agent_skill_discovery::packs::installed(&dirs.skills(),&dirs.chat.join("expert-packs"))
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_expert_review(app:tauri::AppHandle,window:tauri::WebviewWindow,slug:String)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||agent_skill_discovery::packs::inspect(&discovery(&app)?,&slug)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_expert_install(app:tauri::AppHandle,window:tauri::WebviewWindow,slug:String,sha256:String)->Result<(),String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||{
        let _guard=MUTATIONS.lock().map_err(|e|e.to_string())?;
        agent_skill_discovery::packs::install(&discovery(&app)?,&Dirs::of(&app)?.chat.join("expert-packs"),&slug,&sha256)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_expert_remove(app:tauri::AppHandle,window:tauri::WebviewWindow,slug:String)->Result<(),String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||{
        let _guard=MUTATIONS.lock().map_err(|e|e.to_string())?;
        let dirs=Dirs::of(&app)?;
        agent_skill_discovery::packs::uninstall(&dirs.skills(),&dirs.chat.join("expert-packs"),&slug)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_connector_detail(window:tauri::WebviewWindow,slug:String)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||agent_skill_discovery::catalog::connector(&slug)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_connector_status(app:tauri::AppHandle,window:tauri::WebviewWindow)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||Ok(super::connector_manager(&app)?.statuses())).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_connector_connect(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String,name:String,config:Value,confirmed:bool)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||super::connector_manager(&app)?.begin_connect(&id,&name,config,confirmed)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_connector_disconnect(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String)->Result<(),String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||super::connector_manager(&app)?.disconnect(&id)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_connector_reconnect(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||super::connector_manager(&app)?.reconnect_with_authorization(&id)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_connector_auth_status(app:tauri::AppHandle,window:tauri::WebviewWindow,flow_id:String)->Result<Value,String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||super::connector_manager(&app)?.authorization_status(&flow_id)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn ai_store_connector_auth_cancel(app:tauri::AppHandle,window:tauri::WebviewWindow,flow_id:String)->Result<(),String>{
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move||super::connector_manager(&app)?.cancel_authorization(&flow_id)).await.map_err(|e|e.to_string())?
}

pub(super) fn control(window: &tauri::WebviewWindow) -> Result<(), String> {
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
            let market = managed && super::store_market::lock_summary(&lock)["market"] == true;
            json!({"slug":s.name,"name":s.name,"description":s.description,
                "version":lock["version"].as_str().unwrap_or(""),"namespace":lock["namespace"],"iconUrl":lock["iconUrl"],"removable":managed,
                "market":market,"marketKind":if market {lock["origin"]["kind"].clone()} else {Value::Null},"files":if market {lock["files"].clone()} else {Value::Null},"manifestSha256":if market {lock["manifestSha256"].clone()} else {Value::Null},
                "source":if market {"小伴市场"} else if managed {"SkillHub"} else if s.agent_created {"自建"} else {"本地"}})
        }).collect::<Vec<_>>()))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_store_skill_categories(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<Value, String> {
    control(&window)?;
    match catalog_json(&app, "/api/v1/ai/plugins/categories", &[]).await? {
        Some(value) => Ok(value),
        None => tauri::async_runtime::spawn_blocking(SkillDiscovery::categories)
            .await.map_err(|e| e.to_string())?,
    }
}

#[tauri::command]
pub async fn ai_store_browse(
    app: tauri::AppHandle, window: tauri::WebviewWindow, keyword: String, page: u32,
    category: String, source: String, sort_by: String,
) -> Result<Value, String> {
    control(&window)?;
    match catalog_json(&app, "/api/v1/ai/plugins", &[
        ("keyword", keyword.clone()), ("page", page.to_string()), ("category", category.clone()),
        ("source", source.clone()), ("sortBy", sort_by.clone()),
    ]).await? {
        Some(value) => Ok(value),
        None => tauri::async_runtime::spawn_blocking(move ||
            SkillDiscovery::browse(&keyword, page, &category, &source, &sort_by))
            .await.map_err(|e| e.to_string())?,
    }
}

// Prefer the configured Xiaoban website. Only an explicit HTTP 404 permits legacy discovery;
// other errors propagate through `?`, including 403/429 and invalid successful responses.
async fn catalog_json(app: &tauri::AppHandle, path: &str, pairs: &[(&str, String)]) -> Result<Option<Value>, String> {
    let origin = crate::website::site_url(&crate::settings(app).website_url)?;
    catalog::fetch(&origin, path, pairs).await
}

/// The assistant's `skill_search` asks the same catalog as this page. `None` only when the website has no such route.
pub(super) async fn search_skills(app: &tauri::AppHandle, keyword: &str, page: u32, sort_by: &str) -> Result<Option<Value>, String> {
    catalog_json(app, "/api/v1/ai/plugins", &[
        ("keyword", keyword.to_string()), ("page", page.to_string()), ("category", String::new()),
        ("source", String::new()), ("sortBy", sort_by.to_string()),
    ]).await
}

#[tauri::command]
pub async fn ai_store_inspect(app: tauri::AppHandle, window: tauri::WebviewWindow, slug: String, version: String, namespace: Option<String>) -> Result<Value, String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || discovery(&app)?.inspect_coordinate(&slug, &version, namespace.as_deref()))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_store_install(app: tauri::AppHandle, window: tauri::WebviewWindow, slug: String, version: String, sha256: String, namespace: Option<String>) -> Result<(), String> {
    control(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = MUTATIONS.lock().map_err(|e| e.to_string())?;
        discovery(&app)?.install_coordinate(&slug, &version, namespace.as_deref(), &sha256)
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


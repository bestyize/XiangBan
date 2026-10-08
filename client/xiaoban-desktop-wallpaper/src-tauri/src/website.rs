//! Native website transport keeps networking out of wallpaper scripts and the control WebView.
use crate::{content, library, settings, DesktopState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Mutex, time::Duration};
use tauri::{Emitter, Manager};

pub const DEFAULT_SITE: &str = "https://wallpaper.xiangban.xyz";
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub id: String,
    pub title: String,
    pub body: String,
    pub revision: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Status {
    pub announcements: Vec<Notice>,
    pub update: Option<Value>,
    pub dismissed: BTreeMap<String, String>,
    pub errors: Vec<String>,
}
#[derive(Default)]
pub struct WebsiteState {
    operation: tauri::async_runtime::Mutex<()>,
    status: Mutex<Status>,
}
pub fn site_url(value: &str) -> Result<String, String> {
    let value = value.trim().trim_end_matches('/');
    let url = library::network_url(value)?;
    if url.path() != "/" || url.query().is_some() {
        return Err("填写网站源站地址，例如 https://wallpaper.xiangban.xyz".into());
    }
    Ok(value.to_string())
}
fn endpoint(app: &tauri::AppHandle, path: &str) -> Result<String, String> {
    Ok(format!("{}{path}", site_url(&settings(app).website_url)?))
}
async fn get(url: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        library::fetch_json(&url)?.ok_or("网站没有返回数据".into())
    })
    .await
    .map_err(|e| e.to_string())?
}
fn save(app: &tauri::AppHandle, status: &Status) -> Result<(), String> {
    library::save_json(
        &crate::storage::root(app)?.join("config/website-messages.json"),
        status,
    )
}
fn snapshot(app: &tauri::AppHandle) -> Value {
    let state = app.state::<WebsiteState>();
    let status = state.status.lock().unwrap().clone();
    serde_json::to_value(status).unwrap()
}
#[tauri::command]
pub fn get_website_status(app: tauri::AppHandle) -> Value {
    snapshot(&app)
}
#[tauri::command]
pub async fn configure_website(
    app: tauri::AppHandle,
    url: String,
    automatic: bool,
) -> Result<crate::Settings, String> {
    let url = site_url(&url)?;
    let state = app.state::<WebsiteState>();
    let desktop_state = app.state::<DesktopState>();
    let _update = desktop_state.update_lock.lock().unwrap();
    let mut config = settings(&app);
    let changed = config.website_url != url;
    config.website_url = url.clone();
    config.automatic_updates = automatic;
    config.update_url = format!("{url}/api/v1/desktop/updates");
    crate::persist(&app, &config)?;
    *app.state::<DesktopState>().settings.lock().unwrap() = config;
    if changed {
        let mut status = state.status.lock().unwrap();
        *status = Status::default();
        save(&app, &status)?;
        drop(status);
        app.emit("website-status", snapshot(&app))
            .map_err(|e| e.to_string())?;
    }
    crate::publish(&app);
    Ok(settings(&app))
}
#[tauri::command]
pub async fn fetch_site_wallpapers(
    app: tauri::AppHandle,
    q: String,
    category: String,
    media: String,
    page: u32,
) -> Result<Value, String> {
    let mut url = library::network_url(&endpoint(&app, "/api/v1/wallpapers/search")?)?;
    url.query_pairs_mut()
        .append_pair("page", &page.max(1).to_string())
        .append_pair("per_page", "24")
        .append_pair("q", &q)
        .append_pair("category", &category);
    if ["image", "video"].contains(&media.as_str()) {
        url.query_pairs_mut().append_pair("media_type", &media);
    }
    serde_json::from_str(&get(url.to_string()).await?).map_err(|e| format!("网站壁纸数据无效: {e}"))
}
#[tauri::command]
pub async fn fetch_site_categories(app: tauri::AppHandle) -> Result<Value, String> {
    serde_json::from_str(&get(endpoint(&app, "/api/v1/categories")?).await?)
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn select_site_wallpaper(app: tauri::AppHandle, id: String) -> Result<Value, String> {
    let mut url = library::network_url(&endpoint(&app, "/api/v1/desktop/wallpapers/")?)?;
    url.path_segments_mut()
        .map_err(|_| "网站地址无效")?
        .pop_if_empty()
        .push(&id);
    let text = get(url.to_string()).await?;
    let catalog = library::parse_catalog(&text)?;
    let id = catalog.items.first().ok_or("网站没有返回壁纸")?.id.clone();
    let library = content::import_wallpaper_catalog(app, text).await?;
    Ok(serde_json::json!({"library":library,"id":id}))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NoticeResponse {
    schema_version: u32,
    items: Vec<Notice>,
}
fn parse_notices(text: &str) -> Result<Vec<Notice>, String> {
    let response: NoticeResponse = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if response.schema_version != 1
        || response.items.len() > 100
        || response.items.iter().any(|n| {
            n.id.is_empty()
                || n.id.len() > 64
                || n.title.len() > 160
                || n.body.len() > 16000
                || !library::hash_name(&n.revision)
        })
    {
        return Err("网站公告格式无效".into());
    }
    Ok(response.items)
}
#[tauri::command]
pub async fn check_website(app: tauri::AppHandle) -> Result<Value, String> {
    let state = app.state::<WebsiteState>();
    let _operation = state.operation.lock().await;
    let config = settings(&app);
    let notices_url = endpoint(&app, "/api/v1/desktop/announcements");
    let notices_task = tauri::async_runtime::spawn(async move {
        match notices_url {
            Ok(url) => get(url).await.and_then(|text| parse_notices(&text)),
            Err(e) => Err(e),
        }
    });
    let update = content::check_updates(app.clone()).await;
    let notices = notices_task.await.map_err(|e| e.to_string())?;
    let desktop_state = app.state::<DesktopState>();
    let _update = desktop_state.update_lock.lock().unwrap();
    let current = settings(&app);
    if current.website_url != config.website_url || current.update_url != config.update_url {
        return Ok(snapshot(&app));
    }
    {
        let mut status = state.status.lock().unwrap();
        status.errors.clear();
        match notices {
            Ok(items) => status.announcements = items,
            Err(e) => status.errors.push(format!("公告: {e}")),
        }
        match update {
            Ok(value) => status.update = Some(value),
            Err(e) => status.errors.push(format!("更新: {e}")),
        }
        // Entries disappear when withdrawn/expired. Retain dismissals to avoid repeating a
        // temporarily unavailable notice; revisions cause edited notices to become unread.
        save(&app, &status)?;
    }
    let result = snapshot(&app);
    app.emit("website-status", &result)
        .map_err(|e| e.to_string())?;
    update_tray(&app);
    Ok(result)
}
fn update_tray(app: &tauri::AppHandle) {
    if let Some(tray) = app.tray_by_id("xiaoban-tray") {
        let state = app.state::<WebsiteState>();
        let status = state.status.lock().unwrap();
        let unread = status
            .announcements
            .iter()
            .any(|n| status.dismissed.get(&n.id) != Some(&n.revision))
            || status.update.as_ref().is_some_and(|u| {
                u["available"] == true
                    && status.dismissed.get("update").map(String::as_str) != u["version"].as_str()
            });
        let _ = tray.set_tooltip(Some(if unread {
            "小伴桌面 · 有新版本或公告，打开设置查看"
        } else {
            "小伴桌面"
        }));
    }
}
#[tauri::command]
pub fn dismiss_website_message(
    app: tauri::AppHandle,
    id: String,
    revision: String,
) -> Result<Value, String> {
    let state = app.state::<WebsiteState>();
    {
        let mut status = state.status.lock().unwrap();
        let valid = if id == "update" {
            status.update.as_ref().and_then(|u| u["version"].as_str()) == Some(&revision)
        } else {
            status
                .announcements
                .iter()
                .any(|n| n.id == id && n.revision == revision)
        };
        if !valid {
            return Err("消息已更新，请重新查看".into());
        }
        status.dismissed.insert(id, revision);
        save(&app, &status)?;
    }
    let result = snapshot(&app);
    app.emit("website-status", &result)
        .map_err(|e| e.to_string())?;
    update_tray(&app);
    Ok(result)
}
pub fn start(app: &tauri::AppHandle) {
    if app.state::<DesktopState>().isolated {
        return;
    }
    if let Ok(root) = crate::storage::root(app) {
        if let Ok(text) = std::fs::read(root.join("config/website-messages.json")) {
            if let Ok(status) = serde_json::from_slice::<Status>(&text) {
                *app.state::<WebsiteState>().status.lock().unwrap() = status;
            }
        }
    }
    let handle = app.clone();
    update_tray(app);
    std::thread::spawn(move || loop {
        if settings(&handle).automatic_updates {
            let _ = tauri::async_runtime::block_on(check_website(handle.clone()));
        }
        std::thread::sleep(Duration::from_secs(30 * 60));
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_and_notice_protocol_are_checked() {
        assert_eq!(
            site_url("https://wallpaper.xiangban.xyz/").unwrap(),
            DEFAULT_SITE
        );
        assert!(site_url("https://example.com/path").is_err());
        assert!(site_url("javascript:alert(1)").is_err());
        assert!(parse_notices(r#"{"schemaVersion":2,"items":[]}"#).is_err());
        assert!(parse_notices(r#"{"schemaVersion":1,"items":[]}"#)
            .unwrap()
            .is_empty());
    }
}

pub async fn smoke(app: &tauri::AppHandle) -> Result<Value, String> {
    if !app.state::<DesktopState>().isolated {
        return Err("网站联调只能在隔离测试中运行".into());
    }
    let base = std::env::var("XIAOBAN_WEBSITE_TEST_URL").map_err(|_| "没有网站测试服务")?;
    configure_website(app.clone(), base, false).await?;
    if fetch_site_categories(app.clone()).await?["categories"]
        .as_array()
        .is_none_or(|c| c.is_empty())
    {
        return Err("分类接口没有数据".into());
    }
    let mut cases = Vec::new();
    for kind in ["image", "video"] {
        let feed = fetch_site_wallpapers(
            app.clone(),
            "网站联调".into(),
            "nature".into(),
            kind.into(),
            1,
        )
        .await?;
        let id = feed["items"][0]["id"]
            .as_str()
            .ok_or("网站检索没有返回测试壁纸")?
            .to_string();
        let selected = select_site_wallpaper(app.clone(), id).await?;
        let id = selected["id"]
            .as_str()
            .ok_or("网站未返回交付 ID")?
            .to_string();
        content::apply_wallpaper(app.clone(), id.clone(), library::Options::default(), false)
            .await?;
        cases.push(content::smoke_render_report(app, &id)?);
        // A second application verifies that the downloaded resource works from the local cache.
        content::apply_wallpaper(app.clone(), id, library::Options::default(), true).await?;
    }
    let status = check_website(app.clone()).await?;
    if status["update"]["available"] != true {
        return Err("网站版本接口没有返回测试更新".into());
    }
    let notice = &status["announcements"][0];
    let id = notice["id"]
        .as_str()
        .ok_or("网站未返回测试公告")?
        .to_string();
    let revision = notice["revision"]
        .as_str()
        .ok_or("公告缺少修订号")?
        .to_string();
    dismiss_website_message(app.clone(), id.clone(), revision.clone())?;
    let again = check_website(app.clone()).await?;
    if again["dismissed"][&id] != revision {
        return Err("公告已读状态未保留".into());
    }
    let saved: Status = serde_json::from_slice(
        &std::fs::read(crate::storage::root(app)?.join("config/website-messages.json"))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if saved.dismissed.get(&id) != Some(&revision) {
        return Err("公告状态未写入独立测试数据目录".into());
    }
    Ok(
        serde_json::json!({"cases":cases,"updateChecked":true,"announcementsChecked":true,"dismissalPersisted":true}),
    )
}

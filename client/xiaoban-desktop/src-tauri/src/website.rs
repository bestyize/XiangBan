//! Native website transport keeps networking out of wallpaper scripts and the control WebView.
use crate::{content, library, settings, DesktopState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
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
    /// Unix seconds of the last check in which both versions and announcements were fetched.
    pub last_checked: Option<i64>,
}
/// Automatic checks run at most once a day; a failed check is retried on the next wake-up.
const CHECK_EVERY_SECS: i64 = 24 * 60 * 60;
/// The background thread wakes this often to see whether a check is due. The day is measured
/// on the wall clock, so time spent asleep or with the app closed counts.
const WAKE_EVERY: Duration = Duration::from_secs(60 * 60);
fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
/// A clock set back before the last check also counts as due, so a wrong date cannot stop checks.
fn check_due(last_checked: Option<i64>, now: i64) -> bool {
    match last_checked {
        None => true,
        Some(last) => now < last || now - last >= CHECK_EVERY_SECS,
    }
}
#[derive(Default)]
pub struct WebsiteState {
    operation: tauri::async_runtime::Mutex<()>,
    status: Mutex<Status>,
    /// Cancel flag of the wallpaper download in progress, if any.
    download: Mutex<Option<Arc<AtomicBool>>>,
}

/// Registers a new download as the current one and clears it again when dropped, unless a
/// newer download has replaced it in the meantime.
struct DownloadGuard<'a> {
    state: &'a WebsiteState,
    cancel: Arc<AtomicBool>,
}
impl<'a> DownloadGuard<'a> {
    fn start(state: &'a WebsiteState) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        if let Some(previous) = state.download.lock().unwrap_or_else(|e| e.into_inner()).replace(cancel.clone()) {
            previous.store(true, Ordering::Relaxed);
        }
        Self { state, cancel }
    }
    fn check(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(library::CANCELLED.into())
        } else {
            Ok(())
        }
    }
}
impl Drop for DownloadGuard<'_> {
    fn drop(&mut self) {
        let mut current = self.state.download.lock().unwrap_or_else(|e| e.into_inner());
        if current.as_ref().is_some_and(|flag| Arc::ptr_eq(flag, &self.cancel)) {
            *current = None;
        }
    }
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
    let status = state.status.lock().unwrap_or_else(|e| e.into_inner()).clone();
    serde_json::to_value(status).unwrap()
}
#[tauri::command]
pub fn get_website_status(app: tauri::AppHandle) -> Value {
    snapshot(&app)
}
#[tauri::command]
pub fn set_automatic_updates(app: tauri::AppHandle, enabled: bool) -> Result<crate::Settings, String> {
    let desktop_state = app.state::<DesktopState>();
    let _update = desktop_state.update_lock.lock().unwrap_or_else(|e| e.into_inner());
    let mut config = settings(&app);
    config.automatic_updates = enabled;
    crate::persist(&app, &config)?;
    *desktop_state.settings.lock().unwrap_or_else(|e| e.into_inner()) = config;
    crate::publish(&app);
    Ok(settings(&app))
}
/// Opens the official website in the system browser. The address lives here, not in the WebView.
#[tauri::command]
pub fn open_official_site() -> Result<(), String> {
    content::open_download_url(DEFAULT_SITE.into())
}
/// Points the running app at another website. Only the isolated smoke test uses this; the
/// address is not saved across restarts because loading resets it to `DEFAULT_SITE`.
async fn use_test_website(app: tauri::AppHandle, url: String) -> Result<crate::Settings, String> {
    if !app.state::<DesktopState>().isolated {
        return Err("只有隔离测试可以更换网站".into());
    }
    let url = site_url(&url)?;
    let state = app.state::<WebsiteState>();
    let desktop_state = app.state::<DesktopState>();
    let _update = desktop_state.update_lock.lock().unwrap_or_else(|e| e.into_inner());
    let mut config = settings(&app);
    let changed = config.website_url != url;
    config.website_url = url.clone();
    config.automatic_updates = false;
    config.update_url = format!("{url}/api/v1/desktop/updates");
    crate::persist(&app, &config)?;
    *app.state::<DesktopState>().settings.lock().unwrap_or_else(|e| e.into_inner()) = config;
    if changed {
        let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
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
    let mut feed: Value = serde_json::from_str(&get(url.to_string()).await?)
        .map_err(|e| format!("网站壁纸数据无效: {e}"))?;
    let root = content::root(&app)?;
    let items = library::all_items(&root);
    if let Some(entries) = feed["items"].as_array_mut() {
        for entry in entries {
            if let Some(id) = entry["id"].as_str() {
                let id = format!(
                    "site-{}",
                    &format!("{:x}", Sha256::digest(id.as_bytes()))[..40]
                );
                let downloaded = items.iter().find(|item| item.id == id).is_some_and(|item| {
                    library::assets(item)
                        .iter()
                        .all(|r| library::cached(&root, r))
                });
                entry["desktop_id"] = Value::String(id);
                entry["downloaded"] = Value::Bool(downloaded);
            }
        }
    }
    Ok(feed)
}
#[tauri::command]
pub async fn fetch_site_categories(app: tauri::AppHandle) -> Result<Value, String> {
    serde_json::from_str(&get(endpoint(&app, "/api/v1/categories")?).await?)
        .map_err(|e| e.to_string())
}
/// Delivery manifest of one wallpaper. `prefer_source` asks the website to hand out the
/// provider's own link when its hotlink policy allows; without it the answer is our verified copy.
fn manifest_url(app: &tauri::AppHandle, id: &str, prefer_source: bool) -> Result<String, String> {
    let mut url = library::network_url(&endpoint(app, "/api/v1/desktop/wallpapers/")?)?;
    url.path_segments_mut()
        .map_err(|_| "网站地址无效")?
        .pop_if_empty()
        .push(id);
    if prefer_source {
        url.query_pairs_mut().append_pair("prefer", "source");
    }
    Ok(url.to_string())
}
/// Imports a resolved catalog into the library. Items in `local` already carry a locally
/// verified resource (source download or existing cache hit) and keep their own url.
async fn import_site_catalog(
    app: tauri::AppHandle,
    origin: &str,
    mut catalog: library::Catalog,
    local: &HashSet<String>,
) -> Result<Value, String> {
    for item in &mut catalog.items {
        if !local.contains(&item.id) {
            site_resource(&mut item.resource, origin)?;
        }
        if let Some(cover) = &mut item.cover {
            site_resource(cover, origin)?;
        }
    }
    let id = catalog.items.first().ok_or("网站没有返回壁纸")?.id.clone();
    let text = serde_json::to_string(&catalog).map_err(|e| e.to_string())?;
    let library = content::import_wallpaper_catalog(app, text).await?;
    Ok(serde_json::json!({"library":library,"id":id}))
}
#[tauri::command]
pub async fn select_site_wallpaper(app: tauri::AppHandle, id: String) -> Result<Value, String> {
    let origin = settings(&app).website_url;
    let text = get(manifest_url(&app, &id, false)?).await?;
    let catalog = library::parse_catalog(&text)?;
    import_site_catalog(app, &origin, catalog, &HashSet::new()).await
}

// "Source first, our copy as fallback". A `?prefer=source` manifest may carry
// `source: {url, mime}` per item; `resource` is then present only if the website already has
// a copy. A manifest without any `source` is what the plain endpoint returns and is handled
// by the unchanged verified-copy path.
struct SiteEntry {
    item: serde_json::Map<String, Value>,
    source: Option<Value>,
}
struct SiteManifest {
    revision: String,
    entries: Vec<SiteEntry>,
}
struct Resolved {
    catalog: library::Catalog,
    /// Items whose resource is already stored and verified locally, so nothing else is fetched.
    local: HashSet<String>,
}
/// `None` means "no item asks for a source link": the caller then treats the text exactly like
/// a plain manifest (including its errors for malformed input).
fn parse_site_manifest(text: &str) -> Option<SiteManifest> {
    if text.len() > library::MAX_JSON {
        return None;
    }
    let Value::Object(root) = serde_json::from_str(text).ok()? else {
        return None;
    };
    if root.keys().any(|k| !["schemaVersion", "revision", "items"].contains(&k.as_str()))
        || root.get("schemaVersion") != Some(&Value::from(1))
    {
        return None;
    }
    let revision = match root.get("revision") {
        None => String::new(),
        Some(Value::String(value)) => value.clone(),
        Some(_) => return None,
    };
    let Some(Value::Array(items)) = root.get("items") else {
        return None;
    };
    if !items
        .iter()
        .any(|item| item.get("source").is_some_and(|source| !source.is_null()))
    {
        return None;
    }
    let mut entries = Vec::new();
    for item in items {
        let Value::Object(item) = item else {
            return None;
        };
        let mut item = item.clone();
        let source = item.remove("source").filter(|source| !source.is_null());
        entries.push(SiteEntry { item, source });
    }
    Some(SiteManifest { revision, entries })
}
fn with_resource(
    mut item: serde_json::Map<String, Value>,
    resource: Value,
) -> Result<library::Wallpaper, String> {
    item.insert("resource".into(), resource);
    serde_json::from_value(Value::Object(item)).map_err(|e| format!("清单格式不正确：{e}"))
}
/// Decides, per item, which file the client uses:
/// 1. already saved with the same version: keep it (no network);
/// 2. `source` present: `fetch_source(id, kind, url)` downloads and registers the provider file
///    (no hash comparison with the manifest);
/// 3. on any failure: the manifest's `resource` (verified later with sha256/bytes as always);
/// 4. no `resource` either: the plain endpoint (`fetch_plain`, called at most once) and its copy.
fn resolve_manifest(
    manifest: SiteManifest,
    cached: impl Fn(&str, &str) -> Option<library::Resource>,
    mut fetch_source: impl FnMut(&str, &str, &str) -> Result<library::Resource, String>,
    fetch_plain: impl FnOnce() -> Result<String, String>,
) -> Result<Resolved, String> {
    let mut fetch_plain = Some(fetch_plain);
    let mut plain: Option<library::Catalog> = None;
    let mut items = Vec::new();
    let mut local = HashSet::new();
    for entry in manifest.entries {
        let mut item = entry.item;
        let text = |key: &str| item.get(key).and_then(Value::as_str).unwrap_or("").to_string();
        let (id, version, kind) = (text("id"), text("version"), text("kind"));
        let declared = item.remove("resource");
        let mut resolved = None;
        if let Some(source) = entry.source {
            let stored = cached(&id, &version);
            let outcome = match stored {
                Some(resource) => Ok(resource),
                None => source
                    .get("url")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "来源链接缺失".to_string())
                    .and_then(|url| fetch_source(&id, &kind, url)),
            };
            match outcome.and_then(|resource| {
                with_resource(item.clone(), serde_json::to_value(resource).map_err(|e| e.to_string())?)
            }) {
                Ok(wallpaper) => {
                    local.insert(id.clone());
                    resolved = Some(wallpaper);
                }
                // A stop requested by the user is not a broken link: do not start the copy.
                Err(error) if error == library::CANCELLED => return Err(error),
                Err(error) => eprintln!("来源链接不可用，改用网站副本：{error}"),
            }
        }
        let wallpaper = match (resolved, declared) {
            (Some(wallpaper), _) => wallpaper,
            (None, Some(resource)) => with_resource(item, resource)?,
            (None, None) => {
                if plain.is_none() {
                    let fetch = fetch_plain.take().ok_or("网站没有返回壁纸")?;
                    plain = Some(library::parse_catalog(&fetch()?)?);
                }
                plain
                    .iter()
                    .flat_map(|catalog| &catalog.items)
                    .find(|wallpaper| wallpaper.id == id)
                    .cloned()
                    .ok_or("网站没有返回壁纸")?
            }
        };
        items.push(wallpaper);
    }
    let catalog = library::Catalog {
        schema_version: 1,
        revision: manifest.revision,
        items,
    };
    let catalog = library::parse_catalog(&serde_json::to_string(&catalog).map_err(|e| e.to_string())?)?;
    Ok(Resolved { catalog, local })
}
/// The saved copy of a website wallpaper, if the same version is fully cached and intact.
fn saved_resource(root: &std::path::Path, id: &str, version: &str) -> Option<library::Resource> {
    let item = library::read_catalog(root)
        .items
        .into_iter()
        .find(|item| item.id == id && item.version == version)?;
    library::assets(&item)
        .iter()
        .all(|r| library::cached(root, r))
        .then(|| item.resource.clone())
        .filter(|r| {
            library::verify_file(&crate::local_media::resource_path(root, r), r).unwrap_or(false)
        })
}
// The local website preview uses the same unconfigured product defaults as production.
// Only its own content-addressed uploads may be rebased; CDN URLs and public sites retain
// the addresses supplied by the service. Size/hash verification remains mandatory.
fn site_resource(resource: &mut library::Resource, origin: &str) -> Result<(), String> {
    let base = library::network_url(origin)?;
    let url = library::network_url(&resource.url)?;
    if library::hash_name(&resource.sha256)
        && url
            .path()
            .starts_with(&format!("/uploads/desktop-{}.", resource.sha256))
    {
        resource.url = base
            .join(url.path())
            .map_err(|e| e.to_string())?
            .to_string();
    }
    Ok(())
}
#[tauri::command]
pub async fn download_site_wallpaper(app: tauri::AppHandle, id: String) -> Result<Value, String> {
    let state = app.state::<WebsiteState>();
    let guard = DownloadGuard::start(state.inner());
    let origin = settings(&app).website_url;
    let root = content::root(&app)?;
    let text = get(manifest_url(&app, &id, true)?).await?;
    guard.check()?;
    let (selected, local) = match parse_site_manifest(&text) {
        // No source link: exactly the verified-copy flow (also for servers without `prefer`).
        None => (
            import_site_catalog(
                app.clone(),
                &origin,
                library::parse_catalog(&text)?,
                &HashSet::new(),
            )
            .await?,
            HashSet::new(),
        ),
        Some(manifest) => {
            let (handle, directory, cancel) = (app.clone(), root.clone(), guard.cancel.clone());
            let plain = manifest_url(&app, &id, false)?;
            let resolved = tauri::async_runtime::spawn_blocking(move || {
                let state = handle.state::<content::ContentState>();
                let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
                resolve_manifest(
                    manifest,
                    |id, version| saved_resource(&directory, id, version),
                    |id, kind, url| {
                        let mut last = 0u64;
                        library::download_source_cancellable(&directory, url, kind, &cancel, |received, total| {
                            let Some(total) = total else { return };
                            if received.saturating_sub(last) >= 256 * 1024 || received >= total {
                                last = received;
                                let _ = handle.emit_to(
                                    "control",
                                    "wallpaper-progress",
                                    serde_json::json!({"id":id,"received":received,"total":total}),
                                );
                            }
                        })
                    },
                    || library::fetch_json(&plain)?.ok_or("网站没有返回数据".into()),
                )
            })
            .await
            .map_err(|e| e.to_string())??;
            (
                import_site_catalog(app.clone(), &origin, resolved.catalog, &resolved.local)
                    .await?,
                resolved.local,
            )
        }
    };
    guard.check()?;
    let id = selected["id"].as_str().ok_or("网站未返回壁纸")?.to_string();
    if !local.contains(&id) {
        let item = library::all_items(&root)
            .into_iter()
            .find(|item| item.id == id)
            .ok_or("壁纸记录不存在")?;
        let (handle, cancel) = (app.clone(), guard.cancel.clone());
        tauri::async_runtime::spawn_blocking(move || {
            let state = handle.state::<content::ContentState>();
            let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
            library::prepare_cancellable(&handle, &root, &item, &cancel)
        })
        .await
        .map_err(|e| e.to_string())??;
    }
    Ok(serde_json::json!({"id":id,"library":content::get_wallpaper_library(app.clone())?}))
}
/// Stops the wallpaper download in progress; it then fails with `library::CANCELLED`.
/// Returns whether there was one to stop.
#[tauri::command]
pub fn cancel_site_download(app: tauri::AppHandle) -> bool {
    match app.state::<WebsiteState>().download.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        Some(flag) => {
            flag.store(true, Ordering::Relaxed);
            true
        }
        None => false,
    }
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
    let _update = desktop_state.update_lock.lock().unwrap_or_else(|e| e.into_inner());
    let current = settings(&app);
    if current.website_url != config.website_url || current.update_url != config.update_url {
        return Ok(snapshot(&app));
    }
    {
        let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
        status.errors.clear();
        match notices {
            Ok(items) => status.announcements = items,
            Err(e) => status.errors.push(format!("公告: {e}")),
        }
        match update {
            Ok(value) => status.update = Some(value),
            Err(e) => status.errors.push(format!("更新: {e}")),
        }
        if status.errors.is_empty() {
            status.last_checked = Some(now_secs());
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
        let status = state.status.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
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
                *app.state::<WebsiteState>().status.lock().unwrap_or_else(|e| e.into_inner()) = status;
            }
        }
    }
    let handle = app.clone();
    update_tray(app);
    std::thread::spawn(move || loop {
        let last_checked = handle.state::<WebsiteState>().status.lock().unwrap_or_else(|e| e.into_inner()).last_checked;
        if settings(&handle).automatic_updates && check_due(last_checked, now_secs()) {
            let _ = tauri::async_runtime::block_on(check_website(handle.clone()));
        }
        std::thread::sleep(WAKE_EVERY);
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn automatic_checks_run_once_a_day() {
        let day = CHECK_EVERY_SECS;
        let now = 1_800_000_000;
        assert!(check_due(None, now), "never checked: check now");
        assert!(!check_due(Some(now - 60), now), "checked a minute ago");
        assert!(!check_due(Some(now - day + 1), now), "just under a day");
        assert!(check_due(Some(now - day), now), "a full day has passed");
        assert!(check_due(Some(now + 3600), now), "clock set back before the last check");
        let status: Status = serde_json::from_str(r#"{"announcements":[],"dismissed":{}}"#).unwrap();
        assert_eq!(status.last_checked, None, "files from older versions load and check right away");
    }
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
    #[test]
    fn localhost_preview_keeps_immutable_downloads_on_the_connected_website() {
        let hash = "a".repeat(64);
        let mut resource = library::Resource {
            url: format!("https://wallpaper.xiangban.xyz/uploads/desktop-{hash}.mp4"),
            sha256: hash,
            bytes: 10,
            mime: "video/mp4".into(),
        };
        let original = resource.url.clone();
        site_resource(&mut resource, "https://other.example").unwrap();
        assert!(resource.url.starts_with("https://other.example/uploads/"));
        resource.url = original;
        site_resource(&mut resource, "http://127.0.0.1:8087").unwrap();
        assert!(resource
            .url
            .starts_with("http://127.0.0.1:8087/uploads/desktop-"));
        resource.url = "https://cdn.example/video.mp4".into();
        site_resource(&mut resource, "http://127.0.0.1:8087").unwrap();
        assert_eq!(resource.url, "https://cdn.example/video.mp4");
    }

    use crate::library::test_http::{serve, Reply};
    use std::cell::{Cell, RefCell};

    fn png(tag: &str) -> Vec<u8> {
        [b"\x89PNG\r\n\x1a\n".as_slice(), tag.as_bytes()].concat()
    }
    fn hex(data: &[u8]) -> String {
        format!("{:x}", Sha256::digest(data))
    }
    fn copy_resource(data: &[u8]) -> Value {
        serde_json::json!({"url":"https://wallpaper.example/uploads/copy.png","sha256":hex(data),
            "bytes":data.len(),"mime":"image/png"})
    }
    fn item(kind: &str, resource: Option<Value>, source: Option<&str>) -> Value {
        let mut value = serde_json::json!({"id":"site-one","version":"1.0.0","title":"测试","kind":kind});
        if let Some(resource) = resource {
            value["resource"] = resource;
        }
        if let Some(url) = source {
            value["source"] = serde_json::json!({"url":url,"mime":"image/jpeg"});
        }
        value
    }
    fn manifest(items: Vec<Value>) -> String {
        serde_json::json!({"schemaVersion":1,"revision":"r1","items":items}).to_string()
    }
    fn plain_manifest(data: &[u8]) -> String {
        manifest(vec![item("image", Some(copy_resource(data)), None)])
    }
    fn unexpected_source(_: &str, _: &str, _: &str) -> Result<library::Resource, String> {
        panic!("source must not be fetched")
    }
    fn no_saved(_: &str, _: &str) -> Option<library::Resource> {
        None
    }
    fn no_plain() -> Result<String, String> {
        panic!("plain endpoint must not be requested")
    }

    #[test]
    fn manifest_without_source_takes_the_unchanged_verified_copy_path() {
        let copy = png("ours");
        assert!(parse_site_manifest(&plain_manifest(&copy)).is_none());
        // A null source also counts as "no source".
        let mut with_null = item("image", Some(copy_resource(&copy)), None);
        with_null["source"] = Value::Null;
        assert!(parse_site_manifest(&manifest(vec![with_null])).is_none());
        // Malformed input is left to the plain parser, which reports the same errors as before.
        for text in ["not json", "[]", r#"{"schemaVersion":2,"items":[{"source":{}}]}"#] {
            assert!(parse_site_manifest(text).is_none());
        }
        assert!(library::parse_catalog(&plain_manifest(&copy)).is_ok());
    }

    #[test]
    fn working_source_is_used_without_hash_check_or_our_copy() {
        let provider = png("provider");
        let server = serve(vec![("/p.png", Reply::Body(200, "text/html", provider.clone()))]);
        let root = library::test_http::temp_root("site-ok");
        let url = format!("{}/p.png", server.base);
        // The manifest hash describes our copy and differs from the provider file.
        let copy = png("ours-is-different");
        let text = manifest(vec![item("image", Some(copy_resource(&copy)), Some(&url))]);
        let resolved = resolve_manifest(
            parse_site_manifest(&text).unwrap(),
            no_saved,
            |_, kind, url| library::download_source(&root, url, kind, |_, _| {}),
            no_plain,
        )
        .unwrap();
        let resource = &resolved.catalog.items[0].resource;
        assert_eq!(resource.sha256, hex(&provider));
        assert_eq!(resource.url, url);
        assert!(resolved.local.contains("site-one"));
        assert!(library::cached(&root, resource));
        // No file is stored for our copy's hash: it was never fetched.
        assert!(!root.join("cache").join(hex(&copy)).exists());
        assert_eq!(server.hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_cancelled_source_download_does_not_fall_back_to_the_copy() {
        let copy = png("ours");
        let text = manifest(vec![item("image", Some(copy_resource(&copy)), Some("https://cdn.example.test/a.png"))]);
        let plain_called = std::cell::Cell::new(false);
        let result = resolve_manifest(
            parse_site_manifest(&text).unwrap(),
            no_saved,
            |_, _, _| Err(library::CANCELLED.to_string()),
            || {
                plain_called.set(true);
                no_plain()
            },
        );
        assert_eq!(result.err().as_deref(), Some(library::CANCELLED));
        assert!(!plain_called.get());
    }

    #[test]
    fn the_download_guard_is_cleared_only_by_its_own_download() {
        let state = WebsiteState::default();
        let first = DownloadGuard::start(&state);
        let first_flag = first.cancel.clone();
        let second = DownloadGuard::start(&state);
        assert!(first_flag.load(Ordering::Relaxed), "a newer download stops the older one");
        drop(first);
        assert!(state.download.lock().unwrap_or_else(|e| e.into_inner()).is_some(), "the older guard leaves the newer flag in place");
        state.download.lock().unwrap_or_else(|e| e.into_inner()).as_ref().unwrap().store(true, Ordering::Relaxed);
        assert_eq!(second.check(), Err(library::CANCELLED.to_string()));
        drop(second);
        assert!(state.download.lock().unwrap_or_else(|e| e.into_inner()).is_none());
    }

    #[test]
    fn broken_sources_fall_back_to_the_copy_in_the_manifest() {
        let copy = png("ours");
        let server = serve(vec![
            ("/html", Reply::Body(200, "image/png", b"<html>hotlink denied</html>".to_vec())),
            ("/gone", Reply::Body(404, "text/plain", b"x".to_vec())),
            ("/drop", Reply::Drop),
            ("/jpg-as-video", Reply::Body(200, "video/mp4", b"\xff\xd8\xff\xe0jpeg".to_vec())),
        ]);
        for path in ["html", "gone", "drop", "jpg-as-video"] {
            let root = library::test_http::temp_root("site-fallback");
            let url = format!("{}/{path}", server.base);
            let kind = if path == "jpg-as-video" { "video" } else { "image" };
            let mut entry = item(kind, Some(copy_resource(&copy)), Some(&url));
            if kind == "video" {
                entry["resource"]["mime"] = "video/mp4".into();
            }
            let resolved = resolve_manifest(
                parse_site_manifest(&manifest(vec![entry])).unwrap(),
                no_saved,
                |_, kind, url| library::download_source(&root, url, kind, |_, _| {}),
                no_plain,
            )
            .unwrap();
            let resource = &resolved.catalog.items[0].resource;
            // Our copy is used and still needs the usual sha256/bytes verification.
            assert_eq!(resource.url, "https://wallpaper.example/uploads/copy.png", "{path}");
            assert_eq!(resource.sha256, hex(&copy));
            assert!(resolved.local.is_empty());
            assert!(!root.join("cache").exists() || std::fs::read_dir(root.join("cache")).unwrap().next().is_none());
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn without_a_copy_a_failed_source_requests_the_plain_endpoint_once() {
        let copy = png("ours");
        let plain_calls = Cell::new(0);
        let text = manifest(vec![item("image", None, Some("https://cdn.example/denied.png"))]);
        let resolved = resolve_manifest(
            parse_site_manifest(&text).unwrap(),
            no_saved,
            |_, _, _| Err("403".to_string()),
            || {
                plain_calls.set(plain_calls.get() + 1);
                Ok(plain_manifest(&copy))
            },
        )
        .unwrap();
        assert_eq!(plain_calls.get(), 1);
        assert_eq!(resolved.catalog.items[0].resource.sha256, hex(&copy));
        assert!(resolved.local.is_empty());
        // The plain endpoint failing, or not knowing the wallpaper, is an error.
        let failing = resolve_manifest(
            parse_site_manifest(&text).unwrap(),
            no_saved,
            |_, _, _| Err("403".to_string()),
            || Err("网络连接失败".to_string()),
        );
        assert!(failing.is_err());
        let other = manifest(vec![{
            let mut value = item("image", Some(copy_resource(&copy)), None);
            value["id"] = "site-other".into();
            value
        }]);
        assert!(resolve_manifest(
            parse_site_manifest(&text).unwrap(),
            no_saved,
            |_, _, _| Err("403".to_string()),
            || Ok(other.clone()),
        )
        .is_err());
    }

    #[test]
    fn saved_wallpaper_is_reused_and_a_missing_source_url_falls_back() {
        let copy = png("ours");
        let saved = library::Resource {
            url: "https://cdn.example/saved.png".into(),
            sha256: hex(b"saved"),
            bytes: 5,
            mime: "image/png".into(),
        };
        let text = manifest(vec![item("image", Some(copy_resource(&copy)), Some("https://cdn.example/a.png"))]);
        let seen = RefCell::new(Vec::new());
        let resolved = resolve_manifest(
            parse_site_manifest(&text).unwrap(),
            |id, version| {
                seen.borrow_mut().push((id.to_string(), version.to_string()));
                Some(saved.clone())
            },
            unexpected_source,
            no_plain,
        )
        .unwrap();
        assert_eq!(seen.borrow().as_slice(), [("site-one".to_string(), "1.0.0".to_string())]);
        assert_eq!(resolved.catalog.items[0].resource.sha256, saved.sha256);
        assert!(resolved.local.contains("site-one"));
        let mut broken = item("image", Some(copy_resource(&copy)), None);
        broken["source"] = serde_json::json!({"mime":"image/png"});
        let resolved = resolve_manifest(
            parse_site_manifest(&manifest(vec![broken])).unwrap(),
            no_saved,
            unexpected_source,
            no_plain,
        )
        .unwrap();
        assert_eq!(resolved.catalog.items[0].resource.sha256, hex(&copy));
    }

    #[test]
    fn saved_resource_requires_same_version_and_intact_file() {
        let root = library::test_http::temp_root("site-saved");
        let data = png("saved");
        std::fs::create_dir_all(root.join("cache")).unwrap();
        std::fs::write(root.join("cache").join(hex(&data)), &data).unwrap();
        let catalog = library::parse_catalog(&manifest(vec![item(
            "image",
            Some(copy_resource(&data)),
            None,
        )]))
        .unwrap();
        library::save_json(&root.join("catalog.json"), &catalog).unwrap();
        assert!(saved_resource(&root, "site-one", "1.0.0").is_some());
        assert!(saved_resource(&root, "site-one", "1.0.1").is_none());
        assert!(saved_resource(&root, "site-two", "1.0.0").is_none());
        std::fs::write(root.join("cache").join(hex(&data)), png("tampered")[..data.len()].to_vec()).unwrap();
        assert!(saved_resource(&root, "site-one", "1.0.0").is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}

pub async fn smoke(app: &tauri::AppHandle) -> Result<Value, String> {
    if !app.state::<DesktopState>().isolated {
        return Err("网站联调只能在隔离测试中运行".into());
    }
    let base = std::env::var("XIAOBAN_WEBSITE_TEST_URL").map_err(|_| "没有网站测试服务")?;
    use_test_website(app.clone(), base).await?;
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
        let previous_wallpaper = settings(app).wallpaper_id;
        let selected = download_site_wallpaper(app.clone(), id).await?;
        if settings(app).wallpaper_id != previous_wallpaper {
            return Err("保存到我的壁纸不应更换桌面".into());
        }
        if !selected["library"]["items"]
            .as_array()
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item["definition"]["id"] == selected["id"] && item["installed"] == true
                })
            })
        {
            return Err("保存后素材应已缓存，不应再次手动下载".into());
        }
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

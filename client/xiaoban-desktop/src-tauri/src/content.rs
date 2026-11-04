use crate::{
    library::{self, Options, Wallpaper},
    settings,
    widget_geometry::{self, MonitorArea},
    DesktopState,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Default)]
pub struct ContentState {
    pub operation: tauri::async_runtime::Mutex<()>,
    active: Mutex<Option<Wallpaper>>,
    windows: Mutex<Vec<WallpaperWindow>>,
    pub reports: Mutex<BTreeMap<String, Value>>,
    pub io: Mutex<()>,
}
#[derive(Clone)]
struct WallpaperWindow {
    label: String,
    area: MonitorArea,
}

fn active_windows(app: &tauri::AppHandle) -> Vec<WallpaperWindow> {
    app.state::<ContentState>().windows.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::storage::root(app)?.join("wallpapers"))
}
pub fn permitted(label: &str, command: &str) -> bool {
    if ["export_resource_pack", "import_resource_pack", "get_wallpaper_displays"].contains(&command) { return label == "control"; }
    if command.starts_with("tts_") { return label == "control"; }
    if command.contains("account") || command.starts_with("chat_") || command.starts_with("community_") || command.starts_with("phone_link_") || command.starts_with("speech_agent_") || command.starts_with("voice_service_") { return label == "control"; }
    if command.contains("website") || command.starts_with("fetch_site_") || ["select_site_wallpaper","download_site_wallpaper","cancel_site_download"].contains(&command) { return label == "control"; }
    if ["import_local_wallpaper", "delete_local_wallpaper", "open_data_directory", "set_automatic_updates", "open_official_site", "refresh_ai_providers", "get_ai_config", "save_ai_config", "pick_workspace", "fetch_ai_models", "save_ai_provider", "switch_ai_provider", "delete_ai_provider", "save_ai_preferences", "request_system_permissions"].contains(&command) { return label == "control"; }
    if ["control", "pet", "wallpaper"].contains(&label) {
        return true;
    }
    (media_label(label) || crate::effects::is_effect(label)) && command == "scene_ready"
}

pub fn is_wallpaper(label: &str) -> bool {
    ["media-a", "media-b"].iter().any(|prefix| {
        label == *prefix || label.strip_prefix(*prefix).and_then(|rest| rest.strip_prefix('-'))
            .is_some_and(|index| !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()))
    })
}

pub fn media_label(label: &str) -> bool {
    label == "media-preview" || is_wallpaper(label)
}

pub fn labels(app: &tauri::AppHandle) -> Vec<String> {
    app.webview_windows().into_keys().filter(|label| is_wallpaper(label)).collect()
}

pub fn is_active_window(app: &tauri::AppHandle, label: &str) -> bool {
    active_windows(app).iter().any(|window| window.label == label)
}

pub fn composing_on(app: &tauri::AppHandle, area: &MonitorArea) -> bool {
    cfg!(windows) && active_windows(app).iter().any(|window| {
        window.area.id == area.id && app.get_webview_window(&window.label).is_some()
    })
}

pub fn companion_window(app: &tauri::AppHandle) -> Option<tauri::WebviewWindow> {
    if !cfg!(windows) { return None; }
    active_windows(app).into_iter().find(|window| window.area.primary)
        .and_then(|window| app.get_webview_window(&window.label))
}

#[tauri::command]
pub fn get_wallpaper_displays(app: tauri::AppHandle) -> Result<Vec<MonitorArea>, String> {
    crate::widgets::monitors(&app)
}

fn composition(app: &tauri::AppHandle, area: Option<&MonitorArea>) -> Value {
    let config = settings(app);
    let embedded = area.filter(|_| cfg!(windows));
    let overlay = embedded.is_some_and(|m| m.primary) && config.mode == "wallpaper" && config.companion_enabled;
    let position = embedded.filter(|_| overlay).and_then(|m| {
        let p = app.get_webview_window("pet")?.outer_position().ok()?;
        Some(json!({"x":(f64::from(p.x)-m.x)/m.scale,"y":(f64::from(p.y)-m.y)/m.scale}))
    });
    let effects = embedded.map_or(Value::Null, |m| crate::effects::render_payload(app, m));
    let mut value = json!(config);
    value["overlay"] = json!(overlay);
    json!({"config":value,"effects":effects,"petPosition":position})
}

fn payload(app: &tauri::AppHandle, item: &Wallpaper, options: &Options, area: Option<&MonitorArea>) -> Value {
    let composition = composition(app, area);
    json!({"item":{"id":item.id,"title":item.title,"kind":item.kind,"resourceUrl":library::asset_url(&item.resource),
        "assets":item.assets.iter().map(|(key,r)|(key.clone(),json!({"url":library::asset_url(r),"mime":r.mime}))).collect::<BTreeMap<_,_>>()},
        "options":render_options(app, &item.id, options),"config":composition["config"],"effects":composition["effects"],"petPosition":composition["petPosition"],
        "test":app.state::<DesktopState>().isolated})
}

fn render_options(app: &tauri::AppHandle, id: &str, options: &Options) -> Value {
    let mut value = json!(options);
    if id == "builtin-wish-pond" {
        // Only positions cross into the wallpaper sandbox; the user's wish text stays in the trusted scene.
        let positions = crate::rituals::read_ritual_journal(app.clone()).ok()
            .and_then(|data| data["wishes"].as_array().map(|wishes| wishes.iter().map(|wish| json!({"x": wish["x"], "y": wish["y"]})).collect::<Vec<_>>()))
            .unwrap_or_default();
        value["values"]["_wishPositions"] = json!(serde_json::to_string(&positions).unwrap_or_default());
    }
    value
}

fn open(
    app: &tauri::AppHandle,
    item: &Wallpaper,
    options: &Options,
    label: &str,
    area: Option<&MonitorArea>,
) -> Result<(), String> {
    let state = app.state::<ContentState>();
    let preview = area.is_none();
    if let Some(old) = app.get_webview_window(label) {
        old.destroy().map_err(|e| e.to_string())?;
        // Destruction is dispatched to the UI thread; wait until Tauri removes the label before
        // reusing the preview window for the next online wallpaper.
        let deadline = Instant::now() + Duration::from_secs(3);
        while app.get_webview_window(label).is_some() {
            if Instant::now() >= deadline { return Err("上一个壁纸窗口仍在关闭，请稍后重试。".into()); }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    app.state::<DesktopState>()
        .ready_scenes
        .lock()
        .unwrap()
        .remove(label);
    state.reports.lock().unwrap_or_else(|e| e.into_inner()).remove(label);
    #[cfg(windows)]
    let parent = area.map(|_| crate::windows::desktop_parent()).transpose()?;
    let mut builder = WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::App("index.html?scene=content".into()),
    )
    .title(if preview {
        "小伴 · 壁纸预览"
    } else {
        "小伴 · 动态壁纸"
    })
    .decorations(preview)
    // Undecorated Windows shadows reserve a non-client frame even as a
    // WorkerW child, shrinking and offsetting the wallpaper's WebView.
    .shadow(preview)
    .resizable(preview)
    .skip_taskbar(!preview)
    .visible(false)
    .focused(preview)
    .initialization_script(format!(
        "window.__XIAOBAN_CONTENT__={};",
        payload(app, item, options, area)
    ));
    if let Some(area) = area {
        builder = builder.inner_size(area.width / area.scale, area.height / area.scale)
            .position(area.x / area.scale, area.y / area.scale);
    } else {
        builder = builder.inner_size(960.0, 600.0).center();
    }
    #[cfg(target_os = "macos")]
    let builder =
        builder.incognito(true).background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled);
    #[cfg(windows)]
    let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
    #[cfg(windows)]
    let builder = if let Some(parent) = parent {
        builder.parent_raw(::windows::Win32::Foundation::HWND(parent as _))
    } else {
        builder
    };
    let window = builder.build().map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        #[cfg(target_os = "macos")]
        crate::macos::configure_scene(&window, !preview, preview)?;
        if let Some(area) = area {
            #[cfg(windows)]
            crate::windows::attach_wallpaper_area(&window, area, parent.unwrap())?;
            #[cfg(target_os = "macos")]
            crate::macos::place_widget(&window, area.x, area.y, area.width, area.height)?;
            window.set_ignore_cursor_events(true).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() { let _ = window.destroy(); }
    result
}

fn destroy(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) { let _ = window.destroy(); }
    app.state::<DesktopState>().ready_scenes.lock().unwrap_or_else(|e| e.into_inner()).remove(label);
    app.state::<ContentState>().reports.lock().unwrap_or_else(|e| e.into_inner()).remove(label);
}

fn open_wallpapers(
    app: &tauri::AppHandle,
    item: &Wallpaper,
    options: &Options,
    areas: &[&MonitorArea],
) -> Result<Vec<WallpaperWindow>, String> {
    if areas.is_empty() { return Err("没有找到显示器".into()); }
    // Alternate labels so a replacement can finish loading while the old group is visible.
    let bank = if active_windows(app).first().is_some_and(|window| window.label == "media-a") {
        "media-b"
    } else {
        "media-a"
    };
    let mut windows: Vec<WallpaperWindow> = Vec::new();
    for (index, area) in areas.iter().enumerate() {
        let label = if index == 0 { bank.into() } else { format!("{bank}-{index}") };
        if let Err(error) = open(app, item, options, &label, Some(area)) {
            for window in &windows { destroy(app, &window.label); }
            return Err(error);
        }
        windows.push(WallpaperWindow { label, area: (**area).clone() });
    }
    Ok(windows)
}

fn wait(app: &tauri::AppHandle, labels: &[String]) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(25);
    let state = app.state::<ContentState>();
    let desktop = app.state::<DesktopState>();
    while Instant::now() < deadline {
        {
            let reports = state.reports.lock().unwrap_or_else(|e| e.into_inner());
            for label in labels {
                if let Some(error) = reports.get(label).and_then(|report| report["error"].as_str()) {
                    return Err(error.into());
                }
            }
        }
        let ready = desktop.ready_scenes.lock().unwrap_or_else(|e| e.into_inner());
        if labels.iter().all(|label| ready.contains(label)) { return Ok(()); }
        drop(ready);
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("壁纸未在 25 秒内完成加载，已保留之前的壁纸。".into())
}

async fn wait_until_ready(app: &tauri::AppHandle, labels: Vec<String>) -> Result<(), String> {
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || wait(&handle, &labels)).await.map_err(|e| e.to_string())?
}

fn show(app: &tauri::AppHandle, windows: &[WallpaperWindow]) -> Result<(), String> {
    for entry in windows {
        let window = app.get_webview_window(&entry.label).ok_or("壁纸窗口已关闭")?;
        window.show().map_err(|e| e.to_string())?;
    }
    #[cfg(windows)]
    if !windows.is_empty() { crate::windows::order_desktop_layers(app)?; }
    Ok(())
}

fn replace_windows(app: &tauri::AppHandle, windows: Vec<WallpaperWindow>) {
    let state = app.state::<ContentState>();
    let old = std::mem::replace(&mut *state.windows.lock().unwrap_or_else(|e| e.into_inner()), windows);
    for window in old { destroy(app, &window.label); }
}

pub fn restore(app: &tauri::AppHandle) -> Result<bool, String> {
    if crate::games::is_active(app) { return Ok(false); }
    let config = settings(app);
    if config.wallpaper_id == "companion" {
        return Ok(false);
    }
    let data = fs::read(root(app)?.join("active.json"))
        .map_err(|_| "已选壁纸资源不可用，请在动态壁纸页重新应用。")?;
    let item: Wallpaper = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    library::validate_stored(&item)?;
    if item.id != config.wallpaper_id
        || !library::assets(&item)
            .iter()
            .all(|r| library::cached(&root(app).unwrap(), r))
    {
        return Err("壁纸缓存不完整，请重新应用壁纸。".into());
    }
    let options = library::validate_options(&item, config.wallpaper_options)?;
    let displays = crate::widgets::monitors(app)?;
    let areas = widget_geometry::targets(&displays, &config.wallpaper_monitor);
    let windows = open_wallpapers(app, &item, &options, &areas)?;
    *app.state::<ContentState>().active.lock().unwrap_or_else(|e| e.into_inner()) = Some(item);
    replace_windows(app, windows);
    // During game restore the UI thread can acknowledge a window before the group is installed.
    let ready = app.state::<DesktopState>().ready_scenes.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let loaded: Vec<_> = active_windows(app).into_iter().filter(|window| ready.contains(&window.label)).collect();
    if let Err(error) = show(app, &loaded) {
        close_active(app);
        return Err(error);
    }
    Ok(true)
}

pub async fn displays_changed(app: tauri::AppHandle) {
    let state = app.state::<ContentState>();
    let _operation = state.operation.lock().await;
    if crate::games::is_active(&app) { return; }
    let result = async {
        let config = settings(&app);
        let item = state.active.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let Some(item) = item.filter(|item| item.id == config.wallpaper_id) else { return Ok(()); };
        let displays = crate::widgets::monitors(&app)?;
        let areas = widget_geometry::targets(&displays, &config.wallpaper_monitor);
        let current = active_windows(&app);
        if current.iter().map(|window| &window.area).eq(areas.iter().copied()) { return Ok(()); }
        let windows = open_wallpapers(&app, &item, &config.wallpaper_options, &areas)?;
        let labels = windows.iter().map(|window| window.label.clone()).collect();
        if let Err(error) = wait_until_ready(&app, labels).await.and_then(|_| show(&app, &windows)) {
            for window in windows { destroy(&app, &window.label); }
            return Err(error);
        }
        replace_windows(&app, windows);
        Ok::<(), String>(())
    }.await;
    if let Err(error) = result { let _ = app.emit("desktop-error", error); }
    sync(&app);
    crate::effects::sync(&app);
}

pub fn sync(app: &tauri::AppHandle) {
    if crate::games::is_active(app) { return; }
    let config = settings(app);
    for entry in active_windows(app) {
        if let Some(window) = app.get_webview_window(&entry.label) {
            let mut value = composition(app, Some(&entry.area));
            value["options"] = render_options(app, &config.wallpaper_id, &config.wallpaper_options);
            let _ = window.eval(format!(
                "window.dispatchEvent(new CustomEvent('content-config',{{detail:{value}}}));"
            ));
        }
    }
    #[cfg(windows)]
    if let Some(pet) = app.get_webview_window("pet") {
        if companion_window(app).is_some() && config.mode == "wallpaper" { let _ = pet.hide(); }
        else if config.companion_enabled {
            let was_visible = pet.is_visible().unwrap_or(false);
            let _ = pet.show();
            if !was_visible && config.mode == "wallpaper" {
                let _ = crate::windows::configure_companion(&pet, true);
            }
        }
    }
}
pub fn send_cursor(window: &tauri::WebviewWindow, sample: &impl serde::Serialize) {
    if let Ok(value) = serde_json::to_string(sample) {
        let _ = window.eval(format!(
            "window.dispatchEvent(new CustomEvent('content-cursor',{{detail:{value}}}));"
        ));
    }
}
pub fn close_active(app: &tauri::AppHandle) {
    replace_windows(app, Vec::new());
    *app.state::<ContentState>().active.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

#[tauri::command]
pub fn get_wallpaper_library(app: tauri::AppHandle) -> Result<Value, String> {
    let root = root(&app)?;
    let config = settings(&app);
    let items = library::all_items(&root)
        .into_iter()
        .map(|item| {
            let installed = library::assets(&item)
                .iter()
                .all(|r| library::cached(&root, r));
            let local = crate::local_media::is_local(&item);
            let cover = item
                .cover
                .as_ref()
                .filter(|r| library::cached(&root, r))
                .map(library::asset_url)
                .or_else(|| (item.kind == "image" && installed).then(|| library::asset_url(&item.resource)));
            let source = if local { "local" } else if item.id.starts_with("builtin-") { "builtin" } else { "subscription" };
            let preview = (installed && item.kind != "canvas").then(|| library::asset_url(&item.resource));
            json!({"definition":item,"installed":installed,"coverUrl":cover,"previewUrl":preview,"source":source})
        })
        .collect::<Vec<_>>();
    Ok(json!({"items":items,"source":config.catalog_url,"activeId":config.wallpaper_id}))
}

#[tauri::command]
pub async fn import_wallpaper_catalog(
    app: tauri::AppHandle,
    text: String,
) -> Result<Value, String> {
    let catalog = library::parse_catalog(&text)?;
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<ContentState>();
        let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
        let root = root(&handle)?;
        let mut previous = library::read_catalog(&root);
        for item in catalog.items {
            previous.items.retain(|old| old.id != item.id);
            if let Some(cover) = &item.cover {
                let _ = library::download(&root, cover, |_| {});
            }
            previous.items.push(item);
        }
        previous.schema_version = 1;
        previous.revision = catalog.revision;
        library::save_json(&root.join("catalog.json"), &previous)?;
        get_wallpaper_library(handle.clone())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_local_wallpaper(
    app: tauri::AppHandle,
    id: String,
) -> Result<Value, String> {
    let hash = {
        let root_path = root(&app)?;
        let state = app.state::<ContentState>();
        let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
        crate::local_media::delete_local_wallpaper(&root_path, &id)?
    };

    // If currently active, reset to companion
    let current_id = settings(&app).wallpaper_id;
    if current_id == id {
        let _ = apply_wallpaper(app.clone(), "companion".into(), Default::default(), false, None).await;
    }

    let library = get_wallpaper_library(app.clone())?;
    Ok(json!({ "ok": true, "id": id, "hash": hash, "library": library }))
}

#[tauri::command]
pub async fn subscribe_wallpapers(app: tauri::AppHandle, url: String) -> Result<Value, String> {
    library::network_url(&url)?;
    let request = url.clone();
    let text = tauri::async_runtime::spawn_blocking(move || library::fetch_json(&request))
        .await
        .map_err(|e| e.to_string())??
        .ok_or("订阅服务没有返回清单")?;
    let result = import_wallpaper_catalog(app.clone(), text).await?;
    let mut config = settings(&app);
    config.catalog_url = url;
    crate::persist(&app, &config)?;
    *app.state::<DesktopState>().settings.lock().unwrap_or_else(|e| e.into_inner()) = config;
    crate::publish(&app);
    Ok(result)
}

#[tauri::command]
pub async fn apply_wallpaper(
    app: tauri::AppHandle,
    id: String,
    options: Options,
    preview: bool,
    monitor: Option<String>,
) -> Result<Value, String> {
    let operation = app.state::<ContentState>();
    let _selection = operation.operation.lock().await;
    if crate::games::is_active(&app) {
        return Err("请先退出游戏空间，再预览或应用壁纸。".into());
    }
    if id == "companion" {
        if preview {
            return Err("系统桌面无需预览。".into());
        }
        {
            let state = app.state::<DesktopState>();
            let _update = state.update_lock.lock().unwrap_or_else(|e| e.into_inner());
            let mut config = settings(&app);
            config.wallpaper_id = id;
            crate::persist(&app, &config)?;
            *state.settings.lock().unwrap_or_else(|e| e.into_inner()) = config;
        }
        close_active(&app);
        crate::publish(&app);
        return Ok(json!(settings(&app)));
    }
    let directory = root(&app)?;
    let item = library::all_items(&directory)
        .into_iter()
        .find(|item| item.id == id)
        .ok_or("没有找到这张壁纸")?;
    library::validate_stored(&item)?;
    let options = library::validate_options(&item, options)?;
    let handle = app.clone();
    let prepared = item.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<ContentState>();
        let _io = state.io.lock().unwrap_or_else(|e| e.into_inner());
        library::prepare(&handle, &directory, &prepared)
    })
    .await
    .map_err(|e| e.to_string())??;
    if crate::games::is_active(&app) {
        return Err("请先退出游戏空间，再预览或应用壁纸。".into());
    }
    if preview {
        let label = "media-preview";
        open(&app, &item, &options, label, None)?;
        if let Err(error) = wait_until_ready(&app, vec![label.into()]).await {
            destroy(&app, label);
            return Err(error);
        }
        return Ok(json!({"preview":true}));
    }
    let selection = monitor.unwrap_or_else(|| settings(&app).wallpaper_monitor);
    if selection.len() > 512 { return Err("显示器标识无效".into()); }
    let displays = crate::widgets::monitors(&app)?;
    let areas = widget_geometry::targets(&displays, &selection);
    let active_path = root(&app)?.join("active.json");
    let windows = open_wallpapers(&app, &item, &options, &areas)?;
    let labels = windows.iter().map(|window| window.label.clone()).collect();
    let ready = wait_until_ready(&app, labels).await;
    let commit = || -> Result<(), String> {
        ready?;
        show(&app, &windows)?;
        let state = app.state::<DesktopState>();
        let _update = state.update_lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut config = settings(&app);
        config.wallpaper_id = id;
        config.wallpaper_monitor = selection;
        config.wallpaper_options = options;
        let previous = operation.active.lock().unwrap_or_else(|e| e.into_inner()).clone();
        library::save_json(&active_path, &item)?;
        if let Err(error) = crate::persist(&app, &config) {
            if let Some(previous) = previous {
                let _ = library::save_json(&active_path, &previous);
            } else {
                let _ = fs::remove_file(&active_path);
            }
            return Err(error);
        }
        *operation.active.lock().unwrap_or_else(|e| e.into_inner()) = Some(item);
        *state.settings.lock().unwrap_or_else(|e| e.into_inner()) = config;
        Ok(())
    };
    if let Err(error) = commit() {
        for window in windows { destroy(&app, &window.label); }
        return Err(error);
    }
    replace_windows(&app, windows);
    if let Some(window) = app.get_webview_window("wallpaper") {
        let _ = window.destroy();
    }
    crate::publish(&app);
    Ok(json!(settings(&app)))
}

#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle) -> Value {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "windows"
    };
    let mut info = json!({"name":"小伴桌面","version":app.package_info().version.to_string(),"platform":std::env::consts::OS,
        "arch":std::env::consts::ARCH,"updateTarget":format!("{os}-{}",std::env::consts::ARCH),"updateUrl":settings(&app).update_url,
        "dataDirectory":crate::storage::root(&app).ok()});
    if crate::ui_smoke_enabled(&app) { info["uiSmoke"] = json!(true); }
    info
}
/// Points update checks at a test service. Only the isolated smoke test uses this; loading the
/// settings resets the address to the built-in one.
fn set_update_source(app: tauri::AppHandle, url: String) -> Result<crate::Settings, String> {
    if !app.state::<DesktopState>().isolated {
        return Err("只有隔离测试可以更换更新服务".into());
    }
    if !url.is_empty() {
        library::network_url(&url)?;
    }
    let mut config = settings(&app);
    config.update_url = url;
    crate::persist(&app, &config)?;
    *app.state::<DesktopState>().settings.lock().unwrap_or_else(|e| e.into_inner()) = config;
    crate::publish(&app);
    Ok(settings(&app))
}
#[tauri::command]
pub async fn check_updates(app: tauri::AppHandle) -> Result<Value, String> {
    let config = settings(&app);
    if config.update_url.is_empty() {
        return Err("尚未配置更新服务".into());
    }
    let version = app.package_info().version.to_string();
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "windows"
    };
    let target = format!("{os}-{}", std::env::consts::ARCH);
    let mut url = library::network_url(&config.update_url)?;
    url.query_pairs_mut()
        .append_pair("currentVersion", &version)
        .append_pair("target", &target);
    tauri::async_runtime::spawn_blocking(move || match library::fetch_json(url.as_str())? {
        Some(text) => library::update_result(&text, &version, &target),
        None => Ok(json!({"available":false,"version":version,"notes":""})),
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn open_download_url(url: String) -> Result<(), String> {
    library::network_url(&url)?;
    #[cfg(windows)]
    unsafe {
        let operation: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
        let value: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();
        let result = windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            value.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
        if result as isize <= 32 {
            return Err("无法打开浏览器，请稍后重试".into());
        }
    }
    #[cfg(target_os = "macos")]
    {
        if !std::process::Command::new("/usr/bin/open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())?
            .success()
        {
            return Err("无法打开浏览器".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wallpaper_code_cannot_use_desktop_or_startup_commands() {
        for label in ["media-a", "media-b", "media-a-1", "media-b-2", "media-preview", "unknown"] {
            for command in [
                "set_autostart",
                "quit",
                "get_settings",
                "open_download_url",
                "import_wallpaper_catalog",
                "get_wallpaper_displays",
                "import_local_wallpaper",
                "export_resource_pack",
                "import_resource_pack",
                "open_data_directory",
                "set_automatic_updates",
                "open_official_site",
                "fetch_site_wallpapers",
                "check_website",
                "dismiss_website_message",
                "send_account_code",
                "login_account_with_code",
                "logout_account",
                "request_system_permissions",
            ] {
                assert!(!permitted(label, command));
            }
        }
        assert!(permitted("media-a", "scene_ready"));
        assert!(permitted("media-b-2", "scene_ready"));
        assert!(!permitted("media-b-extra", "scene_ready"));
        assert!(permitted("control", "set_autostart"));
    }
}

pub(crate) fn smoke_render_report(app: &tauri::AppHandle, id: &str) -> Result<Value, String> {
    let label = active_windows(app).into_iter().next().ok_or("没有背景窗口")?.label;
    let report = app.state::<ContentState>().reports.lock().unwrap_or_else(|e| e.into_inner()).get(&label).cloned().ok_or("没有渲染报告")?;
    if report.get("privilegedIpcBlocked") != Some(&Value::Bool(true)) { return Err("壁纸窗口没有隔离桌面接口".into()); }
    #[allow(unused_mut)]
    let mut case = json!({"id":id,"report":report});
    #[cfg(windows)]
    {
        let window = app.get_webview_window(&label).ok_or("没有背景窗口")?;
        let geometry = crate::windows::wallpaper_geometry(&window)?;
        if geometry.get("fillsMonitor") != Some(&Value::Bool(true)) {
            return Err(format!("壁纸实际绘图区没有铺满主屏幕：{id} {geometry}"));
        }
        case["geometry"] = geometry;
    }
    Ok(case)
}

pub async fn smoke(app: &tauri::AppHandle) -> Result<Value, String> {
    let url = std::env::var("XIAOBAN_WALLPAPER_TEST_URL").map_err(|_| "没有壁纸测试服务")?;
    if !app.state::<DesktopState>().isolated {
        return Err("只能在隔离测试中运行壁纸测试".into());
    }
    subscribe_wallpapers(app.clone(), url.clone()).await?;
    let mut cases = Vec::new();
    for id in [
        "builtin-soft-hills",
        "builtin-sea-breeze",
        "builtin-mint-orbits",
        "test-image",
        "test-video",
        "test-canvas",
    ] {
        apply_wallpaper(app.clone(), id.into(), Options::default(), false, None).await?;
        cases.push(smoke_render_report(app, id)?);
    }
    let previous = settings(app).wallpaper_id;
    let rejected = apply_wallpaper(
        app.clone(),
        "test-bad-hash".into(),
        Options::default(),
        false,
        None,
    )
    .await
    .is_err();
    let preserved = rejected && settings(app).wallpaper_id == previous;
    if !preserved {
        return Err("完整性失败后没有保留旧壁纸".into());
    }
    let base = url.trim_end_matches("/catalog.json").to_string();
    set_update_source(app.clone(), format!("{base}/updates.json"))?;
    if check_updates(app.clone()).await?.get("available") != Some(&Value::Bool(true)) {
        return Err("更新检查没有识别服务新版本".into());
    }
    let offline = format!("{base}/offline");
    tauri::async_runtime::spawn_blocking(move || library::fetch_json(&offline))
        .await
        .map_err(|e| e.to_string())??;
    apply_wallpaper(app.clone(), "test-canvas".into(), Options::default(), false, None).await?;
    let local_directory = PathBuf::from(std::env::var_os("XIAOBAN_LOCAL_WALLPAPER_TEST_DIR").ok_or("缺少本地壁纸测试目录")?);
    let mut local_cases = Vec::new();
    for filename in ["本地图 image.png", "本地视频.mp4"] {
        let source = local_directory.join(filename);
        let item = crate::local_media::import_file(&root(app)?, &source)?;
        let duplicate = crate::local_media::import_file(&root(app)?, &source)?;
        if item.id != duplicate.id { return Err("重复导入没有去重".into()); }
        fs::remove_file(&source).map_err(|e| e.to_string())?;
        apply_wallpaper(app.clone(), item.id.clone(), Options::default(), false, None).await?;
        local_cases.push(smoke_render_report(app, &item.id)?);
    }
    if crate::local_media::read(&root(app)?)?.items.len() != 2 { return Err("本地导入重复保存了元数据".into()); }
    close_active(app);
    if !restore(app)? { return Err("无法恢复本地壁纸".into()); }
    let restored_label = active_windows(app).into_iter().next().ok_or("没有恢复窗口")?.label;
    let handle = app.clone();
    wait_until_ready(&handle, vec![restored_label]).await?;
    smoke_render_report(app, &settings(app).wallpaper_id)?;
    let effects = if std::env::var("XIAOBAN_EFFECTS_SMOKE").ok().as_deref() == Some("1") {
        Some(crate::effects_smoke::run(app).await?)
    } else { None };
    apply_wallpaper(app.clone(), "companion".into(), Options::default(), false, None).await?;
    Ok(
        json!({"cases":cases,"cacheOffline":true,"hashFailurePreserved":preserved,"updateChecked":true,
            "localImports":{"cases":local_cases,"sourceDeleted":true,"deduplicated":true,"restored":true},"effects":effects}),
    )
}


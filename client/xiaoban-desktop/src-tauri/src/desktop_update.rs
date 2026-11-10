//! The official signed feed is independent of the configurable wallpaper website.
use serde::Serialize;
use serde_json::{json, Value};
use std::{sync::Mutex, time::Duration};
use tauri::Manager;
use tauri_plugin_updater::{Error, Update, UpdaterExt};

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    pub busy: bool,
    pub percent: Option<u8>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct State(Mutex<Installation>);

pub fn status(app: &tauri::AppHandle) -> Installation {
    app.state::<State>().0.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn publish(app: &tauri::AppHandle, status: Installation) {
    *app.state::<State>().0.lock().unwrap_or_else(|e| e.into_inner()) = status;
    crate::website::publish_status(app);
}

async fn find(app: &tauri::AppHandle) -> Result<Option<(Update, String)>, String> {
    if app.state::<crate::DesktopState>().isolated || !app.config().plugins.0.contains_key("updater") {
        return Ok(None);
    }

    let network = crate::network::current();
    let builder = app.updater_builder()
        .timeout(Duration::from_secs(12))
        .configure_client(|client| client.https_only(true));
    let builder = match network.mode.as_str() {
        "direct" => builder.no_proxy(),
        "manual" => builder.proxy(url::Url::parse(&network.proxy_url).map_err(|e| e.to_string())?),
        _ => builder,
    };
    let updater = builder.build().map_err(|e| e.to_string())?;
    let update = match updater.check().await {
        Ok(update) => update,
        Err(Error::ReleaseNotFound) => None,
        Err(error) => return Err(error.to_string()),
    };
    update.map(|update| {
        let release_page = release_url(&update)?;
        Ok((update, release_page))
    }).transpose()
}

fn release_url(update: &Update) -> Result<String, String> {
    let (platform, suffix) = if cfg!(windows) { ("windows-x86_64", ".exe") } else { ("darwin-aarch64", ".tar.gz") };
    let url = &update.download_url;
    let tag = format!("xiaoban-wallpaper-v{}", update.version);
    let prefix = format!("/bestyize/XiangBan/releases/download/{tag}/XiaoBan-Update-{platform}-");
    let hash = url.path().strip_prefix(&prefix).and_then(|value| value.strip_suffix(suffix));
    if url.scheme() != "https" || url.host_str() != Some("github.com") || url.port().is_some()
        || !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some()
        || !hash.is_some_and(crate::library::hash_name)
    {
        return Err("官方更新包地址无效".into());
    }
    Ok(format!("https://github.com/bestyize/XiangBan/releases/tag/{tag}"))
}

pub async fn check(app: &tauri::AppHandle) -> Result<Option<Value>, String> {
    Ok(find(app).await?.map(|(update, release_page)| {
        json!({
            "available": true,
            "version": update.version,
            "notes": update.body.unwrap_or_default(),
            "publishedAt": update.raw_json["pub_date"],
            "downloadUrl": release_page,
            "inApp": true
        })
    }))
}

fn ensure_idle(app: &tauri::AppHandle) -> Result<(), String> {
    if crate::chat::has_active_runs(app) || crate::games::is_active(app) {
        return Err("请等 AI 任务结束并退出游戏空间，再安装更新。".into());
    }
    Ok(())
}

#[tauri::command]
pub fn install_website_update(app: tauri::AppHandle, version: String) -> Result<(), String> {
    ensure_idle(&app)?;
    {
        let state = app.state::<State>();
        let mut status = state.0.lock().unwrap_or_else(|e| e.into_inner());
        if status.busy {
            return Err("已有更新正在安装".into());
        }
        *status = Installation { busy: true, percent: None, error: None };
    }
    crate::website::publish_status(&app);
    tauri::async_runtime::spawn(async move {
        let result = install(&app, &version).await;
        publish(&app, Installation { busy: false, percent: None, error: result.err() });
    });
    Ok(())
}

async fn install(app: &tauri::AppHandle, version: &str) -> Result<(), String> {
    let (mut update, _) = find(app).await?.filter(|(update, _)| update.version == version)
        .ok_or("更新版本已变化或暂不可用，请重新检查更新。")?;
    update.timeout = Some(Duration::from_secs(600));
    let mut downloaded = 0u64;
    let mut last_percent = None;
    let bytes = update.download(
        |chunk, total| {
            downloaded = downloaded.saturating_add(chunk as u64);
            let percent = total.filter(|&total| total > 0)
                .map(|total| (downloaded.saturating_mul(100) / total).min(99) as u8);
            if percent != last_percent {
                last_percent = percent;
                publish(app, Installation { busy: true, percent, error: None });
            }
        },
        || {},
    ).await.map_err(|error| format!("下载或校验更新失败：{error}"))?;

    ensure_idle(app)?;
    publish(app, Installation { busy: true, percent: Some(100), error: None });
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Windows exits directly after handing off to NSIS, so run native cleanup first.
        #[cfg(windows)]
        crate::chat::shutdown_connectors(&handle);
        update.install(bytes).map_err(|error| format!("安装更新失败：{error}"))?;
        #[cfg(target_os = "macos")]
        handle.request_restart();
        Ok(())
    }).await.map_err(|error| error.to_string())?
}

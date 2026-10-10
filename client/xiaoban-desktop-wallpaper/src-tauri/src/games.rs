//! Trusted desktop game host. Pack frames never get IPC or filesystem capabilities.
use serde_json::{json, Value};
use std::{fs, io::Write, sync::{atomic::{AtomicBool, Ordering}, Mutex}};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const IDS: [&str; 7] = ["falling-blocks", "merge-2048", "block-puzzle", "water-sort", "snake-arena", "ink-blade", "dusk-glider"];
#[derive(Default)]
pub struct GameState {
    pub active: AtomicBool,
    operation: tauri::async_runtime::Mutex<()>,
    io: Mutex<()>,
    pet_position: Mutex<Option<tauri::PhysicalPosition<i32>>>,
    host_token: Mutex<String>,
    smoke_report: Mutex<Option<Value>>,
    pub desktop_entry_ready: AtomicBool,
    pub desktop_entry_error: Mutex<Option<String>>,
}
pub fn is_active(app: &tauri::AppHandle) -> bool { app.state::<GameState>().active.load(Ordering::SeqCst) }
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if label == "games" { return Some(["close_game_space", "get_game_catalog", "load_game_save", "save_game_save", "game_smoke_report"].contains(&command)); }
    if ["open_game_space", "close_game_space", "get_game_status", "set_game_desktop_double_click"].contains(&command) { return Some(label == "control"); }
    if ["get_game_catalog", "load_game_save", "save_game_save", "game_smoke_report"].contains(&command) { return Some(false); }
    None
}
pub fn catalog() -> Result<Vec<Value>, String> {
    let manifests = [
        include_str!("../../public/game-packs/falling-blocks/manifest.json"),
        include_str!("../../public/game-packs/merge-2048/manifest.json"),
        include_str!("../../public/game-packs/block-puzzle/manifest.json"),
        include_str!("../../public/game-packs/water-sort/manifest.json"),
        include_str!("../../public/game-packs/snake-arena/manifest.json"),
        include_str!("../../public/game-packs/ink-blade/manifest.json"),
        include_str!("../../public/game-packs/dusk-glider/manifest.json"),
    ];
    manifests.iter().zip(IDS).map(|(text, id)| {
        let value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
        if value["id"] != id || value["runtimeProtocol"] != 1 || value["entry"] != "index.html" || value["cover"] != "cover.svg" { return Err(format!("游戏 {id} 的资源协议无效")); }
        Ok(value)
    }).collect()
}
#[tauri::command]
pub fn get_game_catalog(app: tauri::AppHandle, host_token: String) -> Result<Vec<Value>, String> { authorize(&app, &host_token)?; catalog() }
fn authorize(app: &tauri::AppHandle, token: &str) -> Result<(), String> {
    let state = app.state::<GameState>(); let expected=state.host_token.lock().unwrap();
    if expected.is_empty() || token != expected.as_str() { return Err("游戏资源没有原生宿主权限".into()); } Ok(())
}
#[tauri::command]
pub fn set_game_desktop_double_click(app: tauri::AppHandle, enabled: bool) -> Result<crate::Settings,String> {
    if enabled && !app.state::<GameState>().desktop_entry_ready.load(Ordering::Relaxed) { return Err("桌面双击入口不可用，请使用玩一玩按钮".into()); }
    let state=app.state::<crate::DesktopState>(); let _update=state.update_lock.lock().unwrap();
    let mut config=crate::settings(&app); config.game_desktop_double_click=enabled; crate::persist(&app,&config)?;
    *state.settings.lock().unwrap()=config.clone(); crate::publish(&app); Ok(config)
}
#[tauri::command]
pub fn get_game_status(app: tauri::AppHandle) -> Value {
    let state = app.state::<GameState>();
    json!({"active":is_active(&app), "desktopDoubleClickSupported":state.desktop_entry_ready.load(Ordering::Relaxed),
        "desktopEntryError":state.desktop_entry_error.lock().unwrap().clone(),
        "platform":std::env::consts::OS, "games":IDS})
}
fn save_path(app: &tauri::AppHandle, id: &str) -> Result<std::path::PathBuf, String> {
    // Only the shipped opt-in pack may persist; IDs never become arbitrary filesystem paths.
    if id != "ink-blade" { return Err("此游戏不使用本地存档".into()); }
    Ok(crate::storage::root(app)?.join("games/saves").join(format!("{id}.json")))
}
pub fn validate_save(value: &Value) -> Result<Vec<u8>, String> {
    if !value.is_object() { return Err("存档必须是 JSON 对象".into()); }
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() > 32768 { return Err("存档超过 32 KiB".into()); }
    Ok(bytes)
}
#[tauri::command]
pub fn load_game_save(app: tauri::AppHandle, game_id: String, host_token: String) -> Result<Option<Value>, String> {
    authorize(&app,&host_token)?;
    let state = app.state::<GameState>(); let _io = state.io.lock().unwrap();
    let path = save_path(&app, &game_id)?;
    match fs::metadata(&path) { Ok(metadata) if metadata.len()>32768=>return Err("存档过大，原进度已保留".into()),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(e)=>return Err(format!("存档读取失败，原进度已保留：{e}")),_=>{} }
    let bytes = match fs::read(&path) { Ok(bytes) => bytes, Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None), Err(e) => return Err(format!("存档读取失败，原进度已保留：{e}")) };
    if bytes.len() > 32768 { return Err("存档过大，原进度已保留".into()); }
    let value = serde_json::from_slice(&bytes).map_err(|e| format!("存档损坏，原进度已保留：{e}"))?;
    validate_save(&value)?; Ok(Some(value))
}
#[tauri::command]
pub fn save_game_save(app: tauri::AppHandle, game_id: String, data: Value, host_token: String) -> Result<(), String> {
    authorize(&app,&host_token)?;
    let state = app.state::<GameState>(); let _io = state.io.lock().unwrap();
    let bytes = validate_save(&data)?; let path = save_path(&app, &game_id)?;
    let directory = path.parent().ok_or("没有存档目录")?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let temporary = directory.join(format!("{game_id}-{}.tmp",std::process::id()));
    let result = (|| -> std::io::Result<()> {
        let mut file = fs::File::create(&temporary)?; file.write_all(&bytes)?; file.sync_all()?; drop(file);
        fs::rename(&temporary, &path)
    })();
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result.map_err(|e| format!("存档保存失败，已有进度已保留：{e}"))
}
pub async fn open(app: &tauri::AppHandle) -> Result<(), String> { open_selected(app, None).await }
async fn open_selected(app: &tauri::AppHandle, game_id: Option<String>) -> Result<(), String> {
    if game_id.as_deref().is_some_and(|id| !IDS.contains(&id)) { return Err("没有找到这款本地游戏".into()); }
    let state = app.state::<GameState>(); let _operation = state.operation.lock().await;
    let content_state = app.state::<crate::content::ContentState>(); let _content = content_state.operation.lock().await;
    if let Some(window) = app.get_webview_window("games") {
        if let Some(id) = &game_id { window.eval(format!("window.dispatchEvent(new CustomEvent('game-launch',{{detail:{}}}));", json!(id))).map_err(|e|e.to_string())?; }
        window.unminimize().map_err(|e|e.to_string())?; window.show().map_err(|e|e.to_string())?; return window.set_focus().map_err(|e|e.to_string()); }
    let monitor = app.primary_monitor().map_err(|e| e.to_string())?.ok_or("没有找到主显示器")?;
    let scale = monitor.scale_factor();
    let mut random=[0u8;32]; getrandom::fill(&mut random).map_err(|e|format!("安全宿主初始化失败：{e}"))?;
    let token=random.iter().map(|b|format!("{b:02x}")).collect::<String>();
    *state.host_token.lock().unwrap()=token.clone();
    let smoke=app.state::<crate::DesktopState>().isolated && std::env::var_os("XIAOBAN_GAMES_SMOKE").is_some();
    *state.smoke_report.lock().unwrap()=None;
    let builder = WebviewWindowBuilder::new(app, "games", WebviewUrl::App(format!("index.html?scene=games{}", game_id.as_ref().map(|id|format!("&game={id}")).unwrap_or_default()).into()))
        .initialization_script(format!("if(window===window.top){{window.__XIAOBAN_GAME_TOKEN__={};window.__XIAOBAN_GAME_SMOKE__={smoke};}}",json!(token)))
        .title("小伴 · 游戏空间").decorations(false).shadow(false).resizable(false).visible(false).fullscreen(true)
        .always_on_top(false).skip_taskbar(false).focused(true)
        .position(monitor.position().x as f64 / scale, monitor.position().y as f64 / scale)
        .inner_size(monitor.size().width as f64 / scale, monitor.size().height as f64 / scale);
    #[cfg(windows)] let builder = builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
    #[cfg(target_os="macos")] let builder = builder.incognito(true);
    let window = builder.build().map_err(|e|e.to_string())?;
    *state.pet_position.lock().unwrap() = app.get_webview_window("pet").and_then(|w| w.outer_position().ok());
    state.active.store(true, Ordering::SeqCst);
    crate::reminders::set_game_active(app, true);
    let _ = app.emit("games-active", true);
    for label in ["pet", "wallpaper", "media-a", "media-b", "media-preview"] {
        app.state::<crate::DesktopState>().ready_scenes.lock().unwrap().remove(label);
        if let Some(background) = app.get_webview_window(label) { if let Err(e) = background.destroy() { state.active.store(false,Ordering::SeqCst); let _=window.destroy(); restore_background(app); return Err(e.to_string()); } }
    }
    if let Err(error)=wait_gone(app,&["pet","wallpaper","media-a","media-b","media-preview"]).await {
        state.active.store(false,Ordering::SeqCst); let _=window.destroy(); restore_background(app); return Err(error);
    }
    if let Err(e) = window.show().and_then(|_| window.set_focus()) {
        state.active.store(false,Ordering::SeqCst); let _=window.destroy(); restore_background(app); return Err(e.to_string());
    }
    Ok(())
}
fn restore_background(app: &tauri::AppHandle) {
    crate::reminders::set_game_active(app, false);
    let _ = app.emit("games-active", false);
    let config = crate::settings(app);
    let _media = match crate::content::restore(app) { Ok(media) => media, Err(e) => { let _=app.emit("desktop-error", format!("退出游戏后壁纸恢复失败：{e}")); false } };
    if let Err(e) = crate::open_scene(app,&config.mode) { let _=app.emit("desktop-error",e); }
    if let Some(position) = app.state::<GameState>().pet_position.lock().unwrap().take() {
        if let Some(pet) = app.get_webview_window("pet") { let _ = pet.set_position(position); }
    }
}
pub async fn close(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<GameState>(); let _operation = state.operation.lock().await;
    let content_state = app.state::<crate::content::ContentState>(); let _content = content_state.operation.lock().await;
    if state.active.swap(false,Ordering::SeqCst) { restore_background(app); }
    if let Some(window) = app.get_webview_window("games") { window.destroy().map_err(|e|e.to_string())?; }
    wait_gone(app,&["games"]).await?;
    Ok(())
}
#[tauri::command]
pub async fn open_game_space(app: tauri::AppHandle, game_id: Option<String>) -> Result<(), String> { open_selected(&app, game_id).await }
#[tauri::command]
pub async fn close_game_space(app: tauri::AppHandle, window: tauri::WebviewWindow, host_token: Option<String>) -> Result<(), String> {
    if window.label()=="games" { authorize(&app,host_token.as_deref().unwrap_or(""))?; }
    close(&app).await
}
pub fn start(app: &tauri::AppHandle) {
    #[cfg(windows)] crate::windows::start_desktop_game_entry(app.clone());
    #[cfg(not(windows))] { *app.state::<GameState>().desktop_entry_error.lock().unwrap()=Some("此平台尚未提供可靠的桌面空白双击判断，请使用设置或菜单栏的玩一玩。".into()); }
}
#[tauri::command]
pub fn game_smoke_report(app:tauri::AppHandle,host_token:String,report:Value)->Result<(),String>{
    authorize(&app,&host_token)?;
    if !app.state::<crate::DesktopState>().isolated || std::env::var_os("XIAOBAN_GAMES_SMOKE").is_none() { return Err("仅隔离验证可使用诊断命令".into()); }
    if report.to_string().len()>16384 { return Err("报告过大".into()); }
    *app.state::<GameState>().smoke_report.lock().unwrap()=Some(report); Ok(())
}
pub async fn smoke(app:&tauri::AppHandle)->Result<Value,String>{
    let before=crate::settings(app);
    let position=app.get_webview_window("pet").and_then(|w|w.outer_position().ok());
    open_selected(app, Some("falling-blocks".into())).await?;
    if !is_active(app) || app.get_webview_window("pet").is_some() || app.get_webview_window("wallpaper").is_some() || app.get_webview_window("media-a").is_some() || app.get_webview_window("media-b").is_some(){return Err("沉浸窗口未暂停背景".into());}
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(145);
    let report=loop {
        if let Some(report)=app.state::<GameState>().smoke_report.lock().unwrap().clone(){break report;}
        if std::time::Instant::now()>deadline{let _=close(app).await;return Err("游戏前端验证超时".into());}
        tokio_sleep().await;
    };
    close(app).await?;
    if is_active(app) || app.get_webview_window("games").is_some(){return Err("退出后仍有游戏覆盖窗口".into());}
    if crate::settings(app)!=before{return Err("游戏修改了长期桌面设置".into());}
    let restored=app.get_webview_window("pet").and_then(|w|w.outer_position().ok());
    if position.is_some()&&restored!=position{return Err("游戏退出后桌宠坐标没有恢复".into());}
    if report.get("error").is_some(){return Err(report["error"].as_str().unwrap_or("游戏验证失败").into());}
    if report["ready"].as_array().map_or(0,|a|a.len())!=7{return Err("七款游戏未全部加载".into());}
    Ok(json!({"frontend":report,"backgroundSuspended":true,"settingsPreserved":true,"positionRestored":position.is_none()||position==restored,"closed":true}))
}
async fn tokio_sleep(){ tauri::async_runtime::spawn_blocking(||std::thread::sleep(std::time::Duration::from_millis(100))).await.ok(); }
async fn wait_gone(app:&tauri::AppHandle,labels:&[&str])->Result<(),String>{
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
    while labels.iter().any(|label|app.get_webview_window(label).is_some()) {
        if std::time::Instant::now()>deadline{return Err("旧场景窗口没有及时释放，请重试".into());}
        tokio_sleep().await;
    } Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn all_seven_local_manifests_are_valid() { let items=catalog().unwrap(); assert_eq!(items.len(),7); assert_eq!(items.iter().filter(|v|v["save"]==true).count(),1); }
    #[test] fn permissions_do_not_grant_control_to_games() { assert_eq!(permitted("games","quit"),Some(false)); assert_eq!(permitted("games","save_game_save"),Some(true)); assert_eq!(permitted("media-a","open_game_space"),Some(false)); assert_eq!(permitted("pet","load_game_save"),Some(false)); }
    #[test] fn save_limit_is_utf8_bytes_and_object_only() { assert!(validate_save(&json!([1])).is_err()); assert!(validate_save(&json!({"text":"猫".repeat(12000)})).is_err()); assert!(validate_save(&json!({"progress":3})).is_ok()); }
}

//! Native, non-modal configuration windows. Console-only operations remain in the console:
//! authenticated native requests are consumed once by it, and replies return only to their caller.
mod policy;
pub use policy::kind;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::{atomic::{AtomicU64, Ordering}, mpsc, Mutex}, time::Duration};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Serialize)]
pub struct Request { id: String, command: String, args: Value }
struct Pending { label: String, request: Request, taken: bool, sender: mpsc::Sender<Result<Value, Value>> }
#[derive(Default)]
pub struct ConfigurationState { pending: Mutex<HashMap<String, Pending>> }
static SEQUENCE: AtomicU64 = AtomicU64::new(1);
static OPEN_LOCK: Mutex<()> = Mutex::new(());
#[derive(Clone, Default, Serialize, Deserialize)]
struct Placement { x: i32, y: i32, width: u32, height: u32 }
fn placement_path(app: &tauri::AppHandle, role: &str) -> Option<std::path::PathBuf> {
    let state = app.state::<crate::DesktopState>();
    let path = state.preferences_path.lock().ok()?;
    path.as_ref()?.parent().map(|p| p.join(format!("{role}-window.json")))
}
fn keep_visible(window: &tauri::WebviewWindow, fallback: Option<tauri::Monitor>) -> Result<(), String> {
    if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())?.or_else(|| fallback.clone()) {
        let area = monitor.work_area(); let scale = monitor.scale_factor();
        let width = (area.size.width as f64 / scale - 32.).max(1.);
        let height = (area.size.height as f64 / scale - 64.).max(1.);
        let minimum = if kind(window.label()) == Some("settings") { 560f64 } else { 400f64 };
        window.set_min_size(Some(tauri::LogicalSize::new(minimum.min(width), 360f64.min(height)))).map_err(|e|e.to_string())?;
    }
    crate::design_window::keep_visible(window, fallback)
}
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if command.starts_with("configuration_window_") {
        return Some(match command {
            "configuration_window_open" => label == "control" || kind(label) == Some("settings"),
            "configuration_window_take" | "configuration_window_reply" => label == "control",
            "configuration_window_invoke" | "configuration_window_close" | "configuration_window_login" => kind(label).is_some(),
            _ => false,
        });
    }
    kind(label).map(|_| false)
}
#[tauri::command]
pub async fn configuration_window_open(app: tauri::AppHandle, window: tauri::WebviewWindow, target: String, tab: Option<String>, provider: Option<String>) -> Result<(), String> {
    if window.label() != "control" && kind(window.label()) != Some("settings") { return Err("只能从控制台或设置打开配置窗口".into()); }
    if !matches!(target.as_str(), "settings" | "model") { return Err("配置窗口类型无效".into()); }
    if tab.as_deref().is_some_and(|t| !["general", "appearance", "ai", "assistant", "voice", "network", "safety", "about"].contains(&t)) { return Err("设置分类无效".into()); }
    let provider = provider.unwrap_or_else(|| "__new__".into());
    if target == "model" && (provider.is_empty() || provider.len() > 200 || provider.chars().any(char::is_control)) { return Err("服务商编号无效".into()); }
    let _lock = OPEN_LOCK.lock().map_err(|_| "配置窗口锁不可用")?;
    let label = if target == "settings" { "configuration-settings".into() } else { format!("configuration-model-{:x}", Sha256::digest(provider.as_bytes()))[..44].to_string() };
    let monitor = window.current_monitor().map_err(|e| e.to_string())?.or(app.primary_monitor().map_err(|e| e.to_string())?);
    if let Some(existing) = app.get_webview_window(&label) {
        existing.unminimize().map_err(|e| e.to_string())?;
        keep_visible(&existing, monitor)?;
        if target == "settings" { if let Some(tab) = tab { existing.emit("configuration-tab", tab).map_err(|e| e.to_string())?; } }
        existing.show().map_err(|e| e.to_string())?;
        return existing.set_focus().map_err(|e| e.to_string());
    }
    let path = placement_path(&app, &target);
    let placement = path.as_ref().and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice::<Placement>(&b).ok()).unwrap_or_default();
    let query = url::form_urlencoded::Serializer::new(String::new()).append_pair("scene", "configuration").append_pair("kind", &target).append_pair("tab", tab.as_deref().unwrap_or("general")).append_pair("provider", &provider).finish();
    let appearance = crate::settings(&app).appearance;
    let theme = match appearance.as_str() { "dark" => Some(tauri::Theme::Dark), "light" => Some(tauri::Theme::Light), _ => None };
    let (width, height, min_width) = if target == "settings" { (940f64, 700f64, 560f64) } else { (640., 820., 400.) };
    let (max_w, max_h) = monitor.as_ref().map(|m| { let a=m.work_area(); let s=m.scale_factor(); ((a.size.width as f64/s-32.).max(1.), (a.size.height as f64/s-64.).max(1.)) }).unwrap_or((width, height));
    let mut builder = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(format!("index.html?{query}").into()))
        .title(if target == "settings" { "设置 · 小伴桌面" } else { "模型配置 · 小伴桌面" })
        .inner_size(width.min(max_w), height.min(max_h)).min_inner_size(min_width.min(max_w), 360f64.min(max_h))
        .resizable(true).decorations(true).theme(theme).visible(false);
    // Restore only onto a currently connected display, then fit the full native frame inside its work area.
    let restore = app.available_monitors().map_err(|e| e.to_string())?.into_iter().find(|m| { let a=m.work_area(); placement.width>0 && placement.height>0 && i64::from(placement.x)+i64::from(placement.width)>i64::from(a.position.x) && i64::from(placement.y)+i64::from(placement.height)>i64::from(a.position.y) && i64::from(placement.x)<i64::from(a.position.x)+i64::from(a.size.width) && i64::from(placement.y)<i64::from(a.position.y)+i64::from(a.size.height) });
    if let Some(m) = restore {
        let f=m.scale_factor(); let a=m.work_area(); let mw=(a.size.width as f64/f-32.).max(1.); let mh=(a.size.height as f64/f-64.).max(1.);
        builder=builder.min_inner_size(min_width.min(mw),360f64.min(mh)).position(placement.x as f64/f,placement.y as f64/f).inner_size((placement.width as f64/f).min(mw),(placement.height as f64/f).min(mh));
    } else if let Some(m)=&monitor {
        let a=m.work_area(); let f=m.scale_factor();
        builder=builder.position(a.position.x as f64/f+(a.size.width as f64/f-width.min(max_w))/2.,a.position.y as f64/f+(a.size.height as f64/f-height.min(max_h))/2.);
    } else { builder=builder.center(); }
    let child=builder.build().map_err(|e|e.to_string())?;
    #[cfg(target_os="macos")] let _=crate::macos::apply_control_appearance(&child,&appearance);
    keep_visible(&child,monitor)?;
    let own=child.clone();
    child.on_window_event(move |event| {
        if matches!(event,tauri::WindowEvent::CloseRequested{..}) {
            if let (Some(path),Ok(p),Ok(s))=(&path,own.outer_position(),own.inner_size()) {
                if !own.is_maximized().unwrap_or(false) && !own.is_minimized().unwrap_or(false) {
                    let state=Placement{x:p.x,y:p.y,width:s.width,height:s.height};
                    let _=crate::library::save_json(path,&state);
                }
            }
        }
        if matches!(event,tauri::WindowEvent::Destroyed) {
            own.state::<ConfigurationState>().pending.lock().unwrap_or_else(|e|e.into_inner()).retain(|_,p|p.label!=own.label());
        }
    });
    child.show().map_err(|e|e.to_string())?;
    child.set_focus().map_err(|e|e.to_string())
}
#[tauri::command]
pub fn configuration_window_close(window: tauri::WebviewWindow) -> Result<(), String> {
    if kind(window.label()).is_none() { return Err("只能关闭当前配置窗口".into()); }
    window.close().map_err(|e|e.to_string())
}
#[tauri::command]
pub fn configuration_window_login(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
    if kind(window.label()).is_none() { return Err("只能从配置窗口打开登录".into()); }
    crate::present_control(&app)?;
    app.emit_to("control", "configuration-login", ()).map_err(|e|e.to_string())
}
#[tauri::command]
pub async fn configuration_window_invoke(app: tauri::AppHandle, window: tauri::WebviewWindow, command: String, args: Option<Value>) -> Result<Value, Value> {
    let role=kind(window.label()).ok_or_else(||json!("配置窗口无权限"))?;
    if !policy::command_allowed(role,&command) { return Err(json!("此配置窗口不能执行该操作")); }
    let args=args.unwrap_or_else(||json!({}));
    if !args.is_object() || args.to_string().len()>1_000_000 { return Err(json!("配置参数无效或过大")); }
    let id=SEQUENCE.fetch_add(1,Ordering::Relaxed).to_string(); let (tx,rx)=mpsc::channel();
    {
        let state=app.state::<ConfigurationState>();
        let mut pending=state.pending.lock().map_err(|_|json!("配置请求锁不可用"))?;
        if pending.len()>=32 { return Err(json!("正在处理的配置请求过多，请稍后重试")); }
        pending.insert(id.clone(),Pending{label:window.label().into(),request:Request{id:id.clone(),command,args},taken:false,sender:tx});
    }
    let result=match app.emit_to("control","configuration-command",json!({"id":id})) {
        Ok(()) => tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(Duration::from_secs(180)).unwrap_or_else(|_|Err(json!("配置操作超时，请检查主窗口是否仍在运行")))).await.unwrap_or_else(|_|Err(json!("配置操作已中断"))),
        Err(_) => Err(json!("主窗口不可用，无法保存配置")),
    };
    app.state::<ConfigurationState>().pending.lock().unwrap_or_else(|e|e.into_inner()).remove(&id);
    result
}
#[tauri::command]
pub fn configuration_window_take(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String) -> Result<Request,String> {
    if window.label()!="control" { return Err("只能由主窗口处理配置请求".into()); }
    let state=app.state::<ConfigurationState>();
        let mut pending=state.pending.lock().map_err(|_|"配置请求锁不可用")?;
    let p=pending.get_mut(&id).ok_or("配置请求已过期")?;
    if p.taken { return Err("配置请求已处理".into()); } p.taken=true; Ok(p.request.clone())
}
#[tauri::command]
pub fn configuration_window_reply(app: tauri::AppHandle, window: tauri::WebviewWindow, id: String, ok: bool, value: Value) -> Result<(),String> {
    if window.label()!="control" { return Err("只能由主窗口返回配置结果".into()); }
    let p=app.state::<ConfigurationState>().pending.lock().map_err(|_|"配置请求锁不可用")?.remove(&id).ok_or("配置请求已过期")?;
    if !p.taken { return Err("配置请求尚未处理".into()); }
    let _=p.sender.send(if ok {Ok(value)} else {Err(value)}); Ok(())
}

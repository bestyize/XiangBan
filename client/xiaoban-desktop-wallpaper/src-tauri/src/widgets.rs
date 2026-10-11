//! Small, independent desktop windows. Only the control window can alter the collection.
use crate::care_weather::{City, Weather};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fs, sync::{atomic::{AtomicBool, Ordering}, Mutex}};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Widget {
    pub id: String, pub kind: String, pub theme: String, pub scale: f64,
    pub x: f64, pub y: f64, pub enabled: bool, pub city: Option<City>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout { pub version: u32, pub widgets: Vec<Widget> }
impl Default for Layout { fn default() -> Self { Self { version: 1, widgets: vec![] } } }
#[derive(Default)]
pub struct WidgetState {
    layout: Mutex<Layout>, editing: AtomicBool,
    operation: tauri::async_runtime::Mutex<()>,
    weather_operation: tauri::async_runtime::Mutex<()>,
    load_error: Mutex<Option<String>>, errors: Mutex<Vec<String>>,
    weather: Mutex<HashMap<String, Weather>>, attempts: Mutex<HashMap<String, i64>>,
}
pub fn is_widget(label: &str) -> bool { label.starts_with("widget-") }
fn own_widget(window: &tauri::WebviewWindow) -> Result<Widget, String> {
    window.app_handle().state::<WidgetState>().layout.lock().unwrap().widgets.iter()
        .find(|widget| format!("widget-{}", widget.id) == window.label()).cloned().ok_or("没有找到这个桌面组件".into())
}
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if is_widget(label) { return Some(["get_desktop_widget", "move_desktop_widget", "show_widget_settings", "get_widget_weather", "open_ritual_space"].contains(&command)); }
    if ["get_desktop_widgets", "add_desktop_widget", "update_desktop_widget", "remove_desktop_widget", "set_widgets_editing", "get_desktop_widget", "move_desktop_widget", "show_widget_settings", "get_widget_weather"].contains(&command) { return Some(label == "control"); }
    None
}
pub fn validate(layout: &Layout) -> Result<(), String> {
    if layout.version != 1 || layout.widgets.len() > 12 { return Err("桌面最多放置 12 个组件".into()); }
    let mut ids = std::collections::HashSet::new();
    for widget in &layout.widgets {
        if widget.id.is_empty() || widget.id.len() > 32 || !widget.id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) || !ids.insert(&widget.id)
            || !["fortune", "calendar", "weather"].contains(&widget.kind.as_str())
            || !["glass", "paper", "transparent"].contains(&widget.theme.as_str())
            || !widget.scale.is_finite() || !(0.75..=1.5).contains(&widget.scale)
            || !widget.x.is_finite() || !(0.0..=1.0).contains(&widget.x)
            || !widget.y.is_finite() || !(0.0..=1.0).contains(&widget.y) { return Err("组件配置无效，原布局已保留".into()); }
        if let Some(city) = &widget.city {
            city.validate()?;
            if widget.kind != "weather" || city.country.len() > 160 || city.admin1.len() > 160 { return Err("天气城市配置无效".into()); }
        }
    }
    Ok(())
}
fn ensure_writable(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(error) = app.state::<WidgetState>().load_error.lock().unwrap().clone() { return Err(error); }
    Ok(())
}
fn persist(app: &tauri::AppHandle, layout: &Layout) -> Result<(), String> {
    validate(layout)?; ensure_writable(app)?;
    let folder = crate::storage::root(app)?.join("config"); fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(layout).map_err(|e| e.to_string())?;
    let temporary = folder.join("widgets.tmp"); fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    fs::rename(temporary, folder.join("widgets.json")).map_err(|e| format!("布局保存失败，原布局已保留：{e}"))
}
fn snapshot(app: &tauri::AppHandle) -> Value {
    let state = app.state::<WidgetState>(); let layout = state.layout.lock().unwrap().clone();
    let desktop=app.primary_monitor().ok().flatten().map(|m|json!({"width":m.size().width as f64/m.scale_factor(),"height":m.size().height as f64/m.scale_factor()}));
    json!({"version":1,"widgets":layout.widgets,"editing":state.editing.load(Ordering::Relaxed),"errors":state.errors.lock().unwrap().clone(),"desktop":desktop})
}
pub fn base_size(kind: &str) -> (f64, f64) { match kind { "fortune" => (220.0, 330.0), "calendar" => (280.0, 330.0), _ => (280.0, 230.0) } }
fn geometry(app: &tauri::AppHandle, widget: &Widget) -> Result<(f64, f64, f64, f64, f64), String> {
    let monitor = app.primary_monitor().map_err(|e| e.to_string())?.ok_or("没有找到主显示器")?;
    let scale = monitor.scale_factor(); let (base_w, base_h) = base_size(&widget.kind);
    let width = base_w * widget.scale + 16.0; let height = base_h * widget.scale + 16.0;
    let x = monitor.position().x as f64 / scale + widget.x * (monitor.size().width as f64 / scale - width).max(0.0);
    let y = monitor.position().y as f64 / scale + 32.0 + widget.y * (monitor.size().height as f64 / scale - height - 88.0).max(0.0);
    Ok((x, y, width, height, scale))
}
fn widget_payload(app: &tauri::AppHandle, widget: &Widget) -> Result<Value, String> {
    let (x, y, _, _, _) = geometry(app, widget)?;
    Ok(json!({"widget":widget,"editing":app.state::<WidgetState>().editing.load(Ordering::Relaxed),"screenX":x,"screenY":y}))
}
fn place(app: &tauri::AppHandle, window: &tauri::WebviewWindow, widget: &Widget) -> Result<(), String> {
    let (x, y, width, height, dpi) = geometry(app, widget)?;
    #[cfg(windows)] return crate::windows::place_widget(window, tauri::PhysicalPosition::new((x*dpi).round() as i32,(y*dpi).round() as i32), (width*dpi).round() as i32,(height*dpi).round() as i32);
    #[cfg(not(windows))] { let _ = dpi; window.set_size(tauri::LogicalSize::new(width,height)).and_then(|_| window.set_position(tauri::LogicalPosition::new(x,y))).map_err(|e| e.to_string()) }
}
pub fn sync(app: &tauri::AppHandle) {
    let state = app.state::<WidgetState>(); let layout = state.layout.lock().unwrap().clone(); let mut errors = vec![];
    let enabled: Vec<_> = layout.widgets.iter().filter(|w| w.enabled && !crate::games::is_active(app)).collect();
    for (label, window) in app.webview_windows() {
        if is_widget(&label) && !enabled.iter().any(|w| label == format!("widget-{}",w.id)) { let _ = window.destroy(); }
    }
    for widget in enabled {
        let result = (|| -> Result<(), String> {
            let label = format!("widget-{}",widget.id);
            if let Some(window) = app.get_webview_window(&label) {
                place(app,&window,widget)?;
                window.eval(format!("window.dispatchEvent(new CustomEvent('widget-config',{{detail:{}}}));",widget_payload(app,widget)?)).map_err(|e| e.to_string())?;
                return Ok(());
            }
            let (x,y,width,height,_) = geometry(app,widget)?;
            let builder = WebviewWindowBuilder::new(app,&label,WebviewUrl::App(format!("index.html?scene=widget&id={}",widget.id).into()))
                .title("小伴 · 桌面组件").decorations(false).shadow(false).transparent(true).resizable(false).visible(false)
                .skip_taskbar(true).always_on_top(false).focused(false).position(x,y).inner_size(width,height)
                .initialization_script(format!("window.__XIAOBAN_WIDGET__={};",widget_payload(app,widget)?));
            // Keep the native parent in Tao's own state as well as the final HWND styles.
            #[cfg(windows)] let builder = builder.parent_raw(::windows::Win32::Foundation::HWND(crate::windows::widget_parent()? as _))
                .data_directory(crate::storage::webview(&crate::storage::root(app)?));
            #[cfg(target_os="macos")] let builder = builder.incognito(true);
            let window = builder.build().map_err(|e| e.to_string())?;
            let configured = (|| -> Result<(),String> {
                #[cfg(windows)] crate::windows::attach_widget(&window)?;
                #[cfg(target_os="macos")] crate::macos::configure_scene(&window,true,false)?;
                place(app,&window,widget)?; window.show().map_err(|e| e.to_string())?;
                // Show can reconcile Tao's window styles; apply the desktop child frame last.
                #[cfg(windows)] crate::windows::attach_widget(&window)?;
                place(app,&window,widget)?; Ok(())
            })();
            if let Err(error) = configured { let _=window.destroy(); return Err(error); }
            Ok(())
        })();
        if let Err(error) = result { errors.push(format!("{}：{error}",widget.kind)); }
    }
    *state.errors.lock().unwrap() = errors;
    if let Some(control) = app.get_webview_window("control") { let _ = control.eval(format!("window.dispatchEvent(new CustomEvent('widgets-changed',{{detail:{}}}));",snapshot(app))); }
}
pub fn start(app: &tauri::AppHandle) {
    let result = (|| -> Result<Layout,String> {
        let path=crate::storage::root(app)?.join("config/widgets.json");
        match fs::metadata(&path) { Err(e) if e.kind()==std::io::ErrorKind::NotFound => return Ok(Layout::default()),Err(e)=>return Err(e.to_string()),Ok(m) if m.len()>32768=>return Err("组件布局过大，原文件已保留".into()),_=>{} }
        let layout: Layout = serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|e|format!("组件布局无法读取，原文件已保留：{e}"))?;
        validate(&layout)?; Ok(layout)
    })();
    let state=app.state::<WidgetState>();
    match result { Ok(layout)=>*state.layout.lock().unwrap()=layout,Err(error)=>*state.load_error.lock().unwrap()=Some(error) }
    sync(app);
}
#[tauri::command]
pub fn get_desktop_widgets(app: tauri::AppHandle) -> Result<Value,String> { ensure_writable(&app)?; Ok(snapshot(&app)) }
#[tauri::command]
pub async fn add_desktop_widget(app: tauri::AppHandle, kind: String) -> Result<Value,String> {
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let mut next=state.layout.lock().unwrap().clone(); if next.widgets.len()>=12{return Err("桌面最多放置 12 个组件".into());}
    let mut random=[0u8;8];getrandom::fill(&mut random).map_err(|e|e.to_string())?;
    let index=next.widgets.len();
    let theme=if kind=="fortune"{"paper"}else{"glass"}.into();
    next.widgets.push(Widget{id:random.iter().map(|b|format!("{b:02x}")).collect(),kind,theme,scale:1.0,x:((if index%3==1{0.88}else{0.52})-(index/3) as f64*0.25).max(0.05),y:if index%3==2{0.94}else{0.06},enabled:true,city:None});
    persist(&app,&next)?; *state.layout.lock().unwrap()=next; sync(&app); Ok(snapshot(&app))
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Patch { theme: Option<String>, scale: Option<f64>, x: Option<f64>, y: Option<f64>, enabled: Option<bool>, city: Option<City> }
#[tauri::command]
pub async fn update_desktop_widget(app: tauri::AppHandle, id: String, patch: Patch) -> Result<Value,String> {
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let mut next=state.layout.lock().unwrap().clone();let widget=next.widgets.iter_mut().find(|w|w.id==id).ok_or("没有找到这个组件")?;
    if let Some(v)=patch.theme{widget.theme=v;}if let Some(v)=patch.scale{widget.scale=v;}if let Some(v)=patch.x{widget.x=v;}if let Some(v)=patch.y{widget.y=v;}if let Some(v)=patch.enabled{widget.enabled=v;}if let Some(v)=patch.city{widget.city=Some(v);}
    persist(&app,&next)?;*state.layout.lock().unwrap()=next;sync(&app);Ok(snapshot(&app))
}
#[tauri::command]
pub async fn remove_desktop_widget(app: tauri::AppHandle,id:String)->Result<Value,String>{
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let mut next=state.layout.lock().unwrap().clone();next.widgets.retain(|w|w.id!=id);
    persist(&app,&next)?;*state.layout.lock().unwrap()=next;sync(&app);Ok(snapshot(&app))
}
#[tauri::command]
pub async fn set_widgets_editing(app:tauri::AppHandle,editing:bool)->Result<Value,String>{
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    persist(&app,&state.layout.lock().unwrap().clone())?;state.editing.store(editing,Ordering::Relaxed);sync(&app);Ok(snapshot(&app))
}
#[tauri::command]
pub fn get_desktop_widget(window:tauri::WebviewWindow)->Result<Value,String>{widget_payload(window.app_handle(),&own_widget(&window)?)}
#[tauri::command]
pub async fn move_desktop_widget(window:tauri::WebviewWindow,x:f64,y:f64,commit:bool)->Result<(),String>{
    let app=window.app_handle();let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    if !x.is_finite()||!y.is_finite(){return Err("组件位置无效".into());}
    let original=own_widget(&window)?;let (old_x,old_y,width,height,dpi)=geometry(app,&original)?;
    let monitor=app.primary_monitor().map_err(|e|e.to_string())?.ok_or("没有找到显示器")?;
    let mut next=state.layout.lock().unwrap().clone();let widget=next.widgets.iter_mut().find(|w|w.id==original.id).ok_or("没有找到组件")?;
    let nx=(widget.x+(x-old_x)/(monitor.size().width as f64/dpi-width).max(1.0)).clamp(0.0,1.0);
    let ny=(widget.y+(y-old_y)/(monitor.size().height as f64/dpi-height-88.0).max(1.0)).clamp(0.0,1.0);
    widget.x=nx;widget.y=ny;let moved=widget.clone();
    if commit{persist(app,&next)?;}*state.layout.lock().unwrap()=next;place(app,&window,&moved)?;
    if commit{sync(app);}Ok(())
}
#[tauri::command]
pub fn show_widget_settings(window:tauri::WebviewWindow)->Result<(),String>{
    own_widget(&window)?;crate::present_control(window.app_handle())?;
    if let Some(control)=window.app_handle().get_webview_window("control"){control.eval("window.dispatchEvent(new Event('widgets-manage'));".to_string()).map_err(|e|e.to_string())?;}Ok(())
}
#[tauri::command]
pub async fn get_widget_weather(window:tauri::WebviewWindow)->Result<Weather,String>{
    let widget=own_widget(&window)?;if widget.kind!="weather"{return Err("此组件不读取天气".into());}
    let city=widget.city.ok_or("请在桌面组件里选择城市")?;let app=window.app_handle().clone();let state=app.state::<WidgetState>();
    let _operation=state.weather_operation.lock().await;let source=crate::reminders::widget_weather_source(&app);
    let key=format!("{}:{}:{}",city.latitude,city.longitude,serde_json::to_string(&source).map_err(|e|e.to_string())?);
    let now=chrono::Utc::now().timestamp();
    if let Some(weather)=state.weather.lock().unwrap().get(&key).filter(|w|(0..1800).contains(&(now-w.updated_at))&&(-1800..7200).contains(&(now-w.model_at))).cloned(){return Ok(weather);}
    {let mut attempts=state.attempts.lock().unwrap();if attempts.get(&key).is_some_and(|last|(0..60).contains(&(now-last))){return Err("天气查询每分钟最多一次，请稍后再试".into());}attempts.insert(key.clone(),now);if attempts.len()>48{attempts.retain(|_,t|(0..3600).contains(&(now-*t)));}}
    let weather=tauri::async_runtime::spawn_blocking(move||crate::care_weather::fetch(&source,&city,now)).await.map_err(|e|e.to_string())??;
    let mut cache=state.weather.lock().unwrap();if cache.len()>48{cache.clear();}cache.insert(key,weather.clone());Ok(weather)
}
#[cfg(test)]
mod tests{
    use super::*;
    fn one()->Layout{Layout{version:1,widgets:vec![Widget{id:"abc123".into(),kind:"calendar".into(),theme:"glass".into(),scale:1.0,x:0.8,y:0.2,enabled:true,city:None}]}}
    #[test]fn bounds_and_duplicate_ids_are_checked(){let mut layout=one();assert!(validate(&layout).is_ok());layout.widgets.push(layout.widgets[0].clone());assert!(validate(&layout).is_err());layout=one();layout.widgets[0].x=f64::NAN;assert!(validate(&layout).is_err());layout=one();layout.widgets[0].scale=10.;assert!(validate(&layout).is_err());}
    #[test]fn window_cannot_manage_other_widgets_or_wallpapers(){assert_eq!(permitted("widget-abc123","add_desktop_widget"),Some(false));assert_eq!(permitted("widget-abc123","apply_wallpaper"),Some(false));assert_eq!(permitted("widget-abc123","move_desktop_widget"),Some(true));assert_eq!(permitted("media-a","add_desktop_widget"),Some(false));}
    #[test]fn old_or_corrupt_layouts_are_rejected(){let mut layout=one();layout.version=2;assert!(validate(&layout).is_err());let mut layout=one();layout.widgets[0].id="../escape".into();assert!(validate(&layout).is_err());}
}

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
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub note: Option<NoteData>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub monitor: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub countdown: Option<CountdownData>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub custom_id: Option<String>,
}
const MAX_LAYOUT_BYTES: usize = 128 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TodoItem { pub id: String, pub text: String, pub done: bool }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteData { pub title: String, pub mode: String, pub text: String, pub items: Vec<TodoItem> }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountdownData { pub title: String, pub date: String, pub mode: String, pub repeat: String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WidgetContent { note: Option<NoteData>, countdown: Option<CountdownData> }
fn short_text(value: &str, max: usize) -> bool { !value.trim().is_empty() && value.encode_utf16().count() <= max }
fn valid_identity(value: &str) -> bool { !value.is_empty() && value.len()<=32 && value.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()) }
fn validate_content(widget: &Widget) -> Result<(),String> {
    if let Some(note)=&widget.note {
        if widget.kind!="notes" || !short_text(&note.title,40) || !["note","todo"].contains(&note.mode.as_str()) || note.text.encode_utf16().count()>1200 || note.items.len()>12 { return Err("便签内容无效或超过长度限制".into()); }
        let mut ids=std::collections::HashSet::new();
        for item in &note.items { if !valid_identity(&item.id)||!ids.insert(&item.id)||!short_text(&item.text,120) {return Err("待办内容无效".into());} }
    }
    if let Some(data)=&widget.countdown {
        let valid_date=chrono::NaiveDate::parse_from_str(&data.date,"%Y-%m-%d").is_ok_and(|date|date.format("%Y-%m-%d").to_string()==data.date && data.date.as_str()>="1900-01-01" && data.date.as_str()<="2100-12-31");
        if widget.kind!="countdown" || !short_text(&data.title,40) || !valid_date || !["countdown","elapsed"].contains(&data.mode.as_str()) || !["none","yearly"].contains(&data.repeat.as_str()) || (data.mode=="elapsed"&&data.repeat!="none") {return Err("纪念日配置无效".into());}
    }
    Ok(())
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
    if is_widget(label) { return Some(["get_desktop_widget", "move_desktop_widget", "show_widget_settings", "get_widget_weather", "save_desktop_widget_content", "open_ritual_space"].contains(&command)); }
    if ["get_desktop_widgets", "add_desktop_widget", "update_desktop_widget", "remove_desktop_widget", "set_widgets_editing", "get_desktop_widget", "move_desktop_widget", "show_widget_settings", "get_widget_weather", "save_desktop_widget_content", "list_custom_widgets", "save_custom_widget", "remove_custom_widget"].contains(&command) { return Some(label == "control"); }
    None
}
pub fn validate(layout: &Layout) -> Result<(), String> {
    if layout.version != 1 || layout.widgets.len() > 12 { return Err("桌面最多放置 12 个组件".into()); }
    let mut ids = std::collections::HashSet::new();
    for widget in &layout.widgets {
        if widget.id.is_empty() || widget.id.len() > 32 || !widget.id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) || !ids.insert(&widget.id)
            || !["fortune", "calendar", "weather", "notes", "countdown", "custom"].contains(&widget.kind.as_str())
            || !["glass", "paper", "transparent"].contains(&widget.theme.as_str())
            || !widget.scale.is_finite() || !(0.75..=1.5).contains(&widget.scale)
            || !widget.x.is_finite() || !(0.0..=1.0).contains(&widget.x)
            || !widget.y.is_finite() || !(0.0..=1.0).contains(&widget.y) { return Err("组件配置无效，原布局已保留".into()); }
        if widget.kind == "custom" && !widget.custom_id.as_ref().is_some_and(|id| crate::custom_widgets::valid_widget_id(id)) {
            return Err("自定义组件必须包含有效的 customId".into());
        }
        if widget.monitor.as_ref().is_some_and(|id|id.is_empty()||id.len()>512){return Err("显示器标识无效".into());}
        validate_content(widget)?;
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
    if bytes.len()>MAX_LAYOUT_BYTES{return Err("组件内容过大，请减少文字后重试".into());}
    let temporary = folder.join("widgets.tmp"); fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    fs::rename(temporary, folder.join("widgets.json")).map_err(|e| format!("布局保存失败，原布局已保留：{e}"))
}
fn monitors(app:&tauri::AppHandle)->Result<Vec<crate::widget_geometry::MonitorArea>,String>{
    #[cfg(target_os="macos")] {return crate::macos::widget_monitors(app);}
    #[cfg(not(target_os="macos"))] {
        let primary=app.primary_monitor().map_err(|e|e.to_string())?;
        Ok(app.available_monitors().map_err(|e|e.to_string())?.into_iter().enumerate().map(|(index,m)|{
            let name=m.name().cloned().unwrap_or_else(||format!("显示器 {}",index+1));
            crate::widget_geometry::MonitorArea{id:name.clone(),name,x:m.position().x as f64,y:m.position().y as f64,width:m.size().width as f64,height:m.size().height as f64,scale:m.scale_factor(),primary:primary.as_ref().is_some_and(|p|p.position()==m.position())}
        }).collect())
    }
}
fn cursor(app:&tauri::AppHandle)->Result<(f64,f64),String>{
    #[cfg(target_os="macos")] {return crate::macos::widget_cursor(app);}
    #[cfg(not(target_os="macos"))] {let p=app.cursor_position().map_err(|e|e.to_string())?;Ok((p.x,p.y))}
}
fn attach_custom_data(app: &tauri::AppHandle, widget: &Widget) -> Value {
    let mut val = serde_json::to_value(widget).unwrap_or_default();
    if widget.kind == "custom" {
        if let Some(cid) = &widget.custom_id {
            if let Ok(root) = crate::storage::root(app) {
                if let Some(pack) = crate::custom_widgets::find(&root.join("custom_widgets"), cid) {
                    val["custom"] = pack;
                }
            }
        }
    }
    val
}
fn snapshot(app: &tauri::AppHandle) -> Value {
    let state=app.state::<WidgetState>();let layout=state.layout.lock().unwrap().clone();
    let areas=monitors(app).unwrap_or_default();
    let displays:Vec<_>=areas.iter().map(|m|json!({"id":m.id,"name":m.name,"width":m.width/m.scale,"height":m.height/m.scale,"primary":m.primary})).collect();
    let desktop=crate::widget_geometry::resolve(&areas,None).map(|m|json!({"width":m.width/m.scale,"height":m.height/m.scale}));
    let widgets_with_custom: Vec<_> = layout.widgets.iter().map(|w| attach_custom_data(app, w)).collect();
    json!({"version":1,"widgets":widgets_with_custom,"editing":state.editing.load(Ordering::Relaxed),"errors":state.errors.lock().unwrap().clone(),"desktop":desktop,"monitors":displays})
}
pub fn base_size(kind: &str) -> (f64, f64) { match kind { "fortune" => (220.0, 330.0), "calendar" => (300.0, 400.0), "notes" => (300.0, 360.0), _ => (280.0, 230.0) } }
pub fn widget_base_size(app: &tauri::AppHandle, widget: &Widget) -> (f64, f64) {
    if widget.kind == "custom" {
        if let Some(cid) = &widget.custom_id {
            if let Ok(root) = crate::storage::root(app) {
                if let Some(pack) = crate::custom_widgets::find(&root.join("custom_widgets"), cid) {
                    let w = pack.get("width").and_then(Value::as_f64).unwrap_or(280.0);
                    let h = pack.get("height").and_then(Value::as_f64).unwrap_or(240.0);
                    return (w, h);
                }
            }
        }
        return (280.0, 240.0);
    }
    base_size(&widget.kind)
}
fn geometry(app: &tauri::AppHandle, widget: &Widget) -> Result<(f64, f64, f64, f64, f64), String> {
    let areas=monitors(app)?;let monitor=crate::widget_geometry::resolve(&areas,widget.monitor.as_deref()).ok_or("没有找到显示器")?;
    let scale=monitor.scale;let (base_w,base_h)=widget_base_size(app, widget);
    let width=(base_w*widget.scale+16.0)*scale;let height=(base_h*widget.scale+16.0)*scale;
    let (x,y)=crate::widget_geometry::position(monitor,width,height,widget.x,widget.y);
    Ok((x,y,width,height,scale))
}
fn widget_payload(app: &tauri::AppHandle, widget: &Widget) -> Result<Value, String> {
    let (x,y,_,_,dpi)=geometry(app,widget)?;
    let display_scale=if cfg!(windows){Some(dpi)}else{None};
    let widget_val = attach_custom_data(app, widget);
    Ok(json!({"widget":widget_val,"editing":app.state::<WidgetState>().editing.load(Ordering::Relaxed),"screenX":x,"screenY":y,"displayScale":display_scale}))
}
fn place(app: &tauri::AppHandle, window: &tauri::WebviewWindow, widget: &Widget) -> Result<(), String> {
    let (x,y,width,height,dpi)=geometry(app,widget)?;
    #[cfg(windows)] return crate::windows::place_widget(window,tauri::PhysicalPosition::new(x.round() as i32,y.round() as i32),width.round() as i32,height.round() as i32);
    #[cfg(target_os="macos")] {let _=dpi;return crate::macos::place_widget(window,x,y,width,height);}
    #[cfg(not(any(windows,target_os="macos")))] {window.set_size(tauri::LogicalSize::new(width/dpi,height/dpi)).and_then(|_|window.set_position(tauri::PhysicalPosition::new(x,y))).map_err(|e|e.to_string())}
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
            let (x,y,width,height,dpi) = geometry(app,widget)?;
            let builder = WebviewWindowBuilder::new(app,&label,WebviewUrl::App(format!("index.html?scene=widget&id={}",widget.id).into()))
                .title("小伴 · 桌面组件").decorations(false).shadow(false).transparent(true).resizable(false).visible(false)
                .skip_taskbar(true).always_on_top(false).focused(false).position(x/dpi,y/dpi).inner_size(width/dpi,height/dpi)
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
        match fs::metadata(&path) { Err(e) if e.kind()==std::io::ErrorKind::NotFound => return Ok(Layout::default()),Err(e)=>return Err(e.to_string()),Ok(m) if m.len()>MAX_LAYOUT_BYTES as u64=>return Err("组件布局过大，原文件已保留".into()),_=>{} }
        let layout: Layout = serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|e|format!("组件布局无法读取，原文件已保留：{e}"))?;
        validate(&layout)?; Ok(layout)
    })();
    let state=app.state::<WidgetState>();
    match result { Ok(layout)=>*state.layout.lock().unwrap()=layout,Err(error)=>*state.load_error.lock().unwrap()=Some(error) }
    sync(app);
    let app=app.clone();std::thread::spawn(move||{
        let mut previous=String::new();
        loop{std::thread::sleep(std::time::Duration::from_secs(3));
            if let Ok(areas)=monitors(&app){let signature=serde_json::to_string(&areas).unwrap_or_default();if signature!=previous{previous=signature;sync(&app);}}
        }
    });
}
#[tauri::command]
pub fn get_desktop_widgets(app: tauri::AppHandle) -> Result<Value,String> { ensure_writable(&app)?; Ok(snapshot(&app)) }
#[tauri::command]
pub async fn add_desktop_widget(app: tauri::AppHandle, kind: String, custom_id: Option<String>) -> Result<Value,String> {
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let mut next=state.layout.lock().unwrap().clone(); if next.widgets.len()>=12{return Err("桌面最多放置 12 个组件".into());}
    let mut random=[0u8;8];getrandom::fill(&mut random).map_err(|e|e.to_string())?;
    let index=next.widgets.len();
    let note=if kind=="notes"{Some(NoteData{title:"随手记".into(),mode:"note".into(),text:String::new(),items:vec![]})}else{None};
    let countdown=if kind=="countdown"{Some(CountdownData{title:"值得期待的日子".into(),date:chrono::Local::now().format("%Y-%m-%d").to_string(),mode:"countdown".into(),repeat:"none".into()})}else{None};
    let mut theme=if kind=="fortune"{"paper"}else{"glass"}.into();
    if kind == "custom" {
        let cid = custom_id.as_deref().ok_or("添加自定义组件需要指定 customId")?;
        let root = crate::storage::root(&app)?;
        let pack = crate::custom_widgets::find(&root.join("custom_widgets"), cid).ok_or_else(|| format!("未找到自定义小组件模板「{cid}」"))?;
        if let Some(t) = pack.get("theme").and_then(Value::as_str) {
            theme = t.to_string();
        }
    }
    next.widgets.push(Widget{id:random.iter().map(|b|format!("{b:02x}")).collect(),kind,theme,scale:1.0,x:((if index%3==1{0.88}else{0.52})-(index/3) as f64*0.25).max(0.05),y:if index%3==2{0.94}else{0.06},enabled:true,city:None,note,countdown,monitor:None,custom_id});
    persist(&app,&next)?; *state.layout.lock().unwrap()=next; sync(&app); Ok(snapshot(&app))
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Patch { monitor: Option<String>, theme: Option<String>, scale: Option<f64>, x: Option<f64>, y: Option<f64>, enabled: Option<bool>, city: Option<City>, note: Option<NoteData>, countdown: Option<CountdownData>, custom_id: Option<String> }
#[tauri::command]
pub async fn update_desktop_widget(app: tauri::AppHandle, id: String, patch: Patch) -> Result<Value,String> {
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let mut next=state.layout.lock().unwrap().clone();let widget=next.widgets.iter_mut().find(|w|w.id==id).ok_or("没有找到这个组件")?;
    if let Some(v)=patch.monitor{widget.monitor=Some(v);}if let Some(v)=patch.theme{widget.theme=v;}if let Some(v)=patch.scale{widget.scale=v;}if let Some(v)=patch.x{widget.x=v;}if let Some(v)=patch.y{widget.y=v;}if let Some(v)=patch.enabled{widget.enabled=v;}if let Some(v)=patch.city{widget.city=Some(v);}if let Some(v)=patch.note{widget.note=Some(v);}if let Some(v)=patch.countdown{widget.countdown=Some(v);}if let Some(v)=patch.custom_id{widget.custom_id=Some(v);}
    persist(&app,&next)?;*state.layout.lock().unwrap()=next;sync(&app);Ok(snapshot(&app))
}
/// A widget can save only its own typed content; no caller-supplied identity or layout patch.
#[tauri::command]
pub async fn save_desktop_widget_content(window:tauri::WebviewWindow,content:WidgetContent)->Result<(),String>{
    let app=window.app_handle();let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let original=own_widget(&window)?;
    if state.editing.load(Ordering::Relaxed){return Err("请先完成组件摆放".into());}
    let mut next=state.layout.lock().unwrap().clone();
    let widget=next.widgets.iter_mut().find(|w|w.id==original.id).ok_or("没有找到这个组件")?;
    match (widget.kind.as_str(),content.note,content.countdown) {
        ("notes",Some(note),None)=>widget.note=Some(note),
        ("countdown",None,Some(countdown))=>widget.countdown=Some(countdown),
        _=>return Err("此组件不能保存这类内容".into())
    }
    persist(app,&next)?;*state.layout.lock().unwrap()=next;sync(app);Ok(())
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
pub async fn move_desktop_widget(window:tauri::WebviewWindow,grab_x:f64,grab_y:f64,commit:bool)->Result<(),String>{
    let app=window.app_handle();let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    if !grab_x.is_finite()||!grab_y.is_finite()||!(0.0..=1.0).contains(&grab_x)||!(0.0..=1.0).contains(&grab_y){return Err("组件拖动位置无效".into());}
    let original=own_widget(&window)?;let point=cursor(app)?;let areas=monitors(app)?;
    let monitor=crate::widget_geometry::at_point(&areas,point.0,point.1).ok_or("没有找到显示器")?;
    let (base_w,base_h)=widget_base_size(app,&original);let width=(base_w*original.scale+16.0)*monitor.scale;let height=(base_h*original.scale+16.0)*monitor.scale;
    let (x,y)=crate::widget_geometry::drag_position(monitor,width,height,point,(grab_x,grab_y));
    let mut next=state.layout.lock().unwrap().clone();let widget=next.widgets.iter_mut().find(|w|w.id==original.id).ok_or("没有找到组件")?;
    widget.x=x;widget.y=y;widget.monitor=Some(monitor.id.clone());let moved=widget.clone();
    if commit{persist(app,&next)?;}*state.layout.lock().unwrap()=next;place(app,&window,&moved)?;
    if commit{sync(app);}else{window.eval(format!("window.dispatchEvent(new CustomEvent('widget-config',{{detail:{}}}));",widget_payload(app,&moved)?)).map_err(|e|e.to_string())?;}Ok(())
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
#[tauri::command]
pub fn list_custom_widgets(app: tauri::AppHandle) -> Result<Value, String> {
    let dir = crate::storage::root(&app)?.join("custom_widgets");
    Ok(json!(crate::custom_widgets::load_all(&dir)))
}
#[tauri::command]
pub fn save_custom_widget(app: tauri::AppHandle, pack: Value, overwrite: Option<bool>) -> Result<Value, String> {
    let dir = crate::storage::root(&app)?.join("custom_widgets");
    let path = crate::custom_widgets::save(&dir, &pack, overwrite.unwrap_or(false))?;
    Ok(json!({ "ok": true, "path": path.to_string_lossy() }))
}
#[tauri::command]
pub fn remove_custom_widget(app: tauri::AppHandle, id: String) -> Result<Value, String> {
    let dir = crate::storage::root(&app)?.join("custom_widgets");
    crate::custom_widgets::remove(&dir, &id)?;
    let state = app.state::<WidgetState>();
    let mut next = state.layout.lock().unwrap().clone();
    let before = next.widgets.len();
    next.widgets.retain(|w| !(w.kind == "custom" && w.custom_id.as_deref() == Some(&id)));
    if next.widgets.len() != before {
        let _ = persist(&app, &next);
        *state.layout.lock().unwrap() = next;
        sync(&app);
    }
    Ok(json!({ "ok": true, "id": id }))
}
pub fn delete_custom_pack(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    remove_custom_widget(app.clone(), id.to_string()).map(|_| ())
}
#[cfg(test)]
mod tests{
    use super::*;
    fn one()->Layout{Layout{version:1,widgets:vec![Widget{id:"abc123".into(),kind:"calendar".into(),theme:"glass".into(),scale:1.0,x:0.8,y:0.2,enabled:true,city:None,note:None,countdown:None,monitor:None,custom_id:None}]}}
    #[test]fn custom_widgets_in_layout_validated(){
        let mut layout=one();layout.widgets[0].kind="custom".into();layout.widgets[0].custom_id=Some("pomodoro-timer".into());
        assert!(validate(&layout).is_ok());
        layout.widgets[0].custom_id=None;assert!(validate(&layout).is_err());
        layout.widgets[0].custom_id=Some("Invalid Id!".into());assert!(validate(&layout).is_err());
    }
    #[test]fn bounds_and_duplicate_ids_are_checked(){let mut layout=one();assert!(validate(&layout).is_ok());layout.widgets.push(layout.widgets[0].clone());assert!(validate(&layout).is_err());layout=one();layout.widgets[0].x=f64::NAN;assert!(validate(&layout).is_err());layout=one();layout.widgets[0].scale=10.;assert!(validate(&layout).is_err());}
    #[test]fn window_cannot_manage_other_widgets_or_wallpapers(){assert_eq!(permitted("widget-abc123","add_desktop_widget"),Some(false));assert_eq!(permitted("widget-abc123","apply_wallpaper"),Some(false));assert_eq!(permitted("widget-abc123","move_desktop_widget"),Some(true));assert_eq!(permitted("media-a","add_desktop_widget"),Some(false));}
    #[test]fn old_or_corrupt_layouts_are_rejected(){let mut layout=one();layout.version=2;assert!(validate(&layout).is_err());let mut layout=one();layout.widgets[0].id="../escape".into();assert!(validate(&layout).is_err());}
    #[test]fn legacy_layouts_load_without_new_content_fields(){
        let layout:Layout=serde_json::from_str(r#"{"version":1,"widgets":[{"id":"old","kind":"calendar","theme":"paper","scale":1.0,"x":0.5,"y":0.2,"enabled":true,"city":null}]}"#).unwrap();
        assert!(validate(&layout).is_ok());assert!(layout.widgets[0].note.is_none());assert!(layout.widgets[0].countdown.is_none());
    }
    #[test]fn typed_content_and_dates_are_validated(){
        let mut layout=one();layout.widgets[0].kind="notes".into();
        layout.widgets[0].note=Some(NoteData{title:"记事".into(),mode:"todo".into(),text:String::new(),items:vec![TodoItem{id:"task".into(),text:"喝水".into(),done:true}]});
        assert!(validate(&layout).is_ok());let item=layout.widgets[0].note.as_ref().unwrap().items[0].clone();layout.widgets[0].note.as_mut().unwrap().items.push(item);assert!(validate(&layout).is_err());
        layout=one();layout.widgets[0].kind="countdown".into();layout.widgets[0].countdown=Some(CountdownData{title:"生日".into(),date:"2026-02-29".into(),mode:"countdown".into(),repeat:"yearly".into()});assert!(validate(&layout).is_err());
        layout.widgets[0].countdown.as_mut().unwrap().date="2024-02-29".into();assert!(validate(&layout).is_ok());
        layout.widgets[0].kind="weather".into();assert!(validate(&layout).is_err());
    }
    #[test]fn content_command_has_no_id_or_layout_fields(){
        assert_eq!(permitted("widget-abc","save_desktop_widget_content"),Some(true));assert_eq!(permitted("media-a","save_desktop_widget_content"),Some(false));
        assert!(serde_json::from_value::<WidgetContent>(json!({"id":"other","note":null})).is_err());
        assert!(serde_json::from_value::<WidgetContent>(json!({"theme":"glass"})).is_err());
    }

}


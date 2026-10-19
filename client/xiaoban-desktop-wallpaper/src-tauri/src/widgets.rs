//! Small, independent desktop windows. Only the control window can alter the collection.
use crate::care_weather::{City, Weather};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fs, io::Read, sync::{atomic::{AtomicBool, Ordering}, Mutex}};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Widget {
    pub id: String, pub kind: String, pub theme: String, pub scale: f64,
    pub x: f64, pub y: f64, pub enabled: bool, pub city: Option<City>,
    #[serde(default="desktop_layer")] pub layer:String,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub note: Option<NoteData>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub monitor: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub countdown: Option<CountdownData>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub living: Option<LivingData>,
}
const MAX_LAYOUT_BYTES: usize = 128 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all="camelCase")]
pub struct TodoItem { pub id:String, pub text:String, pub done:bool, #[serde(default)] pub time:String,
 #[serde(default)] pub date:String, #[serde(default)] pub notify:bool, #[serde(default="repeat_none")] pub repeat:String, #[serde(default)] pub completed_on:String }
fn desktop_layer()->String{"desktop".into()}
fn repeat_none()->String{"none".into()}
fn default_true()->bool{true}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteData { pub title: String, pub mode: String, pub text: String, pub items: Vec<TodoItem> }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountdownData { pub title: String, pub date: String, pub mode: String, pub repeat: String }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ReminderItem {pub id:String,pub title:String,pub at:String,pub done:bool,#[serde(default)] pub end:String,#[serde(default)] pub location:String,
 #[serde(default="default_true")] pub notify:bool, #[serde(default="repeat_none")] pub repeat:String, #[serde(default)] pub completed_on:String }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct LivingData { pub reminders:Vec<ReminderItem>, pub count:u32, pub symbol:String, #[serde(default)] pub symbols:Vec<String>, #[serde(default="default_true")] pub sound:bool }
impl Default for LivingData {fn default()->Self{Self{reminders:vec![],count:0,symbol:"1.000001".into(),symbols:vec![],sound:true}}}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WidgetContent { note: Option<NoteData>, countdown: Option<CountdownData>, living: Option<LivingData> }
fn short_text(value: &str, max: usize) -> bool { !value.trim().is_empty() && value.encode_utf16().count() <= max }
fn valid_identity(value: &str) -> bool { !value.is_empty() && value.len()<=32 && value.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()) }
pub const KINDS: [&str;8] = ["clock","weather","notes","reminders","stocks","woodfish","calendar","countdown"];
fn valid_time(v:&str)->bool {chrono::NaiveTime::parse_from_str(v,"%H:%M").is_ok_and(|t|t.format("%H:%M").to_string()==v)}
fn valid_date(v:&str)->bool {chrono::NaiveDate::parse_from_str(v,"%Y-%m-%d").is_ok_and(|d|d.format("%Y-%m-%d").to_string()==v&&v>="1900-01-01"&&v<="2100-12-31")}
fn valid_schedule(repeat:&str,notify:bool,date:&str,time:&str,completed:&str)->bool {
 ["none","daily","weekdays","weekly"].contains(&repeat)&& (completed.is_empty()||valid_date(completed)) && (!(notify||repeat!="none")||(valid_date(date)&&valid_time(time)))
}
fn valid_stock_symbol(symbol:&str)->bool{
    let Some((market,code))=symbol.split_once('.') else{return false;};
    if market=="116" {return code.len()==5 && code.bytes().all(|b|b.is_ascii_digit());}
    code.len()==6 && code.bytes().all(|b|b.is_ascii_digit()) &&
        ((market=="1" && (code=="000001" || code.starts_with('6'))) ||
         (market=="0" && (code=="399001" || code=="399006" || code.starts_with('0') || code.starts_with('3'))))
}
fn validate_content(widget: &Widget) -> Result<(),String> {
    if let Some(note)=&widget.note {
        if widget.kind!="notes" || !short_text(&note.title,40) || note.mode!="todo" || note.text.encode_utf16().count()>1200 || note.items.len()>12 { return Err("便签内容无效或超过长度限制".into()); }
        let mut ids=std::collections::HashSet::new();
        for item in &note.items { if !valid_identity(&item.id)||!ids.insert(&item.id)||!short_text(&item.text,120)||(!item.time.is_empty()&&!valid_time(&item.time))||(!item.date.is_empty()&&!valid_date(&item.date))||!valid_schedule(&item.repeat,item.notify,&item.date,&item.time,&item.completed_on){return Err("待办提醒配置无效".into());} }

    }
    if let Some(data)=&widget.countdown {
        let valid_date=chrono::NaiveDate::parse_from_str(&data.date,"%Y-%m-%d").is_ok_and(|date|date.format("%Y-%m-%d").to_string()==data.date && data.date.as_str()>="1900-01-01" && data.date.as_str()<="2100-12-31");
        if widget.kind!="countdown" || !short_text(&data.title,40) || !valid_date || !["countdown","elapsed"].contains(&data.mode.as_str()) || !["none","yearly"].contains(&data.repeat.as_str()) || (data.mode=="elapsed"&&data.repeat!="none") {return Err("纪念日配置无效".into());}
    }
    if let Some(data)=&widget.living {
        if !["reminders","stocks","woodfish"].contains(&widget.kind.as_str()) || data.count>9999999 || !valid_stock_symbol(&data.symbol) || data.reminders.len()>12 || data.symbols.len()>8 || (!data.symbols.is_empty()&&(!data.symbols.contains(&data.symbol)||data.symbols.iter().any(|s|!valid_stock_symbol(s))||data.symbols.iter().collect::<std::collections::HashSet<_>>().len()!=data.symbols.len())){return Err("生活组件配置无效".into());}
        let mut ids=std::collections::HashSet::new();
        for item in &data.reminders{
            let date=chrono::NaiveDateTime::parse_from_str(&item.at,"%Y-%m-%dT%H:%M").is_ok_and(|v|v.format("%Y-%m-%dT%H:%M").to_string()==item.at);
            if !valid_identity(&item.id) || !ids.insert(&item.id) || !short_text(&item.title,60) || !date || (!item.end.is_empty()&&(!valid_time(&item.end)||item.end.as_str()<=&item.at[11..])) || item.location.encode_utf16().count()>80 || !valid_schedule(&item.repeat,item.notify,item.at.get(..10).unwrap_or(""),item.at.get(11..).unwrap_or(""),&item.completed_on){return Err("日程记录无效".into());}
        }
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout { pub version: u32, pub widgets: Vec<Widget> }
impl Default for Layout { fn default() -> Self { Self { version: 2, widgets: vec![] } } }
#[derive(Clone)]
struct ResizeSession { original:Widget, anchor:(f64,f64), cursor:(f64,f64), monitor:crate::widget_geometry::MonitorArea }
#[derive(Default)]
pub struct WidgetState {
    menu_target:Mutex<Option<String>>, resizes:Mutex<HashMap<String,ResizeSession>>,
    layout: Mutex<Layout>, editing: AtomicBool,
    operation: tauri::async_runtime::Mutex<()>,
    weather_operation: tauri::async_runtime::Mutex<()>,
    market_operation: tauri::async_runtime::Mutex<()>,
    forecast_operation: tauri::async_runtime::Mutex<()>,
    load_error: Mutex<Option<String>>, errors: Mutex<Vec<String>>,
    weather: Mutex<HashMap<String, Weather>>, attempts: Mutex<HashMap<String, i64>>,
    market: Mutex<HashMap<String,(i64,Value)>>,
    forecasts: Mutex<HashMap<String,(i64,Value)>>,
}
pub fn reminder_rules(app:&tauri::AppHandle,now:chrono::DateTime<chrono::Local>)->Vec<crate::reminders::Rule>{
 let layout=app.state::<WidgetState>().layout.lock().unwrap().clone();crate::widget_schedule::rules(&layout,now)
}
pub fn is_widget(label: &str) -> bool { label.starts_with("widget-") }
pub(crate) fn window_layer(app:&tauri::AppHandle,label:&str)->Option<String>{
    let id=label.strip_prefix("widget-")?;
    let state=app.state::<WidgetState>();let value=state.layout.lock().unwrap().widgets.iter().find(|w|w.id==id).map(|w|w.layer.clone());value
}
#[cfg(target_os="macos")]
pub(crate) fn restore_window_layer(app:&tauri::AppHandle,label:&str){
    if let (Some(layer),Some(window))=(window_layer(app,label),app.get_webview_window(label)){
        let _=crate::macos::set_interactive_layer(&window,&layer);
    }
}

fn own_widget(window: &tauri::WebviewWindow) -> Result<Widget, String> {
    let state=window.app_handle().state::<WidgetState>();
    let target=if window.label()=="widgetmenu"{state.menu_target.lock().unwrap().clone()}else{None};
    let result=state.layout.lock().unwrap().widgets.iter()
        .find(|widget| target.as_ref().map_or_else(||format!("widget-{}",widget.id)==window.label(),|id|id==&widget.id)).cloned().ok_or("没有找到这个桌面组件".into());result
}
pub fn permitted(label: &str, command: &str) -> Option<bool> {
    if label=="widgetmenu" {return Some(["get_widget_settings","update_widget_settings","remove_widget_settings","close_widget_settings","size_widget_settings","search_widget_cities"].contains(&command));}
    if ["get_widget_settings","update_widget_settings","remove_widget_settings","close_widget_settings","size_widget_settings","search_widget_cities","resize_desktop_widget"].contains(&command)&&!is_widget(label){return Some(false);}
    if is_widget(label) { return Some(["resize_desktop_widget","get_desktop_widget", "move_desktop_widget", "show_widget_settings", "get_widget_weather", "get_widget_forecast", "get_widget_air_quality", "get_widget_stock_watchlist", "save_desktop_widget_content", "get_widget_stocks"].contains(&command)); }
    if ["get_desktop_widgets", "add_desktop_widget", "update_desktop_widget", "remove_desktop_widget", "set_widgets_editing", "get_desktop_widget", "move_desktop_widget", "show_widget_settings", "get_widget_weather", "get_widget_forecast", "get_widget_air_quality", "get_widget_stock_watchlist", "get_widget_stocks", "save_desktop_widget_content"].contains(&command) { return Some(label == "control"); }
    None
}
pub fn validate(layout: &Layout) -> Result<(), String> {
    if layout.version != 2 || layout.widgets.len() > 12 { return Err("桌面最多放置 12 个组件".into()); }
    let mut ids = std::collections::HashSet::new();
    for widget in &layout.widgets {
        if widget.id.is_empty() || widget.id.len() > 32 || !widget.id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) || !ids.insert(&widget.id)
            || !KINDS.contains(&widget.kind.as_str())
            || !["desktop","topmost"].contains(&widget.layer.as_str())
            || widget.theme!="ivory"
            || !widget.scale.is_finite() || !(0.5..=1.5).contains(&widget.scale)
            || !widget.x.is_finite() || !(0.0..=1.0).contains(&widget.x)
            || !widget.y.is_finite() || !(0.0..=1.0).contains(&widget.y) { return Err("组件配置无效，原布局已保留".into()); }
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
pub(crate) fn monitors(app:&tauri::AppHandle)->Result<Vec<crate::widget_geometry::MonitorArea>,String>{
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
fn snapshot(app: &tauri::AppHandle) -> Value {
    let state=app.state::<WidgetState>();let layout=state.layout.lock().unwrap().clone();
    let areas=monitors(app).unwrap_or_default();
    let displays:Vec<_>=areas.iter().map(|m|json!({"id":m.id,"name":m.name,"width":m.width/m.scale,"height":m.height/m.scale,"primary":m.primary})).collect();
    let desktop=crate::widget_geometry::resolve(&areas,None).map(|m|json!({"width":m.width/m.scale,"height":m.height/m.scale}));
    let widgets = &layout.widgets;
    json!({"version":2,"widgets":widgets,"editing":state.editing.load(Ordering::Relaxed),"errors":state.errors.lock().unwrap().clone(),"desktop":desktop,"monitors":displays})
}
pub fn base_size(kind:&str)->(f64,f64){match kind {"clock"=>(596.,306.),"weather"=>(326.,306.),"notes"=>(352.,306.),"reminders"=>(480.,332.),"stocks"=>(404.,338.),"woodfish"=>(390.,332.),"calendar"=>(500.,286.),"countdown"=>(790.,286.),_=>(352.,306.)}}
pub fn widget_base_size(_app:&tauri::AppHandle,widget:&Widget)->(f64,f64){base_size(&widget.kind)}
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
    let widget_val = serde_json::to_value(widget).map_err(|e|e.to_string())?;
    Ok(json!({"widget":widget_val,"editing":app.state::<WidgetState>().editing.load(Ordering::Relaxed),"screenX":x,"screenY":y,"displayScale":display_scale}))
}
fn place(app: &tauri::AppHandle, window: &tauri::WebviewWindow, widget: &Widget) -> Result<(), String> {
    let (x,y,width,height,dpi)=geometry(app,widget)?;
    #[cfg(windows)] {
        if widget.layer == "desktop" {
            return crate::windows::place_widget(window,tauri::PhysicalPosition::new(x.round() as i32,y.round() as i32),width.round() as i32,height.round() as i32);
        }
        return crate::windows::place_interactive(window,x,y,width,height);
    }
    #[cfg(target_os="macos")] {let _=dpi;return crate::macos::place_widget(window,x,y,width,height);}
    #[cfg(not(any(windows,target_os="macos")))] {window.set_size(tauri::LogicalSize::new(width/dpi,height/dpi)).and_then(|_|window.set_position(tauri::PhysicalPosition::new(x,y))).map_err(|e|e.to_string())}
}
pub fn sync(app: &tauri::AppHandle) {
    let state = app.state::<WidgetState>(); let layout = state.layout.lock().unwrap().clone(); let mut errors = vec![];
    state.resizes.lock().unwrap().retain(|id,_|layout.widgets.iter().any(|w|&w.id==id));
    let enabled: Vec<_> = layout.widgets.iter().filter(|w| w.enabled && !crate::games::is_active(app)).collect();
    for (label, window) in app.webview_windows() {
        if is_widget(&label) && !enabled.iter().any(|w| label == format!("widget-{}",w.id)) { let _ = window.destroy(); }
    }
    for widget in enabled {
        let result = (|| -> Result<(), String> {
            let label = format!("widget-{}",widget.id);
            if let Some(window) = app.get_webview_window(&label) {
                crate::window_control::apply_layer(&window,&widget.layer)?;
                place(app,&window,widget)?;
                window.eval(format!("window.dispatchEvent(new CustomEvent('widget-config',{{detail:{}}}));",widget_payload(app,widget)?)).map_err(|e| e.to_string())?;
                return Ok(());
            }
            let (x,y,width,height,dpi) = geometry(app,widget)?;
            let builder = WebviewWindowBuilder::new(app,&label,WebviewUrl::App(format!("index.html?scene=widget&id={}",widget.id).into()))
                .title("小伴 · 桌面组件").decorations(false).shadow(false).transparent(true).resizable(false).visible(false)
                .skip_taskbar(true).always_on_top(widget.layer=="topmost").focused(false).position(x/dpi,y/dpi).inner_size(width/dpi,height/dpi)
                .initialization_script(format!("window.__XIAOBAN_WIDGET__={};",widget_payload(app,widget)?));
            // Keep the native parent in Tao's own state as well as the final HWND styles.
            #[cfg(windows)] let builder = {
                let builder=if widget.layer=="desktop"{builder.parent_raw(::windows::Win32::Foundation::HWND(crate::windows::widget_parent()? as _))}else{builder};
                builder.data_directory(crate::storage::webview(&crate::storage::root(app)?))
            };
            #[cfg(target_os="macos")] let builder = builder.incognito(true);
            let window = builder.build().map_err(|e| e.to_string())?;
            let configured = (|| -> Result<(),String> {
                #[cfg(windows)] if widget.layer=="desktop"{crate::windows::attach_widget(&window)?;}
                #[cfg(target_os="macos")] crate::macos::configure_scene(&window,true,false)?;
                place(app,&window,widget)?; window.show().map_err(|e| e.to_string())?;
                // Show can reconcile Tao's window styles; apply the desktop child frame last.
                #[cfg(windows)] if widget.layer=="desktop"{crate::windows::attach_widget(&window)?;}
                crate::window_control::apply_layer(&window,&widget.layer)?;
                place(app,&window,widget)?; Ok(())
            })();
            if let Err(error) = configured { let _=window.destroy(); return Err(error); }
            Ok(())
        })();
        if let Err(error) = result { errors.push(format!("{}：{error}",widget.kind)); }
    }
    *state.errors.lock().unwrap() = errors;
    let menu_id=state.menu_target.lock().unwrap().clone();
    if let Some(menu)=app.get_webview_window("widgetmenu"){
        if menu_id.as_ref().is_some_and(|id|layout.widgets.iter().any(|w|&w.id==id))&&!crate::games::is_active(app){
            if let Ok(data)=menu_payload(app){let _=menu.eval(format!("window.dispatchEvent(new CustomEvent('widget-menu-config',{{detail:{data}}}));"));}
        }else{let _=menu.hide();*state.menu_target.lock().unwrap()=None;}
    }
    if let Some(control) = app.get_webview_window("control") { let _ = control.eval(format!("window.dispatchEvent(new CustomEvent('widgets-changed',{{detail:{}}}));",snapshot(app))); }
}
pub fn start(app: &tauri::AppHandle) {
    let result = (|| -> Result<Layout,String> {
        let path=crate::storage::root(app)?.join("config/widgets.json");
        match fs::metadata(&path) { Err(e) if e.kind()==std::io::ErrorKind::NotFound => return Ok(Layout::default()),Err(e)=>return Err(e.to_string()),Ok(m) if m.len()>MAX_LAYOUT_BYTES as u64=>return Err("组件布局过大，原文件已保留".into()),_=>{} }
        let bytes=fs::read(&path).map_err(|e|e.to_string())?;
        let value:Value=serde_json::from_slice(&bytes).map_err(|e|format!("组件布局无法读取，原文件已保留：{e}"))?;
        if value.get("version").and_then(Value::as_u64)==Some(1) {
            // Retire every v1 instance without deleting the user's previous records.
            let backup=path.with_file_name(format!("widgets-v1-{}.bak",chrono::Utc::now().timestamp_millis()));
            if backup.exists(){return Err("旧组件备份已存在，请重试".into());}
            fs::rename(&path,backup).map_err(|e|format!("旧组件备份失败：{e}"))?;
            return Ok(Layout::default());
        }
        let layout:Layout=serde_json::from_value(value).map_err(|e|format!("组件布局无法读取，原文件已保留：{e}"))?;
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
pub async fn add_desktop_widget(app: tauri::AppHandle, kind: String, layer: Option<String>) -> Result<Value,String> {
    let layer=layer.unwrap_or_else(desktop_layer);
    if !["desktop","topmost"].contains(&layer.as_str()) { return Err("组件层级只能是 desktop 或 topmost".into()); }
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let mut next=state.layout.lock().unwrap().clone(); if next.widgets.len()>=12{return Err("桌面最多放置 12 个组件".into());}
    let mut random=[0u8;8];getrandom::fill(&mut random).map_err(|e|e.to_string())?;
    let index=next.widgets.len();
    let note=if kind=="notes"{Some(NoteData{title:"今日待办".into(),mode:"todo".into(),text:String::new(),items:vec![]})}else{None};
    let countdown=if kind=="countdown"{Some(CountdownData{title:"值得期待的日子".into(),date:chrono::Local::now().format("%Y-%m-%d").to_string(),mode:"countdown".into(),repeat:"none".into()})}else{None};
    let living=if ["reminders","stocks","woodfish"].contains(&kind.as_str()){let mut d=LivingData::default();if kind=="stocks"{d.symbol="116.00700".into();}Some(d)}else{None};
    let theme="ivory".to_string();
    next.widgets.push(Widget{id:random.iter().map(|b|format!("{b:02x}")).collect(),kind,theme,layer,scale:0.5,x:((if index%3==1{0.88}else{0.52})-(index/3) as f64*0.25).max(0.05),y:if index%3==2{0.94}else{0.06},enabled:true,city:None,note,countdown,living,monitor:None});
    persist(&app,&next)?; *state.layout.lock().unwrap()=next; sync(&app); Ok(snapshot(&app))
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Patch { layer:Option<String>, monitor: Option<String>, theme: Option<String>, scale: Option<f64>, x: Option<f64>, y: Option<f64>, enabled: Option<bool>, city: Option<City>, note: Option<NoteData>, countdown: Option<CountdownData>, living: Option<LivingData>, }
#[tauri::command]
pub async fn update_desktop_widget(app: tauri::AppHandle, id: String, patch: Patch) -> Result<Value,String> {
    let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let mut next=state.layout.lock().unwrap().clone();let widget=next.widgets.iter_mut().find(|w|w.id==id).ok_or("没有找到这个组件")?;
    let old_layer=widget.layer.clone();if let Some(v)=patch.layer{widget.layer=v;}
    if let Some(v)=patch.monitor{widget.monitor=Some(v);}if let Some(v)=patch.theme{widget.theme=v;}if let Some(v)=patch.scale{widget.scale=v;}if let Some(v)=patch.x{widget.x=v;}if let Some(v)=patch.y{widget.y=v;}if let Some(v)=patch.enabled{widget.enabled=v;}if let Some(v)=patch.city{widget.city=Some(v);}if let Some(v)=patch.note{widget.note=Some(v);}if let Some(v)=patch.countdown{widget.countdown=Some(v);}if let Some(v)=patch.living{widget.living=Some(v);}
    let layer_changed=old_layer!=widget.layer;
    persist(&app,&next)?;*state.layout.lock().unwrap()=next;
    if layer_changed{state.resizes.lock().unwrap().remove(&id);
        // Rebuild the Windows window with the correct Tao parent, not just HWND styles.
        #[cfg(windows)] if let Some(window)=app.get_webview_window(&format!("widget-{id}")){
            let handle=app.clone();
            window.on_window_event(move|event|{if matches!(event,tauri::WindowEvent::Destroyed){
                // Wait until Tao finishes unregistering the old label before rebuilding.
                let next=handle.clone();let _=handle.run_on_main_thread(move||{tauri::async_runtime::spawn_blocking(move||sync(&next));});
            }});
            window.destroy().map_err(|e|e.to_string())?;return Ok(snapshot(&app));
        }
    }
    sync(&app);Ok(snapshot(&app))
}
/// A widget can save only its own typed content; no caller-supplied identity or layout patch.
#[tauri::command]
pub async fn save_desktop_widget_content(window:tauri::WebviewWindow,content:WidgetContent)->Result<(),String>{
    let app=window.app_handle();let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let original=own_widget(&window)?;
    if state.editing.load(Ordering::Relaxed){return Err("请先完成组件摆放".into());}
    let mut next=state.layout.lock().unwrap().clone();
    let widget=next.widgets.iter_mut().find(|w|w.id==original.id).ok_or("没有找到这个组件")?;
    match widget.kind.as_str() {
        "notes" if content.note.is_some() && content.countdown.is_none() && content.living.is_none() => widget.note=content.note,
        "countdown" if content.countdown.is_some() && content.note.is_none() && content.living.is_none() => widget.countdown=content.countdown,
        "reminders" | "stocks" | "woodfish" if content.living.is_some() && content.note.is_none() && content.countdown.is_none() => widget.living=content.living,
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
fn menu_payload(app:&tauri::AppHandle)->Result<Value,String>{
    let state=app.state::<WidgetState>();let id=state.menu_target.lock().unwrap().clone().ok_or("组件菜单已关闭")?;
    let widget=state.layout.lock().unwrap().widgets.iter().find(|w|w.id==id).cloned().ok_or("组件已移除")?;
    let areas=monitors(app)?;let monitor=crate::widget_geometry::resolve(&areas,widget.monitor.as_deref()).ok_or("没有找到显示器")?;
    Ok(json!({"widget":widget,"monitors":snapshot(app)["monitors"],"appearance":crate::settings(app).appearance,"maxHeight":(monitor.height/monitor.scale-24.).clamp(160.,560.)}))
}
fn menu_only(window:&tauri::WebviewWindow)->Result<(),String>{if window.label()=="widgetmenu"{Ok(())}else{Err("只能从组件菜单修改此组件".into())}}
fn place_menu(app:&tauri::AppHandle,menu:&tauri::WebviewWindow,height:f64)->Result<(),String>{
    let widget=own_widget(menu)?;let areas=monitors(app)?;
    let monitor=crate::widget_geometry::resolve(&areas,widget.monitor.as_deref()).ok_or("没有找到显示器")?;
    let (x,y,w,h,_)=geometry(app,&widget)?;let height=height.clamp(160.,560.).min((monitor.height/monitor.scale-24.).max(160.));
    let (px,py)=crate::widget_geometry::menu_position(monitor,(x,y,w,h),(340.,height));
    #[cfg(target_os="macos")] menu.set_size(tauri::LogicalSize::new(340.,height)).map_err(|e|e.to_string())?;
    #[cfg(not(target_os="macos"))] menu.set_size(tauri::PhysicalSize::new((340.*monitor.scale).round() as u32,(height*monitor.scale).round() as u32)).map_err(|e|e.to_string())?;
    // AppKit placement uses points while other platforms expose physical pixels.
    #[cfg(target_os="macos")] {menu.set_position(tauri::LogicalPosition::new(px,py)).map_err(|e|e.to_string())?;}
    #[cfg(not(target_os="macos"))] {menu.set_position(tauri::PhysicalPosition::new(px,py)).map_err(|e|e.to_string())?;}
    Ok(())
}
#[tauri::command]
pub async fn show_widget_settings(window:tauri::WebviewWindow)->Result<(),String>{
    let widget=own_widget(&window)?;let app=window.app_handle();let state=app.state::<WidgetState>();
    *state.menu_target.lock().unwrap()=Some(widget.id);
    let data=menu_payload(app)?;
    let menu=if let Some(menu)=app.get_webview_window("widgetmenu"){
        menu.eval(format!("window.dispatchEvent(new CustomEvent('widget-menu-open',{{detail:{data}}}));")).map_err(|e|e.to_string())?;menu
    }else{
        let builder=WebviewWindowBuilder::new(app,"widgetmenu",WebviewUrl::App("index.html?scene=widget-settings".into()))
            .title("组件设置").decorations(false).shadow(false).transparent(true).resizable(false).visible(false).skip_taskbar(true).always_on_top(true).focused(false).inner_size(340.,340.)
            .initialization_script(format!("window.__XIAOBAN_WIDGET_MENU__={data};"));
        #[cfg(target_os="macos")] let builder=builder.incognito(true);
        #[cfg(windows)] let builder=builder.data_directory(crate::storage::webview(&crate::storage::root(app)?));
        let menu=builder.build().map_err(|e|e.to_string())?;
        let own=menu.clone();menu.on_window_event(move|event|{if matches!(event,tauri::WindowEvent::Focused(false)){let _=own.hide();}});menu
    };
    place_menu(app,&menu,340.)?;menu.show().map_err(|e|e.to_string())?;menu.set_focus().map_err(|e|e.to_string())?;Ok(())
}
#[tauri::command]
pub fn get_widget_settings(window:tauri::WebviewWindow)->Result<Value,String>{menu_only(&window)?;menu_payload(window.app_handle())}
#[tauri::command]
pub async fn update_widget_settings(window:tauri::WebviewWindow,id:String,patch:Patch)->Result<Value,String>{
    menu_only(&window)?;let widget=own_widget(&window)?;
    if widget.id!=id{return Err("组件菜单已切换，请重新操作".into());}
    update_desktop_widget(window.app_handle().clone(),widget.id,patch).await?;menu_payload(window.app_handle())
}
#[tauri::command]
pub async fn remove_widget_settings(window:tauri::WebviewWindow,id:String)->Result<(),String>{
    menu_only(&window)?;let widget=own_widget(&window)?;if widget.id!=id{return Err("组件菜单已切换，请重新操作".into());}remove_desktop_widget(window.app_handle().clone(),widget.id).await?;Ok(())
}
#[tauri::command]
pub fn close_widget_settings(window:tauri::WebviewWindow)->Result<(),String>{menu_only(&window)?;window.hide().map_err(|e|e.to_string())}
#[tauri::command]
pub fn size_widget_settings(window:tauri::WebviewWindow,height:f64)->Result<(),String>{menu_only(&window)?;if !height.is_finite(){return Err("菜单高度无效".into());}place_menu(window.app_handle(),&window,height)}
#[tauri::command]
pub async fn search_widget_cities(window:tauri::WebviewWindow,query:String)->Result<Vec<City>,String>{
    menu_only(&window)?;if own_widget(&window)?.kind!="weather"{return Err("此组件不支持城市查询".into());}
    let app=window.app_handle();let state=app.state::<WidgetState>();let now=chrono::Utc::now().timestamp();
    {let mut attempts=state.attempts.lock().unwrap();if attempts.get("menu-city").is_some_and(|at|now-*at<2){return Err("请稍后再查询城市".into());}attempts.insert("menu-city".into(),now);}
    let source=crate::reminders::widget_weather_source(app);
    tauri::async_runtime::spawn_blocking(move||crate::care_weather::search(&source,&query)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn resize_desktop_widget(window:tauri::WebviewWindow,phase:String)->Result<(),String>{
    let app=window.app_handle();let state=app.state::<WidgetState>();let _operation=state.operation.lock().await;
    let original=own_widget(&window)?;
    if phase=="begin"{
        let (x,y,_,_,_)=geometry(app,&original)?;let areas=monitors(app)?;
        let monitor=crate::widget_geometry::resolve(&areas,original.monitor.as_deref()).ok_or("没有找到显示器")?.clone();
        let point=cursor(app)?;state.resizes.lock().unwrap().insert(original.id.clone(),ResizeSession{original,anchor:(x,y),cursor:point,monitor});return Ok(());
    }
    if !["move","end","cancel"].contains(&phase.as_str()){return Err("无效的缩放操作".into());}
    let session=state.resizes.lock().unwrap().get(&original.id).cloned().ok_or("请先开始缩放")?;
    let mut next=state.layout.lock().unwrap().clone();let widget=next.widgets.iter_mut().find(|w|w.id==original.id).ok_or("组件已移除")?;
    if phase=="cancel"{widget.scale=session.original.scale;widget.x=session.original.x;widget.y=session.original.y;widget.monitor=session.original.monitor.clone();}
    else{
        let point=cursor(app)?;let base=widget_base_size(app,widget);
        widget.scale=crate::widget_geometry::corner_scale(&session.monitor,base,session.original.scale,session.anchor,(point.0-session.cursor.0,point.1-session.cursor.1));
        let width=(base.0*widget.scale+16.)*session.monitor.scale;let height=(base.1*widget.scale+16.)*session.monitor.scale;
        let (x,y)=crate::widget_geometry::drag_position(&session.monitor,width,height,session.anchor,(0.,0.));widget.x=x;widget.y=y;widget.monitor=Some(session.monitor.id.clone());
    }
    let resized=widget.clone();
    if phase=="end"{if let Err(error)=persist(app,&next){
        let mut restored=state.layout.lock().unwrap();if let Some(w)=restored.widgets.iter_mut().find(|w|w.id==original.id){w.scale=session.original.scale;w.x=session.original.x;w.y=session.original.y;w.monitor=session.original.monitor;}
        state.resizes.lock().unwrap().remove(&original.id);drop(restored);sync(app);return Err(error);
    }}
    *state.layout.lock().unwrap()=next;
    if phase=="move"{place(app,&window,&resized)?;window.eval(format!("window.dispatchEvent(new CustomEvent('widget-config',{{detail:{}}}));",widget_payload(app,&resized)?)).map_err(|e|e.to_string())?;}
    else{state.resizes.lock().unwrap().remove(&original.id);sync(app);}Ok(())
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

/// A narrow, read-only market-data adapter; no arbitrary host or symbol supplied by a webview.
/// The public endpoint is best-effort, with no SLA or verified redistribution licence.
#[tauri::command]
pub async fn get_widget_forecast(window:tauri::WebviewWindow)->Result<Value,String>{
    let widget=own_widget(&window)?;if widget.kind!="weather"{return Err("此组件不能读取天气预报".into());}
    let city=widget.city.ok_or("请先在组件设置里选择城市")?;
    let app=window.app_handle().clone();let state=app.state::<WidgetState>();
    let _lock=state.forecast_operation.lock().await;let now=chrono::Utc::now().timestamp();
    let source=crate::reminders::widget_weather_source(&app);
    let key=format!("{}:{}:{}",city.latitude,city.longitude,serde_json::to_string(&source).map_err(|e|e.to_string())?);
    if let Some((when,value))=state.forecasts.lock().unwrap().get(&key).filter(|(when,_)|(0..1800).contains(&(now-*when))).cloned(){return Ok(value);}
    let result=tauri::async_runtime::spawn_blocking(move||crate::care_weather::forecast(&source,&city)).await.map_err(|e|e.to_string())??;
    let mut cache=state.forecasts.lock().unwrap();if cache.len()>48{cache.clear();}cache.insert(key,(now,result.clone()));
    Ok(result)
}

#[tauri::command]
pub async fn get_widget_air_quality(window:tauri::WebviewWindow)->Result<Value,String>{
 let widget=own_widget(&window)?;if widget.kind!="weather"{return Err("此组件不能读取空气质量".into());}
 let city=widget.city.ok_or("请先选择城市")?;let app=window.app_handle().clone();let state=app.state::<WidgetState>();
 let _lock=state.forecast_operation.lock().await;let now=chrono::Utc::now().timestamp();let key=format!("air:{}:{}",city.latitude,city.longitude);
 if let Some((_,value))=state.forecasts.lock().unwrap().get(&key).filter(|(at,_)|(0..1800).contains(&(now-*at))).cloned(){return Ok(value);}
 let value=tauri::async_runtime::spawn_blocking(move||crate::care_weather::air_quality(&city,now)).await.map_err(|e|e.to_string())??;
 let mut cache=state.forecasts.lock().unwrap();if cache.len()>48{cache.clear();}cache.insert(key,(now,value.clone()));Ok(value)
}
fn market_number(value:Option<&Value>)->Option<f64>{value.and_then(|v|v.as_f64().or_else(||v.as_str().and_then(|s|s.parse::<f64>().ok()))).filter(|v|v.is_finite())}
fn watchlist_values(value:&Value,symbols:&[String],now:i64)->Result<Value,String>{
 let rows=value.pointer("/data/diff").and_then(Value::as_array).ok_or("自选行情暂不可用")?;
 let quotes:Vec<Value>=rows.iter().filter_map(|r|{
  let code=r.get("f12")?.as_str()?;let market=r.get("f13")?.as_u64()?;let symbol=format!("{market}.{code}");
  if !symbols.contains(&symbol){return None;}
  let price=market_number(r.get("f2"))?;let percent=market_number(r.get("f3"))?;let change=market_number(r.get("f4"))?;
  if price<=0.{return None;}
  Some(json!({"symbol":symbol,"name":r.get("f14").and_then(Value::as_str).unwrap_or(code),"price":price,"change":change,"percent":percent,"updatedAt":r.get("f124").and_then(Value::as_i64)}))
 }).collect();
 if quotes.is_empty(){return Err("自选行情没有有效价格，请稍后重试".into());}
 Ok(json!({"quotes":quotes,"fetchedAt":now,"source":"东方财富公开数据"}))
}
#[tauri::command]
pub async fn get_widget_stock_watchlist(window:tauri::WebviewWindow)->Result<Value,String>{
 let widget=own_widget(&window)?;if widget.kind!="stocks"{return Err("此组件不能读取自选行情".into());}
 let d=widget.living.unwrap_or_default();let symbols=if d.symbols.is_empty(){vec![d.symbol]}else{d.symbols};
 let app=window.app_handle().clone();let state=app.state::<WidgetState>();let _lock=state.market_operation.lock().await;
 let now=chrono::Utc::now().timestamp();let key=format!("list:{}",symbols.join(","));
 if let Some((_,value))=state.market.lock().unwrap().get(&key).filter(|(at,_)|(0..60).contains(&(now-*at))).cloned(){return Ok(value);}
 let result=tauri::async_runtime::spawn_blocking(move||->Result<Value,String>{
  let mut url=url::Url::parse("https://push2.eastmoney.com/api/qt/ulist.np/get").map_err(|e|e.to_string())?;
  url.query_pairs_mut().append_pair("secids",&symbols.join(",")).append_pair("fields","f2,f3,f4,f12,f13,f14,f124").append_pair("fltt","2");
  let client=reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(12)).redirect(reqwest::redirect::Policy::none()).user_agent("Mozilla/5.0 XiaoBan Desktop").build().map_err(|e|e.to_string())?;
  let response=client.get(url).header("Referer","https://quote.eastmoney.com/").send().map_err(|e|format!("自选行情连接失败：{e}"))?.error_for_status().map_err(|e|e.to_string())?;
  let mut bytes=Vec::new();response.take(256*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;if bytes.len()>256*1024{return Err("自选行情响应过大".into());}
  let value:Value=serde_json::from_slice(&bytes).map_err(|_|"自选行情格式无效")?;watchlist_values(&value,&symbols,now)
 }).await.map_err(|e|e.to_string())??;
 let mut cache=state.market.lock().unwrap();if cache.len()>20{cache.clear();}cache.insert(key,(now,result.clone()));Ok(result)
}
#[tauri::command]
pub async fn get_widget_stocks(window: tauri::WebviewWindow, period:String, symbol:Option<String>)->Result<Value,String>{
    let widget=own_widget(&window)?;
    if widget.kind!="stocks"{return Err("此组件不读取股票行情".into());}
    if !["1D","5D","1M","1Y"].contains(&period.as_str()){return Err("不支持的行情周期".into());}
    let living=widget.living.unwrap_or_default();
    let symbol=symbol.unwrap_or_else(||living.symbol.clone());
    if symbol!=living.symbol&&!living.symbols.contains(&symbol){return Err("证券不在此组件的自选列表中".into());}
    if !valid_stock_symbol(&symbol){return Err("未支持的证券代码".into());}
    let app=window.app_handle();let state=app.state::<WidgetState>();
    let _operation=state.market_operation.lock().await;
    let key=format!("{symbol}/{period}");let now=chrono::Utc::now().timestamp();
    if let Some((when,value))=state.market.lock().unwrap().get(&key).filter(|(when,_)|(0..60).contains(&(now-*when))).cloned(){return Ok(value);}
    let stock=symbol.clone();let range=period.clone();let historical=period=="1M"||period=="1Y";let days=if period=="1D"{1}else if period=="5D"{5}else if period=="1M"{23}else{251};
    let response=tauri::async_runtime::spawn_blocking(move||->Result<Value,String>{
        if let Ok(quote)=crate::widget_market::fetch(&stock,&range){return Ok(quote);}
        let url=if historical{format!("https://push2his.eastmoney.com/api/qt/stock/kline/get?secid={stock}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56&klt=101&fqt=1&end=20500101&lmt={days}")}
        else{format!("https://push2his.eastmoney.com/api/qt/stock/trends2/get?secid={stock}&fields1=f1,f2,f3,f4,f5,f6,f7,f8&fields2=f51,f52,f53,f54,f55&ndays={days}&iscr=0&ut=fa5fd1943c7b386f172d6893dbfba10b")};
        let client=reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(12))
            .redirect(reqwest::redirect::Policy::none()).user_agent("Mozilla/5.0 XiaoBan Desktop").build().map_err(|e|e.to_string())?;
        let mut res=client.get(url).send().map_err(|e|format!("行情连接失败：{e}"))?.error_for_status().map_err(|e|format!("行情服务错误：{e}"))?;
        let mut bytes=Vec::new();res.take(512*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
        if bytes.len()>512*1024{return Err("行情响应过大".into());}
        let json:Value=serde_json::from_slice(&bytes).map_err(|_|"行情服务返回格式无效")?;
        let data=json.get("data").filter(|d|!d.is_null()).ok_or("行情服务暂未返回数据")?;
        let lines=data.get(if historical{"klines"}else{"trends"}).and_then(Value::as_array).ok_or("行情序列不可用")?;
        let values:Vec<(String,f64)>=lines.iter().filter_map(|v|{
            let row=v.as_str()?;let parts:Vec<_>=row.split(',').collect();
            let value=parts.get(2)?.parse::<f64>().ok()?;
            if !value.is_finite()||value<=0.0{return None;}
            Some((parts.first()?.to_string(),value))
        }).collect();
        if values.is_empty(){return Err("可用行情点数不足".into());}
        let last=values.last().ok_or("行情为空")?;let price=last.1;
        let prev=data.get("prePrice").or_else(||data.get("preClose")).and_then(|v|v.as_f64().or_else(||v.as_str().and_then(|s|s.parse::<f64>().ok()))).unwrap_or_else(||if historical&&values.len()>1{values[values.len()-2].1}else{0.0});
        if prev<=0.0{return Err("行情缺少昨收盘价".into());}
        let step=((values.len()+119)/120).max(1);
        let mut points:Vec<f64>=values.iter().step_by(step).map(|(_,value)|*value).collect();
        if points.last()!=Some(&price){points.push(price);}
        let change=price-prev;let pct=100.0*change/prev;
        Ok(json!({"price":price,"change":change,"percent":pct,"updatedAt":last.0,"points":points,
            "source":"东方财富公开数据","name":data.get("name").and_then(Value::as_str).unwrap_or("指数")}))
    }).await.map_err(|e|e.to_string())??;
    let mut response=response;
    response["symbol"]=json!(symbol);response["period"]=json!(period);
    let mut cache=state.market.lock().unwrap();if cache.len()>20{cache.clear();}cache.insert(key,(now,response.clone()));
    Ok(response)
}

#[cfg(test)]
mod tests {
 #[test] fn watchlist_prices_ignore_missing_or_unrequested_data(){
  let data=json!({"data":{"diff":[{"f12":"00700","f13":116,"f2":309.6,"f3":1.58,"f4":4.8,"f14":"腾讯控股"},{"f12":"000001","f13":1,"f2":"-"}]}});
  let symbols=vec!["116.00700".to_string(),"1.000001".to_string()];let value=watchlist_values(&data,&symbols,1000).unwrap();assert_eq!(value["quotes"].as_array().unwrap().len(),1);assert_eq!(value["quotes"][0]["symbol"],"116.00700");assert!(watchlist_values(&data,&["0.399006".into()],1000).is_err());
 }
 use super::*;
 fn one()->Layout{serde_json::from_value(json!({"version":2,"widgets":[{"id":"clock1","kind":"clock","theme":"ivory","scale":1.,"x":0.1,"y":0.1,"enabled":true,"city":null}]})).unwrap()}
 #[test] fn layer_defaults_and_persistence_are_validated(){
  let mut l=one();assert_eq!(l.widgets[0].layer,"desktop");l.widgets[0].layer="topmost".into();assert!(validate(&l).is_ok());
  let saved=serde_json::to_vec(&l).unwrap();let restored:Layout=serde_json::from_slice(&saved).unwrap();assert_eq!(restored.widgets[0].layer,"topmost");
  l.widgets[0].layer="normal".into();assert!(validate(&l).is_err());
 }
 #[test] fn a_new_widget_keeps_the_layer_it_was_added_with(){
  // Adding writes the chosen layer straight into the widget; the frontend no longer needs a second call.
  for layer in ["desktop","topmost"]{let mut l=one();l.widgets[0].layer=layer.into();assert!(validate(&l).is_ok());let restored:Layout=serde_json::from_slice(&serde_json::to_vec(&l).unwrap()).unwrap();assert_eq!(restored.widgets[0].layer,layer);}
 }
 #[test] fn old_kinds_and_materials_are_rejected(){for kind in ["fortune","flower","custom"]{let mut l=one();l.widgets[0].kind=kind.into();assert!(validate(&l).is_err());}let mut l=one();l.widgets[0].theme="glass".into();assert!(validate(&l).is_err());l=one();l.version=1;assert!(validate(&l).is_err());}
 #[test] fn only_eight_reference_components_are_available(){for k in KINDS{let mut l=one();l.widgets[0].kind=k.into();assert!(validate(&l).is_ok());assert!(base_size(k).0>300.);}}
 #[test] fn hk_and_a_share_codes_remain_constrained(){for s in ["116.00700","1.000001","0.399006","1.600519"]{assert!(valid_stock_symbol(s));}for s in ["116.00700&x=1","116.abcde","https://example.com","3.123456"]{assert!(!valid_stock_symbol(s));}}
 #[test] fn date_and_schedule_validation(){assert!(valid_time("09:30"));assert!(!valid_time("25:00"));let mut l=one();l.widgets[0].kind="reminders".into();l.widgets[0].living=Some(LivingData{count:0,symbol:"1.000001".into(),symbols:vec![],sound:true,reminders:vec![ReminderItem{id:"one".into(),title:"评审".into(),at:"2026-10-19T10:00".into(),done:false,end:"09:00".into(),location:"A302".into(),notify:true,repeat:"none".into(),completed_on:String::new()}]});assert!(validate(&l).is_err());l.widgets[0].living.as_mut().unwrap().reminders[0].end="11:00".into();assert!(validate(&l).is_ok());}
 #[test] fn menu_and_resize_commands_are_scoped_to_their_windows(){
  assert_eq!(permitted("widgetmenu","update_widget_settings"),Some(true));assert_eq!(permitted("widgetmenu","update_desktop_widget"),Some(false));
  assert_eq!(permitted("widgetmenu","resize_desktop_widget"),Some(false));assert_eq!(permitted("widget-clock1","resize_desktop_widget"),Some(true));
  assert_eq!(permitted("control","resize_desktop_widget"),Some(false));assert_eq!(permitted("wallpaper","update_widget_settings"),Some(false));
 }
 #[test] fn widget_cannot_modify_collection(){assert_eq!(permitted("widget-clock1","add_desktop_widget"),Some(false));assert_eq!(permitted("widget-clock1","save_desktop_widget_content"),Some(true));}
}

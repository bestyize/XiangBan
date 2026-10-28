//! Trusted project-scoped design windows; previews retain their opaque sandbox origin.
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{sync::{Arc,Mutex}};
use tauri::{Emitter,Manager,WebviewUrl,WebviewWindowBuilder};

pub fn label(id:&str)->String { format!("design-window-{}",id.strip_prefix("design-").unwrap_or("invalid")) }
pub fn project_id(label:&str)->Option<String>{
    let tail=label.strip_prefix("design-window-")?;
    if tail.len()!=24||!tail.bytes().all(|c|c.is_ascii_hexdigit()){return None}
    Some(format!("design-{tail}"))
}
pub fn guard(window:&tauri::WebviewWindow,id:&str)->Result<(),String>{
    if window.label()=="control"||project_id(window.label()).as_deref()==Some(id){Ok(())}else{Err("此设计窗口只能访问当前设计稿".into())}
}
pub fn permitted(label:&str,command:&str)->Option<bool>{
    if command.starts_with("design_"){return Some(label=="control"||project_id(label).is_some()&&[
        "design_read","design_save","design_view_read","design_view_save","design_draft_save","design_draft_clear","design_history","design_restore","design_export","design_copy","design_window_return","design_window_close","design_window_request","design_window_fullscreen"
    ].contains(&command))}
    if project_id(label).is_some(){return Some(command=="get_appearance")}
    None
}
mod placement;

pub(crate) fn keep_visible(window:&tauri::WebviewWindow, fallback:Option<tauri::Monitor>)->Result<(),String>{
    if window.is_maximized().unwrap_or(false)||window.is_fullscreen().unwrap_or(false){return Ok(())}
    let monitor=window.current_monitor().map_err(|e|e.to_string())?.or(fallback);
    let Some(monitor)=monitor else{return Ok(())};
    let a=monitor.work_area();let p=window.outer_position().map_err(|e|e.to_string())?;let outer=window.outer_size().map_err(|e|e.to_string())?;let inner=window.inner_size().map_err(|e|e.to_string())?;
    let rect=placement::Rect{x:p.x,y:p.y,w:outer.width,h:outer.height};let fitted=placement::fit(rect,placement::Rect{x:a.position.x,y:a.position.y,w:a.size.width,h:a.size.height});
    if fitted.w!=rect.w||fitted.h!=rect.h{window.set_size(tauri::PhysicalSize::new(fitted.w.saturating_sub(outer.width.saturating_sub(inner.width)).max(1),fitted.h.saturating_sub(outer.height.saturating_sub(inner.height)).max(1))).map_err(|e|e.to_string())?;}
    if fitted.x!=rect.x||fitted.y!=rect.y{window.set_position(tauri::PhysicalPosition::new(fitted.x,fitted.y)).map_err(|e|e.to_string())?;}Ok(())
}

static OPEN_LOCK:Mutex<()>=Mutex::new(());
#[derive(Clone,Default,Serialize,Deserialize)]
struct Placement { x:i32,y:i32,w:u32,h:u32,maximized:bool }
#[tauri::command]
pub fn design_window_status(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String)->Result<bool,String>{
    guard(&window,&id)?;crate::design::store(&app)?.load(&id)?;Ok(app.get_webview_window(&label(&id)).is_some())
}
#[tauri::command]
pub async fn design_window_open(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String)->Result<(),String>{
    let _open=OPEN_LOCK.lock().map_err(|_|"设计窗口锁不可用")?;
    guard(&window,&id)?;let store=crate::design::store(&app)?;let project=store.load(&id)?;let name=label(&id);
    let fallback=window.current_monitor().map_err(|e|e.to_string())?.or(app.primary_monitor().map_err(|e|e.to_string())?);
    if let Some(existing)=app.get_webview_window(&name){let _=existing.unminimize();keep_visible(&existing,fallback)?;existing.show().map_err(|e|e.to_string())?;return existing.set_focus().map_err(|e|e.to_string())}
    let placement=store.read_view(&id,"window.json").ok().flatten().and_then(|v|serde_json::from_value::<Placement>(v).ok()).unwrap_or_default();
    let appearance=crate::settings(&app).appearance;
    let theme=match appearance.as_str(){"dark"=>Some(tauri::Theme::Dark),"light"=>Some(tauri::Theme::Light),_=>None};
    let mut builder=WebviewWindowBuilder::new(&app,&name,WebviewUrl::App(format!("index.html?scene=design&id={id}").into()))
        .title(format!("{} · 小伴设计",project.title)).inner_size(1280.,840.).min_inner_size(640.,480.).resizable(true).decorations(true).theme(theme).visible(false);
    if let Some(m)=&fallback{let a=m.work_area();let f=m.scale_factor();let w=(a.size.width as f64/f-32.).max(1.);let h=(a.size.height as f64/f-64.).max(1.);builder=builder.inner_size(1280f64.min(w),840f64.min(h)).min_inner_size(640f64.min(w),480f64.min(h));}
    let monitor=app.available_monitors().map_err(|e|e.to_string())?.into_iter().find(|m|{
        let p=m.position();let s=m.size();placement.x>=p.x&&placement.y>=p.y&&i64::from(placement.x)<i64::from(p.x)+i64::from(s.width)-100&&i64::from(placement.y)<i64::from(p.y)+i64::from(s.height)-80
    });
    if let Some(m)=monitor.filter(|_|placement.w>=640&&placement.h>=480){
        let f=m.scale_factor();let a=m.work_area();let w=(a.size.width as f64/f-32.).max(1.);let h=(a.size.height as f64/f-64.).max(1.);builder=builder.min_inner_size(640f64.min(w),480f64.min(h)).position(placement.x as f64/f,placement.y as f64/f).inner_size((placement.w as f64/f).min(w),(placement.h as f64/f).min(h));
    }else{builder=builder.center();}
    let child=builder.build().map_err(|e|e.to_string())?;
    #[cfg(target_os="macos")]
    let _=crate::macos::apply_control_appearance(&child,&appearance);
    keep_visible(&child,fallback)?;
    let own=child.clone();let remembered=Arc::new(Mutex::new(placement.clone()));
    child.on_window_event(move|event|{
        if let Ok(mut state)=remembered.lock(){
            if !own.is_maximized().unwrap_or(false)&&!own.is_fullscreen().unwrap_or(false){
                if let Ok(p)=own.outer_position(){state.x=p.x;state.y=p.y;}
                if let Ok(s)=own.inner_size(){state.w=s.width;state.h=s.height;}
            }
            if matches!(event,tauri::WindowEvent::CloseRequested{..}){
                state.maximized=own.is_maximized().unwrap_or(false);
                if let Ok(value)=serde_json::to_value(&*state){let _=store.save_view(&id,"window.json",&value);}
            }
        }
        if matches!(event,tauri::WindowEvent::Destroyed){let _=own.app_handle().emit_to("control","design-window-closed",json!({"id":id}));}
    });
    if placement.maximized{child.maximize().map_err(|e|e.to_string())?;}
    child.show().map_err(|e|e.to_string())?;
    child.set_focus().map_err(|e|e.to_string())
}
#[tauri::command]
pub fn design_window_close(window:tauri::WebviewWindow,id:String)->Result<(),String>{
    if project_id(window.label()).as_deref()!=Some(id.as_str()){return Err("只能关闭当前设计窗口".into())}window.close().map_err(|e|e.to_string())
}
#[tauri::command]
pub fn design_window_return(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String)->Result<(),String>{
    guard(&window,&id)?;let p=crate::design::store(&app)?.load(&id)?;crate::present_control(&app)?;
    app.emit_to("control","design-open",json!({"id":id,"session":p.session_ids.last().unwrap_or(&p.session_id)})).map_err(|e|e.to_string())?;
    if project_id(window.label()).is_some(){window.close().map_err(|e|e.to_string())?;}Ok(())
}
#[tauri::command]
pub fn design_window_request(app:tauri::AppHandle,window:tauri::WebviewWindow,id:String,message:String,session:String)->Result<Value,String>{
    guard(&window,&id)?;let p=crate::design::store(&app)?.load(&id)?;
    if session!=p.session_id&&!p.session_ids.contains(&session){return Err("请从关联会话继续修改".into())}
    if message.len()>30000{return Err("批注过长".into())}crate::present_control(&app)?;
    app.emit_to("control","design-request",json!({"id":id,"session":session,"message":message})).map_err(|e|e.to_string())?;Ok(json!({"sent":true}))
}
#[cfg(test)]mod tests{use super::*;#[test]fn labels_cannot_expand_project_permissions(){assert_eq!(project_id("design-window-012345678901234567890123").as_deref(),Some("design-012345678901234567890123"));assert!(project_id("design-window-../../private").is_none());assert_eq!(permitted("design-window-012345678901234567890123","chat_send"),Some(false));assert_eq!(permitted("wallpaper","design_read"),Some(false));}}

#[tauri::command]
pub fn design_window_fullscreen(window:tauri::WebviewWindow,id:String)->Result<(),String>{
    if project_id(window.label()).as_deref()!=Some(id.as_str()){return Err("只能操作当前设计窗口".into())}window.set_fullscreen(!window.is_fullscreen().map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}

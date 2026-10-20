//! Transparent assistant headroom must not obstruct the application underneath it.
use std::{sync::Mutex, time::Duration};
use tauri::{Manager, WebviewWindow};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Region { left: f64, bottom: f64, width: f64, height: f64, radius: f64 }
impl Region {
    fn valid(&self) -> bool {
        [self.left,self.bottom,self.width,self.height,self.radius].iter().all(|v|v.is_finite()&&v.abs()<=4096.0)
            && self.width>0.0 && self.height>0.0 && self.radius>=0.0
    }
    fn contains(&self,x:f64,y:f64,window_height:f64)->bool {
        let top=window_height-self.bottom-self.height;
        let (x,y)=(x-self.left,y-top);
        if x<0.0 || y<0.0 || x>self.width || y>self.height { return false; }
        let radius=self.radius.min(self.width/2.0).min(self.height/2.0);
        let dx=x-x.clamp(radius,self.width-radius);let dy=y-y.clamp(radius,self.height-radius);
        dx*dx+dy*dy<=radius*radius
    }
}
#[derive(Default)]
pub struct HitState { regions: Mutex<Vec<Region>>, ignored: Mutex<Option<bool>> }

#[tauri::command]
pub fn assistant_set_regions(window:WebviewWindow,regions:Vec<Region>)->Result<(),String>{
    if window.label()!=crate::float_assistant::LABEL {return Err("只有超级小伴可以更新鼠标区域".into());}
    if regions.len()>8 || regions.iter().any(|r|!r.valid()) {return Err("鼠标区域无效".into());}
    *window.state::<HitState>().regions.lock().unwrap()=regions;
    Ok(())
}

#[cfg(windows)]
fn pointer(window:&WebviewWindow)->Result<(f64,f64,f64,bool),String>{
    let point=crate::windows::cursor_position().ok_or("无法读取鼠标位置")?;
    let pos=window.inner_position().map_err(|e|e.to_string())?;
    let height=window.inner_size().map_err(|e|e.to_string())?.height;
    let scale=window.scale_factor().map_err(|e|e.to_string())?;
    Ok(((point.0-pos.x as f64)/scale,(point.1-pos.y as f64)/scale,height as f64/scale,crate::windows::left_button_down()))
}
#[cfg(target_os="macos")]
fn pointer(window:&WebviewWindow)->Result<(f64,f64,f64,bool),String>{crate::macos::assistant_pointer(window)}
#[cfg(not(any(windows,target_os="macos")))]
fn pointer(_window:&WebviewWindow)->Result<(f64,f64,f64,bool),String>{Err("平台不支持".into())}

pub fn start(app:&tauri::AppHandle){
    let app=app.clone();
    std::thread::spawn(move||loop{
        std::thread::sleep(Duration::from_millis(16));
        if !crate::float_assistant::pointer_active(&app) {std::thread::sleep(Duration::from_millis(80));continue;}
        let Some(window)=app.get_webview_window(crate::float_assistant::LABEL) else {continue;};
        if !window.is_visible().unwrap_or(false){continue;}
        let Ok((x,y,height,down))=pointer(&window) else {continue;};
        let state=app.state::<HitState>();
        let hit=state.regions.lock().unwrap().iter().any(|r|r.contains(x,y,height));
        let ignored=!hit;
        // Do not release an assistant press, selection or native drag before mouse-up.
        // Entering a visible surface stays possible during an external image drag.
        if down && ignored {continue;}
        let mut previous=state.ignored.lock().unwrap();
        if *previous!=Some(ignored) && window.set_ignore_cursor_events(ignored).is_ok(){*previous=Some(ignored);}
    });
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn reserved_headroom_and_large_transparent_margins_are_not_hit(){
        let r=Region{left:8.,bottom:8.,width:464.,height:76.,radius:34.};
        assert!(!r.contains(240.,20.,156.));
        assert!(r.contains(240.,110.,156.));
        assert!(!r.contains(0.,110.,156.));
        assert!(!r.contains(8.,72.,156.));
    }
    #[test] fn bottom_anchor_survives_window_fit_and_retains_gaps(){
        let r=Region{left:8.,bottom:8.,width:464.,height:76.,radius:34.};
        assert!(r.contains(240.,110.,156.));assert!(r.contains(240.,354.,400.));
        assert!(!r.contains(240.,200.,400.));
    }
    #[test] fn rejects_unbounded_regions(){
        let mut r=Region{left:0.,bottom:0.,width:10.,height:10.,radius:2.};assert!(r.valid());
        r.width=f64::NAN;assert!(!r.valid());r.width=5000.;assert!(!r.valid());
    }
}

//! Per-monitor widget placement. Windows uses global physical pixels;
//! macOS uses global AppKit points with a top-left origin.
#[derive(Clone, Debug, serde::Serialize)]
pub struct MonitorArea {
    pub id: String, pub name: String, pub x: f64, pub y: f64,
    pub width: f64, pub height: f64, pub scale: f64, pub primary: bool,
}
pub fn resolve<'a>(areas: &'a [MonitorArea], saved: Option<&str>) -> Option<&'a MonitorArea> {
    saved.and_then(|id|areas.iter().find(|m|m.id==id))
        .or_else(||areas.iter().find(|m|m.primary)).or_else(||areas.first())
}
pub fn at_point(areas: &[MonitorArea], x: f64, y: f64) -> Option<&MonitorArea> {
    areas.iter().find(|m|x>=m.x&&x<m.x+m.width&&y>=m.y&&y<m.y+m.height)
        .or_else(||areas.iter().min_by(|a,b|distance(a,x,y).total_cmp(&distance(b,x,y))))
}
fn distance(m: &MonitorArea, x: f64, y: f64) -> f64 {
    let dx=x-x.clamp(m.x,m.x+m.width);let dy=y-y.clamp(m.y,m.y+m.height);dx*dx+dy*dy
}
pub fn spans(m: &MonitorArea, width: f64, height: f64) -> (f64,f64,f64) {
    let top=(32.0*m.scale).min((m.height-height).max(0.0));
    let bottom=(56.0*m.scale).min((m.height-height-top).max(0.0));
    ((m.width-width).max(0.0),(m.height-height-top-bottom).max(0.0),top)
}
pub fn position(m: &MonitorArea, width: f64, height: f64, x: f64, y: f64) -> (f64,f64) {
    let (span_x,span_y,top)=spans(m,width,height);(m.x+x*span_x,m.y+top+y*span_y)
}
pub fn drag_position(m: &MonitorArea, width: f64, height: f64, cursor: (f64,f64), grab: (f64,f64)) -> (f64,f64) {
    let (span_x,span_y,top)=spans(m,width,height);
    let x=if span_x>0.0{((cursor.0-grab.0*width-m.x)/span_x).clamp(0.0,1.0)}else{0.0};
    let y=if span_y>0.0{((cursor.1-grab.1*height-m.y-top)/span_y).clamp(0.0,1.0)}else{0.0};
    (x,y)
}
pub fn from_bottom_left(primary_height:f64,x:f64,y:f64,height:f64)->(f64,f64){(x,primary_height-y-height)}
#[cfg(test)]
mod tests {
    use super::*;
    fn monitor(id:&str,x:f64,y:f64,width:f64,height:f64,scale:f64,primary:bool)->MonitorArea{MonitorArea{id:id.into(),name:id.into(),x,y,width,height,scale,primary}}
    #[test]fn a_new_card_keeps_its_share_of_the_screen(){
        assert_eq!(default_scale(1440.,900.),0.5);
        assert_eq!(default_scale(2560.,1440.),0.8);
        assert_eq!(default_scale(3840.,2160.),1.2);
        assert_eq!(default_scale(1280.,800.),0.44,"a small screen shrinks the starting size");
        assert_eq!(default_scale(0.,0.),0.5);
        assert_eq!(default_scale(f64::NAN,900.),0.5);
    }
    #[test]fn targets_right_left_and_above_displays(){
        let screens=vec![monitor("main",0.,0.,1920.,1080.,1.,true),monitor("left",-1280.,0.,1280.,1024.,1.,false),monitor("right",1920.,0.,2560.,1440.,2.,false),monitor("above",0.,-900.,1600.,900.,1.,false)];
        assert_eq!(at_point(&screens,-100.,300.).unwrap().id,"left");assert_eq!(at_point(&screens,2200.,300.).unwrap().id,"right");assert_eq!(at_point(&screens,100.,-100.).unwrap().id,"above");
        assert_eq!(at_point(&screens,1920.,100.).unwrap().id,"right");
    }
    #[test]fn restores_saved_screen_and_falls_back_without_rewriting_it(){
        let screens=vec![monitor("main",0.,0.,1920.,1080.,1.,true),monitor("second",1920.,0.,1920.,1080.,1.,false)];
        assert_eq!(resolve(&screens,Some("second")).unwrap().id,"second");assert_eq!(resolve(&screens[..1],Some("second")).unwrap().id,"main");assert_eq!(resolve(&screens,None).unwrap().id,"main");assert!(resolve(&[],None).is_none());
    }
    #[test]fn mixed_dpi_drag_preserves_grab_point_and_round_trips(){
        let screen=monitor("retina",1920.,0.,2560.,1440.,2.,false);let (w,h)=(632.,752.);let cursor=(2600.,500.);let grab=(0.25,0.1);
        let (x,y)=drag_position(&screen,w,h,cursor,grab);let (px,py)=position(&screen,w,h,x,y);
        assert!((px+w*grab.0-cursor.0).abs()<0.001);assert!((py+h*grab.1-cursor.1).abs()<0.001);
    }
    #[test]fn small_screens_and_monitor_gaps_remain_finite(){
        let screen=monitor("small",-500.,0.,200.,150.,1.,true);assert_eq!(drag_position(&screen,600.,700.,(-100.,400.),(0.5,0.5)),(0.,0.));
        let screens=vec![screen,monitor("right",1000.,0.,800.,600.,1.,false)];assert_eq!(at_point(&screens,900.,100.).unwrap().id,"right");
    }
    #[test]fn appkit_conversion_handles_screens_above_and_below(){
        assert_eq!(from_bottom_left(900.,-1280.,0.,1024.),(-1280.,-124.));assert_eq!(from_bottom_left(900.,0.,900.,1080.),(0.,-1080.));assert_eq!(from_bottom_left(900.,0.,-768.,768.),(0.,900.));
    }
}


/// The component library is drawn for a 1440×900 desktop. A new card keeps roughly the same
/// share of the screen elsewhere, instead of always starting at 50% and looking tiny on a large
/// or high-resolution display. Sizes are in logical points/pixels, so Retina scaling cancels out.
pub fn default_scale(width: f64, height: f64) -> f64 {
    const REFERENCE_W: f64 = 1440.0;
    const REFERENCE_H: f64 = 900.0;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return 0.5;
    }
    let fit = (width / REFERENCE_W).min(height / REFERENCE_H);
    ((0.5 * fit) * 100.0).round() / 100.0
}
/// A corner resize projects cursor movement onto the original aspect ratio.
/// Keep the top-left anchor fixed, and stop at the monitor edge when possible.
pub fn corner_scale(m:&MonitorArea,base:(f64,f64),original:f64,anchor:(f64,f64),delta:(f64,f64))->f64{
    let proposed=original+(delta.0*base.0+delta.1*base.1)/(m.scale*(base.0*base.0+base.1*base.1));
    let max_x=((m.x+m.width-anchor.0)/m.scale-16.)/base.0;
    let max_y=((m.y+m.height-56.*m.scale-anchor.1)/m.scale-16.)/base.1;
    proposed.clamp(0.5,max_x.min(max_y).min(1.5).max(0.5))
}
/// A 340px menu fits beside the card, flips left at the edge, and stays on-screen.
pub fn menu_position(m:&MonitorArea,card:(f64,f64,f64,f64),size:(f64,f64))->(f64,f64){
    let gap=8.*m.scale;let (w,h)=(size.0*m.scale,size.1*m.scale);
    let right=card.0+card.2+gap;let left=card.0-w-gap;
    let x=if right+w<=m.x+m.width{right}else{left};
    (x.clamp(m.x+gap,(m.x+m.width-w-gap).max(m.x+gap)),card.1.clamp(m.y+gap,(m.y+m.height-h-gap).max(m.y+gap)))
}
#[cfg(test)]mod interaction_tests{
 use super::*;
 fn screen()->MonitorArea{MonitorArea{id:"main".into(),name:"main".into(),x:0.,y:0.,width:1920.,height:1080.,scale:2.,primary:true}}
 #[test]fn corner_resize_scales_proportionally_at_mixed_dpi(){let m=screen();assert!((corner_scale(&m,(400.,300.),0.5,(100.,100.),(400.,300.))-1.).abs()<0.001);assert_eq!(corner_scale(&m,(400.,300.),0.5,(100.,100.),(-1000.,-1000.)),0.5);}
 #[test]fn corner_resize_stops_at_monitor_edge(){let m=screen();let s=corner_scale(&m,(400.,300.),0.5,(1400.,100.),(10000.,10000.));assert!(1400.+(400.*s+16.)*m.scale<=1920.+0.001);}
 #[test]fn menu_flips_left_and_clamps_at_bottom(){let m=screen();let p=menu_position(&m,(1500.,800.,300.,200.),(340.,300.));assert!(p.0<1500.);assert!(p.1+600.<=1080.);}
}

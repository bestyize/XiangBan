//! Physical coordinates throughout: supports negative monitor origins and mixed DPI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect { pub x: i32, pub y: i32, pub w: u32, pub h: u32 }
pub fn fit(rect: Rect, area: Rect) -> Rect {
    let w = rect.w.max(1).min(area.w.max(1));
    let h = rect.h.max(1).min(area.h.max(1));
    let x = i64::from(rect.x).clamp(i64::from(area.x), i64::from(area.x) + i64::from(area.w.saturating_sub(w)));
    let y = i64::from(rect.y).clamp(i64::from(area.y), i64::from(area.y) + i64::from(area.h.saturating_sub(h)));
    Rect { x: x as i32, y: y as i32, w, h }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn right_edge_restores_the_entire_window() {
        assert_eq!(fit(Rect{x:1840,y:850,w:1280,h:840},Rect{x:0,y:24,w:1920,h:1016}),Rect{x:640,y:200,w:1280,h:840});
    }
    #[test] fn negative_monitor_and_dock_are_respected() {
        assert_eq!(fit(Rect{x:-200,y:-50,w:1600,h:1200},Rect{x:-1440,y:30,w:1440,h:800}),Rect{x:-1440,y:30,w:1440,h:800});
    }
    #[test] fn physical_retina_bounds_are_not_rescaled_twice() {
        let r=Rect{x:4400,y:100,w:2600,h:1700};let a=Rect{x:2880,y:48,w:2880,h:1752};
        assert_eq!(fit(r,a),Rect{x:3160,y:100,w:2600,h:1700});
    }
}

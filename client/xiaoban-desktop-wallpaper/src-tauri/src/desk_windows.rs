//! What a desktop-layer game may learn about the screen: where other applications' windows are and where the
//! pointer is. Rectangles and numbers only; no titles, no owners, no pixels.
//!
//! The operating-system readers live in `macos.rs` and `windows.rs`. They hand over raw rectangles in CSS pixels
//! (points on macOS), and the rules here decide which of them a game ever sees.

use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    fn clip_to(&self, bounds: &Rect) -> Option<Rect> {
        let left = self.x.max(bounds.x);
        let top = self.y.max(bounds.y);
        let right = (self.x + self.w).min(bounds.x + bounds.w);
        let bottom = (self.y + self.h).min(bounds.y + bounds.h);
        (right > left && bottom > top).then(|| Rect::new(left, top, right - left, bottom - top))
    }
}

/// Where the pointer is, and whether the primary button is held, both from the screen's top-left corner.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Pointer {
    pub x: f64,
    pub y: f64,
    pub down: bool,
}

/// One window as the system reports it, in the same absolute units as the screen rectangle.
#[derive(Clone, Copy, Debug)]
pub struct Raw {
    pub rect: Rect,
    /// Belongs to this app, such as the desktop layer itself or the pet.
    pub ours: bool,
    /// An ordinary application window, as opposed to a menu bar, the dock, a tooltip or a desktop element.
    pub ordinary: bool,
}

pub const MAX_WINDOWS: usize = 40;
pub const MAX_REGIONS: usize = 32;
const MIN_WIDTH: f64 = 80.0;
const MIN_HEIGHT: f64 = 40.0;

/// The windows worth standing on, front to back, cut to the screen and moved so (0, 0) is its top-left corner.
pub fn visible(raw: &[Raw], screen: Rect) -> Vec<Rect> {
    raw.iter()
        .filter(|window| !window.ours && window.ordinary && window.rect.w >= MIN_WIDTH && window.rect.h >= MIN_HEIGHT)
        .filter_map(|window| window.rect.clip_to(&screen))
        .map(|rect| Rect::new((rect.x - screen.x).round(), (rect.y - screen.y).round(), rect.w.round(), rect.h.round()))
        .take(MAX_WINDOWS)
        .collect()
}

pub fn pointer_on(screen: Rect, x: f64, y: f64, down: bool) -> Pointer {
    Pointer { x: (x - screen.x).round(), y: (y - screen.y).round(), down }
}

/// Reads the rectangles a game asked to receive the mouse in. Anything malformed is dropped, the rest is cut to the
/// screen, and there are never more than [`MAX_REGIONS`].
pub fn regions_from(value: &Value, screen_width: f64, screen_height: f64) -> Vec<Rect> {
    let screen = Rect::new(0.0, 0.0, screen_width, screen_height);
    let Some(list) = value.as_array() else { return Vec::new() };
    list.iter()
        .filter_map(|item| {
            let number = |key: &str| item.get(key).and_then(Value::as_f64).filter(|n| n.is_finite());
            Rect::new(number("x")?, number("y")?, number("w")?, number("h")?).clip_to(&screen)
        })
        .take(MAX_REGIONS)
        .collect()
}

/// Whether the pointer is over a spot the game wants to handle, which decides if the desktop layer takes the mouse.
pub fn over_region(regions: &[Rect], pointer: &Pointer) -> bool {
    regions.iter().any(|region| region.contains(pointer.x, pointer.y))
}

/// What the game is sent each tick. A capability the game did not ask for is simply missing.
pub fn frame(windows: &[Rect], pointer: &Pointer, capabilities: &[String]) -> Value {
    let mut out = serde_json::Map::new();
    if capabilities.iter().any(|c| c == "windows") {
        out.insert("windows".into(), serde_json::to_value(windows).unwrap_or(Value::Null));
    }
    if capabilities.iter().any(|c| c == "pointer") {
        out.insert("pointer".into(), serde_json::to_value(pointer).unwrap_or(Value::Null));
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SCREEN: Rect = Rect { x: 0.0, y: 0.0, w: 1440.0, h: 900.0 };
    fn raw(x: f64, y: f64, w: f64, h: f64) -> Raw {
        Raw { rect: Rect::new(x, y, w, h), ours: false, ordinary: true }
    }

    #[test]
    fn only_real_application_windows_are_listed_front_first() {
        let windows = [
            raw(100.0, 120.0, 800.0, 600.0),
            Raw { ours: true, ..raw(0.0, 0.0, 1440.0, 900.0) },
            Raw { ordinary: false, ..raw(0.0, 0.0, 1440.0, 25.0) },
            raw(10.0, 10.0, 60.0, 600.0),
            raw(10.0, 10.0, 600.0, 30.0),
            raw(500.0, 300.0, 400.0, 300.0),
        ];
        assert_eq!(visible(&windows, SCREEN), vec![Rect::new(100.0, 120.0, 800.0, 600.0), Rect::new(500.0, 300.0, 400.0, 300.0)]);
    }

    #[test]
    fn windows_are_cut_to_the_screen_and_measured_from_its_corner() {
        let screen = Rect::new(-1920.0, 0.0, 1920.0, 1080.0);
        let windows = [raw(-2000.0, 100.0, 500.0, 400.0), raw(100.0, 100.0, 500.0, 400.0), raw(-1000.0, -50.0, 300.0, 200.0)];
        assert_eq!(
            visible(&windows, screen),
            vec![Rect::new(0.0, 100.0, 420.0, 400.0), Rect::new(920.0, 0.0, 300.0, 150.0)],
            "the second window is on another display and the first sticks out the left edge"
        );
    }

    #[test]
    fn the_list_is_capped() {
        let many: Vec<Raw> = (0..100).map(|i| raw(i as f64, 0.0, 200.0, 200.0)).collect();
        assert_eq!(visible(&many, SCREEN).len(), MAX_WINDOWS);
    }

    #[test]
    fn the_pointer_is_reported_from_the_screen_corner() {
        let screen = Rect::new(-1920.0, 0.0, 1920.0, 1080.0);
        assert_eq!(pointer_on(screen, -1000.4, 300.6, true), Pointer { x: 920.0, y: 301.0, down: true });
    }

    #[test]
    fn regions_are_checked_cut_to_the_screen_and_capped() {
        let value = json!([
            {"x": 10, "y": 20, "w": 100, "h": 50},
            {"x": -30, "y": 0, "w": 100, "h": 50},
            {"x": 1400, "y": 880, "w": 200, "h": 200},
            {"x": 5, "y": 5, "w": 0, "h": 10},
            {"x": 5, "y": 5, "w": -3, "h": 10},
            {"x": "5", "y": 5, "w": 10, "h": 10},
            {"x": 5, "y": 5, "w": 10},
            {"x": 5000, "y": 5, "w": 10, "h": 10},
            "nope",
            null
        ]);
        assert_eq!(
            regions_from(&value, 1440.0, 900.0),
            vec![Rect::new(10.0, 20.0, 100.0, 50.0), Rect::new(0.0, 0.0, 70.0, 50.0), Rect::new(1400.0, 880.0, 40.0, 20.0)]
        );
        assert!(regions_from(&json!({"x": 1}), 1440.0, 900.0).is_empty());
        let many: Vec<Value> = (0..100).map(|i| json!({"x": i, "y": 0, "w": 10, "h": 10})).collect();
        assert_eq!(regions_from(&Value::Array(many), 1440.0, 900.0).len(), MAX_REGIONS);
    }

    #[test]
    fn the_desktop_layer_takes_the_mouse_only_over_the_games_own_regions() {
        let regions = [Rect::new(100.0, 100.0, 50.0, 50.0)];
        let at = |x, y| Pointer { x, y, down: false };
        assert!(over_region(&regions, &at(100.0, 100.0)));
        assert!(over_region(&regions, &at(149.0, 149.0)));
        assert!(!over_region(&regions, &at(150.0, 120.0)));
        assert!(!over_region(&[], &at(120.0, 120.0)));
    }

    #[test]
    fn a_frame_carries_only_what_the_game_asked_for() {
        let windows = [Rect::new(1.0, 2.0, 300.0, 200.0)];
        let pointer = Pointer { x: 5.0, y: 6.0, down: false };
        let caps = |list: &[&str]| list.iter().map(|c| c.to_string()).collect::<Vec<_>>();
        assert_eq!(frame(&windows, &pointer, &caps(&[])), json!({}));
        assert_eq!(frame(&windows, &pointer, &caps(&["pointer"])), json!({"pointer": {"x": 5.0, "y": 6.0, "down": false}}));
        let both = frame(&windows, &pointer, &caps(&["windows", "pointer"]));
        assert_eq!(both["windows"], json!([{"x": 1.0, "y": 2.0, "w": 300.0, "h": 200.0}]));
        assert!(both.get("pointer").is_some());
    }
}

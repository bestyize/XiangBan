//! AppKit content views may have either origin orientation. Coordinates returned
//! by AppKit are already logical points; dividing by the Retina scale again
//! would make gaze and hit testing incorrect on an Apple display.
pub fn content_point_to_css(
    x: f64,
    y: f64,
    origin_x: f64,
    origin_y: f64,
    height: f64,
    flipped: bool,
) -> (f64, f64) {
    (
        x - origin_x,
        if flipped {
            y - origin_y
        } else {
            origin_y + height - y
        },
    )
}

pub fn is_desktop_level(level: i64, desktop: i64, icons: i64) -> bool {
    (desktop..=icons).contains(&level)
}

/// The window levels the scenes sit at, as the platform reports them (they are not the same numbers everywhere).
#[derive(Clone, Copy)]
pub struct Levels {
    pub desktop: i64,
    pub icons: i64,
    /// Above the "always on top" tier that other apps' pinned windows use.
    pub floating: i64,
}

/// Where a scene's window goes. Everything that is part of the wallpaper stays beneath every application window; the
/// companion and the desktop widgets go just over the desktop icons so they can still be clicked and dragged, and the
/// wallpaper content itself and the effects stay under the icons. Floating windows go above the other apps.
pub fn scene_level(label: &str, widget: bool, wallpaper: bool, effects_label: &str, levels: Levels) -> i64 {
    if !wallpaper {
        return levels.floating;
    }
    if widget || label == "pet" {
        return levels.icons + 1;
    }
    if label == effects_label || label.strip_prefix("effects-").is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())) {
        return levels.desktop + 2;
    }
    levels.desktop + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVELS: Levels = Levels { desktop: -2147483623, icons: -2147483603, floating: 25 };

    #[test]
    fn a_companion_blended_into_the_wallpaper_sits_under_every_application_window() {
        let level = scene_level("pet", false, true, "effects", LEVELS);
        assert!(level > LEVELS.icons, "above the desktop icons, so it can be clicked");
        assert!(level < 0, "below level 0, where every ordinary application window is");
        assert_eq!(level, scene_level("widget-1", true, true, "effects", LEVELS));
    }

    #[test]
    fn floating_scenes_go_above_other_apps_and_wallpaper_layers_stay_under_the_icons() {
        assert_eq!(scene_level("pet", false, false, "effects", LEVELS), 25);
        assert_eq!(scene_level("assistant", false, false, "effects", LEVELS), 25);
        assert_eq!(scene_level("media-a", false, true, "effects", LEVELS), LEVELS.desktop + 1);
        assert_eq!(scene_level("effects", false, true, "effects", LEVELS), LEVELS.desktop + 2);
        assert!(is_desktop_level(scene_level("media-a", false, true, "effects", LEVELS), LEVELS.desktop, LEVELS.icons));
    }

    #[test]
    fn appkit_bottom_left_maps_to_browser_top_left() {
        assert_eq!(
            content_point_to_css(80.0, 900.0, 0.0, 0.0, 900.0, false),
            (80.0, 0.0)
        );
        assert_eq!(
            content_point_to_css(80.0, 0.0, 0.0, 0.0, 900.0, false),
            (80.0, 900.0)
        );
    }

    #[test]
    fn flipped_retina_view_keeps_logical_points_and_bounds_origin() {
        assert_eq!(
            content_point_to_css(240.0, 160.0, 20.0, 10.0, 600.0, true),
            (220.0, 150.0)
        );
        assert_eq!(
            content_point_to_css(240.0, 160.0, 20.0, 10.0, 600.0, false),
            (220.0, 450.0)
        );
    }

    #[test]
    fn mouse_outside_the_window_is_not_clamped_into_the_character() {
        assert_eq!(
            content_point_to_css(-400.0, 1100.0, 0.0, 0.0, 900.0, false),
            (-400.0, -200.0)
        );
    }

    #[test]
    fn finder_folder_and_dock_windows_do_not_count_as_desktop_clicks() {
        let desktop = -2147483623;
        let icons = desktop + 20;
        assert!(is_desktop_level(desktop + 1, desktop, icons));
        assert!(is_desktop_level(icons, desktop, icons));
        assert!(!is_desktop_level(0, desktop, icons));
        assert!(!is_desktop_level(20, desktop, icons));
        assert!(!is_desktop_level(desktop - 1, desktop, icons));
    }
}

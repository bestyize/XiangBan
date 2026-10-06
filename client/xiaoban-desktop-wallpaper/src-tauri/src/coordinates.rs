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

#[cfg(test)]
mod tests {
    use super::*;

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

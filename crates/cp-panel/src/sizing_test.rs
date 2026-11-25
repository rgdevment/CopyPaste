use super::*;

#[test]
fn a_size_that_fits_is_kept() {
    assert_eq!(
        fitted((480.0, 620.0), 1.0, (1920.0, 1032.0)),
        (480.0, 620.0)
    );
}

fn area(width: f64, height: f64) -> crate::placing::Area {
    crate::placing::Area {
        left: 0.0,
        top: 0.0,
        right: width,
        bottom: height,
    }
}

#[test]
fn a_large_panel_is_clipped_on_a_small_laptop() {
    let room = room_of(area(1366.0, 728.0), 1.0);
    assert_eq!(fitted((560.0, 800.0), 1.0, room), (560.0, 712.0));
}

#[test]
fn a_tall_panel_takes_the_whole_work_height_less_the_edges() {
    assert_eq!(fitted((480.0, 0.0), 1.0, (1920.0, 1032.0)), (480.0, 1016.0));
    assert_eq!(fitted((480.0, 0.0), 1.0, (1366.0, 768.0)), (480.0, 752.0));
}

#[test]
fn the_zoom_multiplies_the_wanted_size_then_the_room_clamps_it() {
    let (width, height) = fitted((480.0, 620.0), 1.16, (1920.0, 1032.0));
    assert!((width - 480.0).abs() < 1e-3, "{width}");
    assert!((height - 620.0).abs() < 1e-3, "{height}");
    let (width, height) = fitted((560.0, 800.0), 1.16, (1366.0, 728.0));
    assert!((width - 560.0).abs() < 1e-3, "{width}");
    assert!((height - 712.0 / 1.16).abs() < 1e-3, "{height}");
}

#[test]
fn a_narrow_screen_clamps_the_width() {
    let (width, height) = fitted((480.0, 620.0), 1.0, (400.0, 1032.0));
    assert_eq!((width, height), (384.0, 620.0));
}

#[test]
fn a_zoom_that_is_not_positive_counts_as_one() {
    assert_eq!(
        fitted((480.0, 620.0), 0.0, (1920.0, 1032.0)),
        (480.0, 620.0)
    );
    assert_eq!(
        fitted((480.0, 620.0), -2.0, (1920.0, 400.0)),
        (480.0, 384.0)
    );
}

#[test]
fn a_room_smaller_than_the_edges_still_leaves_a_pixel() {
    assert_eq!(fitted((480.0, 620.0), 1.0, (10.0, 10.0)), (1.0, 1.0));
}

#[test]
fn the_room_is_in_logical_units_and_survives_a_scale_of_zero() {
    assert_eq!(room_of(area(3840.0, 2160.0), 2.0), (1920.0, 1080.0));
    assert_eq!(room_of(area(1920.0, 1080.0), 0.0), (1920.0, 1080.0));
    let offset = crate::placing::Area {
        left: 100.0,
        top: 50.0,
        right: 1100.0,
        bottom: 850.0,
    };
    assert_eq!(room_of(offset, 1.25), (800.0, 640.0));
}

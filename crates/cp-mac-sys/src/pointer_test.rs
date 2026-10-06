use super::*;
use objc2_foundation::NSSize;

#[test]
fn appkit_counts_from_the_bottom_and_the_panel_from_the_top() {
    let visible = NSRect::new(NSPoint::new(0.0, 80.0), NSSize::new(1440.0, 795.0));
    let spot = flipped(NSPoint::new(700.0, 600.0), visible, 900.0);
    assert_eq!(
        spot,
        Spot {
            x: 700.0,
            y: 300.0,
            left: 0.0,
            top: 25.0,
            right: 1440.0,
            bottom: 820.0,
        }
    );
}

#[test]
fn a_screen_above_the_main_one_has_negative_tops() {
    let visible = NSRect::new(NSPoint::new(0.0, 900.0), NSSize::new(1920.0, 1055.0));
    let spot = flipped(NSPoint::new(100.0, 1500.0), visible, 900.0);
    assert_eq!(spot.y, -600.0);
    assert_eq!(spot.top, -1055.0);
    assert_eq!(spot.bottom, 0.0);
}

#[test]
fn off_the_main_thread_there_is_no_spot() {
    if MainThreadMarker::new().is_none() {
        assert_eq!(spot(), None);
    }
}

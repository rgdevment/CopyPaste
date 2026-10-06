use super::*;

const SCREEN: Area = Area {
    left: 0.0,
    top: 0.0,
    right: 1920.0,
    bottom: 1040.0,
};
const PANEL: (f64, f64) = (480.0, 620.0);

#[test]
fn the_panel_opens_to_the_right_of_the_pointer_and_centred_on_it() {
    assert_eq!(beside((500.0, 500.0), SCREEN, PANEL, 1.0), (512.0, 190.0));
}

#[test]
fn near_the_right_edge_it_opens_to_the_left_of_the_pointer() {
    assert_eq!(beside((1800.0, 500.0), SCREEN, PANEL, 1.0), (1308.0, 190.0));
}

#[test]
fn near_the_top_or_the_bottom_it_stays_inside_the_work_area() {
    assert_eq!(beside((500.0, 20.0), SCREEN, PANEL, 1.0).1, 8.0);
    assert_eq!(
        beside((500.0, 1030.0), SCREEN, PANEL, 1.0).1,
        1040.0 - 8.0 - 620.0
    );
}

#[test]
fn a_screen_narrower_than_both_sides_pins_it_to_the_right_edge() {
    let narrow = Area {
        right: 700.0,
        ..SCREEN
    };
    let (x, _) = beside((350.0, 500.0), narrow, PANEL, 1.0);
    assert_eq!(x, 700.0 - 480.0 - 12.0);
}

#[test]
fn a_second_monitor_to_the_left_keeps_its_negative_coordinates() {
    let left = Area {
        left: -1920.0,
        top: 0.0,
        right: 0.0,
        bottom: 1040.0,
    };
    assert_eq!(beside((-1500.0, 500.0), left, PANEL, 1.0), (-1488.0, 190.0));
}

#[test]
fn the_gaps_grow_with_the_scale_of_the_screen() {
    assert_eq!(
        beside((500.0, 500.0), SCREEN, (960.0, 800.0), 2.0),
        (524.0, 100.0)
    );
}

#[test]
fn a_panel_taller_than_the_screen_starts_at_its_top() {
    let short = Area {
        bottom: 400.0,
        ..SCREEN
    };
    assert_eq!(beside((500.0, 200.0), short, PANEL, 1.0).1, 0.0);
}

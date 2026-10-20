use super::*;

#[test]
fn the_pointer_sits_on_a_work_area_that_has_room() {
    let Some(spot) = spot() else {
        return;
    };
    assert!(spot.right > spot.left, "{spot:?}");
    assert!(spot.bottom > spot.top, "{spot:?}");
    assert!(spot.scale >= 1.0, "{spot:?}");
}

#[test]
fn a_window_knows_the_work_area_of_the_screen_it_is_on() {
    let edit = crate::window::EditWindow::open("where am i").expect("a window to ask about");
    let Some(spot) = spot_of(edit.window().0 as isize) else {
        return;
    };
    assert!(spot.right > spot.left && spot.bottom > spot.top);
    assert!(spot.scale >= 1.0);
}

#[test]
fn a_window_that_is_not_there_still_falls_on_some_screen() {
    let nearest = spot_of(0);
    assert_eq!(nearest.is_some(), spot().is_some());
}

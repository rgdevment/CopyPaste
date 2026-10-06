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

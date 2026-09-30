use super::*;

#[test]
fn a_panel_that_never_owns_the_dock_can_say_so() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    assert!(as_accessory());
    assert_eq!(is_accessory(), Some(true));
}

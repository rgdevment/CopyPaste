use super::*;

#[test]
fn a_panel_that_never_owns_the_dock_can_say_so() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    assert!(as_accessory());
    assert_eq!(is_accessory(), Some(true));
}

#[test]
fn off_the_main_thread_nobody_claims_to_be_in_front() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert!(
        !is_ours_up_front(),
        "asking AppKit from another thread has to answer no, not reach for it"
    );
}

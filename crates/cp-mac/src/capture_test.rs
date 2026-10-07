use super::*;

#[test]
fn the_patience_sits_between_the_two_measured_worlds() {
    let delivered = std::time::Duration::from_millis(9);
    let promised = std::time::Duration::from_secs(60);
    assert!(delivered < PATIENCE, "Finder delivers 16 types in 9 ms");
    assert!(PATIENCE < std::time::Duration::from_secs(1));
    assert!(PATIENCE < promised, "a promise takes 60 s to give up");
}

#[test]
fn a_capture_that_does_not_finish_in_time_is_abandoned() {
    let seen = reading::anything_within(std::time::Duration::from_millis(20), || {
        std::thread::sleep(std::time::Duration::from_secs(60));
        Captured::Nothing
    });
    assert_eq!(seen, None, "the thread is abandoned and not waited for");
}

#[test]
fn only_a_denied_or_asking_pasteboard_is_unreadable() {
    assert_eq!(unreadable_under(None), None);
    assert_eq!(
        unreadable_under(Some(Access::Default)),
        Some(Unreadable::Asks),
        "before its first alert macOS asks on programmatic access"
    );
    assert_eq!(unreadable_under(Some(Access::AlwaysAllow)), None);
    assert_eq!(unreadable_under(Some(Access::Ask)), Some(Unreadable::Asks));
    assert_eq!(
        unreadable_under(Some(Access::AlwaysDeny)),
        Some(Unreadable::Denied)
    );
}

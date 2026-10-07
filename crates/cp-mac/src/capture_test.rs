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
        None,
        "the default is what nearly everyone has, and no reason to warn"
    );
    assert_eq!(unreadable_under(Some(Access::AlwaysAllow)), None);
    assert_eq!(unreadable_under(Some(Access::Ask)), Some(Unreadable::Asks));
    assert_eq!(
        unreadable_under(Some(Access::AlwaysDeny)),
        Some(Unreadable::Denied)
    );
}

#[test]
fn asking_for_the_reading_permission_never_fails() {
    let _ = unreadable();
}

#[test]
fn a_web_page_keeps_its_text_and_html_but_never_the_whole_archive() {
    let offered = [
        "com.apple.webarchive",
        "Apple Web Archive pasteboard type",
        "public.html",
        "public.utf8-plain-text",
    ];
    let family = CATALOG.classify(&offered);
    assert_eq!(family, Some(Family::Text));
    let read: Vec<&str> = offered
        .iter()
        .map(|id| CATALOG.canonical(id))
        .filter(|id| reads(family, id, &offered))
        .collect();
    assert_eq!(read, ["public.html", "public.utf8-plain-text"]);
}

#[test]
fn the_costlier_twin_of_an_image_is_only_announced() {
    let offered = ["public.png", "public.tiff"];
    let family = CATALOG.classify(&offered);
    assert!(reads(family, "public.png", &offered));
    assert!(!reads(family, "public.tiff", &offered));
}

use super::*;

#[test]
fn a_fresh_show_aims_at_whoever_is_in_front_even_nobody() {
    assert_eq!(aimed_at(42, 7, false), 42);
    assert_eq!(
        aimed_at(0, 7, false),
        0,
        "with nobody in front the paste reaches nobody, never an app from before"
    );
}

#[test]
fn showing_again_while_up_front_keeps_the_app_it_came_from() {
    assert_eq!(aimed_at(0, 7, true), 7);
    assert_eq!(aimed_at(42, 7, true), 42);
}

#[test]
fn leaving_always_forgets_where_it_would_have_pasted() {
    for leaving in [Leaving::Back, Leaving::Away] {
        let ahead = AtomicIsize::new(7);
        let _ = handed_back(&ahead, leaving, || true);
        assert_eq!(ahead.load(Ordering::Relaxed), 0, "{leaving:?}");
    }
}

#[test]
fn going_back_returns_the_focus_only_when_it_was_taken() {
    assert_eq!(
        handed_back(&AtomicIsize::new(7), Leaving::Back, || true),
        Some(7)
    );
    assert_eq!(
        handed_back(&AtomicIsize::new(7), Leaving::Back, || false),
        None,
        "if the app never left the front there is nothing to give back"
    );
    assert_eq!(
        handed_back(&AtomicIsize::new(0), Leaving::Back, || true),
        None
    );
}

#[test]
fn going_away_leaves_the_focus_to_whatever_opens_next() {
    assert_eq!(
        handed_back(&AtomicIsize::new(7), Leaving::Away, || true),
        None
    );
}

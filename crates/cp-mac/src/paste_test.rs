use super::*;

fn ready(can_post: bool, accessibility: bool, secure_input: bool) -> Readiness {
    Readiness {
        can_post,
        accessibility,
        secure_input,
    }
}

#[test]
fn a_keystroke_is_the_way_when_nothing_is_in_the_way() {
    assert_eq!(route_of(&ready(true, true, false)), Some(Route::Keystroke));
}

#[test]
fn without_posting_events_the_menu_still_serves() {
    assert_eq!(route_of(&ready(false, true, false)), Some(Route::Menu));
}

#[test]
fn protected_input_leaves_only_the_menu() {
    assert_eq!(
        route_of(&ready(true, true, true)),
        Some(Route::Menu),
        "a posted keystroke is dropped on the floor while the system protects what is typed"
    );
}

#[test]
fn protected_input_without_the_menu_is_no_way_at_all() {
    assert_eq!(
        route_of(&ready(true, false, true)),
        None,
        "claiming the keystroke here would report a paste that never arrived"
    );
}

#[test]
fn neither_way_is_no_way() {
    assert_eq!(route_of(&ready(false, false, false)), None);
}

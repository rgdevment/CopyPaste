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
fn a_session_wide_flag_does_not_decide_the_route_in_advance() {
    assert_eq!(
        route_of(&ready(true, false, true)),
        Some(Route::Keystroke),
        "any app holding secure input would otherwise stop every paste before it is tried"
    );
}

#[test]
fn protected_input_sends_us_round_by_the_menu() {
    assert_eq!(
        around_protected_input(Route::Keystroke, true),
        Some(Route::Menu),
        "a posted keystroke is dropped while the system protects what is typed, a menu press is not"
    );
    assert_eq!(
        around_protected_input(Route::Menu, true),
        Some(Route::Menu),
        "the menu was already the way in"
    );
}

#[test]
fn protected_input_without_the_menu_is_no_way_at_all() {
    assert_eq!(
        around_protected_input(Route::Keystroke, false),
        None,
        "claiming the keystroke here would report a paste that never arrived"
    );
}

#[test]
fn neither_way_is_no_way() {
    assert_eq!(route_of(&ready(false, false, false)), None);
}

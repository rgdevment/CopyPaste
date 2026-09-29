use super::Readiness;

fn with(can_post: bool, accessibility: bool, secure_input: bool) -> Readiness {
    Readiness {
        can_post,
        accessibility,
        secure_input,
    }
}

#[test]
fn pasting_needs_one_of_the_two_permissions() {
    assert!(with(true, false, false).can_paste());
    assert!(
        with(true, false, true).can_paste(),
        "secure input does not get the final word"
    );
    assert!(with(false, true, false).can_paste());
    assert!(!with(false, false, false).can_paste());
}

#[test]
fn the_menu_fallback_needs_its_own_permission() {
    assert!(!with(true, false, false).can_use_menu_fallback());
    assert!(with(true, true, false).can_use_menu_fallback());
    assert!(with(false, true, false).can_use_menu_fallback());
}
